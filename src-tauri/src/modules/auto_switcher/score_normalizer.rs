//! Candidate Score Rounding & Precision Normalizer.
//! Normalizes composite candidate scores into deterministic 3-decimal fixed precision (`0.000` to `1000.000`).

/// Maximum score ceiling in internal scoring algorithm.
pub const MAX_CANDIDATE_SCORE: f64 = 1000.0;
pub const MIN_CANDIDATE_SCORE: f64 = 0.0;

/// Clamps and rounds a candidate score to exactly 3 decimal places.
pub fn normalize_candidate_score(raw_score: f64) -> f64 {
    if raw_score.is_nan() || raw_score <= MIN_CANDIDATE_SCORE {
        return MIN_CANDIDATE_SCORE;
    }
    let clamped = raw_score.min(MAX_CANDIDATE_SCORE);
    (clamped * 1000.0).round() / 1000.0
}

/// Formats a normalized score into a consistent 3-decimal string representation (e.g., `"942.150"`).
pub fn format_score_fixed(score: f64) -> String {
    let normalized = normalize_candidate_score(score);
    format!("{:.3}", normalized)
}

/// Normalizes an array of scores in-place.
pub fn normalize_scores_slice(scores: &mut [f64]) {
    for s in scores.iter_mut() {
        *s = normalize_candidate_score(*s);
    }
}
