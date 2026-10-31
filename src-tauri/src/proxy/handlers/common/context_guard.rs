//! Model Context Length Overflow Guard.
//! Validates estimated input token volume against known model context limits, rejecting unprocessable payloads early.

use serde::{Deserialize, Serialize};

/// Standard context window token limits across foundation model families.
pub const GEMINI_15_PRO_CONTEXT_LIMIT: u64 = 2_097_152;
pub const GEMINI_FLASH_CONTEXT_LIMIT: u64 = 1_048_576;
pub const CLAUDE_35_SONNET_CONTEXT_LIMIT: u64 = 200_000;
pub const GPT4O_CONTEXT_LIMIT: u64 = 128_000;
pub const DEFAULT_FALLBACK_CONTEXT_LIMIT: u64 = 128_000;

/// Diagnostic check result for model context limit validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCheckResult {
    pub is_within_limit: bool,
    pub estimated_tokens: u64,
    pub max_tokens: u64,
    pub utilization_pct: f64,
    pub model_id: String,
}

/// Resolves known context window capacity for a model identifier.
pub fn get_model_context_limit(model_id: &str) -> u64 {
    let lower = model_id.trim().to_lowercase();
    if lower.contains("gemini-1.5-pro") || lower.contains("gemini-2.0-pro") {
        GEMINI_15_PRO_CONTEXT_LIMIT
    } else if lower.contains("gemini") {
        GEMINI_FLASH_CONTEXT_LIMIT
    } else if lower.contains("claude") || lower.contains("sonnet") || lower.contains("opus") {
        CLAUDE_35_SONNET_CONTEXT_LIMIT
    } else if lower.contains("gpt-4") || lower.contains("o1") || lower.contains("o3") {
        GPT4O_CONTEXT_LIMIT
    } else {
        DEFAULT_FALLBACK_CONTEXT_LIMIT
    }
}

/// Evaluates if input token volume exceeds the model's supported context window.
pub fn validate_context_length(model_id: &str, estimated_tokens: u64) -> ContextCheckResult {
    let max_tokens = get_model_context_limit(model_id);
    let is_within_limit = estimated_tokens <= max_tokens;
    let utilization_pct = if max_tokens > 0 {
        ((estimated_tokens as f64 / max_tokens as f64) * 10000.0).round() / 100.0
    } else {
        0.0
    };

    ContextCheckResult {
        is_within_limit,
        estimated_tokens,
        max_tokens,
        utilization_pct,
        model_id: model_id.to_string(),
    }
}
