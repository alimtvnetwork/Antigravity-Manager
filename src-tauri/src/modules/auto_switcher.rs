use crate::models::config::AutoProfileSwitcherConfig;
use crate::models::Account;
use crate::modules::{account, config, instance, logger};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitcherStatus {
    pub is_running: bool,
    pub active_instance_id: String,
    pub active_account_email: Option<String>,
    pub current_quota_percent: Option<f64>,
    pub last_check_timestamp: i64,
    pub last_switch_timestamp: Option<i64>,
    pub last_switch_reason: Option<String>,
    #[serde(default)]
    pub monitored_instance_count: usize,
    #[serde(default)]
    pub monitored_instances: Vec<InstanceQuotaSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InstanceQuotaSummary {
    pub instance_id: String,
    pub instance_name: String,
    pub bound_email: Option<String>,
    pub quota_percent: Option<f64>,
    pub reset_time_iso: Option<String>,
    pub seconds_until_reset: Option<i64>,
    pub is_running: bool,
    pub is_depleted_before_finish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaPeriodStatus {
    pub quota_percent: f64,
    pub reset_time_iso: Option<String>,
    pub reset_timestamp: Option<i64>,
    pub seconds_until_reset: Option<i64>,
    pub is_period_finished: bool,
    pub is_depleted_before_finish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecoverySnapshot {
    pub instance_id: String,
    pub account_id: String,
    pub timestamp: i64,
    pub reason: String,
    pub is_recovered: bool,
}

struct AutoSwitcherRuntimeState {
    pub is_running: bool,
    pub last_check_timestamp: i64,
    pub last_switch_timestamp: Option<i64>,
    pub last_switch_reason: Option<String>,
}

static RUNTIME_STATE: Lazy<Mutex<AutoSwitcherRuntimeState>> = Lazy::new(|| {
    Mutex::new(AutoSwitcherRuntimeState {
        is_running: false,
        last_check_timestamp: 0,
        last_switch_timestamp: None,
        last_switch_reason: None,
    })
});

/// Path to recovery snapshots folder
pub fn get_recovery_dir() -> Result<PathBuf, String> {
    let data_dir = account::get_data_dir()?;
    let recovery_dir = data_dir.join("task_recovery");
    if !recovery_dir.exists() {
        fs::create_dir_all(&recovery_dir)
            .map_err(|e| format!("Failed to create recovery dir: {}", e))?;
    }
    Ok(recovery_dir)
}

/// Snapshot current task and active instance before switching
pub fn snapshot_task_state(
    instance_id: &str,
    account_id: &str,
    reason: &str,
) -> Result<(), String> {
    let dir = get_recovery_dir()?;
    let now = chrono::Utc::now().timestamp();
    let snapshot = TaskRecoverySnapshot {
        instance_id: instance_id.to_string(),
        account_id: account_id.to_string(),
        timestamp: now,
        reason: reason.to_string(),
        is_recovered: false,
    };

    let path = dir.join(format!("snapshot_{}.json", instance_id));
    let content = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write snapshot: {}", e))?;

    logger::log_info(&format!(
        "[AutoSwitcher] Saved task recovery snapshot for instance '{}'",
        instance_id
    ));
    Ok(())
}

/// Calculate quota percentage for a specific target model in an account
pub fn calculate_account_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    let target = target_model.to_lowercase();
    let is_flash_target = target.contains("flash");

    let mut min_pct: Option<f64> = None;

    // 1. Minimum across models matching target (and non-banned flash models when target is flash)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && !is_banned && name_lower.contains("flash");
        if is_direct || is_flash {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    // 2. Minimum across any actively consumed non-banned models (percentage < 100)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        if !is_banned && m.percentage < 100 {
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
                    || (!win.contains("week") && !win.contains("7d") && !bid.contains("week") && !bid.contains("7d"));
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
/// 1. Primary: Gemini 3.8 Flash (banning 3.0 / 3.1 Flash)
/// 2. Fallback: Claude Sonnet 4.6 / Claude Sonnet
pub fn evaluate_hierarchical_quota(account: &Account) -> Option<(String, f64, bool)> {
    let quota_data = account.quota.as_ref()?;

    // 1. Check Gemini 3.8 Flash (Primary)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        if is_banned {
            continue;
        }
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

    // 3. Any other non-banned Gemini model
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        if is_banned {
            continue;
        }
        if name_lower.contains("gemini") {
            let pct = m.percentage as f64;
            let is_primary = false;
            return Some((m.name.clone(), pct, is_primary));
        }
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
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && !is_banned && name_lower.contains("flash");
        if is_direct || is_flash {
            match matched_model {
                Some(cur) if m.percentage < cur.percentage => matched_model = Some(m),
                None => matched_model = Some(m),
                _ => {}
            }
        }
    }

    // Also check if any non-banned model has been consumed below 100% or <= threshold_percent
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        if !is_banned && (m.percentage as f64 <= threshold_percent || m.percentage < 100) {
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
                    || (!win.contains("week") && !win.contains("7d") && !bid.contains("week") && !bid.contains("7d"));
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
    })
}

/// List instances that are either currently running or marked as active
pub fn list_running_or_active_instances() -> Result<Vec<crate::models::InstanceConfig>, String> {
    let registry = instance::load_registry()?;
    let active_id = registry.active_instance_id.clone();
    let mut result = Vec::new();

    for inst in registry.instances {
        let is_active = inst.id == active_id;
        let is_running = instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);

        if is_active {
            result.push(inst);
        } else if is_running {
            result.push(inst);
        }
    }

    Ok(result)
}

/// Get account IDs currently bound to any running or active instance
pub fn get_active_in_use_account_ids() -> Vec<String> {
    let instances = list_running_or_active_instances().unwrap_or_default();
    let mut in_use = Vec::new();
    for inst in instances {
        let acc_id_opt = inst
            .bound_account_id
            .clone()
            .or_else(|| account::get_current_account_id().ok().flatten());
        if let Some(acc_id) = acc_id_opt {
            if !in_use.contains(&acc_id) {
                in_use.push(acc_id);
            }
        }
    }
    in_use
}

/// Candidate profile target scored for auto-switch
pub struct ProfileCandidate {
    pub instance_id: String,
    pub account_id: String,
    pub email: String,
    pub quota_percent: f64,
    pub score: f64,
}

/// Multiplicative candidate scoring algorithm:
/// Score = S_active * M_tier * Q_weekly
/// - S_active: 1.0 if unused, 0.0 if in active use
/// - M_tier: Ultra=5.0, Pro=3.0, Free=1.0
/// - Q_weekly: 0.0 to 100.0 percentage
pub fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    // 1. Subscription tier multiplier
    let tier = acc
        .quota
        .as_ref()
        .and_then(|q| q.subscription_tier.as_ref())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let tier_multiplier = if tier.contains("ultra") {
        5.0
    } else if tier.contains("pro") {
        3.0
    } else {
        1.0
    };

    // 2. Weekly quota percentage from quota_groups or fallback to models
    let mut weekly_quota_percent = 100.0;
    if let Some(quota_data) = acc.quota.as_ref() {
        let mut weekly_values: Vec<f64> = Vec::new();
        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    let win = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    let is_weekly = win.contains("week") || bid.contains("week");
                    if is_weekly {
                        weekly_values.push((b.remaining_fraction * 100.0).round());
                    }
                }
            }
        }

        if !weekly_values.is_empty() {
            // Take minimum bottleneck across weekly quota groups
            weekly_quota_percent = weekly_values.into_iter().fold(100.0, f64::min);
        } else {
            let valid_models: Vec<_> = quota_data
                .models
                .iter()
                .filter(|m| {
                    let name = m.name.to_lowercase();
                    let is_banned = name.contains("3.0") || name.contains("3.1");
                    !is_banned
                })
                .collect();

            if !valid_models.is_empty() {
                let sum: i32 = valid_models.iter().map(|m| m.percentage).sum();
                weekly_quota_percent = (sum as f64 / valid_models.len() as f64).round();
            }
        }
    }

    // 3. Reset boundary check: if period has finished, provider will refresh credits to 100%
    if let Some(period_stat) = evaluate_account_period_status(acc, target_model, 20.0, now_sec) {
        if period_stat.is_period_finished {
            weekly_quota_percent = 100.0;
        }
    }

    let active_factor = 1.0;
    active_factor * tier_multiplier * weekly_quota_percent
}

/// Specifically evaluate the 4-hour / 5-hour immediate rolling window quota (0-100%)
pub fn calculate_4h_window_quota(account: &Account, target_model: &str) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
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
                    || (!win.contains("week") && !win.contains("7d") && !bid.contains("week") && !bid.contains("7d"));
                if is_short_window && (0.0..=1.0).contains(&b.remaining_fraction) {
                    let pct = (b.remaining_fraction * 100.0).round();
                    min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
                }
            }
        }
    }

    // 2. Minimum across models matching target (and non-banned flash models when target is flash)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        let is_direct = name_lower.contains(&target);
        let is_flash = is_flash_target && !is_banned && name_lower.contains("flash");
        if is_direct || is_flash {
            let pct = m.percentage as f64;
            min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
        }
    }

    // 3. Minimum across any actively consumed non-banned models (percentage < 100)
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
        if !is_banned && m.percentage < 100 {
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

/// Discover, score, and rank candidate profiles with 100% 4h quota accounts prioritized
pub fn select_candidate_profiles(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
    excluded_account_ids: &[String],
) -> Result<Vec<ProfileCandidate>, String> {
    let registry = instance::load_registry()?;
    let now_sec = chrono::Utc::now().timestamp();
    let mut candidates = Vec::new();

    let mut effective_exclusions = excluded_account_ids.to_vec();
    for cross_vm_acc in crate::modules::email_inbound::fetch_recent_cross_vm_switched_accounts(3600)
    {
        if !effective_exclusions.contains(&cross_vm_acc) {
            effective_exclusions.push(cross_vm_acc);
        }
    }

    // 1. Inspect instances not in active use
    for inst in &registry.instances {
        if inst.id == current_instance_id {
            continue;
        }

        let Some(ref acc_id) = inst.bound_account_id else {
            continue;
        };

        if effective_exclusions.contains(acc_id) {
            continue;
        }

        if let Ok(acc) = account::load_account(acc_id) {
            if effective_exclusions.contains(&acc.email) {
                continue;
            }
            if acc.disabled || acc.validation_blocked {
                continue;
            }
            if crate::modules::workspace_lease_manager::is_account_leased_by_other(&acc.id) {
                continue;
            }

            let period_status =
                evaluate_account_period_status(&acc, target_model, threshold, now_sec);
            let quota = period_status
                .as_ref()
                .map(|s| s.quota_percent)
                .or_else(|| calculate_4h_window_quota(&acc, target_model))
                .unwrap_or(100.0);

            let is_period_finished = period_status
                .as_ref()
                .map(|s| s.is_period_finished)
                .unwrap_or(false);

            if quota > threshold || is_period_finished {
                let score = score_candidate_account(&acc, target_model, now_sec);
                candidates.push(ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                });
            }
        }
    }

    // 2. Check unbound accounts in pool
    let all_accounts = account::list_accounts().unwrap_or_default();
    for acc in &all_accounts {
        if effective_exclusions.contains(&acc.id) || effective_exclusions.contains(&acc.email) {
            continue;
        }
        if acc.disabled || acc.validation_blocked {
            continue;
        }
        if crate::modules::workspace_lease_manager::is_account_leased_by_other(&acc.id) {
            continue;
        }
        if candidates.iter().any(|c| c.account_id == acc.id) {
            continue;
        }

        let period_status = evaluate_account_period_status(acc, target_model, threshold, now_sec);
        let quota = period_status
            .as_ref()
            .map(|s| s.quota_percent)
            .or_else(|| calculate_4h_window_quota(acc, target_model))
            .unwrap_or(100.0);

        let is_period_finished = period_status
            .as_ref()
            .map(|s| s.is_period_finished)
            .unwrap_or(false);

        if quota > threshold || is_period_finished {
            let score = score_candidate_account(acc, target_model, now_sec);
            candidates.push(ProfileCandidate {
                instance_id: current_instance_id.to_string(),
                account_id: acc.id.clone(),
                email: acc.email.clone(),
                quota_percent: quota,
                score,
            });
        }
    }

    // 3. Fallback pass when testing with a high threshold (e.g., 98%) where standby accounts are at 16%-97%
    if candidates.is_empty() {
        for acc in &all_accounts {
            if effective_exclusions.contains(&acc.id) || effective_exclusions.contains(&acc.email) {
                continue;
            }
            if acc.disabled || acc.validation_blocked {
                continue;
            }
            if crate::modules::workspace_lease_manager::is_account_leased_by_other(&acc.id) {
                continue;
            }

            let period_status = evaluate_account_period_status(acc, target_model, 15.0, now_sec);
            let quota = period_status
                .as_ref()
                .map(|s| s.quota_percent)
                .or_else(|| calculate_4h_window_quota(acc, target_model))
                .unwrap_or(100.0);

            let is_period_finished = period_status
                .as_ref()
                .map(|s| s.is_period_finished)
                .unwrap_or(false);

            if quota > 15.0 || is_period_finished {
                let score = score_candidate_account(acc, target_model, now_sec);
                candidates.push(ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                });
            }
        }
    }

    // Priority Sorting:
    // 1. Accounts with quota_percent >= 100.0 strictly precede accounts with < 100.0.
    // 2. Among >= 100.0 accounts: sort by subscription tier score descending (Ultra > Pro > Free).
    // 3. Among < 100.0 accounts: sort by quota_percent descending, then score descending.
    // 4. Tie-breaker: deterministic email ordering.
    candidates.sort_by(|a, b| {
        let a_full = a.quota_percent >= 100.0;
        let b_full = b.quota_percent >= 100.0;
        match (a_full, b_full) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                if !a_full && !b_full {
                    match b.quota_percent.partial_cmp(&a.quota_percent) {
                        Some(std::cmp::Ordering::Equal) | None => {
                            match b.score.partial_cmp(&a.score) {
                                Some(std::cmp::Ordering::Equal) | None => {
                                    a.email.to_lowercase().cmp(&b.email.to_lowercase())
                                }
                                Some(ord) => ord,
                            }
                        }
                        Some(ord) => ord,
                    }
                } else {
                    match b.score.partial_cmp(&a.score) {
                        Some(std::cmp::Ordering::Equal) | None => {
                            a.email.to_lowercase().cmp(&b.email.to_lowercase())
                        }
                        Some(ord) => ord,
                    }
                }
            }
        }
    });

    Ok(candidates)
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
    let candidates = select_candidate_profiles(
        current_instance_id,
        target_model,
        threshold,
        excluded_account_ids,
    )?;

    if candidates.is_empty() {
        return Ok(None);
    }

    let mut verified_fallback: Option<ProfileCandidate> = None;

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

        // Live API Quota Refresh directly from Google API
        let fetch_res = account::fetch_quota_with_retry(&mut cand_acc).await;
        let fresh_4h_quota = match fetch_res {
            Ok(fresh_q) => {
                cand_acc.quota = Some(fresh_q);
                let _ = account::save_account(&cand_acc);
                let q_val = calculate_4h_window_quota(&cand_acc, target_model).unwrap_or(100.0);
                logger::log_info(&format!(
                    "[AutoSwitcher] Candidate '{}' refreshed from Google API: {:.1}% (4-hour window)",
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
            logger::log_warn(&format!(
                "[AutoSwitcher] Candidate '{}' live 4h window quota is {:.1}% (< 100.0%). Rejecting candidate and moving to next best...",
                candidate.email, fresh_4h_quota
            ));

            if fresh_4h_quota > threshold {
                match verified_fallback {
                    Some(ref cur) if fresh_4h_quota > cur.quota_percent => {
                        verified_fallback = Some(ProfileCandidate {
                            instance_id: candidate.instance_id.clone(),
                            account_id: candidate.account_id.clone(),
                            email: candidate.email.clone(),
                            quota_percent: fresh_4h_quota,
                            score: candidate.score,
                        });
                    }
                    None => {
                        verified_fallback = Some(ProfileCandidate {
                            instance_id: candidate.instance_id.clone(),
                            account_id: candidate.account_id.clone(),
                            email: candidate.email.clone(),
                            quota_percent: fresh_4h_quota,
                            score: candidate.score,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(fallback) = verified_fallback {
        logger::log_info(&format!(
            "[AutoSwitcher] No candidates verified at 100.0% for 4h window. Using highest verified candidate: '{}' ({:.1}%)",
            fallback.email, fallback.quota_percent
        ));
        return Ok(Some(fallback));
    }

    logger::log_warn("[AutoSwitcher] No candidates with healthy quota found after live verification.");
    Ok(None)
}

#[derive(Debug, Clone, Default)]
pub struct RotationContext {
    pub previous_email: Option<String>,
    pub predicted_email: Option<String>,
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
    let inst_id = &target.instance_id;

    // Step 1: Split Repo DB - Snapshot running prompts for this instance before switching
    let _ = crate::modules::repo_db::backup_running_prompts(inst_id);

    if has_auto_resume {
        let _ = snapshot_task_state(inst_id, &target.account_id, &reason);
    }

    logger::log_info(&format!(
        "[AutoSwitcher] Rotating instance '{}' to account '{}' (email: {}, quota: {:.1}%). Reason: {}",
        inst_id, target.account_id, target.email, target.quota_percent, reason
    ));

    // Step 2: Trigger unified Email and Telegram notifications before switch
    let prev_email = ctx.as_ref().and_then(|c| c.previous_email.clone());
    let predicted = ctx
        .as_ref()
        .and_then(|c| c.predicted_email.clone())
        .unwrap_or_else(|| target.email.clone());
    let credit_before = ctx.as_ref().and_then(|c| c.credit_before_switch);
    let thresh = ctx.as_ref().and_then(|c| c.threshold_activated);

    crate::modules::notification_hub::notify_account_switched_details(
        crate::modules::notification_hub::SwitchNotificationDetails {
            previous_email: prev_email,
            predicted_next_email: Some(predicted),
            selected_email: target.email.clone(),
            credit_before_switch: credit_before,
            threshold_activated: thresh,
            instance_id: inst_id.clone(),
            instance_name: inst_id.clone(),
            instance_mode: String::new(),
            reason: reason.clone(),
            is_auto: true,
        },
    );

    instance::switch_account_to_instance(&target.account_id, Some(inst_id)).await?;

    // Step 2.5: Acquire distributed lease in Supabase Root DB (prevent other nodes from selecting it)
    let target_acc_id = target.account_id.clone();
    let target_inst_id = target.instance_id.clone();
    tokio::spawn(async move {
        let _ = crate::modules::workspace_lease_manager::acquire_lease(
            &target_acc_id,
            &target_inst_id,
            90,
        )
        .await;
    });

    // Step 3: Split Repo DB - Directly send/dispatch backed-up prompts to running projects without queuing
    let _ = crate::modules::repo_db::dispatch_running_prompts(inst_id);
    let _ = crate::modules::repo_db::resend_all_running_commands(20);

    // Auto-resume recent active prompts (<1h) if configured
    let app_config = config::load_app_config().unwrap_or_default();
    if app_config.auto_profile_switcher.auto_resume_recent_prompts {
        let threshold = app_config
            .auto_profile_switcher
            .prompt_recency_threshold_seconds as i64;
        let _ = crate::modules::repo_db::auto_resume_recent_prompts(inst_id, threshold);
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

    let monitored_instances = list_running_or_active_instances()?;
    if monitored_instances.is_empty() {
        return Ok(None);
    }

    let in_use_account_ids = get_active_in_use_account_ids();
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
        let bound_acc_id_opt = inst
            .bound_account_id
            .clone()
            .or_else(|| account::get_current_account_id().ok().flatten());

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
            let _ = account::save_account(&bound_acc);
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
            .or_else(|| calculate_account_quota(&bound_acc, &switcher_cfg.target_model))
            .unwrap_or(100.0);

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
            .await? {
                let rot_ctx = RotationContext {
                    previous_email: Some(bound_acc.email.clone()),
                    predicted_email: Some(candidate.email.clone()),
                    credit_before_switch: Some(quota_percent),
                    threshold_activated: Some(switcher_cfg.critical_threshold_percent),
                };
                execute_profile_rotation_with_context(
                    candidate,
                    reason.clone(),
                    switcher_cfg.has_auto_resume,
                    Some(rot_ctx),
                )
                .await?;
                rotated_reasons.push(reason);
                continue;
            }
        }

        // 2. Standard low-quota or depleted before finish check
        if is_period_finished {
            logger::log_info(&format!(
                "[AutoSwitcher] Instance '{}' quota period finished (reset time reached). Ready for quota reset.",
                inst.id
            ));
            continue;
        }

        let is_low_quota = quota_percent <= effective_low_threshold;
        let mut should_rotate = false;
        if is_depleted_before_finish {
            should_rotate = true;
        } else if is_low_quota {
            should_rotate = true;
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
            .await? {
                let rot_ctx = RotationContext {
                    previous_email: Some(bound_acc.email.clone()),
                    predicted_email: Some(candidate.email.clone()),
                    credit_before_switch: Some(quota_percent),
                    threshold_activated: Some(effective_low_threshold),
                };
                execute_profile_rotation_with_context(
                    candidate,
                    reason.clone(),
                    switcher_cfg.has_auto_resume,
                    Some(rot_ctx),
                )
                .await?;
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

pub async fn check_and_rotate_for_threshold(
    custom_threshold: Option<f64>,
    force: bool,
) -> Result<Option<String>, String> {
    check_and_rotate_with_options(custom_threshold, force).await
}

/// Calculate dynamic polling interval based on credit quota ladder:
/// - >= 15%: check_interval_seconds (default 300s / 5m)
/// - < 15%: caution_interval_seconds (default 60s / 1m)
/// - <= 12%: critical_interval_seconds (default 40s)
pub fn calculate_next_interval_seconds(
    quota_percent: Option<f64>,
    cfg: &AutoProfileSwitcherConfig,
) -> u32 {
    let Some(quota) = quota_percent else {
        return cfg.check_interval_seconds.max(15);
    };

    let caution_threshold = if cfg.low_quota_threshold_percent > cfg.critical_threshold_percent {
        cfg.low_quota_threshold_percent
    } else {
        15.0
    };

    if quota <= cfg.critical_threshold_percent {
        cfg.critical_interval_seconds.max(10)
    } else if quota < caution_threshold {
        cfg.caution_interval_seconds.max(15)
    } else {
        cfg.check_interval_seconds.max(15)
    }
}

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
    let in_use_account_ids = get_active_in_use_account_ids();

    let registry = instance::load_registry()?;
    let current_bound = registry
        .instances
        .iter()
        .find(|i| i.id == inst_id)
        .and_then(|i| i.bound_account_id.clone())
        .or_else(|| account::get_current_account_id().ok().flatten());

    let mut excluded: Vec<String> = in_use_account_ids;
    if let Some(ref curr) = current_bound {
        if !excluded.contains(curr) {
            excluded.push(curr.clone());
        }
    }

    let candidate = select_and_verify_next_best_profile(&inst_id, &switcher_cfg.target_model, 0.0, &excluded)
        .await?
        .ok_or_else(|| "No alternative healthy profile found in pool".to_string())?;

    let reason = "Manual rotation triggered by user".to_string();
    let email = candidate.email.clone();
    let effective_inst = candidate.instance_id.clone();

    let (prev_email, prev_quota) = match current_bound
        .as_ref()
        .and_then(|id| account::load_account(id).ok())
    {
        Some(acc) => {
            let q = calculate_account_quota(&acc, &switcher_cfg.target_model);
            (Some(acc.email), q)
        }
        None => (None, None),
    };

    let rot_ctx = RotationContext {
        previous_email: prev_email,
        predicted_email: Some(email.clone()),
        credit_before_switch: prev_quota,
        threshold_activated: Some(switcher_cfg.low_quota_threshold_percent),
    };

    execute_profile_rotation_with_context(
        candidate,
        reason,
        switcher_cfg.has_auto_resume,
        Some(rot_ctx),
    )
    .await?;

    Ok(format!(
        "Successfully rotated profile '{}' to account '{}'",
        effective_inst, email
    ))
}

/// Trigger manual rotation to the next best profile
pub async fn trigger_manual_rotation() -> Result<String, String> {
    trigger_manual_rotation_for_instance(None).await
}

/// Get current auto-switcher status
pub fn get_status() -> AutoSwitcherStatus {
    let registry = instance::load_registry().unwrap_or_default();
    let active_id = registry.active_instance_id.clone();
    let app_config = config::load_app_config().unwrap_or_default();
    let switcher_cfg = app_config.auto_profile_switcher;
    let now_sec = chrono::Utc::now().timestamp();

    let running_or_active =
        list_running_or_active_instances().unwrap_or_else(|_| registry.instances.clone());
    let mut monitored_instances = Vec::new();

    let mut active_email = None;
    let mut active_quota = None;

    for inst in &running_or_active {
        let is_running = instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        let mut quota_pct = None;
        let mut reset_iso = None;
        let mut secs_until = None;
        let mut is_depleted_before_finish = false;

        let acc_id_opt = inst
            .bound_account_id
            .clone()
            .or_else(|| account::get_current_account_id().ok().flatten());
        let mut email_opt = inst.bound_email.clone();

        if let Some(ref acc_id) = acc_id_opt {
            if let Ok(acc) = account::load_account(acc_id) {
                if email_opt.is_none() {
                    email_opt = Some(acc.email.clone());
                }
                if let Some(period_stat) = evaluate_account_period_status(
                    &acc,
                    &switcher_cfg.target_model,
                    switcher_cfg.low_quota_threshold_percent,
                    now_sec,
                ) {
                    quota_pct = Some(period_stat.quota_percent);
                    reset_iso = period_stat.reset_time_iso;
                    secs_until = period_stat.seconds_until_reset;
                    is_depleted_before_finish = period_stat.is_depleted_before_finish;
                } else {
                    quota_pct = calculate_account_quota(&acc, &switcher_cfg.target_model);
                }
            }
        }

        if inst.id == active_id {
            active_email = email_opt.clone();
            active_quota = quota_pct;
        }

        monitored_instances.push(InstanceQuotaSummary {
            instance_id: inst.id.clone(),
            instance_name: inst.name.clone(),
            bound_email: email_opt,
            quota_percent: quota_pct,
            reset_time_iso: reset_iso,
            seconds_until_reset: secs_until,
            is_running,
            is_depleted_before_finish,
        });
    }

    let state = RUNTIME_STATE.lock().unwrap();
    AutoSwitcherStatus {
        is_running: state.is_running,
        active_instance_id: active_id,
        active_account_email: active_email,
        current_quota_percent: active_quota,
        last_check_timestamp: state.last_check_timestamp,
        last_switch_timestamp: state.last_switch_timestamp,
        last_switch_reason: state.last_switch_reason.clone(),
        monitored_instance_count: monitored_instances.len(),
        monitored_instances,
    }
}

/// Check active instance health passively without spawning IDE windows or stealing OS window focus
pub async fn check_and_recover_crashed_instance() -> Result<(), String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;
    if !switcher_cfg.is_enabled {
        return Ok(());
    }

    let registry = instance::load_registry()?;
    let active_id = registry.active_instance_id.clone();
    let active_inst = match registry.instances.iter().find(|i| i.id == active_id) {
        Some(inst) => inst,
        None => return Ok(()),
    };

    let pids = instance::find_pids_for_data_dir(&active_inst.data_dir, active_inst.is_default);
    let is_running = !pids.is_empty();

    if !is_running {
        // Only clean stale lockfiles passively; NEVER call trigger_manual_rotation() or launch IDE windows
        // automatically in the background, as that causes open/close loops and steals focus from Windows Explorer.
        crate::modules::process::clean_antigravity_lockfiles(Some("ide"));
    }

    Ok(())
}

/// Start background auto-switcher daemon
pub fn start_auto_switcher() {
    tauri::async_runtime::spawn(async move {
        logger::log_info("[AutoSwitcher] Background daemon initialized.");
        {
            let mut state = RUNTIME_STATE.lock().unwrap();
            state.is_running = true;
        }

        let initial_cfg = config::load_app_config()
            .unwrap_or_default()
            .auto_profile_switcher;
        let mut last_enabled = initial_cfg.is_enabled;
        let mut last_threshold = initial_cfg.low_quota_threshold_percent;

        if last_enabled {
            tokio::time::sleep(Duration::from_secs(3)).await;
            if let Err(e) = check_and_rotate_if_needed().await {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Error during initial check cycle: {}",
                    e
                ));
            }
        }

        loop {
            let app_config = config::load_app_config().unwrap_or_default();
            let switcher_cfg = app_config.auto_profile_switcher;
            let cur_status = get_status();
            let lowest_monitored_quota = cur_status
                .monitored_instances
                .iter()
                .filter_map(|i| i.quota_percent)
                .fold(cur_status.current_quota_percent, |acc, q| match acc {
                    Some(a) => Some(a.min(q)),
                    None => Some(q),
                });
            let interval_secs =
                calculate_next_interval_seconds(lowest_monitored_quota, &switcher_cfg) as u64;

            let tick = 5u64;
            let mut elapsed = 0u64;
            while elapsed < interval_secs {
                let sleep_dur = tick.min(interval_secs.saturating_sub(elapsed));
                if sleep_dur == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(sleep_dur)).await;
                elapsed += sleep_dur;

                let latest_cfg = config::load_app_config()
                    .unwrap_or_default()
                    .auto_profile_switcher;
                let enabled_turned_on = !last_enabled && latest_cfg.is_enabled;
                let threshold_changed =
                    (latest_cfg.low_quota_threshold_percent - last_threshold).abs() > f64::EPSILON;

                last_enabled = latest_cfg.is_enabled;
                last_threshold = latest_cfg.low_quota_threshold_percent;

                if enabled_turned_on || threshold_changed {
                    logger::log_info(&format!(
                        "[AutoSwitcher] Config change detected (enabled={}, threshold={:.1}%), triggering immediate check.",
                        latest_cfg.is_enabled, latest_cfg.low_quota_threshold_percent
                    ));
                    break;
                }
            }

            if let Err(e) = check_and_rotate_if_needed().await {
                logger::log_warn(&format!("[AutoSwitcher] Error during check cycle: {}", e));
            }
        }
    });

    // Spawn dedicated 2-minute IDE Crash Recovery & Focus Watchdog
    tauri::async_runtime::spawn(async move {
        logger::log_info("[CrashWatchdog] 2-Minute IDE crash recovery & focus watchdog started.");
        loop {
            let app_config = config::load_app_config().unwrap_or_default();
            let interval = app_config
                .auto_profile_switcher
                .watchdog_interval_seconds
                .max(30);
            tokio::time::sleep(Duration::from_secs(interval as u64)).await;

            if let Err(e) = check_and_recover_crashed_instance().await {
                logger::log_warn(&format!(
                    "[CrashWatchdog] Error during crash watchdog cycle: {}",
                    e
                ));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::AutoProfileSwitcherConfig;
    use crate::models::token::TokenData;

    fn make_test_account(
        id: &str,
        email: &str,
        model: &str,
        pct: i32,
        reset_time: &str,
    ) -> Account {
        let token = TokenData::new(
            "access_token".to_string(),
            "refresh_token".to_string(),
            3600,
            Some(email.to_string()),
            None,
            None,
            false,
            None,
        );
        let mut acc = Account::new(id.to_string(), email.to_string(), token);
        let model_quota = crate::models::quota::ModelQuota {
            name: model.to_string(),
            percentage: pct,
            reset_time: reset_time.to_string(),
            display_name: None,
            supports_images: None,
            supports_thinking: None,
            thinking_budget: None,
            recommended: None,
            max_tokens: None,
            max_output_tokens: None,
            supported_mime_types: None,
        };
        acc.quota = Some(crate::models::quota::QuotaData {
            models: vec![model_quota],
            last_updated: chrono::Utc::now().timestamp(),
            subscription_tier: Some("pro".to_string()),
            is_forbidden: false,
            forbidden_reason: None,
            model_forwarding_rules: std::collections::HashMap::new(),
            quota_groups: None,
        });
        acc
    }

    #[test]
    fn test_calculate_next_interval_seconds_ladder() {
        let cfg = AutoProfileSwitcherConfig::default();
        assert_eq!(cfg.check_interval_seconds, 300);
        assert_eq!(cfg.caution_interval_seconds, 60);
        assert_eq!(cfg.critical_interval_seconds, 40);
        assert_eq!(cfg.low_quota_threshold_percent, 15.0);
        assert_eq!(cfg.critical_threshold_percent, 12.0);

        assert_eq!(calculate_next_interval_seconds(None, &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(85.0), &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(15.0), &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(14.9), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(13.0), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(12.0), &cfg), 40);
        assert_eq!(calculate_next_interval_seconds(Some(5.0), &cfg), 40);
        assert_eq!(calculate_next_interval_seconds(Some(0.0), &cfg), 40);
    }

    #[test]
    fn test_parse_reset_time_to_unix() {
        let valid_rfc3339 = "2026-09-22T14:30:00Z";
        let parsed = parse_reset_time_to_unix(valid_rfc3339);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap(), 1790087400);

        let empty = "";
        assert_eq!(parse_reset_time_to_unix(empty), None);

        let invalid = "not-a-date";
        assert_eq!(parse_reset_time_to_unix(invalid), None);
    }

    #[test]
    fn test_evaluate_account_period_status_before_finish() {
        let now_sec = 1790080000;
        let future_time = "2026-09-22T14:30:00Z"; // 1790087400, +7400s
        let acc = make_test_account("acc-1", "test@domain.com", "gemini-pro", 8, future_time);

        let status = evaluate_account_period_status(&acc, "gemini-pro", 15.0, now_sec);
        assert!(status.is_some());
        let stat = status.unwrap();

        assert_eq!(stat.quota_percent, 8.0);
        assert!(stat.is_depleted_before_finish);
        let period_finished = stat.is_period_finished;
        assert!(!period_finished);
        assert_eq!(stat.seconds_until_reset, Some(7400));
    }

    #[test]
    fn test_evaluate_account_period_status_after_finish() {
        let now_sec = 1790090000;
        let past_time = "2026-09-22T14:30:00Z"; // 1790087400, -2600s
        let acc = make_test_account("acc-2", "test2@domain.com", "gemini-pro", 5, past_time);

        let status = evaluate_account_period_status(&acc, "gemini-pro", 15.0, now_sec);
        assert!(status.is_some());
        let stat = status.unwrap();

        assert_eq!(stat.quota_percent, 5.0);
        assert!(stat.is_period_finished);
        let depleted_before_finish = stat.is_depleted_before_finish;
        assert!(!depleted_before_finish);
        assert_eq!(stat.seconds_until_reset, Some(-2600));
    }

    #[test]
    fn test_score_candidate_account_reset_time_priority() {
        let now_sec = 1790090000;
        let past_time = "2026-09-22T14:30:00Z"; // Period finished (+15000 bonus)
        let future_time = "2026-09-22T20:00:00Z"; // 1790107200, runway bonus

        let mut acc_a = make_test_account("acc-a", "a@domain.com", "gemini-pro", 10, past_time);
        acc_a.last_used = now_sec - 3600;

        let mut acc_b = make_test_account("acc-b", "b@domain.com", "gemini-pro", 10, future_time);
        acc_b.last_used = now_sec - 3600;

        let score_a = score_candidate_account(&acc_a, "gemini-pro", now_sec);
        let score_b = score_candidate_account(&acc_b, "gemini-pro", now_sec);

        // Account A period finished resets to 100% (score: 1.0 * 1.0 * 100.0 = 100.0), higher than un-refilled 10% (score: 10.0)
        assert!(score_a > score_b);
    }

    #[test]
    fn test_score_candidate_account_weekly_quota_groups_bottleneck() {
        let now_sec = 1790090000;
        let future_time = "2026-09-30T14:30:00Z";

        // Candidate A: Pro tier, 100% weekly Gemini, 100% weekly Claude
        let mut acc_a = make_test_account(
            "synth_acc_a",
            "synth_a@test.local",
            "gemini-pro",
            100,
            future_time,
        );
        if let Some(ref mut q) = acc_a.quota {
            q.quota_groups = Some(vec![
                crate::models::quota::QuotaGroup {
                    display_name: "Gemini Models".to_string(),
                    description: None,
                    buckets: vec![crate::models::quota::QuotaBucket {
                        bucket_id: "gemini-weekly".to_string(),
                        window: "weekly".to_string(),
                        remaining_fraction: 1.0,
                        reset_time: future_time.to_string(),
                        observed_at: None,
                        cycle_start: None,
                        cycle_tokens: None,
                        display_name: None,
                        description: None,
                    }],
                },
                crate::models::quota::QuotaGroup {
                    display_name: "Claude and GPT models".to_string(),
                    description: None,
                    buckets: vec![crate::models::quota::QuotaBucket {
                        bucket_id: "claude-weekly".to_string(),
                        window: "weekly".to_string(),
                        remaining_fraction: 1.0,
                        reset_time: future_time.to_string(),
                        observed_at: None,
                        cycle_start: None,
                        cycle_tokens: None,
                        display_name: None,
                        description: None,
                    }],
                },
            ]);
        }

        // Candidate B: Pro tier, 21% weekly Gemini, 100% weekly Claude
        let mut acc_b = make_test_account(
            "synth_acc_b",
            "synth_b@test.local",
            "gemini-pro",
            100,
            future_time,
        );
        if let Some(ref mut q) = acc_b.quota {
            q.quota_groups = Some(vec![
                crate::models::quota::QuotaGroup {
                    display_name: "Gemini Models".to_string(),
                    description: None,
                    buckets: vec![crate::models::quota::QuotaBucket {
                        bucket_id: "gemini-weekly".to_string(),
                        window: "weekly".to_string(),
                        remaining_fraction: 0.21,
                        reset_time: future_time.to_string(),
                        observed_at: None,
                        cycle_start: None,
                        cycle_tokens: None,
                        display_name: None,
                        description: None,
                    }],
                },
                crate::models::quota::QuotaGroup {
                    display_name: "Claude and GPT models".to_string(),
                    description: None,
                    buckets: vec![crate::models::quota::QuotaBucket {
                        bucket_id: "claude-weekly".to_string(),
                        window: "weekly".to_string(),
                        remaining_fraction: 1.0,
                        reset_time: future_time.to_string(),
                        observed_at: None,
                        cycle_start: None,
                        cycle_tokens: None,
                        display_name: None,
                        description: None,
                    }],
                },
            ]);
        }

        let score_a = score_candidate_account(&acc_a, "gemini-pro", now_sec);
        let score_b = score_candidate_account(&acc_b, "gemini-pro", now_sec);

        // Account A: 1.0 * 3.0 * 100 = 300.0
        // Account B: 1.0 * 3.0 * 21 = 63.0
        assert_eq!(score_a, 300.0);
        assert_eq!(score_b, 63.0);
        assert!(score_a > score_b);
    }
}
