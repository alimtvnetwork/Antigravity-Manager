//! Repo DB: prompts crud

use super::models::ActivePrompt;
use super::project_queries::list_all_prompts;
use super::schema::connect_db;
use chrono::Utc;
use rusqlite::params;
use std::path::Path;
use std::path::PathBuf;

/// Insert active prompt into database
pub fn insert_active_prompt(prompt: &ActivePrompt) -> Result<(), String> {
    save_or_requeue_prompt(prompt)
}

/// List active prompts from database
pub fn list_active_prompts() -> Result<Vec<ActivePrompt>, String> {
    list_all_prompts()
}

/// Resolve instance id from project id or fallback, inspecting composite project IDs e.g. __default-copy-5597
pub fn resolve_instance_id_from_project_or_fallback(
    project_id: &str,
    instance_id_opt: Option<&str>,
) -> String {
    if let Some(inst) = instance_id_opt {
        let trimmed = inst.trim();
        if !trimmed.is_empty() && trimmed != "default" && trimmed != "__default__" {
            return crate::modules::instance::resolve_instance_id(trimmed)
                .unwrap_or_else(|_| trimmed.to_string());
        }
    }
    if let Some(suffix) = project_id.split("__").nth(1) {
        let trimmed_suffix = suffix.trim();
        if !trimmed_suffix.is_empty() {
            return crate::modules::instance::resolve_instance_id(trimmed_suffix)
                .unwrap_or_else(|_| trimmed_suffix.to_string());
        }
    }
    if let Some(inst) = instance_id_opt {
        let trimmed = inst.trim();
        if !trimmed.is_empty() {
            return crate::modules::instance::resolve_instance_id(trimmed)
                .unwrap_or_else(|_| trimmed.to_string());
        }
    }
    crate::modules::instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
}

/// Save or re-queue an active prompt into active_prompts and running_projects
pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {
    let mut conn = connect_db()?;
    let now = Utc::now().timestamp();
    let clean_repo_name = Path::new(&prompt.repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| prompt.project_id.clone());

    let canonical_inst =
        resolve_instance_id_from_project_or_fallback(&prompt.project_id, Some(&prompt.instance_id));
    let base_proj_id = prompt
        .project_id
        .split("__")
        .next()
        .unwrap_or(&prompt.project_id);
    let composite_proj_id = format!("{}__{}", base_proj_id, canonical_inst);
    let is_running_int = if prompt.status == "running" { 1 } else { 0 };

    let ws_path = conn
        .query_row(
            "SELECT workspace_storage_path FROM running_projects WHERE id = ?1",
            params![&composite_proj_id],
            |r| r.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .filter(|p| !p.trim().is_empty())
        .or_else(|| {
            let inst_data_dir = if canonical_inst == "default" {
                crate::modules::instance::get_default_antigravity_data_dir()
            } else if let Ok(reg) = crate::modules::instance::load_registry() {
                reg.instances
                    .iter()
                    .find(|i| i.id == canonical_inst || i.name == canonical_inst)
                    .map(|i| PathBuf::from(&i.data_dir))
                    .unwrap_or_else(|| PathBuf::from(&canonical_inst))
            } else {
                PathBuf::from(&canonical_inst)
            };
            let ws_folder = inst_data_dir
                .join("User")
                .join("workspaceStorage")
                .join(base_proj_id);
            Some(ws_folder.to_string_lossy().to_string())
        })
        .unwrap_or_default();

    // Imp-76: Transactional Rollback Protection
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to begin transaction for active prompt: {e}"))?;

    if !ws_path.trim().is_empty() {
        tx.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                workspace_storage_path = CASE
                    WHEN excluded.workspace_storage_path IS NOT NULL AND trim(excluded.workspace_storage_path) != ''
                    THEN excluded.workspace_storage_path
                    ELSE running_projects.workspace_storage_path
                END,
                is_running = excluded.is_running,
                updated_at = excluded.updated_at",
            params![
                &composite_proj_id,
                &canonical_inst,
                &clean_repo_name,
                &prompt.repo_path,
                &ws_path,
                is_running_int,
                now,
                now
            ],
        )
        .map_err(|e| format!("Failed to upsert running project: {e}"))?;
    }

    // Live-status dedup invariant: never clobber an actively running or in-flight prompt with backed_up
    let live_status: Option<String> = match tx.query_row(
        "SELECT status FROM active_prompts WHERE id = ?1",
        params![&prompt.id],
        |r| r.get(0),
    ) {
        Ok(status) => Some(status),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(err) => {
            tracing::warn!(
                "[RepoDB] Failed to query prompt status by id '{}': {err}",
                prompt.id
            );
            None
        }
    }
    .or_else(|| {
        let norm_path_slash = prompt.repo_path.trim().replace('\\', "/");
        let norm_path_bslash = prompt.repo_path.trim().replace('/', "\\");
        match tx.query_row(
            "SELECT status FROM active_prompts 
             WHERE (repo_path = ?1 OR repo_path = ?2 OR repo_path = ?3) 
               AND prompt_content = ?4 
               AND status IN ('running', 'in_flight', 'dispatched') 
             LIMIT 1",
            params![
                &prompt.repo_path,
                &norm_path_slash,
                &norm_path_bslash,
                &prompt.prompt_content
            ],
            |r| r.get(0),
        ) {
            Ok(status) => Some(status),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(err) => {
                tracing::warn!("[RepoDB] Failed to query active prompt status by content: {err}");
                None
            }
        }
    });

    if matches!(
        live_status.as_deref(),
        Some("running") | Some("in_flight") | Some("dispatched")
    ) && prompt.status != "completed"
        && prompt.status != "running"
        && prompt.status != "queued"
    {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Preserving live prompt '{}' (status '{}') - skipping overwrite with incoming '{}'",
            prompt.id,
            live_status.as_deref().unwrap_or("unknown"),
            prompt.status
        ));
        tx.commit()
            .map_err(|e| format!("Failed to commit active prompt transaction: {e}"))?;
        return Ok(());
    }

    tx.execute(
        "INSERT OR REPLACE INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &prompt.id,
            &composite_proj_id,
            &canonical_inst,
            &prompt.repo_path,
            &prompt.prompt_content,
            &prompt.model,
            &prompt.session_id,
            &prompt.status,
            prompt.created_at,
            now,
            &prompt.image_payload,
        ],
    ).map_err(|e| format!("Failed to insert active prompt: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit active prompt transaction: {e}"))?;

    Ok(())
}

/// Imp-47: Rapidly update active prompt status and timestamp using cached prepared statement
pub fn update_prompt_status(prompt_id: &str, new_status: &str) -> Result<bool, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut stmt = conn
        .prepare_cached("UPDATE active_prompts SET status = ?1, updated_at = ?2 WHERE id = ?3")
        .map_err(|e| format!("Failed to prepare update_prompt_status statement: {e}"))?;

    let rows_affected = stmt
        .execute(params![new_status, now, prompt_id])
        .map_err(|e| format!("Failed to execute update_prompt_status: {e}"))?;

    Ok(rows_affected > 0)
}
