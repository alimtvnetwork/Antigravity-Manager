//! Repo DB: project queries

use super::dispatch::parse_flexible_timestamp;
use super::failed_commands::decode_uri_to_path;
use super::gemini_dirs::gemini_dirs_tagged;
use super::models::{ActivePrompt, ProjectExecutionInfo, RunningProject};
use super::schema::connect_db;
use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashMap;
use std::collections::HashSet;

/// List all running projects across instances
pub fn list_running_projects() -> Result<Vec<RunningProject>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
             FROM running_projects 
             WHERE workspace_storage_path IS NOT NULL 
               AND trim(workspace_storage_path) != '' 
               AND instr(id, '__') > 0 
               AND trim(instance_id) != ''
             ORDER BY last_detected_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list projects query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let running_int: i32 = row.get(5)?;
            let is_running = running_int != 0;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query running projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

pub fn list_all_prompts() -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 500",
        )
        .map_err(|e| format!("Failed to prepare list prompts query: {}", e))?;

    let rows = stmt
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
        .map_err(|e| format!("Failed to query prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// Normalize filesystem path for reliable cross-platform comparison
pub(crate) fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Retrieve live project execution information by inspecting Antigravity's
/// conversation_summaries.db (ground-truth for in-flight requests), active OS processes,
/// and SQLite repo_prompts.db.
pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {
    let mut results: Vec<ProjectExecutionInfo> = Vec::new();
    let now = Utc::now().timestamp();

    // Map of (instance_id, normalized repo path) -> (is_running, active_prompt_snippet, last_time)
    let mut live_map: std::collections::HashMap<(String, String), (bool, Option<String>, i64)> =
        std::collections::HashMap::new();
    let mut active_conv_prefixes: Vec<(String, String, Option<String>, i64)> = Vec::new();

    // 1. Inspect Antigravity conversation_summaries.db
    let candidate_dirs = gemini_dirs_tagged(None);
    for (owning_inst_id, base_dir) in &candidate_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }
        let conn = Connection::open_with_flags(
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

        if let Ok(conn) = conn {
            // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
            crate::error::record_ignored(
                conn.pragma_update(None, "busy_timeout", 3000),
                "set busy_timeout on Antigravity summaries database",
            );
            if let Ok(mut stmt) = conn.prepare(
                "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
                 FROM conversation_summaries 
                 ORDER BY last_modified_time DESC 
                 LIMIT 30",
            ) {
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                });
                if let Ok(rows) = rows {
                    let norm_owning_inst = if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                        "default".to_string()
                    } else {
                        crate::modules::instance::resolve_instance_id(owning_inst_id)
                            .unwrap_or_else(|_| owning_inst_id.to_string())
                    };
                    let norm_inst = norm_owning_inst.to_lowercase();

                    for item in rows.flatten() {
                        let (cid, _title, preview, status, not_fully_idle, ws_uris_opt, _last_time_str) = item;

                        // Adaptive 10-Minute Thinking Window: Parse timestamp and allow up to 600s for active reasoning
                        let conv_time = parse_flexible_timestamp(&_last_time_str);
                        let is_recent = conv_time > 0 && (now - conv_time <= 600);

                        let is_idle_count = not_fully_idle == 0;
                        let has_idle_status = status.contains("IDLE")
                            || status.contains("COMPLETED")
                            || status.contains("FAILED")
                            || status.contains("CANCELLED");
                        let is_explicit_idle = is_idle_count || has_idle_status;

                        let is_conv_running = if is_explicit_idle || !is_recent {
                            false
                        } else {
                            not_fully_idle != 0 && status.contains("RUNNING")
                        };
                        let prompt_preview = if !preview.trim().is_empty() {
                            Some(preview)
                        } else {
                            None
                        };

                        let prefix_8 = if cid.len() >= 8 {
                            cid[..8].to_string()
                        } else {
                            cid.clone()
                        };

                        if is_conv_running && prefix_8.len() >= 6 {
                            active_conv_prefixes.push((norm_inst.clone(), prefix_8, prompt_preview.clone(), conv_time));
                        }

                        if let Some(ws_uris_raw) = ws_uris_opt {
                            let ws_uris: Vec<String> =
                                serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                            for u in ws_uris {
                                let clean_p = normalize_path_for_compare(&decode_uri_to_path(&u));
                                let entry = live_map
                                    .entry((norm_inst.clone(), clean_p))
                                    .or_insert((false, None, now));
                                if is_conv_running {
                                    entry.0 = true;
                                    if entry.1.is_none() && prompt_preview.is_some() {
                                        entry.1 = prompt_preview.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Inspect active_prompts in repo_prompts.db for any in-flight prompts
    if let Ok(conn) = connect_db() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT instance_id, repo_path, prompt_content, status, updated_at FROM active_prompts WHERE status = 'running' OR status = 'queued'",
        ) {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            });
            if let Ok(rows) = rows {
                for item in rows.flatten() {
                    let (p_inst, p_path, p_content, status, updated_at) = item;
                    let norm_ap_inst = if p_inst == "default" || p_inst == "__default__" {
                        "default".to_string()
                    } else if p_inst.trim().is_empty() {
                        "unassigned".to_string()
                    } else {
                        crate::modules::instance::resolve_instance_id(&p_inst)
                            .unwrap_or_else(|_| p_inst.clone())
                    };
                    let norm_inst = norm_ap_inst.to_lowercase();
                    let clean_p = normalize_path_for_compare(&p_path);
                    let entry = live_map
                        .entry((norm_inst, clean_p))
                        .or_insert((false, None, now));

                    let is_running_status = status == "running";
                    let is_fresh = (now - updated_at) <= 600;
                    if is_running_status && is_fresh {
                        entry.0 = true;
                    }

                    if entry.1.is_none() {
                        entry.1 = Some(p_content.chars().take(120).collect());
                    }
                }
            }
        }
    }

    // 3. Merge with discovered projects from running_projects
    let projects = list_running_projects().unwrap_or_default();
    let mut seen_keys = std::collections::HashSet::new();
    for p in projects {
        let clean_path = normalize_path_for_compare(&p.repo_path);
        let norm_proj_inst = if p.instance_id == "default" || p.instance_id == "__default__" {
            "default".to_string()
        } else if p.instance_id.trim().is_empty() {
            "unassigned".to_string()
        } else {
            crate::modules::instance::resolve_instance_id(&p.instance_id)
                .unwrap_or_else(|_| p.instance_id.clone())
        };
        let norm_inst = norm_proj_inst.to_lowercase();
        let dedupe_key = (norm_inst.clone(), clean_path.clone());
        if !seen_keys.insert(dedupe_key) {
            continue; // Deduplicate duplicate workspace rows
        }

        let mut is_running = false;
        let mut prompt_snippet = None;
        let mut last_time = p.last_detected_at;

        // Gate 0: Host process must be running on OS for any project to be actively running
        let (is_inst_alive, _, _) =
            crate::modules::instance::is_instance_process_running_smart(&norm_inst);

        // Check path match in live_map only when host instance is actively running
        if is_inst_alive {
            if let Some((run, snippet, l_time)) =
                live_map.get(&(norm_inst.clone(), clean_path.clone()))
            {
                if *run {
                    is_running = true;
                    prompt_snippet = snippet.clone();
                    last_time = *l_time;
                }
            }
        }

        let is_idle = !is_running;
        let status_str = if is_running {
            "RUNNING".to_string()
        } else {
            "IDLE".to_string()
        };

        results.push(ProjectExecutionInfo {
            project_id: p.id,
            repo_name: p.repo_name,
            repo_path: p.repo_path,
            is_running,
            is_idle,
            status: status_str,
            active_prompt: prompt_snippet,
            last_detected_at: last_time,
        });
    }

    results
}

/// Helper to get live execution status for a specific project
pub fn get_project_execution_status(
    project_id: &str,
    instance_id: &str,
) -> Option<ProjectExecutionInfo> {
    if project_id.trim().is_empty() {
        return None;
    }
    let norm_target = normalize_path_for_compare(project_id);
    let norm_inst = crate::modules::instance::resolve_instance_id(instance_id)
        .unwrap_or_else(|_| instance_id.to_string())
        .to_lowercase();
    let infos = get_live_project_execution_info();
    infos.into_iter().find(|p| {
        (p.project_id == project_id
            || (!norm_target.is_empty() && normalize_path_for_compare(&p.repo_path) == norm_target))
            && (instance_id == "all"
                || instance_id.is_empty()
                || p.project_id.to_lowercase().contains(&norm_inst))
    })
}

/// Returns true if ANY project or conversation is currently actively running
pub fn is_any_prompt_actively_running() -> bool {
    // 1. Check Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            let conn = Connection::open_with_flags(
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
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(conn) = conn {
                // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
                crate::error::record_ignored(
                    conn.pragma_update(None, "busy_timeout", 3000),
                    "set busy_timeout on Antigravity summaries database",
                );
                let count: i32 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM conversation_summaries 
                         WHERE not_fully_idle != 0 
                           AND status LIKE '%RUNNING%' 
                           AND status NOT LIKE '%IDLE%' 
                           AND status NOT LIKE '%COMPLETED%' 
                           AND status NOT LIKE '%FAILED%' 
                           AND status NOT LIKE '%CANCELLED%'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if count > 0 {
                    return true;
                }
            }
        }
    }

    // 2. Check repo_prompts.db active_prompts
    if let Ok(conn) = connect_db() {
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status = 'running' OR status = 'queued'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if count > 0 {
            return true;
        }
    }

    false
}
