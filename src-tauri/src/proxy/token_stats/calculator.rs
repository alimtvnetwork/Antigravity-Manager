//! Token Cost Estimator Calculator.
//! Calculates approximate USD monetary expenditure based on prompt, completion, and cached token metrics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricingTier {
    pub model_pattern: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCostEstimate {
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub estimated_cost_usd: f64,
    pub currency: String,
}

/// Resolves standard pricing rates per million tokens for standard AI models.
pub fn get_model_pricing(model: &str) -> ModelPricingTier {
    let m = model.to_lowercase();
    if m.contains("flash") {
        // Gemini Flash rates: $0.075 / 1M input, $0.30 / 1M output, $0.01875 cache
        ModelPricingTier {
            model_pattern: "gemini-flash".to_string(),
            input_per_million: 0.075,
            output_per_million: 0.30,
            cache_read_per_million: 0.01875,
        }
    } else if m.contains("pro") {
        // Gemini Pro rates: $1.25 / 1M input, $5.00 / 1M output, $0.3125 cache
        ModelPricingTier {
            model_pattern: "gemini-pro".to_string(),
            input_per_million: 1.25,
            output_per_million: 5.00,
            cache_read_per_million: 0.3125,
        }
    } else if m.contains("claude") || m.contains("sonnet") {
        // Claude 3.5 Sonnet rates: $3.00 / 1M input, $15.00 / 1M output, $0.30 cache
        ModelPricingTier {
            model_pattern: "claude-sonnet".to_string(),
            input_per_million: 3.00,
            output_per_million: 15.00,
            cache_read_per_million: 0.30,
        }
    } else {
        // Default conservative baseline
        ModelPricingTier {
            model_pattern: "generic-standard".to_string(),
            input_per_million: 0.50,
            output_per_million: 2.00,
            cache_read_per_million: 0.10,
        }
    }
}

/// Computes estimated cost in USD based on input, output, and cached tokens.
pub fn estimate_token_cost(
    model: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
    cached_tokens: u64,
) -> TokenCostEstimate {
    let pricing = get_model_pricing(model);

    // Calculate non-cached prompt tokens
    let effective_prompt_tokens = prompt_tokens.saturating_sub(cached_tokens);

    let prompt_cost = (effective_prompt_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
    let completion_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_per_million;
    let cache_cost = (cached_tokens as f64 / 1_000_000.0) * pricing.cache_read_per_million;

    let total_cost = prompt_cost + completion_cost + cache_cost;

    TokenCostEstimate {
        model: model.to_string(),
        prompt_tokens,
        completion_tokens,
        cached_tokens,
        estimated_cost_usd: (total_cost * 100_000.0).round() / 100_000.0,
        currency: "USD".to_string(),
    }
}
