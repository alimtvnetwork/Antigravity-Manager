//! Project copying between instances.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Copies workspace projects and recent paths from one instance to another.
/// Copies workspaceStorage directories, merges storage.json openedPathsList,
/// copies state.vscdb history rows, and clones repo_db project rows.
pub fn copy_instance_projects(from_id: &str, to_id: &str) -> Result<usize, String> {
    let from_resolved = resolve_instance_id(from_id)?;
    let to_resolved = resolve_instance_id(to_id)?;
    if from_resolved == to_resolved {
        return Ok(0);
    }
    let registry = load_registry()?;
    let from_inst = registry
        .instances
        .iter()
        .find(|i| i.id == from_resolved)
        .ok_or_else(|| format!("Source instance '{}' not found", from_id))?
        .clone();
    let to_inst = registry
        .instances
        .iter()
        .find(|i| i.id == to_resolved)
        .ok_or_else(|| format!("Destination instance '{}' not found", to_id))?
        .clone();

    // 1. Locate source workspaceStorage directories
    let mut src_ws_dirs = Vec::new();
    let src_data = PathBuf::from(&from_inst.data_dir);
    let p1 = src_data.join("User").join("workspaceStorage");
    if p1.exists() {
        src_ws_dirs.push(p1);
    }
    let p2 = src_data.join("workspaceStorage");
    if p2.exists() && !src_ws_dirs.contains(&p2) {
        src_ws_dirs.push(p2);
    }
    #[cfg(target_os = "windows")]
    {
        let p3 = src_data
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage");
        if p3.exists() && !src_ws_dirs.contains(&p3) {
            src_ws_dirs.push(p3);
        }
        if let Ok(src_home) = get_instance_home_dir(&from_inst.id) {
            let p4 = src_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            if p4.exists() && !src_ws_dirs.contains(&p4) {
                src_ws_dirs.push(p4);
            }
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p5 = PathBuf::from(appdata)
                    .join("Antigravity")
                    .join("User")
                    .join("workspaceStorage");
                if p5.exists() && !src_ws_dirs.contains(&p5) {
                    src_ws_dirs.push(p5);
                }
            }
        }
    }

    // 2. Prepare destination workspaceStorage directories
    let dst_data = PathBuf::from(&to_inst.data_dir);
    let dst_ws_main = dst_data.join("User").join("workspaceStorage");
    let mut dst_ws_dirs = vec![dst_ws_main];

    #[cfg(target_os = "windows")]
    {
        let p_dst_appdata = dst_data
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage");
        dst_ws_dirs.push(p_dst_appdata);

        if let Ok(dst_home) = get_instance_home_dir(&to_inst.id) {
            let p_dst_home = dst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            dst_ws_dirs.push(p_dst_home);
        }
    }

    for d in &dst_ws_dirs {
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(d), "create directory");
    }

    let mut copied_count = 0usize;
    let mut copied_names = std::collections::HashSet::new();

    for src_ws in &src_ws_dirs {
        if let Ok(entries) = fs::read_dir(src_ws) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy().to_string();
                        if copied_names.insert(name_str.clone()) {
                            copied_count += 1;
                        }
                        for dst_ws in &dst_ws_dirs {
                            let target = dst_ws.join(&name);
                            if let Err(e) = copy_dir_recursive(&entry.path(), &target) {
                                crate::modules::logger::log_warn(&format!(
                                    "[Instance] Warning: Failed to copy workspace folder {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Clone repo rows in repo_db
    // Justification: background detection; re-runs on the next refresh tick
    crate::error::record_ignored(
        crate::modules::repo_db::detect_running_projects(&from_inst.id),
        "detect running projects",
    );
    if let Ok(rows) = crate::modules::repo_db::clone_instance_repo_rows(&from_inst.id, &to_inst.id)
    {
        if copied_count == 0 && rows > 0 {
            copied_count = rows;
        }
    }

    // 4. Merge recent paths in storage.json
    merge_storage_json_recent_paths(&from_inst, &to_inst);

    // 5. Merge recent paths from state.vscdb
    merge_state_vscdb_recent_paths(&from_inst, &to_inst);

    // Justification: post-clone hygiene; stale summaries are regenerated on demand
    crate::error::record_ignored(
        sanitize_cloned_instance_summaries(&to_inst.id),
        "sanitize cloned summaries",
    );

    crate::modules::logger::log_info(&format!(
        "[Instance] Copied {} workspace projects from '{}' to '{}'",
        copied_count, from_id, to_id
    ));

    Ok(copied_count)
}

/// Scans registered instances and checks if an instance has executable_path matching exe_path,
/// or if exe_path is inside an instance's directory.
pub fn find_instance_by_executable(exe_path: &str) -> Option<String> {
    let clean_exe = exe_path.trim().replace('\\', "/").to_lowercase();
    if clean_exe.is_empty() {
        return None;
    }
    let instances = list_instances().ok()?;

    // 1. Direct match on executable_path
    for inst in &instances {
        if let Some(ref ep) = inst.config.executable_path {
            let clean_ep = ep.trim().replace('\\', "/").to_lowercase();
            if clean_ep == clean_exe {
                return Some(inst.config.id.clone());
            }
        }
    }

    // 2. Check if exe_path contains instance id pattern (e.g. Antigravity-<id>.exe or antigravity-<id>)
    for inst in &instances {
        let id_lower = inst.config.id.to_lowercase();
        let target_name1 = format!("antigravity-{}", id_lower);
        let target_name2 = format!("launch-{}.cmd", id_lower);
        if clean_exe.contains(&target_name1) || clean_exe.contains(&target_name2) {
            return Some(inst.config.id.clone());
        }
    }

    // 3. Check if exe_path is inside an instance's directory tree
    for inst in &instances {
        let data_dir_clean = inst
            .config
            .data_dir
            .trim()
            .replace('\\', "/")
            .to_lowercase();
        if !data_dir_clean.is_empty() && clean_exe.starts_with(&data_dir_clean) {
            return Some(inst.config.id.clone());
        }

        if let Ok(home) = get_instance_home_dir(&inst.config.id) {
            let home_clean = home.to_string_lossy().replace('\\', "/").to_lowercase();
            if clean_exe.starts_with(&home_clean) {
                return Some(inst.config.id.clone());
            }
        }

        if let Ok(instances_root) = get_instances_dir() {
            let inst_root = instances_root
                .join(&inst.config.id)
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase();
            if clean_exe.starts_with(&inst_root) {
                return Some(inst.config.id.clone());
            }
        }
    }

    // 4. If base executable matches and default instance exists, return default
    if let Ok(base_exe) = crate::modules::process::detect_antigravity_with_diagnostics(None) {
        let base_clean = base_exe.to_string_lossy().replace('\\', "/").to_lowercase();
        if base_clean == clean_exe {
            return Some("default".to_string());
        }
    }

    None
}
