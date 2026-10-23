//! Unit tests for Quota Exhaustion Velocity Predictor (`Imp-618`)

use super::quota_exhaustion_predictor::{
    calculate_burn_rate_velocity, predict_quota_exhaustion, predict_quota_exhaustion_default,
    BurnRateVelocity, ExhaustionForecast, ExhaustionSeverity, QuotaExhaustionPredictor,
    UsageObservation, DEFAULT_SWITCH_THRESHOLD_SECONDS,
};

#[test]
fn test_empty_observations_returns_zero_velocity() {
    let velocity = calculate_burn_rate_velocity(&[]);
    assert_eq!(velocity, BurnRateVelocity::zero());
    assert!(velocity.is_zero());
    assert_eq!(velocity.observation_count, 0);
    assert_eq!(velocity.window_seconds, 0);
    assert_eq!(velocity.total_tokens, 0);
    assert_eq!(velocity.tokens_per_sec(), 0.0);
    assert_eq!(velocity.tokens_per_min(), 0.0);
}

#[test]
fn test_single_observation_returns_zero_velocity() {
    let obs = [UsageObservation::new(1000, 500)];
    let velocity = calculate_burn_rate_velocity(&obs);
    assert!(velocity.is_zero());
    assert_eq!(velocity.observation_count, 1);
    assert_eq!(velocity.window_seconds, 0);
    assert_eq!(velocity.total_tokens, 500);
    assert_eq!(velocity.sample_count(), 1);
    assert_eq!(velocity.total_tokens_observed(), 500);
}

#[test]
fn test_constant_consumption_burn_rate() {
    let obs = [
        UsageObservation::new(100, 500),
        UsageObservation::new(110, 500),
        UsageObservation::new(120, 1000),
    ];
    let velocity = calculate_burn_rate_velocity(&obs);
    // Window: 120 - 100 = 20 seconds.
    // Total tokens: 500 + 500 + 1000 = 2000.
    // Velocity: 2000 / 20 = 100.0 tokens/second.
    assert!(!velocity.is_zero());
    assert_eq!(velocity.window_seconds, 20);
    assert_eq!(velocity.total_tokens, 2000);
    assert_eq!(velocity.tokens_per_second, 100.0);
    assert_eq!(velocity.tokens_per_minute, 6000.0);
    assert_eq!(velocity.sample_count(), 3);
    assert_eq!(velocity.total_tokens_observed(), 2000);
}

#[test]
fn test_unsorted_observations_yields_identical_velocity() {
    let obs_unsorted = [
        UsageObservation::new(120, 1000),
        UsageObservation::new(100, 500),
        UsageObservation::new(110, 500),
    ];
    let velocity = calculate_burn_rate_velocity(&obs_unsorted);
    assert_eq!(velocity.window_seconds, 20);
    assert_eq!(velocity.total_tokens, 2000);
    assert_eq!(velocity.tokens_per_second, 100.0);
}

#[test]
fn test_same_timestamp_multiple_observations() {
    let obs = [
        UsageObservation::new(100, 400),
        UsageObservation::new(100, 600),
    ];
    let velocity = calculate_burn_rate_velocity(&obs);
    // Window is 0, clamped to 1.0s effective
    assert_eq!(velocity.window_seconds, 0);
    assert_eq!(velocity.total_tokens, 1000);
    assert_eq!(velocity.tokens_per_second, 1000.0);
}

#[test]
fn test_zero_remaining_tokens_critical_immediate_switch() {
    let obs = [
        UsageObservation::new(100, 500),
        UsageObservation::new(110, 500),
    ];
    let forecast = predict_quota_exhaustion(&obs, 0, 300);
    assert_eq!(forecast.remaining_tokens, 0);
    assert_eq!(forecast.severity, ExhaustionSeverity::Critical);
    assert!(forecast.is_critical());
    assert!(!forecast.is_elevated());
    assert!(!forecast.is_nominal());
    assert!(forecast.should_switch_preemptively);
    assert!(forecast.should_switch());
    assert!(forecast.is_switch_recommended());
    assert_eq!(forecast.runway_seconds, Some(0.0));
    assert!(forecast.reason.contains("Remaining quota is zero"));
}

#[test]
fn test_zero_burn_rate_nominal_infinite_runway() {
    let obs = [UsageObservation::new(100, 500)]; // Single obs -> zero velocity
    let forecast = predict_quota_exhaustion(&obs, 100_000, 300);
    assert_eq!(forecast.severity, ExhaustionSeverity::Nominal);
    assert!(forecast.is_nominal());
    assert!(!forecast.is_critical());
    assert!(!forecast.is_elevated());
    assert!(!forecast.should_switch_preemptively);
    assert_eq!(forecast.runway_seconds, None);
    assert_eq!(forecast.runway_minutes(), None);
    assert!(forecast.reason.contains("infinite"));
}

#[test]
fn test_runway_below_threshold_triggers_critical_switch() {
    // 200 tokens/sec
    let obs = [
        UsageObservation::new(100, 1000),
        UsageObservation::new(110, 1000),
    ];
    // Remaining 20,000 tokens -> 20,000 / (2000 / 10 = 200 tok/sec) -> 100s runway
    let forecast = predict_quota_exhaustion(&obs, 20_000, 300);
    assert_eq!(forecast.severity, ExhaustionSeverity::Critical);
    assert!(forecast.is_critical());
    assert!(forecast.should_switch_preemptively);
    assert!(forecast.is_switch_recommended());
    assert_eq!(forecast.runway_seconds, Some(100.0));
    assert_eq!(forecast.runway_minutes(), Some(1.7));
    assert!(forecast.reason.contains("preemptive switch required"));
}

#[test]
fn test_runway_in_elevated_window() {
    // 200 tokens/sec. Threshold: 300s. Elevated: 300s - 600s.
    let obs = [
        UsageObservation::new(100, 1000),
        UsageObservation::new(110, 1000),
    ];
    // 90,000 tokens / 200 tok/s = 450s runway (between 300s and 600s)
    let forecast = predict_quota_exhaustion(&obs, 90_000, 300);
    assert_eq!(forecast.severity, ExhaustionSeverity::Elevated);
    assert!(forecast.is_elevated());
    assert!(!forecast.should_switch_preemptively);
    assert!(!forecast.is_critical());
    assert_eq!(forecast.runway_seconds, Some(450.0));
    assert_eq!(forecast.runway_minutes(), Some(7.5));
    assert!(forecast.reason.contains("elevated risk"));
}

#[test]
fn test_runway_comfortably_nominal() {
    // 200 tokens/sec. Threshold: 300s.
    let obs = [
        UsageObservation::new(100, 1000),
        UsageObservation::new(110, 1000),
    ];
    // 200,000 tokens / 200 tok/s = 1000s runway (> 600s)
    let forecast = predict_quota_exhaustion(&obs, 200_000, 300);
    assert_eq!(forecast.severity, ExhaustionSeverity::Nominal);
    assert!(forecast.is_nominal());
    assert!(!forecast.should_switch_preemptively);
    assert_eq!(forecast.runway_seconds, Some(1000.0));
    assert_eq!(forecast.runway_minutes(), Some(16.7));
    assert!(forecast.reason.contains("nominal"));
}

#[test]
fn test_default_threshold_helper_and_struct_wrapper() {
    let obs = [
        UsageObservation::new(0, 500),
        UsageObservation::new(10, 500),
    ];
    // Velocity = 100 tok/sec
    let forecast = predict_quota_exhaustion_default(&obs, 20_000);
    assert_eq!(forecast.switch_threshold_seconds, DEFAULT_SWITCH_THRESHOLD_SECONDS);
    assert_eq!(forecast.runway_seconds, Some(200.0));
    assert!(forecast.should_switch());

    // Struct wrapper
    let forecast_struct = QuotaExhaustionPredictor::predict_quota_exhaustion_default(&obs, 20_000);
    assert_eq!(forecast, forecast_struct);

    let velocity_struct = QuotaExhaustionPredictor::calculate_burn_rate_velocity(&obs);
    assert_eq!(velocity_struct.tokens_per_second, 100.0);

    let forecast_struct2 =
        QuotaExhaustionPredictor::predict_quota_exhaustion(&obs, 20_000, DEFAULT_SWITCH_THRESHOLD_SECONDS);
    assert_eq!(forecast, forecast_struct2);
}

#[test]
fn test_observation_accessors() {
    let obs = UsageObservation::new(1700000000, 850);
    assert_eq!(obs.timestamp(), 1700000000);
    assert_eq!(obs.tokens(), 850);
}

#[test]
fn test_serialization_and_deserialization() {
    let obs = UsageObservation::new(12345678, 42);
    let serialized_obs = serde_json::to_string(&obs).expect("Serialize observation");
    let deserialized_obs: UsageObservation =
        serde_json::from_str(&serialized_obs).expect("Deserialize observation");
    assert_eq!(obs, deserialized_obs);

    let forecast = predict_quota_exhaustion_default(&[obs], 1000);
    let json = serde_json::to_string(&forecast).expect("Serialize forecast");
    let deserialized: ExhaustionForecast = serde_json::from_str(&json).expect("Deserialize forecast");
    assert_eq!(forecast, deserialized);
}
