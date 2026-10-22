#[cfg(test)]
mod tests {
    use super::lock_contention_watchdog::{LockContentionStatus, LockContentionWatchdog};

    #[test]
    fn test_initial_state_clean() {
        let wd = LockContentionWatchdog::default();
        assert_eq!(wd.active_lock_count(), 0);
        assert_eq!(wd.peak_hold_ms, 0);
        let (status, offenders) = wd.evaluate_contention(1000);
        assert_eq!(status, LockContentionStatus::Normal);
        assert!(offenders.is_empty());
    }

    #[test]
    fn test_short_lock_lifecycle() {
        let mut wd = LockContentionWatchdog::new(Some(3000), Some(8000));
        wd.acquire_lock("lock-1", "accounts.db", 1000);
        assert_eq!(wd.active_lock_count(), 1);

        // Evaluated at 1500ms (500ms hold, < 3000ms threshold)
        let (status, offenders) = wd.evaluate_contention(1500);
        assert_eq!(status, LockContentionStatus::Normal);
        assert!(offenders.is_empty());

        let duration = wd.release_lock("lock-1", 1600);
        assert_eq!(duration, Some(600));
        assert_eq!(wd.peak_hold_ms, 600);
        assert_eq!(wd.active_lock_count(), 0);
    }

    #[test]
    fn test_contended_state_detected() {
        let mut wd = LockContentionWatchdog::new(Some(3000), Some(8000));
        wd.acquire_lock("lock-slow", "task_index.db", 1000);

        // At 4500ms (3500ms hold >= 3000ms)
        let (status, offenders) = wd.evaluate_contention(4500);
        assert_eq!(status, LockContentionStatus::Contended);
        assert_eq!(offenders, vec!["lock-slow"]);
    }

    #[test]
    fn test_critical_timeout_detected() {
        let mut wd = LockContentionWatchdog::new(Some(3000), Some(8000));
        wd.acquire_lock("lock-deadlock", "repo_prompts.db", 1000);

        // At 9500ms (8500ms hold >= 8000ms)
        let (status, offenders) = wd.evaluate_contention(9500);
        assert_eq!(status, LockContentionStatus::CriticalTimeout);
        assert_eq!(offenders, vec!["lock-deadlock"]);
    }

    #[test]
    fn test_peak_hold_ms_tracks_maximum() {
        let mut wd = LockContentionWatchdog::default();
        wd.acquire_lock("l1", "db1", 1000);
        wd.release_lock("l1", 1500); // 500ms
        assert_eq!(wd.peak_hold_ms, 500);

        wd.acquire_lock("l2", "db2", 2000);
        wd.release_lock("l2", 3200); // 1200ms
        assert_eq!(wd.peak_hold_ms, 1200);

        wd.acquire_lock("l3", "db3", 4000);
        wd.release_lock("l3", 4300); // 300ms
        assert_eq!(wd.peak_hold_ms, 1200); // peak preserved
    }

    #[test]
    fn test_reset_clears_all_locks() {
        let mut wd = LockContentionWatchdog::default();
        wd.acquire_lock("l1", "db1", 1000);
        wd.reset();
        assert_eq!(wd.active_lock_count(), 0);
        assert_eq!(wd.peak_hold_ms, 0);
        let (status, _) = wd.evaluate_contention(2000);
        assert_eq!(status, LockContentionStatus::Normal);
    }
}
