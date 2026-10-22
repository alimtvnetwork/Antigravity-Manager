//! Quota Period Reset Phase Detector.
//! Categorizes account quota status into 4 distinct lifecycle phases (`FRESH`, `ACTIVE`, `HEADROOM_STRETCH`, `EXHAUSTED`).

use serde::{Deserialize, Serialize};

/// Lifecycle phase of an account's quota cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaLifecyclePhase {
    Fresh,
    Active,
    HeadroomStretch,
    Exhausted,
}

impl QuotaLifecyclePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh => "FRESH",
            Self::Active => "ACTIVE",
            Self::HeadroomStretch => "HEADROOM_STRETCH",
            Self::Exhausted => "EXHAUSTED",
        }
    }
}

/// Detects the operational quota phase based on current percentage and time to reset.
pub fn detect_quota_lifecycle_phase(
    quota_percent: f64,
    seconds_until_reset: Option<i64>,
) -> QuotaLifecyclePhase {
    if quota_percent <= 0.01 {
        return QuotaLifecyclePhase::Exhausted;
    }

    let secs_left = seconds_until_reset.unwrap_or(86400);

    if quota_percent >= 99.0 {
        QuotaLifecyclePhase::Fresh
    } else if quota_percent < 20.0 && secs_left <= 7200 {
        // Less than 20% quota, but cycle resets within 2 hours
        QuotaLifecyclePhase::HeadroomStretch
    } else {
        QuotaLifecyclePhase::Active
    }
}
