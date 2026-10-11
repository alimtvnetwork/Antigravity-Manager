//! Instance listing and status queries.
use super::*;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// List all instances with live running status
pub fn list_instances() -> Result<Vec<InstanceStatus>, String> {
    let registry = load_registry()?;
    let mut statuses = Vec::new();

    let cached_procs = get_cached_antigravity_processes();
    if cached_procs.is_empty() {
        for config in registry.instances {
            statuses.push(InstanceStatus {
                config,
                is_running: false,
                pid: None,
                memory_mb: None,
            });
        }
        return Ok(statuses);
    }

    // Pass 1: Resolve PIDs for each instance from cached process table and collect target PIDs for targeted sysinfo refresh
    let mut inst_pids: Vec<(InstanceConfig, Vec<u32>, Option<u32>)> = Vec::new();
    let mut pids_to_refresh: Vec<sysinfo::Pid> = Vec::new();

    for config in registry.instances {
        let is_default_inst = config.is_default || config.id == "default";
        let pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
        let saved_pid = config.pid.or_else(|| get_instance_saved_pid(&config.id));

        if let Some(sp) = saved_pid {
            pids_to_refresh.push(sysinfo::Pid::from_u32(sp));
        }
        if let Some(&first) = pids.first() {
            pids_to_refresh.push(sysinfo::Pid::from_u32(first));
        }

        inst_pids.push((config, pids, saved_pid));
    }

    // Refresh strictly the targeted candidate PIDs instead of enumerating the entire OS process table
    let mut system = System::new();
    if !pids_to_refresh.is_empty() {
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&pids_to_refresh),
            sysinfo::ProcessRefreshKind::new()
                .with_exe(sysinfo::UpdateKind::OnlyIfNotSet)
                .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
                .with_memory(),
        );
    }

    // Pass 2: Finalize instance statuses with verified PIDs and memory usage
    for (config, mut pids, saved_pid) in inst_pids {
        let is_default_inst = config.is_default || config.id == "default";
        if let Some(sp) = saved_pid {
            if let Some(proc) = system.process(sysinfo::Pid::from_u32(sp)) {
                let proc_name = proc.name().to_string_lossy().to_lowercase();
                let proc_exe = proc
                    .exe()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_antigravity =
                    proc_name.contains("antigravity") || proc_exe.contains("antigravity");
                if is_antigravity && !pids.contains(&sp) {
                    let args_str = proc
                        .cmd()
                        .iter()
                        .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
                    let clean_data = norm_data.trim_end_matches('/');
                    if is_default_inst || args_str.contains(clean_data) {
                        pids.push(sp);
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
