//! Account write operations: deactivation, persistence of refreshed Token, and verification interception.

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

        // [Fix Issue #3] Remove disabled accounts from memory to prevent continued use by 60s lock logic
        self.remove_account(account_id);

        tracing::warn!("Account disabled: {} ({:?})", account_id, path);
        Ok(())
    }

    /// Save project_id to account file
    pub(crate) async fn save_project_id(
        &self,
        account_id: &str,
        project_id: &str,
    ) -> Result<(), String> {
        let path = self
            .tokens
            .get(account_id)
            .ok_or("Account does not exist")?
            .account_path
            .clone();
        let project_id_owned = project_id.to_string();
        update_account_json(&path, move |content| {
            content["token"]["project_id"] = serde_json::Value::String(project_id_owned);
        })
        .await?;

        tracing::debug!("Saved project_id to account {}", account_id);
        Ok(())
    }

    /// Save the refreshed token to the account file
    pub(crate) async fn save_refreshed_token(
        &self,
        account_id: &str,
        token_response: &crate::modules::oauth::TokenResponse,
    ) -> Result<(), String> {
        let path = self
            .tokens
            .get(account_id)
            .ok_or("Account does not exist")?
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

            // If a new id_token is obtained, save it
            if let Some(it) = id_token {
                content["token"]["id_token"] = serde_json::Value::String(it);
            }

            // If a new refresh_token (Token rotation) is obtained, it will also be saved.
            if let Some(rt) = refresh_token {
                content["token"]["refresh_token"] = serde_json::Value::String(rt);
            }
        })
        .await?;

        tracing::debug!(
            "The refreshed token has been saved to account {}",
            account_id
        );
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

        // [NEW] Try to extract verification link from message (#1522)
        let extracted_url = if let Ok(parsed_json) =
            serde_json::from_str::<serde_json::Value>(reason)
        {
            // Try to get from a specific Google RPC error structure
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
            // Fallback solution: Decode possible \u0026 through stricter regularization and deserialization
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
        // [FIX] Call encapsulated module functions to ensure thread-safe update of account files and indexes
        crate::modules::account::mark_account_forbidden(account_id, reason)?;

        // Clear sticky session if forbidden
        self.session_accounts.retain(|_, v| *v != account_id);

        // [FIX] Remove account from memory pool to avoid re-selecting when retrying
        self.remove_account(account_id);

        tracing::warn!(
            "🚫 Account {} marked as forbidden (403): {}",
            account_id,
            truncate_reason(reason, 1000)
        );

        Ok(())
    }
}
