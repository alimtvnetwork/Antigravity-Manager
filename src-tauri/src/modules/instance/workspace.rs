//! Workspace folder discovery, snapshot, and migration.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Collect all existing workspace folder paths bound to `instance_id` from `repo_db` and `User/workspaceStorage/*/workspace.json`.
pub fn get_instance_workspace_folders(instance_id: &str, data_dir: &str) -> Vec<String> {
    let mut folders: Vec<String> = Vec::new();
    let mut seen_norm: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push_folder = |raw_path: &str| {
        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            return;
        }
        let p = Path::new(trimmed);
        if !p.exists() {
            return;
        }
        let norm = trimmed
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase();
        if seen_norm.insert(norm) {
            folders.push(trimmed.to_string());
        }
    };

    // 1. Live workspace discovery for this instance
    if let Ok(detected) = crate::modules::repo_db::detect_running_projects(instance_id) {
        for proj in detected {
            push_folder(&proj.repo_path);
        }
    }

    // 2. Persisted running_projects in repo_db for this instance
    if let Ok(all_projs) = crate::modules::repo_db::list_running_projects() {
        for proj in all_projs {
            let matches_inst = proj.instance_id == instance_id
                || ((instance_id == "default" || instance_id == "__default__")
                    && (proj.instance_id == "default" || proj.instance_id == "__default__"));
            if matches_inst {
                push_folder(&proj.repo_path);
            }
        }
    }

    // 3. Direct scan of `<data_dir>/User/workspaceStorage/*/workspace.json`
    if !data_dir.trim().is_empty() {
        let ws_root = PathBuf::from(data_dir)
            .join("User")
            .join("workspaceStorage");
        if ws_root.exists() {
            if let Ok(entries) = fs::read_dir(&ws_root) {
                for entry in entries.flatten() {
                    let ws_json = entry.path().join("workspace.json");
                    if ws_json.exists() {
                        if let Ok(content) = fs::read_to_string(&ws_json) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(uri) = val.get("folder").and_then(|v| v.as_str()) {
                                    let decoded =
                                        crate::modules::repo_db::decode_uri_to_path_pub(uri);
                                    push_folder(&decoded);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    folders.truncate(8);
    folders
}

/// Resolve workspace roots for an instance by its ID
pub fn get_instance_workspace_paths(instance_id: &str) -> Vec<String> {
    let registry = load_registry().unwrap_or_default();
    let data_dir = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .map(|i| i.data_dir.as_str())
        .unwrap_or("");
    get_instance_workspace_folders(instance_id, data_dir)
}

/// Restore and re-inject prompts for a freshly restarted instance
pub fn restore_and_inject_prompts_for_instance(
    instance_id: &str,
    _workspace_roots: &[String],
) -> Result<usize, String> {
    // Justification: background restore; prompts are re-restored on the next launch
    crate::error::record_ignored(
        crate::modules::backup_prompts_db::restore_running_prompts(Some(instance_id), false, None),
        "restore running prompts",
    );
    let resent =
        crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20)
            .unwrap_or_default();
    let dispatched = crate::modules::repo_db::dispatch_running_prompts(instance_id).unwrap_or(0);
    // Justification: intentionally discards the ensured-goals count; the function is infallible and the scheduler re-verifies on its next tick
    let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
    Ok(resent.len() + dispatched)
}

/// Snapshot all currently open or registered workspace project directories for an instance.
pub fn snapshot_active_workspaces(instance_id: &str) -> Vec<String> {
    let registry = match load_registry() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) else {
        return Vec::new();
    };

    let folders = get_instance_workspace_folders(instance_id, &inst.data_dir);
    if !folders.is_empty() {
        return folders;
    }

    // Fallback: query active projects in repo_db
    if let Ok(projects) = crate::modules::repo_db::list_running_projects() {
        let is_target_default = instance_id == "default" || instance_id == "__default__";
        let mut fallback_folders = Vec::new();
        for p in projects {
            let matches = p.instance_id == instance_id
                || (is_target_default
                    && (p.instance_id == "default" || p.instance_id == "__default__"));
            if matches && Path::new(&p.repo_path).exists() {
                fallback_folders.push(p.repo_path);
            }
        }
        if !fallback_folders.is_empty() {
            fallback_folders.truncate(8);
            return fallback_folders;
        }
    }

    Vec::new()
}

/// Migrate active workspaces from a depleted/source instance to a target candidate instance.
/// Seeds `User/workspaceStorage/<ws-id>/workspace.json` in target instance data directory
/// and registers projects in `repo_db`.
pub fn migrate_instance_workspaces(
    from_instance_id: &str,
    to_instance_id: &str,
) -> Result<Vec<String>, String> {
    if from_instance_id == to_instance_id {
        return Ok(snapshot_active_workspaces(from_instance_id));
    }

    let workspaces = snapshot_active_workspaces(from_instance_id);
    if workspaces.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[WorkspaceMigration] No active workspaces found on source '{}' to migrate to '{}'",
            from_instance_id, to_instance_id
        ));
        return Ok(Vec::new());
    }

    crate::modules::logger::log_info(&format!(
        "[WorkspaceMigration] Migrating {} workspace(s) from '{}' to '{}': {:?}",
        workspaces.len(),
        from_instance_id,
        to_instance_id,
        workspaces
    ));

    for ws_path in &workspaces {
        // Justification: workspace-assignment bookkeeping; re-attempted on the next dispatch
        crate::error::record_ignored(
            assign_project_to_instance(to_instance_id, ws_path),
            "assign project to instance",
        );
    }

    Ok(workspaces)
}
