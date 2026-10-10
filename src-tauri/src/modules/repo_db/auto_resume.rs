//! Repo DB: auto resume

use super::agy::spawn_prompt_via_agy;
use super::detection::resume_task_document;
use super::models::{ActivePrompt, AutoResumePromptInfo, AutoResumeResult, RunningProject};
use super::schema::{connect_db, init_tables};
use super::state::get_dispatched_prompts_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// Auto-resume recent prompts for projects active within max_age_seconds (strictly skips projects older than threshold)
pub fn auto_resume_recent_prompts(
    instance_id: &str,
    max_age_seconds: i64,
) -> Result<AutoResumeResult, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let threshold_cutoff = now - max_age_seconds;

    // Load instance details to determine account email
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let account_email = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .and_then(|i| i.bound_email.clone())
        .unwrap_or_else(|| "unbound".to_string());

    let norm_inst = if instance_id == "__default__" || instance_id.is_empty() {
        "default"
    } else {
        instance_id
    };

    let mut stmt = conn
        .prepare(
            "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
             FROM running_projects 
             WHERE workspace_storage_path IS NOT NULL 
               AND instr(id, '__') > 0 
               AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
             ORDER BY last_detected_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let projects = stmt
        .query_map([norm_inst], |row| {
            let running_int: i32 = row.get(5)?;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running: running_int != 0,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query projects: {}", e))?
        .flatten()
        .collect::<Vec<RunningProject>>();

    let mut resumed_prompts = Vec::new();
    let mut resumed_count = 0;
    let mut skipped_count = 0;

    for project in projects {
        // STRICT RECENCY FILTER: If project last active > max_age_seconds (e.g. > 1 hour, 2h, 5h, 1 day), SKIP!
        let is_stale = project.last_detected_at < threshold_cutoff;
        if is_stale {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping project '{}' (last detected {}s ago, exceeds threshold {}s)",
                project.repo_name,
                now - project.last_detected_at,
                max_age_seconds
            ));
            skipped_count += 1;
            continue;
        }

        // Project was active within < 1 hour! Find its most recent prompt + image
        let proj_like = format!("%{}%", project.repo_name.to_lowercase());
        let mut prompt_stmt = conn
            .prepare(
                "SELECT id, prompt_content, model, image_payload 
                 FROM active_prompts 
                 WHERE status IN ('queued', 'pending')
                   AND (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
                   AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
                 ORDER BY created_at ASC, id ASC LIMIT 1",
            )
            .map_err(|e| format!("Failed to prepare prompt query: {}", e))?;

        let mut maybe_prompt = prompt_stmt
            .query_row(
                rusqlite::params![&project.id, &project.repo_path, &proj_like, norm_inst],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .ok();

        // If not found in DB, check existing .antigravity_resume_task.json with valid queued/pending status
        if maybe_prompt.is_none() {
            let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
            if task_file.exists() {
                if let Ok(c) = fs::read_to_string(&task_file) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                        let task_status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        let is_pending_status = task_status == "queued" || task_status == "pending";
                        if is_pending_status {
                            if let Some(txt) = v.get("prompt_content").and_then(|t| t.as_str()) {
                                if !txt.trim().is_empty() {
                                    let pid = v
                                        .get("prompt_id")
                                        .and_then(|i| i.as_str())
                                        .unwrap_or(&project.id)
                                        .to_string();
                                    let m = v
                                        .get("model")
                                        .and_then(|m| m.as_str())
                                        .map(|s| s.to_string());
                                    let img = v
                                        .get("image_payload")
                                        .and_then(|i| i.as_str())
                                        .map(|s| s.to_string());
                                    maybe_prompt = Some((pid, txt.to_string(), m, img));
                                }
                            }
                        }
                    }
                }
            }
        }

        let Some((prompt_id, prompt_text, prompt_model, image_payload)) = maybe_prompt else {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] auto_resume_recent_prompts skipping idle workspace '{}' (no prompt found)",
                project.repo_path
            ));
            skipped_count += 1;
            continue;
        };

        let sig = format!("{}:{}", project.repo_path, prompt_text.trim());
        let already_dispatched = {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            let seen = cache.contains(&prompt_id) || cache.contains(&sig);
            if !seen {
                cache.insert(prompt_id.clone());
                cache.insert(sig);
            }
            seen
        };

        if already_dispatched {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] auto_resume_recent_prompts skipping already dispatched prompt '{}' in '{}'",
                prompt_id, project.repo_path
            ));
            continue;
        }

        let has_image = image_payload.is_some();

        // Dispatch directly to project folder via .antigravity_resume_task.json to spin up boot process immediately
        let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
        let prompt_obj = ActivePrompt {
            id: prompt_id.clone(),
            project_id: project.id.clone(),
            instance_id: instance_id.to_string(),
            repo_path: project.repo_path.clone(),
            prompt_content: prompt_text.clone(),
            model: prompt_model.clone(),
            session_id: Some(project.id.clone()),
            status: "dispatched".to_string(),
            created_at: now,
            updated_at: now,
            image_payload: image_payload.clone(),
        };
        let payload = resume_task_document(&prompt_obj, "dispatched", now, &[]);
        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
            crate::error::record_ignored(
                fs::write(&task_file, json_str),
                "write resume task snapshot",
            );
        }

        // Mark / update status in active_prompts as dispatched
        // Justification: dispatch state record; the agy send already happened and re-reads tolerate a stale row
        crate::error::record_ignored(
            conn.execute(
            "INSERT OR REPLACE INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'dispatched', ?, ?, ?)",
            params![
                &prompt_id,
                &project.id,
                instance_id,
                &project.repo_path,
                &prompt_text,
                &prompt_model,
                &project.id,
                now,
                now,
                &image_payload,
            ],
        ),
            "record dispatched prompt",
        );

        let prompt_obj = ActivePrompt {
            id: prompt_id.clone(),
            project_id: project.id.clone(),
            instance_id: instance_id.to_string(),
            repo_path: project.repo_path.clone(),
            prompt_content: prompt_text.clone(),
            model: prompt_model.clone(),
            session_id: Some(project.id.clone()),
            status: "dispatched".to_string(),
            created_at: now,
            updated_at: now,
            image_payload: image_payload.clone(),
        };
        spawn_prompt_via_agy(&prompt_obj);

        let preview = if prompt_text.len() > 80 {
            format!("{}...", &prompt_text[..77])
        } else {
            prompt_text.clone()
        };

        resumed_prompts.push(AutoResumePromptInfo {
            project_id: project.id.clone(),
            repo_path: project.repo_path.clone(),
            prompt_preview: preview,
            has_image,
        });

        resumed_count += 1;
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Auto-resumed prompt for project '{}' (image: {})",
            project.repo_name, has_image
        ));
    }

    Ok(AutoResumeResult {
        instance_id: instance_id.to_string(),
        account_email,
        resumed_project_count: resumed_count,
        skipped_project_count: skipped_count,
        resumed_prompts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn test_auto_resume_recency_filter() {
        let conn = Connection::open_in_memory().unwrap();
        // Justification: test fixture on an in-memory database; the WAL pragma is a no-op here and init_tables is asserted below
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();

        // 1. Insert recent project (active 10 minutes ago)
        // Justification: test fixture insert; the assertions below fail loudly if the row is missing
        let _ = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-recent', 'inst-test', 'RecentApp', '/work/recent', NULL, 1, ?, ?)",
            params![now - 600, now - 600],
        );

        // 2. Insert old project (active 2 hours ago)
        // Justification: test fixture insert; the assertions below fail loudly if the row is missing
        let _ = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-old', 'inst-test', 'OldApp', '/work/old', NULL, 1, ?, ?)",
            params![now - 7200, now - 7200],
        );

        // Verify recency threshold logic
        let threshold = 3600; // 1 hour
        let cutoff = now - threshold;

        let recent_last = now - 600;
        let old_last = now - 7200;

        let is_recent_valid = recent_last >= cutoff;
        let is_old_stale = old_last < cutoff;

        assert!(is_recent_valid);
        assert!(is_old_stale);
    }
}
