//! Auto-Switch Throttle Brake.
//! Enforces a minimum interval between consecutive auto-rotations on the same instance to prevent thrashing.

use serde::{Deserialize, Serialize};

pub const DEFAULT_SWITCH_THROTTLE_SECONDS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleBrakeStatus {
    pub is_throttled: bool,
    pub wait_remaining_seconds: u64,
}

/// Evaluates if an instance rotation request should be throttled.
pub fn check_switch_throttle(
    last_switch_timestamp: i64,
    min_throttle_secs: Option<u64>,
    now_sec: i64,
) -> ThrottleBrakeStatus {
    if last_switch_timestamp <= 0 {
        return ThrottleBrakeStatus {
            is_throttled: false,
            wait_remaining_seconds: 0,
        };
    }

    let min_delay = min_throttle_secs.unwrap_or(DEFAULT_SWITCH_THROTTLE_SECONDS);
    let elapsed = now_sec.saturating_sub(last_switch_timestamp) as u64;

    if elapsed < min_delay {
        ThrottleBrakeStatus {
            is_throttled: true,
            wait_remaining_seconds: min_delay - elapsed,
        }
    } else {
        ThrottleBrakeStatus {
            is_throttled: false,
            wait_remaining_seconds: 0,
        }
    }
}
