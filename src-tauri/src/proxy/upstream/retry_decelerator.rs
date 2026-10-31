//! Adaptive Retry Delay Decelerator.
//! Adjusts retry backoff durations dynamically based on observed upstream error frequency,
//! applying progressive deceleration to protect struggling backend servers.

use std::time::Duration;

/// Configuration for adaptive retry deceleration.
#[derive(Debug, Clone, Copy)]
pub struct DeceleratorConfig {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for DeceleratorConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter_factor: 0.25,
        }
    }
}

/// Computes the decelerated retry delay for a given retry attempt index and endpoint failure count.
pub fn compute_decelerated_delay(
    attempt: u32,
    rolling_endpoint_failures: u32,
    config: Option<DeceleratorConfig>,
) -> Duration {
    let cfg = config.unwrap_or_default();

    // Standard exponential backoff: base * multiplier^attempt
    let exp_factor = cfg.multiplier.powi(attempt.min(5) as i32);
    let mut delay_ms = cfg.base_delay.as_millis() as f64 * exp_factor;

    // Failure pressure penalty: extra 20% delay per rolling failure
    let pressure_factor = 1.0 + (rolling_endpoint_failures as f64 * 0.20);
    delay_ms *= pressure_factor;

    let max_ms = cfg.max_delay.as_millis() as f64;
    let clamped_ms = delay_ms.min(max_ms);

    // Apply bounded pseudo-random jitter: [-jitter_factor, +jitter_factor]
    let jitter_range = clamped_ms * cfg.jitter_factor;
    let pseudo_rand = (fastrand::f64() * 2.0) - 1.0; // [-1.0, 1.0]
    let jittered_ms = (clamped_ms + (pseudo_rand * jitter_range)).max(100.0);

    Duration::from_millis(jittered_ms as u64)
}
