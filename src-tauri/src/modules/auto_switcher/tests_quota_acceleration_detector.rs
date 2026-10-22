//! Unit tests for QuotaAccelerationDetector (Imp-567)

use std::time::{Duration, Instant};

use super::quota_acceleration_detector::{
    AccelerationWarning, QuotaAccelerationDetector, QuotaDataPoint,
};

#[test]
fn test_insufficient_points_returns_nominal() {
    let empty: [QuotaDataPoint; 0] = [];
    let verdict = QuotaAccelerationDetector::detect(&empty);
    assert_eq!(verdict.warning, AccelerationWarning::Nominal);
    assert_eq!(verdict.velocity_tokens_per_sec, 0.0);
    assert_eq!(verdict.acceleration_tokens_per_sec2, 0.0);
    assert!(!verdict.is_runaway_suspected);

    let single = [QuotaDataPoint {
        timestamp: Instant::now(),
        used_tokens: 500.0,
    }];
    let verdict = QuotaAccelerationDetector::detect(&single);
    assert_eq!(verdict.warning, AccelerationWarning::Nominal);
    assert!(!verdict.is_runaway_suspected);
}

#[test]
fn test_steady_state_constant_velocity() {
    let now = Instant::now();
    let points = [
        QuotaDataPoint {
            timestamp: now,
            used_tokens: 100.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(1),
            used_tokens: 200.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(2),
            used_tokens: 300.0,
        },
    ];

    let verdict = QuotaAccelerationDetector::detect(&points);
    assert_eq!(verdict.warning, AccelerationWarning::Nominal);
    assert!(!verdict.is_runaway_suspected);
    assert!((verdict.velocity_tokens_per_sec - 100.0).abs() < 1e-2);
    assert!(verdict.acceleration_tokens_per_sec2.abs() < 1e-2);
}

#[test]
fn test_elevated_acceleration_warning() {
    let now = Instant::now();
    let points = [
        QuotaDataPoint {
            timestamp: now,
            used_tokens: 0.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(1),
            used_tokens: 200.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(2),
            used_tokens: 800.0,
        },
    ];

    let verdict = QuotaAccelerationDetector::detect(&points);
    assert_eq!(verdict.warning, AccelerationWarning::Elevated);
    assert!(!verdict.is_runaway_suspected);
    assert!((verdict.velocity_tokens_per_sec - 600.0).abs() < 1e-2);
    assert!((verdict.acceleration_tokens_per_sec2 - 400.0).abs() < 1e-2);
}

#[test]
fn test_critical_runaway_acceleration() {
    let now = Instant::now();
    let points = [
        QuotaDataPoint {
            timestamp: now,
            used_tokens: 0.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(1),
            used_tokens: 500.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(2),
            used_tokens: 2500.0,
        },
    ];

    let verdict = QuotaAccelerationDetector::detect(&points);
    assert_eq!(verdict.warning, AccelerationWarning::Critical);
    assert!(verdict.is_runaway_suspected);
    assert!((verdict.velocity_tokens_per_sec - 2000.0).abs() < 1e-2);
    assert!((verdict.acceleration_tokens_per_sec2 - 1500.0).abs() < 1e-2);
}

#[test]
fn test_decelerating_consumption() {
    let now = Instant::now();
    let points = [
        QuotaDataPoint {
            timestamp: now,
            used_tokens: 0.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(1),
            used_tokens: 1000.0,
        },
        QuotaDataPoint {
            timestamp: now + Duration::from_secs(2),
            used_tokens: 1200.0,
        },
    ];

    let verdict = QuotaAccelerationDetector::detect(&points);
    assert_eq!(verdict.warning, AccelerationWarning::Nominal);
    assert!(!verdict.is_runaway_suspected);
    assert!(verdict.acceleration_tokens_per_sec2 < 0.0);
}
