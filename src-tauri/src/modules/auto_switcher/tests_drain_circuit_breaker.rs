#[cfg(test)]
mod tests {
    use super::drain_circuit_breaker::{CircuitBreakerState, DrainCircuitBreaker};

    #[test]
    fn test_initial_state_is_closed_and_eligible() {
        let cb = DrainCircuitBreaker::new("acc-test-1");
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.is_eligible(1000));
    }

    #[test]
    fn test_gradual_consumption_does_not_trip() {
        let mut cb = DrainCircuitBreaker::new("acc-test-2");
        // 100% -> 95% -> 90% over 30s
        assert_eq!(cb.record_observation(100, 1000), CircuitBreakerState::Closed);
        assert_eq!(cb.record_observation(95, 1015), CircuitBreakerState::Closed);
        assert_eq!(cb.record_observation(90, 1030), CircuitBreakerState::Closed);
        assert!(cb.is_eligible(1030));
    }

    #[test]
    fn test_rapid_drop_trips_circuit_breaker() {
        let mut cb = DrainCircuitBreaker::new("acc-test-3");
        // 100% -> 70% in 10s (drop of 30% >= 25% threshold)
        cb.record_observation(100, 1000);
        let state = cb.record_observation(70, 1010);

        assert_eq!(state, CircuitBreakerState::Tripped);
        assert_eq!(cb.tripped_at_sec, Some(1010));
        assert!(!cb.is_eligible(1015));
    }

    #[test]
    fn test_cooldown_transitions_to_half_open() {
        let mut cb = DrainCircuitBreaker::new("acc-test-4");
        cb.record_observation(100, 1000);
        cb.record_observation(60, 1010); // Tripped
        assert_eq!(cb.state, CircuitBreakerState::Tripped);

        // Before 180s cooldown: still ineligible
        assert!(!cb.is_eligible(1100));

        // After 180s cooldown: eligible and transitions on next observation
        assert!(cb.is_eligible(1200));
        let state = cb.record_observation(60, 1200);
        assert_eq!(state, CircuitBreakerState::HalfOpen);
    }

    #[test]
    fn test_stabilized_probe_resets_to_closed() {
        let mut cb = DrainCircuitBreaker::new("acc-test-5");
        cb.record_observation(100, 1000);
        cb.record_observation(60, 1010); // Tripped

        // Fast-forward past cooldown
        cb.record_observation(60, 1200); // Transitions to HalfOpen
        assert_eq!(cb.state, CircuitBreakerState::HalfOpen);

        // Stable usage (drop <= 5%) closes circuit breaker
        let state = cb.record_observation(58, 1210);
        assert_eq!(state, CircuitBreakerState::Closed);
        assert_eq!(cb.tripped_at_sec, None);
    }

    #[test]
    fn test_reset_clears_history() {
        let mut cb = DrainCircuitBreaker::new("acc-test-6");
        cb.record_observation(100, 1000);
        cb.record_observation(50, 1010);
        assert_eq!(cb.state, CircuitBreakerState::Tripped);

        cb.reset();
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert_eq!(cb.tripped_at_sec, None);
        assert!(cb.history.is_empty());
        assert!(cb.is_eligible(1020));
    }
}
