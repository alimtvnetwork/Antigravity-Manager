//! Repo DB: tree project liveness evaluation

use super::dispatch::parse_flexible_timestamp;
use super::models::{AgmConversationNode, RunningProject};
use super::project_queries::normalize_path_for_compare;
use super::state::{get_active_agy_workers, get_memory_prompts_map};
use std::collections::HashSet;

pub(crate) fn evaluate_project_liveness(
    ctx: &super::tree_nodes::TreeNodeCtx,
    proj: &RunningProject,
    project_key: &str,
    is_inst_alive: bool,
    conv_nodes: &[AgmConversationNode],
) -> (bool, String) {
    let registry = ctx.registry;
    let now = ctx.now;
    let has_active_conv = conv_nodes.iter().any(|c| {
        if !c.is_running {
            return false;
        }
        let conv_ts = parse_flexible_timestamp(&c.last_modified);
        conv_ts > 0 && (now - conv_ts <= 45)
    });
    let has_conv_nodes = !conv_nodes.is_empty();

    if !is_inst_alive {
        (
            false,
            "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
                .to_string(),
        )
    } else if has_conv_nodes {
        // Concrete conversation summaries exist on disk: this is verified empirical truth
        if has_active_conv {
            (
                true,
                "ACTIVE_IN_FLIGHT_TASKS: active non-idle conversation turn detected -> marked running".to_string(),
            )
        } else {
            (
                false,
                "IDLE_EXPLICIT_STATUS: verified conversation summaries on disk are all idle -> marked idle".to_string(),
            )
        }
    } else {
        // Guarded Empty Workspace Fallback: conversation summaries on disk are absent or filtered out as ghosts/empty
        let has_valid_ws = proj
            .workspace_storage_path
            .as_deref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(false);

        if !has_valid_ws {
            (
                false,
                "IDLE_EMPTY_WORKSPACE: workspace storage folder not found -> marked idle"
                    .to_string(),
            )
        } else {
            let inst_expected_data_dir =
                if proj.instance_id == "default" || proj.instance_id == "__default__" {
                    crate::modules::instance::get_default_antigravity_data_dir()
                } else if let Some(inst) = registry
                    .instances
                    .iter()
                    .find(|i| i.id == proj.instance_id || i.name == proj.instance_id)
                {
                    PathBuf::from(&inst.data_dir)
                } else {
                    PathBuf::from(&proj.instance_id)
                };

            let norm_inst_data_dir =
                normalize_path_for_compare(&inst_expected_data_dir.to_string_lossy());
            let norm_ws_path = proj
                .workspace_storage_path
                .as_deref()
                .map(normalize_path_for_compare)
                .unwrap_or_default();

            let is_ws_owned_by_instance = !norm_inst_data_dir.is_empty()
                && !norm_ws_path.is_empty()
                && norm_ws_path.starts_with(&norm_inst_data_dir);

            if !is_ws_owned_by_instance {
                (
                    false,
                    "IDLE_EMPTY_WORKSPACE: workspace storage folder belongs to different instance -> marked idle".to_string(),
                )
            } else {
                // Only consider running if there is concrete active process evidence:
                // 1) Active in-memory prompt within last 60 seconds
                let has_active_mem = if let Ok(map) = get_memory_prompts_map().lock() {
                    map.values().any(|p| {
                        let matches_inst =
                            if proj.instance_id == "default" || proj.instance_id == "__default__" {
                                p.instance_id == "default" || p.instance_id == "__default__"
                            } else {
                                p.instance_id == proj.instance_id
                            };
                        let matches_proj = p.project_id == proj.repo_path
                            || p.repo_path == proj.repo_path
                            || p.project_id == project_key
                            || p.repo_path == project_key;
                        matches_inst
                            && matches_proj
                            && p.status == "running"
                            && (now - p.updated_at <= 45)
                    })
                } else {
                    false
                };

                // 2) Or active worker registered for this path
                let has_active_worker = if let Ok(workers) = get_active_agy_workers().lock() {
                    let clean_target = normalize_path_for_compare(&proj.repo_path);
                    workers.iter().any(|(key, &wpid)| {
                        let (w_inst, w_path) = match key.split_once(':') {
                            Some((inst, path)) => (inst, path),
                            None => ("", key.as_str()),
                        };
                        let is_inst_match =
                            if proj.instance_id == "default" || proj.instance_id == "__default__" {
                                w_inst == "default" || w_inst == "__default__"
                            } else {
                                w_inst.eq_ignore_ascii_case(&proj.instance_id)
                            };
                        let is_path_match = !clean_target.is_empty()
                            && normalize_path_for_compare(w_path) == clean_target;
                        if is_inst_match && is_path_match {
                            let mut sys = sysinfo::System::new();
                            let target_pid = sysinfo::Pid::from_u32(wpid);
                            sys.refresh_processes_specifics(
                                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                                sysinfo::ProcessRefreshKind::new()
                                    .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
                            );
                            sys.process(target_pid).is_some()
                        } else {
                            false
                        }
                    })
                } else {
                    false
                };

                if has_active_mem || has_active_worker {
                    (
                        true,
                        "ACTIVE_IN_FLIGHT_TASKS: active prompt/worker detected for empty workspace -> marked running".to_string(),
                    )
                } else {
                    (
                        false,
                        "IDLE_EMPTY_WORKSPACE: empty or filtered workspace summaries on disk -> marked idle".to_string(),
                    )
                }
            }
        }
    }
}
