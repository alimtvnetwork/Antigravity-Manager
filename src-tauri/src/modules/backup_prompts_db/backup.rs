use crate::modules::agy_cleaner;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::*;

/// Backward compatible wrapper with instance support
pub fn backup_active_running_prompts(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    backup_active_running_prompts_for_instance(instance_id, custom_file)
}

/// Backup all currently active and queued running prompts into the split SQLite database scoped to an instance
pub fn backup_active_running_prompts_for_instance(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let target_inst = instance_id.unwrap_or("default");
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        auto_cleanup_expired(custom_file, 86400),
        "auto_cleanup_expired",
    );

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = repo_db::discover_running_prompts_from_antigravity(target_inst);
    let conversations = agy_cleaner::scan_conversations(100);
    let now = Utc::now().timestamp();
    let freshness_cutoff = now - 7200;

    // Consolidate candidate prompts to back up: prioritize live discovered prompts
    let mut candidate_prompts = Vec::new();
    for p in running_prompts.clone() {
        candidate_prompts.push(p);
    }
    for p in all_prompts {
        let is_inst_match = p.instance_id.is_empty()
            || p.instance_id == target_inst
            || (target_inst == "default"
                && (p.instance_id == "default" || p.instance_id.is_empty()));
        if is_inst_match
            && (p.status == "running" || p.status == "queued" || p.status == "backed_up")
            && p.updated_at >= freshness_cutoff
            && !candidate_prompts.iter().any(|c| {
                c.id == p.id
                    || (c.repo_path.to_lowercase().replace('\\', "/")
                        == p.repo_path.to_lowercase().replace('\\', "/")
                        && c.prompt_content.trim() == p.prompt_content.trim())
            })
        {
            candidate_prompts.push(p);
        }
    }

    let conn = connect_backup_db(custom_file)?;
    let batch_id = format!("batch_{}", Uuid::new_v4().simple());

    let db_path_str = get_backup_prompts_db_path(custom_file)?
        .to_string_lossy()
        .to_string();

    let batch_info = BackupBatchInfo {
        id: batch_id.clone(),
        created_at: now,
        prompts_count: candidate_prompts.len(),
        file_path: db_path_str.clone(),
        retention_days: 1,
        is_fully_restored: false,
    };

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            batch_info.id,
            batch_info.created_at,
            batch_info.prompts_count,
            batch_info.file_path,
            batch_info.retention_days,
            batch_info.is_fully_restored,
        ],
    )
    .map_err(|e| format!("Failed to record backup batch: {}", e))?;

    let mut records = Vec::new();
    for (idx, p) in candidate_prompts.iter().enumerate() {
        let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || uris_norm.contains(&p.project_id.to_lowercase())
        });

        let conv_id = matched_conv
            .map(|c| c.conversation_id.clone())
            .or_else(|| p.session_id.clone())
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv.and_then(|c| {
            if c.title.trim().is_empty() {
                None
            } else {
                Some(c.title.clone())
            }
        });

        let proj_name = resolve_friendly_project_name(&p.repo_path, &p.project_id);

        let record = PromptBackupRecord {
            id: format!("rec_{}", Uuid::new_v4().simple()),
            backup_batch_id: batch_id.clone(),
            prompt_id: p.id.clone(),
            project_name: proj_name,
            project_path: p.repo_path.clone(),
            project_id: p.project_id.clone(),
            conversation_id: conv_id,
            conversation_name: conv_name,
            sequence_id: (idx + 1) as i64,
            prompt_text: p.prompt_content.clone(),
            has_images: p.image_payload.is_some(),
            images_payload: p.image_payload.clone(),
            status: p.status.clone(),
            created_at: now,
            is_restored: false,
            restored_at: None,
            instance_id: Some(target_inst.to_string()),
        };

        // Deduplicate records to prevent duplicate reinjections into prompt_backups
        let is_live_prompt = running_prompts.iter().any(|rp| rp.id == record.prompt_id);
        let existing_unrestored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0 LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_unrestored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0 WHERE id = ?4",
                params![record.backup_batch_id, now, target_inst, existing_rec_id],
            ), "db execute");
            records.push(record);
            continue;
        }

        // If previously restored for this instance, reactivate for this new switch batch
        let existing_restored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND (instance_id = ?4 OR instance_id IS NULL OR instance_id = '') LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text, target_inst],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_restored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0, restored_at = NULL WHERE id = ?4",
                params![record.backup_batch_id, now, target_inst, existing_rec_id],
            ), "db execute");
            records.push(record);
            continue;
        }

        conn.execute(
            "INSERT INTO prompt_backups (
                id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.id,
                record.backup_batch_id,
                record.prompt_id,
                record.project_name,
                record.project_path,
                record.project_id,
                record.conversation_id,
                record.conversation_name,
                record.sequence_id,
                record.prompt_text,
                record.has_images,
                record.images_payload,
                record.status,
                record.created_at,
                record.is_restored,
                record.restored_at,
                record.instance_id.as_deref().unwrap_or(target_inst),
            ],
        )
        .map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;

        records.push(record);
    }

    Ok((batch_info, records))
}
