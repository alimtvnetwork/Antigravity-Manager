//! Project assignment and running-prompt channel utilities.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Explicitly bind/assign one or more project workspace directories to a specific instance (`instance_id`, `#seq`, or name).
/// Seeds `<instance.data_dir>/User/workspaceStorage/<id>/workspace.json` and registers the project in `repo_db`.
pub fn assign_project_to_instance(instance_spec: &str, repo_path: &str) -> Result<String, String> {
    let resolved_id = resolve_instance_id(instance_spec)?;
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_spec))?;

    let clean_path = PathBuf::from(repo_path.trim());
    if !clean_path.exists() {
        return Err(format!(
            "Project directory '{}' does not exist on disk",
            repo_path.trim()
        ));
    }
    let canonical_str = clean_path.to_string_lossy().to_string();
    let repo_name = clean_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());

    // 1. Seed workspaceStorage/<workspace_id>/workspace.json inside the target instance's data_dir
    let mut hash: u64 = 14695981039346656037;
    for b in canonical_str.to_lowercase().replace('\\', "/").bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    let ws_id = format!(
        "{}-{:08x}",
        repo_name.to_lowercase(),
        (hash & 0xFFFF_FFFF) as u32
    );
    let ws_dir = PathBuf::from(&inst.data_dir)
        .join("User")
        .join("workspaceStorage")
        .join(&ws_id);
    fs::create_dir_all(&ws_dir)
        .map_err(|e| format!("Failed to create instance workspaceStorage dir: {}", e))?;

    let normalized_slash = canonical_str.replace('\\', "/");
    let folder_uri = if normalized_slash.starts_with('/') {
        format!("file://{}", normalized_slash)
    } else {
        format!("file:///{}", normalized_slash)
    };
    let ws_json_payload = serde_json::json!({
        "folder": folder_uri
    });
    let ws_content = serde_json::to_string_pretty(&ws_json_payload).unwrap_or_default();
    fs::write(ws_dir.join("workspace.json"), &ws_content)
        .map_err(|e| format!("Failed to write workspace.json: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let appdata_ws_dir = PathBuf::from(&inst.data_dir)
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage")
            .join(&ws_id);
        if fs::create_dir_all(&appdata_ws_dir).is_ok() {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(
                fs::write(appdata_ws_dir.join("workspace.json"), &ws_content),
                "write file",
            );
        }

        if let Ok(inst_home) = get_instance_home_dir(&inst.id) {
            let home_ws_dir = inst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage")
                .join(&ws_id);
            if fs::create_dir_all(&home_ws_dir).is_ok() {
                // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                crate::error::record_ignored(
                    fs::write(home_ws_dir.join("workspace.json"), &ws_content),
                    "write file",
                );
            }
        }
    }

    // 2. Register in repo_db running_projects & sequence table
    // Justification: background detection; re-runs on the next refresh tick
    crate::error::record_ignored(
        crate::modules::repo_db::detect_running_projects(&inst.id),
        "detect running projects",
    );

    Ok(format!(
        "✅ Assigned project '{}' ({}) to instance '{}' (#{} {})",
        repo_name,
        canonical_str,
        inst.id,
        inst.seq_num.unwrap_or(1),
        inst.name
    ))
}

/// Wait until the instance process exists, then give the prompt channel a moment to accept a send.
/// Uses adaptive retry logic up to 15s with exponential backoff rather than a hard timeout.
pub fn wait_for_instance_prompt_channel(instance_id: &str) {
    let registry = match load_registry() {
        Ok(reg) => reg,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Prompt channel wait skipped for '{}': {}",
                instance_id, err
            ));
            return;
        }
    };
    let Some(inst) = registry
        .instances
        .iter()
        .find(|item| item.id == instance_id)
    else {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Prompt channel wait skipped; instance '{}' is not registered",
            instance_id
        ));
        return;
    };
    let start_time = std::time::Instant::now();
    let max_wait_duration = std::time::Duration::from_secs(15);
    let mut current_interval_ms = 100u64;
    let max_interval_ms = 1000u64;

    loop {
        let pids = find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        let running = !pids.is_empty()
            || (inst.is_default && crate::modules::process::is_antigravity_running(None));
        if running {
            crate::modules::logger::log_info(&format!(
                "[Instance] Prompt channel ready for '{}' ({} pids after {:?}); allowing 6s stabilization window before inject",
                instance_id,
                pids.len(),
                start_time.elapsed()
            ));
            std::thread::sleep(std::time::Duration::from_millis(6000));
            return;
        }
        if start_time.elapsed() >= max_wait_duration {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Prompt channel wait timed out after 15s for '{}'; injecting anyway",
                instance_id
            ));
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(current_interval_ms));
        current_interval_ms = (current_interval_ms * 3 / 2).min(max_interval_ms);
    }
}

/// Force refresh process cache, enumerate all instances from registry, identify live PIDs, cache them, and return actively running count
pub fn scan_and_cache_all_running_instances() -> usize {
    force_refresh_process_cache();
    let registry = match load_registry() {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let mut running_count = 0;
    let now = chrono::Utc::now().timestamp();
    for inst in &registry.instances {
        let is_default = inst.is_default || inst.id == "default";
        let mut pids = find_pids_for_data_dir(&inst.data_dir, is_default);
        if let Some(saved_pid) = inst.pid.or_else(|| get_instance_saved_pid(&inst.id)) {
            let is_saved_alive =
                saved_pid > 0 && is_pid_alive_targeted(saved_pid) && saved_pid_matches(saved_pid);
            if is_saved_alive && !pids.contains(&saved_pid) {
                pids.push(saved_pid);
            }
        }
        if pids.is_empty() && is_default {
            pids = crate::modules::process::get_antigravity_pids(None);
        }
        let is_running = !pids.is_empty();
        let primary_pid = pids.first().copied();
        if is_running {
            running_count += 1;
        }
        let record = InstanceProcessRecord {
            instance_id: inst.id.clone(),
            pid: primary_pid.unwrap_or(0),
            pids: pids.clone(),
            primary_pid,
            data_dir: inst.data_dir.clone(),
            launched_at: now,
            last_verified_at: now,
            is_alive: is_running,
            command_line: None,
        };
        if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
            cache.insert(inst.id.clone(), record.clone());
            if !inst.name.is_empty() && inst.name != inst.id {
                cache.insert(inst.name.clone(), record);
            }
        }
    }
    running_count
}

/// Count total distinct Antigravity IDE instances currently running on this machine
pub fn get_instance_running_process_count() -> usize {
    scan_and_cache_all_running_instances()
}

/// Initialize and warm up the in-memory smart process cache on application startup.
/// Refreshes the OS process table, cross-checks registered instances against SQLite
/// `instance_processes` and live OS PIDs, and caches all active instances.
pub fn warm_up_smart_process_cache() -> usize {
    crate::modules::logger::log_info(
        "[SmartProcessCache] Starting startup cache warm-up and cross-checking...",
    );
    let running_count = scan_and_cache_all_running_instances();
    crate::modules::logger::log_info(&format!(
        "[SmartProcessCache] Startup warm-up complete. Discovered and cached {} running Antigravity IDE instances.",
        running_count
    ));
    running_count
}
