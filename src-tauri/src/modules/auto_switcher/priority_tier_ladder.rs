//! Account Priority Tier Fallback Ladder
//!
//! Enforces deterministic priority tier routing (Ultra -> Pro -> Sandbox -> Free).
//! When all candidate accounts within a preferred tier are exhausted or in active cooldown,
//! seamlessly transitions to the next available tier without interrupting prompt dispatch.

use tracing::debug;

/// Account priority tiers in descending order of preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PriorityTier {
    Ultra = 4,
    Pro = 3,
    Sandbox = 2,
    Free = 1,
}

impl PriorityTier {
    /// Parses a tier string into an enum variant.
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_ascii_lowercase().trim() {
            "ultra" | "enterprise" => PriorityTier::Ultra,
            "pro" | "plus" => PriorityTier::Pro,
            "sandbox" | "preview" | "staging" => PriorityTier::Sandbox,
            _ => PriorityTier::Free,
        }
    }

    /// Returns the next fallback tier down the ladder.
    pub fn fallback_down(&self) -> Option<PriorityTier> {
        match self {
            PriorityTier::Ultra => Some(PriorityTier::Pro),
            PriorityTier::Pro => Some(PriorityTier::Sandbox),
            PriorityTier::Sandbox => Some(PriorityTier::Free),
            PriorityTier::Free => None,
        }
    }
}

/// Candidate account descriptor for tier classification.
#[derive(Debug, Clone)]
pub struct TieredCandidate {
    pub account_id: String,
    pub tier: PriorityTier,
    pub is_available: bool,
    pub score: f64,
}

/// Resolves the highest tier with available candidates and returns candidates within that tier.
pub fn resolve_highest_available_tier<'a>(
    candidates: &'a [TieredCandidate],
    min_preferred_tier: Option<PriorityTier>,
) -> (Option<PriorityTier>, Vec<&'a TieredCandidate>) {
    let mut current_tier = min_preferred_tier.unwrap_or(PriorityTier::Ultra);

    loop {
        let available_in_tier: Vec<&'a TieredCandidate> = candidates
            .iter()
            .filter(|c| c.tier == current_tier && c.is_available)
            .collect();

        if !available_in_tier.is_empty() {
            debug!(
                tier = ?current_tier,
                count = available_in_tier.len(),
                "Resolved active candidate tier"
            );
            return (Some(current_tier), available_in_tier);
        }

        match current_tier.fallback_down() {
            Some(next) => current_tier = next,
            None => break,
        }
    }

    debug!("No available candidates found across any priority tier");
    (None, Vec::new())
}
