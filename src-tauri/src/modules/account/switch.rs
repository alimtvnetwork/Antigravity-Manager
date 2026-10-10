pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;

use super::*;

/// Switch current account (Core Logic)
pub async fn switch_account(
    account_id: &str,
    target_ide: Option<&str>,
    integration: &(impl modules::integration::SystemIntegration + ?Sized),
) -> Result<(), String> {
    use crate::modules::oauth;

    let index = {
        let _lock = ACCOUNT_INDEX_LOCK
            .lock()
            .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
        load_account_index()?
    };

    // 1. Verify account exists
    if !index.accounts.iter().any(|s| s.id == account_id) {
        return Err(format!("Account not found: {}", account_id));
    }

    let mut account = load_account(account_id)?;
    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::SwitchAccount,
        &account.email,
        target_ide,
    );
    if account.disabled || account.proxy_disabled || account.validation_blocked {
        return Err(format!(
            "Cannot switch to account '{}': Account is disabled or blocked (disabled: {}, proxy_disabled: {}, validation_blocked: {})",
            account.email, account.disabled, account.proxy_disabled, account.validation_blocked
        ));
    }

    // Cross-Machine Distributed Lease Collision Guard
    if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
        &account.id,
        &account.email,
    ) {
        let holder_info = crate::modules::workspace_lease_manager::get_remote_lease_holder_info(
            &account.id,
            &account.email,
        );
        let detail = if let Some((alias, profile, remaining)) = holder_info {
            format!(
                "held by remote machine '{}' (Profile: '{}', expires in {}s)",
                alias, profile, remaining
            )
        } else {
            "currently leased by another active machine in the cluster".to_string()
        };
        return Err(format!(
            "Cannot switch to account '{}': Account is {}",
            account.email, detail
        ));
    }

    // Sibling Local Running Guard: prevent switching to accounts actively bound to running instances on the same host!
    if let Ok(registry) = crate::modules::instance::load_registry() {
        let bound_inst = registry
            .instances
            .iter()
            .find(|i| i.bound_account_id.as_deref() == Some(&account.id));
        if let Some(inst) = bound_inst {
            if inst.id != "default"
                && crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)
            {
                return Err(format!(
                    "Cannot switch to account '{}': Account is actively bound to running sibling instance '{}' on this machine",
                    account.email, inst.name
                ));
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "Switching to account: {} (ID: {}) (target_ide: {:?})",
        account.email, account.id, target_ide
    ));

    // 2. Ensure token is valid before switch. Surface clearer hints for known account-state failures.
    let fresh_token = match oauth::ensure_fresh_token(&account.token, Some(&account.id)).await {
        Ok(token) => token,
        Err(e) => {
            if is_account_access_blocked_message(&e) {
                mark_validation_blocked(&mut account, &e);
            }
            return Err(format_switch_refresh_error(&e));
        }
    };

    // If Token updated, save back to account file
    if fresh_token.access_token != account.token.access_token {
        account.token = fresh_token.clone();
        save_account(&account)?;
    }

    ensure_enterprise_project_ready(&mut account).await?;

    // Quota refresh hits loadCodeAssist. It must not sit in front of the IDE relaunch.
    // The stored quota is what the switch notification uses. A refresh runs after return.

    // [FIX] Ensure account has a device profile for isolation
    if account.device_profile.is_none() {
        crate::modules::logger::log_info(&format!(
            "Account {} has no bound fingerprint, generating new one for isolation...",
            account.email
        ));
        let new_profile = modules::device::generate_profile();
        apply_profile_to_account(
            &mut account,
            new_profile.clone(),
            Some("auto_generated".to_string()),
            true,
        )?;
    }

    // Capture previous account telemetry before switching
    let prev_acc_opt = get_current_account().ok().flatten();
    let prev_email = prev_acc_opt
        .as_ref()
        .map(|a| a.email.clone())
        .filter(|e| !e.trim().eq_ignore_ascii_case(account.email.trim()));
    let (prev_4h, prev_weekly) = if prev_email.is_some() {
        prev_acc_opt
            .as_ref()
            .map(|a| crate::modules::auto_switcher::extract_dual_window_quotas(a, "gemini-2.5-pro"))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    // 3. Execute platform-specific system integration (Close proc, Inject DB, Start proc, etc.)
    integration.on_account_switch(&account, target_ide).await?;

    // 4. Update tool internal state
    set_current_account_id_with_target(account_id, target_ide)?;

    account.update_last_used();
    save_account(&account)?;

    // Acquire distributed lease in Supabase Root DB and sync local node state
    let lease_acc_id = account.id.clone();
    let lease_acc_email = account.email.clone();
    let lease_profile = target_ide.unwrap_or("default").to_string();
    tauri::async_runtime::spawn(async move {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::workspace_lease_manager::acquire_lease_with_details(
                &lease_acc_id,
                &lease_acc_email,
                &lease_profile,
                90,
            )
            .await,
            "acquire_lease_with_details",
        );
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::supabase_sync::sync_local_node_now().await,
            "sync_local_node_now",
        );
    });

    crate::modules::logger::log_info(&format!(
        "Account switch core logic completed: {}",
        account.email
    ));

    let (target_4h, target_weekly) =
        crate::modules::auto_switcher::extract_dual_window_quotas(&account, "gemini-2.5-pro");

    let followup_id = account.id.clone();
    let followup_email = account.email.clone();
    let followup_prev = prev_email.clone();
    let followup_target = target_ide.unwrap_or("default").to_string();
    tauri::async_runtime::spawn(async move {
        // Predicted-next reads the mailbox. That poll used to block the switch command.
        let mut pred_exclusions = vec![followup_id.clone(), followup_email.clone()];
        if let Some(ref p_em) = followup_prev {
            pred_exclusions.push(p_em.clone());
        }
        let predicted_next_email = crate::modules::auto_switcher::select_candidate_profiles(
            &followup_target,
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
        // Justification: non-Result return value intentionally discarded — no error channel to track
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
                instance_id: followup_target.clone(),
                instance_name: followup_target,
                instance_mode: String::new(),
                reason: "Manual account switch".to_string(),
                is_auto: false,
                backed_up_projects: Vec::new(),
                backed_up_prompts_count: None,
                restored_prompts_count: None,
            },
        )
        .await;
        if let Ok(mut refreshed) = load_account(&followup_id) {
            if let Ok(fresh_quota) = fetch_quota_with_retry(&mut refreshed).await {
                refreshed.quota = Some(fresh_quota);
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(save_account(&refreshed), "save_account");
            }
        }
    });

    // This path always switches the default instance. `target_ide` names the IDE flavor ("ide"),
    // and prompts are stored under the instance id, so the snapshot must read "default".
    let snap = crate::modules::repo_db::switch_prompt_snapshot("default");
    let reinjected = crate::modules::integration::take_prompt_reinjected();
    let machine_alias = crate::modules::email_watcher::detect_machine_name();
    let ide_path = crate::modules::process::get_antigravity_executable_path(None)
        .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let payload = crate::modules::task_history_db::switch_payload(
        &crate::modules::task_history_db::SwitchFacts {
            from_email: prev_email.clone().unwrap_or_default(),
            to_email: account.email.clone(),
            reason: "Manual account switch".to_string(),
            how: "The IDE was closed, the new account was written into the session, then the IDE was opened again on the same conversation.".to_string(),
            prompt_id: snap.prompt_id,
            prompt_text: snap.prompt_text,
            conversation_id: snap.conversation_id,
            prompt_reinjected: reinjected,
            switch_ok: true,
            instance_id: "default".to_string(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: machine_alias,
            ide_path,
            switch_reason: "Manual account switch".to_string(),
            steps: None,
        },
    );
    audit.succeed_with_payload("switch finished", &payload);
    Ok(())
}
