//! Repo DB: tree build phases

use super::discovery::discover_running_prompts_from_antigravity;
use super::models::RunningProject;
use super::schema::connect_db;
use crate::commands::instance::list_running_projects;
use crate::modules::repo_db::dispatch::parse_flexible_timestamp;
use crate::modules::repo_db::failed_commands::decode_uri_to_path;
use crate::modules::repo_db::gemini_dirs::gemini_dirs_tagged;
use crate::modules::repo_db::project_queries::normalize_path_for_compare;
use chrono::Utc;
use rusqlite::params;
use std::path::Path;

pub(crate) fn transition_stale_inflight_prompts(now: i64) {
    if let Ok(conn) = connect_db() {
        // Justification: stale-status sweep is housekeeping; rows are re-evaluated on the next scan
        crate::error::record_ignored(
            conn.execute(
                "UPDATE active_prompts
             SET status = 'completed', updated_at = ?1
             WHERE status IN ('running', 'in_flight', 'dispatched')
               AND (?1 - updated_at > 45)",
                rusqlite::params![now],
            ),
            "transition stale in-flight prompts to completed",
        );
    }
}

pub(crate) fn query_tree_projects(target: Option<&str>) -> Vec<RunningProject> {
    let all_projects: Vec<RunningProject> = list_running_projects()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            // Allow entries with no workspace_storage_path (conversation_summaries.db fallback)
            match p.workspace_storage_path.as_deref() {
                Some(path) => Path::new(path).exists(),
                None => true,
            }
        })
        .collect();
    if let Some(target_id) = target {
        if target_id == "default" || target_id == "__default__" {
            all_projects
                .into_iter()
                .filter(|p| p.instance_id == "default" || p.instance_id == "__default__")
                .collect()
        } else {
            all_projects
                .into_iter()
                .filter(|p| p.instance_id == target_id)
                .collect()
        }
    } else {
        all_projects
            .into_iter()
            .filter(|p| !p.instance_id.trim().is_empty())
            .collect()
    }
}

pub(crate) fn discover_fallback_projects(
    target: Option<&str>,
    projects: Vec<RunningProject>,
) -> Vec<RunningProject> {
    // ===== Fallback & Discovery: Supplement projects from conversation_summaries.db =====
    // When workspaceStorage is empty (common on Windows deployments), supplement
    // projects list from workspace_uris in conversation_summaries.db.
    let mut projects: Vec<RunningProject> = projects; // shadow as mutable
    let now_fb = Utc::now().timestamp();
    let candidate_dirs_fb = gemini_dirs_tagged(target);
    let mut seen_fb: std::collections::HashSet<(String, String)> = projects
        .iter()
        .map(|p| {
            (
                p.instance_id.clone(),
                normalize_path_for_compare(&p.repo_path),
            )
        })
        .collect();

    for (owning_inst_id, base) in &candidate_dirs_fb {
        // Verify if the owning instance process is actually alive on the OS
        let registry = crate::modules::instance::registry::load_registry().unwrap_or_default();
        let is_owning_inst_alive = if owning_inst_id == "default" || owning_inst_id == "__default__"
        {
            crate::modules::process::is_antigravity_running(None) || {
                let def_dir = crate::modules::instance::get_default_antigravity_data_dir();
                !crate::modules::instance::find_pids_for_data_dir(&def_dir.to_string_lossy(), true)
                    .is_empty()
            }
        } else if let Some(inst) = registry
            .instances
            .iter()
            .find(|i| i.id == *owning_inst_id || i.name == *owning_inst_id)
        {
            let (is_running, _, _) =
                crate::modules::instance::is_instance_process_running_smart(&inst.id);
            is_running
        } else {
            false
        };

        let summaries_db = base.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }
        let s_conn_res = Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        );
        let s_conn = match s_conn_res {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
        crate::error::record_ignored(
            s_conn.pragma_update(None, "busy_timeout", 3000),
            "set busy_timeout on Antigravity summaries database",
        );
        let sql = "SELECT workspace_uris, last_modified_time, status, not_fully_idle \
                   FROM conversation_summaries \
                   WHERE workspace_uris IS NOT NULL AND workspace_uris != '[]' \
                   ORDER BY last_modified_time DESC \
                   LIMIT 200";
        let mut stmt_opt = s_conn.prepare(sql).ok();
        if let Some(ref mut stmt) = stmt_opt {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i32>(3)?,
                ))
            }) {
                for item in rows.flatten() {
                    let (ws_uris_raw, last_time_str, status, not_fully_idle) = item;
                    let last_time_epoch = parse_flexible_timestamp(&last_time_str);
                    let is_recent = last_time_epoch > 0 && (now_fb - last_time_epoch <= 120);
                    let is_running = is_owning_inst_alive
                        && status.contains("RUNNING")
                        && not_fully_idle > 0
                        && is_recent;
                    let uris: Vec<String> = serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                    for uri in uris {
                        let raw_path = decode_uri_to_path(&uri);
                        if raw_path.len() < 4 {
                            continue;
                        }
                        let norm_inst =
                            if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                                "default".to_string()
                            } else {
                                crate::modules::instance::resolve_instance_id(owning_inst_id)
                                    .unwrap_or_else(|_| owning_inst_id.clone())
                            };
                        let norm_path_cmp = normalize_path_for_compare(&raw_path);
                        if !seen_fb.insert((norm_inst.clone(), norm_path_cmp)) {
                            continue;
                        }
                        let repo_name = Path::new(&raw_path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unnamed".to_string());
                        let project_key = format!("{}__{}", repo_name.to_lowercase(), norm_inst);
                        projects.push(RunningProject {
                            id: project_key,
                            instance_id: norm_inst,
                            repo_name,
                            repo_path: raw_path,
                            workspace_storage_path: None,
                            is_running,
                            last_detected_at: if last_time_epoch > 0 {
                                last_time_epoch
                            } else {
                                now_fb
                            },
                        });
                    }
                }
            }
        }
        drop(stmt_opt);
    }
    // ===== End Fallback & Discovery =====

    projects
}

pub(crate) fn log_tree_telemetry_probe(
    target: Option<&str>,
    registry: &crate::models::instance::InstanceRegistry,
    projects: &[RunningProject],
) {
    // Telemetry probe logging for instance liveness
    if let Some(target_id) = target {
        let (inst_name, data_dir, pids, is_alive) = if target_id == "default"
            || target_id == "__default__"
        {
            let is_running = crate::modules::process::is_antigravity_running(None);
            (
                "default".to_string(),
                "default".to_string(),
                vec![],
                is_running,
            )
        } else if let Some(inst) = registry
            .instances
            .iter()
            .find(|i| i.id == target_id || i.name == target_id)
        {
            let pids =
                crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
            let is_running =
                crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
            (inst.name.clone(), inst.data_dir.clone(), pids, is_running)
        } else {
            (target_id.to_string(), "unknown".to_string(), vec![], false)
        };

        crate::modules::logger::log_info(&format!(
            "[PROMPT_LIVENESS_PROBE] instance_id=\"{}\" name=\"{}\" data_dir=\"{}\" alive={} pids={:?} workspaces={}",
            target_id, inst_name, data_dir, is_alive, pids, projects.len()
        ));
    }
}
