//! Non-Deterministic Candidate Selection Jitter
//!
//! Injects subtle, deterministic-bounded score jitter (+/-0.5%) into identical or
//! near-identical candidate scores to prevent lockstep clustering and distribute traffic
//! smoothly across peer accounts in multi-instance setups.

use std::hash::{Hash, Hasher};

/// Maximum percentage deviation introduced by jitter (+/- 0.5%).
pub const MAX_JITTER_RATIO: f64 = 0.005;

/// Computes a pseudo-random jitter modifier derived from account ID and round sequence.
pub fn apply_selection_jitter(account_id: &str, raw_score: f64, rotation_round: u64) -> f64 {
    if raw_score <= 0.0 {
        return 0.0;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    account_id.hash(&mut hasher);
    rotation_round.hash(&mut hasher);
    let hash_val = hasher.finish();

    // Map 64-bit hash to a normalized float between -1.0 and 1.0
    let normalized = ((hash_val % 10000) as f64 / 5000.0) - 1.0;

    let delta = raw_score * MAX_JITTER_RATIO * normalized;
    let jittered = (raw_score + delta).max(0.0);

    jittered
}

/// Applies jitter across an entire candidate list to break exact score ties.
pub fn add_jitter_to_candidate_scores<T, F>(
    candidates: &mut [T],
    mut get_score: F,
    rotation_round: u64,
) where
    F: FnMut(&mut T) -> (&str, &mut f64),
{
    for item in candidates.iter_mut() {
        let (id, score_ref) = get_score(item);
        let jittered = apply_selection_jitter(id, *score_ref, rotation_round);
        *score_ref = jittered;
    }
}
