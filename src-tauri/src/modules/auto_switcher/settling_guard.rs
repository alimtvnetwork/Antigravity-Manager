//! Switch Settling Guard State Validator.
//! Checks whether target instance is within the 10-second post-switch settling window, suppressing duplicate rotation triggers.

use serde::{Deserialize, Serialize};

/// Standard post-switch settling window: 10 seconds.
pub const DEFAULT_SETTLING_WINDOW_SECS: i64 = 10;

/// Diagnostic status of the settling guard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlingGuardStatus {
    pub is_settling: bool,
    pub elapsed_secs: i64,
    pub remaining_secs: i64,
    pub window_secs: i64,
}

/// Evaluates if the system is currently within the post-switch settling window.
pub fn is_in_settling_window(
    last_switch_epoch: Option<i64>,
    now_epoch: i64,
    window_secs: Option<i64>,
) -> bool {
    let window = window_secs.unwrap_or(DEFAULT_SETTLING_WINDOW_SECS);
    match last_switch_epoch {
        Some(last_epoch) => {
            let elapsed = now_epoch - last_epoch;
            elapsed >= 0 && elapsed < window
        }
        None => false,
    }
}

/// Computes remaining seconds in the settling window, or 0 if already settled.
pub fn remaining_settling_secs(
    last_switch_epoch: Option<i64>,
    now_epoch: i64,
    window_secs: Option<i64>,
) -> i64 {
    let window = window_secs.unwrap_or(DEFAULT_SETTLING_WINDOW_SECS);
    match last_switch_epoch {
        Some(last_epoch) => {
            let elapsed = now_epoch - last_epoch;
            if elapsed >= 0 && elapsed < window {
                window - elapsed
            } else {
                0
            }
        }
        None => 0,
    }
}

/// Returns a comprehensive status report for the settling guard.
pub fn get_settling_guard_status(
    last_switch_epoch: Option<i64>,
    now_epoch: i64,
    window_secs: Option<i64>,
) -> SettlingGuardStatus {
    let window = window_secs.unwrap_or(DEFAULT_SETTLING_WINDOW_SECS);
    let (is_settling, elapsed_secs, remaining_secs) = match last_switch_epoch {
        Some(last_epoch) => {
            let elapsed = now_epoch - last_epoch;
            let settling = elapsed >= 0 && elapsed < window;
            let remaining = if settling { window - elapsed } else { 0 };
            (settling, elapsed, remaining)
        }
        None => (false, -1, 0),
    };

    SettlingGuardStatus {
        is_settling,
        elapsed_secs,
        remaining_secs,
        window_secs: window,
    }
}
