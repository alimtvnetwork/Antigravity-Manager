use crate::models::Account;
use crate::modules::{account, config, instance, logger};

use super::*;

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

    let mut effective_exclusions = excluded_account_ids.to_vec();
    for in_use_id in get_active_in_use_account_ids() {
        if !effective_exclusions.contains(&in_use_id) {
            effective_exclusions.push(in_use_id);
        }
    }
    for cross_vm_acc in crate::modules::email_inbound::fetch_recent_cross_vm_switched_accounts(3600)
    {
        if !effective_exclusions.contains(&cross_vm_acc) {
            effective_exclusions.push(cross_vm_acc);
        }
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
            if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
                continue;
            }
            if acc.disabled || acc.proxy_disabled || acc.validation_blocked {
                continue;
            }
            // Strictly skip accounts leased by another active machine/node
            if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
                &acc.id, &acc.email,
            ) {
                continue;
            }

            let is_period_finished = acc
                .quota
                .as_ref()
                .and_then(|q| q.models.first())
                .map(|m| {
                    parse_reset_time_to_unix(&m.reset_time)
                        .map(|ts| ts <= now_sec)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            let quota = if is_period_finished {
                100.0
            } else {
                calculate_candidate_quota(&acc, target_model).unwrap_or(0.0)
            };

            // Strict 100% 4-Hour Quota Gate:
            // Candidate accounts must have 100% quota, OR their reset period has elapsed (eligible for live refresh).
            if quota >= 100.0 || is_period_finished {
                seen_account_ids.insert(acc.id.clone());
                let score = score_candidate_account(&acc, target_model, now_sec);
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

    // 2. Check unbound accounts in pool
    let all_accounts = account::list_accounts().unwrap_or_default();
    for acc in &all_accounts {
        if seen_account_ids.contains(&acc.id) {
            continue;
        }
        if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
            continue;
        }
        if acc.disabled || acc.proxy_disabled || acc.validation_blocked {
            continue;
        }
        // Strictly skip accounts leased by another active machine/node
        if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
            &acc.id, &acc.email,
        ) {
            continue;
        }

        // Check if bound to an instance running on this machine
        let bound_inst = registry
            .instances
            .iter()
            .find(|i| i.bound_account_id.as_deref() == Some(&acc.id));
        if let Some(inst) = bound_inst {
            if inst.id != current_instance_id
                && crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)
            {
                continue;
            }
        }

        let is_period_finished = acc
            .quota
            .as_ref()
            .and_then(|q| q.models.first())
            .map(|m| {
                parse_reset_time_to_unix(&m.reset_time)
                    .map(|ts| ts <= now_sec)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let quota = if is_period_finished {
            100.0
        } else {
            calculate_candidate_quota(acc, target_model).unwrap_or(0.0)
        };

        // Strict 100% 4-Hour Quota Gate
        if quota >= 100.0 || is_period_finished {
            seen_account_ids.insert(acc.id.clone());
            let score = score_candidate_account(acc, target_model, now_sec);
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

    // 3. Fallback: If no candidate met the 100% or period-finished criteria, fall back to best available accounts above threshold
    if available_pool.is_empty() && cooldown_pool.is_empty() {
        for acc in &all_accounts {
            if seen_account_ids.contains(&acc.id) {
                continue;
            }
            if is_account_excluded(&effective_exclusions, &acc.id, &acc.email) {
                continue;
            }
            if acc.disabled || acc.proxy_disabled || acc.validation_blocked {
                continue;
            }
            // Strictly skip accounts leased by another active machine/node
            if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
                &acc.id, &acc.email,
            ) {
                continue;
            }

            let bound_inst = registry
                .instances
                .iter()
                .find(|i| i.bound_account_id.as_deref() == Some(&acc.id));
            if let Some(inst) = bound_inst {
                if inst.id != current_instance_id
                    && crate::modules::instance::is_instance_running(
                        &inst.id,
                        &inst.data_dir,
                        inst.pid,
                    )
                {
                    continue;
                }
            }

            let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);
            if quota > threshold {
                seen_account_ids.insert(acc.id.clone());
                let score = score_candidate_account_fallback(acc, target_model, now_sec);
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
        // Sort by oldest last_used (ascending last_used timestamp: smallest first)
        cooldown_pool.sort_by(
            |(a_cand, a_last), (b_cand, b_last)| match a_last.cmp(b_last) {
                std::cmp::Ordering::Equal => match b_cand.score.partial_cmp(&a_cand.score) {
                    Some(std::cmp::Ordering::Equal) | None => a_cand
                        .email
                        .to_lowercase()
                        .cmp(&b_cand.email.to_lowercase()),
                    Some(ord) => ord,
                },
                ord => ord,
            },
        );
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

    let candidates = select_candidate_profiles(
        current_instance_id,
        target_model,
        threshold,
        excluded_account_ids,
    )?;

    if candidates.is_empty() {
        logger::log_info(
            "[AutoSwitcher] No initial candidate profiles found meeting 100% quota requirement.",
        );
        return Ok(None);
    }

    let mut best_fallback: Option<ProfileCandidate> = None;
    let mut highest_fallback_quota: f64 = 0.0;

    for candidate in candidates {
        logger::log_info(&format!(
            "[AutoSwitcher] Pre-switch live verification: checking candidate '{}' (cached quota: {:.1}%)...",
            candidate.email, candidate.quota_percent
        ));

        let mut cand_acc = match account::load_account(&candidate.account_id) {
            Ok(acc) => acc,
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Could not load candidate account '{}': {}. Skipping...",
                    candidate.email, e
                ));
                continue;
            }
        };

        // Strict disabled immunity check
        if cand_acc.disabled || cand_acc.proxy_disabled || cand_acc.validation_blocked {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is disabled (proxy_disabled: {}, disabled: {}). Skipping...",
                candidate.email, cand_acc.proxy_disabled, cand_acc.disabled
            ));
            continue;
        }

        // Distributed lease check in Supabase Root DB
        if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
            &cand_acc.id,
            &candidate.email,
        ) {
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' is currently leased by another active node. Skipping...",
                candidate.email
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
                candidate.email
            ));
            continue;
        }

        // Live API Quota Refresh directly from Google API
        let fetch_res = account::fetch_quota_with_retry(&mut cand_acc).await;
        let fresh_4h_quota = match fetch_res {
            Ok(fresh_q) => {
                cand_acc.quota = Some(fresh_q);
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(account::save_account(&cand_acc), "save_account");
                let q_val = calculate_candidate_quota(&cand_acc, target_model).unwrap_or(0.0);
                logger::log_info(&format!(
                    "[AutoSwitcher] Candidate '{}' refreshed from Google API: {:.1}% (candidate quota)",
                    candidate.email, q_val
                ));
                q_val
            }
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Failed to refresh live quota from Google API for candidate '{}': {}. Skipping...",
                    candidate.email, e
                ));
                continue;
            }
        };

        // Strict 100% requirement for 4-hour window:
        if fresh_4h_quota >= 100.0 {
            logger::log_info(&format!(
                "[AutoSwitcher] Candidate '{}' confirmed with 100.0% quota for 4h window. Selected for switch!",
                candidate.email
            ));
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
                candidate.email, fresh_4h_quota
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
            }
        }
    }

    if let Some(fallback) = best_fallback {
        logger::log_info(&format!(
            "[AutoSwitcher] No 100.0% quota profile found; selecting best available candidate '{}' with {:.1}% quota.",
            fallback.email, fallback.quota_percent
        ));
        return Ok(Some(fallback));
    }

    logger::log_warn(
        "[AutoSwitcher] All candidate profiles examined. Zero profiles verified with usable quota. Aborting rotation.",
    );
    Ok(None)
}
