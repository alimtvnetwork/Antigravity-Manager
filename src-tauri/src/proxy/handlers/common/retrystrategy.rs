use super::*;

// ===== Unified retry and backoff strategy =====

/// Retry strategy enumeration
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// No retry, return error directly
    NoRetry,
    /// fixed delay
    FixedDelay(Duration),
    /// Linear backoff: base_ms * (attempt + 1)
    LinearBackoff { base_ms: u64 },
    /// Exponential backoff: base_ms * 2^attempt, upper limit max_ms
    ExponentialBackoff { base_ms: u64, max_ms: u64 },
    /// [NEW] Retry in place (Grace Retry): Retry directly after waiting in a small window on the current account, not counted in regular switching
    GraceRetry(Duration),
}

#[derive(Debug, Default)]
pub struct RequestRetryState {
    grace_retried_accounts: HashSet<String>,
}

impl RequestRetryState {
    pub fn determine_strategy(
        &mut self,
        account_id: &str,
        status_code: u16,
        error_text: &str,
        retry_after: Option<&str>,
        retried_without_thinking: bool,
    ) -> RetryStrategy {
        self.determine_strategy_with_grace(
            account_id,
            status_code,
            error_text,
            retry_after,
            retried_without_thinking,
            true,
        )
    }

    pub fn determine_strategy_with_grace(
        &mut self,
        account_id: &str,
        status_code: u16,
        error_text: &str,
        retry_after: Option<&str>,
        retried_without_thinking: bool,
        allow_grace: bool,
    ) -> RetryStrategy {
        self.determine_strategy_adaptive(
            account_id,
            status_code,
            error_text,
            retry_after,
            retried_without_thinking,
            0,
            if allow_grace { 1 } else { 2 },
        )
    }

    /// Adaptive retry strategy decider taking into account multi-account pool cycles and quota reset delays
    pub fn determine_strategy_adaptive(
        &mut self,
        account_id: &str,
        status_code: u16,
        error_text: &str,
        retry_after: Option<&str>,
        retried_without_thinking: bool,
        attempt: usize,
        pool_size: usize,
    ) -> RetryStrategy {
        let allow_grace_retry = !self.grace_retried_accounts.contains(account_id);
        let strategy = determine_retry_strategy_adaptive(
            status_code,
            error_text,
            retry_after,
            retried_without_thinking,
            allow_grace_retry,
            attempt,
            pool_size,
        );
        if matches!(strategy, RetryStrategy::GraceRetry(_)) {
            self.grace_retried_accounts.insert(account_id.to_string());
        }
        strategy
    }
}

pub fn next_rotation_attempt(
    used_attempts: &mut usize,
    max_attempts: usize,
    retry_same_account: bool,
) -> Option<usize> {
    if retry_same_account {
        return used_attempts.checked_sub(1);
    }
    if *used_attempts >= max_attempts {
        return None;
    }

    let attempt = *used_attempts;
    *used_attempts += 1;
    Some(attempt)
}

#[derive(Debug, Default)]
pub struct FailureStatusTracker {
    saw_failure: bool,
    last_non_rate_limit: Option<StatusCode>,
}

impl FailureStatusTracker {
    pub fn record(&mut self, status: StatusCode) {
        self.saw_failure = true;
        if status != StatusCode::TOO_MANY_REQUESTS {
            self.last_non_rate_limit = Some(status);
        }
    }

    pub fn final_status(&self) -> StatusCode {
        self.last_non_rate_limit.unwrap_or_else(|| {
            if self.saw_failure {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_GATEWAY
            }
        })
    }
}

/// Adaptively calculate maximum retry attempts based on account pool size
/// - Single account (pool_size <= 1): allow initial request + 2 backoff retries based on quotaResetDelay (3 total)
/// - Multi-account (pool_size > 1): guarantees whole pool can rotate at least 2 full rounds
///   Minimum 4 attempts, maximum 12 attempts
pub fn calculate_max_retry_attempts(pool_size: usize) -> usize {
    if pool_size <= 1 {
        3
    } else {
        (pool_size * 2).clamp(4, 12)
    }
}

/// Determine retry strategy based on HTTP status code and error text
pub fn determine_retry_strategy(
    status_code: u16,
    error_text: &str,
    retried_without_thinking: bool,
) -> RetryStrategy {
    determine_retry_strategy_with_grace(status_code, error_text, retried_without_thinking, true)
}

pub fn determine_retry_strategy_with_grace(
    status_code: u16,
    error_text: &str,
    retried_without_thinking: bool,
    allow_grace: bool,
) -> RetryStrategy {
    determine_retry_strategy_adaptive(
        status_code,
        error_text,
        None,
        retried_without_thinking,
        allow_grace,
        0,
        if allow_grace { 1 } else { 2 },
    )
}

/// Adaptive multi-account pool and single-account backoff decision logic
pub fn determine_retry_strategy_adaptive(
    status_code: u16,
    error_text: &str,
    retry_after: Option<&str>,
    retried_without_thinking: bool,
    allow_grace_retry: bool,
    attempt: usize,
    pool_size: usize,
) -> RetryStrategy {
    // [DEFENSE] If the error is clearly that the model does not exist, invalid rotation or backoff retries are absolutely prohibited!
    if is_model_not_found_error(status_code, error_text) {
        return RetryStrategy::NoRetry;
    }

    let lower = error_text.to_lowercase();
    match status_code {
        // 400 Error: Retry only if specific Thinking signature fails
        400 if !retried_without_thinking
            && (lower.contains("invalid thought signature")
                || lower.contains("invalid `signature`")
                || lower.contains("invalid signature")
                || lower.contains("thought_signature")
                || lower.contains("thoughtsignature")
                || lower.contains("thinking.signature")
                || lower.contains("thinking.thinking")
                || lower.contains("corrupted thought signature")) =>
        {
            RetryStrategy::FixedDelay(Duration::from_millis(200))
        }

        // 429 Rate limit error
        429 => {
            // 1. First attempt to parse structured or natural language retry delay (quotaResetDelay / Retry-After)
            let parsed_delay = crate::proxy::upstream::retry::parse_retry_delay_with_source(
                error_text,
                retry_after,
            );

            // 2. Real hard quota exhaustion check: only when no delay was returned and definitive exhaustion keywords match
            let is_hard_quota_exhausted = parsed_delay.is_none()
                && (lower.contains("quota_exhausted")
                    || lower.contains("exceeded your current quota")
                    || lower.contains("insufficient_quota")
                    || lower.contains("credits")
                    || lower.contains("zero_quota")
                    || lower.contains("weekly quota"));
            if is_hard_quota_exhausted {
                return RetryStrategy::FixedDelay(Duration::from_millis(50));
            }

            // 3. Single account mode (pool_size <= 1): Unable to switch accounts, waiting is the only option
            if pool_size <= 1 {
                if let Some(delay) = parsed_delay {
                    let actual_ms = delay.actual_wait_ms();
                    if actual_ms <= 30_000 && allow_grace_retry {
                        tracing::info!(
                            "[Retry] Single account 429: quotaResetDelay detected ({}ms), applying GraceRetry",
                            actual_ms
                        );
                        return RetryStrategy::GraceRetry(Duration::from_millis(actual_ms));
                    } else {
                        return RetryStrategy::FixedDelay(Duration::from_millis(
                            actual_ms.min(30_000),
                        ));
                    }
                } else {
                    // Guaranteed backoff when no clear delay is given (wait 3s~5s for a single account, avoid 50ms lightning exhaustion and retry)
                    let backoff_ms = (3000 * (attempt + 1) as u64).min(10_000);
                    tracing::info!(
                        "[Retry] Single account 429 without explicit delay: backing off {}ms",
                        backoff_ms
                    );
                    if allow_grace_retry {
                        return RetryStrategy::GraceRetry(Duration::from_millis(backoff_ms));
                    } else {
                        return RetryStrategy::FixedDelay(Duration::from_millis(backoff_ms));
                    }
                }
            }

            // 4. Multi-account mode (pool_size > 1)
            let is_first_round = attempt < pool_size;
            if is_first_round {
                // Round 1 (first round): Lightning fast cut across the entire pool, escaping to other healthy accounts in milliseconds
                tracing::info!(
                    "[Retry] Multi-account Round 1 (attempt {}/{}): fast rotating to next account in pool (50ms)",
                    attempt + 1, pool_size
                );
                return RetryStrategy::FixedDelay(Duration::from_millis(50));
            }

            // Round 2 (Second round): The entire pool has encountered 429 in the first round and has entered the stage of intelligent retreat and small gap waiting.
            if let Some(delay) = parsed_delay {
                let actual_ms = delay.actual_wait_ms();
                // When encountering a small gap (<= 5s), wait in the current account and try again.
                if actual_ms <= 5000 && allow_grace_retry {
                    tracing::info!(
                        "[Retry] Multi-account Round 2 (attempt {}): small reset gap ({}ms <= 5s), applying GraceRetry",
                        attempt + 1, actual_ms
                    );
                    return RetryStrategy::GraceRetry(Duration::from_millis(actual_ms));
                }

                // The delay is large (> 5s, such as more than 10s) and the number of available accounts is > 2: Give priority to continuing the lightning rotation to look for other accounts that may expire.
                if pool_size > 2 && attempt + 1 < pool_size * 2 {
                    tracing::info!(
                        "[Retry] Multi-account Round 2 (attempt {}): delay is {}ms (> 5s) with pool_size > 2, fast rotating",
                        attempt + 1, actual_ms
                    );
                    return RetryStrategy::FixedDelay(Duration::from_millis(50));
                }

                // Other situations (such as only 2 accounts or the second half of the second round): wait for the delay (upper limit 12s to prevent client timeout)
                let capped_ms = actual_ms.min(12_000);
                tracing::info!(
                    "[Retry] Multi-account Round 2: waiting capped delay {}ms before retry",
                    capped_ms
                );
                return RetryStrategy::FixedDelay(Duration::from_millis(capped_ms));
            }

            // The second round did not resolve the specific delay: linear gentle backoff of 2s~4s to allow the instantaneous concurrency peak to subside.
            let backoff_ms = (2000 * ((attempt.saturating_sub(pool_size)) + 1) as u64).min(5000);
            RetryStrategy::FixedDelay(Duration::from_millis(backoff_ms))
        }

        // 503 Service Unavailable / 529 Server Overloaded
        503 | 529 => {
            if pool_size > 1 && attempt < pool_size {
                // Multi-account first round encounter 503/529: Quickly switch to other healthy accounts (50ms)
                tracing::info!(
                    "[Retry] 503/529 detected in multi-account pool (attempt {}/{}): fast escaping to other accounts",
                    attempt + 1, pool_size
                );
                RetryStrategy::FixedDelay(Duration::from_millis(50))
            } else {
                // Single account or the entire pool has been traversed: exponential backoff (starting 5s, upper limit 30s)
                RetryStrategy::ExponentialBackoff {
                    base_ms: 5000,
                    max_ms: 30000,
                }
            }
        }

        // 500 Server internal error
        500 => {
            // Linear backoff: starting 3s
            RetryStrategy::LinearBackoff { base_ms: 3000 }
        }

        // 401/403 Authentication/permission error: give a very short buffer before switching accounts
        401 | 403 => RetryStrategy::FixedDelay(Duration::from_millis(200)),

        // 404 Resource Not Found: 404 for Google Cloud Code API is usually an intermittent issue at the account level
        // (Grayscale publishing, out-of-sync account permissions, etc.), rotating accounts can often solve the problem
        404 => RetryStrategy::FixedDelay(Duration::from_millis(300)),

        // Other errors: Do not retry
        _ => RetryStrategy::NoRetry,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn task_short_429_preserves_rotation_budget_and_structured_status() {
        let body = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"1s"}]}}"#;

        let drive_failures = |account_count| {
            let mut state = RequestRetryState::default();
            let mut used_attempts = 0;
            let mut retry_same_account = false;
            let mut sends = Vec::new();

            while let Some(attempt) =
                next_rotation_attempt(&mut used_attempts, account_count, retry_same_account)
            {
                retry_same_account = false;
                sends.push(attempt);
                let account_id = format!("account-{}", attempt);
                let strategy = state.determine_strategy(&account_id, 429, body, None, false);
                if matches!(strategy, RetryStrategy::GraceRetry(_)) {
                    assert!(!should_rotate_account(429, Some(&strategy)));
                    retry_same_account = true;
                } else {
                    assert!(should_rotate_account(429, Some(&strategy)));
                }
            }
            sends
        };

        assert_eq!(drive_failures(1), vec![0, 0]);
        assert_eq!(drive_failures(2), vec![0, 0, 1, 1]);

        let mut all_429 = FailureStatusTracker::default();
        all_429.record(StatusCode::TOO_MANY_REQUESTS);
        all_429.record(StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(all_429.final_status(), StatusCode::TOO_MANY_REQUESTS);

        for non_429 in [StatusCode::FORBIDDEN, StatusCode::SERVICE_UNAVAILABLE] {
            let mut mixed = FailureStatusTracker::default();
            mixed.record(non_429);
            mixed.record(StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(mixed.final_status(), non_429);
        }

        let mut all_503 = FailureStatusTracker::default();
        all_503.record(StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(all_503.final_status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

/// Execute the backoff strategy and return whether it should continue to try again
pub async fn apply_retry_strategy(
    strategy: RetryStrategy,
    attempt: usize,
    max_attempts: usize,
    status_code: u16,
    trace_id: &str,
) -> bool {
    match strategy {
        RetryStrategy::NoRetry => {
            debug!(
                "[{}] Non-retryable error {}, stopping",
                trace_id, status_code
            );
            false
        }

        RetryStrategy::FixedDelay(duration) => {
            let base_ms = duration.as_millis() as u64;
            info!(
                "[{}] ⏱️ Retry with fixed delay: status={}, attempt={}/{}, delay={}ms",
                trace_id,
                status_code,
                attempt + 1,
                max_attempts,
                base_ms
            );
            sleep(duration).await;
            true
        }

        RetryStrategy::LinearBackoff { base_ms } => {
            let calculated_ms = base_ms * (attempt as u64 + 1);
            info!(
                "[{}] ⏱️ Retry with linear backoff: status={}, attempt={}/{}, delay={}ms",
                trace_id,
                status_code,
                attempt + 1,
                max_attempts,
                calculated_ms
            );
            sleep(Duration::from_millis(calculated_ms)).await;
            true
        }

        RetryStrategy::ExponentialBackoff { base_ms, max_ms } => {
            let calculated_ms = (base_ms * 2_u64.pow(attempt as u32)).min(max_ms);
            info!(
                "[{}] ⏱️ Retry with exponential backoff: status={}, attempt={}/{}, delay={}ms",
                trace_id,
                status_code,
                attempt + 1,
                max_attempts,
                calculated_ms
            );
            sleep(Duration::from_millis(calculated_ms)).await;
            true
        }

        RetryStrategy::GraceRetry(duration) => {
            info!(
                "[{}] ⚡ Grace Retry: Performing micro-wait ({}ms) on current account...",
                trace_id,
                duration.as_millis()
            );
            sleep(duration).await;
            true // Retry in place and use should_rotate_account at the handlers level to determine whether to switch.
        }
    }
}

/// Determine whether accounts should be rotated
pub fn should_rotate_account(status_code: u16, strategy: Option<&RetryStrategy>) -> bool {
    // [NEW] If identified as Grace Retry, explicitly request not to rotate the account
    if let Some(RetryStrategy::GraceRetry(_)) = strategy {
        return false;
    }

    match status_code {
        // These errors are account level or specific node quota and need to be rotated
        // [FIX #3485] 503/529 Edge node meltdown or specific account load is too high. Support rotation escape when multiple accounts are used to prevent deadlocks, etc.
        429 | 401 | 403 | 404 | 500 | 503 | 529 => true,
        _ => false,
    }
}
