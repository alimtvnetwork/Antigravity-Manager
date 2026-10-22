// Unit Test Suite for Auto-Switcher 5-Hour Quota Boundary Synchronizer & Reset Tracker (Imp-487 & Imp-517)
// Validates rolling 5-hour quota reset boundaries, recovery anticipation,
// and historical quota replenishment tracking.

#[cfg(test)]
mod tests {
    use super::super::quota_boundary_synchronizer::{
        QuotaBoundarySynchronizer, DEFAULT_IMPENDING_RECOVERY_SECS, DEFAULT_ROLLING_WINDOW_SECS,
    };
    use super::super::quota_reset_epoch_tracker::QuotaResetEpochTracker;

    #[test]
    fn test_quota_boundary_synchronizer_evaluates_exact_window() {
        let synchronizer = QuotaBoundarySynchronizer::new(18000, 180); // 5h window, 3m recovery

        // Elapsed = 17900s (100s until reset, within 180s threshold)
        let status = synchronizer.evaluate_boundary(27900, 10000);

        assert_eq!(status.secs_until_reset, 100);
        assert!(status.is_impending_recovery);
        assert!(status.should_withhold_rotation);

        let postpone = synchronizer.recommended_postpone_secs(&status);
        assert_eq!(postpone, Some(105)); // 100 + 5s buffer
    }

    #[test]
    fn test_quota_boundary_synchronizer_not_impending() {
        let synchronizer = QuotaBoundarySynchronizer::new(18000, 180);

        // Elapsed = 10000s (8000s until reset, well outside 180s)
        let status = synchronizer.evaluate_boundary(20000, 10000);

        assert_eq!(status.secs_until_reset, 8000);
        assert!(!status.is_impending_recovery);
        assert!(!status.should_withhold_rotation);
        assert_eq!(synchronizer.recommended_postpone_secs(&status), None);
    }

    #[test]
    fn test_quota_boundary_zero_window_fallback() {
        let synchronizer = QuotaBoundarySynchronizer::new(0, 60);
        // Falls back to DEFAULT_ROLLING_WINDOW_SECS (18000s)
        let status = synchronizer.evaluate_boundary(18000, 0);
        assert_eq!(status.secs_until_reset, DEFAULT_ROLLING_WINDOW_SECS);
    }

    #[test]
    fn test_quota_reset_epoch_tracker_records_and_predicts() {
        let tracker = QuotaResetEpochTracker::new(18000);

        let account = "acc_test_pro_01";
        let initial_epoch = 1_000_000;

        // Record initial reset
        tracker.record_reset(account, initial_epoch);

        // Next predicted reset should be initial_epoch + 18000
        let predicted = tracker.predict_next_reset(account, initial_epoch);
        assert_eq!(predicted, initial_epoch + 18000);

        // Record second reset after 7200s (2h)
        tracker.record_reset(account, initial_epoch + 7200);

        // Should now adapt estimated cycle to 7200s
        let predicted_next = tracker.predict_next_reset(account, initial_epoch + 7200);
        assert_eq!(predicted_next, initial_epoch + 7200 + 7200);
    }

    #[test]
    fn test_quota_reset_epoch_tracker_unknown_account_fallback() {
        let tracker = QuotaResetEpochTracker::new(18000);
        let predicted = tracker.predict_next_reset("unknown_account", 5000);
        assert_eq!(predicted, 5000 + 18000);
    }
}
