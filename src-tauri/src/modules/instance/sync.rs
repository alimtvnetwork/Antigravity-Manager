//! PID/quota synchronization logic.
use super::*;
use std::path::PathBuf;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Synchronize live running PID and quota for a single instance
pub async fn sync_instance_pid_and_quota_logic(
    instance_id: &str,
) -> Result<InstanceStatus, String> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let mut registry = load_registry()?;
    let idx = registry
        .instances
        .iter()
        .position(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", resolved_id))?;

    let is_default_inst =
        registry.instances[idx].is_default || registry.instances[idx].id == "default";
    let data_dir = registry.instances[idx].data_dir.clone();

    // 1. Inspect live running PIDs
    let pids = find_pids_for_data_dir(&data_dir, is_default_inst);
    let live_pid = pids.first().copied();
    let is_running = !pids.is_empty()
        || is_instance_running(
            &registry.instances[idx].id,
            &data_dir,
            registry.instances[idx].pid,
        );

    if let Some(pid) = live_pid {
        registry.instances[idx].pid = Some(pid);
        // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
        crate::error::record_ignored(
            record_instance_pid(&registry.instances[idx].id, pid, &data_dir),
            "record instance pid",
        );
    } else if !is_running {
        registry.instances[idx].pid = None;
    }

    // 2. Read authenticated email from instance's state.vscdb
    let db_path = if is_default_inst {
        crate::modules::db::get_db_path(None).unwrap_or_else(|_| {
            PathBuf::from(&data_dir)
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")
        })
    } else {
        let p = PathBuf::from(&data_dir)
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if p.exists() {
            p
        } else {
            let roaming = PathBuf::from(&data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb");
            if roaming.exists() {
                roaming
            } else {
                p
            }
        }
    };

    let detected_email = crate::modules::db::read_injected_email(&db_path);

    // 3. Match account in account list
    let all_accounts = crate::modules::account::list_accounts().unwrap_or_default();
    let matching_account = if let Some(ref email) = detected_email {
        all_accounts
            .iter()
            .find(|a| a.email.trim().eq_ignore_ascii_case(email.trim()))
            .cloned()
    } else {
        None
    };

    if let Some(ref acc) = matching_account {
        registry.instances[idx].bound_account_id = Some(acc.id.clone());
        registry.instances[idx].bound_email = Some(acc.email.clone());
    }

    // 4. Refresh quota for the bound or detected account
    let target_account_id = matching_account
        .as_ref()
        .map(|a| a.id.clone())
        .or_else(|| registry.instances[idx].bound_account_id.clone());

    if let Some(ref acc_id) = target_account_id {
        if let Ok(mut acc) = crate::modules::account::load_account(acc_id) {
            match crate::modules::account::fetch_quota_with_retry(&mut acc).await {
                Ok(fresh_quota) => {
                    // Justification: quota bookkeeping; re-fetched on the next quota sync
                    crate::error::record_ignored(
                        crate::modules::account::update_account_quota(acc_id, fresh_quota),
                        "update account quota",
                    );
                }
                Err(err) => {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Failed to refresh quota for account {} ({}): {}",
                        acc.email, acc_id, err
                    ));
                }
            }
        }
    }

    // 5. Persist updated instances.json
    save_registry(&registry)?;

    // 6. Emit UI refresh events
    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: frontend refresh hint; the UI re-polls state on its own cadence
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit frontend event",
        );
        // Justification: frontend refresh hint; the UI re-polls state on its own cadence
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit frontend event",
        );
    }

    // 7. Calculate memory usage
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let memory_mb = live_pid.and_then(|p| {
        system
            .process(sysinfo::Pid::from_u32(p))
            .map(|proc| proc.memory() as f64 / (1024.0 * 1024.0))
    });

    let updated_config = registry.instances[idx].clone();

    Ok(InstanceStatus {
        config: updated_config,
        is_running,
        pid: live_pid,
        memory_mb,
    })
}

/// Synchronize live running PIDs and quotas for all instances
pub async fn sync_all_instances_and_quotas_logic() -> Result<Vec<InstanceStatus>, String> {
    let registry = load_registry()?;
    let mut statuses = Vec::new();

    for inst in &registry.instances {
        match sync_instance_pid_and_quota_logic(&inst.id).await {
            Ok(status) => statuses.push(status),
            Err(err) => {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to sync PID and quota for instance {}: {}",
                    inst.id, err
                ));
                let is_default_inst = inst.is_default || inst.id == "default";
                let pids = find_pids_for_data_dir(&inst.data_dir, is_default_inst);
                let first_pid = pids.first().copied();
                let is_running = !pids.is_empty();
                statuses.push(InstanceStatus {
                    config: inst.clone(),
                    is_running,
                    pid: first_pid,
                    memory_mb: None,
                });
            }
        }
    }

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: frontend refresh hint; the UI re-polls state on its own cadence
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit frontend event",
        );
        // Justification: frontend refresh hint; the UI re-polls state on its own cadence
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit frontend event",
        );
    }

    Ok(statuses)
}
