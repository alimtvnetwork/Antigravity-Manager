#[cfg(test)]
mod tests {
    use super::super::weekly_headroom_ratio::WeeklyHeadroomRatio;

    #[test]
    fn test_constants() {
        assert_eq!(WeeklyHeadroomRatio::TOTAL_WEEKLY_HOURS, 168.0);
        let expected_burn = 100.0 / 168.0;
        assert!((WeeklyHeadroomRatio::IDEAL_HOURLY_BURN_PCT - expected_burn).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_ratio_on_track() {
        let hours = 84.0;
        let quota = hours * WeeklyHeadroomRatio::IDEAL_HOURLY_BURN_PCT;
        let ratio = WeeklyHeadroomRatio::calculate_ratio(quota, hours);
        assert!((ratio - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_calculate_ratio_surplus_and_deficit() {
        // Surplus: 50% quota with 42 hours remaining (expected ~25%) -> ratio ~2.0
        let surplus_ratio = WeeklyHeadroomRatio::calculate_ratio(50.0, 42.0);
        assert!(surplus_ratio > 1.9 && surplus_ratio < 2.1);

        // Deficit: 10% quota with 84 hours remaining (expected 50%) -> ratio 0.2
        let deficit_ratio = WeeklyHeadroomRatio::calculate_ratio(10.0, 84.0);
        assert!((deficit_ratio - 0.2).abs() < 1e-4);
    }

    #[test]
    fn test_calculate_ratio_boundary_clamping() {
        // hours < 1 is clamped to 1.0
        let r_zero_hours = WeeklyHeadroomRatio::calculate_ratio(10.0, 0.0);
        let r_one_hour = WeeklyHeadroomRatio::calculate_ratio(10.0, 1.0);
        assert_eq!(r_zero_hours, r_one_hour);

        // hours > 168 is clamped to 168.0
        let r_over = WeeklyHeadroomRatio::calculate_ratio(50.0, 500.0);
        let r_168 = WeeklyHeadroomRatio::calculate_ratio(50.0, 168.0);
        assert_eq!(r_over, r_168);
    }

    #[test]
    fn test_headroom_multiplier_clamping() {
        // Extreme surplus ratio (e.g. 10.0) maps to max multiplier 1.5
        let max_mult = WeeklyHeadroomRatio::headroom_multiplier(100.0, 1.0);
        assert_eq!(max_mult, 1.5);

        // Extreme deficit ratio (0.0) maps to min multiplier 0.5
        let min_mult = WeeklyHeadroomRatio::headroom_multiplier(0.0, 100.0);
        assert_eq!(min_mult, 0.5);

        // On-track (ratio = 1.0) maps to 0.5 + 0.5*1.0 = 1.0
        let on_track_quota = 84.0 * WeeklyHeadroomRatio::IDEAL_HOURLY_BURN_PCT;
        let on_track_mult = WeeklyHeadroomRatio::headroom_multiplier(on_track_quota, 84.0);
        assert!((on_track_mult - 1.0).abs() < 1e-4);
    }
}
