//! Rate limit mark: mark_rate_limited series and atomic records.

use super::types::TrackerParserMode;
use super::TokenManager;

impl TokenManager {
    /// Use quota refresh time to accurately lock accounts
    ///
    /// When the API returns 429 but there is no quotaResetDelay, try to use the account's quota refresh time
    ///
    /// # Parameters
    /// - `account_id`: account ID
    /// - `reason`: current limiting reason (QuotaExhausted/ServerError, etc.)
    /// - `model`: optional model name, used for model-level current limiting
    pub fn set_precise_lockout(
        &self,
        account_id: &str,
        reason: crate::proxy::rate_limit::RateLimitReason,
        model: Option<String>,
    ) -> bool {
        // [FIX #2209] Unify normalized model names
        let normalized_model = model
            .as_deref()
            .and_then(|m| crate::proxy::common::model_mapping::normalize_to_standard_id(m));
        let model_to_lock = normalized_model.or(model);

        let cap = if let Ok(cfg) = self.circuit_breaker_config.try_read() {
            !cfg.lock_on_zero_quota
        } else {
            true
        };

        if let Some(reset_time_str) = self.get_quota_reset_time(account_id) {
            tracing::info!(
                "Find the quota refresh time of account {} : {} (cap_to_max: {} )",
                account_id,
                reset_time_str,
                cap
            );
            self.rate_limit_tracker.set_lockout_until_iso_with_cap(
                account_id,
                &reset_time_str,
                reason,
                model_to_lock,
                cap,
            )
        } else {
            tracing::debug!(
                "The quota refresh time for account {} was not found, and the default backoff policy will be used.",
                account_id
            );
            false
        }
    }

    /// Refresh quotas in real time and lock accounts accurately
    ///
    /// This method is called when 429 occurs:
    /// 1. Call the quota refresh API in real time to obtain the latest reset_time
    /// 2. Use the latest reset_time to accurately lock the account
    /// 3. If the acquisition fails, return false to let the caller use the fallback strategy.
    ///
    /// # Parameters
    /// - `model`: optional model name, used for model-level current limiting
    pub async fn fetch_and_lock_with_realtime_quota(
        &self,
        email: &str,
        reason: crate::proxy::rate_limit::RateLimitReason,
        model: Option<String>,
    ) -> bool {
        // 1. Obtain the access_token and account_id of the account from tokens
        // At the same time, obtain the account_id and ensure that the lock key is consistent with the check key.
        let (access_token, account_id) = {
            let mut found: Option<(String, String)> = None;
            for entry in self.tokens.iter() {
                if entry.value().email == email {
                    found = Some((
                        entry.value().access_token.clone(),
                        entry.value().account_id.clone(),
                    ));
                    break;
                }
            }
            found
        }
        .unzip();

        let (access_token, account_id) = match (access_token, account_id) {
            (Some(token), Some(id)) => (token, id),
            _ => {
                tracing::warn!("The access_token of account {} cannot be found and the quota cannot be refreshed in real time.", email);
                return false;
            }
        };

        // 2. Call the quota refresh API
        tracing::info!("Account {} is refreshing the quota in real time...", email);
        match crate::modules::quota::fetch_quota(&access_token, email, Some(&account_id)).await {
            Ok((quota_data, _project_id)) => {
                // 3. Extract reset_time from latest quota
                let earliest_reset = quota_data
                    .models
                    .iter()
                    .filter_map(|m| {
                        if !m.reset_time.is_empty() {
                            Some(m.reset_time.as_str())
                        } else {
                            None
                        }
                    })
                    .min();

                if let Some(reset_time_str) = earliest_reset {
                    tracing::info!(
                        "Account {} real-time quota refresh successful, reset_time: {}",
                        email,
                        reset_time_str
                    );

                    // [FIX #2209] Unify normalized model names
                    let normalized_model = model.as_deref().and_then(|m| {
                        crate::proxy::common::model_mapping::normalize_to_standard_id(m)
                    });
                    let model_to_lock = normalized_model.or(model);

                    let cap = if let Ok(cfg) = self.circuit_breaker_config.try_read() {
                        !cfg.lock_on_zero_quota
                    } else {
                        true
                    };

                    // [FIX] Use account_id as key, consistent with is_rate_limited check
                    self.rate_limit_tracker.set_lockout_until_iso_with_cap(
                        &account_id,
                        reset_time_str,
                        reason,
                        model_to_lock,
                        cap,
                    )
                } else {
                    tracing::warn!(
                        "Account {} quota refreshed successfully but reset_time not found",
                        email
                    );
                    false
                }
            }
            Err(e) => {
                tracing::warn!("Account {} real-time quota refresh failed: {:?}", email, e);
                false
            }
        }
    }

    pub(crate) fn has_explicit_retry_time(
        parser_mode: TrackerParserMode,
        retry_after_header: Option<&str>,
        error_body: &str,
    ) -> bool {
        match parser_mode {
            TrackerParserMode::Current => {
                crate::proxy::upstream::retry::parse_retry_delay(error_body, retry_after_header)
                    .is_some()
            }
            TrackerParserMode::Baseline => {
                retry_after_header.is_some() || error_body.contains("quotaResetDelay")
            }
        }
    }

    fn parse_rate_limit_with_mode(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
        backoff_steps: &[u64],
        parser_mode: TrackerParserMode,
    ) -> Option<crate::proxy::rate_limit::RateLimitInfo> {
        match parser_mode {
            TrackerParserMode::Current => self.rate_limit_tracker.parse_from_error(
                account_id,
                status,
                retry_after_header,
                error_body,
                model.map(str::to_string),
                backoff_steps,
            ),
            TrackerParserMode::Baseline => self.rate_limit_tracker.parse_from_error_baseline(
                account_id,
                status,
                retry_after_header,
                error_body,
                model.map(str::to_string),
                backoff_steps,
            ),
        }
    }

    pub(crate) fn record_rate_limit_atomic(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
        backoff_steps: &[u64],
        parser_mode: TrackerParserMode,
    ) -> Option<crate::proxy::rate_limit::RateLimitInfo> {
        if status != 429 && status != 529 {
            return None;
        }
        if model
            .and_then(crate::proxy::rate_limit::normalize_image_model_id)
            .is_some()
        {
            match crate::modules::account::lock_account_file_updates() {
                Ok(_account_write) => {
                    let info = self.parse_rate_limit_with_mode(
                        account_id,
                        status,
                        retry_after_header,
                        error_body,
                        model,
                        backoff_steps,
                        parser_mode,
                    );
                    if parser_mode == TrackerParserMode::Current {
                        if let Some(ref info) = info {
                            self.persist_live_limit_locked(
                                account_id,
                                model,
                                status,
                                retry_after_header,
                                error_body,
                                info,
                            );
                        }
                    }
                    return info;
                }
                Err(error) => {
                    tracing::debug!(
                        "Failed to serialize live limit update for {}: {}",
                        account_id,
                        error
                    );
                }
            }
        }

        self.parse_rate_limit_with_mode(
            account_id,
            status,
            retry_after_header,
            error_body,
            model,
            backoff_steps,
            parser_mode,
        )
    }
}
