// Unit Test Suite for Candidate Quota Recovery Forecaster (Imp-543)
// Validates time remaining calculation, imminent replenishment thresholds,
// elapsed reset timestamps, and quota percentage clamping.

#[cfg(test)]
mod tests {
    use crate::modules::auto_switcher::quota_recovery_forecaster::QuotaRecoveryForecaster;

    #[test]
    fn test_forecast_recovery_future_reset() {
        let now = 1_000_000;
        let reset = now + 7200; // Exactly 2 hours later

        let forecast = QuotaRecoveryForecaster::forecast_recovery("acc_1", 25.0, reset, now);

        assert_eq!(forecast.account_id, "acc_1");
        assert_eq!(forecast.current_quota_percent, 25.0);
        assert_eq!(forecast.seconds_until_recovery, 7200);
        assert_eq!(forecast.hours_until_recovery, 2.0);
        assert!(!forecast.is_recovery_imminent);
    }

    #[test]
    fn test_forecast_recovery_imminent_threshold() {
        let now = 1_000_000;
        // 10 minutes = 600s <= 900s imminent threshold
        let reset = now + 600;

        let forecast = QuotaRecoveryForecaster::forecast_recovery("acc_2", 0.0, reset, now);

        assert_eq!(forecast.seconds_until_recovery, 600);
        assert_eq!(forecast.hours_until_recovery, 0.17);
        assert!(forecast.is_recovery_imminent);
    }

    #[test]
    fn test_forecast_recovery_past_reset_is_zero() {
        let now = 1_000_000;
        let past_reset = now - 500;

        let forecast = QuotaRecoveryForecaster::forecast_recovery("acc_3", 5.0, past_reset, now);

        assert_eq!(forecast.seconds_until_recovery, 0);
        assert_eq!(forecast.hours_until_recovery, 0.0);
        assert!(forecast.is_recovery_imminent);
    }

    #[test]
    fn test_forecast_recovery_quota_clamping() {
        let now = 1_000_000;
        let forecast_neg = QuotaRecoveryForecaster::forecast_recovery("acc_4", -15.0, now + 3600, now);
        assert_eq!(forecast_neg.current_quota_percent, 0.0);

        let forecast_over = QuotaRecoveryForecaster::forecast_recovery("acc_5", 150.0, now + 3600, now);
        assert_eq!(forecast_over.current_quota_percent, 100.0);
    }
}
