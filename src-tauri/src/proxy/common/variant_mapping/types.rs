// Canonical model variant types (split from variant_mapping.rs)

/// Variant tier inferred from the client's `thinking.budget_tokens`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantTier {
    Low,
    Medium,
    High,
}

/// How an alias selects a variant tier within its canonical Gemini family.
#[derive(Debug, Clone, Copy)]
pub enum AliasPolicy {
    HonorTier,
    Fixed(VariantTier),
}

/// Static metadata and tier routing for one canonical Gemini model family.
pub struct CanonicalFamily {
    pub canonical_id: &'static str,
    pub display_name: &'static str,
    pub context_limit: u32,
    pub output_limit: u32,
    pub input_modalities: &'static [&'static str],
    pub output_modalities: &'static [&'static str],
    pub reasoning: bool,
    pub tiers: &'static [(VariantTier, RealModelSpec)],
    pub aliases: &'static [(&'static str, AliasPolicy)],
}

/// A resolved real model with its verified request params.
#[derive(Debug, Clone, Copy)]
pub struct RealModelSpec {
    /// The real model ID to put in the upstream `model` field.
    pub id: &'static str,
    /// verified thinkingBudget (0 means no thinking).
    pub thinking_budget: u32,
    /// verified maxOutputTokens.
    pub max_output_tokens: u32,
    /// Whether to include thoughts in the response.
    pub include_thoughts: bool,
    /// Whether a Claude client-selected thinking budget may pass through.
    pub preserve_client_budget: bool,
}

impl RealModelSpec {
    /// Select the upstream thinking budget for this resolved model.
    pub const fn effective_thinking_budget(&self, client_budget: Option<u32>) -> u32 {
        if self.preserve_client_budget {
            match client_budget {
                Some(budget) => budget,
                None => self.thinking_budget,
            }
        } else {
            self.thinking_budget
        }
    }
}
