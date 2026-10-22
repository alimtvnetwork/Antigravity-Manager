//! Quota consumption acceleration detector.
//! Computes the second derivative of quota usage over time to preemptively detect
//! runaway prompt recursion and burst loops before hard rate limits trip.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Point in time quota observation.
#[derive(Debug, Clone)]
pub struct QuotaDataPoint {
    pub timestamp: Instant,
    pub used_tokens: f64,
}

/// Severity level of detected consumption acceleration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccelerationWarning {
    Nominal,
    Elevated,
    Critical,
}

/// Outcome of the acceleration analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaAccelerationVerdict {
    pub velocity_tokens_per_sec: f64,
    pub acceleration_tokens_per_sec2: f64,
    pub warning: AccelerationWarning,
    pub is_runaway_suspected: bool,
}

pub struct QuotaAccelerationDetector;

impl QuotaAccelerationDetector {
    /// Analyzes a time-ordered slice of at least 3 quota data points.
    pub fn detect(points: &[QuotaDataPoint]) -> QuotaAccelerationVerdict {
        if points.len() < 3 {
            return QuotaAccelerationVerdict {
                velocity_tokens_per_sec: 0.0,
                acceleration_tokens_per_sec2: 0.0,
                warning: AccelerationWarning::Nominal,
                is_runaway_suspected: false,
            };
        }

        let p0 = &points[points.len() - 3];
        let p1 = &points[points.len() - 2];
        let p2 = &points[points.len() - 1];

        let dt1 = (p1.timestamp.duration_since(p0.timestamp).as_secs_f64()).max(0.001);
        let dt2 = (p2.timestamp.duration_since(p1.timestamp).as_secs_f64()).max(0.001);

        let v1 = (p1.used_tokens - p0.used_tokens) / dt1;
        let v2 = (p2.used_tokens - p1.used_tokens) / dt2;

        let dt_mid = (dt1 + dt2) / 2.0;
        let acceleration = (v2 - v1) / dt_mid;

        let (warning, runaway) = if acceleration > 500.0 && v2 > 1000.0 {
            (AccelerationWarning::Critical, true)
        } else if acceleration > 150.0 && v2 > 400.0 {
            (AccelerationWarning::Elevated, false)
        } else {
            (AccelerationWarning::Nominal, false)
        };

        QuotaAccelerationVerdict {
            velocity_tokens_per_sec: v2.max(0.0),
            acceleration_tokens_per_sec2: acceleration,
            warning,
            is_runaway_suspected: runaway,
        }
    }
}
