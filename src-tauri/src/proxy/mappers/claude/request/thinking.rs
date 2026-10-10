// Thinking config (split from request.rs).
// Claude 请求转换 (Claude → Gemini v1internal)
// 对应 transformClaudeRequestIn

use super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Check if thinking mode should be enabled by default for a given model
///
/// Claude Code v2.0.67+ enables thinking by default for Opus 4.5 models.
/// This function determines if the model should have thinking enabled
/// when no explicit thinking configuration is provided.
pub(crate) fn should_enable_thinking_by_default(model: &str) -> bool {
    // [保底防御] Gemini < 3 的模型（例如 gemini-2.5-flash, gemini-2.5-pro, gemini-2.0 等），直接不注入思考参数，保留思考为关
    if crate::proxy::model_specs::is_gemini_under_v3(model) {
        return false;
    }
    if crate::proxy::thinking_store::model_forces_server_thinking(model) {
        return true;
    }
    let model_lower = model.to_lowercase();

    // Enable thinking by default for Opus 4.5 and 4.6 variants
    if model_lower.contains("opus-4-5")
        || model_lower.contains("opus-4.5")
        || model_lower.contains("opus-4-6")
        || model_lower.contains("opus-4.6")
    {
        tracing::debug!(
            "[Thinking-Mode] Auto-enabling thinking for Opus model: {}",
            model
        );
        return true;
    }

    // Also enable for explicit thinking model variants
    if model_lower.contains("-thinking") {
        return true;
    }

    // Gemini 3 及以上模型（如 gemini-3.7-flash, gemini-3-pro, gemini-3.1-pro, gemini-3.8-flash 等）强行开启思考
    if crate::proxy::model_specs::is_gemini_v3_or_above(model) {
        tracing::debug!(
            "[Thinking-Mode] Auto-enabling thinking for Gemini 3+ model: {}",
            model
        );
        return true;
    }

    false
}

/// Whether the mapped target model supports Gemini `thinkingConfig`.
///
/// `gemini-pro-agent` is the mapped target of `gemini-3.1-pro-high` /
/// `gemini-3-pro-high` (`model_mapping.rs`) and is a forced-thinking Pro agent
/// model. It is kept in sync with the OpenAI protocol path
/// (`openai/request.rs` `is_gemini_3_thinking` includes `-pro-agent`) and the
/// model spec `SPEC_PRO_AGENT { include_thoughts: true }` (`variant_mapping.rs`).

/// Whether the mapped target model supports Gemini `thinkingConfig`.
///
/// `gemini-pro-agent` is the mapped target of `gemini-3.1-pro-high` /
/// `gemini-3-pro-high` (`model_mapping.rs`) and is a forced-thinking Pro agent
/// model. It is kept in sync with the OpenAI protocol path
/// (`openai/request.rs` `is_gemini_3_thinking` includes `-pro-agent`) and the
/// model spec `SPEC_PRO_AGENT { include_thoughts: true }` (`variant_mapping.rs`).
pub(crate) fn model_supports_thinking(mapped_model: &str) -> bool {
    if crate::proxy::thinking_store::model_forces_server_thinking(mapped_model) {
        return true;
    }
    if crate::proxy::model_specs::is_gemini_v3_or_above(mapped_model) {
        return true;
    }
    // [保底防御] Gemini < 3 的普通非思考模型（如 gemini-2.5-flash）不支持 thinkingConfig，严禁注入
    if crate::proxy::model_specs::is_gemini_under_v3(mapped_model) {
        return false;
    }
    mapped_model.contains("-thinking") || mapped_model.starts_with("claude-")
}

/// Minimum length for a valid thought_signature
const MIN_SIGNATURE_LENGTH: usize = 32;

/// Sentinel signature for models that support skipping signature validation
const SENTINEL_SIGNATURE: &str = "skip_thought_signature_validator";

pub(crate) fn clean_system_prompt_text(text: &str) -> String {
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::strip_pipeline_markers(text)
}

pub(crate) fn is_gemini_client_billing_metadata(_model: &str, text: &str) -> bool {
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::is_billing_metadata(text)
}
