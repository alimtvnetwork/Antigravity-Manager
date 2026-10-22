//! Switch Decision Telemetry Logger.
//! Emits structured diagnostic records capturing auto-switch candidate matrix evaluations,
//! winning margin differentials, and caller context for audit trails.

use serde::{Deserialize, Serialize};
use tracing::info;

/// Summary record of an auto-switcher evaluation pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTelemetryRecord {
    pub instance_id: String,
    pub trigger_reason: String,
    pub previous_account_id: Option<String>,
    pub selected_account_id: String,
    pub winning_score: f64,
    pub runner_up_score: Option<f64>,
    pub score_margin: f64,
    pub total_candidates_evaluated: usize,
    pub execution_duration_ms: u64,
}

/// Logs a decision event both through structured tracing and serialization.
pub fn log_switch_decision(record: &DecisionTelemetryRecord) {
    info!(
        target: "auto_switcher::decision",
        instance = %record.instance_id,
        trigger = %record.trigger_reason,
        prev = ?record.previous_account_id,
        selected = %record.selected_account_id,
        score = record.winning_score,
        margin = record.score_margin,
        candidates = record.total_candidates_evaluated,
        duration_ms = record.execution_duration_ms,
        "Auto-switch decision committed"
    );
}

/// Computes score differential and runner up given a sorted slice of (account_id, score).
pub fn compute_score_differential(candidates: &[(String, f64)]) -> (f64, Option<f64>, f64) {
    if candidates.is_empty() {
        return (0.0, None, 0.0);
    }

    let winning_score = candidates[0].1;
    if candidates.len() == 1 {
        return (winning_score, None, winning_score);
    }

    let runner_up = candidates[1].1;
    let margin = (winning_score - runner_up).max(0.0);
    (winning_score, Some(runner_up), margin)
}
