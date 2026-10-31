//! Quota polling jitter shield.
//! Adds controlled pseudo-random variation (default ±10%) to background intervals
//! to prevent synchronous request bursts across multi-instance clusters.

use std::time::Duration;

/// Applies pseudo-randomized jitter to a duration within the range `[-jitter_percent, +jitter_percent]`.
/// Minimum bound is clamped to 1 second.
pub fn apply_jitter(base_duration: Duration, jitter_percent: f64) -> Duration {
    let base_ms = base_duration.as_millis() as f64;
    let clamped_jitter = jitter_percent.abs().clamp(0.0, 50.0);

    // Fast xorshift pseudo-random generator based on nanosecond clock
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let mut state = nanos.wrapping_add(1);
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;

    // Map into range [-1.0, 1.0]
    let factor = ((state % 2001) as f64 / 1000.0) - 1.0;
    let delta = base_ms * (clamped_jitter / 100.0) * factor;
    let final_ms = (base_ms + delta).max(1000.0);

    Duration::from_millis(final_ms as u64)
}

/// Applies standard ±10% jitter shield to the given interval.
pub fn apply_standard_jitter(base_duration: Duration) -> Duration {
    apply_jitter(base_duration, 10.0)
}
