//! Split-SQLite Automatic Index Fragmentation Scorer (Imp-552)
//! Evaluates B-Tree index page density and out-of-order leaf structures,
//! recommending `ANALYZE` or `REINDEX` maintenance operations to prevent query plan degradation.

use serde::{Deserialize, Serialize};

pub const DEFAULT_REINDEX_THRESHOLD_PERCENT: f64 = 30.0;
pub const DEFAULT_ANALYZE_THRESHOLD_PERCENT: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexMaintenanceRecommendation {
    None,
    AnalyzeOnly,
    ReindexRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFragmentationScore {
    pub index_name: String,
    pub fragmentation_percent: f64,
    pub recommendation: IndexMaintenanceRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFragmentationScorer {
    pub reindex_threshold_percent: f64,
    pub analyze_threshold_percent: f64,
}

impl Default for IndexFragmentationScorer {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl IndexFragmentationScorer {
    pub fn new(reindex_threshold: Option<f64>, analyze_threshold: Option<f64>) -> Self {
        Self {
            reindex_threshold_percent: reindex_threshold.unwrap_or(DEFAULT_REINDEX_THRESHOLD_PERCENT),
            analyze_threshold_percent: analyze_threshold.unwrap_or(DEFAULT_ANALYZE_THRESHOLD_PERCENT),
        }
    }

    /// Evaluates fragmentation score from index page statistics.
    /// - `total_pages`: Total pages allocated to the index B-Tree.
    /// - `page_fullness_ratio`: Average percentage of page bytes utilized (0.0 to 1.0).
    /// - `out_of_order_pages`: Pages that are fragmented/non-contiguous.
    pub fn score_index(
        &self,
        index_name: impl Into<String>,
        total_pages: u32,
        page_fullness_ratio: f64,
        out_of_order_pages: u32,
    ) -> IndexFragmentationScore {
        let name = index_name.into();
        if total_pages == 0 {
            return IndexFragmentationScore {
                index_name: name,
                fragmentation_percent: 0.0,
                recommendation: IndexMaintenanceRecommendation::None,
            };
        }

        // Density penalty: 1.0 - fullness (e.g. 60% full -> 40% penalty)
        let density_penalty = (1.0 - page_fullness_ratio.clamp(0.0, 1.0)) * 50.0;

        // Sequence penalty: ratio of non-contiguous pages
        let sequence_ratio = (out_of_order_pages as f64 / total_pages as f64).clamp(0.0, 1.0);
        let sequence_penalty = sequence_ratio * 50.0;

        let fragmentation_percent = (density_penalty + sequence_penalty).clamp(0.0, 100.0);

        let recommendation = if fragmentation_percent >= self.reindex_threshold_percent {
            IndexMaintenanceRecommendation::ReindexRequired
        } else if fragmentation_percent >= self.analyze_threshold_percent {
            IndexMaintenanceRecommendation::AnalyzeOnly
        } else {
            IndexMaintenanceRecommendation::None
        };

        IndexFragmentationScore {
            index_name: name,
            fragmentation_percent,
            recommendation,
        }
    }
}
