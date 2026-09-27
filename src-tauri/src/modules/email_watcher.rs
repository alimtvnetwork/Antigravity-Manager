//! Background Email Watcher Daemon Module
//! Captures machine telemetry, monitors low quota, detects idle running projects,
//! and polls inbound mailbox for remote execution commands.

#![allow(dead_code)]

use crate::modules::email_inbound;
use crate::modules::email_sender;
use crate::modules::email_vault_db;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Runtime status of the background email watcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub is_running: bool,
    pub machine_name: String,
    pub machine_ip: String,
    pub last_telemetry_check: i64,
    pub last_inbox_check: i64,
    pub last_alert_sent: Option<i64>,
}

static WATCHER_RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static AWAITING_REPLY_DEADLINE: Lazy<Arc<AtomicI64>> = Lazy::new(|| Arc::new(AtomicI64::new(0)));
static LAST_STATUS: Lazy<Arc<Mutex<WatcherStatus>>> = Lazy::new(|| {
    Arc::new(Mutex::new(WatcherStatus {
        is_running: false,
        machine_name: String::new(),
        machine_ip: String::new(),
        last_telemetry_check: 0,
        last_inbox_check: 0,
        last_alert_sent: None,
    }))
});

/// Tracks the last alerted quota percentage per account email to avoid spamming identical alerts
static LAST_QUOTA_ALERTED_PERCENT: Lazy<std::sync::Mutex<std::collections::HashMap<String, f64>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Activate fast adaptive polling window (in seconds)
pub fn activate_awaiting_reply(duration_seconds: i64) {
    let deadline = Utc::now().timestamp() + duration_seconds;
    AWAITING_REPLY_DEADLINE.store(deadline, Ordering::SeqCst);
    crate::modules::logger::log_info(&format!(
        "[EmailWatcher] Fast adaptive polling activated for {}s (deadline: {})",
        duration_seconds, deadline
    ));
}

/// Check if current timestamp is within awaiting reply deadline
pub fn is_awaiting_reply() -> bool {
    let deadline = AWAITING_REPLY_DEADLINE.load(Ordering::SeqCst);
    let now = Utc::now().timestamp();
    deadline > now
}

/// Detect local machine network IP address
pub fn detect_local_ip() -> String {
    // Non-blocking trick: bind UDP socket and connect to public DNS to inspect routing
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Detect local machine hostname or saved custom node name
pub fn detect_machine_name() -> String {
    if let Ok(settings) = email_vault_db::get_notification_settings() {
        if !settings.local_machine_name.trim().is_empty() {
            return settings.local_machine_name.trim().to_string();
        }
    }
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        if !name.trim().is_empty() {
            return name.trim().to_string();
        }
    }
    if let Ok(name) = std::env::var("HOSTNAME") {
        if !name.trim().is_empty() {
            return name.trim().to_string();
        }
    }
    "antigravity-node".to_string()
}

/// Get current watcher status
pub async fn get_watcher_status() -> WatcherStatus {
    let mut status = LAST_STATUS.lock().await.clone();
    status.is_running = WATCHER_RUNNING.load(Ordering::SeqCst);
    if status.machine_name.is_empty() {
        status.machine_name = detect_machine_name();
        status.machine_ip = detect_local_ip();
    }
    status
}

fn determine_inbox_interval(settings: &email_vault_db::EmailNotificationSettings) -> i64 {
    let is_awaiting = is_awaiting_reply();
    if is_awaiting {
        return settings.active_awaiting_interval_seconds.clamp(5, 30) as i64;
    }
    let minutes = settings.baseline_polling_interval_minutes.clamp(1, 15);
    (minutes * 60) as i64
}

async fn update_status_telemetry(m_name: String, m_ip: String, now: i64) {
    let mut st = LAST_STATUS.lock().await;
    st.machine_name = m_name;
    st.machine_ip = m_ip;
    st.last_telemetry_check = now;
}

async fn update_status_inbox(now: i64) {
    let mut st = LAST_STATUS.lock().await;
    st.last_inbox_check = now;
}

async fn check_telemetry_sensors(
    settings: &email_vault_db::EmailNotificationSettings,
    m_name: &str,
    m_ip: &str,
    last_quota: &mut i64,
    last_idle: &mut i64,
    now: i64,
) {
    if settings.notify_on_quota_drop {
        let is_cooldown_ready = now - *last_quota > 600;
        if is_cooldown_ready {
            check_quota_drop_sensor(settings, m_name, m_ip, last_quota).await;
        }
    }
    if settings.notify_on_idle_workspace {
        let is_cooldown_ready = now - *last_idle > 600;
        if is_cooldown_ready {
            check_idle_projects_sensor(m_name, m_ip, last_idle).await;
        }
    }
}

async fn execute_heartbeat_tick(
    settings: &email_vault_db::EmailNotificationSettings,
    last_inbox: &mut i64,
    last_telemetry: &mut i64,
    last_quota: &mut i64,
    last_idle: &mut i64,
) {
    let m_name = detect_machine_name();
    let m_ip = detect_local_ip();
    let now = Utc::now().timestamp();

    let inbox_interval = determine_inbox_interval(settings);
    let is_inbox_due = now - *last_inbox >= inbox_interval;
    if is_inbox_due {
        *last_inbox = now;
        poll_inbox_cycle(&m_name, &m_ip).await;
        update_status_inbox(now).await;
    }

    let telemetry_interval = (settings.polling_interval_minutes.max(1) * 60) as i64;
    let is_telemetry_due = now - *last_telemetry >= telemetry_interval;
    if is_telemetry_due {
        *last_telemetry = now;
        update_status_telemetry(m_name.clone(), m_ip.clone(), now).await;
        check_telemetry_sensors(settings, &m_name, &m_ip, last_quota, last_idle, now).await;
    }
}

async fn run_watcher_heartbeat_loop() {
    let mut last_inbox: i64 = 0;
    let mut last_telemetry: i64 = 0;
    let mut last_quota: i64 = 0;
    let mut last_idle: i64 = 0;

    while WATCHER_RUNNING.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let settings = match email_vault_db::get_notification_settings() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let is_enabled = settings.is_enabled;
        if !is_enabled {
            continue;
        }
        execute_heartbeat_tick(
            &settings,
            &mut last_inbox,
            &mut last_telemetry,
            &mut last_quota,
            &mut last_idle,
        )
        .await;
    }
    crate::modules::logger::log_info("[EmailWatcher] Background watcher stopped");
}

/// Start the background watcher loop
pub fn start_email_watcher() {
    let is_already_running = WATCHER_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    crate::modules::logger::log_info(
        "[EmailWatcher] Starting background telemetry & mailbox watcher",
    );

    tauri::async_runtime::spawn(async move {
        run_watcher_heartbeat_loop().await;
    });
}

/// Stop the background watcher loop
pub fn stop_email_watcher() {
    WATCHER_RUNNING.store(false, Ordering::SeqCst);
    crate::modules::logger::log_info("[EmailWatcher] Stopping background email watcher");
}

/// Run a single cycle of inbox checking
async fn poll_inbox_cycle(m_name: &str, m_ip: &str) {
    let accounts = match email_vault_db::list_email_accounts() {
        Ok(a) => a,
        Err(_) => return,
    };

    let active_account = accounts.into_iter().find(|a| a.is_default && a.is_active);
    let account = match active_account {
        Some(a) => a,
        None => return,
    };

    // Spawn blocking for socket IMAP calls
    let acc_clone = account.clone();
    let unread =
        tokio::task::spawn_blocking(move || email_inbound::poll_unread_messages(&acc_clone, 5))
            .await
            .unwrap_or_else(|_| Ok(Vec::new()));

    if let Ok(messages) = unread {
        for msg in messages {
            if let Ok(seen) = email_vault_db::is_message_already_processed(&msg.message_id) {
                if seen {
                    continue;
                }
            }

            let action = email_inbound::parse_email_command(&msg.subject, &msg.body);
            let _ = email_inbound::execute_inbound_action(&msg, action, m_ip, m_name);
        }
    }
}

/// Check quota drop sensor
async fn check_quota_drop_sensor(
    settings: &email_vault_db::EmailNotificationSettings,
    m_name: &str,
    m_ip: &str,
    last_alert: &mut i64,
) {
    // Only check if Antigravity IDE, isolated instances, active projects, or proxy services are currently running
    if !crate::modules::auto_switcher::is_antigravity_or_instance_running(None) {
        return;
    }

    let accounts = match crate::modules::account::list_accounts() {
        Ok(accs) => accs,
        Err(_) => return,
    };

    let recipients = match email_vault_db::list_notify_recipients() {
        Ok(r) => r,
        Err(_) => return,
    };

    let active_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if active_recipients.is_empty() {
        return;
    }

    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let threshold = if settings.quota_drop_threshold_percent > 0 {
        settings.quota_drop_threshold_percent as f64
    } else {
        app_cfg.auto_profile_switcher.low_quota_threshold_percent
    };

    if threshold <= 0.0 {
        return;
    }

    for acc in accounts {
        // Enforce: ONLY send low credit alert if we are currently using this account!
        if !crate::modules::auto_switcher::is_account_in_use(&acc) {
            continue;
        }

        if let Some(quota) = acc.quota {
            let mut lowest_model_opt: Option<(String, f64)> = None;

            for m in quota.models {
                let name_lower = m.name.to_lowercase();
                let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
                if is_banned {
                    continue;
                }
                let pct = m.percentage as f64;
                if pct > 0.0 {
                    match lowest_model_opt {
                        Some((_, cur_min)) if pct < cur_min => {
                            lowest_model_opt = Some((m.name.clone(), pct));
                        }
                        None => {
                            lowest_model_opt = Some((m.name.clone(), pct));
                        }
                        _ => {}
                    }
                }
            }

            if let Some((_model_name, pct)) = lowest_model_opt {
                let acc_key = acc.email.trim().to_lowercase();
                if pct < threshold {
                    // Check deduplication: only alert if not alerted yet or if dropped by >= 1.0% further
                    let should_alert = {
                        let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
                        match alerts.get(&acc_key) {
                            Some(&last_pct) => {
                                if (last_pct - pct) >= 1.0 {
                                    alerts.insert(acc_key.clone(), pct);
                                    true
                                } else {
                                    false
                                }
                            }
                            None => {
                                alerts.insert(acc_key.clone(), pct);
                                true
                            }
                        }
                    };

                    if should_alert {
                        let (subj, html) = email_sender::render_quota_drop_email(
                            &acc.email,
                            pct,
                            threshold as u32,
                            m_name,
                            m_ip,
                        );
                        let _ = email_sender::dispatch_email_with_failover(
                            &subj,
                            &html,
                            &active_recipients,
                        );
                        *last_alert = Utc::now().timestamp();
                        let mut st = LAST_STATUS.lock().await;
                        st.last_alert_sent = Some(*last_alert);
                        return;
                    }
                } else {
                    // Quota is healthy/reset above threshold, clear from alert cache
                    if let Ok(mut alerts) = LAST_QUOTA_ALERTED_PERCENT.lock() {
                        alerts.remove(&acc_key);
                    }
                }
            }
        }
    }
}

/// Check idle running projects sensor
async fn check_idle_projects_sensor(m_name: &str, m_ip: &str, last_alert: &mut i64) {
    // 1. If ANY prompt or conversation is actively running in Antigravity or repo_db, we are NOT idle!
    if crate::modules::repo_db::is_any_prompt_actively_running() {
        return;
    }

    // 2. Query live execution states
    let project_infos = crate::modules::repo_db::get_live_project_execution_info();
    if project_infos.is_empty() {
        return;
    }

    // 3. If any project is actively running, suppress idle alert completely
    let has_any_running = project_infos.iter().any(|p| p.is_running);
    if has_any_running {
        return;
    }

    // 4. Collect genuinely idle projects
    let idle_projects: Vec<_> = project_infos.into_iter().filter(|p| p.is_idle).collect();
    if idle_projects.is_empty() {
        return;
    }

    let recipients = match email_vault_db::list_notify_recipients() {
        Ok(r) => r,
        Err(_) => return,
    };

    let active_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if !active_recipients.is_empty() {
        let (subj, html) = email_sender::render_idle_projects_email(&idle_projects, m_name, m_ip);
        let _ = email_sender::dispatch_email_with_failover(&subj, &html, &active_recipients);
        *last_alert = Utc::now().timestamp();
    }
}

/// Send workspace switched notification immediately
pub fn notify_workspace_switched(from_instance: &str, to_instance: &str, reason: &str) {
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(_) => return,
    };

    let is_enabled = settings.is_enabled;
    if !is_enabled {
        return;
    }

    let is_switch_notify_enabled = settings.notify_on_workspace_switch;
    if !is_switch_notify_enabled {
        return;
    }

    let recipients = match email_vault_db::list_notify_recipients() {
        Ok(r) => r,
        Err(_) => return,
    };

    let active_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if active_recipients.is_empty() {
        return;
    }

    let m_name = detect_machine_name();
    let m_ip = detect_local_ip();
    let (subj, html) = email_sender::render_workspace_switch_email(
        from_instance,
        to_instance,
        reason,
        &m_name,
        &m_ip,
    );

    let _ = email_sender::dispatch_email_with_failover(&subj, &html, &active_recipients);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_drop_deduplication_tracking() {
        let test_email = "test-sensor-dedup@example.com".to_string();
        {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            alerts.remove(&test_email);
        }

        // First alert at 18%
        let should_alert_1 = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => (last_pct - 18.0) >= 1.0,
                None => {
                    alerts.insert(test_email.clone(), 18.0);
                    true
                }
            }
        };
        assert!(should_alert_1, "First low quota occurrence should alert");

        // Duplicate check at same 18%
        let should_alert_duplicate = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => (last_pct - 18.0) >= 1.0,
                None => {
                    alerts.insert(test_email.clone(), 18.0);
                    true
                }
            }
        };
        assert!(
            !should_alert_duplicate,
            "Identical quota percentage must be suppressed"
        );

        // Further drop to 15% (>= 1.0% drop)
        let should_alert_further_drop = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => {
                    if (last_pct - 15.0) >= 1.0 {
                        alerts.insert(test_email.clone(), 15.0);
                        true
                    } else {
                        false
                    }
                }
                None => true,
            }
        };
        assert!(
            should_alert_further_drop,
            "Significant drop should re-alert"
        );

        // Recovery above threshold (e.g. 100%)
        {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            alerts.remove(&test_email);
        }
        let is_cleared = {
            let alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            !alerts.contains_key(&test_email)
        };
        assert!(is_cleared, "Recovery should clear cache");
    }

    #[test]
    fn test_detect_machine_and_ip() {
        let ip = detect_local_ip();
        assert!(!ip.is_empty());
        let name = detect_machine_name();
        assert!(!name.is_empty());
    }
}
