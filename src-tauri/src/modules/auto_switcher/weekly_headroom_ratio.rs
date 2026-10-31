//! Weekly Quota-to-Time Headroom Ratio Calculator
//! Compares remaining weekly quota against the linear ideal consumption slope
//! to favor accounts that have surplus quota relative to remaining weekly hours.

pub struct WeeklyHeadroomRatio;

impl WeeklyHeadroomRatio {
    pub const TOTAL_WEEKLY_HOURS: f64 = 168.0; // 7 days * 24 hours
    pub const IDEAL_HOURLY_BURN_PCT: f64 = 100.0 / Self::TOTAL_WEEKLY_HOURS; // ~0.595% per hour

    /// Calculates the headroom ratio.
    /// Ratio > 1.0 = Account is ahead of schedule (surplus weekly quota).
    /// Ratio < 1.0 = Account has consumed more quota than time elapsed (deficit).
    pub fn calculate_ratio(weekly_quota_pct: f64, hours_remaining: f64) -> f64 {
        let clamped_hours = hours_remaining.clamp(1.0, Self::TOTAL_WEEKLY_HOURS);
        let expected_quota_pct = clamped_hours * Self::IDEAL_HOURLY_BURN_PCT;

        let quota = weekly_quota_pct.max(0.0);
        if expected_quota_pct > 0.0 {
            quota / expected_quota_pct
        } else {
            1.0
        }
    }

    /// Computes a score multiplier [0.5, 1.5] based on the weekly headroom ratio.
    pub fn headroom_multiplier(weekly_quota_pct: f64, hours_remaining: f64) -> f64 {
        let ratio = Self::calculate_ratio(weekly_quota_pct, hours_remaining);
        // Map ratio into [0.5, 1.5] multiplier
        (0.5 + (ratio * 0.5)).clamp(0.5, 1.5)
    }
}
