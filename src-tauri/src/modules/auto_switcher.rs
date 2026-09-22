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

    // Direct model match
    for m in &quota_data.models {
        if m.name.to_lowercase().contains(&target) {
            return Some(m.percentage as f64);
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

    let mut matched_model = None;
    for m in &quota_data.models {
        if m.name.to_lowercase().contains(&target) {
            matched_model = Some(m);
            break;
        }
    }

    let (quota_percent, reset_time_str) = if let Some(m) = matched_model {
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
    } else {
        return None;
    };

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
        if let Some(acc_id) = inst.bound_account_id {
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

/// Multi-factor scoring for candidate accounts matching instanceService
pub fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
    let mut score = 0.0;

    // 1. Idle time factor (Longest time not used, or never used)
    let has_never_used = acc.last_used == 0;
    if has_never_used {
        score += 100000.0;
    } else {
        let idle_hours = ((now_sec - acc.last_used) as f64 / 3600.0).max(0.0);
        score += (idle_hours * 1000.0).min(50000.0);
    }

    // 2. Lowest remaining quota factor
    let mut lowest_remaining = 100.0;
    if let Some(quota) = acc.quota.as_ref() {
        for m in &quota.models {
            let pct = m.percentage as f64;
            if pct < lowest_remaining {
                lowest_remaining = pct;
            }
        }
    }
    score += lowest_remaining * 200.0;

    // 3. Target model bonus
    if let Some(quota) = calculate_account_quota(acc, target_model) {
        score += quota * 100.0;
    }

    // 4. Subscription tier bonus
    let tier = acc
        .quota
        .as_ref()
        .and_then(|q| q.subscription_tier.as_ref())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if tier.contains("pro") || tier.contains("ultra") {
        score += 10000.0;
    }

    // 5. Reset Window Lookahead Factor
    if let Some(period_stat) = evaluate_account_period_status(acc, target_model, 20.0, now_sec) {
        if period_stat.is_period_finished {
            // Quota has elapsed reset boundary, provider will refresh credits
            score += 15000.0;
        } else if let Some(secs_left) = period_stat.seconds_until_reset {
            if secs_left > 0 {
                // When quota is healthy (> threshold), having more runway before reset adds stability
                let runway_bonus = ((secs_left as f64) / 3600.0 * 500.0).min(5000.0);
                score += runway_bonus;
            }
        }
    }

    score
}

/// Find next best candidate profile with healthy quota, excluding accounts in active use by other instances
pub fn select_next_best_profile(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
    excluded_account_ids: &[String],
) -> Result<Option<ProfileCandidate>, String> {
    let registry = instance::load_registry()?;
    let now_sec = chrono::Utc::now().timestamp();
    let mut candidates = Vec::new();

    // 1. Inspect instances not in active use
    for inst in &registry.instances {
        if inst.id == current_instance_id {
            continue;
        }

        let Some(ref acc_id) = inst.bound_account_id else {
            continue;
        };

        if excluded_account_ids.contains(acc_id) {
            continue;
        }

        if let Ok(acc) = account::load_account(acc_id) {
            if acc.disabled {
                continue;
            }
            if acc.validation_blocked {
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
                .or_else(|| calculate_account_quota(&acc, target_model))
                .unwrap_or(0.0);

            let is_period_finished = period_status
                .as_ref()
                .map(|s| s.is_period_finished)
                .unwrap_or(false);

            let mut is_eligible = false;
            if quota > threshold {
                is_eligible = true;
            } else if is_period_finished {
                is_eligible = true;
            }

            if is_eligible {
                let score = score_candidate_account(&acc, target_model, now_sec);
                let effective_quota = if is_period_finished {
                    if quota <= threshold {
                        100.0
                    } else {
                        quota
                    }
                } else {
                    quota
                };

                candidates.push(ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: effective_quota,
                    score,
                });
            }
        }
    }

    // 2. Check unbound accounts in pool
    let all_accounts = account::list_accounts().unwrap_or_default();
    for acc in all_accounts {
        if excluded_account_ids.contains(&acc.id) {
            continue;
        }
        if acc.disabled {
            continue;
        }
        if acc.validation_blocked {
            continue;
        }
        if crate::modules::workspace_lease_manager::is_account_leased_by_other(&acc.id) {
            continue;
        }
        if candidates.iter().any(|c| c.account_id == acc.id) {
            continue;
        }

        let period_status = evaluate_account_period_status(&acc, target_model, threshold, now_sec);
        let quota = period_status
            .as_ref()
            .map(|s| s.quota_percent)
            .or_else(|| calculate_account_quota(&acc, target_model))
            .unwrap_or(0.0);

        let is_period_finished = period_status
            .as_ref()
            .map(|s| s.is_period_finished)
            .unwrap_or(false);

        let mut is_eligible = false;
        if quota > threshold {
            is_eligible = true;
        } else if is_period_finished {
            is_eligible = true;
        }

        if is_eligible {
            let score = score_candidate_account(&acc, target_model, now_sec);
            let effective_quota = if is_period_finished {
                if quota <= threshold {
                    100.0
                } else {
                    quota
                }
            } else {
                quota
            };

            candidates.push(ProfileCandidate {
                instance_id: current_instance_id.to_string(),
                account_id: acc.id,
                email: acc.email,
                quota_percent: effective_quota,
                score,
            });
        }
    }

    // Sort descending by multi-factor score
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(best) = candidates.into_iter().next() {
        return Ok(Some(best));
    }

    Ok(None)
}

/// Execute rotation to target candidate
pub async fn execute_profile_rotation(
    target: ProfileCandidate,
    reason: String,
    has_auto_resume: bool,
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
    crate::modules::notification_hub::notify_account_switched(
        &target.email,
        inst_id,
        &reason,
        true,
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

    let now = chrono::Utc::now().timestamp();
    let mut state = RUNTIME_STATE.lock().unwrap();
    state.last_switch_timestamp = Some(now);
    state.last_switch_reason = Some(reason);

    Ok(())
}

/// Check all monitored instance copies' quota and auto-rotate any instance below threshold
pub async fn check_and_rotate_if_needed() -> Result<Option<String>, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;

    if !switcher_cfg.is_enabled {
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

    let mut rotated_reasons = Vec::new();

    for inst in monitored_instances {
        let Some(ref bound_acc_id) = inst.bound_account_id else {
            continue;
        };

        let bound_acc = match account::load_account(bound_acc_id) {
            Ok(acc) => acc,
            Err(_) => continue,
        };

        let period_status = evaluate_account_period_status(
            &bound_acc,
            &switcher_cfg.target_model,
            switcher_cfg.low_quota_threshold_percent,
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

        // Filter out accounts in use by OTHER instances
        let excluded_accounts: Vec<String> = in_use_account_ids
            .iter()
            .filter(|id| *id != bound_acc_id)
            .cloned()
            .collect();

        // 1. Critical threshold check (<= 12.0% or configured critical_threshold_percent)
        let is_critical = quota_percent <= switcher_cfg.critical_threshold_percent;
        if is_critical {
            if switcher_cfg.auto_fast_forward_on_critical {
                let reason = format!(
                    "Critical quota alert on instance '{}': model '{}' dropped to {:.1}% (<= {:.1}%). Fast-forwarding to highest credit candidate...",
                    inst.id, switcher_cfg.target_model, quota_percent, switcher_cfg.critical_threshold_percent
                );
                logger::log_warn(&format!("[AutoSwitcher] {}", reason));

                if let Some(candidate) = select_next_best_profile(
                    &inst.id,
                    &switcher_cfg.target_model,
                    switcher_cfg.critical_threshold_percent,
                    &excluded_accounts,
                )? {
                    execute_profile_rotation(
                        candidate,
                        reason.clone(),
                        switcher_cfg.has_auto_resume,
                    )
                    .await?;
                    rotated_reasons.push(reason);
                    continue;
                }
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

        let is_low_quota = quota_percent <= switcher_cfg.low_quota_threshold_percent;
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
                    switcher_cfg.target_model,
                    inst.id,
                    quota_percent,
                    switcher_cfg.low_quota_threshold_percent
                )
            };

            logger::log_info(&format!("[AutoSwitcher] {}", reason));

            if let Some(candidate) = select_next_best_profile(
                &inst.id,
                &switcher_cfg.target_model,
                switcher_cfg.low_quota_threshold_percent,
                &excluded_accounts,
            )? {
                execute_profile_rotation(candidate, reason.clone(), switcher_cfg.has_auto_resume)
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

/// Calculate dynamic polling interval based on credit quota ladder
pub fn calculate_next_interval_seconds(
    quota_percent: Option<f64>,
    cfg: &AutoProfileSwitcherConfig,
) -> u32 {
    let Some(quota) = quota_percent else {
        return cfg.check_interval_seconds.max(15);
    };

    if quota <= cfg.critical_threshold_percent {
        cfg.critical_interval_seconds.max(10)
    } else if quota < 20.0 {
        cfg.caution_interval_seconds.max(15)
    } else {
        cfg.check_interval_seconds.max(15)
    }
}

/// Trigger manual rotation to the next best profile
pub async fn trigger_manual_rotation() -> Result<String, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;
    let active_id = instance::get_active_instance_id()?;
    let in_use_account_ids = get_active_in_use_account_ids();

    let registry = instance::load_registry()?;
    let current_bound = registry
        .instances
        .iter()
        .find(|i| i.id == active_id)
        .and_then(|i| i.bound_account_id.clone());

    let excluded: Vec<String> = in_use_account_ids
        .into_iter()
        .filter(|id| Some(id) != current_bound.as_ref())
        .collect();

    let candidate =
        select_next_best_profile(&active_id, &switcher_cfg.target_model, 0.0, &excluded)?
            .ok_or_else(|| "No alternative healthy profile found in pool".to_string())?;

    let reason = "Manual rotation triggered by user".to_string();
    let email = candidate.email.clone();
    let inst_id = candidate.instance_id.clone();

    execute_profile_rotation(candidate, reason, switcher_cfg.has_auto_resume).await?;

    Ok(format!(
        "Successfully rotated to profile '{}' with account '{}'",
        inst_id, email
    ))
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

        if let Some(ref acc_id) = inst.bound_account_id {
            if let Ok(acc) = account::load_account(acc_id) {
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
            active_email = inst.bound_email.clone();
            active_quota = quota_pct;
        }

        monitored_instances.push(InstanceQuotaSummary {
            instance_id: inst.id.clone(),
            instance_name: inst.name.clone(),
            bound_email: inst.bound_email.clone(),
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

/// Start background auto-switcher daemon
pub fn start_auto_switcher() {
    tauri::async_runtime::spawn(async move {
        logger::log_info("[AutoSwitcher] Background daemon initialized.");
        {
            let mut state = RUNTIME_STATE.lock().unwrap();
            state.is_running = true;
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
                calculate_next_interval_seconds(lowest_monitored_quota, &switcher_cfg);

            tokio::time::sleep(Duration::from_secs(interval_secs as u64)).await;

            if let Err(e) = check_and_rotate_if_needed().await {
                logger::log_warn(&format!("[AutoSwitcher] Error during check cycle: {}", e));
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
        let mut cfg = AutoProfileSwitcherConfig::default();
        cfg.check_interval_seconds = 300;
        cfg.caution_interval_seconds = 180;
        cfg.critical_interval_seconds = 60;
        cfg.critical_threshold_percent = 12.0;

        assert_eq!(calculate_next_interval_seconds(None, &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(85.0), &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(20.0), &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(19.9), &cfg), 180);
        assert_eq!(calculate_next_interval_seconds(Some(13.0), &cfg), 180);
        assert_eq!(calculate_next_interval_seconds(Some(12.0), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(5.0), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(0.0), &cfg), 60);
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

        // Account A period finished gets +15000, scoring higher than distant runway bonus
        assert!(score_a > score_b);
    }
}
