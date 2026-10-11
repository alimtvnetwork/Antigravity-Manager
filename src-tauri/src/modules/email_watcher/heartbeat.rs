use crate::modules::email_vault_db;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::Duration;

use super::sensors::check_idle_projects_sensor;
use super::sensors::check_quota_drop_sensor;
use super::sensors::poll_inbox_cycle;
use super::*;

pub(crate) fn determine_inbox_interval(
    settings: &email_vault_db::EmailNotificationSettings,
) -> i64 {
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

    let telemetry_interval = (settings.polling_interval_minutes.max(5) * 60) as i64;
    let is_telemetry_due = now - *last_telemetry >= telemetry_interval;
    if is_telemetry_due {
        *last_telemetry = now;
        update_status_telemetry(m_name.clone(), m_ip.clone(), now).await;
        check_telemetry_sensors(settings, &m_name, &m_ip, last_quota, last_idle, now).await;
    }
}

pub(crate) async fn run_watcher_heartbeat_loop() {
    // Enforce 60-second startup quiet period: nothing runs until 1 minute after launch
    tokio::time::sleep(Duration::from_secs(60)).await;

    let init_now = Utc::now().timestamp();
    let mut last_inbox: i64 = init_now;
    let mut last_telemetry: i64 = init_now;
    let mut last_quota: i64 = init_now;
    let mut last_idle: i64 = init_now;

    while WATCHER_RUNNING.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(30)).await;
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
