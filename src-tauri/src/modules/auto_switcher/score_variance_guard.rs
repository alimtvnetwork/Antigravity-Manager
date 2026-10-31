//! Candidate Score Variance & Outlier Dampener
//!
//! Evaluates candidate account scores across an evaluation pool, calculates standard
//! deviation and mean metrics, and dampens extreme outlier scores to prevent rapid
//! oscillation or erratic switching driven by transient single-metric anomalies.

use tracing::debug;

/// Maximum allowable Z-score deviation before dampening is applied.
pub const MAX_ALLOWED_Z_SCORE: f64 = 2.5;

/// Score record for variance inspection.
#[derive(Debug, Clone)]
pub struct CandidateScoreItem {
    pub account_id: String,
    pub raw_score: f64,
    pub effective_score: f64,
}

/// Analyzes and dampens candidate score variance across an evaluation batch.
pub fn dampen_score_variance(scores: &mut [CandidateScoreItem]) {
    if scores.len() < 3 {
        // Statistical variance requires at least 3 samples to be meaningful
        return;
    }

    let n = scores.len() as f64;
    let mean: f64 = scores.iter().map(|s| s.raw_score).sum::<f64>() / n;

    let variance: f64 = scores
        .iter()
        .map(|s| {
            let diff = s.raw_score - mean;
            diff * diff
        })
        .sum::<f64>()
        / n;

    let std_dev = variance.sqrt();
    if std_dev < 1e-4 {
        // Zero or negligible variance, no outliers to dampen
        return;
    }

    for item in scores.iter_mut() {
        let z_score = (item.raw_score - mean) / std_dev;

        if z_score.abs() > MAX_ALLOWED_Z_SCORE {
            let clamped_z = z_score.signum() * MAX_ALLOWED_Z_SCORE;
            let dampened_score = mean + clamped_z * std_dev;

            debug!(
                account_id = %item.account_id,
                raw = item.raw_score,
                dampened = dampened_score,
                z_score = z_score,
                "Dampened outlier candidate score"
            );

            item.effective_score = dampened_score;
        } else {
            item.effective_score = item.raw_score;
        }
    }
}
