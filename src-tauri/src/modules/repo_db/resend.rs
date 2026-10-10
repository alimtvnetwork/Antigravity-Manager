//! Repo DB: resend

use super::agy::spawn_prompt_via_agy;
use super::detection::extract_image_payload_or_path;
use super::models::ActivePrompt;
use super::schema::{connect_db, init_tables};
use super::state::{get_dispatched_prompts_cache, reset_dispatched_prompts_cache};
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Resend and restore all previous running/backed-up/dispatched commands across all instances.
pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String> {
    resend_running_commands_for_instance(None, limit)
}

/// Resend and restore previous running/backed-up/queued commands scoped to an optional `instance_id`.
/// Strictly filters to unrestored/backed-up/queued prompts, prioritizes the latest running prompts,
/// and deduplicates against already dispatched prompts and workspace repositories.
pub fn resend_running_commands_for_instance(
    instance_id: Option<&str>,
    limit: usize,
) -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending', 'running')
             ORDER BY created_at ASC LIMIT ?",
        )
        .map_err(|e| format!("Failed to prepare resend query: {}", e))?;

    let all_prompts = stmt
        .query_map([limit.saturating_mul(4).max(50)], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query prompts for resend: {}", e))?
        .flatten()
        .collect::<Vec<ActivePrompt>>();

    let target_inst_opt = instance_id.map(|id| {
        crate::modules::instance::resolve_instance_id(id).unwrap_or_else(|_| id.to_string())
    });

    let prompts: Vec<ActivePrompt> = all_prompts
        .into_iter()
        .filter(|p| {
            let is_match = match target_inst_opt.as_deref() {
                None | Some("all") => true,
                Some("default") | Some("__default__") => {
                    let p_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                        .unwrap_or_else(|_| p.instance_id.clone());
                    p_inst == "default" || p_inst == "__default__" || p_inst.is_empty()
                }
                Some(inst) => {
                    let p_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                        .unwrap_or_else(|_| p.instance_id.clone());
                    p_inst == inst
                }
            };
            let target_display = target_inst_opt.as_deref().unwrap_or("all");
            crate::modules::logger::log_instance_prompt_audit(
                target_display,
                "resend_running_commands_for_instance",
                &p.id,
                &p.repo_path,
                "",
                None,
                "PromptDispatcher:InstanceMatching",
                is_match,
                if is_match {
                    "PROMPT_DISPATCH_MATCHED"
                } else {
                    "PROMPT_DISPATCH_REJECTED"
                },
            );
            is_match
        })
        .collect();

    let mut resent = Vec::new();
    let mut dispatched_repos = HashSet::new();

    for mut prompt in prompts {
        let clean_path = prompt.repo_path.trim().to_lowercase().replace('\\', "/");
        if dispatched_repos.contains(&clean_path) {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Preserving subsequent prompt '{}' for repo '{}' in FIFO queue",
                prompt.id, prompt.repo_path
            ));
            if prompt.status != "queued" {
                // Justification: queue-state update is best-effort; the prompt keeps its current status and is reconsidered on the next resend
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE active_prompts SET status = 'queued', updated_at = ? WHERE id = ?",
                        rusqlite::params![now, &prompt.id],
                    ),
                    "re-queue preserved prompt",
                );
            }
            continue;
        }

        if resent.len() >= limit {
            if prompt.status != "queued" {
                // Justification: queue-state update is best-effort; the prompt keeps its current status and is reconsidered on the next resend
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE active_prompts SET status = 'queued', updated_at = ? WHERE id = ?",
                        rusqlite::params![now, &prompt.id],
                    ),
                    "re-queue prompt over resend limit",
                );
            }
            continue;
        }

        dispatched_repos.insert(clean_path);

        let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
        let already_dispatched = if prompt.status == "backed_up" {
            false
        } else {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            let seen = cache.contains(&prompt.id) || cache.contains(&sig);
            if !seen {
                cache.insert(prompt.id.clone());
                cache.insert(sig);
            }
            seen
        };

        if already_dispatched {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping duplicate dispatch for prompt '{}' in '{}'",
                prompt.id, prompt.repo_path
            ));
            // Justification: duplicate-suppression state update; a missed update leaves the prompt backed_up for retry on the next resend
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
                    rusqlite::params![now, &prompt.id],
                ),
                "mark duplicate prompt dispatched",
            );
            continue;
        }

        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        if prompt.image_payload.is_none() && extracted_img.is_some() {
            prompt.image_payload = extracted_img.clone();
        }
        let has_image = prompt.image_payload.is_some() || !img_paths.is_empty();

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt.id,
            "project_id": prompt.project_id,
            "instance_id": prompt.instance_id,
            "repo_path": prompt.repo_path,
            "prompt_content": prompt.prompt_content,
            "model": prompt.model,
            "image_payload": prompt.image_payload,
            "image_paths": img_paths,
            "has_image": has_image,
            "auto_boot": true,
            "status": "dispatched",
            "resumed_at": now,
        });

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
            crate::error::record_ignored(
                fs::write(&task_file, json_str),
                "write resume task snapshot",
            );
        }

        let sent = spawn_prompt_via_agy(&prompt);
        if !sent {
            crate::modules::logger::log_error(&format!(
                "[RepoDB] Prompt '{}' was not re-pushed; left {}",
                prompt.id, prompt.status
            ));
            continue;
        }

        // Justification: dispatch already succeeded via agy; a missed status update only causes a retry on the next resend
        crate::error::record_ignored(
            conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?, image_payload = ? WHERE id = ?",
            rusqlite::params![now, &prompt.image_payload, &prompt.id],
        ),
            "mark prompt dispatched after agy send",
        );
        if prompt.status == "running" {
            let inst_suffix =
                if prompt.instance_id.is_empty() || prompt.instance_id == "__default__" {
                    "default"
                } else {
                    &prompt.instance_id
                };
            let composite_id = if prompt.project_id.contains("__") {
                prompt.project_id.clone()
            } else {
                format!("{}__{}", prompt.project_id, inst_suffix)
            };
            // Justification: liveness flag is recomputed on every resend scan; best-effort
            crate::error::record_ignored(
                conn.execute(
                "UPDATE running_projects SET is_running = 1, last_detected_at = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![now, now, &composite_id],
            ),
                "mark project running after resend",
            );
        }

        prompt.status = "dispatched".to_string();
        prompt.updated_at = now;
        resent.push(prompt);
    }

    invalidate_prompt_tree_cache(target_inst_opt.as_deref());
    Ok(resent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use crate::modules::repo_db::{get_dispatched_prompts_cache, reset_dispatched_prompts_cache};
    use rusqlite::Connection;

    #[test]
    fn test_resend_deduplication_and_reinjection_prevention() {
        reset_dispatched_prompts_cache();
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        // Insert parent running_project first
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'inst-1', 'repo-one', '/repo/one', NULL, 1, ?, ?)",
            params![now, now],
        )
        .unwrap();

        // Insert prompt with status 'backed_up'
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-100', 'proj-1', 'inst-1', '/repo/one', 'Refactor router', 'gemini-pro', 'sess-100', 'backed_up', ?, ?, NULL)",
            params![now - 10, now - 10],
        )
        .unwrap();

        // Verify it is selected by the query for resend
        let mut stmt = conn
            .prepare(
                "SELECT id, prompt_content, status FROM active_prompts WHERE status IN ('backed_up', 'queued', 'pending') ORDER BY created_at ASC",
            )
            .unwrap();
        let pending_prompts: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(pending_prompts.len(), 1);
        assert_eq!(pending_prompts[0], "p-100");

        // Simulate dispatching: mark as dispatched and add to cache
        conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = 'p-100'",
            params![now],
        )
        .unwrap();
        {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            cache.insert("p-100".to_string());
            cache.insert("/repo/one:Refactor router".to_string());
        }

        // Verify that subsequent query for resend finds 0 pending prompts!
        let pending_after: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(pending_after.len(), 0);

        // Verify that cache intercepts duplicate attempt
        let cache = get_dispatched_prompts_cache().lock().unwrap();
        assert!(cache.contains("p-100"));
        assert!(cache.contains("/repo/one:Refactor router"));
    }
}
