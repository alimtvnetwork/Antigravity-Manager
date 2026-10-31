#[cfg(test)]
mod tests {
    use super::super::deadlock_detector::{
        ConcurrencyContentionStats, DeadlockDetectorConfig, LockSessionTracker,
    };
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[test]
    fn test_deadlock_detector_default_config() {
        let config = DeadlockDetectorConfig::default();
        assert_eq!(
            config.lock_contention_warning_threshold,
            Duration::from_millis(3000)
        );
        assert_eq!(config.busy_retry_danger_threshold, 15);
    }

    #[test]
    fn test_lock_session_tracker_initial_state_defaults() {
        let tracker = LockSessionTracker::start("workspace.db", None);

        assert_eq!(tracker.db_name(), "workspace.db");
        assert_eq!(tracker.busy_retries(), 0);
        // Default warning threshold is 3000ms; elapsed immediately is << 3000ms
        assert_eq!(tracker.check_contention(), None);
    }

    #[test]
    fn test_lock_session_tracker_initial_state_custom_config() {
        let custom_config = DeadlockDetectorConfig {
            lock_contention_warning_threshold: Duration::from_millis(500),
            busy_retry_danger_threshold: 5,
        };
        let tracker = LockSessionTracker::start("installation.db", Some(custom_config));

        assert_eq!(tracker.db_name(), "installation.db");
        assert_eq!(tracker.busy_retries(), 0);
    }

    #[test]
    fn test_record_busy_retry_custom_danger_threshold() {
        let custom_config = DeadlockDetectorConfig {
            lock_contention_warning_threshold: Duration::from_millis(3000),
            busy_retry_danger_threshold: 3,
        };
        let mut tracker = LockSessionTracker::start("repodb/pipeline.db", Some(custom_config));

        // Retry 1: count 1 < 3 -> false
        assert!(!tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 1);

        // Retry 2: count 2 < 3 -> false
        assert!(!tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 2);

        // Retry 3: count 3 >= 3 -> true (danger threshold hit!)
        assert!(tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 3);

        // Retry 4: count 4 >= 3 -> true (danger continues)
        assert!(tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 4);
    }

    #[test]
    fn test_record_busy_retry_default_threshold_ladder() {
        let mut tracker = LockSessionTracker::start("auth.db", None);

        // Retries 1 through 14 should all return false
        for i in 1..=14 {
            assert!(
                !tracker.record_busy_retry(),
                "Retry {i} should be below threshold 15"
            );
            assert_eq!(tracker.busy_retries(), i);
        }

        // Retry 15: exactly at threshold -> danger true
        assert!(tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 15);

        // Retry 16: above threshold -> danger true
        assert!(tracker.record_busy_retry());
        assert_eq!(tracker.busy_retries(), 16);
    }

    #[test]
    fn test_check_contention_zero_threshold_triggers_warning() {
        let immediate_config = DeadlockDetectorConfig {
            lock_contention_warning_threshold: Duration::from_millis(0),
            busy_retry_danger_threshold: 10,
        };
        let tracker = LockSessionTracker::start("accounts.db", Some(immediate_config));

        let warning = tracker.check_contention();
        assert!(warning.is_some());
        assert!(warning.unwrap() >= Duration::from_millis(0));
    }

    #[test]
    fn test_check_contention_high_threshold_returns_none() {
        let high_config = DeadlockDetectorConfig {
            lock_contention_warning_threshold: Duration::from_secs(3600),
            busy_retry_danger_threshold: 10,
        };
        let tracker = LockSessionTracker::start("audit.db", Some(high_config));

        assert_eq!(tracker.check_contention(), None);
    }

    #[test]
    fn test_concurrency_contention_stats_atomic_operations() {
        let stats = ConcurrencyContentionStats::default();

        assert_eq!(stats.total_busy_events.load(Ordering::Relaxed), 0);
        assert_eq!(stats.lock_escalation_warnings.load(Ordering::Relaxed), 0);

        stats.total_busy_events.fetch_add(1, Ordering::Relaxed);
        stats.total_busy_events.fetch_add(1, Ordering::Relaxed);
        stats
            .lock_escalation_warnings
            .fetch_add(1, Ordering::Relaxed);

        assert_eq!(stats.total_busy_events.load(Ordering::Relaxed), 2);
        assert_eq!(stats.lock_escalation_warnings.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_lock_session_tracker_debug_formatting() {
        let tracker = LockSessionTracker::start("debug.db", None);
        let debug_str = format!("{tracker:?}");
        assert!(debug_str.contains("LockSessionTracker"));
        assert!(debug_str.contains("debug.db"));
    }
}
