use super::*;

#[cfg(test)]
mod tests {
    use super::epoch_drift_compensator::EpochDriftCompensator;

    #[test]
    fn test_initial_state_applies_safety_buffer() {
        let compensator = EpochDriftCompensator::new(Some(30));
        assert_eq!(compensator.observed_drift_sec, 0);

        // Without observed drift, effective epoch is reported + 30s buffer
        let effective = compensator.effective_reset_epoch(1000);
        assert_eq!(effective, 1030);

        // At exactly 1000, has not elapsed because buffer hasn't passed
        assert!(!compensator.has_quota_reset_elapsed(1000, 1000));
        assert!(!compensator.has_quota_reset_elapsed(1000, 1029));
        assert!(compensator.has_quota_reset_elapsed(1000, 1030));
        assert!(compensator.has_quota_reset_elapsed(1000, 1050));
    }

    #[test]
    fn test_positive_drift_compensates_earlier() {
        let mut compensator = EpochDriftCompensator::new(Some(30));
        // Server clock is 10s ahead of local clock
        compensator.record_server_timestamp(1010, 1000);
        assert_eq!(compensator.observed_drift_sec, 10);

        // Effective epoch = 1000 + 30 - 10 = 1020
        let effective = compensator.effective_reset_epoch(1000);
        assert_eq!(effective, 1020);
        assert!(compensator.has_quota_reset_elapsed(1000, 1020));
    }

    #[test]
    fn test_negative_drift_compensates_later() {
        let mut compensator = EpochDriftCompensator::new(Some(30));
        // Server clock is 10s behind local clock
        compensator.record_server_timestamp(990, 1000);
        assert_eq!(compensator.observed_drift_sec, -10);

        // Effective epoch = 1000 + 30 - (-10) = 1040
        let effective = compensator.effective_reset_epoch(1000);
        assert_eq!(effective, 1040);
        assert!(!compensator.has_quota_reset_elapsed(1000, 1035));
        assert!(compensator.has_quota_reset_elapsed(1000, 1040));
    }

    #[test]
    fn test_drift_clamping_prevents_extremes() {
        let mut compensator = EpochDriftCompensator::default();
        // Server sends extreme future timestamp (10,000s drift)
        compensator.record_server_timestamp(11000, 1000);
        // Clamped to max 300s
        assert_eq!(compensator.observed_drift_sec, 300);
    }

    #[test]
    fn test_smoothing_over_multiple_samples() {
        let mut compensator = EpochDriftCompensator::default();
        compensator.record_server_timestamp(1010, 1000); // 10s
        assert_eq!(compensator.observed_drift_sec, 10);

        compensator.record_server_timestamp(1020, 1000); // 20s
                                                         // (10 * 4 + 20) / 5 = 60 / 5 = 12s
        assert_eq!(compensator.observed_drift_sec, 12);
    }

    #[test]
    fn test_reset_clears_drift() {
        let mut compensator = EpochDriftCompensator::default();
        compensator.record_server_timestamp(1050, 1000);
        compensator.reset();
        assert_eq!(compensator.observed_drift_sec, 0);
        assert_eq!(compensator.sample_count, 0);
    }
}
