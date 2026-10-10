//! 账号写操作：停用、持久化刷新后的 Token、验证拦截。

use super::helpers::truncate_reason;
use super::helpers::update_account_json;
use super::TokenManager;

impl TokenManager {
    /// Apply a saved priority without resetting sessions or live rate limits.
    pub fn update_account_priority(&self, account_id: &str, priority: u8) {
        if let Some(mut token) = self.tokens.get_mut(account_id) {
            token.priority = priority;
        }
    }

    pub(crate) async fn disable_account(
        &self,
        account_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let path = if let Some(entry) = self.tokens.get(account_id) {
            entry.account_path.clone()
        } else {
            self.resolved_data_dir()
                .join("accounts")
                .join(format!("{}.json", account_id))
        };

        let now = chrono::Utc::now().timestamp();
        let reason_owned = reason.to_string();
        update_account_json(&path, move |content| {
            content["disabled"] = serde_json::Value::Bool(true);
            content["disabled_at"] = serde_json::Value::Number(now.into());
            content["disabled_reason"] =
                serde_json::Value::String(truncate_reason(&reason_owned, 800));
        })
        .await?;

        // 【修复 Issue #3】从内存中移除禁用的账号，防止被60s锁定逻辑继续使用
        self.remove_account(account_id);

        tracing::warn!("Account disabled: {} ({:?})", account_id, path);
        Ok(())
    }

    /// 保存 project_id 到账号文件
    pub(crate) async fn save_project_id(
        &self,
        account_id: &str,
        project_id: &str,
    ) -> Result<(), String> {
        let path = self
            .tokens
            .get(account_id)
            .ok_or("账号不存在")?
            .account_path
            .clone();
        let project_id_owned = project_id.to_string();
        update_account_json(&path, move |content| {
            content["token"]["project_id"] = serde_json::Value::String(project_id_owned);
        })
        .await?;

        tracing::debug!("已保存 project_id 到账号 {}", account_id);
        Ok(())
    }

    /// 保存刷新后的 token 到账号文件
    pub(crate) async fn save_refreshed_token(
        &self,
        account_id: &str,
        token_response: &crate::modules::oauth::TokenResponse,
    ) -> Result<(), String> {
        let path = self
            .tokens
            .get(account_id)
            .ok_or("账号不存在")?
            .account_path
            .clone();
        let now = chrono::Utc::now().timestamp();
        let access_token = token_response.access_token.clone();
        let expires_in = token_response.expires_in;
        let id_token = token_response.id_token.clone();
        let refresh_token = token_response.refresh_token.clone();
        let expiry_timestamp = now + expires_in;
        update_account_json(&path, move |content| {
            content["token"]["access_token"] = serde_json::Value::String(access_token);
            content["token"]["expires_in"] = serde_json::Value::Number(expires_in.into());
            content["token"]["expiry_timestamp"] =
                serde_json::Value::Number(expiry_timestamp.into());

            // 如果获取到了新的 id_token，则保存它
            if let Some(it) = id_token {
                content["token"]["id_token"] = serde_json::Value::String(it);
            }

            // 如果获取到了新的 refresh_token（Token 轮转），也一并保存
            if let Some(rt) = refresh_token {
                content["token"]["refresh_token"] = serde_json::Value::String(rt);
            }
        })
        .await?;

        tracing::debug!("已保存刷新后的 token 到账号 {}", account_id);
        Ok(())
    }

    /// Set validation blocked status for an account (internal)
    pub async fn set_validation_block(
        &self,
        account_id: &str,
        block_until: i64,
        reason: &str,
    ) -> Result<(), String> {
        // 1. Update memory
        if let Some(mut token) = self.tokens.get_mut(account_id) {
            token.validation_blocked = true;
            token.validation_blocked_until = block_until;
        }

        // 2. Persist to disk
        let path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));
        if !path.exists() {
            return Err(format!("Account file not found: {:?}", path));
        }

        // [NEW] 尝试从消息中提取验证链接 (#1522)
        let extracted_url = if let Ok(parsed_json) =
            serde_json::from_str::<serde_json::Value>(reason)
        {
            // 尝试从特定的 Google RPC error 结构中取
            let mut url = None;
            if let Some(details) = parsed_json.pointer("/error/details") {
                if let Some(arr) = details.as_array() {
                    for detail in arr {
                        if let Some(meta) = detail.get("metadata") {
                            if let Some(v_url) = meta.get("validation_url").and_then(|v| v.as_str())
                            {
                                url = Some(v_url.to_string());
                                break;
                            }
                            if let Some(a_url) = meta.get("appeal_url").and_then(|v| v.as_str()) {
                                url = Some(a_url.to_string());
                                break;
                            }
                        }
                    }
                }
            }
            url
        } else {
            // 回退方案：通过更严格的正则及反序列化解码可能的 \u0026
            let url_regex = regex::Regex::new(r#"https://[^\s"'\\]+"#).unwrap();
            url_regex.find(reason).map(|m| {
                let raw_url = m.as_str().to_string();
                raw_url.replace("\\u0026", "&")
            })
        };

        if let Some(ref url) = extracted_url {
            if let Some(mut token) = self.tokens.get_mut(account_id) {
                token.validation_url = Some(url.clone());
            }
        }

        let reason_owned = reason.to_string();
        update_account_json(&path, move |account| {
            account["validation_blocked"] = serde_json::Value::Bool(true);
            account["validation_blocked_until"] =
                serde_json::Value::Number(serde_json::Number::from(block_until));
            account["validation_blocked_reason"] = serde_json::Value::String(reason_owned);
            if let Some(url) = extracted_url {
                account["validation_url"] = serde_json::Value::String(url);
            }
        })
        .await?;

        // Clear sticky session if blocked
        self.session_accounts.retain(|_, v| *v != account_id);

        tracing::info!(
            "🚫 Account {} validation blocked until {} (reason: {})",
            account_id,
            block_until,
            reason
        );

        Ok(())
    }

    /// Public method to set validation block (called from handlers)
    pub async fn set_validation_block_public(
        &self,
        account_id: &str,
        block_until: i64,
        reason: &str,
    ) -> Result<(), String> {
        self.set_validation_block(account_id, block_until, reason)
            .await
    }

    /// Set is_forbidden status for an account (called when proxy encounters 403)
    pub async fn set_forbidden(&self, account_id: &str, reason: &str) -> Result<(), String> {
        // [FIX] 调用封装好的模块函数，确保线程安全地更新账号文件和索引
        crate::modules::account::mark_account_forbidden(account_id, reason)?;

        // Clear sticky session if forbidden
        self.session_accounts.retain(|_, v| *v != account_id);

        // [FIX] 从内存池中移除账号，避免重试时再次选中
        self.remove_account(account_id);

        tracing::warn!(
            "🚫 Account {} marked as forbidden (403): {}",
            account_id,
            truncate_reason(reason, 1000)
        );

        Ok(())
    }
}
