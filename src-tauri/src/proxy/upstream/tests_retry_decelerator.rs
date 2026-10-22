#[cfg(test)]
mod tests {
    use super::super::retry_decelerator::{compute_decelerated_delay, DeceleratorConfig};
    use std::time::Duration;

    #[test]
    fn test_decelerator_default_config() {
        let cfg = DeceleratorConfig::default();
        assert_eq!(cfg.base_delay, Duration::from_millis(500));
        assert_eq!(cfg.max_delay, Duration::from_secs(10));
        assert!((cfg.multiplier - 2.0).abs() < f64::EPSILON);
        assert!((cfg.jitter_factor - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_decelerated_delay_initial_attempt() {
        let delay = compute_decelerated_delay(0, 0, None);
        // Base delay 500ms with 25% jitter gives range [375ms, 625ms]
        assert!(delay >= Duration::from_millis(350));
        assert!(delay <= Duration::from_millis(700));
    }

    #[test]
    fn test_compute_decelerated_delay_exponential_growth() {
        let delay_attempt_0 = compute_decelerated_delay(0, 0, None);
        let delay_attempt_2 = compute_decelerated_delay(2, 0, None);
        // Attempt 2 base is 500 * 2^2 = 2000ms ± 25% jitter [1500, 2500]
        assert!(delay_attempt_2 > delay_attempt_0);
        assert!(delay_attempt_2 >= Duration::from_millis(1400));
        assert!(delay_attempt_2 <= Duration::from_millis(2600));
    }

    #[test]
    fn test_compute_decelerated_delay_with_failures() {
        let cfg = DeceleratorConfig {
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(20),
            multiplier: 2.0,
            jitter_factor: 0.0, // Zero jitter for exact assertion
        };
        // 0 failures: 1000ms
        let d0 = compute_decelerated_delay(0, 0, Some(cfg));
        assert_eq!(d0, Duration::from_millis(1000));

        // 5 failures: 1000 * (1.0 + 5 * 0.20) = 2000ms
        let d5 = compute_decelerated_delay(0, 5, Some(cfg));
        assert_eq!(d5, Duration::from_millis(2000));
    }

    #[test]
    fn test_compute_decelerated_delay_max_clamping() {
        let cfg = DeceleratorConfig {
            base_delay: Duration::from_millis(5000),
            max_delay: Duration::from_millis(6000),
            multiplier: 3.0,
            jitter_factor: 0.0,
        };
        // Attempt 2 base would be 5000 * 9 = 45000ms, clamped to 6000ms
        let clamped = compute_decelerated_delay(2, 10, Some(cfg));
        assert_eq!(clamped, Duration::from_millis(6000));
    }
}
