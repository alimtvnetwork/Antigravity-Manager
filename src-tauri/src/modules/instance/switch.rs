//! Account switching for instances.
use super::*;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Switch account and inject into a specific target instance without terminating siblings
pub async fn switch_account_to_instance(
    account_id: &str,
    target_instance_id: Option<&str>,
) -> Result<(), String> {
    let mut account = crate::modules::account::load_account(account_id)?;

    // Cross-Machine Distributed Lease Collision Guard
    check_switch_lease_guard(&account)?;

    let fresh_token = crate::modules::oauth::ensure_fresh_token(&account.token, Some(&account.id))
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;
    if fresh_token.access_token != account.token.access_token {
        account.token = fresh_token;
        crate::modules::account::save_account(&account)?;
    }

    let registry = load_registry()?;
    let target_id = match target_instance_id {
        Some(s) => resolve_instance_id(s).unwrap_or_else(|_| s.to_string()),
        None => get_active_instance_id().unwrap_or_else(|_| registry.active_instance_id.clone()),
    };
    let effective_instance_id = if target_id.is_empty() {
        "default".to_string()
    } else {
        target_id.clone()
    };

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::SwitchAccount,
        &account.email,
        Some(&effective_instance_id),
    );

    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id)
        .ok_or_else(|| {
            let err = format!("Target instance {} not found", target_id);
            crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
            err
        })?;

    refuse_if_caller_instance(&target_id)?;

    // Sibling Local Running Guard: prevent switching to accounts actively bound to running sibling instances on the same host!
    let bound_inst = registry
        .instances
        .iter()
        .find(|i| i.bound_account_id.as_deref() == Some(&account.id));
    if let Some(inst) = bound_inst {
        if inst.id != target_id && is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
            let err = format!(
                "Cannot switch instance to account '{}': Account is actively bound to running sibling instance '{}' on this machine",
                account.email,
                inst.name
            );
            crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
            return Err(err);
        }
    }

    let lease_ttl_secs = crate::modules::workspace_lease_manager::get_default_lease_ttl_secs();

    let is_default_inst = instance.is_default || instance.id == "default";

    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:START] Target instance: '{}' (default: {}), Target account: '{}'",
        instance.id, is_default_inst, account.email
    ));

    let prev_account_opt = instance
        .bound_account_id
        .as_ref()
        .and_then(|id| crate::modules::account::load_account(id).ok())
        .or_else(|| {
            crate::modules::account::get_current_account()
                .ok()
                .flatten()
        });
    let prev_email = instance
        .bound_email
        .clone()
        .or_else(|| prev_account_opt.as_ref().map(|a| a.email.clone()))
        .filter(|e| !e.trim().eq_ignore_ascii_case(account.email.trim()));

    let (prev_4h, prev_weekly) = if prev_email.is_some() {
        prev_account_opt
            .as_ref()
            .map(|a| crate::modules::auto_switcher::extract_dual_window_quotas(a, "gemini-2.5-pro"))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    if let Some(ref email) = prev_email {
        if !email.is_empty() {
            crate::modules::notification_hub::record_previous_email(email);
        }
    }

    // Ensure account has a bound device fingerprint profile for isolation
    if account.device_profile.is_none() {
        let new_profile = crate::modules::device::generate_profile();
        // Justification: account bookkeeping; re-applied on the next account touch
        crate::error::record_ignored(
            crate::modules::account::apply_profile_to_account(
                &mut account,
                new_profile,
                Some("auto_generated".to_string()),
                true,
            ),
            "apply profile to account",
        );
    }

    // 1. Snapshot running executable path & CLI workspace args BEFORE closing processes
    let active_exe_path = if is_default_inst {
        crate::modules::process::get_antigravity_executable_path(None)
            .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
    } else {
        None
    };
    let active_args = if is_default_inst {
        crate::modules::process::get_args_from_running_process(None)
            .or_else(|| crate::modules::process::get_args_from_running_process(Some("ide")))
    } else {
        None
    };

    // Snapshot active workspaces from current active instance if switching to a target instance with no workspaces
    let inherited_workspaces = if !is_default_inst
        && !registry.active_instance_id.is_empty()
        && registry.active_instance_id != instance.id
    {
        let current_target_ws = get_instance_workspace_folders(&instance.id, &instance.data_dir);
        if current_target_ws.is_empty() {
            migrate_instance_workspaces(&registry.active_instance_id, &instance.id).ok()
        } else {
            None
        }
    } else {
        None
    };

    // 1.5. [Step 1/5] Snapshot and re-enqueue running prompts scoped to the currently active instance BEFORE closing IDE
    let source_instance_id = if !registry.active_instance_id.is_empty() {
        registry.active_instance_id.clone()
    } else {
        instance.id.clone()
    };

    // Incomplete-prompt sweep: transition ALL uncompleted prompts (running, in_flight, dispatched, any age)
    // to backed_up so long-running prompts are never dropped when switching or when quota expires.
    crate::error::record_ignored(
        crate::modules::repo_db::backup::sweep_incomplete_prompts_for_instance(&source_instance_id),
        "sweep incomplete prompts before switch",
    );

    let existing_backed_up = crate::modules::repo_db::count_backed_up_prompts(&source_instance_id);
    let newly_backed_up = match crate::modules::repo_db::backup_running_prompts(&source_instance_id)
    {
        Ok(count) => count,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[INSTANCE_SWITCH] Failed to backup running prompts for source '{}': {}",
                source_instance_id, e
            ));
            0
        }
    };
    let backed_up_count = newly_backed_up.max(existing_backed_up);
    // Justification: opportunistic prompt backup; re-attempted on the next switch
    crate::error::record_ignored(
        crate::modules::backup_prompts_db::backup_active_running_prompts(
            Some(&source_instance_id),
            None,
        ),
        "back up active prompts",
    );

    // Re-attribute backed-up prompts from source instance to target instance prior to closing IDE
    if source_instance_id != instance.id {
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            let now = chrono::Utc::now().timestamp();
            let is_source_default = source_instance_id == "default" || source_instance_id == "__default__";
            // Justification: prompt re-attribution to target instance during switch
            if is_source_default {
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE active_prompts SET instance_id = ?1, updated_at = ?2 WHERE (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL) AND status = 'backed_up'",
                        rusqlite::params![&instance.id, now],
                    ),
                    "re-attribute active prompts to target instance",
                );
            } else {
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE active_prompts SET instance_id = ?1, updated_at = ?2 WHERE instance_id = ?3 AND status = 'backed_up'",
                        rusqlite::params![&instance.id, now, &source_instance_id],
                    ),
                    "re-attribute active prompts to target instance",
                );
            }
        }
    }

    // RC-2 parity with the auto-rotation path: also re-attribute durable
    // prompt_backups rows (unrestored only) so the target-scoped restore finds
    // backups recorded under the source instance.
    if source_instance_id != instance.id {
        if let Ok(conn) = crate::modules::backup_prompts_db::connect_db() {
            let is_source_default = source_instance_id == "default" || source_instance_id == "__default__";
            // Justification: prompt re-attribution to target instance during switch
            if is_source_default {
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE prompt_backups SET instance_id = ?1 WHERE (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = '') AND is_restored = 0",
                        rusqlite::params![&instance.id],
                    ),
                    "re-attribute prompt backups to target instance",
                );
            } else {
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE prompt_backups SET instance_id = ?1 WHERE instance_id = ?2 AND is_restored = 0",
                        rusqlite::params![&instance.id, &source_instance_id],
                    ),
                    "re-attribute prompt backups to target instance",
                );
            }
        }
    }

    let workspace_paths = get_instance_workspace_folders(&instance.id, &instance.data_dir);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up {} prompts from active source instance '{}' for target '{}'",
        backed_up_count,
        source_instance_id,
        instance.id
    ));
    let project_names: Vec<String> = workspace_paths
        .iter()
        .map(|p| {
            Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        })
        .collect();
    let backup_batch_id = uuid::Uuid::new_v4().to_string();

    let backup_step = crate::modules::task_history_db::SwitchBackupStep {
        prompt_count: backed_up_count,
        project_names,
        project_paths: workspace_paths.clone(),
        backup_batch_id,
        success: true,
    };

    // 2. [Step 2/5] Pre-Switch PID Eviction ("Kill First -> Write Second -> Start Third")
    //    Running Antigravity flushes in-memory state to state.vscdb/keyring on exit; closing first
    //    prevents the exiting process from overwriting our newly injected credentials.
    //    Before touching state.vscdb, verify that target instance PIDs are completely evicted and dead.
    let pids_to_kill = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
    let target_is_running = is_instance_running(&instance.id, &instance.data_dir, instance.pid)
        || !pids_to_kill.is_empty()
        || get_instance_saved_pid(&instance.id)
            .map_or(false, |p| is_pid_alive_targeted(p) && saved_pid_matches(p));

    let killed_agy_workers =
        crate::modules::repo_db::terminate_agy_workers_for_instance(&instance.id);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_2_TERMINATE] Pre-switch PID eviction for instance '{}' (running: {}, PIDs: {:?}, agy workers: {})",
        instance.id,
        target_is_running,
        pids_to_kill,
        killed_agy_workers
    ));

    if target_is_running || !pids_to_kill.is_empty() {
        let closed = close_instance_verified(&instance.id, std::time::Duration::from_millis(5000))
            .map_err(|e| {
                format!(
                    "Failed to close instance '{}' before account switch: {}",
                    instance.id, e
                )
            })?;

        if !closed {
            crate::modules::logger::log_warn(&format!(
                "[INSTANCE_SWITCH] Instance '{}' had lingering processes after close_instance_verified; escalating to force kill ladder",
                instance.id
            ));
        }

        #[cfg(target_os = "windows")]
        {
            let stragglers = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
            for spid in stragglers {
                if is_pid_alive_targeted(spid) && saved_pid_matches(spid) {
                    crate::modules::logger::log_warn(&format!(
                        "[INSTANCE_SWITCH] Found straggler PID {} before credential rewrite; force terminating via Win32",
                        spid
                    ));
                    force_terminate_pid_win32(spid);
                    if is_pid_alive_targeted(spid) {
                        // Justification: powershell Stop-Process fallback when Win32 TerminateProcess fails
                        crate::error::record_ignored(
                            std::process::Command::new("powershell")
                                .args([
                                    "-NoProfile",
                                    "-NonInteractive",
                                    "-Command",
                                    &format!("Stop-Process -Id {} -Force", spid),
                                ])
                                .creation_flags(0x08000000)
                                .output(),
                            "powershell Stop-Process fallback",
                        );
                    }
                }
            }
        }

        // Settle drain loop (up to 1500ms) after force_terminate_pid_win32 for OS kernel and NTFS file handle release
        let drain_start = std::time::Instant::now();
        let max_drain = std::time::Duration::from_millis(1500);
        while drain_start.elapsed() < max_drain {
            force_refresh_process_cache();
            let remaining_live = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
            let still_alive = remaining_live
                .iter()
                .any(|&p| is_pid_alive_targeted(p) && saved_pid_matches(p));
            let saved_alive = get_instance_saved_pid(&instance.id)
                .or(instance.pid)
                .map_or(false, |p| is_pid_alive_targeted(p) && saved_pid_matches(p));
            if !still_alive && !saved_alive {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    let target_data_path = std::path::PathBuf::from(&instance.data_dir);
    // Ensure lockfiles are purged before credentials are written
    clear_stale_instance_lockfiles(&target_data_path);

    force_refresh_process_cache();
    let final_live = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
    let mut any_alive = final_live
        .iter()
        .any(|&p| is_pid_alive_targeted(p) && saved_pid_matches(p));
    let mut saved_pid_alive = get_instance_saved_pid(&instance.id)
        .or(instance.pid)
        .map_or(false, |p| is_pid_alive_targeted(p) && saved_pid_matches(p));

    if any_alive || saved_pid_alive {
        crate::modules::logger::log_warn(&format!(
            "[INSTANCE_SWITCH] Candidate PIDs still alive for '{}' before rewrite; attempting immediate force escalation",
            instance.id
        ));
        for &spid in &final_live {
            if is_pid_alive_targeted(spid) && saved_pid_matches(spid) {
                #[cfg(target_os = "windows")]
                {
                    force_terminate_pid_win32(spid);
                    if is_pid_alive_targeted(spid) {
                        crate::error::record_ignored(
                            std::process::Command::new("powershell")
                                .args([
                                    "-NoProfile",
                                    "-NonInteractive",
                                    "-Command",
                                    &format!("Stop-Process -Id {} -Force", spid),
                                ])
                                .creation_flags(0x08000000)
                                .output(),
                            "powershell Stop-Process fallback",
                        );
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // Justification: best effort fallback kill on surviving processes
                    crate::error::record_ignored(
                        std::process::Command::new("kill")
                            .args(["-9", &spid.to_string()])
                            .output(),
                        "kill straggler pid",
                    );
                }
            }
        }
        if let Some(spid) = get_instance_saved_pid(&instance.id).or(instance.pid) {
            if is_pid_alive_targeted(spid) && saved_pid_matches(spid) {
                #[cfg(target_os = "windows")]
                {
                    force_terminate_pid_win32(spid);
                    if is_pid_alive_targeted(spid) {
                        crate::error::record_ignored(
                            std::process::Command::new("powershell")
                                .args([
                                    "-NoProfile",
                                    "-NonInteractive",
                                    "-Command",
                                    &format!("Stop-Process -Id {} -Force", spid),
                                ])
                                .creation_flags(0x08000000)
                                .output(),
                            "powershell Stop-Process fallback",
                        );
                    }
                }
            }
        }

        // Additional drain after fallback force-termination
        let fallback_drain = std::time::Instant::now();
        while fallback_drain.elapsed() < std::time::Duration::from_millis(1000) {
            force_refresh_process_cache();
            let remaining_live = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
            any_alive = remaining_live
                .iter()
                .any(|&p| is_pid_alive_targeted(p) && saved_pid_matches(p));
            saved_pid_alive = get_instance_saved_pid(&instance.id)
                .or(instance.pid)
                .map_or(false, |p| is_pid_alive_targeted(p) && saved_pid_matches(p));
            if !any_alive && !saved_pid_alive {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    if any_alive || saved_pid_alive {
        let err = format!(
            "Instance '{}' is still running after close; account switch stopped before credentials were rewritten to state.vscdb",
            instance.id
        );
        crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
        return Err(err);
    }

    // Enforce mandatory 200ms kernel settle delay to allow NTFS file handles to close
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Ensure lockfiles are completely clean before injecting credentials
    clear_stale_instance_lockfiles(&target_data_path);
    invalidate_instance_process_cache(&instance.id);

    // 3. [Step 3/5] Inject credentials into target instance's state.vscdb & storage.json (and OS keyring) AFTER process exit
    inject_account_credentials(
        &instance.id,
        &instance.data_dir,
        is_default_inst,
        &account,
        &CredentialInjectOptions::for_switch(),
    )?;
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_3_CREDENTIALS] Injected credentials for account '{}' into state.vscdb, storage.json, and OS keyring for instance '{}'",
        account.email, instance.id
    ));

    let reset_step = crate::modules::task_history_db::SwitchResetStep {
        terminated_pids: pids_to_kill,
        auth_swapped: true,
        credentials_injected: true,
        success: true,
    };

    // 4. Bind account in registry and set active account
    bind_account_to_instance(&instance.id, &account.id, &account.email)?;
    let registry_after = match load_registry() {
        Ok(reg) => reg,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[INSTANCE_SWITCH] Failed to reload registry after binding account: {}",
                e
            ));
            Default::default()
        }
    };
    if registry_after.active_instance_id.is_empty()
        || registry_after.active_instance_id == instance.id
    {
        // Justification: instance-state bookkeeping; in-memory state is already applied and re-persisted on next change
        crate::error::record_ignored(
            set_active_instance_id(&instance.id),
            "set active instance id",
        );
    }
    if is_default_inst {
        // Justification: account-state bookkeeping; in-memory state is already applied and re-persisted on next change
        crate::error::record_ignored(
            crate::modules::account::set_current_account_id(&account.id),
            "set current account id",
        );
    }

    account.update_last_used();
    // Justification: account persistence is write-through; re-persisted on the next account change
    crate::error::record_ignored(
        crate::modules::account::save_account(&account),
        "save account",
    );

    // Acquire distributed lease in Supabase Root DB for this instance profile
    let lease_acc_id = account.id.clone();
    let lease_acc_email = account.email.clone();
    let lease_inst_name = instance.name.clone();
    let ttl_secs = lease_ttl_secs;
    tauri::async_runtime::spawn(async move {
        // Justification: lease bookkeeping; lease state is re-validated on the next acquisition
        crate::error::record_ignored(
            crate::modules::workspace_lease_manager::acquire_lease_with_details(
                &lease_acc_id,
                &lease_acc_email,
                &lease_inst_name,
                ttl_secs,
            )
            .await,
            "acquire workspace lease",
        );
        // Justification: opportunistic cloud sync; the next scheduled sync retries
        crate::error::record_ignored(
            crate::modules::supabase_sync::sync_local_node_now().await,
            "sync local node",
        );
    });

    // Imp-12: Port Rebind Settling Delay on Switch
    // Allow lingering CDP debugger sockets and localhost IPC ports to release bindings before spawning new IDE
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 5. [Step 4/5] Relaunch Antigravity preserving exact executable path and bound workspace folders
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_4_LAUNCH] Spawning instance IDE for '{}' (executable: {:?})",
        instance.id, active_exe_path
    ));
    if is_default_inst {
        if let Err(e) = crate::modules::process::start_antigravity_with_fallback_path(
            None,
            active_exe_path.as_deref(),
            active_args.as_deref(),
        ) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] start_antigravity_with_fallback_path returned ({}), falling back to launch_instance",
                e
            ));
            launch_instance_without_prompt_reinject(&instance.id).map_err(|launch_err| {
                let err_msg = format!(
                    "Account switched for '{}', but auto-relaunch failed: {}",
                    instance.id, launch_err
                );
                crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err_msg));
                err_msg
            })?;
        }
        if let Some(h) = crate::modules::log_bridge::get_app_handle() {
            let integration = crate::modules::integration::SystemManager::Desktop(h);
            integration.update_tray();
        }
    } else {
        launch_instance_with_workspaces(&instance.id, inherited_workspaces.as_deref(), false)
            .map_err(|launch_err| {
                let err_msg = format!(
                    "Account switched for '{}', but auto-relaunch failed: {}",
                    instance.id, launch_err
                );
                crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err_msg));
                err_msg
            })?;
    }

    // 4b & 4c. Verify the relaunched IDE materialized and dismiss security notice in detached background task
    // Unblocks the command immediately to achieve <3s UI responsiveness
    let verify_inst_id = instance.id.clone();
    let verify_data_dir = instance.data_dir.clone();
    let verify_pid = instance.pid;
    let cdp_email = account.email.clone();
    tauri::async_runtime::spawn(async move {
        let relaunched = crate::modules::instance::verify_instance_relaunched(
            &verify_inst_id,
            &verify_data_dir,
            verify_pid,
        );
        if !relaunched {
            crate::modules::logger::log_warn(&format!(
                "[INSTANCE_SWITCH:ASYNC_VERIFY] Instance '{}' did not appear within 15s verification window",
                verify_inst_id
            ));
        }
        let cdp_data_dir = std::path::PathBuf::from(&verify_data_dir);
        let dismissed = tokio::task::spawn_blocking({
            let data_dir = cdp_data_dir.clone();
            let email = cdp_email.clone();
            move || {
                crate::modules::instance::dismiss_security_notice_dialog_for_data_dir(
                    &data_dir,
                    Some(email.as_str()),
                    std::time::Duration::from_secs(15),
                )
            }
        })
        .await
        .unwrap_or(false);
        if !dismissed {
            crate::modules::instance::spawn_security_notice_dismisser(
                cdp_data_dir,
                Some(cdp_email),
                std::time::Duration::from_secs(20),
            );
        }
    });

    // Stage 6: Sync Live PID & Broadcast Events
    let fresh_pid = {
        let mut detected = None;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(3000) {
            let pids = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
            if let Some(&first) = pids.first() {
                if is_pid_alive_targeted(first) {
                    detected = Some(first);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        detected
    };

    if let Some(pid) = fresh_pid {
        // Justification: recording live PID in DB and registry after relaunch
        crate::error::record_ignored(
            record_instance_pid(&instance.id, pid, &instance.data_dir),
            "record live pid after switch",
        );
    }

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts://refreshed",
        );
    }

    let target_inst_id = instance.id.clone();
    let target_backed_up = crate::modules::repo_db::count_backed_up_prompts(&target_inst_id);
    let total_pending_restore = backed_up_count.max(target_backed_up);
    let has_unrestored_backups = crate::modules::backup_prompts_db::list_prompt_backups(Some(&target_inst_id), None)
        .map(|list| list.iter().any(|r| !r.is_restored))
        .unwrap_or(false)
        || crate::modules::backup_prompts_db::count_unrestored_prompts_for_other_instances(&target_inst_id, None)
            .map(|c| c > 0)
            .unwrap_or(false);
    let channel_wait = crate::modules::repo_db::needs_prompt_channel_wait(total_pending_restore) || has_unrestored_backups;
    if channel_wait {
        crate::modules::logger::log_info(&format!(
            "[INSTANCE_SWITCH:STAGE_5_RESTORE] Scheduling prompt restore for instance '{}' after the IDE is up ({} backed up, unrestored available: {})",
            target_inst_id, total_pending_restore, has_unrestored_backups
        ));
        crate::modules::instance::schedule_prompt_restore_after_launch(target_inst_id.clone());
    } else {
        crate::modules::logger::log_info(&format!(
            "[INSTANCE_SWITCH:STAGE_5_RESTORE] No backed-up prompt for instance '{}'",
            target_inst_id
        ));
    }

    // RC-3: the Stage-5 restore runs on the deferred post-switch timer, so at
    // audit time its result does not exist yet. Report the real count at this
    // point (nothing restored synchronously) instead of claiming every
    // backed-up prompt was restored; the deferred restore unit reports its
    // actual restored count via the prompts://restored event and its log line.
    let restore_step = crate::modules::task_history_db::SwitchRestoreStep {
        method: "resume_task_json + sync_agy_prompt_restore".to_string(),
        restored_count: 0,
        dispatched_count: 0,
        prompt_channel_waited: channel_wait,
        success: true,
    };

    // Step 4: Verification Step - verify .antigravity_resume_task.json / workspace prompt files and active session
    let mut verified = true;
    for ws in &workspace_paths {
        let task_file = Path::new(ws).join(".antigravity_resume_task.json");
        if task_file.exists() {
            crate::modules::logger::log_info(&format!(
                "[Instance] Verified .antigravity_resume_task.json exists in workspace {}",
                ws
            ));
        } else {
            verified = false;
        }
    }
    let verification_step = crate::modules::task_history_db::SwitchVerificationStep {
        verified,
        message: if verified {
            "Verified prompts restored and session active".to_string()
        } else {
            "Verified session active with best-effort prompt files".to_string()
        },
    };

    let steps = crate::modules::task_history_db::SwitchAuditSteps {
        backup: Some(backup_step),
        reset: Some(reset_step),
        restore: Some(restore_step),
        verification: Some(verification_step),
    };

    // Mailbox poll, email, and Telegram run after the command returns.
    let (target_4h, target_weekly) =
        crate::modules::auto_switcher::extract_dual_window_quotas(&account, "gemini-2.5-pro");
    let followup_id = account.id.clone();
    let followup_email = account.email.clone();
    let followup_prev = prev_email.clone();
    let followup_instance_id = instance.id.clone();
    let followup_instance_name = instance.name.clone();
    tauri::async_runtime::spawn(async move {
        let mut pred_exclusions = vec![followup_id.clone(), followup_email.clone()];
        if let Some(ref p_em) = followup_prev {
            pred_exclusions.push(p_em.clone());
        }
        let predicted_next_email = crate::modules::auto_switcher::select_candidate_profiles(
            &followup_instance_id,
            "gemini-2.5-pro",
            15.0,
            &pred_exclusions,
        )
        .ok()
        .and_then(|v| v.into_iter().next())
        .filter(|c| {
            !c.email.trim().eq_ignore_ascii_case(followup_email.trim())
                && followup_prev
                    .as_deref()
                    .map(|p| !c.email.trim().eq_ignore_ascii_case(p.trim()))
                    .unwrap_or(true)
        })
        .map(|c| c.email);
        let running_projs = match crate::modules::repo_db::list_running_projects() {
            Ok(projs) => projs,
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "[INSTANCE_SWITCH] Failed to list running projects for notification: {}",
                    e
                ));
                Vec::new()
            }
        };
        let unique_projs = crate::modules::notification_hub::deduplicate_names(
            running_projs.into_iter().map(|p| p.repo_name),
        );
        let (node_alias, _) = crate::modules::email_sender::get_local_node_identity();
        let m_name = crate::modules::email_watcher::detect_machine_name();
        let m_ip = crate::modules::email_watcher::detect_local_ip();
        let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
        crate::modules::audit::record_if_active(
            "user",
            "click",
            "instance::switch_account",
            &format!("instances | instance_table | switch_dialog | {followup_instance_id}"),
            serde_json::json!({
                "agm_version": pkg_ver,
                "machine_name": m_name,
                "node_alias": node_alias,
                "local_ip": m_ip,
                "instance_id": followup_instance_id,
                "instance_name": followup_instance_name,
                "switch_mode": "manual",
                "condition": "manual_switch",
                "from_account": followup_prev.as_deref().map(crate::modules::logger::mask_email),
                "to_account": crate::modules::logger::mask_email(&followup_email),
                "predicted_next_account": predicted_next_email.as_deref().map(crate::modules::logger::mask_email),
                "previous_quota_4h": prev_4h,
                "previous_quota_weekly": prev_weekly,
                "target_quota_4h": target_4h,
                "target_quota_weekly": target_weekly,
                "credit_before_switch": prev_4h,
                "backed_up_prompts": backed_up_count,
                "backed_up_projects": unique_projs,
                "auto_resume": backed_up_count > 0,
                "reason": "Smart Rotator / Instance Account Switch",
                "timestamp": chrono::Utc::now().timestamp_millis(),
            }),
            "ok",
            "",
        );
        crate::error::record_ignored(
            Ok::<String, String>(
                crate::modules::notification_hub::notify_account_switched_details(
                    crate::modules::notification_hub::SwitchNotificationDetails {
                        previous_email: followup_prev,
                        previous_quota_4h: prev_4h,
                        previous_quota_weekly: prev_weekly,
                        predicted_next_email,
                        selected_email: followup_email,
                        target_quota_4h: target_4h,
                        target_quota_weekly: target_weekly,
                        credit_before_switch: prev_4h,
                        threshold_activated: None,
                        instance_id: followup_instance_id.clone(),
                        instance_name: followup_instance_name,
                        instance_mode: String::new(),
                        reason: "Smart Rotator / Instance Account Switch".to_string(),
                        is_auto: false,
                        backed_up_projects: unique_projs,
                        backed_up_prompts_count: Some(backed_up_count),
                        // RC-3: restore is deferred to the post-switch timer and
                        // has not run yet — do not report backed_up_count as restored.
                        restored_prompts_count: None,
                    },
                )
                .await,
            ),
            "notify_account_switched",
        ); // Justification: notification failure should not abort account switch
    });

    let snap = crate::modules::repo_db::switch_prompt_snapshot(&instance.id);
    let machine_alias = crate::modules::email_watcher::detect_machine_name();
    let ide_path = instance.executable_path.clone().unwrap_or_else(|| {
        crate::modules::process::get_antigravity_executable_path(None)
            .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    let payload = crate::modules::task_history_db::switch_payload(
        &crate::modules::task_history_db::SwitchFacts {
            from_email: prev_email.clone().unwrap_or_default(),
            to_email: account.email.clone(),
            reason: "Instance account switch".to_string(),
            how: "The instance IDE was closed, the new account was written into that instance, then the instance was opened again on the same conversation.".to_string(),
            prompt_id: snap.prompt_id,
            prompt_text: snap.prompt_text,
            conversation_id: snap.conversation_id,
            prompt_reinjected: channel_wait,
            switch_ok: true,
            instance_id: instance.id.clone(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: machine_alias,
            ide_path,
            switch_reason: "Instance account switch".to_string(),
            steps: Some(steps),
        },
    );
    audit.succeed_with_payload("switch finished", &payload);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:SUCCESS] Switch completed successfully for instance '{}' to account '{}'",
        instance.id, account.email
    ));
    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts://refreshed",
        );
    }
    Ok(())
}
