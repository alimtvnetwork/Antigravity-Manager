//! Account Quota Change Velocity Forecaster.
//! Projects estimated time until account reaches low-quota threshold based on recent consumption slope (% per minute).

use serde::{Deserialize, Serialize};

/// High consumption velocity flag threshold (% per minute).
pub const RAPID_DRAIN_THRESHOLD_PCT_PER_MIN: f64 = 1.5;

/// Historic quota observation data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaDataPoint {
    pub timestamp_sec: i64,
    pub quota_percent: f64,
}

/// Velocity and depletion time forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaForecast {
    pub burn_rate_pct_per_min: f64,
    pub estimated_minutes_to_threshold: Option<f64>,
    pub is_rapid_drain: bool,
    pub sample_count: usize,
}

/// Calculates consumption slope over historical points and projects minutes until target threshold is hit.
pub fn forecast_quota_depletion(
    history: &[QuotaDataPoint],
    current_quota: f64,
    threshold: f64,
) -> QuotaForecast {
    if history.len() < 2 {
        return QuotaForecast {
            burn_rate_pct_per_min: 0.0,
            estimated_minutes_to_threshold: None,
            is_rapid_drain: false,
            sample_count: history.len(),
        };
    }

    // Sort by timestamp ascending
    let mut sorted = history.to_vec();
    sorted.sort_by_key(|p| p.timestamp_sec);

    let first = &sorted[0];
    let last = &sorted[sorted.len() - 1];

    let time_delta_sec = last.timestamp_sec - first.timestamp_sec;
    if time_delta_sec <= 0 {
        return QuotaForecast {
            burn_rate_pct_per_min: 0.0,
            estimated_minutes_to_threshold: None,
            is_rapid_drain: false,
            sample_count: sorted.len(),
        };
    }

    // Positive delta means quota decreased (consumed)
    let quota_delta = first.quota_percent - last.quota_percent;
    let time_delta_min = time_delta_sec as f64 / 60.0;
    let burn_rate_pct_per_min = (quota_delta / time_delta_min).max(0.0);

    let is_rapid_drain = burn_rate_pct_per_min >= RAPID_DRAIN_THRESHOLD_PCT_PER_MIN;

    let estimated_minutes_to_threshold =
        if burn_rate_pct_per_min > 0.01 && current_quota > threshold {
            let quota_headroom = current_quota - threshold;
            let mins = quota_headroom / burn_rate_pct_per_min;
            Some((mins * 10.0).round() / 10.0)
        } else {
            None
        };

    QuotaForecast {
        burn_rate_pct_per_min: (burn_rate_pct_per_min * 1000.0).round() / 1000.0,
        estimated_minutes_to_threshold,
        is_rapid_drain,
        sample_count: sorted.len(),
    }
}
