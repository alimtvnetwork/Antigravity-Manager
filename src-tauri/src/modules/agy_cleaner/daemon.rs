use crate::modules::config;
use crate::modules::logger;

use super::*;

/// Start background conversation cleanup ticker (runs every N hours if enabled)
pub fn start_cleanup_daemon() {
    tauri::async_runtime::spawn(async move {
        logger::log_info("[AgyCleaner] Background cleanup daemon initialized.");
        loop {
            let app_config = config::load_app_config().unwrap_or_default();
            let cleanup_cfg = app_config.conversation_cleanup;

            let interval_hours = if cleanup_cfg.interval_hours == 0 {
                1
            } else {
                cleanup_cfg.interval_hours
            };

            tokio::time::sleep(std::time::Duration::from_secs(interval_hours as u64 * 3600)).await;

            let current_config = config::load_app_config().unwrap_or_default();
            if current_config.conversation_cleanup.is_enabled {
                let keep_count = current_config.conversation_cleanup.keep_count;
                logger::log_info(&format!(
                    "[AgyCleaner] Periodic auto-cleanup triggered (keeping top {} conversations)",
                    keep_count
                ));
                match prune_and_clean(keep_count) {
                    Ok(report) => {
                        logger::log_info(&format!(
                            "[AgyCleaner] Periodic cleanup finished: pruned {} conversations, freed {} bytes",
                            report.pruned_count, report.total_freed_bytes
                        ));
                    }
                    Err(e) => {
                        logger::log_warn(&format!("[AgyCleaner] Periodic cleanup error: {}", e));
                    }
                }
            }
        }
    });
}
