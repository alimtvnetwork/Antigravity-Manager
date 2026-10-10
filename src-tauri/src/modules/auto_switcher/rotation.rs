use crate::modules::{account, config, instance, logger};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::*;

#[derive(Debug, Clone, Default)]
pub struct RotationContext {
    pub current_instance_id: Option<String>,
    pub previous_email: Option<String>,
    pub previous_quota_4h: Option<f64>,
    pub previous_quota_weekly: Option<f64>,
    pub predicted_email: Option<String>,
    pub target_quota_4h: Option<f64>,
    pub target_quota_weekly: Option<f64>,
    pub credit_before_switch: Option<f64>,
    pub threshold_activated: Option<f64>,
}

/// Execute rotation to target candidate with rich telemetry context
pub async fn execute_profile_rotation_with_context(
    target: ProfileCandidate,
    reason: String,
    has_auto_resume: bool,
    ctx: Option<RotationContext>,
) -> Result<(), String> {
    let current_instance_id = ctx
        .as_ref()
        .and_then(|c| c.current_instance_id.clone())
        .unwrap_or_else(|| {
            crate::modules::instance::get_active_instance_id()
                .unwrap_or_else(|_| "default".to_string())
        });
    let inst_id = &target.instance_id;

    // Step 0: Ensure all running and queued prompts are snapshotted and backed up before profile switch
    let backup_res = crate::modules::repo_db::backup_running_prompts(&current_instance_id);
    let backed_up_count = backup_res.as_ref().copied().unwrap_or(0);
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::backup_prompts_db::backup_active_running_prompts(
            Some(&current_instance_id),
            None,
        ),
        "backup_active_running_prompts",
    );
    match &backup_res {
        Ok(c) => {
            logger::log_info(&format!(
                "[AutoSwitcher] Pre-switch backup created for {} running prompts (instance '{}')",
                c, current_instance_id
            ));
        }
        Err(e) => {
            logger::log_warn(&format!(
                "[AutoSwitcher] Pre-switch prompt backup failed for instance '{}': {}",
                current_instance_id, e
            ));
        }
    }

    if has_auto_resume {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            snapshot_task_state(&current_instance_id, &target.account_id, &reason),
            "snapshot_task_state",
        );
    }

    logger::log_info(&format!(
        "[AutoSwitcher] Rotating instance '{}' to account '{}' (email: {}, quota: {:.1}%). Reason: {}",
        inst_id, target.account_id, target.email, target.quota_percent, reason
    ));

    // Step 2: Trigger unified Email and Telegram notifications before switch
    let prev_email = ctx
        .as_ref()
        .and_then(|c| c.previous_email.clone())
        .filter(|pe| !pe.trim().eq_ignore_ascii_case(target.email.trim()));
    let mut prev_q_4h = ctx.as_ref().and_then(|c| c.previous_quota_4h);
    let mut prev_q_weekly = ctx.as_ref().and_then(|c| c.previous_quota_weekly);
    let predicted_opt = ctx
        .as_ref()
        .and_then(|c| c.predicted_email.clone())
        .filter(|em| {
            !em.trim().eq_ignore_ascii_case(target.email.trim())
                && prev_email
                    .as_deref()
                    .map(|pe| !em.trim().eq_ignore_ascii_case(pe.trim()))
                    .unwrap_or(true)
        })
        .or_else(|| {
            let mut pred_exclusions = vec![target.account_id.clone(), target.email.clone()];
            if let Some(ref pe) = prev_email {
                pred_exclusions.push(pe.clone());
            }
            select_candidate_profiles(&inst_id, "gemini-2.5-pro", 15.0, &pred_exclusions)
                .ok()
                .and_then(|v| v.into_iter().next())
                .filter(|c| {
                    !c.email.trim().eq_ignore_ascii_case(target.email.trim())
                        && prev_email
                            .as_deref()
                            .map(|pe| !c.email.trim().eq_ignore_ascii_case(pe.trim()))
                            .unwrap_or(true)
                })
                .map(|c| c.email)
        });
    let mut target_q_4h = ctx.as_ref().and_then(|c| c.target_quota_4h);
    let mut target_q_weekly = ctx.as_ref().and_then(|c| c.target_quota_weekly);
    let credit_before = ctx.as_ref().and_then(|c| c.credit_before_switch);
    let thresh = ctx.as_ref().and_then(|c| c.threshold_activated);

    if target_q_4h.is_none() || target_q_weekly.is_none() {
        if let Ok(t_acc) = account::load_account(&target.account_id) {
            let (q4, qw) = extract_dual_window_quotas(&t_acc, "gemini-2.5-pro");
            if target_q_4h.is_none() {
                target_q_4h = q4;
            }
            if target_q_weekly.is_none() {
                target_q_weekly = qw;
            }
        }
    }

    if (prev_q_4h.is_none() || prev_q_weekly.is_none()) && prev_email.is_some() {
        if let Ok(accounts) = account::list_accounts() {
            if let Some(p_acc) = accounts
                .iter()
                .find(|a| Some(&a.email) == prev_email.as_ref())
            {
                let (q4, qw) = extract_dual_window_quotas(p_acc, "gemini-2.5-pro");
                if prev_q_4h.is_none() {
                    prev_q_4h = q4;
                }
                if prev_q_weekly.is_none() {
                    prev_q_weekly = qw;
                }
            }
        }
    }

    let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();
    let mut seen_projs = std::collections::HashSet::new();
    let proj_names: Vec<String> = running_projs
        .into_iter()
        .map(|p| p.repo_name)
        .filter(|n| !n.is_empty() && seen_projs.insert(n.clone()))
        .collect();
    let backup_count_opt = backup_res.as_ref().ok().copied();

    let _notify = crate::modules::notification_hub::notify_account_switched_details(
        crate::modules::notification_hub::SwitchNotificationDetails {
            previous_email: prev_email,
            previous_quota_4h: prev_q_4h,
            previous_quota_weekly: prev_q_weekly,
            predicted_next_email: predicted_opt,
            selected_email: target.email.clone(),
            target_quota_4h: target_q_4h,
            target_quota_weekly: target_q_weekly,
            credit_before_switch: credit_before,
            threshold_activated: thresh,
            instance_id: inst_id.clone(),
            instance_name: inst_id.clone(),
            instance_mode: String::new(),
            reason: reason.clone(),
            is_auto: true,
            backed_up_projects: proj_names.clone(),
            backed_up_prompts_count: backup_count_opt,
            restored_prompts_count: None,
        },
    )
    .await;

    // Delegate directly to the exact account switch pipeline (the exact ⇄ button handler)
    let app_handle_opt = crate::modules::log_bridge::get_app_handle();
    let integration = match app_handle_opt.as_ref() {
        Some(h) => crate::modules::integration::SystemManager::Desktop(h.clone()),
        None => crate::modules::integration::SystemManager::Headless,
    };

    let app_config = config::load_app_config().unwrap_or_default();

    // Snapshot active workspace folders from current_instance_id before closing it
    let source_data_dir = crate::modules::instance::load_registry()
        .ok()
        .and_then(|r| {
            r.instances
                .into_iter()
                .find(|i| i.id == current_instance_id)
        })
        .map(|i| i.data_dir)
        .unwrap_or_else(|| {
            crate::modules::instance::get_default_antigravity_data_dir()
                .to_string_lossy()
                .to_string()
        });
    let active_workspaces = crate::modules::instance::get_instance_workspace_folders(
        &current_instance_id,
        &source_data_dir,
    );

    if target.instance_id != current_instance_id {
        logger::log_info(&format!(
            "[AutoSwitcher] Cross-instance rotation: rotating from '{}' to '{}' (account '{}', email '{}')",
            current_instance_id, target.instance_id, target.account_id, target.email
        ));
        // a) Save in-flight prompts
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::repo_db::requeue_running_conversations_for_instance(
                &current_instance_id,
            ),
            "requeue_running_conversations_for_instance",
        );
        // b) Inherit/copy workspace projects from depleted instance to target instance
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::instance::copy_instance_projects(
                &current_instance_id,
                &target.instance_id,
            ),
            "copy_instance_projects",
        );
        for ws in &active_workspaces {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::instance::assign_project_to_instance(&target.instance_id, ws),
                "assign_project_to_instance",
            );
        }
        // c) Close the depleted instance
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::instance::close_instance(&current_instance_id),
            "close_instance",
        );
        // d) Update active instance pointer
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            crate::modules::instance::set_active_instance_id(&target.instance_id),
            "set_active_instance_id",
        );
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::instance::bind_account_to_instance(
                &target.instance_id,
                &target.account_id,
                &target.email,
            ),
            "bind_account_to_instance",
        );
        if target.instance_id == "default" {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::account::set_current_account_id(&target.account_id),
                "set_current_account_id",
            );
        }
        // e) Ensure target instance credentials are fully injected and instance is launched
        let _ = crate::modules::instance::switch_account_to_instance(
            &target.account_id,
            Some(&target.instance_id),
        )
        .await?;

        if !app_config.auto_profile_switcher.auto_reopen_on_switch {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::instance::close_instance(&target.instance_id),
                "close_instance",
            );
        }
    } else {
        logger::log_info(&format!(
            "[AutoSwitcher] Same-instance rotation on '{}' to account '{}' (email '{}')",
            current_instance_id, target.account_id, target.email
        ));
        if inst_id == "default" {
            let service = crate::modules::account_service::AccountService::new(integration);
            service.switch_account(&target.account_id, None).await?;
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::instance::bind_account_to_instance(
                    "default",
                    &target.account_id,
                    &target.email,
                ),
                "bind_account_to_instance",
            );
        } else {
            instance::switch_account_to_instance(&target.account_id, Some(inst_id)).await?;
        }

        if !app_config.auto_profile_switcher.auto_reopen_on_switch {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::instance::close_instance(&current_instance_id),
                "close_instance",
            );
        }
    }

    // Emit event to frontend so UI reflects the button activation and refreshes state
    if let Some(ref handle) = app_handle_opt {
        #[derive(serde::Serialize, Clone)]
        struct AutoSwitchPayload {
            account_id: String,
            email: String,
            instance_id: String,
        }
        use tauri::Emitter;
        // Justification: best-effort frontend event; a dropped event only skips a UI refresh
        crate::error::record_ignored(
            handle.emit(
                "account://auto-switched",
                AutoSwitchPayload {
                    account_id: target.account_id.clone(),
                    email: target.email.clone(),
                    instance_id: inst_id.clone(),
                },
            ),
            "emit event",
        );
        // Justification: best-effort frontend event; a dropped event only skips a UI refresh
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts://refreshed",
        );
        // Justification: best-effort frontend event; a dropped event only skips a UI refresh
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
    }

    // Step 2.5: Acquire distributed lease in Supabase Root DB (prevent other nodes from selecting it)
    let target_acc_id = target.account_id.clone();
    let target_inst_id = target.instance_id.clone();
    let lease_ttl = crate::modules::workspace_lease_manager::get_default_lease_ttl_secs();
    tauri::async_runtime::spawn(async move {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::workspace_lease_manager::acquire_lease(
                &target_acc_id,
                &target_inst_id,
                lease_ttl,
            )
            .await,
            "acquire_lease",
        );
    });

    // Step 3: Asynchronous 5-second post-launch prompt re-injection and status notification
    let target_inst_id_clone = inst_id.clone();
    let proj_names_clone = proj_names.clone();
    let backed_up_count_val = backed_up_count;
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let resent_count = match crate::modules::repo_db::resend_running_commands_for_instance(
            if target_inst_id_clone == "default" {
                None
            } else {
                Some(&target_inst_id_clone)
            },
            20,
        ) {
            Ok(resent) => {
                logger::log_info(&format!(
                    "[AutoSwitcher] Post-switch re-injected {} backed-up running prompts across workspaces",
                    resent.len()
                ));
                resent.len()
            }
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Failed to resend backed-up running commands after switch: {}",
                    e
                ));
                0
            }
        };

        let dispatched_count =
            crate::modules::repo_db::dispatch_running_prompts(&target_inst_id_clone).unwrap_or(0);
        let verified_running = crate::modules::repo_db::verify_prompts_running();
        logger::log_info(&format!(
            "[AutoSwitcher] Prompt restoration complete: {} resent, {} dispatched, {} verified active running for instance '{}'",
            resent_count, dispatched_count, verified_running, target_inst_id_clone
        ));

        crate::modules::notification_hub::notify_post_switch_prompt_status(
            &target_inst_id_clone,
            backed_up_count_val,
            resent_count + dispatched_count,
            verified_running,
            proj_names_clone,
        );
    });

    // Auto-resume recent active prompts (<1h) if configured
    let app_config = config::load_app_config().unwrap_or_default();
    if app_config.auto_profile_switcher.auto_resume_recent_prompts {
        let threshold = app_config
            .auto_profile_switcher
            .prompt_recency_threshold_seconds as i64;
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            crate::modules::repo_db::auto_resume_recent_prompts(inst_id, threshold),
            "auto_resume_recent_prompts",
        );
    }

    let now = chrono::Utc::now().timestamp();
    let mut state = RUNTIME_STATE.lock().unwrap();
    state.last_switch_timestamp = Some(now);
    state.last_switch_reason = Some(reason);

    Ok(())
}

/// Execute rotation to target candidate (backward-compatible wrapper)
pub async fn execute_profile_rotation(
    target: ProfileCandidate,
    reason: String,
    has_auto_resume: bool,
) -> Result<(), String> {
    execute_profile_rotation_with_context(target, reason, has_auto_resume, None).await
}
