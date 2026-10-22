//! Upstream Retry Delay with Decorrelated Exponential Jitter.
//! Computes optimal wait durations for 429 RateLimit and 503 Overloaded backoffs.

use std::time::Duration;

pub const DEFAULT_RETRY_BASE_MS: u64 = 500;
pub const DEFAULT_RETRY_MAX_MS: u64 = 30_000;

/// Computes backoff duration scaled exponentially by attempt count with pseudo-random jitter.
pub fn calculate_retry_delay_with_jitter(
    attempt: u32,
    base_ms: Option<u64>,
    max_ms: Option<u64>,
) -> Duration {
    let base = base_ms.unwrap_or(DEFAULT_RETRY_BASE_MS);
    let max = max_ms.unwrap_or(DEFAULT_RETRY_MAX_MS);

    let factor = 2u64.saturating_pow(attempt.min(6));
    let uncapped = base.saturating_mul(factor);
    let exponential = uncapped.min(max);

    // Add +/- 15% pseudo-randomized jitter
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let jitter_ratio = ((nanos % 301) as f64 / 1000.0) - 0.15; // -0.15 to +0.15
    let jitter_ms = (exponential as f64 * jitter_ratio) as i64;
    let final_ms = ((exponential as i64) + jitter_ms).clamp(100, max as i64) as u64;

    Duration::from_millis(final_ms)
}
