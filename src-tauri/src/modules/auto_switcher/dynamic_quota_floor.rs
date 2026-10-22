//! Dynamic quota safety floor guard.
//! Calculates adaptive minimum quota floors based on model family and prompt burst profiles
//! to prevent premature exhaustion mid-turn during high-token generation tasks.

use serde::{Deserialize, Serialize};

/// Target model family consumption tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelConsumptionTier {
    Lightweight, // e.g. Flash-Lite, GPT-4o-mini (~2k-5k tokens/turn)
    Standard,    // e.g. Flash, Claude 3.5 Haiku (~10k-20k tokens/turn)
    Heavyweight, // e.g. Claude 3.5 Sonnet, Pro (~30k-80k tokens/turn)
    Reasoning,   // e.g. o1, Gemini 2.0 Flash Thinking (~50k-128k tokens/turn)
}

impl ModelConsumptionTier {
    pub fn from_model_name(model: &str) -> Self {
        let m = model.to_ascii_lowercase();
        if m.contains("thinking") || m.contains("o1") || m.contains("o3") {
            ModelConsumptionTier::Reasoning
        } else if m.contains("sonnet") || m.contains("pro") || m.contains("opus") {
            ModelConsumptionTier::Heavyweight
        } else if m.contains("flash-lite") || m.contains("mini") {
            ModelConsumptionTier::Lightweight
        } else {
            ModelConsumptionTier::Standard
        }
    }

    /// Minimum safe remaining quota percentage (0-100) before switching away.
    pub fn minimum_safe_headroom_percent(&self) -> f64 {
        match self {
            ModelConsumptionTier::Lightweight => 3.0,
            ModelConsumptionTier::Standard => 5.0,
            ModelConsumptionTier::Heavyweight => 8.0,
            ModelConsumptionTier::Reasoning => 12.0,
        }
    }
}

/// Evaluation of candidate account quota floor sufficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicFloorEvaluation {
    pub model_tier: ModelConsumptionTier,
    pub remaining_percent: f64,
    pub minimum_required_percent: f64,
    pub is_sufficient: bool,
    pub headroom_margin: f64,
}

pub struct DynamicQuotaFloorGuard;

impl DynamicQuotaFloorGuard {
    /// Evaluates if remaining quota percent clears the safety floor for the target model.
    pub fn evaluate(model_name: &str, remaining_percent: f64) -> DynamicFloorEvaluation {
        let tier = ModelConsumptionTier::from_model_name(model_name);
        let min_required = tier.minimum_safe_headroom_percent();
        let is_sufficient = remaining_percent >= min_required;
        let margin = remaining_percent - min_required;

        DynamicFloorEvaluation {
            model_tier: tier,
            remaining_percent,
            minimum_required_percent: min_required,
            is_sufficient,
            headroom_margin: margin,
        }
    }
}
