//! Unit tests for Upstream Latency Exponential Smoother (Imp-557).

use super::latency_exponential_smoother::{
    LatencyExponentialSmoother, LatencySmootherConfig,
};

#[test]
fn test_latency_smoother_initial_state() {
    let smoother = LatencyExponentialSmoother::new(None);
    assert_eq!(smoother.samples_processed(), 0);
    assert_eq!(smoother.smoothed_latency(), 150.0);
}

#[test]
fn test_latency_smoother_first_sample_initializes_value() {
    let mut smoother = LatencyExponentialSmoother::new(None);
    smoother.record_sample(80.0);
    assert_eq!(smoother.samples_processed(), 1);
    assert_eq!(smoother.smoothed_latency(), 80.0);
}

#[test]
fn test_latency_smoother_exponential_decay() {
    let cfg = LatencySmootherConfig {
        alpha: 0.5,
        initial_latency_ms: 100.0,
    };
    let mut smoother = LatencyExponentialSmoother::new(Some(cfg));

    smoother.record_sample(100.0); // sample 1: exactly 100.0
    assert_eq!(smoother.smoothed_latency(), 100.0);

    smoother.record_sample(200.0); // sample 2: 0.5 * 200 + 0.5 * 100 = 150.0
    assert_eq!(smoother.smoothed_latency(), 150.0);

    smoother.record_sample(200.0); // sample 3: 0.5 * 200 + 0.5 * 150 = 175.0
    assert_eq!(smoother.smoothed_latency(), 175.0);
}

#[test]
fn test_latency_smoother_floor_clamp() {
    let mut smoother = LatencyExponentialSmoother::new(None);
    smoother.record_sample(-50.0); // Clamped to 1.0 minimum
    assert_eq!(smoother.smoothed_latency(), 1.0);
}
