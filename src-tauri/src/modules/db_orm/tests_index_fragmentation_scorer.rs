use super::*;

#[cfg(test)]
mod tests {
    use super::index_fragmentation_scorer::{
        IndexFragmentationScorer, IndexMaintenanceRecommendation,
    };

    #[test]
    fn test_zero_pages_returns_none() {
        let scorer = IndexFragmentationScorer::default();
        let res = scorer.score_index("idx_empty", 0, 1.0, 0);
        assert_eq!(res.fragmentation_percent, 0.0);
        assert_eq!(res.recommendation, IndexMaintenanceRecommendation::None);
    }

    #[test]
    fn test_pristine_index_returns_none() {
        let scorer = IndexFragmentationScorer::default();
        // 100% full, 0 out-of-order pages
        let res = scorer.score_index("idx_clean", 50, 1.0, 0);
        assert_eq!(res.fragmentation_percent, 0.0);
        assert_eq!(res.recommendation, IndexMaintenanceRecommendation::None);
    }

    #[test]
    fn test_moderate_fragmentation_recommends_analyze() {
        let scorer = IndexFragmentationScorer::new(Some(30.0), Some(15.0));
        // 80% full (10% penalty), 15% out-of-order (7.5% penalty) = 17.5%
        let res = scorer.score_index("idx_moderate", 100, 0.8, 15);
        assert!(res.fragmentation_percent >= 15.0 && res.fragmentation_percent < 30.0);
        assert_eq!(
            res.recommendation,
            IndexMaintenanceRecommendation::AnalyzeOnly
        );
    }

    #[test]
    fn test_heavy_fragmentation_recommends_reindex() {
        let scorer = IndexFragmentationScorer::new(Some(30.0), Some(15.0));
        // 50% full (25% penalty), 40% out-of-order (20% penalty) = 45%
        let res = scorer.score_index("idx_bloated", 200, 0.5, 80);
        assert!(res.fragmentation_percent >= 30.0);
        assert_eq!(
            res.recommendation,
            IndexMaintenanceRecommendation::ReindexRequired
        );
    }

    #[test]
    fn test_clamped_bounds_at_extremes() {
        let scorer = IndexFragmentationScorer::default();
        // Negative fullness or extreme out-of-order clamp safely to 100%
        let res = scorer.score_index("idx_extreme", 10, -0.5, 50);
        assert_eq!(res.fragmentation_percent, 100.0);
        assert_eq!(
            res.recommendation,
            IndexMaintenanceRecommendation::ReindexRequired
        );
    }
}
