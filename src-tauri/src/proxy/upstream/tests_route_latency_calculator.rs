//! Unit tests for RouteLatencyCalculator (Imp-566)

use std::time::Duration;

use super::route_latency_calculator::{LatencyWeightConfig, RouteLatencyCalculator};

#[test]
fn test_unobserved_endpoint_returns_default_weight() {
    let calc = RouteLatencyCalculator::new(None);
    assert_eq!(calc.compute_route_weight("https://api.unknown.internal"), 1.0);
    assert_eq!(calc.get_ema_latency("https://api.unknown.internal"), None);
}

#[test]
fn test_first_sample_initializes_ema() {
    let mut calc = RouteLatencyCalculator::new(Some(LatencyWeightConfig {
        alpha: 0.25,
        baseline_latency_ms: 300.0,
    }));

    calc.record_sample("https://api.upstream-a.internal", Duration::from_millis(200));

    let ema = calc.get_ema_latency("https://api.upstream-a.internal").unwrap();
    assert!((ema - 200.0).abs() < 1e-4);

    let weight = calc.compute_route_weight("https://api.upstream-a.internal");
    // 300 / 200 = 1.5
    assert!((weight - 1.5).abs() < 1e-4);
}

#[test]
fn test_ema_smoothing_progression() {
    let mut calc = RouteLatencyCalculator::new(Some(LatencyWeightConfig {
        alpha: 0.5,
        baseline_latency_ms: 300.0,
    }));

    // First sample: 200ms -> EMA = 200ms
    calc.record_sample("https://node-1.internal", Duration::from_millis(200));
    // Second sample: 400ms -> EMA = 0.5 * 400 + 0.5 * 200 = 300ms
    calc.record_sample("https://node-1.internal", Duration::from_millis(400));

    let ema = calc.get_ema_latency("https://node-1.internal").unwrap();
    assert!((ema - 300.0).abs() < 1e-4);

    let weight = calc.compute_route_weight("https://node-1.internal");
    assert!((weight - 1.0).abs() < 1e-4);
}

#[test]
fn test_weight_clamping_upper_and_lower_bounds() {
    let mut calc = RouteLatencyCalculator::new(Some(LatencyWeightConfig {
        alpha: 1.0,
        baseline_latency_ms: 300.0,
    }));

    // Ultra-low latency: 10ms -> 300 / 10 = 30.0 -> clamped to 2.0
    calc.record_sample("https://fast-node.internal", Duration::from_millis(10));
    assert_eq!(calc.compute_route_weight("https://fast-node.internal"), 2.0);

    // Ultra-high latency: 10000ms -> 300 / 10000 = 0.03 -> clamped to 0.1
    calc.record_sample("https://slow-node.internal", Duration::from_millis(10000));
    assert_eq!(calc.compute_route_weight("https://slow-node.internal"), 0.1);
}

#[test]
fn test_distinct_routes_isolation() {
    let mut calc = RouteLatencyCalculator::new(None);

    calc.record_sample("https://route-a.internal", Duration::from_millis(150));
    calc.record_sample("https://route-b.internal", Duration::from_millis(600));

    let ema_a = calc.get_ema_latency("https://route-a.internal").unwrap();
    let ema_b = calc.get_ema_latency("https://route-b.internal").unwrap();

    assert!((ema_a - 150.0).abs() < 1e-4);
    assert!((ema_b - 600.0).abs() < 1e-4);

    assert!(calc.compute_route_weight("https://route-a.internal") > 1.0);
    assert!(calc.compute_route_weight("https://route-b.internal") < 1.0);
}
