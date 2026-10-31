//! Headroom Velocity Trend Forecaster.
//! Analyzes multi-point headroom trends to detect accelerating quota drain before critical exhaustion.

use serde::{Deserialize, Serialize};

/// Headroom trend analysis outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadroomTrend {
    pub is_accelerating_drain: bool,
    pub drain_rate_hours_per_day: f64,
    pub projected_hours_remaining: f64,
    pub sample_count: usize,
}

/// Analyzes chronological headroom data points (epoch, remaining_hours) to forecast drain trajectory.
pub fn analyze_headroom_trend(points: &[(i64, f64)], _now_epoch: i64) -> HeadroomTrend {
    if points.len() < 2 {
        let last_hours = points.last().map(|(_, h)| *h).unwrap_or(168.0);
        return HeadroomTrend {
            is_accelerating_drain: false,
            drain_rate_hours_per_day: 0.0,
            projected_hours_remaining: last_hours,
            sample_count: points.len(),
        };
    }

    let first = &points[0];
    let last = &points[points.len() - 1];

    let time_span_secs = (last.0 - first.0).max(1);
    let headroom_diff = first.1 - last.1; // positive means headroom decreased

    let time_span_days = time_span_secs as f64 / 86400.0;
    let drain_rate_hours_per_day = (headroom_diff / time_span_days).max(0.0);

    // Natural weekly time decay is 24 hours per day. If drain rate > 36 hours/day, drain is accelerating.
    let is_accelerating_drain = drain_rate_hours_per_day > 36.0;

    let projected_hours_remaining = if drain_rate_hours_per_day > 0.1 {
        let days_left = last.1 / drain_rate_hours_per_day;
        ((days_left * 24.0) * 10.0).round() / 10.0
    } else {
        last.1
    };

    HeadroomTrend {
        is_accelerating_drain,
        drain_rate_hours_per_day: (drain_rate_hours_per_day * 100.0).round() / 100.0,
        projected_hours_remaining,
        sample_count: points.len(),
    }
}
