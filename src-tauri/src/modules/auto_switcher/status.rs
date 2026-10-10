use crate::modules::{account, config, instance, logger};
use std::time::Duration;

use super::*;

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

/// Get high-precision daemon runtime status, including next check countdown and current stage
pub fn get_daemon_status() -> AutoSwitcherDaemonStatus {
    let status = get_status();
    let now = chrono::Utc::now().timestamp();
    let state = RUNTIME_STATE.lock().unwrap();

    let next_in_secs = (state.next_check_timestamp - now).max(0);
    AutoSwitcherDaemonStatus {
        is_daemon_running: state.is_running,
        last_evaluated_at: state.last_check_timestamp,
        next_check_timestamp: state.next_check_timestamp,
        next_check_in_seconds: next_in_secs,
        check_interval_seconds: state.check_interval_seconds,
        current_stage: state.current_stage.clone(),
        active_account_email: status.active_account_email,
        current_quota_percent: status.current_quota_percent.unwrap_or(100.0),
        monitored_instance_count: status.monitored_instance_count,
    }
}

/// Emit daemon status tick event to frontend
pub fn emit_daemon_status() {
    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        let daemon_status = get_daemon_status();
        // Justification: best-effort frontend event; a dropped event only skips a UI refresh
        crate::error::record_ignored(
            handle.emit("auto-switcher://status-tick", &daemon_status),
            "emit auto-switcher://status-tick",
        );
        // Justification: best-effort frontend event; a dropped event only skips a UI refresh
        crate::error::record_ignored(
            handle.emit("auto-switcher://daemon-status", &daemon_status),
            "emit auto-switcher://daemon-status",
        );
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

    let is_running =
        instance::is_instance_running(&active_inst.id, &active_inst.data_dir, active_inst.pid);

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
        let now = chrono::Utc::now().timestamp();
        {
            let mut state = RUNTIME_STATE.lock().unwrap();
            state.is_running = true;
            state.next_check_timestamp = now + 3;
            state.check_interval_seconds = 120;
            state.current_stage = "normal".to_string();
        }
        emit_daemon_status();

        let initial_cfg = config::load_app_config()
            .unwrap_or_default()
            .auto_profile_switcher;
        let mut last_enabled = initial_cfg.is_enabled;
        let mut last_threshold = initial_cfg.low_quota_threshold_percent;

        // Brief 3-second startup stabilization period: evaluate active instances without multi-minute delay
        tokio::time::sleep(Duration::from_secs(3)).await;

        if last_enabled {
            if let Err(e) = evaluate_and_execute_startup_rotation().await {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Error during startup rotation check: {}",
                    e
                ));
            }
        }
        emit_daemon_status();

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
            let stage = determine_quota_stage(lowest_monitored_quota, &switcher_cfg).to_string();
            let interval_secs =
                calculate_next_interval_seconds(lowest_monitored_quota, &switcher_cfg) as u64;

            let now = chrono::Utc::now().timestamp();
            let next_check_timestamp = now + interval_secs as i64;
            {
                let mut state = RUNTIME_STATE.lock().unwrap();
                state.next_check_timestamp = next_check_timestamp;
                state.check_interval_seconds = interval_secs as u32;
                state.current_stage = stage;
            }
            emit_daemon_status();

            let tick = 5u64;
            let mut elapsed = 0u64;
            while elapsed < interval_secs {
                let sleep_dur = tick.min(interval_secs.saturating_sub(elapsed));
                if sleep_dur == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(sleep_dur)).await;
                elapsed += sleep_dur;
                emit_daemon_status();

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

            // Justification: non-Result return value intentionally discarded — no error channel to track
            let _ = instance::refresh_pid_cache_if_due(switcher_cfg.pid_refresh_seconds);

            if let Err(e) = check_and_rotate_if_needed().await {
                logger::log_warn(&format!("[AutoSwitcher] Error during check cycle: {}", e));
            }
            emit_daemon_status();
        }
    });

    // Spawn dedicated 2-minute IDE Crash Recovery & Focus Watchdog
    tauri::async_runtime::spawn(async move {
        logger::log_info("[CrashWatchdog] 2-Minute IDE crash recovery & focus watchdog started.");
        tokio::time::sleep(Duration::from_secs(60)).await;
        loop {
            let app_config = config::load_app_config().unwrap_or_default();
            let interval = app_config
                .auto_profile_switcher
                .watchdog_interval_seconds
                .max(60);
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
