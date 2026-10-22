//! Short-Term Quota Velocity Spike Dampener
//!
//! Evaluates short-term (1-minute) quota consumption velocity against long-term (15-minute)
//! baselines, applying exponential dampening to transient velocity spikes so that single
//! large multimodal prompts do not prematurely trip automatic account switching.

use tracing::debug;

/// Maximum allowed ratio between short-term and baseline velocity before dampening is applied.
pub const SPIKE_THRESHOLD_RATIO: f64 = 3.0;

/// Dampening factor applied to reduce the influence of transient spikes.
pub const DAMPENING_FACTOR: f64 = 0.4;

/// Result of evaluating velocity spike dampening.
#[derive(Debug, Clone, PartialEq)]
pub struct DampenedVelocity {
    pub raw_short_term_velocity: f64,
    pub baseline_velocity: f64,
    pub effective_velocity: f64,
    pub was_dampened: bool,
}

/// Dampens short-term velocity if it exceeds the baseline by the spike ratio threshold.
pub fn dampen_velocity_spike(
    short_term_velocity: f64,
    baseline_velocity: f64,
) -> DampenedVelocity {
    // If baseline is effectively zero, clamp short-term velocity to safe default ceiling
    let effective_baseline = baseline_velocity.max(0.01);
    let ratio = short_term_velocity / effective_baseline;

    if ratio > SPIKE_THRESHOLD_RATIO {
        // Apply smooth logarithmic dampening above the threshold
        let excess = short_term_velocity - (effective_baseline * SPIKE_THRESHOLD_RATIO);
        let dampened_excess = excess * DAMPENING_FACTOR;
        let effective = (effective_baseline * SPIKE_THRESHOLD_RATIO) + dampened_excess;

        debug!(
            raw = short_term_velocity,
            baseline = baseline_velocity,
            dampened = effective,
            ratio = ratio,
            "Dampened transient quota velocity spike"
        );

        DampenedVelocity {
            raw_short_term_velocity: short_term_velocity,
            baseline_velocity,
            effective_velocity: effective,
            was_dampened: true,
        }
    } else {
        DampenedVelocity {
            raw_short_term_velocity: short_term_velocity,
            baseline_velocity,
            effective_velocity: short_term_velocity,
            was_dampened: false,
        }
    }
}
