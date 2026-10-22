//! Jittered Quota Velocity Estimator.
//! Adds bounded micro-jitter to account consumption velocity estimates,
//! desynchronizing multiple parallel instances from simultaneously rotating at the same threshold.

use std::time::{Duration, Instant};

/// Sample of quota consumption.
#[derive(Debug, Clone)]
pub struct QuotaConsumptionSample {
    pub recorded_at: Instant,
    pub used_ratio: f64,
}

/// Estimates quota consumption velocity with intentional micro-jitter.
pub fn estimate_jittered_velocity(
    samples: &[QuotaConsumptionSample],
    observation_window: Duration,
    jitter_percent: f64,
) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }

    let now = Instant::now();
    let recent_samples: Vec<&QuotaConsumptionSample> = samples
        .iter()
        .filter(|s| now.duration_since(s.recorded_at) <= observation_window)
        .collect();

    if recent_samples.len() < 2 {
        return 0.0;
    }

    let oldest = recent_samples.first().unwrap();
    let newest = recent_samples.last().unwrap();

    let delta_ratio = (newest.used_ratio - oldest.used_ratio).max(0.0);
    let delta_secs = newest.recorded_at.duration_since(oldest.recorded_at).as_secs_f64();

    if delta_secs <= 0.0 {
        return 0.0;
    }

    let raw_velocity = delta_ratio / (delta_secs / 3600.0); // Ratio per hour

    // Apply bounded pseudo-random jitter factor [-jitter_percent, +jitter_percent]
    let bounded_jitter = jitter_percent.clamp(0.01, 0.20);
    let random_factor = (fastrand::f64() * 2.0) - 1.0;
    let jittered = raw_velocity * (1.0 + (random_factor * bounded_jitter));

    jittered.max(0.0)
}
