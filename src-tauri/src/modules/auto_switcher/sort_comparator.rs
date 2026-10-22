//! Candidate Account Sort Key Comparator.
//! Deterministic multi-criteria sorting comparator: Score DESC, Quota Percent DESC, Headroom Hours DESC, Email ASC.

use std::cmp::Ordering;

/// Candidate account sort model containing all comparison dimensions.
#[derive(Debug, Clone)]
pub struct CandidateSortEntry {
    pub account_id: String,
    pub email: String,
    pub score: f64,
    pub quota_percent: f64,
    pub headroom_hours: f64,
}

/// Multi-tier comparator implementing strict tie-breaking across candidates.
pub fn compare_candidates(a: &CandidateSortEntry, b: &CandidateSortEntry) -> Ordering {
    // 1. Primary: Score DESC
    match b.score.partial_cmp(&a.score) {
        Some(Ordering::Equal) | None => {}
        Some(non_eq) => return non_eq,
    }

    // 2. Secondary: Quota Percent DESC
    match b.quota_percent.partial_cmp(&a.quota_percent) {
        Some(Ordering::Equal) | None => {}
        Some(non_eq) => return non_eq,
    }

    // 3. Tertiary: Headroom Hours DESC
    match b.headroom_hours.partial_cmp(&a.headroom_hours) {
        Some(Ordering::Equal) | None => {}
        Some(non_eq) => return non_eq,
    }

    // 4. Quaternary: Email ASC (lexicographical tie-breaker)
    a.email.cmp(&b.email)
}

/// Sorts a slice of candidate entries in-place using deterministic tie-breaking.
pub fn sort_candidates_deterministic(candidates: &mut [CandidateSortEntry]) {
    candidates.sort_by(compare_candidates);
}
