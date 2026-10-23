//! Account query: search by ID/email/model availability.

use super::ProxyToken;
use super::TokenManager;
use std::collections::HashSet;

impl TokenManager {
    /// Get the complete ProxyToken object based on the account ID (v4.1.29)
    pub fn get_token_by_id(&self, account_id: &str) -> Option<ProxyToken> {
        self.tokens.get(account_id).map(|t| t.clone())
    }

    /// Obtain the token of the specified account through email (used for scenarios such as preheating that require a specified account)
    /// This method will automatically refresh expired tokens
    pub async fn get_token_by_email(
        &self,
        email: &str,
    ) -> Result<(String, String, String, String, u64), String> {
        // Find account information
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
            None => return Err(format!("Account not found: {}", email)),
        };

        let project_id = project_id_opt
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "bamboo-precept-lgxtn".to_string());

        // Check if expired (5 minutes in advance)
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

        // Call OAuth to refresh token
        match crate::modules::oauth::refresh_access_token(&refresh_token, Some(&account_id)).await {
            Ok(token_response) => {
                tracing::info!("[Warmup] Token refresh successful for {}", email);
                let new_now = chrono::Utc::now().timestamp();

                // Update cache
                if let Some(mut entry) = self.tokens.get_mut(&account_id) {
                    entry.access_token = token_response.access_token.clone();
                    entry.expires_in = token_response.expires_in;
                    entry.timestamp = new_now;
                }

                // save to disk
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

    // ===== Current limiting management method =====

    /// Check if there is an available Google account
    ///
    /// Intelligent judgment for "cover only" mode: only use external providers when all Google accounts are unavailable.
    ///
    /// # Parameters
    /// - `quota_group`: Quota group ("claude" or "gemini"), not used yet but reserved for future expansion
    /// - `target_model`: target model name (normalized), used for quota protection checks
    ///
    /// # Return value
    /// - `true`: There is at least one available account (not limited and not protected by quota)
    /// - `false`: All accounts are unavailable (limited or protected by quotas)
    ///
    /// # Example
    /// ```ignore
    /// // Check if there is an available account to handle claude-sonnet request
    /// let has_available = token_manager.has_available_account("claude", "claude-sonnet-4-20250514").await;
    /// if !has_available {
    ///     // switch to external provider
    /// }
    /// ```
    pub async fn has_available_account(&self, _quota_group: &str, target_model: &str) -> bool {
        // Check if quota protection is enabled
        let quota_protection_enabled = crate::modules::config::load_app_config()
            .map(|cfg| cfg.quota_protection.enabled)
            .unwrap_or(false);

        // Traverse all accounts and check if any are available
        for entry in self.tokens.iter() {
            let token = entry.value();

            // 1. Check whether the current is limited
            if self.is_rate_limited(&token.account_id, None).await {
                tracing::debug!(
                    "[Fallback Check] Account {} is rate-limited, skipping",
                    token.email
                );
                continue;
            }

            // 2. Check whether it is protected by quota (if enabled)
            if quota_protection_enabled && token.protected_models.contains(target_model) {
                tracing::debug!(
                    "[Fallback Check] Account {} is quota-protected for model {}, skipping",
                    token.email,
                    target_model
                );
                continue;
            }

            // Find at least one available account
            tracing::debug!(
                "[Fallback Check] Found available account: {} for model {}",
                token.email,
                target_model
            );
            return true;
        }

        // All accounts are unavailable
        tracing::info!(
            "[Fallback Check] No available Google accounts for model {}, fallback should be triggered",
            target_model
        );
        false
    }

    /// Get all officially issued dynamic model collections collected from all currently available accounts.
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

    /// [NEW] Get the max_output_tokens of a specific model from the dynamic quota data of the specified account
    ///
    /// # Return
    /// - `Some(u64)`: Dynamic limit data found
    /// - `None`: The account does not exist or the model has no data (the caller should continue to check the static default table)
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
