//! Fast-Forward Target Account Resolver.
//! Selects the ideal account to switch to during a manual or automated Fast-Forward (FF) action,
//! enforcing exclusion of currently bound active sibling instances and depleted accounts.

use std::collections::HashSet;

/// Candidate account descriptor for Fast-Forward resolution.
#[derive(Debug, Clone)]
pub struct FfCandidate {
    pub id: String,
    pub email: String,
    pub rolling_5h_ratio: f64,
    pub is_disabled: bool,
    pub score: f64,
}

/// Resolution outcome for Fast-Forward targeting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FfResolutionOutcome {
    FoundCandidate { account_id: String, email: String },
    NoEligibleCandidates { reason: String },
}

/// Resolves the single highest-scoring account that is not currently bound to another active instance.
pub fn resolve_fast_forward_target(
    candidates: &[FfCandidate],
    active_bound_account_ids: &HashSet<String>,
    current_instance_account_id: Option<&str>,
) -> FfResolutionOutcome {
    let mut eligible: Vec<&FfCandidate> = candidates
        .iter()
        .filter(|c| {
            if c.is_disabled {
                return false;
            }
            // Skip current account being switched away from
            if let Some(curr) = current_instance_account_id {
                if c.id == curr {
                    return false;
                }
            }
            // Skip accounts in use by other running IDE instances
            if active_bound_account_ids.contains(&c.id) {
                return false;
            }
            // Candidate must have valid headroom
            c.rolling_5h_ratio > 0.05
        })
        .collect();

    if eligible.is_empty() {
        return FfResolutionOutcome::NoEligibleCandidates {
            reason: "All available accounts are either disabled, in use by running instances, or depleted".to_string(),
        };
    }

    // Sort descending by score, breaking ties by remaining quota ratio
    eligible.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.rolling_5h_ratio
                    .partial_cmp(&a.rolling_5h_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let best = eligible[0];
    FfResolutionOutcome::FoundCandidate {
        account_id: best.id.clone(),
        email: best.email.clone(),
    }
}
