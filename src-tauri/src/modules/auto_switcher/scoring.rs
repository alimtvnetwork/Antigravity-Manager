use crate::models::Account;
use crate::modules::{account, config, instance, logger};

use super::*;

/// Candidate profile target scored for auto-switch
pub struct ProfileCandidate {
    pub instance_id: String,
    pub account_id: String,
    pub email: String,
    pub quota_percent: f64,
    pub score: f64,
}

/// Helper to check if a model name matches target_model or family wildcards
pub fn is_model_matching_target(model_name: &str, target_model: &str) -> bool {
    let m = model_name.trim().to_lowercase();
    let t = target_model.trim().to_lowercase();
    if t.is_empty() || t == "*" {
        return true;
    }
    if m == t || m.contains(&t) || t.contains(&m) {
        return true;
    }
    let is_target_flash = t.contains("flash");
    let is_target_pro = t.contains("pro");
    let is_target_claude = t.contains("claude") || t.contains("sonnet");
    let is_target_gpt = t.contains("gpt") || t.contains("o1") || t.contains("o3");
    if is_target_flash && m.contains("flash") {
        return true;
    }
    if is_target_pro && m.contains("pro") && !m.contains("flash") {
        return true;
    }
    if is_target_claude && (m.contains("claude") || m.contains("sonnet")) {
        return true;
    }
    if is_target_gpt && (m.contains("gpt") || m.contains("o1") || m.contains("o3")) {
        return true;
    }
    false
}

/// Compute remaining headroom hours until the weekly quota cycle resets for a given bucket.
/// Returns remaining hours (0.0..=168.0), or `168.0` as fallback if reset_time is unparseable or finished.
pub(crate) fn compute_weekly_remaining_headroom_hours(
    bucket: &crate::models::quota::QuotaBucket,
    now_sec: i64,
) -> f64 {
    const TOTAL_WEEK_HOURS: f64 = 168.0;
    let reset_ts = match parse_reset_time_to_unix(&bucket.reset_time) {
        Some(ts) => ts,
        None => return TOTAL_WEEK_HOURS, // fallback: assume full cycle remaining
    };
    if reset_ts <= now_sec {
        return TOTAL_WEEK_HOURS; // period finished, fresh cycle
    }
    let remaining_secs = (reset_ts - now_sec).max(0);
    let remaining_hours = remaining_secs as f64 / 3600.0;
    remaining_hours.max(0.0).min(TOTAL_WEEK_HOURS)
}

/// Compute hours elapsed since the weekly quota cycle started for a given bucket.
/// Returns hours elapsed (0.0..=168.0), or `168.0` as fallback if reset_time is unparseable.
pub(crate) fn compute_weekly_hours_elapsed(
    bucket: &crate::models::quota::QuotaBucket,
    now_sec: i64,
) -> f64 {
    const TOTAL_WEEK_HOURS: f64 = 168.0;
    let remaining_hours = compute_weekly_remaining_headroom_hours(bucket, now_sec);
    (TOTAL_WEEK_HOURS - remaining_hours)
        .max(0.0)
        .min(TOTAL_WEEK_HOURS)
}

/// Helper to check if an account is active, enabled, and permitted for proxy/switch usage.
pub fn is_account_active_and_enabled(acc: &Account) -> bool {
    !acc.disabled && !acc.proxy_disabled
}

/// Helper to check if candidate is eligible for switch scoring.
pub fn is_candidate_eligible_for_switch(acc: &Account) -> bool {
    is_account_active_and_enabled(acc) && acc.quota.is_some()
}

/// Helper to filter native Gemini models from third-party or Claude models.
pub fn is_gemini_native_model(model_name: &str) -> bool {
    let n = model_name.to_lowercase();
    !n.contains("claude") && !n.contains("3p")
}

/// Helper to check if a quota bucket belongs to Gemini weekly cycle.
pub fn is_gemini_weekly_bucket(window: &str, bucket_id: &str) -> bool {
    let win = window.to_lowercase();
    let bid = bucket_id.to_lowercase();
    let is_weekly = win.contains("week") || bid.contains("week");
    let is_gemini = !bid.contains("3p") && !bid.contains("claude");
    is_weekly && is_gemini
}

/// Headroom-weighted weekly quota scoring algorithm:
/// - Any candidate with < 100% 4-hour quota (and period not finished) evaluates to `0.0`.
/// - Fresh accounts with 168h headroom receive maximum scaling (Headroom Multiplier = 2.0).
/// - Accounts nearing reset boundary receive minimal headroom multiplier (Headroom Multiplier = 1.0).
/// - Formula: `Score = (tier_multiplier * weekly_effective_score) / 100.0`
///   where weekly_effective_score = effective_weekly_pct * remaining_hours (or 168.0 * 100.0 on reset).
/// - Sorting direction: DESCENDING (highest score = freshly reset 100% account = selected first).
/// - Weekly quota < floor threshold (default 15%, user configurable) treated as 0 (below viable threshold for Gemini).
/// - Claude/3p buckets are excluded.
pub fn calculate_weekly_base_score(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    if !is_account_active_and_enabled(acc) {
        return 0.0;
    }

    let period_stat = evaluate_account_period_status(acc, target_model, 20.0, now_sec);
    let is_period_finished = period_stat
        .as_ref()
        .map(|s| s.is_period_finished)
        .unwrap_or(false);

    // 1. Subscription tier multiplier (user-configurable via Settings → Algorithm;
    //    defaults: Ultra 4.0, Pro 2.0, Free 1.0)
    let tier = acc
        .quota
        .as_ref()
        .and_then(|q| q.subscription_tier.as_ref())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let switcher_cfg = config::load_app_config()
        .map(|c| c.auto_profile_switcher)
        .unwrap_or_default();
    let tier_multiplier = if tier.contains("ultra") {
        switcher_cfg.ultra_tier_multiplier
    } else if tier.contains("pro") {
        switcher_cfg.pro_tier_multiplier
    } else {
        switcher_cfg.free_tier_multiplier
    };

    const TOTAL_WEEK_HOURS: f64 = 168.0;
    let weekly_zero_threshold: f64 = switcher_cfg.min_weekly_quota_percent.max(0.0);

    let mut weekly_effective_score = 0.0_f64;

    let q_4h_check = calculate_4h_window_quota(acc, target_model).unwrap_or(0.0);
    if is_period_finished && q_4h_check >= 100.0 {
        // Reset boundary: credits refresh to full 100% capacity with maximum headroom
        weekly_effective_score = TOTAL_WEEK_HOURS * 100.0;
    } else if let Some(quota_data) = acc.quota.as_ref() {
        let mut gemini_weekly_scores: Vec<f64> = Vec::new();

        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    if !is_gemini_weekly_bucket(&b.window, &b.bucket_id) {
                        continue;
                    }
                    if !(0.0..=1.0).contains(&b.remaining_fraction) {
                        continue;
                    }
                    let pct = (b.remaining_fraction * 100.0).round();
                    let eff_pct = if pct < weekly_zero_threshold {
                        0.0
                    } else {
                        pct
                    };
                    // Modernized: remaining cycle headroom gives highest score to freshly reset accounts
                    let remaining_hours = compute_weekly_remaining_headroom_hours(b, now_sec);
                    gemini_weekly_scores.push(eff_pct * remaining_hours);
                }
            }
        }

        if !gemini_weekly_scores.is_empty() {
            // Use minimum bottleneck across multiple Gemini weekly buckets
            weekly_effective_score = gemini_weekly_scores
                .into_iter()
                .fold(f64::INFINITY, f64::min);
        } else {
            // Fallback: evaluate target model or models
            let target_pct = quota_data
                .models
                .iter()
                .find(|m| is_model_matching_target(&m.name, target_model))
                .map(|m| m.percentage as f64);
            let eff_pct = match target_pct {
                Some(pct) if pct < weekly_zero_threshold => 0.0,
                Some(pct) => pct,
                None => {
                    let valid_models: Vec<_> = quota_data
                        .models
                        .iter()
                        .filter(|m| is_gemini_native_model(&m.name))
                        .collect();
                    if !valid_models.is_empty() {
                        let sum: i32 = valid_models.iter().map(|m| m.percentage).sum();
                        let avg_pct = (sum as f64 / valid_models.len() as f64).round();
                        if avg_pct < weekly_zero_threshold {
                            0.0
                        } else {
                            avg_pct
                        }
                    } else {
                        100.0
                    }
                }
            };
            weekly_effective_score = eff_pct * TOTAL_WEEK_HOURS;
        }
    }

    (tier_multiplier * weekly_effective_score) / 100.0
}

/// Primary candidate scoring ("usually"):
/// - If 4h quota is less than 100% (even reduced by 1% or 2%): strictly returns 0.0.
/// - Only genuine 100% 4h quota accounts receive base weekly score.
/// Sort direction: DESCENDING (highest score selected first).
pub fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    if !is_account_active_and_enabled(acc) {
        return 0.0;
    }

    let q_4h = calculate_4h_window_quota(acc, target_model).unwrap_or(0.0);

    // Primary 100% 4-Hour Quota Gate:
    // Any candidate with < 100% 4-hour quota (even reduced by 1% or 2%) strictly evaluates to 0.0.
    if q_4h < 100.0 {
        return 0.0;
    }

    calculate_weekly_base_score(acc, target_model, now_sec)
}

/// Fallback candidate scoring ("unless there is another option / if nothing found"):
/// - When no account has 100% 4h quota, evaluate partial 4h quota:
///   Score = 4h percentage % × (score what we have) = ((q_4h / 100.0) * base_weekly_score).
/// - Accounts with weekly quota < 8% strictly evaluate to 0.0.
pub fn score_candidate_account_fallback(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    if !is_account_active_and_enabled(acc) {
        return 0.0;
    }

    let q_4h = calculate_4h_window_quota(acc, target_model).unwrap_or(0.0);
    if q_4h <= 0.0 {
        return 0.0;
    }

    let base_score = calculate_weekly_base_score(acc, target_model, now_sec);
    calculate_proportional_fallback_score(q_4h, base_score)
}

/// Helper computing proportional score: (q_4h / 100.0) * weekly_score.
/// Returns 0.0 if q_4h <= 0.0 or weekly_score <= 0.0.
pub fn calculate_proportional_fallback_score(q_4h: f64, weekly_score: f64) -> f64 {
    if q_4h <= 0.0 || weekly_score <= 0.0 {
        return 0.0;
    }
    ((q_4h.clamp(0.0, 100.0)) / 100.0) * weekly_score
}

/// Specifically evaluate the 4-hour / 5-hour immediate rolling window quota (0-100%)
pub fn calculate_4h_window_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    if quota_data.is_forbidden {
        return Some(0.0);
    }
    let target = target_model.to_lowercase();
    let is_claude_or_gpt_target =
        target.contains("claude") || target.contains("gpt") || target.contains("sonnet");

    let mut min_pct: Option<f64> = None;

    // 1. Check quota_groups for 4h / 5h short-window buckets matching target group
    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
            let gname = g.display_name.to_lowercase();
            let is_group_claude_or_gpt =
                gname.contains("claude") || gname.contains("gpt") || gname.contains("3p");
            let is_group_gemini = gname.contains("gemini") || (!is_group_claude_or_gpt);

            let group_matches = if is_claude_or_gpt_target {
                is_group_claude_or_gpt
            } else {
                is_group_gemini
            };
            if !group_matches {
                continue; // Isolate target group
            }

            for b in &g.buckets {
                let win = b.window.to_lowercase();
                let bid = b.bucket_id.to_lowercase();
                let is_short_window = win.contains("5h")
                    || win.contains("4h")
                    || bid.contains("5h")
                    || bid.contains("4h")
                    || (!win.contains("week")
                        && !win.contains("7d")
                        && !bid.contains("week")
                        && !bid.contains("7d"));
                if is_short_window && (0.0..=1.0).contains(&b.remaining_fraction) {
                    let pct = (b.remaining_fraction * 100.0).round();
                    min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
                }
            }
        }
    }

    // 2. Minimum across models matching target (strict target model isolation)
    for m in &quota_data.models {
        if is_model_matching_target(&m.name, target_model) {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    if let Some(pct) = min_pct {
        return Some(pct);
    }

    // If account has valid quota and target model has no consumed usage records, it is 100% available
    if !quota_data.models.is_empty() {
        return Some(100.0);
    }

    // Fallback: general calculate_account_quota
    calculate_account_quota(account, target_model)
}

/// Specifically evaluate the 7-day weekly total allowance window quota (0-100%)
pub fn calculate_weekly_window_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    let target = target_model.to_lowercase();
    let is_claude_or_gpt_target =
        target.contains("claude") || target.contains("gpt") || target.contains("sonnet");
    let mut weekly_values: Vec<f64> = Vec::new();

    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
            let gname = g.display_name.to_lowercase();
            let is_group_claude_or_gpt =
                gname.contains("claude") || gname.contains("gpt") || gname.contains("3p");
            let is_group_gemini = gname.contains("gemini") || (!is_group_claude_or_gpt);
            let group_matches = if is_claude_or_gpt_target {
                is_group_claude_or_gpt
            } else {
                is_group_gemini
            };
            if !group_matches {
                continue;
            }

            for b in &g.buckets {
                let win = b.window.to_lowercase();
                let bid = b.bucket_id.to_lowercase();
                let is_weekly = win.contains("week")
                    || bid.contains("week")
                    || win.contains("7d")
                    || bid.contains("7d");
                if is_weekly && (0.0..=1.0).contains(&b.remaining_fraction) {
                    weekly_values.push((b.remaining_fraction * 100.0).round());
                }
            }
        }
    }

    if !weekly_values.is_empty() {
        return Some(weekly_values.into_iter().fold(100.0, f64::min));
    }

    // Model-level match
    for m in &quota_data.models {
        if is_model_matching_target(&m.name, target_model) {
            return Some(m.percentage as f64);
        }
    }

    if !quota_data.models.is_empty() {
        return Some(100.0);
    }

    calculate_account_quota(account, target_model)
}

/// Extract dual-window quotas: (4-hour immediate rolling percentage, weekly percentage)
pub fn extract_dual_window_quotas(
    account: &Account,
    target_model: &str,
) -> (Option<f64>, Option<f64>) {
    let q_4h = calculate_4h_window_quota(account, target_model);
    let q_weekly = calculate_weekly_window_quota(account, target_model);
    (q_4h, q_weekly)
}

/// Helper to robustly check if an account ID or email matches an exclusion list case-insensitively
pub fn is_account_excluded(exclusions: &[String], id: &str, email: &str) -> bool {
    let clean_id = id.trim();
    let clean_email = email.trim();
    exclusions.iter().any(|ex| {
        let clean_ex = ex.trim();
        (!clean_id.is_empty() && clean_ex.eq_ignore_ascii_case(clean_id))
            || (!clean_email.is_empty() && clean_ex.eq_ignore_ascii_case(clean_email))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_formula_rewards_headroom() {
        let now_sec = 1790090000;
        let future_time_fresh = "2026-09-29T14:30:00Z";
        let future_time_low = "2026-09-22T17:30:00Z";

        let mut acc_fresh = make_test_account(
            "acc-fresh",
            "fresh@test.local",
            "gemini-pro",
            100,
            future_time_fresh,
        );
        if let Some(ref mut q) = acc_fresh.quota {
            q.quota_groups = Some(vec![crate::models::quota::QuotaGroup {
                display_name: "Gemini Models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "gemini-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 1.0,
                    reset_time: future_time_fresh.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            }]);
        }

        let mut acc_low = make_test_account(
            "acc-low",
            "low@test.local",
            "gemini-pro",
            100,
            future_time_low,
        );
        if let Some(ref mut q) = acc_low.quota {
            q.quota_groups = Some(vec![crate::models::quota::QuotaGroup {
                display_name: "Gemini Models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "gemini-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 1.0,
                    reset_time: future_time_low.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            }]);
        }

        let score_fresh = score_candidate_account(&acc_fresh, "gemini-pro", now_sec);
        let score_low = score_candidate_account(&acc_low, "gemini-pro", now_sec);

        assert!(
            score_fresh > score_low,
            "Freshly reset accounts must rank higher than low headroom accounts: fresh={} low={}",
            score_fresh,
            score_low
        );
    }

    #[test]
    fn test_calculate_4h_window_quota_model_isolation() {
        let mut acc = make_test_account(
            "acc-iso",
            "iso@test.local",
            "gemini-1.5-flash",
            0,
            "2026-09-30T14:30:00Z",
        );
        if let Some(ref mut q) = acc.quota {
            q.models = vec![crate::models::quota::ModelQuota {
                name: "gemini-1.5-flash".to_string(),
                percentage: 0,
                reset_time: "2026-09-30T14:30:00Z".to_string(),
                display_name: None,
                supports_images: None,
                supports_thinking: None,
                thinking_budget: None,
                recommended: None,
                max_tokens: None,
                max_output_tokens: None,
                supported_mime_types: None,
            }];
        }

        let q_4h_pro = calculate_4h_window_quota(&acc, "gemini-2.5-pro");
        assert_eq!(
            q_4h_pro,
            Some(100.0),
            "Target gemini-2.5-pro must be isolated from exhausted gemini-1.5-flash"
        );
    }

    #[test]
    fn test_score_candidate_account_fallback_sub_100() {
        let now_sec = 1790090000;
        let future_time = "2026-09-30T14:30:00Z";
        let mut acc = make_test_account(
            "acc-sub100",
            "sub100@test.local",
            "gemini-pro",
            95,
            future_time,
        );
        if let Some(ref mut q) = acc.quota {
            q.quota_groups = Some(vec![crate::models::quota::QuotaGroup {
                display_name: "Gemini Models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "gemini-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 0.95,
                    reset_time: future_time.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            }]);
            q.models = vec![crate::models::quota::ModelQuota {
                name: "gemini-pro".to_string(),
                percentage: 95,
                reset_time: future_time.to_string(),
                display_name: None,
                supports_images: None,
                supports_thinking: None,
                thinking_budget: None,
                recommended: None,
                max_tokens: None,
                max_output_tokens: None,
                supported_mime_types: None,
            }];
        }

        let primary_score = score_candidate_account(&acc, "gemini-pro", now_sec);
        assert_eq!(
            primary_score, 0.0,
            "Primary score for <100% quota account must be 0.0"
        );

        let fallback_score = score_candidate_account_fallback(&acc, "gemini-pro", now_sec);
        assert!(
            fallback_score > 0.0,
            "Fallback score for 95% quota account must be > 0.0, got {}",
            fallback_score
        );
    }

    #[test]
    fn test_score_candidate_account_disabled_is_zero() {
        let now_sec = 1790090000;
        let future_time = "2026-09-30T14:30:00Z";
        let mut acc = make_test_account(
            "acc-disabled",
            "disabled@test.local",
            "gemini-pro",
            100,
            future_time,
        );
        acc.disabled = true;

        assert_eq!(
            calculate_weekly_base_score(&acc, "gemini-pro", now_sec),
            0.0
        );
        assert_eq!(score_candidate_account(&acc, "gemini-pro", now_sec), 0.0);
        assert_eq!(
            score_candidate_account_fallback(&acc, "gemini-pro", now_sec),
            0.0
        );

        acc.disabled = false;
        acc.proxy_disabled = true;
        assert_eq!(
            calculate_weekly_base_score(&acc, "gemini-pro", now_sec),
            0.0
        );
        assert_eq!(score_candidate_account(&acc, "gemini-pro", now_sec), 0.0);
        assert_eq!(
            score_candidate_account_fallback(&acc, "gemini-pro", now_sec),
            0.0
        );
    }
}
