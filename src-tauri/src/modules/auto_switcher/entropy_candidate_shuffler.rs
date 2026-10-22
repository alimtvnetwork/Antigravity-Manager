//! Entropy-protected candidate shuffler.
//! Uses deterministic pseudo-random shuffling for candidate accounts within
//! tied score bands (within delta tolerance) to ensure balanced utilization
//! and eliminate affinity starvation across identical accounts.

use serde::{Deserialize, Serialize};

/// Candidate entry with scoring metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCandidate {
    pub account_id: String,
    pub score: f64,
    pub priority: i32,
}

pub struct EntropyCandidateShuffler;

impl EntropyCandidateShuffler {
    /// Shuffles candidates within tied score bands (score difference <= tolerance).
    /// Uses a lightweight linear congruential generator (LCG) seeded by current nanoseconds
    /// to avoid external dependencies.
    pub fn shuffle_tied_candidates(mut candidates: Vec<ScoredCandidate>, score_tolerance: f64) -> Vec<ScoredCandidate> {
        if candidates.len() <= 1 {
            return candidates;
        }

        // Sort descending by score first
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(123456789);

        let mut rng_state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

        // Group into bands and shuffle within each band
        let mut result = Vec::with_capacity(candidates.len());
        let mut current_band: Vec<ScoredCandidate> = Vec::new();

        for cand in candidates {
            if let Some(first) = current_band.first() {
                if (first.score - cand.score).abs() <= score_tolerance {
                    current_band.push(cand);
                    continue;
                } else {
                    // Flush current band with shuffle
                    Self::fisher_yates_shuffle(&mut current_band, &mut rng_state);
                    result.append(&mut current_band);
                }
            }
            current_band.push(cand);
        }

        if !current_band.is_empty() {
            Self::fisher_yates_shuffle(&mut current_band, &mut rng_state);
            result.append(&mut current_band);
        }

        result
    }

    fn fisher_yates_shuffle(slice: &mut [ScoredCandidate], state: &mut u64) {
        let n = slice.len();
        if n <= 1 {
            return;
        }
        for i in (1..n).rev() {
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (*state as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
}
