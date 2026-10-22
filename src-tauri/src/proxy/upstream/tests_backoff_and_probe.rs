// Unit Test Suite for Upstream Connection Backoff Matrix & Keepalive Probe (Imp-512 & Imp-482)
// Validates exponential backoff progression, jitter clamping, overflow safety,
// and TCP keepalive socket configurations.

#[cfg(test)]
mod tests {
    use super::super::connection_backoff_matrix::{BackoffMatrixConfig, ConnectionBackoffMatrix};
    use super::super::tcp_keepalive_probe::{
        configure_tcp_keepalive, TcpKeepaliveProbeConfig, DEFAULT_TCP_KEEPALIVE_IDLE,
        DEFAULT_TCP_KEEPALIVE_INTERVAL,
    };
    use std::time::Duration;

    #[test]
    fn test_default_config_values() {
        let config = BackoffMatrixConfig::default();
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert!((config.multiplier - 2.0).abs() < f64::EPSILON);
        assert!((config.jitter_factor - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_jitter_exponential_progression() {
        let matrix = ConnectionBackoffMatrix::new(BackoffMatrixConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter_factor: 0.0,
        });

        assert_eq!(matrix.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(matrix.calculate_backoff(1), Duration::from_millis(200));
        assert_eq!(matrix.calculate_backoff(2), Duration::from_millis(400));
        assert_eq!(matrix.calculate_backoff(3), Duration::from_millis(800));
        assert_eq!(matrix.calculate_backoff(4), Duration::from_millis(1600));
    }

    #[test]
    fn test_max_delay_clamping() {
        let matrix = ConnectionBackoffMatrix::new(BackoffMatrixConfig {
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(3000),
            multiplier: 2.0,
            jitter_factor: 0.0,
        });

        assert_eq!(matrix.calculate_backoff(0), Duration::from_millis(1000));
        assert_eq!(matrix.calculate_backoff(1), Duration::from_millis(2000));
        assert_eq!(matrix.calculate_backoff(2), Duration::from_millis(3000));
        assert_eq!(matrix.calculate_backoff(10), Duration::from_millis(3000));
    }

    #[test]
    fn test_jitter_factor_range_bounds() {
        let matrix = ConnectionBackoffMatrix::new(BackoffMatrixConfig {
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter_factor: 0.20,
        });

        // For attempt 1: base = 2000ms. Range = [1600ms, 2400ms]
        for _ in 0..30 {
            let backoff = matrix.calculate_backoff(1);
            let millis = backoff.as_millis();
            assert!(
                millis >= 1600 && millis <= 2400,
                "Backoff {}ms outside expected jitter range [1600, 2400]",
                millis
            );
        }
    }

    #[test]
    fn test_attempt_large_value_overflow_safety() {
        let matrix = ConnectionBackoffMatrix::default();
        let backoff = matrix.calculate_backoff(100);
        // Max delay 30s + 25% jitter = max 37.5s
        assert!(backoff <= Duration::from_millis(37500));
    }

    #[test]
    fn test_default_constants_and_probe_config() {
        assert_eq!(DEFAULT_TCP_KEEPALIVE_IDLE, Duration::from_secs(30));
        assert_eq!(DEFAULT_TCP_KEEPALIVE_INTERVAL, Duration::from_secs(10));

        let default_config = TcpKeepaliveProbeConfig::default();
        assert_eq!(default_config.idle_time, Duration::from_secs(30));
        assert_eq!(default_config.probe_interval, Duration::from_secs(10));
        assert_eq!(default_config.probe_count, 3);
        assert!(default_config.enabled);
    }

    #[test]
    fn test_configure_tcp_keepalive_with_defaults() {
        let config = configure_tcp_keepalive(None, None);
        assert_eq!(config, TcpKeepaliveProbeConfig::default());
    }

    #[test]
    fn test_configure_tcp_keepalive_custom_idle_and_interval() {
        let config = configure_tcp_keepalive(
            Some(Duration::from_secs(45)),
            Some(Duration::from_secs(5)),
        );

        assert_eq!(config.idle_time, Duration::from_secs(45));
        assert_eq!(config.probe_interval, Duration::from_secs(5));
        assert_eq!(config.probe_count, 3);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_clone_and_eq() {
        let c1 = configure_tcp_keepalive(Some(Duration::from_secs(15)), None);
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }
}
