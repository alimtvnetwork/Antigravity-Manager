//! Session and scheduling configuration: sticky session binding, scheduling mode, preferred account.

use super::TokenManager;
use crate::proxy::sticky_config::SchedulingMode;
use crate::proxy::sticky_config::StickySessionConfig;

impl TokenManager {
    /// Get the current scheduling configuration
    pub async fn get_sticky_config(&self) -> StickySessionConfig {
        self.sticky_config.read().await.clone()
    }

    /// Update scheduling configuration
    pub async fn update_sticky_config(&self, new_config: StickySessionConfig) {
        let mut config = self.sticky_config.write().await;
        *config = new_config;
        tracing::debug!("Scheduling configuration updated: {:?}", *config);
    }

    /// [NEW] Update fuse configuration
    pub async fn update_circuit_breaker_config(&self, config: crate::models::CircuitBreakerConfig) {
        let mut lock = self.circuit_breaker_config.write().await;
        *lock = config;
        tracing::debug!("Circuit breaker configuration updated");
    }

    /// [NEW] Get fuse configuration
    pub async fn get_circuit_breaker_config(&self) -> crate::models::CircuitBreakerConfig {
        self.circuit_breaker_config.read().await.clone()
    }

    /// Clear sticky mapping for a specific session
    #[allow(dead_code)]
    pub fn clear_session_binding(&self, session_id: &str) {
        self.session_accounts.remove(session_id);
    }

    /// [FIX] When encountering 429/529 or other current throttling or overload, unbind the session and clear the recent usage records to break the sticky deadlock.
    pub async fn unbind_session_and_clear_last_used(&self, session_id: Option<&str>) {
        if let Some(sid) = session_id {
            self.session_accounts.remove(sid);
        }
        let mut last_used = self.last_used_account.lock().await;
        *last_used = None;
    }

    /// Get the currently effective load balancing scheduling mode (safely read from memory without touching the disk)
    pub async fn get_scheduling_mode(&self) -> crate::proxy::sticky_config::SchedulingMode {
        self.sticky_config.read().await.mode
    }

    /// Clear sticky maps for all sessions
    pub fn clear_all_sessions(&self) {
        self.session_accounts.clear();
    }

    // ===== [FIX #820] Fixed account mode related methods =====

    /// Set the account ID to be used first (fixed account mode)
    /// Pass in Some(account_id) to enable fixed account mode, pass in None to restore polling mode
    pub async fn set_preferred_account(&self, account_id: Option<String>) {
        let mut preferred = self.preferred_account_id.write().await;
        if let Some(ref id) = account_id {
            tracing::info!("🔒 [FIX #820] Fixed account mode enabled: {}", id);
        } else {
            tracing::info!("🔄 [FIX #820] Round-robin mode enabled (no preferred account)");
        }
        *preferred = account_id;
    }

    /// Get the currently preferred account ID
    pub async fn get_preferred_account(&self) -> Option<String> {
        self.preferred_account_id.read().await.clone()
    }
}
