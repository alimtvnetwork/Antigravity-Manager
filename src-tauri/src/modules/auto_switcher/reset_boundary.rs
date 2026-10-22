//! Headroom Cycle Reset Boundary Inspector.
//! Detects when a weekly quota cycle has reset within the recent boundary window, granting fresh account boost scores.

use serde::{Deserialize, Serialize};

/// Default boundary window: 300 seconds (5 minutes).
pub const DEFAULT_RESET_BOUNDARY_WINDOW_SECS: i64 = 300;
/// Fresh reset boost score bonus added to newly reset accounts.
pub const RECENT_RESET_BOOST_SCORE: f64 = 25.0;

/// Diagnostic result of a reset boundary inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetBoundaryCheck {
    pub has_reset_recently: bool,
    pub seconds_since_reset: i64,
    pub reset_boost_score: f64,
}

/// Evaluates if an account's quota reset timestamp indicates a recently refreshed cycle.
pub fn inspect_reset_boundary(
    reset_timestamp: Option<i64>,
    now_epoch: i64,
    window_secs: Option<i64>,
) -> ResetBoundaryCheck {
    let window = window_secs.unwrap_or(DEFAULT_RESET_BOUNDARY_WINDOW_SECS);

    let reset_ts = match reset_timestamp {
        Some(ts) => ts,
        None => {
            return ResetBoundaryCheck {
                has_reset_recently: false,
                seconds_since_reset: -1,
                reset_boost_score: 0.0,
            };
        }
    };

    // If reset_ts has passed and occurred within [now - window, now]
    let diff = now_epoch - reset_ts;
    let has_reset_recently = diff >= 0 && diff <= window;

    let reset_boost_score = if has_reset_recently {
        // Linearly decay boost from RECENT_RESET_BOOST_SCORE to 0 across the window
        let ratio = 1.0 - (diff as f64 / window as f64).clamp(0.0, 1.0);
        (RECENT_RESET_BOOST_SCORE * ratio * 100.0).round() / 100.0
    } else {
        0.0
    };

    ResetBoundaryCheck {
        has_reset_recently,
        seconds_since_reset: diff,
        reset_boost_score,
    }
}
