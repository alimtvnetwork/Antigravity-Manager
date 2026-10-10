//! PID identity, liveness, and refresh utilities.
use super::*;
use once_cell::sync::Lazy;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// A saved PID matches when that one process is still Antigravity. This does not scan the process table.
pub fn process_identity_matches(name: &str, exe: &str) -> bool {
    let name = name.to_lowercase();
    let exe = exe.to_lowercase().replace('\\', "/");

    // Fast reject known non-IDE developer tools
    if crate::modules::process::is_non_ide_binary(&name, &exe, "") {
        return false;
    }

    let blocked = name.contains("agm")
        || exe.contains("agm")
        || name.contains("webview")
        || exe.contains("webview");
    if blocked {
        return false;
    }

    // Positive identity check: executable name itself must reflect Antigravity
    let is_antigravity_name =
        name.contains("antigravity") || name == "apprun" || name.starts_with("code");

    let is_antigravity_exe = exe.ends_with("/antigravity.exe")
        || exe.ends_with("/antigravity")
        || exe.contains("/antigravity-")
        || exe.contains("antigravity ide.exe")
        || exe.contains(".app/contents/macos");

    is_antigravity_name || is_antigravity_exe
}

pub fn saved_pid_matches(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(test)]
    if let Ok(guard) = MOCK_PID_ALIVE.read() {
        if let Some(set) = guard.as_ref() {
            return set.contains(&pid);
        }
    }
    let mut sys = System::new();
    let target_pid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target_pid]),
        sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );
    let Some(proc) = sys.process(target_pid) else {
        return false;
    };
    let proc_name = proc.name().to_string_lossy().to_string();
    let proc_exe = proc
        .exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    process_identity_matches(&proc_name, &proc_exe)
}

/// Full process-table rebuild interval. Default 10 minutes. Never under 3 minutes, never over 20.
pub fn pid_refresh_interval_seconds(configured: u32) -> u32 {
    configured.clamp(180, 1200)
}

static LAST_PID_CACHE_REFRESH: Lazy<std::sync::Mutex<std::time::Instant>> = Lazy::new(|| {
    std::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(3600))
});

pub(crate) fn force_refresh_process_cache() {
    if let Ok(mut cache) = PROCESS_SCAN_CACHE.lock() {
        cache.0 = std::time::Instant::now() - std::time::Duration::from_secs(3600);
        cache.1.clear();
    }
    // Justification: intentionally discards the cached Vec; the call's only purpose is its side effect of refreshing the process cache
    let _ = get_cached_antigravity_processes();
}

/// Rebuild the PID table and write a PID only when the saved one no longer matches.
pub fn refresh_saved_instance_pids() {
    force_refresh_process_cache();
    let Ok(registry) = load_registry() else {
        return;
    };
    for inst in registry.instances {
        let saved = inst.pid.or_else(|| get_instance_saved_pid(&inst.id));
        if saved.map(saved_pid_matches).unwrap_or(false) {
            continue;
        }
        let is_default = inst.is_default || inst.id == "default";
        let pids = find_pids_for_data_dir(&inst.data_dir, is_default);
        if let Some(pid) = pids.first().copied() {
            // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
            crate::error::record_ignored(
                record_instance_pid(&inst.id, pid, &inst.data_dir),
                "record instance pid",
            );
        }
    }
}

pub fn refresh_pid_cache_if_due(configured_seconds: u32) -> bool {
    let interval = pid_refresh_interval_seconds(configured_seconds) as u64;
    let Ok(mut last) = LAST_PID_CACHE_REFRESH.lock() else {
        return false;
    };
    if last.elapsed() < std::time::Duration::from_secs(interval) {
        return false;
    }
    *last = std::time::Instant::now();
    drop(last);
    refresh_saved_instance_pids();
    true
}

/// Used only after a quota check is under the threshold and the saved PID does not match.
pub fn resolve_instance_pid_for_switch(
    instance_id: &str,
    data_dir: &str,
    is_default: bool,
) -> Option<u32> {
    if let Some(pid) = get_instance_saved_pid(instance_id) {
        if saved_pid_matches(pid) {
            return Some(pid);
        }
    }
    let pids = find_pids_for_data_dir(data_dir, is_default);
    let pid = pids.first().copied()?;
    // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
    crate::error::record_ignored(
        record_instance_pid(instance_id, pid, data_dir),
        "record instance pid",
    );
    Some(pid)
}

/// Running check trusts the PID saved at launch. Only when that PID no longer matches
/// (for example the macOS `open` wrapper exited) does it search once and save the real PID,
/// so the next check is a single-process lookup again.
pub fn is_instance_running(instance_id: &str, data_dir: &str, config_pid: Option<u32>) -> bool {
    let is_default = instance_id == "default" || instance_id == "__default__";
    let saved = config_pid.or_else(|| get_instance_saved_pid(instance_id));
    if let Some(pid) = saved {
        if saved_pid_matches(pid) {
            let pids = find_pids_for_data_dir(data_dir, is_default);
            if pids.contains(&pid) {
                return true;
            }
        }
    }
    let pids = find_pids_for_data_dir(data_dir, is_default);
    let Some(pid) = pids.first().copied() else {
        return false;
    };
    // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
    crate::error::record_ignored(
        record_instance_pid(instance_id, pid, data_dir),
        "record instance pid",
    );
    true
}
