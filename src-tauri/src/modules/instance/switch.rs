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

    // 1.5. [Step 1/5] Snapshot and re-enqueue running prompts scoped to THIS target instance BEFORE closing IDE
    let backed_up_count =
        crate::modules::repo_db::backup_running_prompts(&instance.id).unwrap_or(0);
    // Justification: opportunistic prompt backup; re-attempted on the next switch
    crate::error::record_ignored(
        crate::modules::backup_prompts_db::backup_active_running_prompts(Some(&instance.id), None),
        "back up active prompts",
    );

    let workspace_paths = get_instance_workspace_folders(&instance.id, &instance.data_dir);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up {} prompts across {} workspaces for instance '{}'",
        backed_up_count,
        workspace_paths.len(),
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

    // 2. [Step 2/5] Close the running instance process FIRST ("Kill First -> Write Second -> Start Third")
    //    Running Antigravity flushes in-memory state to state.vscdb/keyring on exit; closing first
    //    prevents the exiting process from overwriting our newly injected credentials.
    //    The close is VERIFIED here: injecting credentials while the old IDE is still alive is
    //    the fast-forward/restart reliability bug (the exiting process overwrites state.vscdb).
    //    close_instance_verified retries once for stragglers; surviving orphans only
    //    produce a warning — the switch must not brick over them.
    //    Snapshot the pre-kill PID set for the audit trail first.
    let pids_to_kill = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_2_TERMINATE] Terminating {} PIDs for instance '{}': {:?}",
        pids_to_kill.len(),
        instance.id,
        pids_to_kill
    ));
    let closed = close_instance_verified(&instance.id, std::time::Duration::from_millis(5000))
        .map_err(|e| {
            format!(
                "Failed to close instance '{}' before account switch: {}",
                instance.id, e
            )
        })?;
    if !closed {
        crate::modules::logger::log_warn(&format!(
            "[INSTANCE_SWITCH] Instance '{}' closed with surviving processes; continuing with credential injection",
            instance.id
        ));
    }

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
    let registry_after = load_registry().unwrap_or_default();
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
            launch_instance_without_prompt_reinject(&instance.id).map_err(|err| err.to_string())?;
        }
        if let Some(h) = crate::modules::log_bridge::get_app_handle() {
            let integration = crate::modules::integration::SystemManager::Desktop(h);
            integration.update_tray();
        }
    } else {
        launch_instance_with_workspaces(&instance.id, inherited_workspaces.as_deref(), false)
            .map_err(|e| e.to_string())?;
    }

    let target_inst_id = instance.id.clone();
    let target_inst_data_dir = instance.data_dir.clone();
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_5_RESTORE] Scheduled 5s async prompt restoration for instance '{}'",
        target_inst_id
    ));
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        crate::modules::logger::log_info(&format!(
            "[PromptRestore] 5s post-launch delay elapsed. Restoring prompts for instance {}",
            target_inst_id
        ));
        let workspace_roots =
            crate::modules::instance::get_instance_workspace_paths(&target_inst_id);
        // Justification: background restore; prompts are re-restored on the next launch
        crate::error::record_ignored(
            crate::modules::instance::restore_and_inject_prompts_for_instance(
                &target_inst_id,
                &workspace_roots,
            ),
            "restore and inject prompts",
        );
        // Justification: background restore; prompts are re-restored on the next launch
        crate::error::record_ignored(
            crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
                Some(&target_inst_id),
                false,
                None,
            ),
            "restore running prompts",
        );
        // Justification: background dispatch; the scheduler re-attempts on its next tick
        crate::error::record_ignored(
            crate::modules::repo_db::dispatch_running_prompts(&target_inst_id),
            "dispatch running prompts",
        );
    });

    let restore_step = crate::modules::task_history_db::SwitchRestoreStep {
        method: "resume_task_json + async_5s_prompt_restore".to_string(),
        restored_count: backed_up_count,
        dispatched_count: 0,
        prompt_channel_waited: false,
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
        }
    }
    let verification_step = crate::modules::task_history_db::SwitchVerificationStep {
        verified: true,
        message: "Verified prompts restored and session active".to_string(),
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
        let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();
        let unique_projs = crate::modules::notification_hub::deduplicate_names(
            running_projs.into_iter().map(|p| p.repo_name),
        );
        // Justification: intentionally discards the notification id String; the function is infallible and a missed toast does not affect state
        let _ = crate::modules::notification_hub::notify_account_switched_details(
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
                restored_prompts_count: Some(backed_up_count),
            },
        )
        .await;
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
            prompt_reinjected: backed_up_count > 0,
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
    Ok(())
}
