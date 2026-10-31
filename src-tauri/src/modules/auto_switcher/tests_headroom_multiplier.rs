// Unit Test Suite for Dynamic Headroom Score Multiplier (Imp-311)
// Validates non-linear sigmoid response curve, exact key point anchors,
// strict zero quota gates, ample vs fragile asymmetry, and monotonicity.

#[cfg(test)]
mod tests {
    use crate::modules::auto_switcher::headroom_score_multiplier::{
        apply_headroom_weighting, calculate_headroom_multiplier, HeadroomCurveConfig,
    };

    #[test]
    fn test_headroom_multiplier_exact_key_points() {
        let m_0 = calculate_headroom_multiplier(0.0, None);
        assert_eq!(m_0, 0.0);

        let m_1 = calculate_headroom_multiplier(1.0, None);
        assert_eq!(m_1, 1.0);

        let m_mid = calculate_headroom_multiplier(0.50, None);
        assert!((m_mid - 0.50).abs() < 1e-9);
    }

    #[test]
    fn test_headroom_multiplier_clamping_and_zero_quota_gate() {
        assert_eq!(calculate_headroom_multiplier(-0.10, None), 0.0);
        assert_eq!(calculate_headroom_multiplier(-10.0, None), 0.0);

        assert_eq!(calculate_headroom_multiplier(1.05, None), 1.0);
        assert_eq!(calculate_headroom_multiplier(5.00, None), 1.0);

        // Zero Quota Gate
        let weighted_zero = apply_headroom_weighting(1000.0, 0.0);
        assert_eq!(weighted_zero, 0.0);

        let weighted_negative = apply_headroom_weighting(850.0, -0.05);
        assert_eq!(weighted_negative, 0.0);
    }

    #[test]
    fn test_headroom_multiplier_ample_vs_fragile_asymmetry() {
        let m_ample = calculate_headroom_multiplier(0.80, None);
        let m_fragile = calculate_headroom_multiplier(0.20, None);

        assert!(m_ample > 0.85);
        assert!(m_fragile < 0.15);

        // Symmetry around 0.50: m(0.80) + m(0.20) ≈ 1.0
        assert!(((m_ample + m_fragile) - 1.0).abs() < 1e-9);

        // Multiplier ratio > 8.0x
        assert!(m_ample / m_fragile > 8.0);
    }

    #[test]
    fn test_headroom_multiplier_monotonicity() {
        let ratios = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let mut prev = -1.0;

        for &r in &ratios {
            let m = calculate_headroom_multiplier(r, None);
            assert!(m >= prev);
            prev = m;
        }
    }

    #[test]
    fn test_apply_headroom_weighting_scaling() {
        let base_score = 1000.0;

        assert_eq!(apply_headroom_weighting(base_score, 1.0), 1000.0);
        assert_eq!(apply_headroom_weighting(base_score, 0.0), 0.0);
        assert!((apply_headroom_weighting(base_score, 0.50) - 500.0).abs() < 1e-6);

        assert_eq!(apply_headroom_weighting(0.0, 0.95), 0.0);
    }

    #[test]
    fn test_headroom_multiplier_custom_curve_config() {
        let steep_cfg = HeadroomCurveConfig {
            steepness: 12.0,
            midpoint: 0.60,
        };

        let m_mid = calculate_headroom_multiplier(0.60, Some(steep_cfg));
        assert!((m_mid - 0.50).abs() < 1e-9);

        assert_eq!(calculate_headroom_multiplier(0.0, Some(steep_cfg)), 0.0);
        assert_eq!(calculate_headroom_multiplier(1.0, Some(steep_cfg)), 1.0);
    }
}
