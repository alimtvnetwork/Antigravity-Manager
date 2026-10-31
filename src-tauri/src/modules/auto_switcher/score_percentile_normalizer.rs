// Candidate Score Percentile Normalizer (Imp-520)
// Maps heterogeneous multi-criteria candidate scores into standard percentile
// bands (0.0 to 1000.0) for uniform scoreboard sorting and UI visual bars.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCandidateScore {
    pub account_id: String,
    pub raw_score: f64,
    pub normalized_score: f64,
    pub percentile_band: u32, // 0 to 10
}

pub struct ScorePercentileNormalizer;

impl ScorePercentileNormalizer {
    /// Normalizes a single raw candidate score against a defined maximum expected ceiling.
    pub fn normalize_score(raw: f64, max_ceiling: f64) -> f64 {
        if raw <= 0.0 || max_ceiling <= 0.0 {
            return 0.0;
        }
        let ratio = (raw / max_ceiling).clamp(0.0, 1.0);
        (ratio * 1000.0).round()
    }

    /// Normalizes a batch of candidates relative to the top performer in the pool.
    pub fn normalize_batch(candidates: &[(String, f64)]) -> Vec<NormalizedCandidateScore> {
        if candidates.is_empty() {
            return Vec::new();
        }

        let max_raw = candidates
            .iter()
            .map(|(_, s)| *s)
            .fold(0.0_f64, f64::max)
            .max(1.0);

        candidates
            .iter()
            .map(|(id, raw)| {
                let normalized = Self::normalize_score(*raw, max_raw);
                let band = (normalized / 100.0).floor() as u32;
                NormalizedCandidateScore {
                    account_id: id.clone(),
                    raw_score: *raw,
                    normalized_score: normalized,
                    percentile_band: band.min(10),
                }
            })
            .collect()
    }
}
