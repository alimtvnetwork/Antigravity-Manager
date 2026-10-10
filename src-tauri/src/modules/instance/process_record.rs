//! In-memory process records and the smart process caches.
use super::*;
use crate::error::AppError;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstanceProcessRecord {
    pub instance_id: String,
    pub pid: u32,
    pub pids: Vec<u32>,
    pub primary_pid: Option<u32>,
    pub data_dir: String,
    pub launched_at: i64,
    pub last_verified_at: i64,
    pub is_alive: bool,
    pub command_line: Option<String>,
}

pub type InstanceProcessCacheItem = InstanceProcessRecord;

pub static INSTANCE_PROCESS_CACHE: Lazy<Arc<RwLock<HashMap<String, InstanceProcessRecord>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub static SMART_PROCESS_CACHE: Lazy<Arc<RwLock<HashMap<String, InstanceProcessRecord>>>> =
    Lazy::new(|| INSTANCE_PROCESS_CACHE.clone());

#[cfg(test)]
pub static MOCK_PID_ALIVE: Lazy<std::sync::RwLock<Option<std::collections::HashSet<u32>>>> =
    Lazy::new(|| std::sync::RwLock::new(None));

/// Check if a PID is alive in the OS process table
pub fn is_pid_alive_os(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(unix)]
    {
        let ret = unsafe { libc::kill(pid as libc::pid_t, 0) };
        if ret == 0 {
            return true;
        }
        let err = std::io::Error::last_os_error().raw_os_error();
        err == Some(libc::EPERM)
    }
    #[cfg(not(unix))]
    {
        let mut sys = sysinfo::System::new();
        let target_pid = sysinfo::Pid::from_u32(pid);
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[target_pid]),
            sysinfo::ProcessRefreshKind::new(),
        );
        sys.process(target_pid).is_some()
    }
}

/// Check if a specific PID is alive in the operating system in sub-millisecond time.
pub fn is_pid_alive_targeted(pid: u32) -> bool {
    #[cfg(test)]
    if let Ok(guard) = MOCK_PID_ALIVE.read() {
        if let Some(set) = guard.as_ref() {
            return set.contains(&pid);
        }
    }
    is_pid_alive_os(pid)
}

/// Get cached instance process record if present
pub fn get_cached_instance_process(instance_id: &str) -> Option<InstanceProcessRecord> {
    let cache = INSTANCE_PROCESS_CACHE.read().ok()?;
    cache.get(instance_id).cloned()
}

/// Invalidate process cache entry for a specific instance and its canonical ID
pub fn invalidate_instance_process_cache(instance_id: &str) {
    if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
        cache.remove(instance_id);
        if let Ok(canonical_id) = resolve_instance_id(instance_id) {
            cache.remove(&canonical_id);
        }
    }
}

/// Update cached process record for an instance
pub fn update_cached_instance_process(record: InstanceProcessRecord) {
    if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
        cache.insert(record.instance_id.clone(), record);
    }
}

/// Query the in-memory smart process cache or discover living PID from the OS.
pub fn get_or_detect_instance_process(instance_id: &str) -> Option<InstanceProcessRecord> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let (is_running, primary_pid, pids) = is_instance_process_running_smart(&canonical_id);
    if is_running {
        get_cached_instance_process(&canonical_id).or_else(|| {
            let now = chrono::Utc::now().timestamp();
            Some(InstanceProcessRecord {
                instance_id: canonical_id.clone(),
                pid: primary_pid.unwrap_or(0),
                pids,
                primary_pid,
                data_dir: String::new(),
                launched_at: now,
                last_verified_at: now,
                is_alive: true,
                command_line: None,
            })
        })
    } else {
        None
    }
}

/// Guarantee that the instance is running prior to prompt dispatch.
/// If already running, attempts focus but NEVER terminates or relaunches.
/// Only if confirmed dead across the OS, cold launches targeted at workspace.
pub fn ensure_instance_running_for_dispatch(
    instance_id: &str,
    workspace_path: Option<&str>,
) -> Result<u32, AppError> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let (is_running, pid) = ensure_instance_running_smart(&canonical_id, workspace_path)?;
    if is_running {
        Ok(pid.unwrap_or(0))
    } else {
        Err(AppError::Process("Failed to launch instance".to_string()))
    }
}
