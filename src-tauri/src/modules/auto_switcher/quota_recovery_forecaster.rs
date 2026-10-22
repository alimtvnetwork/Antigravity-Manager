//! Candidate Quota Recovery Forecaster (Imp-543)
//! Predicts remaining duration and future timestamp until an account's depleted quota refreshes.
//! Integrates 5-hour rolling windows and 7-day weekly reset boundaries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaRecoveryForecast {
    pub account_id: String,
    pub current_quota_percent: f64,
    pub reset_epoch_secs: i64,
    pub seconds_until_recovery: i64,
    pub hours_until_recovery: f64,
    pub is_recovery_imminent: bool,
}

pub struct QuotaRecoveryForecaster;

impl QuotaRecoveryForecaster {
    /// Forecasts the recovery duration until the given reset timestamp.
    /// If reset_epoch <= current_epoch, recovery is already reached (0.0 hours remaining, imminent).
    pub fn forecast_recovery(
        account_id: &str,
        current_quota_percent: f64,
        reset_epoch_secs: i64,
        current_epoch_secs: i64,
    ) -> QuotaRecoveryForecast {
        let diff_secs = (reset_epoch_secs - current_epoch_secs).max(0);
        let hours_until_recovery = ((diff_secs as f64 / 3600.0) * 100.0).round() / 100.0;
        let is_recovery_imminent = diff_secs <= 900; // <= 15 minutes

        QuotaRecoveryForecast {
            account_id: account_id.to_string(),
            current_quota_percent: current_quota_percent.clamp(0.0, 100.0),
            reset_epoch_secs,
            seconds_until_recovery: diff_secs,
            hours_until_recovery,
            is_recovery_imminent,
        }
    }
}
