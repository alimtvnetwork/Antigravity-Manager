//! Account Model Capability Tier Filter
//!
//! Evaluates candidate accounts against requested model families (e.g. Gemini 2.0 Flash,
//! Claude 3.5 Sonnet, GPT-4o) and required features (Thinking blocks, Multimodal vision),
//! filtering out accounts that lack permissions or quota models before score calculations.

use std::collections::HashSet;
use tracing::debug;

/// Requirements needed for a specific request.
#[derive(Debug, Clone)]
pub struct ModelCapabilityRequirement {
    pub target_model: String,
    pub requires_thinking: bool,
    pub requires_vision: bool,
    pub min_tier: Option<String>,
}

/// Candidate account capability metadata.
#[derive(Debug, Clone)]
pub struct AccountCapabilities {
    pub account_id: String,
    pub tier: String,
    pub supported_models: HashSet<String>,
    pub supports_thinking: bool,
    pub supports_vision: bool,
    pub is_active: bool,
}

/// Evaluates if an account fulfills the capability requirements.
pub fn is_account_eligible(
    account: &AccountCapabilities,
    req: &ModelCapabilityRequirement,
) -> bool {
    if !account.is_active {
        return false;
    }

    // Check minimum tier if specified
    if let Some(ref min_tier) = req.min_tier {
        if !account.tier.eq_ignore_ascii_case(min_tier) && account.tier.to_lowercase() < min_tier.to_lowercase() {
            return false;
        }
    }

    // Check thinking requirement
    if req.requires_thinking && !account.supports_thinking {
        return false;
    }

    // Check vision requirement
    if req.requires_vision && !account.supports_vision {
        return false;
    }

    // Check model support
    if !account.supported_models.is_empty() {
        let model_clean = req.target_model.to_ascii_lowercase();
        let matches = account.supported_models.iter().any(|m| {
            let m_clean = m.to_ascii_lowercase();
            m_clean == model_clean || model_clean.contains(&m_clean)
        });
        if !matches {
            return false;
        }
    }

    true
}

/// Filters candidate list and returns only eligible accounts.
pub fn filter_eligible_candidates<'a>(
    accounts: &'a [AccountCapabilities],
    req: &ModelCapabilityRequirement,
) -> Vec<&'a AccountCapabilities> {
    let eligible: Vec<&'a AccountCapabilities> = accounts
        .iter()
        .filter(|acc| is_account_eligible(acc, req))
        .collect();

    debug!(
        total = accounts.len(),
        eligible = eligible.len(),
        target_model = %req.target_model,
        "Filtered candidate accounts by capability"
    );

    eligible
}
