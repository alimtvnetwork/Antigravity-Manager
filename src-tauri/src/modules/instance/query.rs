//! Instance listing and status queries.
use super::*;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// List all instances with live running status
pub fn list_instances() -> Result<Vec<InstanceStatus>, String> {
    let registry = load_registry()?;
    let mut statuses = Vec::new();

    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    for config in registry.instances {
        let is_default_inst = config.is_default || config.id == "default";
        let mut pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
        if let Some(saved_pid) = config.pid.or_else(|| get_instance_saved_pid(&config.id)) {
            if let Some(proc) = system.process(sysinfo::Pid::from_u32(saved_pid)) {
                let proc_name = proc.name().to_string_lossy().to_lowercase();
                let proc_exe = proc
                    .exe()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_antigravity =
                    proc_name.contains("antigravity") || proc_exe.contains("antigravity");
                if is_antigravity && !pids.contains(&saved_pid) {
                    let args_str = proc
                        .cmd()
                        .iter()
                        .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
                    let clean_data = norm_data.trim_end_matches('/');
                    if is_default_inst || args_str.contains(clean_data) {
                        pids.push(saved_pid);
                    }
                }
            }
        }
        let is_running = !pids.is_empty();
        let first_pid = pids.first().copied();

        let memory_mb = first_pid.and_then(|p| {
            system
                .process(sysinfo::Pid::from_u32(p))
                .map(|proc| proc.memory() as f64 / (1024.0 * 1024.0))
        });

        statuses.push(InstanceStatus {
            config,
            is_running,
            pid: first_pid,
            memory_mb,
        });
    }

    Ok(statuses)
}
