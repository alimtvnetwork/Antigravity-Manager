//! Multi-Model Headroom Composite Scorer.
//! Computes unified multi-model composite score across standard model quota buckets,
//! factoring in Gemini 2.0 Flash, Claude 3.5 Sonnet, and GPT-4o availabilities.

/// Quota headroom breakdown across supported model families.
#[derive(Debug, Clone, Copy)]
pub struct ModelHeadroomInputs {
    pub gemini_flash_ratio: f64,
    pub gemini_pro_ratio: f64,
    pub claude_sonnet_ratio: f64,
}

/// Weights assigned to different model headroom buckets.
#[derive(Debug, Clone, Copy)]
pub struct ModelWeightMatrix {
    pub gemini_flash_weight: f64,
    pub gemini_pro_weight: f64,
    pub claude_sonnet_weight: f64,
}

impl Default for ModelWeightMatrix {
    fn default() -> Self {
        Self {
            gemini_flash_weight: 0.40,
            gemini_pro_weight: 0.35,
            claude_sonnet_weight: 0.25,
        }
    }
}

/// Computes a weighted composite score (0.0 to 100.0) from multi-model headroom inputs.
pub fn compute_composite_headroom_score(
    inputs: &ModelHeadroomInputs,
    weights: Option<ModelWeightMatrix>,
) -> f64 {
    let w = weights.unwrap_or_default();

    let flash_score = inputs.gemini_flash_ratio.clamp(0.0, 1.0) * w.gemini_flash_weight;
    let pro_score = inputs.gemini_pro_ratio.clamp(0.0, 1.0) * w.gemini_pro_weight;
    let claude_score = inputs.claude_sonnet_ratio.clamp(0.0, 1.0) * w.claude_sonnet_weight;

    let total_weight = w.gemini_flash_weight + w.gemini_pro_weight + w.claude_sonnet_weight;
    let normalized_score = (flash_score + pro_score + claude_score) / total_weight.max(0.001);

    (normalized_score * 100.0).clamp(0.0, 100.0)
}
