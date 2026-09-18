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
use std::sync::atomic::{AtomicBool, Ordering};
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

/// Detect local machine hostname
pub fn detect_machine_name() -> String {
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

/// Start the background watcher loop
pub fn start_email_watcher() {
    let is_already_running = WATCHER_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    crate::modules::logger::log_info(
        "[EmailWatcher] Starting background telemetry & mailbox watcher",
    );

    tokio::spawn(async move {
        let mut loop_tick: u32 = 0;
        let mut last_idle_alert: i64 = 0;
        let mut last_quota_alert: i64 = 0;

        while WATCHER_RUNNING.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(60)).await;
            loop_tick += 1;

            let settings = match email_vault_db::get_notification_settings() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let is_enabled = settings.is_enabled;
            if !is_enabled {
                continue;
            }

            let m_name = detect_machine_name();
            let m_ip = detect_local_ip();
            let now = Utc::now().timestamp();

            // Update status
            {
                let mut st = LAST_STATUS.lock().await;
                st.machine_name = m_name.clone();
                st.machine_ip = m_ip.clone();
                st.last_telemetry_check = now;
            }

            // 1. Inbound Inbox Poller (every inbox_check_interval_minutes)
            let in_interval = settings.inbox_check_interval_minutes.max(1);
            if loop_tick % in_interval == 0 {
                poll_inbox_cycle(&m_name, &m_ip).await;
                let mut st = LAST_STATUS.lock().await;
                st.last_inbox_check = now;
            }

            // 2. Telemetry Sensors (every polling_interval_minutes)
            let poll_interval = settings.polling_interval_minutes.max(1);
            if loop_tick % poll_interval == 0 {
                // Sensor A: Quota Drop Check
                if settings.notify_on_quota_drop {
                    let is_quota_check_due = now - last_quota_alert > 600; // 10 min cooldown
                    if is_quota_check_due {
                        check_quota_drop_sensor(&settings, &m_name, &m_ip, &mut last_quota_alert)
                            .await;
                    }
                }

                // Sensor B: Idle Running Projects Check
                if settings.notify_on_idle_workspace {
                    let is_idle_check_due = now - last_idle_alert > 600; // 10 min cooldown
                    if is_idle_check_due {
                        check_idle_projects_sensor(&m_name, &m_ip, &mut last_idle_alert).await;
                    }
                }
            }
        }

        crate::modules::logger::log_info("[EmailWatcher] Background watcher stopped");
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

    for acc in accounts {
        if let Some(quota) = acc.quota {
            let threshold = settings.quota_drop_threshold_percent as f64;
            for m in quota.models {
                let pct = m.percentage as f64;
                if pct < threshold && pct > 0.0 {
                    let (subj, html) = email_sender::render_quota_drop_email(
                        &acc.email,
                        pct,
                        settings.quota_drop_threshold_percent,
                        m_name,
                        m_ip,
                    );
                    let _ = email_sender::dispatch_email_with_failover(
                        &subj,
                        &html,
                        &active_recipients,
                    );
                    *last_alert = Utc::now().timestamp();
                    return;
                }
            }
        }
    }
}

/// Check idle running projects sensor
async fn check_idle_projects_sensor(m_name: &str, m_ip: &str, last_alert: &mut i64) {
    let projects = match crate::modules::repo_db::list_running_projects() {
        Ok(p) => p,
        Err(_) => return,
    };

    let running_projs: Vec<_> = projects.into_iter().filter(|p| p.is_running).collect();
    if running_projs.is_empty() {
        return;
    }

    // Check active prompts
    let active_prompts = match crate::modules::repo_db::list_backed_up_prompts() {
        Ok(p) => p,
        Err(_) => return,
    };

    let has_prompts = !active_prompts.is_empty();
    if !has_prompts {
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
            let names: Vec<String> = running_projs.into_iter().map(|p| p.repo_name).collect();
            let (subj, html) = email_sender::render_idle_projects_email(&names, m_name, m_ip);
            let _ = email_sender::dispatch_email_with_failover(&subj, &html, &active_recipients);
            *last_alert = Utc::now().timestamp();
        }
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
