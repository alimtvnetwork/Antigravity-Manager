use crate::modules::{account, config, instance, logger};

use super::*;

/// Check all monitored instance copies' quota and auto-rotate any instance below threshold
pub async fn check_and_rotate_with_options(
    custom_threshold: Option<f64>,
    force: bool,
) -> Result<Option<String>, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;

    let has_custom = custom_threshold.is_some();
    if !switcher_cfg.is_enabled && !force && !has_custom {
        return Ok(None);
    }

    let mut monitored_instances = list_running_or_active_instances().unwrap_or_default();
    if monitored_instances.is_empty() {
        if let Ok(registry) = instance::load_registry() {
            if let Some(def) = registry
                .instances
                .iter()
                .find(|i| i.is_default || i.id == "default")
            {
                let mut def_inst = def.clone();
                if def_inst.bound_account_id.is_none() {
                    def_inst.bound_account_id = account::get_current_account_id().ok().flatten();
                }
                monitored_instances.push(def_inst);
            }
        }
        if monitored_instances.is_empty() {
            if let Ok(Some(current_acc_id)) = account::get_current_account_id() {
                monitored_instances.push(crate::models::instance::InstanceConfig {
                    id: "default".to_string(),
                    name: "Default Workspace".to_string(),
                    data_dir: instance::get_default_antigravity_data_dir()
                        .to_string_lossy()
                        .to_string(),
                    executable_path: None,
                    extensions_dir: None,
                    bound_account_id: Some(current_acc_id),
                    bound_email: None,
                    created_at: 0,
                    last_used: 0,
                    is_default: true,
                    pid: None,
                    seq_num: Some(1),
                });
            }
        }
    }
    if monitored_instances.is_empty() {
        return Ok(None);
    }

    let mut in_use_account_ids = get_active_in_use_account_ids();
    let now = chrono::Utc::now().timestamp();
    {
        let mut state = RUNTIME_STATE.lock().unwrap();
        state.last_check_timestamp = now;
    }

    let effective_low_threshold = custom_threshold
        .unwrap_or(switcher_cfg.low_quota_threshold_percent)
        .clamp(0.0, 99.0);

    let mut rotated_reasons = Vec::new();

    for inst in monitored_instances {
        let bound_acc_id_opt = if inst.id == "default" || inst.is_default {
            account::get_current_account_id()
                .ok()
                .flatten()
                .or_else(|| inst.bound_account_id.clone())
        } else {
            inst.bound_account_id
                .clone()
                .or_else(|| account::get_current_account_id().ok().flatten())
        };

        let Some(bound_acc_id) = bound_acc_id_opt else {
            continue;
        };

        let mut bound_acc = match account::load_account(&bound_acc_id) {
            Ok(acc) => acc,
            Err(_) => continue,
        };

        // Live Quota Refresh from Google API before threshold evaluation
        if let Ok(fresh_quota) = account::fetch_quota_with_retry(&mut bound_acc).await {
            bound_acc.quota = Some(fresh_quota);
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(account::save_account(&bound_acc), "save_account");
        }

        let period_status = evaluate_account_period_status(
            &bound_acc,
            &switcher_cfg.target_model,
            effective_low_threshold,
            now,
        );

        let quota_percent = period_status
            .as_ref()
            .map(|s| s.quota_percent)
            .or_else(|| calculate_4h_window_quota(&bound_acc, &switcher_cfg.target_model))
            .or_else(|| calculate_account_quota(&bound_acc, &switcher_cfg.target_model))
            .unwrap_or(100.0);

        let below_threshold = period_status
            .as_ref()
            .map(|s| s.below_threshold)
            .unwrap_or(false)
            || quota_percent <= effective_low_threshold
            || quota_percent <= switcher_cfg.critical_threshold_percent;
        if below_threshold || force {
            // Justification: non-Result return value intentionally discarded — no error channel to track
            let _ = instance::resolve_instance_pid_for_switch(
                &inst.id,
                &inst.data_dir,
                inst.is_default || inst.id == "default",
            );
        }

        let is_depleted_before_finish = period_status
            .as_ref()
            .map(|s| s.is_depleted_before_finish)
            .unwrap_or(false);

        let is_period_finished = period_status
            .as_ref()
            .map(|s| s.is_period_finished)
            .unwrap_or(false);

        // Filter out accounts in use by ANY running/active instances, AND explicitly exclude the current bound account to rotate away from it
        let mut excluded_accounts: Vec<String> = in_use_account_ids.clone();
        if !excluded_accounts.contains(&bound_acc_id) {
            excluded_accounts.push(bound_acc_id.clone());
        }
        if !excluded_accounts.contains(&bound_acc.email) {
            excluded_accounts.push(bound_acc.email.clone());
        }

        // 1. Critical threshold check or forced rotation
        let is_critical = quota_percent <= switcher_cfg.critical_threshold_percent;
        let should_force_or_critical =
            force || (is_critical && switcher_cfg.auto_fast_forward_on_critical);
        if should_force_or_critical {
            let reason = if force {
                format!(
                    "Forced rotation on instance '{}' (active email: {}, quota: {:.1}%)",
                    inst.id, bound_acc.email, quota_percent
                )
            } else {
                format!(
                    "Critical quota alert on instance '{}': model '{}' dropped to {:.1}% (<= {:.1}%). Fast-forwarding to highest credit candidate...",
                    inst.id, switcher_cfg.target_model, quota_percent, switcher_cfg.critical_threshold_percent
                )
            };
            logger::log_warn(&format!("[AutoSwitcher] {}", reason));

            if let Some(candidate) = select_and_verify_next_best_profile(
                &inst.id,
                &switcher_cfg.target_model,
                switcher_cfg.critical_threshold_percent,
                &excluded_accounts,
            )
            .await?
            {
                let previous_email_opt = if bound_acc
                    .email
                    .trim()
                    .eq_ignore_ascii_case(candidate.email.trim())
                {
                    None
                } else {
                    Some(bound_acc.email.clone())
                };

                let (prev_4h, prev_weekly) = if previous_email_opt.is_some() {
                    extract_dual_window_quotas(&bound_acc, &switcher_cfg.target_model)
                } else {
                    (None, None)
                };
                let (target_4h, target_weekly) =
                    if let Ok(cand_acc) = account::load_account(&candidate.account_id) {
                        extract_dual_window_quotas(&cand_acc, &switcher_cfg.target_model)
                    } else {
                        (None, None)
                    };

                let mut pred_exclusions = excluded_accounts.clone();
                pred_exclusions.push(candidate.account_id.clone());
                pred_exclusions.push(candidate.email.clone());
                if let Some(ref pe) = previous_email_opt {
                    pred_exclusions.push(pe.clone());
                }
                let predicted_candidate = select_candidate_profiles(
                    &inst.id,
                    &switcher_cfg.target_model,
                    switcher_cfg.critical_threshold_percent,
                    &pred_exclusions,
                )
                .ok()
                .and_then(|v| v.into_iter().next())
                .filter(|c| {
                    !c.email.trim().eq_ignore_ascii_case(candidate.email.trim())
                        && previous_email_opt
                            .as_deref()
                            .map(|pe| !c.email.trim().eq_ignore_ascii_case(pe.trim()))
                            .unwrap_or(true)
                });
                let predicted_email = predicted_candidate.map(|c| c.email);

                let rot_ctx = RotationContext {
                    current_instance_id: Some(inst.id.clone()),
                    previous_email: previous_email_opt,
                    previous_quota_4h: prev_4h,
                    previous_quota_weekly: prev_weekly,
                    predicted_email,
                    target_quota_4h: target_4h,
                    target_quota_weekly: target_weekly,
                    credit_before_switch: Some(quota_percent),
                    threshold_activated: Some(switcher_cfg.critical_threshold_percent),
                };
                let rotated_acc_id = candidate.account_id.clone();
                let rotated_email = candidate.email.clone();
                execute_profile_rotation_with_context(
                    candidate,
                    reason.clone(),
                    switcher_cfg.has_auto_resume,
                    Some(rot_ctx),
                )
                .await?;
                in_use_account_ids.push(rotated_acc_id);
                in_use_account_ids.push(rotated_email);
                rotated_reasons.push(reason);
                continue;
            }
        }

        // 2. Standard low-quota or depleted before finish check
        let is_low_quota = quota_percent <= effective_low_threshold || below_threshold;
        let should_rotate = is_depleted_before_finish || is_low_quota;

        if !should_rotate && is_period_finished {
            logger::log_info(&format!(
                "[AutoSwitcher] Instance '{}' quota period finished (reset time reached). Quota is healthy ({:.1}%).",
                inst.id, quota_percent
            ));
            continue;
        }

        if should_rotate {
            let reason = if is_depleted_before_finish {
                let secs_left = period_status
                    .as_ref()
                    .and_then(|s| s.seconds_until_reset)
                    .unwrap_or(0);
                format!(
                    "Quota depleted before period finish on instance '{}': model '{}' at {:.1}% with {}s until reset",
                    inst.id, switcher_cfg.target_model, quota_percent, secs_left
                )
            } else {
                format!(
                    "Quota for model '{}' on instance '{}' dropped to {:.1}% (threshold: {:.1}%)",
                    switcher_cfg.target_model, inst.id, quota_percent, effective_low_threshold
                )
            };

            logger::log_info(&format!("[AutoSwitcher] {}", reason));

            if let Some(candidate) = select_and_verify_next_best_profile(
                &inst.id,
                &switcher_cfg.target_model,
                effective_low_threshold,
                &excluded_accounts,
            )
            .await?
            {
                let previous_email_opt = if bound_acc
                    .email
                    .trim()
                    .eq_ignore_ascii_case(candidate.email.trim())
                {
                    None
                } else {
                    Some(bound_acc.email.clone())
                };

                let (prev_4h, prev_weekly) = if previous_email_opt.is_some() {
                    extract_dual_window_quotas(&bound_acc, &switcher_cfg.target_model)
                } else {
                    (None, None)
                };
                let (target_4h, target_weekly) =
                    if let Ok(cand_acc) = account::load_account(&candidate.account_id) {
                        extract_dual_window_quotas(&cand_acc, &switcher_cfg.target_model)
                    } else {
                        (None, None)
                    };

                let mut pred_exclusions = excluded_accounts.clone();
                pred_exclusions.push(candidate.account_id.clone());
                pred_exclusions.push(candidate.email.clone());
                if let Some(ref pe) = previous_email_opt {
                    pred_exclusions.push(pe.clone());
                }
                let predicted_candidate = select_candidate_profiles(
                    &inst.id,
                    &switcher_cfg.target_model,
                    effective_low_threshold,
                    &pred_exclusions,
                )
                .ok()
                .and_then(|v| v.into_iter().next())
                .filter(|c| {
                    !c.email.trim().eq_ignore_ascii_case(candidate.email.trim())
                        && previous_email_opt
                            .as_deref()
                            .map(|pe| !c.email.trim().eq_ignore_ascii_case(pe.trim()))
                            .unwrap_or(true)
                });
                let predicted_email = predicted_candidate.map(|c| c.email);

                let rot_ctx = RotationContext {
                    current_instance_id: Some(inst.id.clone()),
                    previous_email: previous_email_opt,
                    previous_quota_4h: prev_4h,
                    previous_quota_weekly: prev_weekly,
                    predicted_email,
                    target_quota_4h: target_4h,
                    target_quota_weekly: target_weekly,
                    credit_before_switch: Some(quota_percent),
                    threshold_activated: Some(effective_low_threshold),
                };
                let rotated_acc_id = candidate.account_id.clone();
                let rotated_email = candidate.email.clone();
                execute_profile_rotation_with_context(
                    candidate,
                    reason.clone(),
                    switcher_cfg.has_auto_resume,
                    Some(rot_ctx),
                )
                .await?;
                in_use_account_ids.push(rotated_acc_id);
                in_use_account_ids.push(rotated_email);
                rotated_reasons.push(reason);
            } else {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Instance '{}' quota low ({:.1}%) but no healthy alternative profile found in pool.",
                    inst.id, quota_percent
                ));
            }
        }
    }

    if rotated_reasons.is_empty() {
        Ok(None)
    } else {
        Ok(Some(rotated_reasons.join("; ")))
    }
}

pub async fn check_and_rotate_if_needed() -> Result<Option<String>, String> {
    check_and_rotate_with_options(None, false).await
}

/// Proactively inspect active account and running instances on startup, rotating immediately if quota is depleted or below threshold
pub async fn evaluate_and_execute_startup_rotation() -> Result<Option<String>, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;
    if !switcher_cfg.is_enabled {
        logger::log_info(
            "[AutoSwitcher] Auto profile switcher is disabled, skipping startup check.",
        );
        return Ok(None);
    }
    logger::log_info("[AutoSwitcher] Proactive startup check: evaluating quota for active instances and default workspace...");
    check_and_rotate_with_options(None, false).await
}

pub async fn check_and_rotate_for_threshold(
    custom_threshold: Option<f64>,
    force: bool,
) -> Result<Option<String>, String> {
    check_and_rotate_with_options(custom_threshold, force).await
}
