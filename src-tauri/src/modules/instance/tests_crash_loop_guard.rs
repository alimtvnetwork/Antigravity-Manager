#[cfg(test)]
mod tests {
    use super::crash_loop_guard::CrashLoopGuard;

    #[test]
    fn test_initial_state_not_quarantined() {
        let guard = CrashLoopGuard::new();
        assert!(!guard.is_quarantined("inst-clean"));
    }

    #[test]
    fn test_single_crash_does_not_quarantine() {
        let guard = CrashLoopGuard::new();
        guard.record_launch("inst-crash-1");
        let quarantined = guard.record_termination("inst-crash-1");
        assert!(!quarantined);
        assert!(!guard.is_quarantined("inst-crash-1"));
    }

    #[test]
    fn test_three_rapid_crashes_trigger_quarantine() {
        let guard = CrashLoopGuard::new();
        // Rapid crash 1
        guard.record_launch("inst-loop");
        assert!(!guard.record_termination("inst-loop"));

        // Rapid crash 2
        guard.record_launch("inst-loop");
        assert!(!guard.record_termination("inst-loop"));

        // Rapid crash 3 -> reaches threshold = 3
        guard.record_launch("inst-loop");
        let entered_quarantine = guard.record_termination("inst-loop");
        assert!(entered_quarantine);
        assert!(guard.is_quarantined("inst-loop"));
    }

    #[test]
    fn test_reset_clears_quarantine() {
        let guard = CrashLoopGuard::new();
        for _ in 0..3 {
            guard.record_launch("inst-reset");
            guard.record_termination("inst-reset");
        }
        assert!(guard.is_quarantined("inst-reset"));

        guard.reset("inst-reset");
        assert!(!guard.is_quarantined("inst-reset"));
    }

    #[test]
    fn test_distinct_instances_quarantine_isolation() {
        let guard = CrashLoopGuard::new();
        for _ in 0..3 {
            guard.record_launch("inst-bad");
            guard.record_termination("inst-bad");
        }
        assert!(guard.is_quarantined("inst-bad"));
        assert!(!guard.is_quarantined("inst-good"));
    }
}
