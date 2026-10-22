#[cfg(test)]
mod tests {
    use super::super::binding_affinity::{
        calculate_binding_affinity_bonus, DEFAULT_AFFINITY_BONUS_SCORE,
    };

    #[test]
    fn test_binding_affinity_matching_default_bonus() {
        let score = calculate_binding_affinity_bonus("acc-123", Some("acc-123"), None);
        assert!((score - DEFAULT_AFFINITY_BONUS_SCORE).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binding_affinity_matching_custom_bonus() {
        let score = calculate_binding_affinity_bonus("acc-456", Some("acc-456"), Some(32.5));
        assert!((score - 32.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binding_affinity_mismatch_returns_zero() {
        let score = calculate_binding_affinity_bonus("acc-123", Some("acc-999"), None);
        assert!((score - 0.0).abs() < f64::EPSILON);

        let score_custom = calculate_binding_affinity_bonus("acc-123", Some("acc-999"), Some(50.0));
        assert!((score_custom - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binding_affinity_none_current_returns_zero() {
        let score = calculate_binding_affinity_bonus("acc-123", None, None);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binding_affinity_empty_account_ids() {
        let score_match = calculate_binding_affinity_bonus("", Some(""), None);
        assert!((score_match - DEFAULT_AFFINITY_BONUS_SCORE).abs() < f64::EPSILON);

        let score_mismatch = calculate_binding_affinity_bonus("", Some("acc-1"), None);
        assert!((score_mismatch - 0.0).abs() < f64::EPSILON);
    }
}
