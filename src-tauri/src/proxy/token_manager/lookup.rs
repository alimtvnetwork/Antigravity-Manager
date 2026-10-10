//! 账号查询：按 ID / 邮箱 / 模型可用性查找。

use super::ProxyToken;
use super::TokenManager;
use std::collections::HashSet;

impl TokenManager {
    /// 根据账号 ID 获取完整的 ProxyToken 对象 (v4.1.29)
    pub fn get_token_by_id(&self, account_id: &str) -> Option<ProxyToken> {
        self.tokens.get(account_id).map(|t| t.clone())
    }

    /// 通过 email 获取指定账号的 Token（用于预热等需要指定账号的场景）
    /// 此方法会自动刷新过期的 token
    pub async fn get_token_by_email(
        &self,
        email: &str,
    ) -> Result<(String, String, String, String, u64), String> {
        // 查找账号信息
        let token_info = {
            let mut found = None;
            for entry in self.tokens.iter() {
                let token = entry.value();
                if token.email == email {
                    found = Some((
                        token.account_id.clone(),
                        token.access_token.clone(),
                        token.refresh_token.clone(),
                        token.timestamp,
                        token.expires_in,
                        chrono::Utc::now().timestamp(),
                        token.project_id.clone(),
                    ));
                    break;
                }
            }
            found
        };

        let (
            account_id,
            current_access_token,
            refresh_token,
            timestamp,
            expires_in,
            now,
            project_id_opt,
        ) = match token_info {
            Some(info) => info,
            None => return Err(format!("未找到账号: {}", email)),
        };

        let project_id = project_id_opt
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "bamboo-precept-lgxtn".to_string());

        // 检查是否过期 (提前5分钟)
        if now < timestamp + expires_in - 300 {
            return Ok((
                current_access_token,
                project_id,
                email.to_string(),
                account_id,
                0,
            ));
        }

        tracing::info!("[Warmup] Token for {} is expiring, refreshing...", email);

        // 调用 OAuth 刷新 token
        match crate::modules::oauth::refresh_access_token(&refresh_token, Some(&account_id)).await {
            Ok(token_response) => {
                tracing::info!("[Warmup] Token refresh successful for {}", email);
                let new_now = chrono::Utc::now().timestamp();

                // 更新缓存
                if let Some(mut entry) = self.tokens.get_mut(&account_id) {
                    entry.access_token = token_response.access_token.clone();
                    entry.expires_in = token_response.expires_in;
                    entry.timestamp = new_now;
                }

                // 保存到磁盘
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    self.save_refreshed_token(&account_id, &token_response)
                        .await,
                    "operation",
                );

                Ok((
                    token_response.access_token,
                    project_id,
                    email.to_string(),
                    account_id,
                    0,
                ))
            }
            Err(e) => Err(format!(
                "[Warmup] Token refresh failed for {}: {}",
                email, e
            )),
        }
    }

    // ===== 限流管理方法 =====

    /// 检查是否有可用的 Google 账号
    ///
    /// 用于"仅兜底"模式的智能判断:当所有 Google 账号不可用时才使用外部提供商。
    ///
    /// # 参数
    /// - `quota_group`: 配额组("claude" 或 "gemini"),暂未使用但保留用于未来扩展
    /// - `target_model`: 目标模型名称(已归一化),用于配额保护检查
    ///
    /// # 返回值
    /// - `true`: 至少有一个可用账号(未限流且未被配额保护)
    /// - `false`: 所有账号都不可用(被限流或被配额保护)
    ///
    /// # 示例
    /// ```ignore
    /// // 检查是否有可用账号处理 claude-sonnet 请求
    /// let has_available = token_manager.has_available_account("claude", "claude-sonnet-4-20250514").await;
    /// if !has_available {
    ///     // 切换到外部提供商
    /// }
    /// ```
    pub async fn has_available_account(&self, _quota_group: &str, target_model: &str) -> bool {
        // 检查配额保护是否启用
        let quota_protection_enabled = crate::modules::config::load_app_config()
            .map(|cfg| cfg.quota_protection.enabled)
            .unwrap_or(false);

        // 遍历所有账号,检查是否有可用的
        for entry in self.tokens.iter() {
            let token = entry.value();

            // 1. 检查是否被限流
            if self.is_rate_limited(&token.account_id, None).await {
                tracing::debug!(
                    "[Fallback Check] Account {} is rate-limited, skipping",
                    token.email
                );
                continue;
            }

            // 2. 检查是否被配额保护(如果启用)
            if quota_protection_enabled && token.protected_models.contains(target_model) {
                tracing::debug!(
                    "[Fallback Check] Account {} is quota-protected for model {}, skipping",
                    token.email,
                    target_model
                );
                continue;
            }

            // 找到至少一个可用账号
            tracing::debug!(
                "[Fallback Check] Found available account: {} for model {}",
                token.email,
                target_model
            );
            return true;
        }

        // 所有账号都不可用
        tracing::info!(
            "[Fallback Check] No available Google accounts for model {}, fallback should be triggered",
            target_model
        );
        false
    }

    /// 获取当前所有可用账号中收集到的官方下发的所有动态模型集合
    pub fn get_all_collected_models(&self) -> std::collections::HashSet<String> {
        let mut all_models = std::collections::HashSet::new();
        for entry in self.tokens.iter() {
            let token = entry.value();

            // Keep the raw quota model IDs for /v1/models discovery. `model_quotas`
            // intentionally stores normalized protection buckets (e.g. gemini-3-flash),
            // but clients need concrete usable IDs such as gemini-3-flash-agent.
            if let Some(raw_models) = Self::get_available_models_from_json(&token.account_path) {
                for model_id in raw_models {
                    all_models.insert(model_id);
                }
            }

            // Also keep normalized bucket IDs for existing quota/protection behavior.
            for model_id in token.model_quotas.keys() {
                all_models.insert(model_id.clone());
            }
        }
        all_models
    }

    /// [NEW] 从指定账号的动态额度数据中获取特定模型的 max_output_tokens
    ///
    /// # 返回
    /// - `Some(u64)`: 找到了动态限额数据
    /// - `None`: 账号不存在或该模型无数据（调用方应继续查静态默认表）
    pub fn get_model_output_limit_for_account(
        &self,
        account_id: &str,
        model_name: &str,
    ) -> Option<u64> {
        self.tokens
            .get(account_id)
            .and_then(|token| token.model_limits.get(model_name).copied())
    }

    /// Helper to find account ID by email
    pub fn get_account_id_by_email(&self, email: &str) -> Option<String> {
        for entry in self.tokens.iter() {
            if entry.value().email == email {
                return Some(entry.key().clone());
            }
        }
        None
    }
}
