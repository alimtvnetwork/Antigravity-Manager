//! Smart process cache: vitality checks and OS re-scan.
use super::*;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub(crate) fn check_cached_pid_alive(
    instance_id: &str,
    canonical_id: &str,
) -> Option<(bool, Option<u32>, Vec<u32>)> {
    let mut entry = get_cached_instance_process(canonical_id)
        .or_else(|| get_cached_instance_process(instance_id))?;

    // 1. Verify primary PID
    if let Some(primary) = entry.primary_pid {
        if is_pid_alive_targeted(primary) && saved_pid_matches(primary) {
            return Some((true, Some(primary), entry.pids));
        }
    }

    // 2. Multi-PID Vitality Fallback: Check all surviving PIDs
    let surviving_pids: Vec<u32> = entry
        .pids
        .iter()
        .copied()
        .filter(|&pid| pid > 0 && is_pid_alive_targeted(pid) && saved_pid_matches(pid))
        .collect();

    if !surviving_pids.is_empty() {
        let promoted_pid = surviving_pids[0];
        crate::modules::logger::log_info(&format!(
            "[SmartProcessCache] Primary PID {:?} exited for instance '{}'. Promoted surviving child PID {} from {} candidates.",
            entry.primary_pid, canonical_id, promoted_pid, surviving_pids.len()
        ));

        // Promote new primary PID and update cache
        entry.primary_pid = Some(promoted_pid);
        entry.pid = promoted_pid;
        entry.pids = surviving_pids.clone();
        entry.last_verified_at = chrono::Utc::now().timestamp();
        entry.is_alive = true;

        if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
            cache.insert(canonical_id.to_string(), entry.clone());
            if instance_id != canonical_id {
                cache.insert(instance_id.to_string(), entry.clone());
            }
        }
        // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
        crate::error::record_ignored(
            record_instance_pid(canonical_id, promoted_pid, &entry.data_dir),
            "record instance pid",
        );
        return Some((true, Some(promoted_pid), surviving_pids));
    }

    // 3. Only invalidate cache when all candidate PIDs are dead
    invalidate_instance_process_cache(canonical_id);
    if instance_id != canonical_id {
        invalidate_instance_process_cache(instance_id);
    }
    None
}

fn scan_instance_os_pids(inst: &InstanceConfig, canonical_id: &str) -> Vec<u32> {
    let is_default_inst = inst.is_default || inst.id == "default" || canonical_id == "default";
    let mut pids = find_pids_for_data_dir(&inst.data_dir, is_default_inst);
    let saved = inst.pid.or_else(|| get_instance_saved_pid(canonical_id));
    if let Some(saved_pid) = saved {
        let is_saved_alive =
            saved_pid > 0 && is_pid_alive_targeted(saved_pid) && saved_pid_matches(saved_pid);
        if is_saved_alive && !pids.contains(&saved_pid) {
            pids.push(saved_pid);
        }
    }
    if pids.is_empty() && is_default_inst {
        pids = crate::modules::process::get_antigravity_pids(None);
    }
    pids
}

fn cache_live_instance_process(
    canonical_id: &str,
    alias: &str,
    name: &str,
    data_dir: &str,
    pids: Vec<u32>,
) -> (Option<u32>, Vec<u32>) {
    let primary_pid = pids.first().copied();
    let now = chrono::Utc::now().timestamp();
    let record = InstanceProcessRecord {
        instance_id: canonical_id.to_string(),
        pid: primary_pid.unwrap_or(0),
        pids: pids.clone(),
        primary_pid,
        data_dir: data_dir.to_string(),
        launched_at: now,
        last_verified_at: now,
        is_alive: !pids.is_empty(),
        command_line: None,
    };
    if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
        cache.insert(canonical_id.to_string(), record.clone());
        if alias != canonical_id {
            cache.insert(alias.to_string(), record.clone());
        }
        if !name.is_empty() && name != canonical_id {
            cache.insert(name.to_string(), record);
        }
    }
    if let Some(pid) = primary_pid {
        // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
        crate::error::record_ignored(
            record_instance_pid(canonical_id, pid, data_dir),
            "record instance pid",
        );
    }
    (primary_pid, pids)
}

/// Smart check if instance process is running, using cache with PID liveness verification
pub fn is_instance_process_running_smart(instance_id: &str) -> (bool, Option<u32>, Vec<u32>) {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    if let Some(res) = check_cached_pid_alive(instance_id, &canonical_id) {
        return res;
    }
    force_refresh_process_cache();
    let registry = match load_registry() {
        Ok(r) => r,
        Err(_) => return (false, None, Vec::new()),
    };
    let inst = registry.instances.iter().find(|i| {
        i.id == canonical_id
            || i.id == instance_id
            || i.name == instance_id
            || (canonical_id == "default" && i.is_default)
    });
    let (inst_name, inst_data_dir, pids) = if let Some(inst) = inst {
        let pids = scan_instance_os_pids(inst, &canonical_id);
        (inst.name.clone(), inst.data_dir.clone(), pids)
    } else if canonical_id == "default" || instance_id == "default" {
        let default_dir = get_default_antigravity_data_dir()
            .to_string_lossy()
            .to_string();
        let pids = crate::modules::process::get_antigravity_pids(None);
        ("default".to_string(), default_dir, pids)
    } else {
        return (false, None, Vec::new());
    };
    if pids.is_empty() {
        return (false, None, Vec::new());
    }
    let (primary_pid, pids) =
        cache_live_instance_process(&canonical_id, instance_id, &inst_name, &inst_data_dir, pids);
    (true, primary_pid, pids)
}

pub(crate) fn focus_running_instance(pids: &[u32], workspace_path: Option<&str>) {
    if let Some(ws) = workspace_path {
        let clean_ws = ws.trim_end_matches(['/', '\\']);
        let repo_name = Path::new(clean_ws)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(clean_ws);
        // Justification: intentionally discards the focus-success flag; focusing is a best-effort hint and a missed focus is immediately visible to the user
        let _ = crate::modules::process::focus_instance_workspace_window(pids, repo_name);
    } else {
        // Justification: intentionally discards the focus-success flag; focusing is a best-effort hint and a missed focus is immediately visible to the user
        let _ = crate::modules::process::focus_instance_pids(pids);
    }
}
