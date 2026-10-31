//! Account loading: batch loading, single account reloading, account removal, disk status detection.

use super::types::OnDiskAccountState;
use super::TokenManager;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

impl TokenManager {
    /// Load all accounts from the main application account directory
    pub async fn load_accounts(&self) -> Result<usize, String> {
        let accounts_dir = self.resolved_data_dir().join("accounts");

        if !accounts_dir.exists() {
            return Err(format!(
                "Account directory does not exist: {:?}",
                accounts_dir
            ));
        }

        // Reload should reflect current on-disk state (accounts can be added/removed/disabled).
        self.tokens.clear();
        self.sync_image_scheduler_accounts();
        self.current_index.store(0, Ordering::SeqCst);
        {
            let mut last_used = self.last_used_account.lock().await;
            *last_used = None;
        }

        let entries = std::fs::read_dir(&accounts_dir)
            .map_err(|e| format!("Failed to read account directory: {}", e))?;

        let mut count = 0;

        for entry in entries {
            let entry = entry.map_err(|e| {
                self.sync_image_scheduler_accounts();
                format!("Failed to read directory entry: {}", e)
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try loading account
            match self.load_single_account(&path).await {
                Ok(Some(token)) => {
                    let account_id = token.account_id.clone();
                    self.tokens.insert(account_id, token);
                    count += 1;
                }
                Ok(None) => {
                    // Skip invalid accounts
                }
                Err(e) => {
                    tracing::warn!("Failed to load account {:?} : {}", path, e);
                }
            }
        }

        self.sync_image_scheduler_accounts();
        Ok(count)
    }

    /// Reload the specified account (for real-time synchronization after quota update)
    pub async fn reload_account(&self, account_id: &str) -> Result<(), String> {
        let path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));
        if !path.exists() {
            return Err(format!("Account file does not exist: {:?}", path));
        }

        match self.load_single_account(&path).await {
            Ok(Some(token)) => {
                self.tokens.insert(account_id.to_string(), token);
                self.sync_image_scheduler_accounts();
                Ok(())
            }
            Ok(None) => {
                // [FIX] Completely remove account from memory pool when disabled or unavailable (Issue #1565)
                // load_single_account returning None means the account should be skipped in its
                // current state (disabled / proxy_disabled / quota_protection / validation_blocked...).
                self.remove_account(account_id);
                Ok(())
            }
            Err(e) => Err(format!("Account synchronization failed: {}", e)),
        }
    }

    /// Reload all accounts
    pub async fn reload_all_accounts(&self) -> Result<usize, String> {
        self.load_accounts().await
    }

    /// Completely remove the specified account and its associated data from memory (Issue #1477)
    pub fn remove_account(&self, account_id: &str) {
        // ... (omitting original logic)
        if self.tokens.remove(account_id).is_some() {
            tracing::info!("[Proxy] Removed account {} from memory cache", account_id);
        }
        self.health_scores.remove(account_id);
        self.rate_limit_tracker.clear(account_id);
        self.session_accounts.retain(|_, v| v != account_id);
        if let Ok(mut preferred) = self.preferred_account_id.try_write() {
            if preferred.as_deref() == Some(account_id) {
                *preferred = None;
                tracing::info!(
                    "[Proxy] Cleared preferred account status for {}",
                    account_id
                );
            }
        }
        self.sync_image_scheduler_accounts();
    }

    /// Check if an account has been disabled on disk.
    ///
    /// Safety net: avoids selecting a disabled account when the in-memory pool hasn't been
    /// reloaded yet (e.g. fixed account mode / sticky session).
    ///
    /// Note: this is intentionally tolerant to transient read/parse failures (e.g. concurrent
    /// writes). Failures are reported as `Unknown` so callers can skip without purging the in-memory
    /// token pool.
    pub(crate) async fn get_account_state_on_disk(
        account_path: &std::path::PathBuf,
    ) -> OnDiskAccountState {
        const MAX_RETRIES: usize = 2;
        const RETRY_DELAY_MS: u64 = 5;

        for attempt in 0..=MAX_RETRIES {
            let content = match tokio::fs::read_to_string(account_path).await {
                Ok(c) => c,
                Err(e) => {
                    // If the file is gone, the in-memory token is definitely stale.
                    if e.kind() == std::io::ErrorKind::NotFound {
                        return OnDiskAccountState::Disabled;
                    }
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                        continue;
                    }
                    tracing::debug!(
                        "Failed to read account file on disk {:?}: {}",
                        account_path,
                        e
                    );
                    return OnDiskAccountState::Unknown;
                }
            };

            let account = match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(v) => v,
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                        continue;
                    }
                    tracing::debug!(
                        "Failed to parse account JSON on disk {:?}: {}",
                        account_path,
                        e
                    );
                    return OnDiskAccountState::Unknown;
                }
            };

            let disabled = account
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || account
                    .get("proxy_disabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                || account
                    .get("quota")
                    .and_then(|q| q.get("is_forbidden"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

            return if disabled {
                OnDiskAccountState::Disabled
            } else {
                OnDiskAccountState::Enabled
            };
        }

        OnDiskAccountState::Unknown
    }
}
