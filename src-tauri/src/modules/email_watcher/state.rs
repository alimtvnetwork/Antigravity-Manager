use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub is_running: bool,
    pub machine_name: String,
    pub machine_ip: String,
    pub last_telemetry_check: i64,
    pub last_inbox_check: i64,
    pub last_alert_sent: Option<i64>,
}

pub(crate) static WATCHER_RUNNING: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

pub(crate) static AWAITING_REPLY_DEADLINE: Lazy<Arc<AtomicI64>> =
    Lazy::new(|| Arc::new(AtomicI64::new(0)));

pub(crate) static LAST_STATUS: Lazy<Arc<Mutex<WatcherStatus>>> = Lazy::new(|| {
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
pub(crate) static LAST_QUOTA_ALERTED_PERCENT: Lazy<
    std::sync::Mutex<std::collections::HashMap<String, f64>>,
> = Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

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
