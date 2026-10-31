//! Consecutive Rotation Frequency Brake.
//! Limits account switch frequency (default max 5/hr) to guard against rapid flip-flopping.

use serde::{Deserialize, Serialize};

/// Default switch rate limit: maximum 5 switches per 1-hour window.
pub const DEFAULT_MAX_SWITCHES_PER_HOUR: usize = 5;

/// Diagnostic check result from rotation frequency evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationFrequencyCheck {
    pub can_rotate: bool,
    pub switches_in_past_hour: usize,
    pub max_allowed_per_hour: usize,
}

/// Evaluates if an account has exceeded the hourly switch frequency threshold.
pub fn check_rotation_frequency(
    switch_timestamps_epoch: &[i64],
    now_epoch: i64,
    max_switches: Option<usize>,
) -> RotationFrequencyCheck {
    let limit = max_switches.unwrap_or(DEFAULT_MAX_SWITCHES_PER_HOUR);
    let one_hour_ago = now_epoch - 3600;

    let recent_count = switch_timestamps_epoch
        .iter()
        .filter(|&&ts| ts >= one_hour_ago)
        .count();

    let can_rotate = recent_count < limit;

    RotationFrequencyCheck {
        can_rotate,
        switches_in_past_hour: recent_count,
        max_allowed_per_hour: limit,
    }
}
