//! Unit tests for Transport Backoff Stepper (`Imp-620`).

use super::transport_backoff_stepper::{
    BackoffConfig, JitterMode, NetworkFailureType, TransportBackoffStepper,
};

#[test]
fn test_unrecoverable_failure_rejected() {
    let mut stepper = TransportBackoffStepper::default();
    let failure = NetworkFailureType::UnrecoverableError("Invalid model parameter".to_string());
    let res = stepper.step(&failure);

    assert!(!res.should_retry);
    assert_eq!(res.delay_ms, 0);
    assert!(res.reason.unwrap().contains("non-retryable"));
}

#[test]
fn test_max_attempts_exhaustion() {
    let config = BackoffConfig::new(100, 1000, 3, JitterMode::None);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::TcpReset;

    let res1 = stepper.step_with_random(&failure, 1.0);
    assert!(res1.should_retry);
    assert_eq!(res1.attempt, 1);
    assert_eq!(res1.delay_ms, 100);

    let res2 = stepper.step_with_random(&failure, 1.0);
    assert!(res2.should_retry);
    assert_eq!(res2.attempt, 2);
    assert_eq!(res2.delay_ms, 200);

    let res3 = stepper.step_with_random(&failure, 1.0);
    assert!(res3.should_retry);
    assert_eq!(res3.attempt, 3);
    assert_eq!(res3.delay_ms, 400);

    // 4th attempt should be exhausted
    let res4 = stepper.step_with_random(&failure, 1.0);
    assert!(!res4.should_retry);
    assert_eq!(res4.attempt, 3);
    assert!(res4.reason.unwrap().contains("exhausted"));
}

#[test]
fn test_exponential_ceiling_clamping() {
    let config = BackoffConfig::new(1000, 2500, 5, JitterMode::None);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::DnsResolutionFailed;

    let res1 = stepper.step_with_random(&failure, 1.0);
    assert_eq!(res1.delay_ms, 1000); // 1000 * 2^0 = 1000

    let res2 = stepper.step_with_random(&failure, 1.0);
    assert_eq!(res2.delay_ms, 2000); // 1000 * 2^1 = 2000

    let res3 = stepper.step_with_random(&failure, 1.0);
    assert_eq!(res3.delay_ms, 2500); // 1000 * 2^2 = 4000, clamped to 2500 max
}

#[test]
fn test_full_jitter_behavior() {
    let config = BackoffConfig::new(1000, 10000, 3, JitterMode::FullJitter);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::GatewayStatus(503);

    // With rand = 0.5 and ceiling = 1000 -> delay = 500
    let res = stepper.step_with_random(&failure, 0.5);
    assert!(res.should_retry);
    assert_eq!(res.delay_ms, 500);

    // With rand = 0.0 -> delay = 0
    let mut stepper2 = TransportBackoffStepper::new(config);
    let res2 = stepper2.step_with_random(&failure, 0.0);
    assert_eq!(res2.delay_ms, 0);
}

#[test]
fn test_equal_jitter_behavior() {
    let config = BackoffConfig::new(1000, 10000, 3, JitterMode::EqualJitter);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::GatewayStatus(504);

    // ceiling = 1000, half = 500, rand = 0.5 -> 500 + 250 = 750
    let res = stepper.step_with_random(&failure, 0.5);
    assert!(res.should_retry);
    assert_eq!(res.delay_ms, 750);
}

#[test]
fn test_decorrelated_jitter_behavior() {
    let config = BackoffConfig::new(200, 5000, 3, JitterMode::DecorrelatedJitter);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::ConnectionTimeout;

    // First step: low = 200, last = 200, high = 600, rand = 0.5 -> 200 + 200 = 400
    let res1 = stepper.step_with_random(&failure, 0.5);
    assert!(res1.should_retry);
    assert_eq!(res1.delay_ms, 400);

    // Second step: low = 200, last = 400, high = 1200, rand = 0.5 -> 200 + 500 = 700
    let res2 = stepper.step_with_random(&failure, 0.5);
    assert!(res2.should_retry);
    assert_eq!(res2.delay_ms, 700);
}

#[test]
fn test_reset_and_cumulative_wait() {
    let config = BackoffConfig::new(500, 10000, 3, JitterMode::None);
    let mut stepper = TransportBackoffStepper::new(config);
    let failure = NetworkFailureType::TlsHandshakeFailed;

    stepper.step_with_random(&failure, 1.0); // 500
    stepper.step_with_random(&failure, 1.0); // 1000
    assert_eq!(stepper.total_wait_ms(), 1500);
    assert_eq!(stepper.current_attempt(), 2);

    stepper.reset();
    assert_eq!(stepper.total_wait_ms(), 0);
    assert_eq!(stepper.current_attempt(), 0);
}
