//! Repo DB: dispatch

use super::agy::spawn_or_replace_prompt_via_agy;
use super::detection::resume_task_document;
use super::models::{ActivePrompt, ProjectAuditRationaleType, PromptAuditCriteriaType};
use super::schema::{connect_db, init_tables};
use super::state::{get_dispatched_prompts_cache, get_memory_prompts_map};
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// True only when `agy` accepted the prompt. A resume file on disk is not enough.
pub(crate) fn prompt_push_landed(agy_ok: bool) -> bool {
    agy_ok
}

/// Directly dispatch/send backed-up prompts to the running projects with flexible instance matching.
pub fn dispatch_backed_up_prompts(instance_id: &str) -> Result<usize, String> {
    dispatch_running_prompts(instance_id)
}

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
             WHERE status IN ('backed_up', 'pending', 'in_progress', 'queued')
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

    let mut prompts: Vec<ActivePrompt> = all_backed_up
        .iter()
        .cloned()
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
            let rationale = ProjectAuditRationaleType::from_dispatch(is_match);
            crate::modules::logger::log_instance_prompt_audit(
                &target_inst,
                "dispatch_running_prompts",
                &p.id,
                &p.repo_path,
                "",
                None,
                PromptAuditCriteriaType::PromptDispatcherInstanceMatching.as_str(),
                is_match,
                rationale.as_str(),
            );
            is_match
        })
        .collect();

    // Flexible instance matching: If the target instance doesn't match or is missing,
    // dispatch to active instance or default instance instead of silently failing with 0 matches.
    if prompts.is_empty() && !all_backed_up.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] No exact instance match for target '{}'; falling back flexibly across {} backed-up prompt(s)",
            target_inst,
            all_backed_up.len()
        ));

        // 1. Try active instance
        let active_inst = crate::modules::instance::get_active_instance_id()
            .unwrap_or_else(|_| "default".to_string());
        let active_resolved = crate::modules::instance::resolve_instance_id(&active_inst)
            .unwrap_or_else(|_| "default".to_string());
        let is_active_default = active_resolved == "default" || active_resolved == "__default__";

        prompts = all_backed_up
            .iter()
            .cloned()
            .filter(|p| {
                let prompt_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                    .unwrap_or_else(|_| {
                        if p.instance_id == "__default__" || p.instance_id.is_empty() {
                            "default".to_string()
                        } else {
                            p.instance_id.clone()
                        }
                    });
                if is_active_default {
                    prompt_inst == "default"
                        || prompt_inst == "__default__"
                        || prompt_inst.is_empty()
                } else {
                    prompt_inst == active_resolved
                }
            })
            .collect();

        // 2. If still empty, try default instance
        if prompts.is_empty() {
            prompts = all_backed_up
                .iter()
                .cloned()
                .filter(|p| {
                    let prompt_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                        .unwrap_or_else(|_| {
                            if p.instance_id == "__default__" || p.instance_id.is_empty() {
                                "default".to_string()
                            } else {
                                p.instance_id.clone()
                            }
                        });
                    prompt_inst == "default"
                        || prompt_inst == "__default__"
                        || prompt_inst.is_empty()
                })
                .collect();
        }

        // 3. If still empty, dispatch all available backed-up prompts
        if prompts.is_empty() {
            crate::modules::logger::log_info(
                "[RepoDB] Fallback to all backed-up prompts regardless of instance tag",
            );
            prompts = all_backed_up;
        }
    }

    let mut dispatched_count = 0;

    for prompt in prompts {
        let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
        let already_dispatched = if prompt.status == "backed_up"
            || prompt.status == "pending"
            || prompt.status == "in_progress"
            || prompt.status == "queued"
        {
            false
        } else {
            let mut cache = match get_dispatched_prompts_cache().lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
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
        let (extracted_img, img_paths) =
            super::detection::extract_image_payload_or_path(&prompt.prompt_content);
        let final_img = prompt.image_payload.clone().or(extracted_img);
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let mut payload = resume_task_document(&prompt, "dispatched", now, &img_paths);
        if final_img.is_some() {
            payload["image_payload"] = serde_json::json!(final_img);
        }
        let file_written = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };

        let clean_prompt = super::text_utils::extract_clean_user_prompt(&prompt.prompt_content);
        let fallback_prompt = if clean_prompt.trim().is_empty() {
            prompt.prompt_content.trim()
        } else {
            clean_prompt.as_str()
        };
        let target_is_new = prompt.session_id.as_deref().map(str::trim) == Some("new");

        let (is_running, _, pids) =
            crate::modules::instance::is_instance_process_running_smart(&target_inst);

        let mut agy_sent: Result<(), String> = Err("not_attempted".to_string());
        if is_running {
            crate::modules::instance::focus_running_instance(&pids, Some(&prompt.repo_path));
            if crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance_with_mode(
                &target_inst,
                fallback_prompt,
                false,
                target_is_new,
            )
            .is_ok()
            {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Direct CDP prompt submission succeeded for prompt '{}' on instance '{}' (new_chat: {})",
                    prompt.id, target_inst, target_is_new
                ));
                agy_sent = Ok(());
            }
        }

        if agy_sent.is_err() {
            agy_sent = spawn_or_replace_prompt_via_agy(&prompt, true);
        }
        // The resume file is only a hint. A written file with a session id is not
        // proof the IDE continued the prompt. Leave the row backed_up so the
        // post-launch retry can push it once the process is accepting input.
        if !prompt_push_landed(agy_sent.is_ok()) {
            if file_written {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] Prompt '{}' resume file was written but agy/CDP did not accept it; left backed_up",
                    prompt.id
                ));
            }
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
            let byte_size = prompt.prompt_content.len();
            let image_count = if !img_paths.is_empty() {
                img_paths.len()
            } else if final_img.is_some() {
                1
            } else {
                0
            };
            tracing::info!(
                "[PROMPT_RESTORE] Restoring running prompt for instance {} from backup (images: {}, bytes: {})",
                prompt.instance_id,
                image_count,
                byte_size
            );
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

/// Load every prompt in a dispatchable status, in FIFO order
/// (`ORDER BY created_at ASC, id ASC` — the queue invariant).
fn query_dispatchable_prompts(conn: &Connection) -> Result<Vec<ActivePrompt>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
             FROM active_prompts
             WHERE status IN ('backed_up', 'pending', 'in_progress', 'queued')
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to prepare dispatch query: {}", e))?;

    let prompts = stmt
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
    Ok(prompts)
}

/// Strict instance match for dispatch selection, with the same audit logging
/// as `dispatch_running_prompts`. No flexible fallback: a prompt tagged for a
/// different instance never matches here.
fn prompt_matches_target(
    prompt: &ActivePrompt,
    target_inst: &str,
    is_default_target: bool,
    action: &str,
) -> bool {
    let prompt_inst = crate::modules::instance::resolve_instance_id(&prompt.instance_id)
        .unwrap_or_else(|_| {
            if prompt.instance_id == "__default__" || prompt.instance_id.is_empty() {
                "default".to_string()
            } else {
                prompt.instance_id.clone()
            }
        });
    let is_match = if is_default_target {
        prompt_inst == "default" || prompt_inst == "__default__" || prompt_inst.is_empty()
    } else {
        prompt_inst == target_inst
    };
    let rationale = ProjectAuditRationaleType::from_dispatch(is_match);
    crate::modules::logger::log_instance_prompt_audit(
        target_inst,
        action,
        &prompt.id,
        &prompt.repo_path,
        "",
        None,
        PromptAuditCriteriaType::PromptDispatcherInstanceMatching.as_str(),
        is_match,
        rationale.as_str(),
    );
    is_match
}

/// Pick the single most recently active prompt: greatest `updated_at`, ties
/// broken by greatest `created_at`, then greatest `id`. Candidates arrive in
/// FIFO order; this is a one-row selection, not a queue reordering — the FIFO
/// dispatch invariant for the backlog is unchanged.
fn pick_latest_prompt(prompts: &[ActivePrompt]) -> Option<ActivePrompt> {
    prompts
        .iter()
        .max_by(|a, b| {
            a.updated_at
                .cmp(&b.updated_at)
                .then_with(|| a.created_at.cmp(&b.created_at))
                .then_with(|| a.id.cmp(&b.id))
        })
        .cloned()
}

/// Send one prompt through the same resume-file + agy/CDP path as the
/// per-prompt step of `dispatch_running_prompts`. Returns true only when the
/// send landed and the row was marked `dispatched`.
fn dispatch_one_prompt(
    conn: &Connection,
    prompt: &ActivePrompt,
    target_inst: &str,
    instance_id: &str,
    now: i64,
) -> bool {
    let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
    let already_dispatched = if prompt.status == "backed_up"
        || prompt.status == "pending"
        || prompt.status == "in_progress"
        || prompt.status == "queued"
    {
        false
    } else {
        let mut cache = match get_dispatched_prompts_cache().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
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
        return false;
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Directly dispatching prompt '{}' to project at '{}' without queuing",
        prompt.id, prompt.repo_path
    ));

    // Keep the conversation id on the file the IDE reads. A later rewrite must not drop it.
    let (extracted_img, img_paths) =
        super::detection::extract_image_payload_or_path(&prompt.prompt_content);
    let final_img = prompt.image_payload.clone().or(extracted_img);
    let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
    let mut payload = resume_task_document(prompt, "dispatched", now, &img_paths);
    if final_img.is_some() {
        payload["image_payload"] = serde_json::json!(final_img);
    }
    let file_written = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
        fs::write(&task_file, json_str).is_ok()
    } else {
        false
    };

    let clean_prompt = super::text_utils::extract_clean_user_prompt(&prompt.prompt_content);
    let fallback_prompt = if clean_prompt.trim().is_empty() {
        prompt.prompt_content.trim()
    } else {
        clean_prompt.as_str()
    };
    let target_is_new = prompt.session_id.as_deref().map(str::trim) == Some("new");

    let (is_running, _, pids) =
        crate::modules::instance::is_instance_process_running_smart(target_inst);

    let mut agy_sent: Result<(), String> = Err("not_attempted".to_string());
    if is_running {
        crate::modules::instance::focus_running_instance(&pids, Some(&prompt.repo_path));
        if crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance_with_mode(
            target_inst,
            fallback_prompt,
            false,
            target_is_new,
        )
        .is_ok()
        {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Direct CDP prompt submission succeeded for prompt '{}' on instance '{}' (new_chat: {})",
                prompt.id, target_inst, target_is_new
            ));
            agy_sent = Ok(());
        }
    }

    if agy_sent.is_err() {
        agy_sent = spawn_or_replace_prompt_via_agy(prompt, true);
    }
    // The resume file is only a hint. A written file with a session id is not
    // proof the IDE continued the prompt. Leave the row backed_up so the
    // post-launch retry can push it once the process is accepting input.
    if !prompt_push_landed(agy_sent.is_ok()) {
        if file_written {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] Prompt '{}' resume file was written but agy/CDP did not accept it; left backed_up",
                prompt.id
            ));
        }
        crate::modules::logger::log_error(&format!(
            "[RepoDB] Prompt '{}' was not re-pushed for instance '{}'; left backed_up",
            prompt.id, instance_id
        ));
        return false;
    }

    let updated = conn.execute(
        "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
        params![now, &prompt.id],
    );

    if updated.is_ok() {
        let byte_size = prompt.prompt_content.len();
        let image_count = if !img_paths.is_empty() {
            img_paths.len()
        } else if final_img.is_some() {
            1
        } else {
            0
        };
        tracing::info!(
            "[PROMPT_RESTORE] Restoring running prompt for instance {} from backup (images: {}, bytes: {})",
            prompt.instance_id,
            image_count,
            byte_size
        );
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            if let Some(p) = map.get_mut(&prompt.id) {
                p.status = "dispatched".to_string();
                p.updated_at = now;
            }
        }
    }
    updated.is_ok()
}

/// Post-switch / post-fast-forward resume (user decision, 2026-10-24):
/// re-dispatch exactly ONE prompt — the most recently active backed-up prompt
/// strictly belonging to `instance_id`. Every other backed-up prompt stays
/// `backed_up` for a later manual or full dispatch. Unlike
/// `dispatch_running_prompts` there is NO flexible cross-instance fallback:
/// another instance's prompt is never dispatched from here.
pub fn dispatch_single_latest_prompt(instance_id: &str) -> Result<usize, String> {
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

    let all = query_dispatchable_prompts(&conn)?;
    let candidates: Vec<ActivePrompt> = all
        .iter()
        .cloned()
        .filter(|p| {
            prompt_matches_target(
                p,
                &target_inst,
                is_default_target,
                "dispatch_single_latest_prompt",
            )
        })
        .collect();

    let Some(prompt) = pick_latest_prompt(&candidates) else {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Single-prompt resume: no backed-up prompt for instance '{}'; nothing to dispatch",
            target_inst
        ));
        return Ok(0);
    };

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Single-prompt resume for instance '{}': dispatching only prompt '{}' ({} other backed-up prompt(s) stay backed up)",
        target_inst,
        prompt.id,
        candidates.len().saturating_sub(1)
    ));

    let landed = dispatch_one_prompt(&conn, &prompt, &target_inst, instance_id, now);
    invalidate_prompt_tree_cache(Some(&target_inst));
    Ok(if landed { 1 } else { 0 })
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

    #[test]
    fn resume_file_without_agy_does_not_count_as_restored() {
        assert!(!prompt_push_landed(false));
        assert!(prompt_push_landed(true));
    }

    #[test]
    fn test_dispatch_restores_prompt_with_images_and_workspace() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        let prompt_id = "test-restore-id";
        let project_id = "workspace-proj-123";
        let prompt_text = "Prompt with screenshot file:///d:/assets/screen.png";
        let img_payload = Some("data:image/png;base64,TESTDATA".to_string());

        // 1. Insert running project
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, 'inst-beta', 'proj-123', '/work/proj-123', NULL, 1, ?2, ?2)",
            params![project_id, now],
        ).unwrap();

        // 2. Insert prompt in backed_up state
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, 'inst-beta', '/work/proj-123', ?3, 'gemini-pro', 'conv-123', 'backed_up', ?4, ?4, ?5)",
            params![prompt_id, project_id, prompt_text, now, &img_payload],
        ).unwrap();

        // 3. Verify status transitions to dispatched and image payloads are retained
        let updated = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?1 WHERE id = ?2 AND status = 'backed_up'",
            params![now + 5, prompt_id],
        ).unwrap();
        assert_eq!(updated, 1);

        let (status, content, stored_img, proj) = conn
            .query_row(
                "SELECT status, prompt_content, image_payload, project_id FROM active_prompts WHERE id = ?1",
                params![prompt_id],
                |r| Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                )),
            )
            .unwrap();

        assert_eq!(status, "dispatched");
        assert_eq!(content, prompt_text);
        assert_eq!(stored_img, img_payload);
        assert_eq!(proj, project_id);
    }

    #[test]
    fn test_dispatch_backed_up_prompts_signature() {
        assert!(prompt_push_landed(true));
        assert!(!prompt_push_landed(false));
    }

    fn make_prompt(id: &str, created_at: i64, updated_at: i64) -> ActivePrompt {
        ActivePrompt {
            id: id.to_string(),
            project_id: "proj".to_string(),
            instance_id: "inst".to_string(),
            repo_path: "/work/proj".to_string(),
            prompt_content: "content".to_string(),
            model: None,
            session_id: None,
            status: "backed_up".to_string(),
            created_at,
            updated_at,
            image_payload: None,
        }
    }

    #[test]
    fn pick_latest_prompt_returns_none_for_empty() {
        assert!(pick_latest_prompt(&[]).is_none());
    }

    #[test]
    fn pick_latest_prompt_selects_most_recently_active() {
        let prompts = vec![
            make_prompt("old-active", 100, 500),
            make_prompt("newest", 50, 900),
            make_prompt("mid", 300, 700),
        ];
        let picked = pick_latest_prompt(&prompts).expect("candidate");
        assert_eq!(picked.id, "newest");
    }

    #[test]
    fn pick_latest_prompt_breaks_ties_by_creation_then_id() {
        let prompts = vec![make_prompt("a", 100, 900), make_prompt("b", 200, 900)];
        let picked = pick_latest_prompt(&prompts).expect("candidate");
        assert_eq!(picked.id, "b");
    }
}
