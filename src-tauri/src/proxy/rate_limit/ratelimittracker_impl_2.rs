use super::*;

impl RateLimitTracker {
    pub fn clear_model(&self, account_id: &str, model: &str) -> bool {
        let normalized = crate::proxy::common::model_mapping::normalize_to_standard_id(model)
            .unwrap_or_else(|| model.to_string());
        let mut cleared = self
            .limits
            .remove(&self.get_limit_key(account_id, Some(&normalized)))
            .is_some();
        if normalized != model {
            cleared |= self
                .limits
                .remove(&self.get_limit_key(account_id, Some(model)))
                .is_some();
        }
        cleared
    }

    /// Safely release persistent locks due to quota exhaustion:
    /// - Only clear records whose reason is QuotaExhausted. Unintentional triggering of independent current limits such as 429 rate limit (RateLimitExceeded), server error, etc. is strictly prohibited;
    /// - Accurate comparison for both direct and standard IDs;
    pub fn reconcile_quota_recovery(&self, account_id: &str, model: &str) -> bool {
        let normalized = crate::proxy::common::model_mapping::normalize_to_standard_id(model)
            .unwrap_or_else(|| model.to_string());

        let keys = if normalized != model {
            vec![
                self.get_limit_key(account_id, Some(&normalized)),
                self.get_limit_key(account_id, Some(model)),
            ]
        } else {
            vec![self.get_limit_key(account_id, Some(model))]
        };

        let mut cleared = false;
        for key in keys {
            if let Some(entry) = self.limits.get(&key) {
                if entry.reason == RateLimitReason::QuotaExhausted {
                    drop(entry);
                    if self.limits.remove(&key).is_some() {
                        cleared = true;
                    }
                }
            }
        }
        cleared
    }

    /// Check whether the account is still being restricted
    /// Check whether the account is still restricted (supports model level)
    pub fn is_rate_limited(&self, account_id: &str, model: Option<&str>) -> bool {
        // Checking using get_remaining_wait which handles both global and model keys
        self.get_remaining_wait(account_id, model) > 0
    }

    /// Get the number of seconds until the current limit is reset
    pub fn get_reset_seconds(&self, account_id: &str) -> Option<u64> {
        if let Some(info) = self.get(account_id) {
            info.reset_time
                .duration_since(SystemTime::now())
                .ok()
                .map(|d| d.as_secs())
        } else {
            None
        }
    }

    /// Clear expired current limiting records
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) -> usize {
        let now = SystemTime::now();
        let mut count = 0;

        self.limits.retain(|_k, v| {
            if v.reset_time <= now {
                count += 1;
                false
            } else {
                true
            }
        });

        if count > 0 {
            tracing::debug!("Cleared {} expired throttling records", count);
        }

        count
    }

    /// Clear the current limit record of the specified account
    pub fn clear(&self, account_id: &str) -> bool {
        let prefix = format!("{}:", account_id);
        let before = self.limits.len();
        self.limits
            .retain(|key, _| key != account_id && !key.starts_with(&prefix));
        self.failure_counts.remove(account_id);
        self.limits.len() != before
    }

    pub fn clear_for_optimistic_reset(&self) {
        let now = SystemTime::now();
        self.limits.retain(|_, info| {
            info.reason == RateLimitReason::QuotaExhausted
                && info
                    .reset_time
                    .duration_since(info.detected_at)
                    .is_ok_and(|duration| duration > Duration::from_secs(MAX_LOCKOUT_SECONDS))
                && info.reset_time.duration_since(now).is_ok()
        });
    }

    /// Clear all throttling records (optimistic reset strategy)
    ///
    /// Used for optimistic reset mechanism, when all accounts are restricted but the waiting time is short,
    /// Clear all throttling records to resolve timing race conditions
    pub fn clear_all(&self) {
        let count = self.limits.len();
        self.limits.clear();
        tracing::warn!(
            "🔄 Optimistic reset: Cleared all {} rate limit record(s)",
            count
        );
    }
}

impl Default for RateLimitTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_retry_time_minutes_seconds() {
        let tracker = RateLimitTracker::new();
        let body = "Rate limit exceeded. Try again in 2m 30s";
        let time = tracker.parse_retry_time_from_body(body);
        assert_eq!(time, Some(150));
    }

    #[test]
    fn test_parse_google_json_delay() {
        let tracker = RateLimitTracker::new();
        let body = r#"{
            "error": {
                "details": [
                    { 
                        "metadata": {
                            "quotaResetDelay": "42s" 
                        }
                    }
                ]
            }
        }"#;
        let time = tracker.parse_retry_time_from_body(body);
        assert_eq!(time, Some(42));
    }

    #[test]
    fn test_parse_retry_after_ignore_case() {
        let tracker = RateLimitTracker::new();
        let body = "Quota limit hit. Retry After 99 Seconds";
        let time = tracker.parse_retry_time_from_body(body);
        assert_eq!(time, Some(99));
    }

    #[test]
    fn test_get_remaining_wait() {
        let tracker = RateLimitTracker::new();
        tracker.parse_from_error("acc1", 429, Some("30"), "", None, &[]);
        let wait = tracker.get_remaining_wait("acc1", None);
        assert!(wait > 25 && wait <= 30);
    }

    #[test]
    fn test_safety_buffer() {
        let tracker = RateLimitTracker::new();
        // If the API returns 1s, we force it to 2s
        tracker.parse_from_error("acc1", 429, Some("1"), "", None, &[]);
        let wait = tracker.get_remaining_wait("acc1", None);
        // Due to time passing, it might be 1 or 2
        assert!(wait >= 1 && wait <= 2);
    }

    #[test]
    fn task_preserves_explicit_long_image_quota_deadline() {
        let tracker = RateLimitTracker::new();
        let body = r#"{
            "error": {
                "details": [{
                    "reason": "QUOTA_EXHAUSTED",
                    "metadata": {"quotaResetDelay": "58h24m53s"}
                }]
            }
        }"#;
        let info = tracker
            .parse_from_error(
                "acc-long",
                429,
                None,
                body,
                Some("gemini-3-pro-image".to_string()),
                &[60, 300],
            )
            .unwrap();
        assert_eq!(info.retry_after_sec, 210_293);

        tracker.clear_for_optimistic_reset();
        assert!(tracker.is_rate_limited("acc-long", Some("gemini-3-pro-image")));

        let now = SystemTime::now();
        assert!(tracker.restore_persisted_long_image_limit(
            "acc-expiring",
            now + Duration::from_secs(30),
            now - Duration::from_secs(301),
            "gemini-3.1-flash-image",
        ));
        tracker.clear_for_optimistic_reset();
        assert!(tracker.is_rate_limited("acc-expiring", Some("gemini-3.1-flash-image")));

        let inferred = tracker
            .parse_from_error(
                "acc-inferred",
                429,
                None,
                "Quota limit hit; reset after 72h",
                Some("gemini-3-pro-image".to_string()),
                &[60, 300],
            )
            .unwrap();
        assert_eq!(inferred.retry_after_sec, 300);

        let text_model = tracker
            .parse_from_error(
                "acc-text",
                429,
                Some("72h"),
                r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED"}]}}"#,
                Some("gemini-2.5-pro".to_string()),
                &[60, 300],
            )
            .unwrap();
        assert_eq!(text_model.retry_after_sec, 300);

        let broad_quota = tracker
            .parse_from_error(
                "acc-broad",
                429,
                None,
                "Quota limit hit; reset after 72h",
                Some("gemini-3.1-flash-image".to_string()),
                &[60, 300],
            )
            .unwrap();
        assert_eq!(broad_quota.retry_after_sec, 300);

        let inferred_reset = SystemTime::now() + Duration::from_secs(72 * 3600);
        tracker.set_lockout_until(
            "acc-inferred-reset",
            inferred_reset,
            RateLimitReason::QuotaExhausted,
            Some("gemini-3-pro-image".to_string()),
        );
        assert!(
            tracker.get_remaining_wait("acc-inferred-reset", Some("gemini-3-pro-image")) <= 300
        );
    }

    #[test]
    fn task_claude_baseline_ignores_retry_info_delay() {
        for (index, delay) in ["1s", "3s"].into_iter().enumerate() {
            let body = format!(
                r#"{{"error":{{"details":[{{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"{}"}}]}}}}"#,
                delay
            );
            let current = RateLimitTracker::new()
                .parse_from_error(
                    &format!("current-{}", index),
                    429,
                    None,
                    &body,
                    None,
                    &[60, 300],
                )
                .unwrap();
            assert_eq!(current.retry_after_sec, if index == 0 { 2 } else { 3 });

            let baseline = RateLimitTracker::new()
                .parse_from_error_baseline(
                    &format!("baseline-{}", index),
                    429,
                    None,
                    &body,
                    None,
                    &[60, 300],
                )
                .unwrap();
            assert_eq!(baseline.retry_after_sec, 60);
        }
    }

    #[test]
    fn test_tpm_exhausted_is_rate_limit_exceeded() {
        let tracker = RateLimitTracker::new();
        // Simulates real-world TPM errors, including both "Resource exhausted" and "per minute"
        let body = "Resource has been exhausted (e.g. check quota). Quota limit 'Tokens per minute' exceeded.";
        let reason = tracker.parse_rate_limit_reason(body);
        // Should be recognized as RateLimitExceeded, not QuotaExhausted
        assert_eq!(reason, RateLimitReason::RateLimitExceeded);
    }

    #[test]
    fn test_generic_resource_exhausted_is_short_rate_limit() {
        let tracker = RateLimitTracker::new();
        let body = r#"{
            "error": {
                "code": 429,
                "message": "Resource has been exhausted (e.g. check quota).",
                "status": "RESOURCE_EXHAUSTED"
            }
        }"#;
        let reason = tracker.parse_rate_limit_reason(body);
        assert_eq!(reason, RateLimitReason::RateLimitExceeded);
    }

    #[test]
    fn test_server_error_does_not_accumulate_failure_count() {
        let tracker = RateLimitTracker::new();
        let backoff_steps = vec![60, 300, 1800, 7200];

        // Simulate 5 consecutive 529 (ServerError) errors
        for i in 1..=5 {
            let info = tracker.parse_from_error(
                "acc1",
                529,
                None,
                "Service Overloaded",
                None,
                &backoff_steps,
            );
            assert!(
                info.is_some(),
                "The {} time 529 should return RateLimitInfo",
                i
            );
            let info = info.unwrap();
            // 529 should always lock for 8 seconds regardless of failure_count
            assert_eq!(
                info.retry_after_sec, 8,
                "529 {} time should be locked for 8 seconds",
                i
            );
        }

        // 429 QuotaExhausted is now triggered once (without quotaResetDelay)
        let quota_body = r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED"}]}}"#;
        let info = tracker.parse_from_error("acc1", 429, None, quota_body, None, &backoff_steps);
        assert!(info.is_some());
        let info = info.unwrap();

        // Key assertion: 429 should start from 1st time (lock for 60 seconds) instead of inheriting the count of 5xx
        assert_eq!(
            info.retry_after_sec, 60,
            "429 should start from the 1st backoff (60 seconds) instead of being tainted by 5xx"
        );
    }

    #[test]
    fn test_quota_exhausted_does_accumulate_failure_count() {
        let tracker = RateLimitTracker::new();
        let backoff_steps = vec![60, 300, 1800, 7200];
        let quota_body = r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED"}]}}"#;

        // 1st time 429 → 60 seconds
        let info = tracker.parse_from_error("acc2", 429, None, quota_body, None, &backoff_steps);
        assert_eq!(info.unwrap().retry_after_sec, 60);

        // 2nd time 429 → 300 seconds
        let info = tracker.parse_from_error("acc2", 429, None, quota_body, None, &backoff_steps);
        assert_eq!(info.unwrap().retry_after_sec, 300);

        // 3rd time 429 → 1800 seconds
        let info = tracker.parse_from_error("acc2", 429, None, quota_body, None, &backoff_steps);
        assert_eq!(info.unwrap().retry_after_sec, 1800);

        // 4th time 429 → 7200 seconds
        let info = tracker.parse_from_error("acc2", 429, None, quota_body, None, &backoff_steps);
        assert_eq!(info.unwrap().retry_after_sec, 7200);
    }

    #[test]
    fn test_set_lockout_until_with_cap() {
        let tracker = RateLimitTracker::new();
        let target_time = SystemTime::now() + Duration::from_secs(5 * 3600); // 5 hours

        // Capped: should be capped to 300s
        tracker.set_lockout_until_with_cap(
            "acc_cap",
            target_time,
            RateLimitReason::QuotaExhausted,
            None,
            true,
        );
        let wait_capped = tracker.get_remaining_wait("acc_cap", None);
        assert!(wait_capped <= 300 && wait_capped >= 290);

        // Uncapped (Zero Quota): should retain full 5 hours duration
        tracker.set_lockout_until_with_cap(
            "acc_uncap",
            target_time,
            RateLimitReason::QuotaExhausted,
            None,
            false,
        );
        let wait_uncapped = tracker.get_remaining_wait("acc_uncap", None);
        assert!(wait_uncapped > 300 && wait_uncapped <= 5 * 3600);
    }

    #[test]
    fn test_reconcile_quota_recovery_clears_quota_exhausted_but_keeps_rate_limit_exceeded() {
        let tracker = RateLimitTracker::new();
        let target_time = SystemTime::now() + Duration::from_secs(3600);

        // 1. Set a QuotaExhausted lock
        tracker.set_lockout_until_with_cap(
            "acc_test",
            target_time,
            RateLimitReason::QuotaExhausted,
            Some("claude-sonnet-4-6".to_string()),
            false,
        );
        assert!(tracker.is_rate_limited("acc_test", Some("claude-sonnet-4-6")));

        // Restoring quotas: should be lifted successfully
        assert!(tracker.reconcile_quota_recovery("acc_test", "claude-sonnet-4-6"));
        assert!(!tracker.is_rate_limited("acc_test", Some("claude-sonnet-4-6")));

        // 2. Set a RateLimitExceeded (429 rate limit)
        tracker.set_lockout_until_with_cap(
            "acc_test",
            target_time,
            RateLimitReason::RateLimitExceeded,
            Some("claude-sonnet-4-6".to_string()),
            false,
        );
        assert!(tracker.is_rate_limited("acc_test", Some("claude-sonnet-4-6")));

        // Quota Recovery: Clearing independent 429 rate limits is strictly prohibited!
        assert!(!tracker.reconcile_quota_recovery("acc_test", "claude-sonnet-4-6"));
        assert!(tracker.is_rate_limited("acc_test", Some("claude-sonnet-4-6")));
    }

    #[test]
    fn test_restore_persisted_long_limit_supports_text_models() {
        let tracker = RateLimitTracker::new();
        let now = SystemTime::now();
        let detected_at = now - Duration::from_secs(60);
        let reset_time = now + Duration::from_secs(86400); // 24 hours explicit long lock

        // Text models should successfully recover from long locks
        assert!(tracker.restore_persisted_long_limit(
            "acc_text",
            reset_time,
            detected_at,
            "gemini-2.5-pro",
        ));
        assert!(tracker.is_rate_limited("acc_text", Some("gemini-2.5-pro")));
        // The lock is also detected by the normalized standard ID
        assert!(tracker.is_rate_limited("acc_text", Some("gemini-3-pro-high")));
    }
}
