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
fn calculate_account_quota(account: &Account, target_model: &str) -> Option<f64> {
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

/// Candidate profile target scored for auto-switch
struct ProfileCandidate {
    pub instance_id: String,
    pub account_id: String,
    pub email: String,
    pub quota_percent: f64,
    pub score: f64,
}

/// Multi-factor scoring for candidate accounts matching instanceService
fn score_candidate_account(acc: &Account, target_model: &str, now_sec: i64) -> f64 {
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

    score
}

/// Find next best candidate profile with healthy quota
fn select_next_best_profile(
    current_instance_id: &str,
    target_model: &str,
    threshold: f64,
) -> Result<Option<ProfileCandidate>, String> {
    let registry = instance::load_registry()?;
    let now_sec = chrono::Utc::now().timestamp();
    let mut candidates = Vec::new();

    for inst in &registry.instances {
        if inst.id == current_instance_id {
            continue;
        }

        let Some(ref acc_id) = inst.bound_account_id else {
            continue;
        };

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

            let quota = calculate_account_quota(&acc, target_model).unwrap_or(0.0);
            if quota > threshold {
                let score = score_candidate_account(&acc, target_model, now_sec);
                candidates.push(ProfileCandidate {
                    instance_id: inst.id.clone(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                });
            }
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

    // If no candidate instance has healthy quota, check unbound accounts in pool
    let all_accounts = account::list_accounts().unwrap_or_default();
    let mut account_candidates = Vec::new();

    for acc in all_accounts {
        if acc.disabled {
            continue;
        }
        if acc.validation_blocked {
            continue;
        }
        if crate::modules::workspace_lease_manager::is_account_leased_by_other(&acc.id) {
            continue;
        }
        let quota = calculate_account_quota(&acc, target_model).unwrap_or(0.0);
        if quota > threshold {
            let score = score_candidate_account(&acc, target_model, now_sec);
            account_candidates.push((acc, quota, score));
        }
    }

    account_candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((best_acc, quota, score)) = account_candidates.into_iter().next() {
        return Ok(Some(ProfileCandidate {
            instance_id: current_instance_id.to_string(),
            account_id: best_acc.id,
            email: best_acc.email,
            quota_percent: quota,
            score,
        }));
    }

    Ok(None)
}

/// Execute rotation to target candidate
pub async fn execute_profile_rotation(
    target: ProfileCandidate,
    reason: String,
    has_auto_resume: bool,
) -> Result<(), String> {
    let active_id = instance::get_active_instance_id().unwrap_or_default();

    // Step 1: Split Repo DB - Snapshot running prompts for all active projects before switching
    let _ = crate::modules::repo_db::backup_running_prompts(&active_id);

    if has_auto_resume {
        let _ = snapshot_task_state(&target.instance_id, &target.account_id, &reason);
    }

    logger::log_info(&format!(
        "[AutoSwitcher] Rotating to instance '{}' (email: {}, quota: {:.1}%). Reason: {}",
        target.instance_id, target.email, target.quota_percent, reason
    ));

    // Step 2: Trigger email notification just before the workspace switch
    crate::modules::email_watcher::notify_workspace_switched(
        &active_id,
        &target.instance_id,
        &reason,
    );

    instance::switch_account_to_instance(&target.account_id, Some(&target.instance_id)).await?;

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
    let _ = crate::modules::repo_db::dispatch_running_prompts(&target.instance_id);

    let now = chrono::Utc::now().timestamp();
    let mut state = RUNTIME_STATE.lock().unwrap();
    state.last_switch_timestamp = Some(now);
    state.last_switch_reason = Some(reason);

    Ok(())
}

/// Check active instance quota and auto-rotate if below threshold
pub async fn check_and_rotate_if_needed() -> Result<Option<String>, String> {
    let app_config = config::load_app_config()?;
    let switcher_cfg = app_config.auto_profile_switcher;

    if !switcher_cfg.is_enabled {
        return Ok(None);
    }

    let active_id = instance::get_active_instance_id()?;
    let registry = instance::load_registry()?;
    let active_inst = registry
        .instances
        .iter()
        .find(|i| i.id == active_id)
        .ok_or_else(|| format!("Active instance {} not found", active_id))?;

    let Some(ref bound_acc_id) = active_inst.bound_account_id else {
        return Ok(None);
    };

    let bound_acc = account::load_account(bound_acc_id)?;
    let quota_percent =
        calculate_account_quota(&bound_acc, &switcher_cfg.target_model).unwrap_or(100.0);

    let now = chrono::Utc::now().timestamp();
    {
        let mut state = RUNTIME_STATE.lock().unwrap();
        state.last_check_timestamp = now;
    }

    // 1. Critical threshold check (<= 12.0% or configured critical_threshold_percent)
    let is_critical = quota_percent <= switcher_cfg.critical_threshold_percent;
    if is_critical {
        if switcher_cfg.auto_fast_forward_on_critical {
            let reason = format!(
                "Critical quota alert: model '{}' dropped to {:.1}% (<= {:.1}%). Fast-forwarding to highest credit candidate...",
                switcher_cfg.target_model, quota_percent, switcher_cfg.critical_threshold_percent
            );
            logger::log_warn(&format!("[AutoSwitcher] {}", reason));

            if let Some(candidate) = select_next_best_profile(
                &active_id,
                &switcher_cfg.target_model,
                switcher_cfg.critical_threshold_percent,
            )? {
                execute_profile_rotation(candidate, reason.clone(), switcher_cfg.has_auto_resume)
                    .await?;
                return Ok(Some(reason));
            }
        }
    }

    // 2. Standard low-quota threshold check
    if quota_percent <= switcher_cfg.low_quota_threshold_percent {
        let reason = format!(
            "Quota for model '{}' dropped to {:.1}% (threshold: {:.1}%)",
            switcher_cfg.target_model, quota_percent, switcher_cfg.low_quota_threshold_percent
        );

        if let Some(candidate) = select_next_best_profile(
            &active_id,
            &switcher_cfg.target_model,
            switcher_cfg.low_quota_threshold_percent,
        )? {
            execute_profile_rotation(candidate, reason.clone(), switcher_cfg.has_auto_resume)
                .await?;
            return Ok(Some(reason));
        } else {
            logger::log_warn(&format!(
                "[AutoSwitcher] Quota low ({:.1}%) but no healthy alternative profile found.",
                quota_percent
            ));
        }
    }

    Ok(None)
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

    let candidate = select_next_best_profile(&active_id, &switcher_cfg.target_model, 0.0)?
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
    let active_inst = registry.instances.into_iter().find(|i| i.id == active_id);

    let email = active_inst.as_ref().and_then(|i| i.bound_email.clone());
    let quota = active_inst
        .as_ref()
        .and_then(|i| i.bound_account_id.as_ref())
        .and_then(|id| account::load_account(id).ok())
        .and_then(|acc| calculate_account_quota(&acc, "gemini-pro"));

    let state = RUNTIME_STATE.lock().unwrap();
    AutoSwitcherStatus {
        is_running: state.is_running,
        active_instance_id: active_id,
        active_account_email: email,
        current_quota_percent: quota,
        last_check_timestamp: state.last_check_timestamp,
        last_switch_timestamp: state.last_switch_timestamp,
        last_switch_reason: state.last_switch_reason.clone(),
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
            let interval_secs = calculate_next_interval_seconds(
                cur_status.current_quota_percent,
                &switcher_cfg,
            );

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

    #[test]
    fn test_calculate_next_interval_seconds_ladder() {
        let mut cfg = AutoProfileSwitcherConfig::default();
        cfg.check_interval_seconds = 300;
        cfg.caution_interval_seconds = 180;
        cfg.critical_interval_seconds = 60;
        cfg.critical_threshold_percent = 12.0;

        // None quota defaults to check_interval_seconds
        assert_eq!(calculate_next_interval_seconds(None, &cfg), 300);

        // Healthy quota (>= 20%) uses standard check_interval_seconds
        assert_eq!(calculate_next_interval_seconds(Some(85.0), &cfg), 300);
        assert_eq!(calculate_next_interval_seconds(Some(20.0), &cfg), 300);

        // Caution quota (< 20% and > critical_threshold_percent) uses caution_interval_seconds
        assert_eq!(calculate_next_interval_seconds(Some(19.9), &cfg), 180);
        assert_eq!(calculate_next_interval_seconds(Some(13.0), &cfg), 180);

        // Critical quota (<= critical_threshold_percent) uses critical_interval_seconds
        assert_eq!(calculate_next_interval_seconds(Some(12.0), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(5.0), &cfg), 60);
        assert_eq!(calculate_next_interval_seconds(Some(0.0), &cfg), 60);
    }
}
