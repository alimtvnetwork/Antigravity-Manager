//! Candidate Score JSON Snapshotter
//! Captures candidate account ranking breakdowns and factor weights
//! into structured serializable snapshots for UI scoreboard tooltips and CLI inspection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScoreEntry {
    pub account_id: String,
    pub email: String,
    pub composite_score: f64,
    pub four_hour_quota: f64,
    pub weekly_quota: f64,
    pub tier_multiplier: f64,
    pub is_cooldown: bool,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScoreSnapshot {
    pub captured_at_epoch_sec: i64,
    pub target_model: String,
    pub total_candidates: usize,
    pub eligible_candidates: usize,
    pub candidates: Vec<CandidateScoreEntry>,
}

pub struct ScoreSnapshot;

impl ScoreSnapshot {
    /// Creates a formatted JSON string of a candidate score snapshot.
    pub fn to_json(snapshot: &CandidateScoreSnapshot) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(snapshot)
    }

    /// Sorts candidates by composite_score DESC and assigns ranks 1..N.
    pub fn assign_ranks(mut candidates: Vec<CandidateScoreEntry>) -> Vec<CandidateScoreEntry> {
        candidates.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (idx, candidate) in candidates.iter_mut().enumerate() {
            candidate.rank = idx + 1;
        }

        candidates
    }
}
