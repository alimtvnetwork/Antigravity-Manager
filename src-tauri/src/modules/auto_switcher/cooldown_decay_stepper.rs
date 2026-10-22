//! Stepped Exponential Cooldown Decay
//!
//! Applies progressive stepped decay to cooldown durations as accounts sit idle
//! without triggering further quota limits or transport errors, enabling faster
//! recovery of transiently rate-limited accounts back into the rotation pool.

use std::time::{Duration, Instant};
use tracing::debug;

/// Half-life interval for cooldown decay (default 5 minutes).
pub const DEFAULT_DECAY_HALF_LIFE: Duration = Duration::from_secs(300);

/// Minimum residual cooldown below which cooldown is considered fully cleared.
pub const COOLDOWN_CLEAR_FLOOR_SECS: u64 = 5;

/// Calculates remaining cooldown with stepped exponential decay based on idle time.
pub fn calculate_stepped_cooldown_decay(
    initial_duration: Duration,
    cooldown_started_at: Instant,
    last_error_at: Instant,
    now: Instant,
) -> Duration {
    let total_elapsed = now.duration_since(cooldown_started_at);
    if total_elapsed >= initial_duration {
        return Duration::ZERO;
    }

    let remaining_linear = initial_duration - total_elapsed;

    // Measure idle time since the last recorded error
    let idle_time = now.duration_since(last_error_at);

    // Number of completed half-life periods of clean idle operation
    let half_lives = (idle_time.as_secs_f64() / DEFAULT_DECAY_HALF_LIFE.as_secs_f64()).floor();

    if half_lives <= 0.0 {
        return remaining_linear;
    }

    // Apply 50% decay step for each clean half-life
    let decay_factor = 0.5_f64.powf(half_lives);
    let decayed_secs = (remaining_linear.as_secs_f64() * decay_factor).round() as u64;

    if decayed_secs <= COOLDOWN_CLEAR_FLOOR_SECS {
        debug!(
            initial_secs = initial_duration.as_secs(),
            half_lives = half_lives,
            "Cooldown fully cleared via stepped decay"
        );
        Duration::ZERO
    } else {
        debug!(
            remaining_linear = remaining_linear.as_secs(),
            decayed = decayed_secs,
            half_lives = half_lives,
            "Applied stepped cooldown decay"
        );
        Duration::from_secs(decayed_secs)
    }
}
