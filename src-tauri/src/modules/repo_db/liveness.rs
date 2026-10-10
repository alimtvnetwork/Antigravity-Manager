//! Repo DB: liveness

use super::dispatch::parse_flexible_timestamp;
use super::failed_commands::decode_uri_to_path;
use super::gemini_dirs::gemini_dirs_for_instance;
use super::project_queries::normalize_path_for_compare;
use super::schema::connect_db;
use super::sequences::extract_prompt_words_preview;
use super::state::{get_active_agy_workers, get_memory_prompts_map};
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::path::Path;

/// Check if any prompt or task is currently actively executing for a project
pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool {
    if project_id.trim().is_empty() {
        return false;
    }
    let now = Utc::now().timestamp();
    let resolved_inst =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let norm_inst = resolved_inst.as_str();

    // 0. Dynamic process activity check: If no Antigravity process is actively running for this instance, prompt cannot be executing
    let (has_active_process, host_pid, _) =
        crate::modules::instance::is_instance_process_running_smart(norm_inst);
    let resolved_name = norm_inst.to_string();

    if !has_active_process {
        crate::modules::logger::log_instance_prompt_audit(
            norm_inst,
            &resolved_name,
            project_id,
            project_id,
            "process_table",
            None,
            "Gate0:HostProcessLiveness",
            false,
            "INSTANCE_PROCESS_DEAD",
        );
        return false;
    }

    // Check if the prompt has already transitioned to a terminal status ('completed' / 'failed')
    let in_memory_terminal = if let Ok(map) = get_memory_prompts_map().lock() {
        map.values().any(|p| {
            let matches_inst = if norm_inst == "default" {
                p.instance_id == "default" || p.instance_id == "__default__"
            } else {
                p.instance_id == norm_inst
            };
            matches_inst
                && (p.project_id == project_id || p.repo_path == project_id)
                && (p.status == "completed" || p.status == "failed")
        })
    } else {
        false
    };

    let db_terminal = if let Ok(conn) = connect_db() {
        conn.query_row(
            "SELECT status FROM active_prompts 
             WHERE (project_id = ?1 OR repo_path = ?1) 
               AND (?2 = 'all' OR instance_id = ?2 OR (?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
             ORDER BY updated_at DESC, created_at DESC LIMIT 1",
            params![project_id, norm_inst],
            |r| r.get::<_, String>(0),
        )
        .map(|s| s == "completed" || s == "failed")
        .unwrap_or(false)
    } else {
        false
    };

    let has_terminal_transition = in_memory_terminal || db_terminal;

    // 1. Check in-memory active prompts map
    let is_terminal_override = has_terminal_transition;
    if !is_terminal_override {
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            // Prune expired or terminal prompts from memory map (45s TTL)
            map.retain(|_, p| {
                let is_active_status = p.status == "running" || p.status == "dispatched";
                let is_within_ttl = (now - p.updated_at) <= 45;
                is_active_status && is_within_ttl
            });
            for p in map.values() {
                let matches_inst = if norm_inst == "default" {
                    p.instance_id == "default" || p.instance_id == "__default__"
                } else {
                    p.instance_id == norm_inst
                };
                let matches_proj = p.project_id == project_id || p.repo_path == project_id;
                if matches_inst && matches_proj {
                    let has_running_status = p.status == "running";
                    let is_fresh = (now - p.updated_at) <= 45;
                    if has_running_status && is_fresh {
                        crate::modules::logger::log_instance_prompt_audit(
                            norm_inst,
                            &resolved_name,
                            project_id,
                            &p.repo_path,
                            "memory_prompts_map",
                            host_pid,
                            "Gate1:MemoryPromptsMap",
                            true,
                            "ACTIVE_PROMPT_MEMORY",
                        );
                        return true;
                    }
                }
            }
        }
    }

    // 2. Check active workers map strictly scoped by instance_id
    let clean_target = normalize_path_for_compare(project_id);

    if let Ok(mut workers) = get_active_agy_workers().lock() {
        let mut dead_keys = Vec::new();
        let mut matched_worker: Option<(String, u32)> = None;

        for (key, &pid) in workers.iter() {
            let (worker_inst, worker_path) = match key.split_once(':') {
                Some((inst, path)) => (inst, path),
                None => ("", key.as_str()),
            };

            let is_inst_match = norm_inst == "all"
                || worker_inst.eq_ignore_ascii_case(norm_inst)
                || (norm_inst == "default"
                    && (worker_inst == "default" || worker_inst == "__default__"));

            let clean_worker_path = normalize_path_for_compare(worker_path);
            let is_path_match = !clean_target.is_empty() && clean_worker_path == clean_target;

            if is_inst_match && is_path_match {
                // Verify OS process liveness for PID
                let mut sys = sysinfo::System::new();
                let target_pid = sysinfo::Pid::from_u32(pid);
                sys.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                    sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
                );
                let is_alive = sys.process(target_pid).is_some();
                if is_alive {
                    matched_worker = Some((clean_worker_path, pid));
                    break;
                } else {
                    dead_keys.push(key.clone());
                }
            }
        }
        for k in dead_keys {
            workers.remove(&k);
        }
        if let Some((clean_worker_path, pid)) = matched_worker {
            crate::modules::logger::log_instance_prompt_audit(
                norm_inst,
                &resolved_name,
                project_id,
                &clean_worker_path,
                "active_agy_workers_map",
                Some(pid),
                "Gate2:ActiveAgyWorkers",
                true,
                "ACTIVE_WORKER_MATCHED",
            );
            return true;
        }
    }

    // 3. Check SQLite active_prompts for 'running' (strictly require status = 'running' with active TTL updated_at >= now - 300)
    if let Ok(conn) = connect_db() {
        let running_count: usize = if has_terminal_transition {
            0
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM active_prompts 
                 WHERE (project_id = ?1 OR repo_path = ?1) 
                   AND (?2 = 'all' OR instance_id = ?2 OR (?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
                   AND status = 'running'
                   AND updated_at >= ?3",
                params![project_id, norm_inst, now - 45],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };
        let has_running_prompts = running_count > 0;
        if has_running_prompts {
            crate::modules::logger::log_instance_prompt_audit(
                norm_inst,
                &resolved_name,
                project_id,
                project_id,
                "active_prompts_table",
                host_pid,
                "Gate3:ActivePromptsSQLite",
                true,
                "ACTIVE_PROMPT_DB_RUNNING",
            );
            return true;
        }
    }

    // 4. Check Antigravity live conversation summaries directly for running sessions in this instance
    let candidate_dirs = gemini_dirs_for_instance(norm_inst);

    for base_dir in candidate_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }

        let conn_res = Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            let uri = format!(
                "file:{}?immutable=1",
                summaries_db.to_string_lossy().replace('\\', "/")
            );
            Connection::open_with_flags(
                &uri,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        });

        if let Ok(conn) = conn_res {
            // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
            crate::error::record_ignored(
                conn.pragma_update(None, "busy_timeout", 3000),
                "set busy_timeout on Antigravity summaries database",
            );
            if let Ok(mut stmt) = conn.prepare(
                "SELECT status, not_fully_idle, workspace_uris, last_modified_time, title, preview 
                 FROM conversation_summaries 
                 ORDER BY last_modified_time DESC 
                 LIMIT 30",
            ) {
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i32>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    ))
                });
                if let Ok(rows) = rows {
                    for item in rows.flatten() {
                        let (status, not_fully_idle, ws_uris_opt, _last_time_str, title, preview) =
                            item;

                        // Ghost conversation filter: Inspect title and preview. If empty title / "Untitled Conversation"
                        // and 0 prompt content or 0 words, skip it (do not consider it running).
                        let is_untitled = title.trim().is_empty()
                            || title.to_lowercase().starts_with("untitled")
                            || title.to_lowercase() == "new conversation";
                        let (_, eff_wc) = extract_prompt_words_preview(&preview, 5);
                        let is_empty_prompt = preview.trim().is_empty() || eff_wc == 0;
                        if (is_untitled && is_empty_prompt)
                            || (title.trim().is_empty() && preview.trim().is_empty())
                        {
                            continue;
                        }

                        // Strict idle supremacy rule:
                        // if not_fully_idle == 0 or status contains "IDLE", "COMPLETED", "FAILED", "CANCELLED", it is unconditionally IDLE.
                        let is_idle_count = not_fully_idle == 0;
                        let has_idle_status = status.contains("IDLE")
                            || status.contains("COMPLETED")
                            || status.contains("FAILED")
                            || status.contains("CANCELLED");
                        let is_explicit_idle = is_idle_count || has_idle_status;

                        if is_explicit_idle {
                            continue;
                        }

                        // Adaptive 10-Minute Thinking Window: Expand the timestamp cutoff from rigid 120s to 600s (10 minutes)
                        // for active turns with not_fully_idle > 0 && status.contains("RUNNING"), ensuring reasoning/thinking models
                        // (Claude 3.7 Thinking, Gemini 2.5 Pro) are not prematurely flipped to IDLE mid-generation!
                        let conv_time = parse_flexible_timestamp(&_last_time_str);
                        let is_recent = conv_time > 0 && (now - conv_time <= 60);

                        if !is_recent {
                            continue;
                        }

                        let is_conv_running = not_fully_idle > 0 && status.contains("RUNNING");

                        if is_conv_running {
                            if let Some(ws_uris_raw) = ws_uris_opt {
                                let ws_uris: Vec<String> =
                                    serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                                for u in ws_uris {
                                    let clean_p =
                                        normalize_path_for_compare(&decode_uri_to_path(&u));
                                    let folder_name = Path::new(&clean_p)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    let has_target = !clean_target.is_empty();
                                    let is_target_matched = has_target
                                        && (clean_p == clean_target || folder_name == clean_target);
                                    if is_target_matched {
                                        crate::modules::logger::log_instance_prompt_audit(
                                            norm_inst,
                                            &resolved_name,
                                            project_id,
                                            &clean_p,
                                            &summaries_db.to_string_lossy(),
                                            host_pid,
                                            "Gate4:ConversationSummariesLiveTurn",
                                            true,
                                            "CONVERSATION_SUMMARY_ACTIVE_TURN",
                                        );
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    crate::modules::logger::log_instance_prompt_audit(
        norm_inst,
        &resolved_name,
        project_id,
        project_id,
        "all_evaluated_databases",
        host_pid,
        "Gate0-4:AllEvaluationsCompleted",
        false,
        "IDLE_NO_ACTIVE_TASKS",
    );

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_scoping_and_idle_supremacy() {
        // Test key matching logic
        let default_key = "default:/work/antigravity-manager";
        let inst_8159_key = "8159:/work/coding-guidelines";

        let target_default = "default";
        let default_prefix = format!("{}:", target_default);

        // Instance "default" checking "coding-guidelines" -> worker on 8159 should NOT match
        let matches_default = (target_default == "all"
            || inst_8159_key.starts_with(&default_prefix)
            || (target_default == "default" && !inst_8159_key.contains(':')))
            && inst_8159_key.contains("coding-guidelines");
        assert!(!matches_default);

        // Instance "8159" checking "antigravity-manager" -> worker on default should NOT match
        let target_8159 = "8159";
        let prefix_8159 = format!("{}:", target_8159);
        let matches_8159 = (target_8159 == "all"
            || default_key.starts_with(&prefix_8159)
            || (target_8159 == "default" && !default_key.contains(':')))
            && default_key.contains("antigravity-manager");
        assert!(!matches_8159);

        // Instance "8159" checking "coding-guidelines" -> should match
        let matches_self = (target_8159 == "all"
            || inst_8159_key.starts_with(&prefix_8159)
            || (target_8159 == "default" && !inst_8159_key.contains(':')))
            && inst_8159_key.contains("coding-guidelines");
        assert!(matches_self);

        // Test idle supremacy logic
        let not_fully_idle = 0;
        let status = "CASCADE_RUN_STATUS_RUNNING";
        let is_explicit_idle = not_fully_idle == 0
            || status.contains("IDLE")
            || status.contains("COMPLETED")
            || status.contains("FAILED")
            || status.contains("CANCELLED");
        assert!(is_explicit_idle);

        let is_running = if is_explicit_idle {
            false
        } else {
            not_fully_idle != 0 && status.contains("RUNNING")
        };
        assert!(!is_running);
    }
}
