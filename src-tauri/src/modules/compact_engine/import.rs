use super::db;
use super::types::*;
use super::workdir::{clean_path_str, from_relative_work_path, WorkDirResolver};
use crate::error::AppError;
use crate::modules::json_envelope::extract_payload;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Ingests a compact snapshot (.json envelope or .db SQLite) and restores its entities
/// into local SQLite databases (`repo_prompts.db` and `backup-prompts.db`).
pub fn import_compact(options: &ImportOptions) -> Result<ImportResult, AppError> {
    let start_instant = std::time::Instant::now();
    let file_path = PathBuf::from(options.file.trim());

    if !file_path.exists() {
        return Err(AppError::Config(format!(
            "Import candidate file not found: {:?}",
            file_path
        )));
    }

    let target_work_dir = WorkDirResolver::resolve_work_dir(options.work_dir.as_deref())?;
    let target_work_clean = clean_path_str(&target_work_dir);

    // 1. Unpack snapshot data
    let is_sqlite = file_path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
        .unwrap_or(false);

    let data = if is_sqlite {
        db::read_compact_db(&file_path)?
    } else {
        let raw_content = fs::read_to_string(&file_path).map_err(|e| {
            AppError::Config(format!("Failed to read compact JSON file {:?}: {e}", file_path))
        })?;
        let (extracted_data, _attrs) = extract_payload::<CompactExportData>(&raw_content)
            .map_err(|e| AppError::Config(format!("Failed to parse compact JSON envelope: {e}")))?;
        extracted_data
    };

    let mut pinned_restored = 0;
    let mut conversations_restored = 0;
    let mut running_restored = 0;
    let mut history_restored = 0;
    let mut project_key_to_abs_path = HashMap::<String, (String, String)>::new(); // key -> (repo_name, abs_path)

    let now_epoch = Utc::now().timestamp();

    // 2. Ingest into `repo_prompts.db`
    if let Ok(mut repo_conn) = crate::modules::repo_db::schema::connect_db() {
        if let Ok(tx) = repo_conn.transaction() {
            // A. Pinned Projects
            for p in &data.pinned_projects {
                let target_abs = from_relative_work_path(&p.relative_path, &target_work_dir);
                let abs_str = clean_path_str(&target_abs);
                let proj_id = format!("proj-{}", p.project_key.to_lowercase());

                project_key_to_abs_path.insert(
                    p.project_key.clone(),
                    (p.repo_name.clone(), abs_str.clone()),
                );

                // Upsert into agm_project_sequences
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO agm_project_sequences (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'default', ?6)",
                    params![
                        p.project_key,
                        p.display_order as i64,
                        proj_id,
                        p.repo_name,
                        abs_str,
                        now_epoch,
                    ],
                );

                // Upsert into running_projects
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO running_projects (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                     VALUES (?1, 'default', ?2, ?3, '', 1, ?4, ?4)",
                    params![
                        proj_id,
                        p.repo_name,
                        abs_str,
                        now_epoch,
                    ],
                );

                pinned_restored += 1;
            }

            // B. Conversations
            for c in &data.conversations {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
                     VALUES (?1, ?2, ?3, ?4, 'default', ?5)",
                    params![
                        c.conversation_id,
                        c.seq_id,
                        c.project_key,
                        c.title,
                        now_epoch,
                    ],
                );
                conversations_restored += 1;
            }

            // C. Running Prompts
            for r in &data.running_prompts {
                let (repo_name, repo_path) = project_key_to_abs_path
                    .get(&r.project_key)
                    .cloned()
                    .unwrap_or_else(|| {
                        (
                            "Unknown-Project".to_string(),
                            target_work_clean.clone(),
                        )
                    });
                let proj_id = format!("proj-{}", r.project_key.to_lowercase());

                let _ = tx.execute(
                    "INSERT OR REPLACE INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?1, ?2, 'default', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        r.prompt_id,
                        proj_id,
                        repo_path,
                        r.prompt_content,
                        r.model,
                        r.conversation_id,
                        r.status,
                        now_epoch,
                        now_epoch,
                        r.image_payload,
                    ],
                );
                running_restored += 1;
                let _ = repo_name;
            }

            // Justification: committing transactions to repo_prompts.db during import; failure returns early
            if let Err(e) = tx.commit() {
                return Err(AppError::Config(format!("Failed to commit import transaction to repo_prompts.db: {e}")));
            }
        }
    }

    // 3. Ingest into `backup-prompts.db`
    if let Ok(mut backup_conn) = crate::modules::backup_prompts_db::connect_backup_db(None) {
        if let Ok(tx) = backup_conn.transaction() {
            let batch_id = format!("import-{}", Utc::now().format("%Y%m%d%H%M%S"));
            let total_count = data.running_prompts.len() + data.prompt_history.len();

            let _ = tx.execute(
                "INSERT OR REPLACE INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
                 VALUES (?1, ?2, ?3, ?4, 30, 0)",
                params![
                    batch_id,
                    now_epoch,
                    total_count as i64,
                    clean_path_str(&file_path),
                ],
            );

            // Insert running prompts
            for r in &data.running_prompts {
                let (repo_name, repo_path) = project_key_to_abs_path
                    .get(&r.project_key)
                    .cloned()
                    .unwrap_or_else(|| {
                        (
                            "Unknown-Project".to_string(),
                            target_work_clean.clone(),
                        )
                    });
                let proj_id = format!("proj-{}", r.project_key.to_lowercase());

                let _ = tx.execute(
                    "INSERT OR REPLACE INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, conversation_name, sequence_id, prompt_text, has_images, images_payload, status, created_at, is_restored, restored_at, instance_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?11, ?12, ?13, ?14, NULL, 'default')",
                    params![
                        format!("imp-{}", r.prompt_id),
                        batch_id,
                        r.prompt_id,
                        repo_name,
                        repo_path,
                        proj_id,
                        r.conversation_id,
                        r.conversation_id,
                        r.prompt_content,
                        if r.image_payload.is_some() { 1 } else { 0 },
                        r.image_payload,
                        r.status,
                        now_epoch,
                        if r.is_restored { 1 } else { 0 },
                    ],
                );
            }

            // Insert prompt history
            for h in &data.prompt_history {
                let (repo_name, repo_path) = project_key_to_abs_path
                    .get(&h.project_key)
                    .cloned()
                    .unwrap_or_else(|| {
                        (
                            "Unknown-Project".to_string(),
                            target_work_clean.clone(),
                        )
                    });
                let proj_id = format!("proj-{}", h.project_key.to_lowercase());

                let _ = tx.execute(
                    "INSERT OR REPLACE INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, conversation_name, sequence_id, prompt_text, has_images, images_payload, status, created_at, is_restored, restored_at, instance_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, 0, NULL, 'completed', ?10, 1, ?10, 'default')",
                    params![
                        format!("hist-{}", h.history_id),
                        batch_id,
                        h.history_id,
                        repo_name,
                        repo_path,
                        proj_id,
                        h.conversation_id,
                        h.conversation_id,
                        h.prompt_content,
                        now_epoch,
                    ],
                );
                history_restored += 1;
            }

            // Justification: committing transactions to backup-prompts.db during import; failure returns early
            if let Err(e) = tx.commit() {
                return Err(AppError::Config(format!("Failed to commit import transaction to backup-prompts.db: {e}")));
            }
        }
    }

    let duration_ms = start_instant.elapsed().as_millis() as u64;

    Ok(ImportResult {
        source_path: clean_path_str(&file_path),
        target_work_dir: target_work_clean,
        pinned_restored,
        conversations_restored,
        running_restored,
        history_restored,
        duration_ms,
    })
}
