use crate::modules::{account, config, instance, logger};

use super::*;

/// Trigger manual rotation to the next best profile for a specific instance (or active instance if None)
pub async fn trigger_manual_rotation_for_instance(
    target_instance_id: Option<&str>,
) -> Result<String, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;
    let inst_id = match target_instance_id {
        Some(id) => instance::resolve_instance_id(id)?,
        None => instance::get_active_instance_id()?,
    };

    // Step 0: Ensure running prompts are snapshotted and backed up before rotation starts
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::repo_db::backup_running_prompts(&inst_id),
        "backup_running_prompts",
    );
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::backup_prompts_db::backup_active_running_prompts(Some(&inst_id), None),
        "backup_active_running_prompts",
    );
    let registry = instance::load_registry()?;
    let current_bound = registry
        .instances
        .iter()
        .find(|i| i.id == inst_id)
        .and_then(|i| i.bound_account_id.clone())
        .or_else(|| account::get_current_account_id().ok().flatten());

    let mut excluded: Vec<String> = Vec::new();
    if let Some(ref curr) = current_bound {
        if !excluded.contains(curr) {
            excluded.push(curr.clone());
        }
        if let Ok(acc) = account::load_account(curr) {
            if !excluded.contains(&acc.email) {
                excluded.push(acc.email.clone());
            }
        }
    }
    if let Some(target) = registry.instances.iter().find(|i| i.id == inst_id) {
        if let Some(ref email) = target.bound_email {
            if !excluded.contains(email) {
                excluded.push(email.clone());
            }
            if let Ok(Some(acc)) = account::get_account_by_email(email) {
                if !excluded.contains(&acc.id) {
                    excluded.push(acc.id.clone());
                }
            }
        }
    }
    for inst in &registry.instances {
        let is_running_sibling =
            inst.id != inst_id && instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_running_sibling {
            let active_acc_id = inst.bound_account_id.clone().or_else(|| {
                let is_default_instance =
                    inst.is_default || inst.id == "default" || inst.id == "__default__";
                if is_default_instance {
                    account::get_current_account_id().ok().flatten()
                } else {
                    None
                }
            });
            if let Some(ref acc_id) = active_acc_id {
                if !excluded.contains(acc_id) {
                    excluded.push(acc_id.clone());
                }
                if let Ok(acc) = account::load_account(acc_id) {
                    if !excluded.contains(&acc.email) {
                        excluded.push(acc.email.clone());
                    }
                }
            }
            if let Some(ref email) = inst.bound_email {
                if !excluded.contains(email) {
                    excluded.push(email.clone());
                }
                if let Ok(Some(acc)) = account::get_account_by_email(email) {
                    if !excluded.contains(&acc.id) {
                        excluded.push(acc.id.clone());
                    }
                }
            }
        }
    }

    let remote_cluster_accs = crate::modules::supabase_sync::get_remote_cluster_active_accounts();
    for r_acc in &remote_cluster_accs {
        if !excluded.iter().any(|x| x.eq_ignore_ascii_case(r_acc)) {
            excluded.push(r_acc.clone());
        }
        if let Ok(Some(acc)) = account::get_account_by_email(r_acc) {
            if !excluded.iter().any(|x| x.eq_ignore_ascii_case(&acc.id)) {
                excluded.push(acc.id.clone());
            }
        }
    }

    let candidate = select_and_verify_next_best_profile(
        &inst_id,
        &switcher_cfg.target_model,
        switcher_cfg.low_quota_threshold_percent,
        &excluded,
    )
    .await?
    .ok_or_else(|| "No alternative healthy profile found in pool".to_string())?;

    let reason = "Manual rotation triggered by user".to_string();
    let email = candidate.email.clone();
    let effective_inst = candidate.instance_id.clone();

    let (prev_email, prev_4h, prev_weekly) = match current_bound
        .as_ref()
        .and_then(|id| account::load_account(id).ok())
    {
        Some(acc) => {
            let (q4, qw) = extract_dual_window_quotas(&acc, &switcher_cfg.target_model);
            if acc
                .email
                .trim()
                .eq_ignore_ascii_case(candidate.email.trim())
            {
                (None, q4, qw)
            } else {
                (Some(acc.email), q4, qw)
            }
        }
        None => (None, None, None),
    };

    let (target_4h, target_weekly) =
        if let Ok(cand_acc) = account::load_account(&candidate.account_id) {
            extract_dual_window_quotas(&cand_acc, &switcher_cfg.target_model)
        } else {
            (None, None)
        };

    let mut pred_exclusions = excluded.clone();
    pred_exclusions.push(candidate.account_id.clone());
    pred_exclusions.push(candidate.email.clone());
    if let Some(ref pe) = prev_email {
        pred_exclusions.push(pe.clone());
    }
    let predicted_candidate = select_candidate_profiles(
        &inst_id,
        &switcher_cfg.target_model,
        switcher_cfg.low_quota_threshold_percent,
        &pred_exclusions,
    )
    .ok()
    .and_then(|v| v.into_iter().next())
    .filter(|c| {
        !c.email.trim().eq_ignore_ascii_case(candidate.email.trim())
            && prev_email
                .as_deref()
                .map(|pe| !c.email.trim().eq_ignore_ascii_case(pe.trim()))
                .unwrap_or(true)
    });
    let predicted_email = predicted_candidate.map(|c| c.email);

    let rot_ctx = RotationContext {
        current_instance_id: Some(inst_id.clone()),
        previous_email: prev_email,
        previous_quota_4h: prev_4h,
        previous_quota_weekly: prev_weekly,
        predicted_email,
        target_quota_4h: target_4h,
        target_quota_weekly: target_weekly,
        credit_before_switch: prev_4h,
        threshold_activated: Some(switcher_cfg.low_quota_threshold_percent),
    };

    execute_profile_rotation_with_context(
        candidate,
        reason,
        switcher_cfg.has_auto_resume,
        Some(rot_ctx),
    )
    .await?;

    let selection_tier_desc = if candidate.rolling_4h_quota >= 99.5 {
        "Primary Tier (100% 4h headroom, earliest reset)"
    } else {
        "Fallback Tier (highest combined credits)"
    };
    let remote_excl_cnt = remote_cluster_accs.len();
    Ok(format!(
        "Successfully rotated profile '{}' to account '{}' [{}] (Excluded {} cluster-active account(s))",
        effective_inst, email, selection_tier_desc, remote_excl_cnt
    ))
}

/// Trigger manual rotation to the next best profile
pub async fn trigger_manual_rotation() -> Result<String, String> {
    trigger_manual_rotation_for_instance(None).await
}
