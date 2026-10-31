//! Candidate Score Reason Breakdown Formatter.
//! Generates transparent audit telemetry explaining multi-factor algorithm scores.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScoreBreakdown {
    pub account_id: String,
    pub email: String,
    pub tier: String,
    pub tier_multiplier: f64,
    pub quota_4h_percent: f64,
    pub is_4h_gate_passed: bool,
    pub weekly_score: f64,
    pub final_score: f64,
    pub primary_reason: String,
}

/// Formats a complete breakdown for candidate selection audits.
pub fn format_candidate_breakdown(
    account_id: &str,
    email: &str,
    tier: &str,
    tier_multiplier: f64,
    quota_4h_percent: f64,
    weekly_score: f64,
    final_score: f64,
    is_fallback: bool,
) -> CandidateScoreBreakdown {
    let is_4h_gate_passed = quota_4h_percent >= 100.0;

    let primary_reason = if final_score <= 0.0 {
        if quota_4h_percent < 100.0 && !is_fallback {
            format!(
                "Zeroed by strict 100% 4h quota gate (current: {:.1}%)",
                quota_4h_percent
            )
        } else if weekly_score <= 0.0 {
            "Zeroed by weekly headroom or quota < 8%".to_string()
        } else {
            "Account disabled or restricted".to_string()
        }
    } else if is_fallback {
        format!(
            "Fallback proportional score: ({:.1}% / 100) * {:.2} = {:.2}",
            quota_4h_percent, weekly_score, final_score
        )
    } else {
        format!(
            "Primary eligible (100% 4h): Tier {:.1}x, Headroom Score: {:.2}",
            tier_multiplier, final_score
        )
    };

    CandidateScoreBreakdown {
        account_id: account_id.to_string(),
        email: email.to_string(),
        tier: tier.to_string(),
        tier_multiplier,
        quota_4h_percent,
        is_4h_gate_passed,
        weekly_score,
        final_score,
        primary_reason,
    }
}
