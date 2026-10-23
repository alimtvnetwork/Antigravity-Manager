use crate::models::Account;
use crate::modules::{account, config, instance, logger};

use super::*;

/// Robust check if an account's quota reset period has completed for the target model or buckets
pub(crate) fn is_account_period_finished(acc: &Account, target_model: &str, now_sec: i64) -> bool {
    let period_stat = evaluate_account_period_status(acc, target_model, 20.0, now_sec);
    if let Some(ref stat) = period_stat {
        if stat.is_period_finished {
            return true;
        }
    }
    // Check quota_groups short-window and weekly buckets directly
    if let Some(ref quota_data) = acc.quota {
        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    if !b.reset_time.is_empty() {
                        if let Some(ts) = parse_reset_time_to_unix(&b.reset_time) {
                            if ts <= now_sec {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        for m in &quota_data.models {
            if is_model_matching_target(&m.name, target_model) && !m.reset_time.is_empty() {
                if let Some(ts) = parse_reset_time_to_unix(&m.reset_time) {
                    if ts <= now_sec {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if account is in proxy rate limit cooldown for target model (or account-wide 429)
pub(crate) fn is_account_rate_limited(acc: &Account, target_model: &str, now_sec: i64) -> bool {
    let target = target_model.trim().to_lowercase();
    acc.live_limited_models.iter().any(|(model, status)| {
        if status.until <= now_sec {
            return false;
        }
        let m = model.trim().to_lowercase();
        m.is_empty()
            || m == target
            || (!target.is_empty() && (m.contains(&target) || target.contains(&m)))
            || (target.contains("flash") && m.contains("flash"))
            || (target.contains("pro") && m.contains("pro"))
    })
}

/// Check if validation block on account is active
pub(crate) fn is_account_validation_blocked(acc: &Account, now_sec: i64) -> bool {
    if !acc.validation_blocked {
        return false;
    }
    match acc.validation_blocked_until {
        Some(until) => until > now_sec,
        None => true,
    }
}

/// Check if an account (by ID or email) is actively bound to or active on any running sibling instance on this host
pub(crate) fn is_account_bound_to_running_sibling(
    registry: &crate::models::InstanceRegistry,
    current_instance_id: &str,
    account_id: &str,
    account_email: &str,
) -> bool {
    registry.instances.iter().any(|i| {
        let is_running_sibling = i.id != current_instance_id
            && crate::modules::instance::is_instance_running(&i.id, &i.data_dir, i.pid);
        if !is_running_sibling {
            return false;
        }
        let is_bound_id_match = i.bound_account_id.as_deref() == Some(account_id);
        let is_bound_email_match = i
            .bound_email
            .as_deref()
            .map(|e| e.eq_ignore_ascii_case(account_email))
            .unwrap_or(false);
        let is_default_instance = i.is_default || i.id == "default" || i.id == "__default__";
        let is_default_match = is_default_instance
            && account::get_current_account_id().ok().flatten().as_deref() == Some(account_id);

        if is_bound_id_match || is_bound_email_match || is_default_match {
            return true;
        }

        if let Some(ref bid) = i.bound_account_id {
            if let Ok(bacc) = account::load_account(bid) {
                if bacc.email.eq_ignore_ascii_case(account_email) {
                    return true;
                }
            }
        }

        false
    })
}

/// Discover, score, and rank candidate profiles with 100% 4h quota accounts prioritized
pub fn select_candidate_profiles(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
    excluded_account_ids: &[String],
) -> Result<Vec<ProfileCandidate>, String> {
    let registry = instance::load_registry()?;
    let now_sec = chrono::Utc::now().timestamp();
    let app_config = config::load_app_config();
    let cooldown_minutes = app_config
        .as_ref()
        .map(|c| c.auto_profile_switcher.account_cooldown_minutes)
        .unwrap_or(60);
    let cooldown_secs: i64 = (cooldown_minutes as i64) * 60;

    let mut effective_exclusions: Vec<String> = excluded_account_ids
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut push_exclusion = |val: &str| {
        let clean = val.trim().to_lowercase();
        if !clean.is_empty() && !effective_exclusions.contains(&clean) {
            effective_exclusions.push(clean);
        }
    };

    for in_use_id in get_active_in_use_account_ids() {
        push_exclusion(&in_use_id);
    }
    // Strictly exclude the current instance's own account so it cannot rotate to itself
    if let Some(cur_inst) = registry
        .instances
        .iter()
        .find(|i| i.id == current_instance_id)
    {
        if let Some(ref acc_id) = cur_inst.bound_account_id {
            push_exclusion(acc_id);
            if let Ok(acc) = account::load_account(acc_id) {
                push_exclusion(&acc.email);
            }
        }
        if let Some(ref email) = cur_inst.bound_email {
            push_exclusion(email);
            if let Ok(Some(acc)) = account::get_account_by_email(email) {
                push_exclusion(&acc.id);
            }
        }
        let is_default =
            cur_inst.is_default || cur_inst.id == "default" || cur_inst.id == "__default__";
        if is_default {
            if let Ok(Some(def_id)) = account::get_current_account_id() {
                push_exclusion(&def_id);
                if let Ok(acc) = account::load_account(&def_id) {
                    push_exclusion(&acc.email);
                }
            }
        }
    }
    for inst in &registry.instances {
        let is_running_sibling = inst.id != current_instance_id
            && crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
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
                push_exclusion(acc_id);
                if let Ok(acc) = account::load_account(acc_id) {
                    push_exclusion(&acc.email);
                }
            }
            if let Some(ref email) = inst.bound_email {
                push_exclusion(email);
                if let Ok(Some(acc)) = account::get_account_by_email(email) {
                    push_exclusion(&acc.id);
                }
            }
        }
    }
    for cross_vm_acc in crate::modules::email_inbound::fetch_recent_cross_vm_switched_accounts(3600)
    {
        push_exclusion(&cross_vm_acc);
    }

    // Helper closure to evaluate if an account is in cooldown window
    let is_in_cooldown = |acc: &account::Account| -> bool {
        let is_recently_used = acc.last_used > 0 && (now_sec - acc.last_used) < cooldown_secs;
        let has_remote_lease = if let Some(lease) =
            crate::modules::workspace_lease_manager::find_cached_lease(&acc.id, &acc.email)
        {
            (lease.leased_at > 0 && (now_sec - lease.leased_at) < cooldown_secs)
                || lease.expires_at > now_sec
        } else {
            false
        };
        is_recently_used || has_remote_lease
    };

    let mut available_pool: Vec<ProfileCandidate> = Vec::new();
    let mut cooldown_pool: Vec<(ProfileCandidate, i64)> = Vec::new();
    let mut seen_account_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. Inspect instances not in active use
    for inst in &registry.instances {
        if inst.id == current_instance_id {
            continue;
        }

        // If instance is running on this machine, never steal its account
        if crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
            continue;
        }

        let Some(ref acc_id) = inst.bound_account_id else {
            continue;
        };

        if is_account_excluded(
            &effective_exclusions,
            acc_id,
            inst.bound_email.as_deref().unwrap_or(""),
        ) {
            continue;
        }

        if let Ok(acc) = account::load_account(acc_id) {
            if is_account_bound_to_running_sibling(
                &registry,
                current_instance_id,
                &acc.id,
                &acc.email,
            ) {
                continue;
            }
            if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
                continue;
            }
            if acc.disabled || acc.proxy_disabled || is_account_validation_blocked(&acc, now_sec) {
                continue;
            }
            if is_account_rate_limited(&acc, target_model, now_sec) {
                logger::log_info(&format!(
                    "[AutoSwitcher] Bound account '{}' is currently in proxy rate-limit cooldown for model '{}'. Skipping...",
                    acc.email, target_model
                ));
                continue;
            }
            // Strictly skip accounts leased by another active machine/node
            if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
                &acc.id, &acc.email,
            ) {
                continue;
            }

            let quota = calculate_candidate_quota(&acc, target_model).unwrap_or(0.0);

            // Strict 100% 4-Hour Quota Gate:
            // Candidate accounts in primary pool must genuinely have 100% quota and score > 0.
            if quota >= 100.0 {
                let score = score_candidate_account(&acc, target_model, now_sec);
                if score > 0.0 {
                    seen_account_ids.insert(acc.id.clone());
                    let candidate = ProfileCandidate {
                        instance_id: current_instance_id.to_string(),
                        account_id: acc.id.clone(),
                        email: acc.email.clone(),
                        quota_percent: quota,
                        score,
                    };
                    if is_in_cooldown(&acc) {
                        cooldown_pool.push((candidate, acc.last_used));
                    } else {
                        available_pool.push(candidate);
                    }
                }
            }
        }
    }

    // 2. Check unbound accounts in pool
    let all_accounts = account::list_accounts().unwrap_or_default();
    for acc in &all_accounts {
        if seen_account_ids.contains(&acc.id) {
            continue;
        }
        if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
            continue;
        }
        if acc.disabled || acc.proxy_disabled || is_account_validation_blocked(acc, now_sec) {
            continue;
        }
        if is_account_rate_limited(acc, target_model, now_sec) {
            logger::log_info(&format!(
                "[AutoSwitcher] Account '{}' is currently in proxy rate-limit cooldown for model '{}'. Skipping...",
                acc.email, target_model
            ));
            continue;
        }
        // Strictly skip accounts leased by another active machine/node
        if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
            &acc.id, &acc.email,
        ) {
            continue;
        }

        // Check if bound to or active on any running sibling instance on this host
        if is_account_bound_to_running_sibling(&registry, current_instance_id, &acc.id, &acc.email)
        {
            continue;
        }

        let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);

        // Strict 100% 4-Hour Quota Gate
        if quota >= 100.0 {
            let score = score_candidate_account(acc, target_model, now_sec);
            if score > 0.0 {
                seen_account_ids.insert(acc.id.clone());
                let candidate = ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                };
                if is_in_cooldown(acc) {
                    cooldown_pool.push((candidate, acc.last_used));
                } else {
                    available_pool.push(candidate);
                }
            }
        }
    }

    // 3. Fallback: If no candidate met the 100% criteria, fall back to best available accounts above threshold
    if available_pool.is_empty() {
        for acc in &all_accounts {
            if seen_account_ids.contains(&acc.id) {
                continue;
            }
            if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
                continue;
            }
            if acc.disabled || acc.proxy_disabled || is_account_validation_blocked(acc, now_sec) {
                continue;
            }
            if is_account_rate_limited(acc, target_model, now_sec) {
                continue;
            }
            // Strictly skip accounts leased by another active machine/node
            if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
                &acc.id, &acc.email,
            ) {
                continue;
            }

            if is_account_bound_to_running_sibling(
                &registry,
                current_instance_id,
                &acc.id,
                &acc.email,
            ) {
                continue;
            }

            let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);
            if quota > threshold {
                let score = score_candidate_account_fallback(acc, target_model, now_sec);
                if score > 0.0 {
                    seen_account_ids.insert(acc.id.clone());
                    let candidate = ProfileCandidate {
                        instance_id: current_instance_id.to_string(),
                        account_id: acc.id.clone(),
                        email: acc.email.clone(),
                        quota_percent: quota,
                        score,
                    };
                    if is_in_cooldown(acc) {
                        cooldown_pool.push((candidate, acc.last_used));
                    } else {
                        available_pool.push(candidate);
                    }
                }
            }
        }
    }

    // 3b. Emergency Fallback: If no candidate met quota > threshold, fall back to any healthy account with headroom (> 5.0%)
    if available_pool.is_empty() && cooldown_pool.is_empty() {
        for acc in &all_accounts {
            if seen_account_ids.contains(&acc.id) {
                continue;
            }
            if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
                continue;
            }
            if acc.disabled || acc.proxy_disabled || is_account_validation_blocked(acc, now_sec) {
                continue;
            }
            if is_account_rate_limited(acc, target_model, now_sec) {
                continue;
            }
            if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
                &acc.id, &acc.email,
            ) {
                continue;
            }
            if is_account_bound_to_running_sibling(
                &registry,
                current_instance_id,
                &acc.id,
                &acc.email,
            ) {
                continue;
            }
            let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);
            if quota > 5.0 {
                let score = score_candidate_account_fallback(acc, target_model, now_sec);
                if score > 0.0 {
                    seen_account_ids.insert(acc.id.clone());
                    let candidate = ProfileCandidate {
                        instance_id: current_instance_id.to_string(),
                        account_id: acc.id.clone(),
                        email: acc.email.clone(),
                        quota_percent: quota,
                        score,
                    };
                    if is_in_cooldown(acc) {
                        cooldown_pool.push((candidate, acc.last_used));
                    } else {
                        available_pool.push(candidate);
                    }
                }
            }
        }
    }

    // 4. Graceful Fallback Logic:
    // If available pool is not empty, use ONLY the available pool.
    // If and ONLY IF available pool is completely empty (all healthy accounts are in cooldown),
    // fall back to the cooldown pool sorted by oldest `last_used` so the system never gets stuck.
    let selected_candidates = if !available_pool.is_empty() {
        // Priority Sorting for available pool:
        // Sort by subscription tier score descending (Ultra > Pro > Free).
        // Tie-breaker: deterministic email ordering.
        available_pool.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
            Some(std::cmp::Ordering::Equal) | None => {
                a.email.to_lowercase().cmp(&b.email.to_lowercase())
            }
            Some(ord) => ord,
        });
        available_pool
    } else if !cooldown_pool.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[AutoSwitcher] All {} healthy account(s) are currently in cooldown. Triggering graceful fallback to oldest last_used account.",
            cooldown_pool.len()
        ));
        // Imp-05: Cooldown Fallback Never-Used Priority
        // Sort by never-used first (last_used == 0), then by oldest last_used (ascending last_used timestamp: smallest first)
        cooldown_pool.sort_by(|(a_cand, a_last), (b_cand, b_last)| {
            match (*a_last == 0, *b_last == 0) {
                (true, false) => std::cmp::Ordering::Less, // a never used, prioritize a
                (false, true) => std::cmp::Ordering::Greater, // b never used, prioritize b
                (true, true) => match b_cand.score.partial_cmp(&a_cand.score) {
                    Some(std::cmp::Ordering::Equal) | None => a_cand
                        .email
                        .to_lowercase()
                        .cmp(&b_cand.email.to_lowercase()),
                    Some(ord) => ord,
                },
                (false, false) => match a_last.cmp(b_last) {
                    std::cmp::Ordering::Equal => match b_cand.score.partial_cmp(&a_cand.score) {
                        Some(std::cmp::Ordering::Equal) | None => a_cand
                            .email
                            .to_lowercase()
                            .cmp(&b_cand.email.to_lowercase()),
                        Some(ord) => ord,
                    },
                    ord => ord,
                },
            }
        });
        cooldown_pool.into_iter().map(|(cand, _)| cand).collect()
    } else {
        Vec::new()
    };

    Ok(selected_candidates)
}

/// Find next best candidate profile with healthy quota, prioritizing 100% quota accounts
pub fn select_next_best_profile(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
    excluded_account_ids: &[String],
) -> Result<Option<ProfileCandidate>, String> {
    let candidates = select_candidate_profiles(
        current_instance_id,
        target_model,
        threshold,
        excluded_account_ids,
    )?;
    Ok(candidates.into_iter().next())
}

/// Find and live-verify next best candidate profile with Google API refresh:
/// Strictly confirms candidate has 100% quota for 4-hour window before selecting!
pub async fn select_and_verify_next_best_profile(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
    excluded_account_ids: &[String],
) -> Result<Option<ProfileCandidate>, String> {
    // 1. Proactively hydrate active remote leases from Supabase Root DB
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        crate::modules::workspace_lease_manager::list_active_leases().await,
        "list_active_leases",
    );

    let mut effective_exclusions = excluded_account_ids.to_vec();
    if let Ok(reg) = instance::load_registry() {
        // Exclude current instance's own account so it cannot rotate to itself
        if let Some(cur_inst) = reg.instances.iter().find(|i| i.id == current_instance_id) {
            if let Some(ref acc_id) = cur_inst.bound_account_id {
                if !effective_exclusions.contains(acc_id) {
                    effective_exclusions.push(acc_id.clone());
                }
                if let Ok(acc) = account::load_account(acc_id) {
                    if !effective_exclusions.contains(&acc.email) {
                        effective_exclusions.push(acc.email.clone());
                    }
                }
            }
            if let Some(ref email) = cur_inst.bound_email {
                if !effective_exclusions.contains(email) {
                    effective_exclusions.push(email.clone());
                }
                if let Ok(Some(acc)) = account::get_account_by_email(email) {
                    if !effective_exclusions.contains(&acc.id) {
                        effective_exclusions.push(acc.id.clone());
                    }
                }
            }
            let is_default =
                cur_inst.is_default || cur_inst.id == "default" || cur_inst.id == "__default__";
            if is_default {
                if let Ok(Some(def_id)) = account::get_current_account_id() {
                    if !effective_exclusions.contains(&def_id) {
                        effective_exclusions.push(def_id.clone());
                    }
                    if let Ok(acc) = account::load_account(&def_id) {
                        if !effective_exclusions.contains(&acc.email) {
                            effective_exclusions.push(acc.email.clone());
                        }
                    }
                }
            }
        }
        for inst in &reg.instances {
            let is_running_sibling = inst.id != current_instance_id
                && crate::modules::instance::is_instance_running(
                    &inst.id,
                    &inst.data_dir,
                    inst.pid,
                );
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
                    if !effective_exclusions.contains(acc_id) {
                        effective_exclusions.push(acc_id.clone());
                    }
                    if let Ok(acc) = account::load_account(acc_id) {
                        if !effective_exclusions.contains(&acc.email) {
                            effective_exclusions.push(acc.email.clone());
                        }
                    }
                }
                if let Some(ref email) = inst.bound_email {
                    if !effective_exclusions.contains(email) {
                        effective_exclusions.push(email.clone());
                    }
                    if let Ok(Some(acc)) = account::get_account_by_email(email) {
                        if !effective_exclusions.contains(&acc.id) {
                            effective_exclusions.push(acc.id.clone());
                        }
                    }
                }
            }
        }
    }

    let candidates = select_candidate_profiles(
        current_instance_id,
        target_model,
        threshold,
        &effective_exclusions,
    )?;

    if candidates.is_empty() {
        logger::log_info(
            "[AutoSwitcher] No eligible candidate profiles found meeting quota threshold requirement.",
        );
        return Ok(None);
    }

    let mut best_fallback: Option<ProfileCandidate> = None;
    let mut highest_fallback_quota: f64 = 0.0;
    let mut best_emergency_fallback: Option<ProfileCandidate> = None;
    let mut highest_emergency_quota: f64 = 0.0;
    let (node_alias, _) = crate::modules::email_sender::get_local_node_identity();
    let m_name = crate::modules::email_watcher::detect_machine_name();
    let m_ip = crate::modules::email_watcher::detect_local_ip();
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));

    for candidate in candidates {
        let masked_email = crate::modules::logger::mask_email(&candidate.email);

        // Pre-switch live verification against running sibling instances on this host
        if let Ok(reg) = instance::load_registry() {
            let is_bound_to_running_sibling = is_account_bound_to_running_sibling(
                &reg,
                current_instance_id,
                &candidate.account_id,
                &candidate.email,
            );
            if is_bound_to_running_sibling {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Candidate '{}' is actively bound to running sibling instance on this host. Skipping...",
                    masked_email
                ));
                continue;
            }
        }

        logger::log_info(&format!(
            "[AutoSwitcher] Pre-switch live verification: checking candidate '{}' (cached quota: {:.1}%)...",
            masked_email, candidate.quota_percent
        ));

        let mut cand_acc = match account::load_account(&candidate.account_id) {
            Ok(acc) => acc,
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Could not load candidate account '{}': {}. Skipping...",
                    masked_email, e
                ));
                continue;
            }
        };

        // Strict eligibility check (disabled, validation-blocked, or rate-limited)
        let now_sec = chrono::Utc::now().timestamp();
        if cand_acc.disabled
            || cand_acc.proxy_disabled
            || is_account_validation_blocked(&cand_acc, now_sec)
        {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is disabled or validation-blocked. Skipping...",
                masked_email
            ));
            continue;
        }
        if is_account_rate_limited(&cand_acc, target_model, now_sec) {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is in active proxy rate-limit cooldown for model '{}'. Skipping...",
                masked_email, target_model
            ));
            crate::modules::audit::record_if_active(
                "system",
                "programmatic",
                "auto_switcher::cooldown_skip",
                "instances | auto_switcher | cooldown_guard",
                serde_json::json!({
                    "agm_version": pkg_ver,
                    "machine_name": m_name,
                    "node_alias": node_alias,
                    "local_ip": m_ip,
                    "account_id": cand_acc.id,
                    "account_email_masked": masked_email,
                    "cooldown_reason": "rate_limited",
                    "model": target_model,
                }),
                "ok",
                "",
            );
            continue;
        }

        // Distributed lease check in Supabase Root DB
        if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
            &cand_acc.id,
            &candidate.email,
        ) {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is currently leased by another active node. Skipping...",
                masked_email
            ));
            continue;
        }

        // Cross-VM email status broadcast check
        if crate::modules::email_inbound::fetch_recent_cross_vm_switched_accounts(3600)
            .iter()
            .any(|x| {
                x.eq_ignore_ascii_case(&cand_acc.id) || x.eq_ignore_ascii_case(&candidate.email)
            })
        {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is held by another VM node via inbound email broadcast. Skipping...",
                masked_email
            ));
            continue;
        }

        // Imp-06: Pre-Switch Live Refresh Timeout Guard (bounded 5000ms) with cached quota fallback
        let fetch_res = match tokio::time::timeout(
            std::time::Duration::from_millis(5000),
            account::fetch_quota_with_retry(&mut cand_acc),
        )
        .await
        {
            Ok(res) => res,
            Err(_) => {
                let cached_q = calculate_candidate_quota(&cand_acc, target_model).unwrap_or(0.0);
                logger::log_warn(&format!(
                    "[AutoSwitcher] Live quota refresh timed out after 5000ms for candidate '{}'. Using cached quota: {:.1}%",
                    masked_email, cached_q
                ));
                Ok(cand_acc.quota.clone().unwrap_or_default())
            }
        };
        let fresh_4h_quota = match fetch_res {
            Ok(fresh_q) => {
                cand_acc.quota = Some(fresh_q);
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(account::save_account(&cand_acc), "save_account");
                let q_val = calculate_candidate_quota(&cand_acc, target_model).unwrap_or(0.0);
                logger::log_info(&format!(
                    "[AutoSwitcher] Candidate '{}' refreshed from Google API: {:.1}% (candidate quota)",
                    masked_email, q_val
                ));
                q_val
            }
            Err(e) => {
                let cached_q = calculate_candidate_quota(&cand_acc, target_model).unwrap_or(0.0);
                logger::log_warn(&format!(
                    "[AutoSwitcher] Failed to refresh live quota from Google API for candidate '{}': {}. Using cached quota: {:.1}%",
                    masked_email, e, cached_q
                ));
                cached_q
            }
        };

        if fresh_4h_quota <= 0.0 {
            continue;
        }

        // Strict 100% requirement for 4-hour window:
        if fresh_4h_quota >= 100.0 {
            logger::log_info(&format!(
                "[AutoSwitcher] Candidate '{}' confirmed with 100.0% quota for 4h window. Selected for switch!",
                masked_email
            ));
            crate::modules::audit::record_if_active(
                "system",
                "programmatic",
                "auto_switcher::candidate_evaluation",
                &format!("instances | auto_switcher | candidate_evaluator | {current_instance_id}"),
                serde_json::json!({
                    "agm_version": pkg_ver,
                    "machine_name": m_name,
                    "node_alias": node_alias,
                    "local_ip": m_ip,
                    "instance_id": current_instance_id,
                    "target_model": target_model,
                    "threshold_percent": threshold,
                    "selected_candidate": candidate.account_id,
                    "selected_email_masked": masked_email,
                    "quota_4h": fresh_4h_quota,
                    "score": candidate.score,
                    "reason": "100% 4h window quota confirmed",
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                }),
                "ok",
                "",
            );
            return Ok(Some(ProfileCandidate {
                instance_id: candidate.instance_id,
                account_id: candidate.account_id,
                email: candidate.email,
                quota_percent: fresh_4h_quota,
                score: candidate.score,
            }));
        } else {
            logger::log_info(&format!(
                "[AutoSwitcher] Candidate '{}' live 4h window quota is {:.1}% (< 100.0%). Staging as fallback candidate...",
                masked_email, fresh_4h_quota
            ));
            if fresh_4h_quota > threshold && fresh_4h_quota > highest_fallback_quota {
                highest_fallback_quota = fresh_4h_quota;
                best_fallback = Some(ProfileCandidate {
                    instance_id: candidate.instance_id,
                    account_id: candidate.account_id,
                    email: candidate.email,
                    quota_percent: fresh_4h_quota,
                    score: candidate.score,
                });
            } else if fresh_4h_quota > 5.0 && fresh_4h_quota > highest_emergency_quota {
                highest_emergency_quota = fresh_4h_quota;
                best_emergency_fallback = Some(ProfileCandidate {
                    instance_id: candidate.instance_id,
                    account_id: candidate.account_id,
                    email: candidate.email,
                    quota_percent: fresh_4h_quota,
                    score: candidate.score,
                });
            }
        }
    }

    let effective_fallback = best_fallback.or(best_emergency_fallback);
    if let Some(fallback) = effective_fallback {
        let fallback_masked = crate::modules::logger::mask_email(&fallback.email);
        logger::log_info(&format!(
            "[AutoSwitcher] No 100.0% quota profile found; selecting best available candidate '{}' with {:.1}% quota.",
            fallback_masked, fallback.quota_percent
        ));
        crate::modules::audit::record_if_active(
            "system",
            "programmatic",
            "auto_switcher::candidate_evaluation",
            &format!("instances | auto_switcher | candidate_evaluator | {current_instance_id}"),
            serde_json::json!({
                "agm_version": pkg_ver,
                "machine_name": m_name,
                "node_alias": node_alias,
                "local_ip": m_ip,
                "instance_id": current_instance_id,
                "target_model": target_model,
                "threshold_percent": threshold,
                "selected_candidate": fallback.account_id,
                "selected_email_masked": fallback_masked,
                "quota_4h": fallback.quota_percent,
                "score": fallback.score,
                "reason": "fallback best available candidate",
                "timestamp": chrono::Utc::now().timestamp_millis(),
            }),
            "ok",
            "",
        );
        return Ok(Some(fallback));
    }

    crate::modules::audit::record_if_active(
        "system",
        "programmatic",
        "auto_switcher::candidate_evaluation",
        &format!("instances | auto_switcher | candidate_evaluator | {current_instance_id}"),
        serde_json::json!({
            "agm_version": pkg_ver,
            "machine_name": m_name,
            "node_alias": node_alias,
            "local_ip": m_ip,
            "instance_id": current_instance_id,
            "target_model": target_model,
            "threshold_percent": threshold,
            "selected_candidate": serde_json::Value::Null,
            "reason": "no usable quota profile found",
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
        "ok",
        "",
    );

    logger::log_warn(
        "[AutoSwitcher] All candidate profiles examined. Zero profiles verified with usable quota. Aborting rotation.",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TokenData;

    #[test]
    fn test_is_account_rate_limited_detection() {
        let now_sec = 1790090000;
        let token = TokenData::new(
            "a".to_string(),
            "r".to_string(),
            3600,
            None,
            None,
            None,
            false,
            None,
        );
        let mut acc = Account::new("acc-rl".to_string(), "rl@test.local".to_string(), token);
        acc.live_limited_models.insert(
            "gemini-2.5-pro".to_string(),
            crate::models::account::LiveLimitStatus {
                model: "gemini-2.5-pro".to_string(),
                status: 429,
                reason: "QUOTA_EXHAUSTED".to_string(),
                until: now_sec + 600,
                detected_at: now_sec,
                message: None,
            },
        );

        assert!(is_account_rate_limited(&acc, "gemini-2.5-pro", now_sec));
        assert!(!is_account_rate_limited(&acc, "claude-sonnet-4-6", now_sec));

        // Expired cooldown
        acc.live_limited_models
            .get_mut("gemini-2.5-pro")
            .unwrap()
            .until = now_sec - 10;
        assert!(!is_account_rate_limited(&acc, "gemini-2.5-pro", now_sec));
    }

    #[test]
    fn test_is_account_validation_blocked_detection() {
        let now_sec = 1790090000;
        let token = TokenData::new(
            "a".to_string(),
            "r".to_string(),
            3600,
            None,
            None,
            None,
            false,
            None,
        );
        let mut acc = Account::new("acc-vb".to_string(), "vb@test.local".to_string(), token);
        assert!(!is_account_validation_blocked(&acc, now_sec));

        acc.validation_blocked = true;
        acc.validation_blocked_until = Some(now_sec + 300);
        assert!(is_account_validation_blocked(&acc, now_sec));

        acc.validation_blocked_until = Some(now_sec - 10);
        assert!(!is_account_validation_blocked(&acc, now_sec));
    }

    #[test]
    fn test_is_account_bound_to_running_sibling_stopped_or_self() {
        use crate::models::{InstanceConfig, InstanceRegistry};
        let mut reg = InstanceRegistry::default();
        let inst1 = InstanceConfig {
            id: "inst-1".to_string(),
            name: "Instance 1".to_string(),
            data_dir: "/tmp/inst1".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: Some("acc-1".to_string()),
            bound_email: Some("acc1@test.local".to_string()),
            created_at: 100,
            last_used: 100,
            is_default: false,
            pid: None,
            seq_num: Some(1),
        };
        reg.instances.push(inst1);

        // When self is inst-1, it is not considered a sibling
        assert!(!is_account_bound_to_running_sibling(
            &reg,
            "inst-1",
            "acc-1",
            "acc1@test.local"
        ));

        // When target is inst-2, but inst-1 is NOT running (pid is 0/none and fake path), it returns false
        assert!(!is_account_bound_to_running_sibling(
            &reg,
            "inst-2",
            "acc-1",
            "acc1@test.local"
        ));
    }
}
