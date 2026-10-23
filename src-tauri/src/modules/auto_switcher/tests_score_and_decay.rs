// Unit Test Suite for Score Percentile Normalizer & Idle Candidate Score Decay (Imp-490 & Imp-520)
// Validates candidate score percentile scaling, batch normalization,
// half-life decay curves, and recovery boost factors.

#[cfg(test)]
mod tests {
    use super::super::candidate_score_decay::{CandidateScoreDecay, DecayParameters};
    use super::super::score_percentile_normalizer::ScorePercentileNormalizer;

    #[test]
    fn test_score_percentile_normalizer_single() {
        assert_eq!(
            ScorePercentileNormalizer::normalize_score(50.0, 100.0),
            500.0
        );
        assert_eq!(
            ScorePercentileNormalizer::normalize_score(100.0, 100.0),
            1000.0
        );
        assert_eq!(ScorePercentileNormalizer::normalize_score(0.0, 100.0), 0.0);
        assert_eq!(
            ScorePercentileNormalizer::normalize_score(-10.0, 100.0),
            0.0
        );
        assert_eq!(
            ScorePercentileNormalizer::normalize_score(150.0, 100.0),
            1000.0
        ); // Clamped
    }

    #[test]
    fn test_score_percentile_normalizer_batch() {
        let candidates = vec![
            ("acc_top".to_string(), 200.0),
            ("acc_mid".to_string(), 100.0),
            ("acc_low".to_string(), 20.0),
            ("acc_zero".to_string(), 0.0),
        ];

        let normalized = ScorePercentileNormalizer::normalize_batch(&candidates);

        assert_eq!(normalized.len(), 4);
        assert_eq!(normalized[0].account_id, "acc_top");
        assert_eq!(normalized[0].normalized_score, 1000.0);
        assert_eq!(normalized[0].percentile_band, 10);

        assert_eq!(normalized[1].account_id, "acc_mid");
        assert_eq!(normalized[1].normalized_score, 500.0);
        assert_eq!(normalized[1].percentile_band, 5);

        assert_eq!(normalized[2].account_id, "acc_low");
        assert_eq!(normalized[2].normalized_score, 100.0);
        assert_eq!(normalized[2].percentile_band, 1);

        assert_eq!(normalized[3].account_id, "acc_zero");
        assert_eq!(normalized[3].normalized_score, 0.0);
        assert_eq!(normalized[3].percentile_band, 0);
    }

    #[test]
    fn test_candidate_score_decay_curve() {
        let decay = CandidateScoreDecay::new(DecayParameters {
            half_life_secs: 1000.0,
            max_boost_factor: 1.20,
            min_dampen_factor: 0.80,
        });

        // Immediately after use (idle = 0): min_dampen_factor (0.80)
        let score_0 = decay.calculate_adjusted_score(500.0, 0.0);
        assert_eq!(score_0, 400.0); // 500 * 0.80

        // At 1 half-life (1000s): recovery ratio = 0.5 -> factor = 0.80 + 0.40 * 0.5 = 1.00
        let score_1000 = decay.calculate_adjusted_score(500.0, 1000.0);
        assert!((score_1000 - 500.0).abs() < 1.0);

        // At 10 half-lives (idle very high): approaches max_boost_factor (1.20)
        let score_high = decay.calculate_adjusted_score(500.0, 100000.0);
        assert!((score_high - 600.0).abs() < 1.0); // 500 * 1.20

        // Zero base score remains zero
        assert_eq!(decay.calculate_adjusted_score(0.0, 5000.0), 0.0);
    }
}
