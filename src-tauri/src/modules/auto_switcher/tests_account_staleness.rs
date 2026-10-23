// Unit Test Suite for Inactive Account Staleness Detector (Imp-519)
// Validates account activity epoch evaluation, staleness thresholds,
// clock skew resilience, and auto-probe candidate queueing priority.

#[cfg(test)]
mod tests {
    use crate::modules::auto_switcher::account_staleness_detector::{
        AccountStalenessDetector, DEFAULT_STALENESS_THRESHOLD_SECS,
    };

    #[test]
    fn test_account_staleness_default_and_zero_threshold_fallback() {
        let default_detector = AccountStalenessDetector::default();
        let rep_default = default_detector.evaluate_staleness(
            "acc_default",
            10_000,
            10_000 + DEFAULT_STALENESS_THRESHOLD_SECS,
        );
        assert!(rep_default.is_stale);
        assert!(!rep_default.requires_force_refresh);

        let zero_detector = AccountStalenessDetector::new(0);
        let rep_zero_stale = zero_detector.evaluate_staleness(
            "acc_zero",
            10_000,
            10_000 + DEFAULT_STALENESS_THRESHOLD_SECS,
        );
        let rep_zero_fresh = zero_detector.evaluate_staleness(
            "acc_zero",
            10_000,
            10_000 + DEFAULT_STALENESS_THRESHOLD_SECS - 1,
        );
        assert!(rep_zero_stale.is_stale);
        assert!(!rep_zero_fresh.is_stale);

        let custom_detector = AccountStalenessDetector::new(600);
        let rep_custom_fresh = custom_detector.evaluate_staleness("acc_custom", 1_000, 1_599);
        let rep_custom_stale = custom_detector.evaluate_staleness("acc_custom", 1_000, 1_600);
        let rep_custom_force = custom_detector.evaluate_staleness("acc_custom", 1_000, 2_200);
        assert!(!rep_custom_fresh.is_stale);
        assert!(rep_custom_stale.is_stale);
        assert!(!rep_custom_stale.requires_force_refresh);
        assert!(rep_custom_force.is_stale);
        assert!(rep_custom_force.requires_force_refresh);
    }

    #[test]
    fn test_account_staleness_exact_threshold_boundaries() {
        let detector = AccountStalenessDetector::new(1800);
        let base_sync = 1_000_000;

        let rep_1799 = detector.evaluate_staleness("acc_test", base_sync, base_sync + 1799);
        assert_eq!(rep_1799.elapsed_secs, 1799);
        assert!(!rep_1799.is_stale);
        assert!(!rep_1799.requires_force_refresh);

        let rep_1800 = detector.evaluate_staleness("acc_test", base_sync, base_sync + 1800);
        assert_eq!(rep_1800.elapsed_secs, 1800);
        assert!(rep_1800.is_stale);
        assert!(!rep_1800.requires_force_refresh);

        let rep_3599 = detector.evaluate_staleness("acc_test", base_sync, base_sync + 3599);
        assert_eq!(rep_3599.elapsed_secs, 3599);
        assert!(rep_3599.is_stale);
        assert!(!rep_3599.requires_force_refresh);

        let rep_3600 = detector.evaluate_staleness("acc_test", base_sync, base_sync + 3600);
        assert_eq!(rep_3600.elapsed_secs, 3600);
        assert!(rep_3600.is_stale);
        assert!(rep_3600.requires_force_refresh);

        let rep_deep = detector.evaluate_staleness("acc_test", base_sync, base_sync + 86400);
        assert_eq!(rep_deep.elapsed_secs, 86400);
        assert!(rep_deep.is_stale);
        assert!(rep_deep.requires_force_refresh);
    }

    #[test]
    fn test_account_staleness_clock_skew_and_future_sync_epoch() {
        let detector = AccountStalenessDetector::default();

        let rep_skew = detector.evaluate_staleness("acc_skew", 2_000_000, 1_999_500);
        assert_eq!(rep_skew.elapsed_secs, 0);
        assert!(!rep_skew.is_stale);
        assert!(!rep_skew.requires_force_refresh);

        let rep_now = detector.evaluate_staleness("acc_now", 2_000_000, 2_000_000);
        assert_eq!(rep_now.elapsed_secs, 0);
        assert!(!rep_now.is_stale);
        assert!(!rep_now.requires_force_refresh);
    }

    #[test]
    fn test_account_staleness_never_synced_epoch_zero() {
        let detector = AccountStalenessDetector::default();

        let rep_uninitialized = detector.evaluate_staleness("acc_new", 0, 1_700_000_000);
        assert_eq!(rep_uninitialized.last_synced_epoch_secs, 0);
        assert_eq!(rep_uninitialized.elapsed_secs, 1_700_000_000);
        assert!(rep_uninitialized.is_stale);
        assert!(rep_uninitialized.requires_force_refresh);
    }

    #[test]
    fn test_account_staleness_auto_probe_candidate_queueing_priority() {
        let detector = AccountStalenessDetector::new(1800);
        let now = 10_000;

        let accounts = vec![
            ("acc_fresh", now - 300),
            ("acc_stale", now - 2400),
            ("acc_urgent", now - 5000),
            ("acc_ancient", 0),
        ];

        let mut reports: Vec<_> = accounts
            .into_iter()
            .map(|(id, last_synced)| detector.evaluate_staleness(id, last_synced, now))
            .collect();

        reports.sort_by(|a, b| {
            b.requires_force_refresh
                .cmp(&a.requires_force_refresh)
                .then_with(|| b.is_stale.cmp(&a.is_stale))
                .then_with(|| b.elapsed_secs.cmp(&a.elapsed_secs))
        });

        assert_eq!(reports[0].account_id, "acc_ancient");
        assert!(reports[0].requires_force_refresh);

        assert_eq!(reports[1].account_id, "acc_urgent");
        assert!(reports[1].requires_force_refresh);

        assert_eq!(reports[2].account_id, "acc_stale");
        assert!(reports[2].is_stale);
        assert!(!reports[2].requires_force_refresh);

        assert_eq!(reports[3].account_id, "acc_fresh");
        assert!(!reports[3].is_stale);
        assert!(!reports[3].requires_force_refresh);
    }
}
