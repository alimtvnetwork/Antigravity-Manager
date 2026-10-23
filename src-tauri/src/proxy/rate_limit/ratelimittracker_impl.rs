use super::retryparsermode::FAILURE_COUNT_EXPIRY_SECONDS;
use super::*;

impl RateLimitTracker {
    pub fn set_lockout_until_iso_with_cap(
        &self,
        account_id: &str,
        reset_time_str: &str,
        reason: RateLimitReason,
        model: Option<String>,
        cap_to_max: bool,
    ) -> bool {
        // Try to parse ISO 8601 format
        match chrono::DateTime::parse_from_rfc3339(reset_time_str) {
            Ok(dt) => {
                let reset_time =
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64);
                self.set_lockout_until_with_cap(account_id, reset_time, reason, model, cap_to_max);
                true
            }
            Err(e) => {
                tracing::warn!(
                    "Unable to parse quota refresh time ' {} ': {} , default backoff policy will be used",
                    reset_time_str,
                    e
                );
                false
            }
        }
    }

    /// Precisely lock accounts using ISO 8601 time strings
    ///
    /// Parse a time string in a format similar to "2026-01-08T17:00:00Z"
    ///
    /// # Parameters
    /// - `model`: optional model name, used for model-level current limiting
    pub fn set_lockout_until_iso(
        &self,
        account_id: &str,
        reset_time_str: &str,
        reason: RateLimitReason,
        model: Option<String>,
    ) -> bool {
        self.set_lockout_until_iso_with_cap(account_id, reset_time_str, reason, model, true)
    }

    /// Parse throttling information from error response
    ///
    /// # Arguments
    /// * `account_id` - Account ID
    /// * `status` - HTTP status code
    /// * `retry_after_header` - Retry-After header value
    /// * `body` - error response body
    pub fn parse_from_error(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64], // [NEW] Incoming backoff configuration
    ) -> Option<RateLimitInfo> {
        self.parse_from_error_with_mode(
            account_id,
            status,
            retry_after_header,
            body,
            model,
            backoff_steps,
            RetryParserMode::Current,
        )
    }

    pub fn parse_from_error_baseline(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64],
    ) -> Option<RateLimitInfo> {
        self.parse_from_error_with_mode(
            account_id,
            status,
            retry_after_header,
            body,
            model,
            backoff_steps,
            RetryParserMode::Baseline,
        )
    }

    fn parse_from_error_with_mode(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64],
        parser_mode: RetryParserMode,
    ) -> Option<RateLimitInfo> {
        // Key defense: Queuing/no available account errors generated internally within the gateway are absolutely prohibited from being resolved into upstream current limiting, to prevent self-destructive loops.
        let lower_body = body.to_lowercase();
        if lower_body.contains("all accounts limited")
            || lower_body.contains("no accounts available")
            || lower_body.contains("all accounts failed")
            || lower_body.contains("token pool is empty")
            || lower_body.contains("all accounts exhausted")
            || lower_body.contains("all accounts unhealthy")
        {
            return None;
        }

        // Only the real upstream current limit 429 and overload 529 are tracked for account cooling; 500/503 means the service is temporarily unavailable and will never be entered into the cooling pool!
        if status != 429 && status != 529 {
            return None;
        }

        // 1. Analyze the type of current limiting reason
        let reason = if status == 429 {
            tracing::warn!("Google 429 Error Body: {}", body);
            self.parse_rate_limit_reason(body)
        } else {
            RateLimitReason::ServerError
        };

        let retry_after_sec = match parser_mode {
            RetryParserMode::Current => {
                crate::proxy::upstream::retry::parse_retry_delay(body, retry_after_header)
                    .map(|delay_ms| delay_ms.saturating_add(999) / 1000)
            }
            RetryParserMode::Baseline => retry_after_header
                .and_then(|value| value.parse::<u64>().ok())
                .or_else(|| self.parse_retry_time_from_body_baseline(body)),
        };
        let has_explicit_retry_time = retry_after_sec.is_some();
        let preserve_long_image_quota = parser_mode == RetryParserMode::Current
            && status == 429
            && reason == RateLimitReason::QuotaExhausted
            && has_explicit_quota_exhausted(body)
            && has_explicit_retry_time
            && model
                .as_deref()
                .and_then(normalize_image_model_id)
                .is_some();

        // 4. Process default values and soft avoidance logic (set different default values according to current limiting type)
        let retry_sec = match retry_after_sec {
            Some(s) => {
                // Set safety buffer: minimum 2 seconds to prevent extremely high frequency invalid retries
                if s < 2 {
                    2
                } else {
                    s
                }
            }
            None => {
                // Get the number of consecutive failures, used for exponential backoff (with automatic expiration logic)
                // [FIX] ServerError (5xx) does not accumulate failure_count to avoid polluting the 429 backoff ladder
                let failure_count = if reason != RateLimitReason::ServerError {
                    // Failure count is accumulated only for non-ServerError (used for exponential backoff)
                    let now = SystemTime::now();
                    // Here we use account_id as the key without distinguishing between models.
                    // Because this is to calculate the backoff of consecutive "account level" problems.
                    // If you need consecutive failure counts for your model, you may need to change the failure_counts key.
                    // Keep the account_id temporarily so that if a model keeps hanging, the count will be increased, which is logical.
                    let mut entry = self
                        .failure_counts
                        .entry(account_id.to_string())
                        .or_insert((0, now));

                    let elapsed = now
                        .duration_since(entry.1)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    if elapsed > FAILURE_COUNT_EXPIRY_SECONDS {
                        tracing::debug!(
                            "Account {} failure count has expired ( {} seconds) and reset to 0",
                            account_id,
                            elapsed
                        );
                        *entry = (0, now);
                    }
                    entry.0 += 1;
                    entry.1 = now;
                    entry.0
                } else {
                    // ServerError (5xx) uses a fixed value of 1 and does not accumulate to avoid polluting the 429 backoff ladder.
                    1
                };

                match reason {
                    RateLimitReason::QuotaExhausted => {
                        // [Intelligent current limiting] Calculated based on failure_count and configured backoff_steps
                        let index = (failure_count as usize).saturating_sub(1);
                        let lockout = if index < backoff_steps.len() {
                            backoff_steps[index]
                        } else {
                            *backoff_steps.last().unwrap_or(&7200)
                        };

                        tracing::warn!(
                            "Quota exhausted (QUOTA_EXHAUSTED) detected, {} consecutive failures, locked for {} seconds as configured",
                            failure_count,
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::RateLimitExceeded => {
                        // Rate Limiting (TPM/RPM)
                        let body_lower = body.to_lowercase();
                        let lockout = if body_lower.contains("resource has been exhausted")
                            || body_lower.contains("resource_exhausted")
                        {
                            30
                        } else {
                            5
                        };
                        tracing::debug!(
                            "Rate limit detected (RATE_LIMIT_EXCEEDED), using default value of {} seconds",
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::ModelCapacityExhausted => {
                        // Model capacity exhausted
                        let lockout = match failure_count {
                            1 => 5,
                            2 => 10,
                            _ => 15,
                        };
                        tracing::warn!(
                            "Insufficient model capacity detected (MODEL_CAPACITY_EXHAUSTED), failed for {} times, try again in {} seconds",
                            failure_count,
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::ServerError => {
                        let lockout = 8;
                        tracing::warn!(
                            "{} error detected, executing {} s soft avoidance...",
                            status,
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::Unknown => {
                        // unknown reason
                        tracing::debug!("Unable to parse 429 current limit reason, use the default value of 60 seconds");
                        60
                    }
                }
            }
        };

        let mut retry_sec = retry_sec;
        let max_allowed_lockout = backoff_steps
            .iter()
            .copied()
            .max()
            .unwrap_or(MAX_LOCKOUT_SECONDS)
            .max(MAX_LOCKOUT_SECONDS);
        if retry_sec > max_allowed_lockout && !preserve_long_image_quota {
            tracing::info!(
                "Capping retry lockout time for {} from {}s to {}s (max backoff limit)",
                account_id,
                retry_sec,
                max_allowed_lockout
            );
            retry_sec = max_allowed_lockout;
        }

        let info = RateLimitInfo {
            reset_time: SystemTime::now() + Duration::from_secs(retry_sec),
            retry_after_sec: retry_sec,
            detected_at: SystemTime::now(),
            reason,
            model: model.clone(),
        };

        // [FIX] Fine-grained model isolation: As long as the caller passes in a specific model, the current limit must be isolated for that model!
        // Prevent all models in the entire account from being blocked due to a single model (or non-existent model/specific model current limit), causing normal account downtime.
        let key = if let Some(m) = model.as_deref().filter(|s| !s.is_empty()) {
            self.get_limit_key(account_id, Some(m))
        } else {
            account_id.to_string()
        };

        self.limits.insert(key, info.clone());

        tracing::warn!(
            "Account {} [ {} ] Current limit type: {:?} , reset delay: {} seconds",
            account_id,
            status,
            reason,
            retry_sec
        );

        Some(info)
    }

    /// Analyze the current limiting reason type
    pub(crate) fn parse_rate_limit_reason(&self, body: &str) -> RateLimitReason {
        // Try to extract reason field from JSON
        let trimmed = body.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(reason_str) = json
                    .get("error")
                    .and_then(|e| e.get("details"))
                    .and_then(|d| d.as_array())
                    .and_then(|a| a.get(0))
                    .and_then(|o| o.get("reason"))
                    .and_then(|v| v.as_str())
                {
                    return match reason_str {
                        "QUOTA_EXHAUSTED" => RateLimitReason::QuotaExhausted,
                        "RATE_LIMIT_EXCEEDED" => RateLimitReason::RateLimitExceeded,
                        "MODEL_CAPACITY_EXHAUSTED" => RateLimitReason::ModelCapacityExhausted,
                        _ => RateLimitReason::Unknown,
                    };
                }
                // [NEW] Attempt text matching from message field (to prevent missed reason)
                if let Some(msg) = json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|v| v.as_str())
                {
                    let msg_lower = msg.to_lowercase();
                    if msg_lower.contains("per minute") || msg_lower.contains("rate limit") {
                        return RateLimitReason::RateLimitExceeded;
                    }
                }
            }
        }

        // If it cannot be parsed from JSON, try judging from the message text
        let body_lower = body.to_lowercase();
        // [FIX] Prioritize judgment of minute-level limits to avoid misjudgment of TPM as Quota
        let generic_resource_exhausted = body_lower.contains("resource has been exhausted")
            || body_lower.contains("resource_exhausted");
        let explicit_quota_exhausted = body_lower.contains("quotaresetdelay")
            || body_lower.contains("quotareset")
            || body_lower.contains("quota reset")
            || body_lower.contains("quota limit")
            || body_lower.contains("per day")
            || body_lower.contains("daily quota")
            || (body_lower.contains("quota_exhausted")
                && crate::proxy::upstream::retry::parse_retry_delay(body, None).is_some());

        if body_lower.contains("per minute")
            || body_lower.contains("rate limit")
            || body_lower.contains("too many requests")
            || (generic_resource_exhausted && !explicit_quota_exhausted)
        {
            RateLimitReason::RateLimitExceeded
        } else if explicit_quota_exhausted {
            RateLimitReason::QuotaExhausted
        } else if body_lower.contains("exhausted") || body_lower.contains("quota") {
            RateLimitReason::RateLimitExceeded
        } else {
            RateLimitReason::Unknown
        }
    }

    /// Parse reset time from error message body
    pub(crate) fn parse_retry_time_from_body(&self, body: &str) -> Option<u64> {
        crate::proxy::upstream::retry::parse_retry_delay(body, None)
            .map(|delay_ms| delay_ms.saturating_add(999) / 1000)
    }

    fn parse_duration_string_baseline(&self, value: &str) -> Option<u64> {
        let re = Regex::new(r"(?:(\d+)h)?(?:(\d+)m)?(?:(\d+(?:\.\d+)?)s)?(?:(\d+(?:\.\d+)?)ms)?")
            .ok()?;
        let captures = re.captures(value)?;
        let hours = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let minutes = captures
            .get(2)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let seconds = captures
            .get(3)
            .and_then(|value| value.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let milliseconds = captures
            .get(4)
            .and_then(|value| value.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let total_seconds = hours * 3600
            + minutes * 60
            + seconds.ceil() as u64
            + (milliseconds / 1000.0).ceil() as u64;
        (total_seconds > 0).then_some(total_seconds)
    }

    fn parse_retry_time_from_body_baseline(&self, body: &str) -> Option<u64> {
        let trimmed = body.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(delay) = json
                    .get("error")
                    .and_then(|error| error.get("details"))
                    .and_then(|details| details.as_array())
                    .and_then(|details| details.first())
                    .and_then(|detail| detail.get("metadata"))
                    .and_then(|metadata| metadata.get("quotaResetDelay"))
                    .and_then(|value| value.as_str())
                    .and_then(|value| self.parse_duration_string_baseline(value))
                {
                    return Some(delay);
                }
                if let Some(retry) = json
                    .get("error")
                    .and_then(|error| error.get("retry_after"))
                    .and_then(|value| value.as_u64())
                {
                    return Some(retry);
                }
            }
        }

        for pattern in [
            r"(?i)try again in (\d+)m\s*(\d+)s",
            r"(?i)(?:try again in|backoff for|wait)\s*(\d+)s",
            r"(?i)quota will reset in (\d+) second",
            r"(?i)retry after (\d+) second",
            r"\(wait (\d+)s\)",
        ] {
            let captures = Regex::new(pattern).ok()?.captures(body);
            let Some(captures) = captures else {
                continue;
            };
            if captures.len() == 3 {
                let minutes = captures.get(1)?.as_str().parse::<u64>().ok()?;
                let seconds = captures.get(2)?.as_str().parse::<u64>().ok()?;
                return Some(minutes * 60 + seconds);
            }
            if let Some(seconds) = captures
                .get(1)
                .and_then(|value| value.as_str().parse::<u64>().ok())
            {
                return Some(seconds);
            }
        }
        None
    }

    /// Get the current limit information of the account
    pub fn get(&self, account_id: &str) -> Option<RateLimitInfo> {
        self.limits.get(account_id).map(|r| r.clone())
    }
}
