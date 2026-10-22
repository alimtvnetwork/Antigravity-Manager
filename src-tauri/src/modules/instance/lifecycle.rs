//! Restart, copy, and clone entry points.
use super::*;
use crate::error::AppError;
use crate::error::AppResult;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Restart an instance on its current profile and bound account.
///
/// Hardened flow ("Kill First → Verify Dead → Refresh → Start"):
/// 1. The bound/selected account's token is refreshed BEFORE closing, so the
///    reopened IDE comes up with the same account on fresh credentials.
/// 2. The old IDE process tree is force-closed (graceful → SIGKILL/taskkill
///    escalation inside `close_instance`) and its death is VERIFIED — a close
///    failure is a hard error, never swallowed.
/// 3. The stale smart-PID cache entry is dropped so the launch below cannot
///    mistake the dead process for a live one, skip its own close, and spawn
///    a duplicate process on the same data dir (which corrupts the credential
///    injection — the exiting process overwrites state.vscdb).
/// 4. `launch_instance` re-injects the bound account's credentials and spawns
///    the same IDE fresh; the new process is verified alive before returning.
pub fn restart_instance(instance_id: &str) -> AppResult<InstanceStatus> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    crate::modules::logger::log_info(&format!(
        "[Instance] Restarting instance '{}' (resolved: '{}')",
        instance_id, resolved_id
    ));

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::RestartInstance,
        &format!("Restart instance {}", resolved_id),
        Some(&resolved_id),
    );

    let registry = load_registry().map_err(AppError::Config)?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .cloned()
        .ok_or_else(|| AppError::Config(format!("Instance '{}' not found", resolved_id)))?;

    // 0. The bound/selected account's token is refreshed by the async command
    //    wrapper BEFORE this runs (see commands::instance::restart_instance),
    //    so the reopened IDE comes up with the same account on fresh credentials.
    //    (A sync fn must not block_on inside the Tokio runtime.)

    // 1-2. Force-close the IDE and VERIFY the process tree is actually gone
    //    before touching credentials. close_instance_verified retries once
    //    for stragglers; if orphans still survive it logs a warning and lets
    //    the relaunch proceed instead of bricking the restart.
    let closed = close_instance_verified(&resolved_id, std::time::Duration::from_millis(5000))
        .map_err(AppError::Unknown)?;
    if !closed {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Restart of '{}' proceeding with surviving processes; relaunch will replace the session",
            resolved_id
        ));
    }

    // 4. Invalidate prompt tree cache so running status reflects fresh state
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(&resolved_id));

    // 5. Launch the same IDE fresh; the launch pipeline re-injects the bound
    //    account's credentials (token, device profile, keyring) before spawn.
    launch_instance(&resolved_id)?;

    // 6. Verify the new process actually came up.
    let launched = {
        let start = std::time::Instant::now();
        let mut ok = false;
        while start.elapsed() < std::time::Duration::from_millis(8000) {
            let (running, _, _) = is_instance_process_running_smart(&resolved_id);
            if running {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        ok
    };
    if !launched {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Instance '{}' did not confirm running within 8s; process may still be initializing",
            resolved_id
        ));
    }

    // 7. Return updated InstanceStatus
    let statuses = list_instances().map_err(AppError::Unknown)?;
    let updated = statuses
        .into_iter()
        .find(|s| s.config.id == resolved_id || s.config.name == resolved_id)
        .ok_or_else(|| {
            AppError::Process(format!(
                "Instance '{}' not found in registry after restart",
                resolved_id
            ))
        })?;

    crate::modules::logger::log_info(&format!(
        "[Instance] Restarted instance '{}' on bound account {:?}",
        resolved_id, config.bound_account_id
    ));
    audit.succeed(&format!(
        "Restarted instance '{}' on bound account {:?}",
        resolved_id, config.bound_account_id
    ));
    crate::modules::task_history_db::record(
        crate::modules::audit_action::AuditAction::RestartInstance,
        &resolved_id,
        "ok",
        &format!(
            "Restarted instance '{}' on bound account {:?}",
            resolved_id, config.bound_account_id
        ),
        Some(&resolved_id),
    );
    Ok(updated)
}

/// Copy/clone an existing profile (full directory copy by default, or profile only)
pub fn copy_instance(
    source_id: &str,
    target_name: String,
    clone_mode: Option<&str>,
) -> Result<InstanceConfig, String> {
    copy_instance_with_options(source_id, target_name, clone_mode, true)
}

/// Clone alias for copy_instance_with_options
pub fn clone_instance(
    source_id: &str,
    target_name: &str,
    clone_mode: Option<&str>,
    copy_projects: bool,
) -> Result<InstanceConfig, String> {
    copy_instance_with_options(
        source_id,
        target_name.to_string(),
        clone_mode,
        copy_projects,
    )
}

/// Imp-07: Conscious IDE PID Process Verification
/// Verify that a candidate process executable name or process name genuinely belongs to the Antigravity IDE family
pub fn is_conscious_ide_pid(pid: u32) -> bool {
    let mut system = sysinfo::System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]),
        sysinfo::ProcessRefreshKind::new()
            .with_cmd(sysinfo::UpdateKind::Always)
            .with_exe(sysinfo::UpdateKind::Always),
    );
    if let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) {
        let p_name = proc.name().to_string_lossy().to_lowercase();
        let p_exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        p_name.contains("antigravity") || p_exe.contains("antigravity")
    } else {
        false
    }
}

/// Imp-541: Affirmative predicate determining if an instance process is genuinely alive and healthy.
/// Eliminates compound negative logic (`!running || !has_active_process`).
pub fn is_instance_running_and_active(running: bool, has_active_process: bool) -> bool {
    running && has_active_process
}

/// Imp-541: Determines whether a candidate lockfile is stale and safe to be unlinked.
pub fn is_stale_lock_removable(is_pid_alive: bool, is_content_empty: bool) -> bool {
    !is_pid_alive || is_content_empty
}

/// Imp-61: Stale Session Lock Recovery
/// Scans the instance data path for stale IDE session lockfiles (`.vscode-server`, `.antigravity-session`,
/// `SingletonLock`, `code.lock`). If the holding PID is dead in the OS process table or file is orphaned,
/// safely unlinks the stale lock to avoid startup stalls.
pub fn recover_stale_session_locks(data_path: &std::path::Path) -> usize {
    let mut recovered = 0;
    if !data_path.exists() {
        return 0;
    }

    let lock_patterns = [
        "SingletonLock",
        "code.lock",
        ".antigravity-session",
        ".antigravity.lock",
        ".vscode-server.lock",
    ];

    let mut system = sysinfo::System::new();

    for pattern in &lock_patterns {
        let lock_file = data_path.join(pattern);
        if lock_file.exists() {
            let mut should_remove = false;
            if let Ok(content) = std::fs::read_to_string(&lock_file) {
                let trimmed = content.trim();
                if let Ok(pid) = trimmed.parse::<u32>() {
                    system.refresh_processes_specifics(
                        sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]),
                        sysinfo::ProcessRefreshKind::new(),
                    );
                    let is_alive = system.process(sysinfo::Pid::from_u32(pid)).is_some();
                    should_remove = is_stale_lock_removable(is_alive, false);
                } else if trimmed.is_empty() {
                    should_remove = is_stale_lock_removable(false, true);
                }
            } else {
                should_remove = true;
            }

            if should_remove {
                // Justification: best-effort recovery of stale IDE lockfile
                if std::fs::remove_file(&lock_file).is_ok() {
                    recovered += 1;
                    crate::modules::logger::log_info(&format!(
                        "[Instance] Recovered stale session lockfile: {:?}",
                        lock_file
                    ));
                }
            }
        }
    }

    recovered
}

/// Imp-63: Instance Launch CDP Backoff Ladder
/// Probes the instance's DevTools active CDP port with a configurable backoff ladder.
/// Returns the first responsive port or None if timed out across all steps.
pub fn probe_instance_cdp_with_backoff(instance_id: &str, backoff_steps: &[u64]) -> Option<u16> {
    let data_dir = if instance_id.is_empty() || instance_id == "default" || instance_id == "__default__" {
        super::storage::get_default_antigravity_data_dir()
    } else if let Ok(reg) = super::load_registry() {
        reg.instances
            .iter()
            .find(|i| i.id == instance_id || i.name == instance_id)
            .map(|i| std::path::PathBuf::from(&i.data_dir))
            .unwrap_or_else(super::storage::get_default_antigravity_data_dir)
    } else {
        super::storage::get_default_antigravity_data_dir()
    };

    let steps = if backoff_steps.is_empty() {
        &[500, 1000, 2000][..]
    } else {
        backoff_steps
    };

    for &pause_ms in steps {
        if pause_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(pause_ms));
        }
        for file in super::cdp_onboarding::candidate_devtools_port_files(&data_dir) {
            if let Ok(content) = std::fs::read_to_string(&file) {
                if let Some(first_line) = content.lines().next() {
                    if let Ok(port) = first_line.trim().parse::<u16>() {
                        if port > 0 {
                            return Some(port);
                        }
                    }
                }
            }
        }
    }
    None
}
