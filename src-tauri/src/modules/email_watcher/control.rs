use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use super::*;
use super::heartbeat::run_watcher_heartbeat_loop;

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
