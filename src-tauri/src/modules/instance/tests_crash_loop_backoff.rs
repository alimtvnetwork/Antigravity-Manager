use super::*;

#[cfg(test)]
mod tests {
    use super::crash_loop_backoff::CrashLoopBackoffGuard;
    use std::time::Duration;

    #[test]
    fn test_initial_state_can_launch() {
        let guard = CrashLoopBackoffGuard::default();
        assert!(guard.can_launch_now("inst-test-1"));
        assert_eq!(guard.required_backoff_delay("inst-test-1"), Duration::ZERO);
    }

    #[test]
    fn test_short_lifespan_increments_crash_count() {
        let mut guard = CrashLoopBackoffGuard::new(
            Some(Duration::from_secs(10)),
            Some(Duration::from_secs(30)),
        );
        guard.record_launch("inst-crash-1");
        // Immediate termination (<10s) increments crash count
        let count = guard.record_termination("inst-crash-1");
        assert_eq!(count, 1);

        // Required backoff should be 2^1 = 2s
        let delay = guard.required_backoff_delay("inst-crash-1");
        assert_eq!(delay, Duration::from_secs(2));
    }

    #[test]
    fn test_escalating_backoff_progression() {
        let mut guard = CrashLoopBackoffGuard::new(
            Some(Duration::from_secs(10)),
            Some(Duration::from_secs(30)),
        );

        guard.record_launch("inst-crash-loop");
        guard.record_termination("inst-crash-loop"); // count 1 -> 2s
        assert_eq!(
            guard.required_backoff_delay("inst-crash-loop"),
            Duration::from_secs(2)
        );

        guard.record_launch("inst-crash-loop");
        guard.record_termination("inst-crash-loop"); // count 2 -> 4s
        assert_eq!(
            guard.required_backoff_delay("inst-crash-loop"),
            Duration::from_secs(4)
        );

        guard.record_launch("inst-crash-loop");
        guard.record_termination("inst-crash-loop"); // count 3 -> 8s
        assert_eq!(
            guard.required_backoff_delay("inst-crash-loop"),
            Duration::from_secs(8)
        );

        guard.record_launch("inst-crash-loop");
        guard.record_termination("inst-crash-loop"); // count 4 -> 16s
        assert_eq!(
            guard.required_backoff_delay("inst-crash-loop"),
            Duration::from_secs(16)
        );
    }

    #[test]
    fn test_backoff_clamps_to_max() {
        let mut guard = CrashLoopBackoffGuard::new(
            Some(Duration::from_secs(10)),
            Some(Duration::from_secs(30)),
        );

        for _ in 0..6 {
            guard.record_launch("inst-excessive");
            guard.record_termination("inst-excessive");
        }

        // 2^5 = 32s, but clamped to max_backoff = 30s
        let delay = guard.required_backoff_delay("inst-excessive");
        assert_eq!(delay, Duration::from_secs(30));
    }

    #[test]
    fn test_healthy_run_resets_counter() {
        // Use very short threshold so we can test healthy run without waiting long
        let mut guard = CrashLoopBackoffGuard::new(
            Some(Duration::from_millis(5)),
            Some(Duration::from_secs(30)),
        );

        guard.record_launch("inst-recover");
        guard.record_termination("inst-recover"); // immediate -> count 1
        assert_eq!(
            guard.required_backoff_delay("inst-recover"),
            Duration::from_secs(2)
        );

        guard.record_launch("inst-recover");
        std::thread::sleep(Duration::from_millis(15));
        let count = guard.record_termination("inst-recover"); // healthy (>5ms) -> resets to 0
        assert_eq!(count, 0);
        assert_eq!(guard.required_backoff_delay("inst-recover"), Duration::ZERO);
        assert!(guard.can_launch_now("inst-recover"));
    }
}
