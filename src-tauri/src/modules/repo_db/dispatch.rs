//! Repo DB: dispatch

use super::agy::spawn_prompt_via_agy;
use super::detection::resume_task_document;
use super::models::ActivePrompt;
use super::schema::{connect_db, init_tables};
use super::state::{get_dispatched_prompts_cache, get_memory_prompts_map};
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// Directly dispatch/send backed-up prompts to the running projects without queuing
pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let target_inst =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let is_default_target = target_inst == "default" || target_inst == "__default__";

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE status = 'backed_up'
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to prepare dispatch query: {}", e))?;

    let all_backed_up = stmt
        .query_map([], |row| {
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
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect::<Vec<ActivePrompt>>();

    let prompts: Vec<ActivePrompt> = all_backed_up
        .into_iter()
        .filter(|p| {
            let prompt_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                .unwrap_or_else(|_| {
                    if p.instance_id == "__default__" || p.instance_id.is_empty() {
                        "default".to_string()
                    } else {
                        p.instance_id.clone()
                    }
                });
            let is_match = if is_default_target {
                prompt_inst == "default" || prompt_inst == "__default__" || prompt_inst.is_empty()
            } else {
                prompt_inst == target_inst
            };
            crate::modules::logger::log_instance_prompt_audit(
                &target_inst,
                "dispatch_running_prompts",
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

    let mut dispatched_count = 0;

    for prompt in prompts {
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
            // Justification: duplicate-suppression state update; a missed update leaves the prompt backed_up for retry on the next dispatch
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
                    rusqlite::params![now, &prompt.id],
                ),
                "mark duplicate prompt dispatched",
            );
            continue;
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Directly dispatching prompt '{}' to project at '{}' without queuing",
            prompt.id, prompt.repo_path
        ));

        // Keep the conversation id on the file the IDE reads. A later rewrite must not drop it.
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = resume_task_document(&prompt, "dispatched", now, &[]);
        let file_written = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };

        let sent = spawn_prompt_via_agy(&prompt);
        let same_conversation = prompt
            .session_id
            .as_deref()
            .map(|id| !id.trim().is_empty())
            .unwrap_or(false);
        if !sent && !(file_written && same_conversation) {
            crate::modules::logger::log_error(&format!(
                "[RepoDB] Prompt '{}' was not re-pushed for instance '{}'; left backed_up",
                prompt.id, instance_id
            ));
            continue;
        }

        let updated = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
            params![now, &prompt.id],
        );

        if updated.is_ok() {
            dispatched_count += 1;
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                if let Some(p) = map.get_mut(&prompt.id) {
                    p.status = "dispatched".to_string();
                    p.updated_at = now;
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Successfully dispatched {} prompts to running projects for instance '{}'",
        dispatched_count, instance_id
    ));

    invalidate_prompt_tree_cache(Some(&target_inst));
    Ok(dispatched_count)
}

/// Robust multi-format timestamp parsing handling RFC3339/ISO8601 with fractional seconds,
/// timezones, ISO8601 without timezone, space-delimited with fractional seconds, standard seconds,
/// and epoch integer timestamps.
pub fn parse_flexible_timestamp(time_str: &str) -> i64 {
    let trimmed = time_str.trim();
    if trimmed.is_empty() {
        return 0;
    }

    // Standard epoch seconds or milliseconds
    if let Ok(ts) = trimmed.parse::<i64>() {
        return if ts > 1_000_000_000_000 {
            ts / 1000
        } else {
            ts
        };
    }

    let norm_time = trimmed.replacen(' ', "T", 1);

    // 1) RFC3339 / ISO8601 with fractional seconds and timezone
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&norm_time) {
        return dt.timestamp();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return dt.timestamp();
    }

    // 2) ISO8601 without timezone (%Y-%m-%dT%H:%M:%S%.f and %Y-%m-%dT%H:%M:%S)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&norm_time, "%Y-%m-%dT%H:%M:%S%.f") {
        return dt.and_utc().timestamp();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&norm_time, "%Y-%m-%dT%H:%M:%S") {
        return dt.and_utc().timestamp();
    }

    // 3) Space-delimited with fractional seconds (%Y-%m-%d %H:%M:%S%.f)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S%.f") {
        return dt.and_utc().timestamp();
    }

    // 4) Standard seconds format (%Y-%m-%d %H:%M:%S)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return dt.and_utc().timestamp();
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::params;
    use rusqlite::Connection;

    #[test]
    fn test_prompt_backup_and_direct_dispatch_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        // Justification: test fixture on an in-memory database; the WAL pragma is a no-op here and init_tables is asserted below
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        // 1. Insert running project
        let proj_res = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'inst-alpha', 'CoreApp', '/work/core', NULL, 1, 1000, 1000)",
            [],
        );
        assert!(proj_res.is_ok());

        // 2. Backup prompt
        let prompt_id = "test-prompt-uuid";
        let prompt_insert = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
             VALUES (?, 'proj-1', 'inst-alpha', '/work/core', 'Implement neural router', 'gemini-pro', 'sess-1', 'backed_up', 1000, 1000)",
            params![prompt_id],
        );
        assert!(prompt_insert.is_ok());

        // 3. Verify status is backed_up
        let status_before: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE id = ?",
                params![prompt_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status_before, "backed_up");

        // 4. Direct dispatch (no queuing)
        let dispatch_update = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = 1010 
             WHERE instance_id = 'inst-alpha' AND status = 'backed_up'",
            [],
        );
        assert_eq!(dispatch_update.unwrap(), 1);

        // 5. Verify status is directly dispatched
        let status_after: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE id = ?",
                params![prompt_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status_after, "dispatched");
    }
}
