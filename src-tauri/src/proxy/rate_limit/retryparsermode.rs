use super::*;

pub(crate) const MAX_LOCKOUT_SECONDS: u64 = 300;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetryParserMode {
    Current,
    Baseline,
}

/// Current limiting reason type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RateLimitReason {
    /// Quota exhausted (QUOTA_EXHAUSTED)
    QuotaExhausted,
    /// Rate Limit (RATE_LIMIT_EXCEEDED)
    RateLimitExceeded,
    /// Model capacity exhausted (MODEL_CAPACITY_EXHAUSTED)
    ModelCapacityExhausted,
    /// Server error (5xx)
    ServerError,
    /// unknown reason
    Unknown,
}

pub(crate) fn normalize_image_model_id(model: &str) -> Option<String> {
    let normalized = crate::proxy::common::model_mapping::normalize_to_standard_id(model)?;
    matches!(
        normalized.as_str(),
        "gemini-3.1-flash-image" | "gemini-3-pro-image"
    )
    .then_some(normalized)
}

pub(crate) fn has_explicit_quota_exhausted(body: &str) -> bool {
    body.to_ascii_uppercase().contains("QUOTA_EXHAUSTED")
}

pub(crate) fn is_active_persisted_long_limit(
    _model_key: &str,
    status: &crate::models::account::LiveLimitStatus,
    now: i64,
) -> bool {
    status.status == 429
        && status.reason == "QuotaExhausted"
        && status.until > now
        && status.until.saturating_sub(status.detected_at) > MAX_LOCKOUT_SECONDS as i64
        && status.message.as_deref().is_some_and(|message| {
            has_explicit_quota_exhausted(message)
                && crate::proxy::upstream::retry::parse_retry_delay(message, None).is_some()
        })
}

pub(crate) fn is_active_persisted_long_image_limit(
    model_key: &str,
    status: &crate::models::account::LiveLimitStatus,
    now: i64,
) -> bool {
    normalize_image_model_id(model_key).is_some()
        && is_active_persisted_long_limit(model_key, status, now)
}

/// Current limiting information
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Current limit reset time
    pub reset_time: SystemTime,
    /// Retry interval (seconds)
    #[allow(dead_code)]
    pub retry_after_sec: u64,
    /// Detection time
    #[allow(dead_code)]
    pub detected_at: SystemTime,
    /// Reasons for current limiting
    #[allow(dead_code)] // Used for logging and diagnostics
    pub reason: RateLimitReason,
    /// Associated models (for model-level current limiting)
    /// None indicates account level current limit, Some(model) indicates specific model current limit
    #[allow(dead_code)] // Used for model-level rate limiting
    pub model: Option<String>,
}

/// Failure count expiration time: 1 hour (if there is no failure after this time, the count will be reset)
pub(crate) const FAILURE_COUNT_EXPIRY_SECONDS: u64 = 3600;

/// Current limit tracker
pub(crate) struct QuotaBucketLimit {
    pub(crate) observed_at: i64,
    pub(crate) reset_time: Option<SystemTime>,
    pub(crate) weekly: bool,
}

pub struct RateLimitTracker {
    pub(crate) limits: DashMap<String, RateLimitInfo>,
    // Independent official quota windows must survive transient-limit resets.
    pub(crate) quota_limits: DashMap<(String, String), QuotaBucketLimit>,
    /// Consecutive failure count (for smart exponential backoff) with timestamp for automatic expiration
    pub(crate) failure_counts: DashMap<String, (u32, SystemTime)>,
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self {
            limits: DashMap::new(),
            quota_limits: DashMap::new(),
            failure_counts: DashMap::new(),
        }
    }

    /// Generate current limiting Key
    /// - Account level: "account_id"
    /// - Model level: "account_id:model_id"
    pub(crate) fn get_limit_key(&self, account_id: &str, model: Option<&str>) -> String {
        match model {
            Some(m) if !m.is_empty() => format!("{}:{}", account_id, m),
            _ => account_id.to_string(),
        }
    }

    /// Get the remaining waiting time of the account (seconds)
    /// Supports checking account-level and model-level locks
    pub fn get_remaining_wait(&self, account_id: &str, model: Option<&str>) -> u64 {
        let now = SystemTime::now();
        let model_key = self.get_limit_key(account_id, model);
        let direct_wait = [account_id, model_key.as_str()]
            .into_iter()
            .filter_map(|key| self.limits.get(key))
            .filter_map(|info| info.reset_time.duration_since(now).ok())
            .map(|duration| duration.as_secs().max(1))
            .max()
            .unwrap_or(0)
            .max(self.get_quota_wait(account_id, model, false));

        if direct_wait > 0 {
            return direct_wait;
        }

        // [Double guard] If direct matching misses, pass normalized standard ID back-up check (make sure all variants such as gemini-*-pro are aligned with the standard group lock)
        if let Some(m) = model {
            if let Some(std_id) = crate::proxy::common::model_mapping::normalize_to_standard_id(m) {
                if std_id != m {
                    let std_key = self.get_limit_key(account_id, Some(&std_id));
                    let std_wait = self
                        .limits
                        .get(&std_key)
                        .and_then(|info| info.reset_time.duration_since(now).ok())
                        .map(|duration| duration.as_secs().max(1))
                        .unwrap_or(0)
                        .max(self.get_quota_wait(account_id, Some(&std_id), false));
                    if std_wait > 0 {
                        return std_wait;
                    }
                }
            }
        }

        0
    }

    pub fn get_quota_wait(&self, account_id: &str, model: Option<&str>, weekly_only: bool) -> u64 {
        let key = self.get_limit_key(account_id, model);
        let now = SystemTime::now();
        self.quota_limits
            .iter()
            .filter(|entry| {
                (entry.key().0 == key || entry.key().0 == account_id)
                    && (!weekly_only || entry.weekly)
            })
            .filter_map(|entry| entry.reset_time?.duration_since(now).ok())
            .map(|duration| duration.as_secs().max(1))
            .max()
            .unwrap_or(0)
    }

    pub fn sync_quota_bucket(
        &self,
        account_id: &str,
        model: &str,
        bucket_id: &str,
        observed_at: i64,
        exhausted_until: Option<SystemTime>,
        weekly: bool,
    ) {
        let key = (
            self.get_limit_key(account_id, Some(model)),
            bucket_id.to_string(),
        );
        // Keep recovered observations too: reloading an older snapshot must not relock/unlock.
        self.quota_limits
            .entry(key)
            .and_modify(|current| {
                if observed_at > current.observed_at {
                    *current = QuotaBucketLimit {
                        observed_at,
                        reset_time: exhausted_until,
                        weekly,
                    };
                }
            })
            .or_insert(QuotaBucketLimit {
                observed_at,
                reset_time: exhausted_until,
                weekly,
            });
    }

    /// Mark the account request as successful and reset the consecutive failure count
    ///
    /// This method is called when the account successfully completes the request to reset its failure count to zero.
    /// In this way, the next time it fails, it will start from the shortest lock time (60 seconds).
    pub fn mark_success(&self, account_id: &str) {
        if self.failure_counts.remove(account_id).is_some() {
            tracing::debug!(
                "The request for account {} was successful and the failure count has been reset.",
                account_id
            );
        }
        // Clear account-level current limit
        self.limits.remove(account_id);
        // Note: We are currently unable to clear all model-level locks under this account because we do not know which models are locked.
        // Unless limits are traversed. Considering that model-level locks are usually QuotaExhausted, it is also acceptable to let them expire naturally.
        // Or we can introduce an index, but for the sake of simplicity, only clear Account-level locks for now.
    }

    /// Accurately lock the account to a specified time point (supports whether to comply with the MAX_LOCKOUT_SECONDS upper limit)
    ///
    /// # Parameters
    /// - `model`: Optional model name, used for model-level current limiting. None means account level current limit
    /// - `cap_to_max`: Whether to limit the lock duration to MAX_LOCKOUT_SECONDS (300s).
    ///   If false, lock directly to the real reset_time (for zero quota continuous circuit breaker).
    pub fn set_lockout_until_with_cap(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        reason: RateLimitReason,
        model: Option<String>,
        cap_to_max: bool,
    ) {
        let now = SystemTime::now();
        let (mut retry_sec, mut effective_reset_time) = reset_time
            .duration_since(now)
            .map(|duration| (duration.as_secs(), reset_time))
            .unwrap_or((60, now + Duration::from_secs(60)));

        if cap_to_max && retry_sec > MAX_LOCKOUT_SECONDS {
            tracing::info!(
                "Capping lockout time for {} from {}s to 300s (5 minutes)",
                account_id,
                retry_sec
            );
            retry_sec = MAX_LOCKOUT_SECONDS;
            effective_reset_time = now + Duration::from_secs(retry_sec);
        }

        let info = RateLimitInfo {
            reset_time: effective_reset_time,
            retry_after_sec: retry_sec,
            detected_at: now,
            reason,
            model: model.clone(), // 🆕 Support model level current limiting
        };

        let key = self.get_limit_key(account_id, model.as_deref());

        // [Anti-regression protection] If there is a longer, unexpired lock time, prevent it from being overwritten by a subsequent shorter reset time (for example, the weekly quota of 5 days is not covered by the 5H window)
        if let Some(existing) = self.limits.get(&key) {
            if existing.reset_time > now && existing.reset_time > effective_reset_time {
                tracing::info!(
                    "Retaining existing longer lockout for {} (existing: {}s > new: {}s)",
                    key,
                    existing.retry_after_sec,
                    retry_sec
                );
                return;
            }
        }

        self.limits.insert(key, info);

        if let Some(m) = &model {
            tracing::info!(
                "Model {} of account {} has been accurately locked to the quota refresh time, with {} seconds remaining (cap_to_max: {} )",
                account_id,
                m,
                retry_sec,
                cap_to_max
            );
        } else {
            tracing::info!(
                "Account {} has been accurately locked to the quota refresh time, with {} seconds remaining (cap_to_max: {} )",
                account_id,
                retry_sec,
                cap_to_max
            );
        }
    }

    /// Precisely lock the account to a specified time point (default limit is within 300s)
    pub fn set_lockout_until(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        reason: RateLimitReason,
        model: Option<String>,
    ) {
        self.set_lockout_until_with_cap(account_id, reset_time, reason, model, true);
    }

    pub fn restore_persisted_long_limit(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        detected_at: SystemTime,
        model: &str,
    ) -> bool {
        let normalized_model = crate::proxy::common::model_mapping::normalize_to_standard_id(model)
            .unwrap_or_else(|| model.to_string());
        let now = SystemTime::now();
        let Ok(original_duration) = reset_time.duration_since(detected_at) else {
            return false;
        };
        let Ok(remaining) = reset_time.duration_since(now) else {
            return false;
        };
        if original_duration <= Duration::from_secs(MAX_LOCKOUT_SECONDS) {
            return false;
        }

        let info = RateLimitInfo {
            reset_time,
            retry_after_sec: remaining.as_secs(),
            detected_at,
            reason: RateLimitReason::QuotaExhausted,
            model: Some(normalized_model.clone()),
        };
        let key = self.get_limit_key(account_id, Some(&normalized_model));
        self.limits.insert(key, info);
        true
    }

    pub fn restore_persisted_long_image_limit(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        detected_at: SystemTime,
        model: &str,
    ) -> bool {
        if normalize_image_model_id(model).is_none() {
            return false;
        }
        self.restore_persisted_long_limit(account_id, reset_time, detected_at, model)
    }
}
