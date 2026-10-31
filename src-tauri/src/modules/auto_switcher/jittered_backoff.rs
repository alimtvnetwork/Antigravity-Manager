//! Decorrelated Jitter Backoff Calculator
//! Implements AWS-style decorrelated jitter backoff algorithms
//! to prevent thundering-herd reconnect storms when multiple accounts encounter rate limits.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct JitteredBackoff {
    base_backoff: Duration,
    max_backoff: Duration,
}

impl Default for JitteredBackoff {
    fn default() -> Self {
        Self::new(Duration::from_millis(500), Duration::from_secs(30))
    }
}

impl JitteredBackoff {
    pub fn new(base_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            base_backoff,
            max_backoff,
        }
    }

    /// Computes full jitter: uniform random in [0, min(max_backoff, base * 2^attempt)]
    pub fn calculate_full_jitter(&self, attempt: u32, seed: u64) -> Duration {
        let base_ms = self.base_backoff.as_millis() as f64;
        let max_ms = self.max_backoff.as_millis() as f64;
        let exp = (2.0f64).powi(attempt.min(10) as i32);
        let cap = (base_ms * exp).min(max_ms);

        // Deterministic pseudo-random fraction based on seed and attempt
        let hash = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(attempt as u64);
        let fraction = ((hash % 1000) as f64) / 1000.0;
        let jittered_ms = (fraction * cap).max(base_ms);

        Duration::from_millis(jittered_ms as u64)
    }

    /// Computes decorrelated jitter: min(max_backoff, random(base, prev_sleep * 3))
    pub fn calculate_decorrelated_jitter(&self, prev_sleep: Duration, seed: u64) -> Duration {
        let base_ms = self.base_backoff.as_millis() as f64;
        let max_ms = self.max_backoff.as_millis() as f64;
        let prev_ms = prev_sleep.as_millis() as f64;

        let high = (prev_ms * 3.0).min(max_ms);
        let low = base_ms.min(high);

        let hash = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let fraction = ((hash % 1000) as f64) / 1000.0;
        let sleep_ms = low + (fraction * (high - low));

        Duration::from_millis(sleep_ms as u64)
    }
}
