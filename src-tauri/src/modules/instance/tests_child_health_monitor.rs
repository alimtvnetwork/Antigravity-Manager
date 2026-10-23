use super::*;

#[cfg(test)]
mod tests {
    use super::child_health_monitor::ChildHealthMonitor;
    use std::time::Duration;

    #[test]
    fn test_unrecorded_instance_is_stalled() {
        let monitor = ChildHealthMonitor::default();
        // Instance with no heartbeats is considered stalled
        assert!(monitor.is_stalled("inst-unknown"));
        assert_eq!(monitor.silence_duration("inst-unknown"), None);
    }

    #[test]
    fn test_record_heartbeat_clears_stall() {
        let mut monitor = ChildHealthMonitor::default();
        monitor.record_heartbeat("inst-live");
        assert!(!monitor.is_stalled("inst-live"));
        assert!(monitor.silence_duration("inst-live").is_some());
    }

    #[test]
    fn test_stall_threshold_detection() {
        // Use 10ms threshold for fast testing
        let mut monitor = ChildHealthMonitor::new(Some(Duration::from_millis(10)));
        monitor.record_heartbeat("inst-slow");
        assert!(!monitor.is_stalled("inst-slow"));

        std::thread::sleep(Duration::from_millis(20));
        assert!(monitor.is_stalled("inst-slow"));
    }

    #[test]
    fn test_repeated_heartbeats_maintain_freshness() {
        let mut monitor = ChildHealthMonitor::new(Some(Duration::from_millis(50)));
        monitor.record_heartbeat("inst-active");
        std::thread::sleep(Duration::from_millis(20));
        assert!(!monitor.is_stalled("inst-active"));

        monitor.record_heartbeat("inst-active");
        let silence = monitor.silence_duration("inst-active").unwrap();
        assert!(silence < Duration::from_millis(10));
    }

    #[test]
    fn test_remove_instance() {
        let mut monitor = ChildHealthMonitor::default();
        monitor.record_heartbeat("inst-retire");
        assert!(monitor.remove_instance("inst-retire"));
        assert!(!monitor.remove_instance("inst-retire"));
        assert!(monitor.is_stalled("inst-retire"));
    }
}
