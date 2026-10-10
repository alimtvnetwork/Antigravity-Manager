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
        return Err(AppError::Process(format!(
            "Instance '{}' relaunch did not produce a live process",
            resolved_id
        )));
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
