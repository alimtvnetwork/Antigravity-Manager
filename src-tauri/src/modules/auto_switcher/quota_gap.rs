//! Rolling 5-Hour vs Weekly Quota Gap Calculator.
//! Quantifies disparity between rolling 5h window and weekly quota to prefer accounts with balanced headroom.

use serde::{Deserialize, Serialize};

/// Assessment detailing alignment between 5h and weekly quota availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaGapAssessment {
    pub rolling_5h_pct: f64,
    pub weekly_pct: f64,
    pub disparity_pct: f64,
    pub balance_factor: f64, // 1.0 = perfectly balanced, 0.0 = maximal disparity
}

/// Evaluates the quota gap and computes a balance multiplier factor.
pub fn assess_quota_gap(rolling_5h_pct: f64, weekly_pct: f64) -> QuotaGapAssessment {
    let q5 = rolling_5h_pct.clamp(0.0, 100.0);
    let qw = weekly_pct.clamp(0.0, 100.0);

    let disparity_pct = (q5 - qw).abs();
    let balance_factor = (1.0 - (disparity_pct / 100.0)).clamp(0.0, 1.0);

    QuotaGapAssessment {
        rolling_5h_pct: q5,
        weekly_pct: qw,
        disparity_pct: (disparity_pct * 100.0).round() / 100.0,
        balance_factor: (balance_factor * 1000.0).round() / 1000.0,
    }
}
