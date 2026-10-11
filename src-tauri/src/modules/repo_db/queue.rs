//! Repo DB: queue

use super::agy::{enqueue_prompt_for_instance, spawn_prompt_via_agy};
use super::backup::backup_running_prompts;
use super::detection::{extract_image_payload_or_path, resume_task_document};
use super::liveness::is_prompt_running_for_project;
use super::models::ActivePrompt;
use super::schema::{connect_db, init_tables};
use super::state::get_memory_prompts_map;
use super::tree::invalidate_prompt_tree_cache;
use crate::error::AppError;
use chrono::Utc;
use rusqlite::OptionalExtension;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Count total enqueued/backed_up/pending prompts, optionally filtered by target_instance.
pub fn count_enqueued_prompts(target_instance: Option<&str>) -> usize {
    let Ok(conn) = connect_db() else {
        return 0;
    };
    if let Some(target) = target_instance {
        let norm_target = crate::modules::instance::resolve_instance_id(target)
            .unwrap_or_else(|_| target.to_string());
        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts 
                 WHERE status IN ('backed_up', 'queued', 'pending')
                   AND (?1 = 'all' OR instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))",
                rusqlite::params![norm_target],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count
    } else {
        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status IN ('backed_up', 'queued', 'pending')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count
    }
}

/// Bookkeep and check enqueued/backed_up prompts across projects.
/// If a project has no prompt running (idle verified), automatically pushes
/// the first enqueued prompt (FIFO) and logs the action to Audit Trail.
pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let run_id = format!("sched-{}", Utc::now().format("%Y%m%d-%H%M%S"));

    // 1. Gather candidate projects that have enqueued prompts ('backed_up', 'queued', 'pending')
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT project_id, instance_id, repo_path FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending')
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to query candidate enqueued projects: {}", e))?;

    let candidate_projects = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect::<Vec<_>>();

    if candidate_projects.is_empty() {
        return Ok(0);
    }

    let mut dispatched_count = 0;

    for (project_id, inst_id, repo_path) in candidate_projects {
        if let Some(target) = target_instance {
            let norm_target = crate::modules::instance::resolve_instance_id(target)
                .unwrap_or_else(|_| target.to_string());
            let norm_inst = crate::modules::instance::resolve_instance_id(&inst_id)
                .unwrap_or_else(|_| inst_id.clone());
            let is_target_default = norm_target == "default" || norm_target == "__default__";
            let is_inst_default =
                norm_inst == "default" || norm_inst == "__default__" || norm_inst.is_empty();
            let is_match = norm_target == "all"
                || norm_inst == norm_target
                || (is_target_default && is_inst_default);
            crate::modules::logger::log_instance_prompt_audit(
                &norm_target,
                "check_and_dispatch_enqueued_prompts",
                &project_id,
                &repo_path,
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
            if !is_match {
                continue;
            }
        }

        // Check if project is currently running a prompt
        let is_running = is_prompt_running_for_project(&project_id, &inst_id)
            || is_prompt_running_for_project(&repo_path, &inst_id);

        let clean_project_name = Path::new(&repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| project_id.clone());

        if is_running {
            crate::modules::logger::log_info(&format!(
                "[PromptQueueScheduler] Project '{}' ({}) is currently active/busy; enqueued prompts remain queued",
                clean_project_name, project_id
            ));
            continue;
        }

        // Project is IDLE! Pick the first enqueued prompt (FIFO: earliest created_at)
        let prompt_opt: Option<ActivePrompt> = conn
            .query_row(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                 FROM active_prompts
                 WHERE (project_id = ?1 OR repo_path = ?2)
                   AND (instance_id = ?3 OR (?3 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
                   AND status IN ('backed_up', 'queued', 'pending')
                 ORDER BY created_at ASC, id ASC
                 LIMIT 1",
                params![&project_id, &repo_path, &inst_id],
                |row| {
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
                },
            )
            .optional()
            .map_err(|e| format!("Failed to fetch enqueued prompt for {}: {}", project_id, e))?;

        let Some(prompt) = prompt_opt else {
            continue;
        };

        crate::modules::logger::log_info(&format!(
            "[PromptQueueScheduler] Project '{}' verified idle. Dispatching enqueued prompt '{}' (conv: {:?})",
            clean_project_name, prompt.id, prompt.session_id
        ));

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        let final_img = prompt.image_payload.clone().or(extracted_img);
        let mut payload = resume_task_document(&prompt, "dispatched", now, &img_paths);
        if final_img.is_some() {
            payload["image_payload"] = serde_json::json!(final_img);
        }
        // Justification: resume snapshot is a convenience file for the IDE; dispatch proceeds via agy regardless
        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            crate::error::record_ignored(
                fs::write(&task_file, json_str),
                "write resume task snapshot",
            );
        }

        // Spawn prompt via agy
        let sent = spawn_prompt_via_agy(&prompt);

        // Update active_prompts status to 'dispatched'
        // Justification: dispatch state update; a missed update leaves the prompt backed_up so it is retried on the next dispatch
        crate::error::record_ignored(
            conn.execute(
                "UPDATE active_prompts SET status = 'dispatched', updated_at = ?1 WHERE id = ?2",
                params![now, &prompt.id],
            ),
            "mark prompt dispatched after agy send",
        );

        if let Ok(mut map) = get_memory_prompts_map().lock() {
            if let Some(p) = map.get_mut(&prompt.id) {
                p.status = "dispatched".to_string();
                p.updated_at = now;
            }
        }

        // Record SchedulerFacts in Audit Trail
        let prompt_preview = if prompt.prompt_content.len() > 140 {
            format!("{}...", &prompt.prompt_content[..140])
        } else {
            prompt.prompt_content.clone()
        };

        let action_taken = if sent {
            "dispatched"
        } else {
            "resume_task_written"
        };
        let reason = format!(
            "Project verified idle; enqueued prompt pushed automatically via {}",
            action_taken
        );

        let facts = crate::modules::task_history_db::SchedulerFacts {
            scheduler_run_id: run_id.clone(),
            project_name: clean_project_name.clone(),
            repo_path: prompt.repo_path.clone(),
            prompt_id: prompt.id.clone(),
            prompt_preview,
            conversation_id: prompt.session_id.clone().unwrap_or_default(),
            action_taken: action_taken.to_string(),
            reason,
            idle_check_passed: true,
            instance_id: prompt.instance_id.clone(),
            timestamp: now,
        };

        // Justification: audit-trail write is observability; the dispatch already succeeded
        crate::error::record_ignored(
            crate::modules::task_history_db::record_scheduler_event(&facts),
            "record scheduler event in audit trail",
        );
        dispatched_count += 1;
    }

    invalidate_prompt_tree_cache(target_instance);
    Ok(dispatched_count)
}

/// Enqueue a prompt with optional conversation_id and project_id metadata, delegating to the authoritative enqueue_prompt_for_instance.
pub fn enqueue_prompt_for_instance_full(
    instance_id: &str,
    prompt_text: &str,
    workspace_path: Option<&str>,
    conversation_id: Option<&str>,
    _project_id: Option<&str>,
) -> Result<i64, AppError> {
    let repo_path = workspace_path.unwrap_or("").trim();
    let prompt = enqueue_prompt_for_instance(instance_id, repo_path, prompt_text, conversation_id)
        .map_err(AppError::Config)?;
    let conn = connect_db().map_err(AppError::Config)?;
    let row_id: i64 = conn
        .query_row(
            "SELECT rowid FROM active_prompts WHERE id = ?1 LIMIT 1",
            rusqlite::params![&prompt.id],
            |r| r.get(0),
        )
        .unwrap_or(1);
    Ok(row_id)
}

/// Re-enqueue running conversations for an instance before switch/restart.
/// Marks active prompts as 'backed_up', updates resume document, and logs to Audit.
pub fn requeue_running_conversations_for_instance(instance_id: &str) -> Result<usize, String> {
    let count = backup_running_prompts(instance_id)?;
    if count == 0 {
        return Ok(0);
    }

    if let Ok(conn) = connect_db() {
        let is_all_or_default =
            instance_id == "all" || instance_id == "__default__" || instance_id == "default";
        let stmt_res = if instance_id == "all" {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' ORDER BY created_at ASC, id ASC LIMIT 50",
            )
        } else if is_all_or_default {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' AND instance_id IN ('default', '__default__', '') ORDER BY created_at ASC, id ASC LIMIT 50",
            )
        } else {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' AND instance_id = ?1 ORDER BY created_at ASC, id ASC LIMIT 50",
            )
        };

        if let Ok(mut stmt) = stmt_res {
            let rows: Vec<(String, String, String, String, String, Option<String>)> =
                if instance_id == "all" || is_all_or_default {
                    stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                        ))
                    })
                    .map(|mapped| mapped.flatten().collect())
                    .unwrap_or_default()
                } else {
                    stmt.query_map(params![instance_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                        ))
                    })
                    .map(|mapped| mapped.flatten().collect())
                    .unwrap_or_default()
                };

            for (_id, project_id, inst_id, repo_path, prompt_content, session_id) in rows {
                let project_name = Path::new(&repo_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(project_id);
                let cid = session_id.unwrap_or_default();
                let preview = if prompt_content.len() > 100 {
                    format!("{}...", &prompt_content[..100])
                } else {
                    prompt_content
                };
                let reason = "In-flight conversation re-enqueued for restart/switch continuity";
                // Justification: audit-trail write is observability; the requeue already completed
                crate::error::record_ignored(
                    crate::modules::task_history_db::record_requeue_event(
                        &project_name,
                        &inst_id,
                        &cid,
                        reason,
                        &preview,
                    ),
                    "record requeue event in audit trail",
                );
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn test_enqueued_prompts_fifo_dispatch_order() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-t1', 'test-proj', 'default', '/test/repo', 'First prompt', 'gemini-2.5-pro', NULL, 'queued', 100, 100, NULL),
                    ('p-t2', 'test-proj', 'default', '/test/repo', 'Second prompt', 'gemini-2.5-pro', NULL, 'queued', 200, 200, NULL),
                    ('p-t3', 'test-proj', 'default', '/test/repo', 'Third prompt', 'gemini-2.5-pro', NULL, 'queued', 300, 300, NULL)",
            [],
        ).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id FROM active_prompts 
                 WHERE status IN ('backed_up', 'queued', 'pending')
                 ORDER BY created_at ASC, id ASC",
            )
            .unwrap();
        let prompt_ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();

        assert_eq!(prompt_ids, vec!["p-t1", "p-t2", "p-t3"]);
    }
}
