//! 账号加载：批量加载、单账号重载、移除账号、磁盘状态探测。

use super::types::OnDiskAccountState;
use super::TokenManager;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

impl TokenManager {
    /// 从主应用账号目录加载所有账号
    pub async fn load_accounts(&self) -> Result<usize, String> {
        let accounts_dir = self.resolved_data_dir().join("accounts");

        if !accounts_dir.exists() {
            return Err(format!("账号目录不存在: {:?}", accounts_dir));
        }

        // Reload should reflect current on-disk state (accounts can be added/removed/disabled).
        self.tokens.clear();
        self.sync_image_scheduler_accounts();
        self.current_index.store(0, Ordering::SeqCst);
        {
            let mut last_used = self.last_used_account.lock().await;
            *last_used = None;
        }

        let entries =
            std::fs::read_dir(&accounts_dir).map_err(|e| format!("读取账号目录失败: {}", e))?;

        let mut count = 0;

        for entry in entries {
            let entry = entry.map_err(|e| {
                self.sync_image_scheduler_accounts();
                format!("读取目录项失败: {}", e)
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // 尝试加载账号
            match self.load_single_account(&path).await {
                Ok(Some(token)) => {
                    let account_id = token.account_id.clone();
                    self.tokens.insert(account_id, token);
                    count += 1;
                }
                Ok(None) => {
                    // 跳过无效账号
                }
                Err(e) => {
                    tracing::warn!("加载账号失败 {:?}: {}", path, e);
                }
            }
        }

        self.sync_image_scheduler_accounts();
        Ok(count)
    }

    /// 重新加载指定账号（用于配额更新后的实时同步）
    pub async fn reload_account(&self, account_id: &str) -> Result<(), String> {
        let path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));
        if !path.exists() {
            return Err(format!("账号文件不存在: {:?}", path));
        }

        match self.load_single_account(&path).await {
            Ok(Some(token)) => {
                self.tokens.insert(account_id.to_string(), token);
                self.sync_image_scheduler_accounts();
                Ok(())
            }
            Ok(None) => {
                // [FIX] 账号被禁用或不可用时，从内存池中彻底移除 (Issue #1565)
                // load_single_account returning None means the account should be skipped in its
                // current state (disabled / proxy_disabled / quota_protection / validation_blocked...).
                self.remove_account(account_id);
                Ok(())
            }
            Err(e) => Err(format!("同步账号失败: {}", e)),
        }
    }

    /// 重新加载所有账号
    pub async fn reload_all_accounts(&self) -> Result<usize, String> {
        self.load_accounts().await
    }

    /// 从内存中彻底移除指定账号及其关联数据 (Issue #1477)
    pub fn remove_account(&self, account_id: &str) {
        // ... (省略原有逻辑)
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
