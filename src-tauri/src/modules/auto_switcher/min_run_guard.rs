//! Auto-Switch Minimum Run Duration Guard.
//! Enforces a minimum active run duration (default 60s) before an account can be rotated, preventing thrashing.

use serde::{Deserialize, Serialize};

/// Default minimum active execution duration: 60 seconds.
pub const DEFAULT_MIN_RUN_DURATION_SECS: i64 = 60;

/// Diagnostic check result for minimum run duration gating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinRunGuardCheck {
    pub can_switch: bool,
    pub elapsed_secs: i64,
    pub required_secs: i64,
    pub remaining_secs: i64,
}

/// Evaluates if an account has been actively running long enough to be eligible for rotation.
pub fn check_min_run_duration(
    started_running_epoch: Option<i64>,
    now_epoch: i64,
    min_duration_secs: Option<i64>,
) -> MinRunGuardCheck {
    let required = min_duration_secs.unwrap_or(DEFAULT_MIN_RUN_DURATION_SECS);

    let started = match started_running_epoch {
        Some(epoch) => epoch,
        None => {
            // If never recorded when started, allow rotation
            return MinRunGuardCheck {
                can_switch: true,
                elapsed_secs: required,
                required_secs: required,
                remaining_secs: 0,
            };
        }
    };

    let elapsed = now_epoch - started;
    let can_switch = elapsed >= required;
    let remaining = if can_switch { 0 } else { required - elapsed };

    MinRunGuardCheck {
        can_switch,
        elapsed_secs: elapsed,
        required_secs: required,
        remaining_secs: remaining,
    }
}
