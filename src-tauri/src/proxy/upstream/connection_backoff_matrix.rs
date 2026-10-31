// Upstream Connection Exponential Backoff Matrix (Imp-512)
// Calculates backoff duration with full jitter for retryable upstream errors,
// avoiding synchronized retry storms against upstream model providers.

use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BackoffMatrixConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for BackoffMatrixConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter_factor: 0.25,
        }
    }
}

pub struct ConnectionBackoffMatrix {
    config: BackoffMatrixConfig,
}

impl Default for ConnectionBackoffMatrix {
    fn default() -> Self {
        Self::new(BackoffMatrixConfig::default())
    }
}

impl ConnectionBackoffMatrix {
    pub fn new(config: BackoffMatrixConfig) -> Self {
        Self { config }
    }

    /// Computes backoff duration for the given attempt count (0-indexed).
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base_millis = self.config.initial_delay.as_millis() as f64
            * self.config.multiplier.powi(attempt as i32);
        let max_millis = self.config.max_delay.as_millis() as f64;
        let clamped_millis = base_millis.min(max_millis);

        // Apply full jitter within [clamped * (1 - jitter), clamped * (1 + jitter)]
        let mut rng = rand::thread_rng();
        let jitter_range = clamped_millis * self.config.jitter_factor;
        let lower = (clamped_millis - jitter_range).max(0.0);
        let upper = clamped_millis + jitter_range;
        let final_millis = rng.gen_range(lower..=upper);

        Duration::from_millis(final_millis as u64)
    }
}
