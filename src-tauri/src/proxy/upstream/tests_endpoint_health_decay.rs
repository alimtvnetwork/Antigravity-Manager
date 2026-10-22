//! Unit tests for Upstream Endpoint Health Decay (Imp-561).

use super::endpoint_health_decay::{EndpointHealthTracker, HealthDecayConfig};

#[test]
fn test_endpoint_health_tracker_initial_state() {
    let tracker = EndpointHealthTracker::new("https://api.openai.com", None);
    assert_eq!(tracker.endpoint_url(), "https://api.openai.com");
    assert_eq!(tracker.current_penalty(), 0.0);
    assert_eq!(tracker.health_score(), 1.0);
}

#[test]
fn test_endpoint_health_tracker_record_failure() {
    let mut tracker = EndpointHealthTracker::new("https://api.anthropic.com", None);

    tracker.record_failure(1.0);
    let penalty = tracker.current_penalty();
    assert!(penalty > 0.9 && penalty <= 1.0);

    let score = tracker.health_score();
    // Health score = 1.0 / (1.0 + penalty) => approx 0.5
    assert!(score > 0.45 && score <= 0.55);
}

#[test]
fn test_endpoint_health_tracker_record_success_recovers() {
    let mut tracker = EndpointHealthTracker::new("https://generativelanguage.googleapis.com", None);

    tracker.record_failure(2.0);
    assert!(tracker.current_penalty() > 1.9);

    tracker.record_success();
    // Success subtracts 0.5 penalty
    assert!(tracker.current_penalty() < 1.6);
}

#[test]
fn test_endpoint_health_tracker_custom_half_life() {
    let cfg = HealthDecayConfig {
        half_life_seconds: 30.0,
        clear_threshold: 0.01,
    };
    let tracker = EndpointHealthTracker::new("https://api.groq.com", Some(cfg));
    assert_eq!(tracker.health_score(), 1.0);
}
