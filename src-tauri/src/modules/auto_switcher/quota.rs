use crate::models::Account;
use crate::modules::{account, config, instance, logger};

use super::*;

/// Calculate quota percentage for a specific target model in an account
pub fn calculate_account_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    let target = target_model.to_lowercase();
    let is_flash_target = target.contains("flash");

    let mut min_pct: Option<f64> = None;

    // 1. Minimum across models matching target (and flash models when target is flash)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && name_lower.contains("flash");
        if is_direct || is_flash {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    // 2. Minimum across any actively consumed models (percentage < 100)
    for m in &quota_data.models {
        if m.percentage < 100 {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    // 3. Minimum across 4h / 5h short-window quota_groups buckets (immediate rolling quota)
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

    if let Some(pct) = min_pct {
        return Some(pct);
    }

    // Hierarchical match for Gemini 3.8 Flash High / Flash targets
    if is_flash_target {
        if let Some((_, pct, _)) = evaluate_hierarchical_quota(account) {
            return Some(pct);
        }
    }

    // Fallback: average percentage across all models
    if !quota_data.models.is_empty() {
        let total: i32 = quota_data.models.iter().map(|m| m.percentage).sum();
        let avg = total as f64 / quota_data.models.len() as f64;
        return Some(avg);
    }

    None
}

/// Hierarchical model quota evaluation:
/// 1. Primary: Gemini Flash
/// 2. Fallback: Claude Sonnet 4.6 / Claude Sonnet
pub fn evaluate_hierarchical_quota(account: &Account) -> Option<(String, f64, bool)> {
    let quota_data = account.quota.as_ref()?;

    // 1. Check Gemini Flash (Primary)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_gemini_flash = (name_lower.contains("3.8") && name_lower.contains("flash"))
            || (name_lower.contains("gemini") && name_lower.contains("flash"));
        if is_gemini_flash {
            let pct = m.percentage as f64;
            let is_primary = true;
            return Some((m.name.clone(), pct, is_primary));
        }
    }

    // 2. Check Claude Sonnet 4.6 (Fallback)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_sonnet = name_lower.contains("sonnet") || name_lower.contains("claude");
        if is_sonnet {
            let pct = m.percentage as f64;
            let is_primary = false;
            return Some((m.name.clone(), pct, is_primary));
        }
    }

    // 3. Any other Gemini model
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        if name_lower.contains("gemini") {
            let pct = m.percentage as f64;
            let is_primary = false;
            return Some((m.name.clone(), pct, is_primary));
        }
    }

    None
}

/// Specifically evaluate quota for a candidate account against target model:
/// Evaluates candidate's quota for the target model or hierarchical match (0-100%).
/// Unlike active account monitoring, this does NOT penalize candidate for unrelated exhausted models.
pub fn calculate_candidate_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    let target = target_model.to_lowercase();
    let is_flash_target = target.contains("flash");

    // 1. Direct target model match (and flash models when target is flash)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && name_lower.contains("flash");
        if is_direct || is_flash {
            return Some(m.percentage as f64);
        }
    }

    // 2. Short-window buckets matching target or 4h/5h
    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
            for b in &g.buckets {
                let win = b.window.to_lowercase();
                let bid = b.bucket_id.to_lowercase();
                let is_short = win.contains("5h")
                    || win.contains("4h")
                    || bid.contains("5h")
                    || bid.contains("4h");
                if is_short && (0.0..=1.0).contains(&b.remaining_fraction) {
                    return Some((b.remaining_fraction * 100.0).round());
                }
            }
        }
    }

    // 3. Hierarchical match (Gemini Flash -> Claude Sonnet -> other Gemini)
    if let Some((_, pct, _)) = evaluate_hierarchical_quota(account) {
        return Some(pct);
    }

    // 4. Fallback: average percentage across models
    if !quota_data.models.is_empty() {
        let total: i32 = quota_data.models.iter().map(|m| m.percentage).sum();
        let avg = total as f64 / quota_data.models.len() as f64;
        return Some(avg);
    }

    None
}

/// Parse an ISO 8601 or RFC 3339 reset time string into UNIX timestamp (seconds)
pub fn parse_reset_time_to_unix(reset_time_str: &str) -> Option<i64> {
    let trimmed = reset_time_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(dt.timestamp());
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp());
    }
    None
}

/// Calculate quota percentage and period boundary status for an account
pub fn evaluate_account_period_status(
    account: &Account,
    target_model: &str,
    threshold_percent: f64,
    now_sec: i64,
) -> Option<QuotaPeriodStatus> {
    let quota_data = account.quota.as_ref()?;
    let target = target_model.to_lowercase();
    let is_flash_target = target.contains("flash");

    let mut matched_model: Option<&crate::models::quota::ModelQuota> = None;
    // 1. Evaluate configured target_model first
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && name_lower.contains("flash");
        if is_direct || is_flash {
            match matched_model {
                Some(cur) if m.percentage < cur.percentage => matched_model = Some(m),
                None => matched_model = Some(m),
                _ => {}
            }
        }
    }

    // 2. In addition, evaluate lowest remaining quota across all models that are actively consumed (remaining < 100%)
    for m in &quota_data.models {
        if m.percentage < 100 || (m.percentage as f64) <= threshold_percent {
            match matched_model {
                Some(cur) if m.percentage < cur.percentage => matched_model = Some(m),
                None => matched_model = Some(m),
                _ => {}
            }
        }
    }

    let (mut quota_percent, mut reset_time_str) = if let Some(m) = matched_model {
        (m.percentage as f64, m.reset_time.as_str())
    } else if !quota_data.models.is_empty() {
        let total: i32 = quota_data.models.iter().map(|m| m.percentage).sum();
        let avg = total as f64 / quota_data.models.len() as f64;
        let first_reset = quota_data
            .models
            .iter()
            .find(|m| !m.reset_time.is_empty())
            .map(|m| m.reset_time.as_str())
            .unwrap_or("");
        (avg, first_reset)
    } else if quota_data
        .quota_groups
        .as_ref()
        .is_some_and(|g| !g.is_empty())
    {
        (100.0, "")
    } else {
        return None;
    };

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
                    let bucket_pct = (b.remaining_fraction * 100.0).round();
                    if bucket_pct < quota_percent {
                        quota_percent = bucket_pct;
                        if !b.reset_time.is_empty() {
                            reset_time_str = b.reset_time.as_str();
                        }
                    }
                }
            }
        }
    }

    let reset_timestamp = parse_reset_time_to_unix(reset_time_str);
    let (seconds_until_reset, is_period_finished) = match reset_timestamp {
        Some(reset_ts) => {
            let diff = reset_ts - now_sec;
            let finished = diff <= 0;
            (Some(diff), finished)
        }
        None => (None, false),
    };

    let is_low_quota = quota_percent <= threshold_percent;
    let below_threshold = is_low_quota;
    let mut is_depleted_before_finish = false;
    if is_low_quota {
        if reset_timestamp.is_some() {
            if !is_period_finished {
                is_depleted_before_finish = true;
            }
        }
    }

    Some(QuotaPeriodStatus {
        quota_percent,
        reset_time_iso: if reset_time_str.is_empty() {
            None
        } else {
            Some(reset_time_str.to_string())
        },
        reset_timestamp,
        seconds_until_reset,
        is_period_finished,
        is_depleted_before_finish,
        is_low_quota,
        below_threshold,
    })
}
