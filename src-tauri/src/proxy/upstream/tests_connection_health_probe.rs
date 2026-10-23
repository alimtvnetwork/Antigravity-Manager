use super::*;

#[cfg(test)]
mod tests {
    use super::connection_health_probe::{ConnectionHealthProbe, ConnectionHealthState};

    #[test]
    fn test_initial_state_is_healthy() {
        let probe = ConnectionHealthProbe::new("https://api.anthropic.com");
        assert_eq!(probe.health_state(), ConnectionHealthState::Healthy);
        assert!(probe.is_viable());
        assert_eq!(probe.average_latency_ms(), None);
    }

    #[test]
    fn test_consecutive_failures_transition_to_degraded() {
        let mut probe = ConnectionHealthProbe::new("https://api.openai.com");
        probe.record_failure();
        assert_eq!(probe.health_state(), ConnectionHealthState::Healthy);
        probe.record_failure();
        assert_eq!(probe.health_state(), ConnectionHealthState::Degraded);
        assert!(probe.is_viable());
    }

    #[test]
    fn test_consecutive_failures_transition_to_dead() {
        let mut probe = ConnectionHealthProbe::new("https://api.gemini.google.com");
        for _ in 0..5 {
            probe.record_failure();
        }
        assert_eq!(probe.health_state(), ConnectionHealthState::Dead);
        assert!(!probe.is_viable());
    }

    #[test]
    fn test_recovery_on_success() {
        let mut probe = ConnectionHealthProbe::new("https://api.openai.com");
        for _ in 0..4 {
            probe.record_failure();
        }
        assert_eq!(probe.health_state(), ConnectionHealthState::Degraded);

        probe.record_success(120);
        assert_eq!(probe.consecutive_failures, 0);
        assert_eq!(probe.health_state(), ConnectionHealthState::Healthy);
        assert_eq!(probe.average_latency_ms(), Some(120));
    }

    #[test]
    fn test_high_latency_transitions_to_degraded() {
        let mut probe = ConnectionHealthProbe::new("https://slow-upstream.com");
        probe.record_success(3000);
        probe.record_success(3200);
        assert_eq!(probe.health_state(), ConnectionHealthState::Degraded);
        assert!(probe.is_viable());
    }

    #[test]
    fn test_latency_history_bounded() {
        let mut probe = ConnectionHealthProbe::new("https://bounded-history.com");
        for i in 1..=20 {
            probe.record_success(i * 10);
        }
        assert_eq!(probe.latency_history_ms.len(), 10);
        // Only the last 10 entries (110 to 200) should be retained
        assert_eq!(*probe.latency_history_ms.first().unwrap(), 110);
        assert_eq!(*probe.latency_history_ms.last().unwrap(), 200);
    }
}
