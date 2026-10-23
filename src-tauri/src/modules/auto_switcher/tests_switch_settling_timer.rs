#[cfg(test)]
mod tests {
    use super::super::switch_settling_timer::{
        SwitchSettlingTimer, DEFAULT_SETTLING_DELAY_SECS,
    };
    use std::time::Duration;

    #[test]
    fn test_switch_settling_timer_default_constants() {
        assert_eq!(DEFAULT_SETTLING_DELAY_SECS, 10);

        let timer = SwitchSettlingTimer::default();
        assert_eq!(timer.duration(), Duration::from_secs(10));
        assert_eq!(timer.duration().as_secs(), 10);
    }

    #[test]
    fn test_switch_settling_timer_custom_duration() {
        let duration = Duration::from_millis(250);
        let timer = SwitchSettlingTimer::new(duration);
        assert_eq!(timer.duration(), duration);

        let zero_timer = SwitchSettlingTimer::new(Duration::ZERO);
        assert_eq!(zero_timer.duration(), Duration::from_millis(0));
    }

    #[test]
    fn test_switch_settling_timer_clone_and_debug() {
        let timer = SwitchSettlingTimer::new(Duration::from_secs(5));
        let cloned = timer.clone();
        assert_eq!(cloned.duration(), Duration::from_secs(5));

        let debug_str = format!("{:?}", timer);
        assert!(debug_str.contains("SwitchSettlingTimer"));
        assert!(debug_str.contains("settling_duration"));
    }

    #[tokio::test]
    async fn test_switch_settling_timer_await_settling_zero_duration() {
        let timer = SwitchSettlingTimer::new(Duration::ZERO);
        let elapsed_ms = timer.await_settling("test-zero-inst").await;
        // Zero delay barrier returns immediately
        assert!(elapsed_ms < 500);
    }

    #[tokio::test]
    async fn test_switch_settling_timer_await_settling_short_duration() {
        let timer = SwitchSettlingTimer::new(Duration::from_millis(15));
        let elapsed_ms = timer.await_settling("test-short-inst").await;
        // Allows tolerance for thread scheduling jitter
        assert!(elapsed_ms >= 10);
    }

    #[tokio::test(start_paused = true)]
    async fn test_switch_settling_timer_await_settling_paused_virtual_time() {
        let timer = SwitchSettlingTimer::default();
        assert_eq!(timer.duration(), Duration::from_secs(10));

        // In paused tokio time, auto-advances through the 10s barrier without real-time delay
        let elapsed_ms = timer.await_settling("inst-virtual-10s").await;
        assert!(elapsed_ms >= 10_000);
    }

    #[tokio::test]
    async fn test_switch_settling_timer_instance_id_variations() {
        let timer = SwitchSettlingTimer::new(Duration::from_millis(1));

        // Empty instance ID
        let t1 = timer.await_settling("").await;
        assert!(t1 < 500);

        // Complex instance identifier
        let t2 = timer.await_settling("instance-node-test-001").await;
        assert!(t2 < 500);
    }
}
