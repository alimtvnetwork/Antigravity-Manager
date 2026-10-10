//! 限流状态查询与清理。

use super::TokenManager;

impl TokenManager {
    /// 标记账号限流(从外部调用,通常在 handler 中)
    /// 参数为 email，内部会自动转换为 account_id
    pub async fn mark_rate_limited(
        &self,
        email: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
    ) {
        // [NEW] 检查熔断是否启用 (使用内存缓存，极快)
        let config = self.circuit_breaker_config.read().await.clone();
        if !config.enabled {
            return;
        }

        // 【替代方案】转换 email -> account_id
        let key = self
            .email_to_account_id(email)
            .unwrap_or_else(|| email.to_string());

        self.rate_limit_tracker.parse_from_error(
            &key,
            status,
            retry_after_header,
            error_body,
            None,
            &config.backoff_steps, // [NEW] 传入配置
        );
    }

    /// 检查账号是否在限流中 (支持模型级)
    pub async fn is_rate_limited(&self, account_id: &str, model: Option<&str>) -> bool {
        // [NEW] 检查熔断是否启用
        let config = self.circuit_breaker_config.read().await;
        if !config.enabled {
            return self
                .rate_limit_tracker
                .get_quota_wait(account_id, model, true)
                > 0;
        }
        self.rate_limit_tracker.is_rate_limited(account_id, model)
    }

    /// 获取距离限流重置还有多少秒
    #[allow(dead_code)]
    pub fn get_rate_limit_reset_seconds(&self, account_id: &str) -> Option<u64> {
        self.rate_limit_tracker.get_reset_seconds(account_id)
    }

    /// 清除过期的限流记录
    #[allow(dead_code)]
    pub fn clean_expired_rate_limits(&self) {
        self.rate_limit_tracker.cleanup_expired();
    }

    /// 【替代方案】通过 email 查找对应的 account_id
    /// 用于将 handlers 传入的 email 转换为 tracker 使用的 account_id
    pub(crate) fn email_to_account_id(&self, email: &str) -> Option<String> {
        self.tokens
            .iter()
            .find(|entry| entry.value().email == email)
            .map(|entry| entry.value().account_id.clone())
    }

    /// 清除指定账号的限流记录
    pub fn clear_rate_limit(&self, account_id: &str) -> bool {
        let cleared = self.rate_limit_tracker.clear(account_id);
        let persisted_cleared = self.clear_all_persisted_live_limits(account_id);
        cleared || persisted_cleared
    }

    pub fn clear_rate_limit_memory(&self, account_id: &str) -> bool {
        self.rate_limit_tracker.clear(account_id)
    }

    /// 清除所有限流记录
    pub fn clear_all_rate_limits(&self) {
        self.rate_limit_tracker.clear_all();
        let accounts_dir = self.resolved_data_dir().join("accounts");
        if let Ok(entries) = std::fs::read_dir(accounts_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
                    if let Some(account_id) =
                        entry.path().file_stem().and_then(|value| value.to_str())
                    {
                        self.clear_all_persisted_live_limits(account_id);
                    }
                }
            }
        }
    }

    fn clear_all_persisted_live_limits(&self, account_id: &str) -> bool {
        let path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));
        let Ok(_account_write) = crate::modules::account::lock_account_file_updates() else {
            return false;
        };
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return false;
        };
        let Ok(mut content) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return false;
        };
        let Some(live_limits) = content
            .get_mut("live_limited_models")
            .and_then(|value| value.as_object_mut())
        else {
            return false;
        };
        if live_limits.is_empty() {
            return false;
        }
        live_limits.clear();
        let Ok(serialized) = serde_json::to_string_pretty(&content) else {
            return false;
        };
        std::fs::write(path, serialized).is_ok()
    }

    pub fn clear_persisted_live_limit(&self, account_id: &str, model: Option<&str>) {
        let Some(raw_model) = model.filter(|m| !m.is_empty()) else {
            return;
        };
        let model_key = crate::proxy::common::model_mapping::normalize_to_standard_id(raw_model)
            .unwrap_or_else(|| raw_model.to_string());

        let path = if let Some(entry) = self.tokens.get(account_id) {
            entry.account_path.clone()
        } else {
            self.resolved_data_dir()
                .join("accounts")
                .join(format!("{}.json", account_id))
        };

        let Ok(_account_write) = crate::modules::account::lock_account_file_updates() else {
            return;
        };

        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.rate_limit_tracker.clear_model(account_id, &model_key);
                return;
            }
            Err(error) => {
                tracing::debug!("Failed to read live limit for {}: {}", account_id, error);
                return;
            }
        };
        let Ok(mut content) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return;
        };

        let Some(live_limits) = content
            .get_mut("live_limited_models")
            .and_then(|value| value.as_object_mut())
        else {
            self.rate_limit_tracker.clear_model(account_id, &model_key);
            return;
        };

        let mut changed = live_limits.remove(&model_key).is_some();
        if model_key != raw_model {
            changed |= live_limits.remove(raw_model).is_some();
        }

        if changed {
            let Ok(serialized) = serde_json::to_string_pretty(&content) else {
                return;
            };
            if let Err(error) = std::fs::write(&path, serialized) {
                tracing::debug!("Failed to clear live limit for {}: {}", account_id, error);
                return;
            }
        }
        self.rate_limit_tracker.clear_model(account_id, &model_key);
    }

    // ===== 调度配置相关方法 =====
}
