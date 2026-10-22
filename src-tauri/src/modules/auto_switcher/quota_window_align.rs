//! Candidate Quota Window Align Checker.
//! Evaluates the time remaining until an account's 5-hour rolling window reset,
//! giving bonus scoring priority to accounts that will replenish earlier.

use std::time::Duration;

/// Result of evaluating window alignment.
#[derive(Debug, Clone, Copy)]
pub struct WindowAlignmentScore {
    pub seconds_to_reset: u64,
    pub bonus_score: f64,
}

/// Evaluates bonus score based on seconds remaining until the 5h quota reset.
/// Accounts resetting sooner (<30m) receive higher priority so their replenishing quota is capitalized on.
pub fn evaluate_window_reset_bonus(seconds_to_reset: u64) -> WindowAlignmentScore {
    let five_hours_secs = 5 * 3600;
    let clamped_secs = seconds_to_reset.min(five_hours_secs);

    // If reset is within 15 minutes, award max bonus
    let bonus_score = if clamped_secs <= 900 {
        10.0
    } else if clamped_secs <= 3600 {
        // Linear scale between 15m and 1h (10.0 down to 5.0)
        let factor = 1.0 - ((clamped_secs - 900) as f64 / 2700.0);
        5.0 + (5.0 * factor)
    } else {
        // Gradual falloff for resets further away
        let factor = 1.0 - ((clamped_secs - 3600) as f64 / (five_hours_secs - 3600) as f64);
        (5.0 * factor).max(0.0)
    };

    WindowAlignmentScore {
        seconds_to_reset: clamped_secs,
        bonus_score,
    }
}
