//! PID protection: sparing other instances' processes.
use super::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// True when this process is another instance and must not be closed.
pub fn should_spare_pid(
    pid: u32,
    args: &str,
    exe: &str,
    name: &str,
    protected_pids: &[u32],
    markers: &[String],
) -> bool {
    if pid > 0 && protected_pids.contains(&pid) {
        return true;
    }
    let args = args.to_lowercase().replace('\\', "/");
    let exe = exe.to_lowercase().replace('\\', "/");
    let name = name.to_lowercase();
    markers.iter().any(|marker| {
        let marker = marker.trim().to_lowercase().replace('\\', "/");
        !marker.is_empty()
            && (args.contains(&marker) || exe.contains(&marker) || name.contains(&marker))
    })
}

/// Saved process ids and path markers for every instance except the one being switched.
pub fn other_instance_protection(except_id: &str) -> (Vec<u32>, Vec<String>) {
    let Ok(registry) = load_registry() else {
        return (Vec::new(), Vec::new());
    };
    let mut pids = Vec::new();
    let mut markers = Vec::new();
    for inst in registry.instances {
        let same = inst.id == except_id
            || (except_id == "default" && (inst.is_default || inst.id == "default"));
        if same {
            continue;
        }
        if let Some(pid) = inst.pid.filter(|pid| *pid > 0) {
            if !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        if let Some(pid) = get_instance_saved_pid(&inst.id) {
            if pid > 0 && !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        let id = inst.id.to_lowercase();
        if !id.is_empty() && id != "default" {
            markers.push(format!("/instances/{}/", id));
            markers.push(format!("antigravity-{}", id));
        }
        if !inst.is_default && inst.id != "default" {
            let dir = inst
                .data_dir
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_lowercase();
            if !dir.is_empty() {
                markers.push(dir);
            }
        }
    }
    if let Ok(cache) = INSTANCE_PROCESS_CACHE.read() {
        for (id, record) in cache.iter() {
            let same = id == except_id || (except_id == "default" && id == "default");
            if !same && record.is_alive {
                for &pid in &record.pids {
                    if pid > 0 && !pids.contains(&pid) {
                        pids.push(pid);
                    }
                }
                if let Some(primary) = record.primary_pid {
                    if primary > 0 && !pids.contains(&primary) {
                        pids.push(primary);
                    }
                }
            }
        }
    }
    (pids, markers)
}
