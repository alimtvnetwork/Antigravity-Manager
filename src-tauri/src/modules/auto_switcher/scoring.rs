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

/// Compute hours elapsed since the weekly quota cycle started for a given bucket.
/// Returns hours elapsed (0.0..=168.0), or `168.0` as fallback if reset_time is unparseable.
pub(crate) fn compute_weekly_hours_elapsed(
    bucket: &crate::models::quota::QuotaBucket,
    now_sec: i64,
) -> f64 {
    const TOTAL_WEEK_HOURS: f64 = 168.0;
    let reset_ts = match parse_reset_time_to_unix(&bucket.reset_time) {
        Some(ts) => ts,
        None => return TOTAL_WEEK_HOURS, // fallback: assume full week elapsed
    };
    let remaining_secs = reset_ts.saturating_sub(now_sec).max(0);
    let remaining_hours = remaining_secs as f64 / 3600.0;
    (TOTAL_WEEK_HOURS - remaining_hours)
        .max(0.0)
        .min(TOTAL_WEEK_HOURS)
}

/// Hours-elapsed-weighted weekly quota scoring algorithm:
/// - Any candidate with < 100% 4-hour quota (and period not finished) evaluates to `0.0`.
/// - Otherwise: `Score = (S_active * M_tier * (weekly_quota_pct × hours_elapsed)) / 16800.0`
///   where S_active = 1.0, M_tier is user-configurable via Settings → Algorithm
///   (defaults: Ultra 4.0, Pro 2.0, Free 1.0).
/// - Sorting direction: ASCENDING (lowest score = account just reset = selected first).
/// - Weekly quota < 8% treated as 0 (below viable threshold for Gemini).
/// - Claude/3p buckets are excluded (TODO: define Claude weekly scoring algorithm).
/// Compute the base weekly score ("score what we have"):
/// Formula: (tier_multiplier * weekly_effective_score) / 100.0
/// where weekly_effective_score = effective_weekly_pct * (168.0 - hours_remaining).
/// Accounts with weekly quota < 8% strictly score 0.0 (Gemini threshold).
/// If period is finished (reset time elapsed), weekly_effective_score = 168.0 * 100.0 (maximum fresh credits).
/// TODO(claude): Claude/3p weekly scoring algorithm undefined — 3p buckets excluded with ambiguity marker.
pub fn calculate_weekly_base_score(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
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
    const WEEKLY_ZERO_THRESHOLD: f64 = 8.0;

    let mut weekly_effective_score = 0.0_f64;

    if let Some(quota_data) = acc.quota.as_ref() {
        let mut gemini_weekly_scores: Vec<f64> = Vec::new();

        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    let win = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    // Gemini-only: skip 3p/claude buckets (Claude TODO)
                    let is_gemini_weekly = (win.contains("week") || bid.contains("week"))
                        && !bid.contains("3p")
                        && !bid.contains("claude");
                    if is_gemini_weekly && (0.0..=1.0).contains(&b.remaining_fraction) {
                        let pct = (b.remaining_fraction * 100.0).round();
                        let eff_pct = if pct < WEEKLY_ZERO_THRESHOLD {
                            0.0
                        } else {
                            pct
                        };
                        // hours_elapsed = 168 − hours_remaining (compute_weekly_hours_elapsed returns this distance)
                        let hours_elapsed = compute_weekly_hours_elapsed(b, now_sec);
                        gemini_weekly_scores.push(eff_pct * hours_elapsed);
                    }
                }
            }
        }

        if !gemini_weekly_scores.is_empty() {
            // Use minimum bottleneck across multiple Gemini weekly buckets
            weekly_effective_score = gemini_weekly_scores
                .into_iter()
                .fold(f64::INFINITY, f64::min);
        } else {
            // Fallback: legacy model percentage × half-week elapsed (assume mid-cycle)
            let valid_models: Vec<_> = quota_data
                .models
                .iter()
                .filter(|m| {
                    let n = m.name.to_lowercase();
                    !n.contains("claude") && !n.contains("3p")
                })
                .collect();
            if !valid_models.is_empty() {
                let sum: i32 = valid_models.iter().map(|m| m.percentage).sum();
                let avg_pct = (sum as f64 / valid_models.len() as f64).round();
                let eff_pct = if avg_pct < WEEKLY_ZERO_THRESHOLD {
                    0.0
                } else {
                    avg_pct
                };
                weekly_effective_score = eff_pct * (TOTAL_WEEK_HOURS / 2.0);
            }
        }
    }

    // Reset boundary: if period has finished, credits refresh to full 100% capacity
    if is_period_finished {
        weekly_effective_score = TOTAL_WEEK_HOURS * 100.0;
    }

    (tier_multiplier * weekly_effective_score) / 100.0
}

/// Primary candidate scoring ("usually"):
/// - If 4h quota is less than 100% (and period not finished): strictly returns 0.0.
/// - Otherwise (4h quota == 100% or period finished): returns base weekly score floor.
/// Sort direction: DESCENDING (highest score selected first).
pub fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    let period_stat = evaluate_account_period_status(acc, target_model, 20.0, now_sec);
    let is_period_finished = period_stat
        .as_ref()
        .map(|s| s.is_period_finished)
        .unwrap_or(false);
    let q_4h = period_stat
        .as_ref()
        .map(|s| s.quota_percent)
        .or_else(|| calculate_4h_window_quota(acc, target_model))
        .unwrap_or(0.0);

    // Primary 100% 4-Hour Quota Gate:
    // If 4h time is less than 100%, usually it should be 0.
    if q_4h < 100.0 && !is_period_finished {
        return 0.0;
    }

    let base_score = calculate_weekly_base_score(acc, target_model, now_sec);
    base_score.floor()
}

/// Fallback candidate scoring ("unless there is another option / if nothing found"):
/// - When no account has 100% 4h quota, evaluate partial 4h quota:
///   Score = 4h percentage % × (score what we have) = ((q_4h / 100.0) * base_weekly_score).floor().
/// - Accounts with weekly quota < 8% strictly evaluate to 0.0.
pub fn score_candidate_account_fallback(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    let period_stat = evaluate_account_period_status(acc, target_model, 20.0, now_sec);
    let is_period_finished = period_stat
        .as_ref()
        .map(|s| s.is_period_finished)
        .unwrap_or(false);
    let q_4h = if is_period_finished {
        100.0
    } else {
        period_stat
            .as_ref()
            .map(|s| s.quota_percent)
            .or_else(|| calculate_4h_window_quota(acc, target_model))
            .unwrap_or(0.0)
    };

    if q_4h <= 0.0 {
        return 0.0;
    }

    let base_score = calculate_weekly_base_score(acc, target_model, now_sec);
    ((q_4h / 100.0) * base_score).floor()
}

/// Specifically evaluate the 4-hour / 5-hour immediate rolling window quota (0-100%)
pub fn calculate_4h_window_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    if quota_data.is_forbidden {
        return Some(0.0);
    }
    let target = target_model.to_lowercase();
    let is_flash_target = target.contains("flash");

    let mut min_pct: Option<f64> = None;

    // 1. Check quota_groups for 4h / 5h short-window buckets
    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
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

    // 2. Minimum across models matching target (and flash models when target is flash)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && name_lower.contains("flash");
        if is_direct || is_flash {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    // 3. Minimum across any actively consumed models (percentage < 100)
    for m in &quota_data.models {
        if m.percentage < 100 {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    if let Some(pct) = min_pct {
        return Some(pct);
    }

    // Fallback: general calculate_account_quota
    calculate_account_quota(account, target_model)
}

/// Specifically evaluate the 7-day weekly total allowance window quota (0-100%)
pub fn calculate_weekly_window_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    let mut weekly_values: Vec<f64> = Vec::new();

    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
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

    // Fallback across models
    let valid_models: Vec<_> = quota_data.models.iter().collect();

    if !valid_models.is_empty() {
        let sum: i32 = valid_models.iter().map(|m| m.percentage).sum();
        return Some((sum as f64 / valid_models.len() as f64).round());
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
