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

/// Save or re-queue an active prompt into active_prompts and running_projects
pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let clean_repo_name = Path::new(&prompt.repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| prompt.project_id.clone());

    let canonical_inst = crate::modules::instance::resolve_instance_id(&prompt.instance_id)
        .unwrap_or_else(|_| {
            if prompt.instance_id == "__default__" || prompt.instance_id.trim().is_empty() {
                "default".to_string()
            } else {
                prompt.instance_id.clone()
            }
        });
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

    if !ws_path.trim().is_empty() {
        // Justification: project-row scaffolding is auxiliary; the prompt row insert below is the authoritative op
        crate::error::record_ignored(
            conn.execute(
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
        ),
            "upsert running project for prompt",
        );
    }

    conn.execute(
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

    Ok(())
}
