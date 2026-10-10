//! 单账号文件解析（load_single_account）。

use super::helpers::update_account_json;
use super::types::OnDiskAccountState;
use super::ProxyToken;
use super::TokenManager;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

impl TokenManager {
    /// 加载单个账号
    pub(crate) async fn load_single_account(
        &self,
        path: &PathBuf,
    ) -> Result<Option<ProxyToken>, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

        let mut account: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

        // [修复 #1344] 先检查账号是否被手动禁用(非配额保护原因)
        let is_proxy_disabled = account
            .get("proxy_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let disabled_reason = account
            .get("proxy_disabled_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if is_proxy_disabled && disabled_reason != "quota_protection" {
            // Account manually disabled
            tracing::debug!(
                "Account skipped due to manual disable: {:?} (email={}, reason={})",
                path,
                account
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>"),
                disabled_reason
            );
            return Ok(None);
        }

        // [NEW] Check for validation block (VALIDATION_REQUIRED temporary block)
        if account
            .get("validation_blocked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let block_until = account
                .get("validation_blocked_until")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let now = chrono::Utc::now().timestamp();

            if now < block_until {
                // Still blocked
                tracing::debug!(
                    "Skipping validation-blocked account: {:?} (email={}, blocked until {})",
                    path,
                    account
                        .get("email")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>"),
                    chrono::DateTime::from_timestamp(block_until, 0)
                        .map(|dt| dt.format("%H:%M:%S").to_string())
                        .unwrap_or_else(|| block_until.to_string())
                );
                return Ok(None);
            } else {
                // Block expired - clear it
                account["validation_blocked"] = serde_json::json!(false);
                account["validation_blocked_until"] = serde_json::json!(0);
                account["validation_blocked_reason"] = serde_json::Value::Null;

                update_account_json(path, |latest| {
                    latest["validation_blocked"] = serde_json::json!(false);
                    latest["validation_blocked_until"] = serde_json::json!(0);
                    latest["validation_blocked_reason"] = serde_json::Value::Null;
                })
                .await?;
                tracing::info!(
                    "Validation block expired and cleared for account: {}",
                    account
                        .get("email")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>")
                );
            }
        }

        // 最终检查账号主开关
        if account
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            tracing::debug!(
                "Skipping disabled account file: {:?} (email={})",
                path,
                account
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>")
            );
            return Ok(None);
        }

        // Safety check: verify state on disk again to handle concurrent mid-parse writes
        if Self::get_account_state_on_disk(path).await == OnDiskAccountState::Disabled {
            tracing::debug!("Account file {:?} is disabled on disk, skipping.", path);
            return Ok(None);
        }

        // 配额保护检查 - 只处理配额保护逻辑
        // 这样可以在加载时自动恢复配额已恢复的账号
        if self.check_and_protect_quota(&mut account, path).await {
            tracing::debug!(
                "Account skipped due to quota protection: {:?} (email={})",
                path,
                account
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>")
            );
            return Ok(None);
        }

        // [兼容性] 再次确认最终状态（可能被 check_and_protect_quota 修改）
        if account
            .get("proxy_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            tracing::debug!(
                "Skipping proxy-disabled account file: {:?} (email={})",
                path,
                account
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>")
            );
            return Ok(None);
        }

        let account_id = account["id"].as_str().ok_or("缺少 id 字段")?.to_string();

        let email = account["email"]
            .as_str()
            .ok_or("缺少 email 字段")?
            .to_string();

        let token_obj = account["token"].as_object().ok_or("缺少 token 字段")?;

        let access_token = token_obj["access_token"]
            .as_str()
            .ok_or("缺少 access_token")?
            .to_string();

        let refresh_token = token_obj["refresh_token"]
            .as_str()
            .ok_or("缺少 refresh_token")?
            .to_string();

        let expires_in = token_obj["expires_in"].as_i64().ok_or("缺少 expires_in")?;

        let timestamp = token_obj["expiry_timestamp"]
            .as_i64()
            .ok_or("缺少 expiry_timestamp")?;

        // project_id 是可选的
        let project_id = token_obj
            .get("project_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 【新增】提取订阅等级 (subscription_tier 为 "FREE" | "PRO" | "ULTRA")
        let subscription_tier = account
            .get("quota")
            .and_then(|q| q.get("subscription_tier"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // [FIX #563] 提取最大剩余配额百分比用于优先级排序 (Option<i32> now)
        let remaining_quota = account
            .get("quota")
            .and_then(|q| self.calculate_quota_stats(q));
        // .filter(|&r| r > 0); // 移除 >0 过滤，因为 0% 也是有效数据，只是优先级低

        // 【新增 #621】提取受限模型列表
        let protected_models: HashSet<String> = account
            .get("protected_models")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let health_score = self
            .health_scores
            .get(&account_id)
            .map(|v| *v)
            .unwrap_or(1.0);

        // [NEW] 提取最近的配额刷新时间（用于排序优化：刷新时间越近优先级越高）
        let reset_time = self.extract_earliest_reset_time(&account);

        // [OPTIMIZATION] 构建模型配额内存缓存，避免排序时读取磁盘
        let mut model_quotas = HashMap::new();
        // [NEW] 构建模型输出限额内存缓存 (max_output_tokens)
        let mut model_limits: HashMap<String, u64> = HashMap::new();
        if let Some(models) = account
            .get("quota")
            .and_then(|q| q.get("models"))
            .and_then(|m| m.as_array())
        {
            for model in models {
                if let (Some(name), Some(pct)) = (
                    model.get("name").and_then(|v| v.as_str()),
                    model.get("percentage").and_then(|v| v.as_i64()),
                ) {
                    // Normalize name to standard ID
                    let standard_id =
                        crate::proxy::common::model_mapping::normalize_to_standard_id(name)
                            .unwrap_or_else(|| name.to_string());
                    model_quotas.insert(standard_id, pct as i32);
                }
                // [NEW] 解析并缓存 max_output_tokens (按原始 model name，不归一化)
                if let (Some(name), Some(limit)) = (
                    model.get("name").and_then(|v| v.as_str()),
                    model.get("max_output_tokens").and_then(|v| v.as_u64()),
                ) {
                    model_limits.insert(name.to_string(), limit);
                }
            }
        }

        if let Some(live_limits) = account
            .get("live_limited_models")
            .and_then(|value| value.as_object())
        {
            let now = chrono::Utc::now().timestamp();
            for (model_key, status) in live_limits {
                let Ok(status) = serde_json::from_value::<crate::models::account::LiveLimitStatus>(
                    status.clone(),
                ) else {
                    continue;
                };
                if !crate::proxy::rate_limit::is_active_persisted_long_limit(
                    model_key, &status, now,
                ) {
                    continue;
                }
                let (Ok(until_seconds), Ok(detected_at_seconds)) = (
                    u64::try_from(status.until),
                    u64::try_from(status.detected_at),
                ) else {
                    continue;
                };
                self.rate_limit_tracker.restore_persisted_long_limit(
                    &account_id,
                    std::time::SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(until_seconds),
                    std::time::SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(detected_at_seconds),
                    model_key,
                );
            }
        }

        // [NEW] 启动时自动同步持久化的淘汰模型路由表，注入热更新拦截器
        if let Some(rules) = account
            .get("quota")
            .and_then(|q| q.get("model_forwarding_rules"))
            .and_then(|r| r.as_object())
        {
            for (k, v) in rules {
                if let Some(new_model) = v.as_str() {
                    // Register dynamic forwarding rules (including those mapping to gemini-pro-agent)
                    crate::proxy::common::model_mapping::update_dynamic_forwarding_rules(
                        k.to_string(),
                        new_model.to_string(),
                    );
                }
            }
        }

        // Weekly availability is mandatory; the optional switch only controls 5h locks.
        self.sync_zero_quota_circuit_breaker(&account_id, &account);

        Ok(Some(ProxyToken {
            account_id,
            priority: crate::models::account::deserialize_priority(
                account.get("priority").unwrap_or(&serde_json::json!(
                    crate::models::account::default_priority()
                )),
            )
            .map_err(|e| format!("invalid account priority: {}", e))?,
            access_token,
            refresh_token,
            expires_in,
            timestamp,
            email,
            account_path: path.clone(),
            project_id,
            subscription_tier,
            remaining_quota,
            protected_models,
            health_score,
            reset_time,
            validation_blocked: account
                .get("validation_blocked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            validation_blocked_until: account
                .get("validation_blocked_until")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            validation_url: account
                .get("validation_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            model_quotas,
            model_limits,
        }))
    }
}
