// V2 request wrapper orchestrator (split from wrapper.rs).
// NOTE: wrap_request_v2 underwent behavior-preserving phase extraction
// (setup/compression/prep/contents/thinking/finalize) to bring files under 500 lines.
use super::v2_compression::phase_compression;
use super::v2_contents::phase_contents;
use super::v2_finalize::phase_finalize;
use super::v2_prep::phase_thinking_prep;
use super::v2_thinking::phase_thinking;
use serde_json::Value;

pub fn wrap_request_v2(
    body: &Value,
    project_id: &str,
    mapped_model: &str,
    account_id: Option<&str>,
    session_id: Option<&str>,
    token: Option<&crate::proxy::token_manager::ProxyToken>, // [NEW] Dynamic specification injection
    token_manager: Option<&std::sync::Arc<crate::proxy::TokenManager>>,
    // [NEW] Dedicated upstream client for Layer-3 backend digests.
    // `&state.upstream` must be passed in by the handler so that it can parse the proxy pool (`client_cache`) by account
    // It will take effect with the hot update of the upstream proxy; Layer-3 will not be triggered when `None` is passed.
    upstream: Option<&std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>>,
) -> Value {
    // First use the incoming mapped_model, and secondly try to get it from body
    let original_model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(mapped_model);

    // If mapped_model is empty, original_model is used
    let final_model_name = if !mapped_model.is_empty() {
        mapped_model
    } else {
        original_model
    };

    // [ADDED v4.1.24] Calculate message_count for use by requestId
    let _message_count = body
        .get("contents")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .unwrap_or(1);

    // Copy body for modification
    let mut inner_request = body.clone();

    // Deep cleaning [undefined] string (common injection in clients such as Cherry Studio)
    crate::proxy::mappers::common_utils::deep_clean_undefined(&mut inner_request, 0);

    // [PIPELINE] Unified cleaning prompt words and risk control pseudo-Headers (compatible with feature residues when third-party aggregators such as New API transfer in with Gemini native protocol)
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
        &mut inner_request,
    );

    // [FIX #1522] Inject dummy IDs for Claude models in Gemini protocol
    let is_target_claude = final_model_name.to_lowercase().contains("claude");

    let compression_level = crate::proxy::config::get_global_compression_level();

    let mut compression_applied = false;

    // Phase 1: Context compression
    phase_compression(
        &mut inner_request,
        &compression_level,
        final_model_name,
        &mut compression_applied,
        token_manager,
        upstream,
        session_id,
        project_id,
    );

    // Phase 2: Thinking-budget prep
    let prep = phase_thinking_prep(final_model_name, original_model, &inner_request);

    // Phase 3: Contents processing
    phase_contents(
        &mut inner_request,
        final_model_name,
        session_id,
        is_target_claude,
    );

    // Phase 4: Thinking budget
    phase_thinking(
        &mut inner_request,
        final_model_name,
        token,
        &prep.lower_model,
        prep.is_under_v3,
        prep.force_server_thinking,
        prep.should_inject,
    );

    // Phase 5: Finalize
    phase_finalize(
        inner_request,
        final_model_name,
        original_model,
        body,
        token,
        account_id,
        session_id,
        project_id,
        prep.is_under_v3,
    )
}
