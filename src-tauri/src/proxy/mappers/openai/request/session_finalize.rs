// Finalize phase (split from request.rs).
// OpenAI → Gemini request conversion
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

use super::session_setup::SetupState;

use crate::proxy::mappers::common_utils;

pub(crate) fn phase_finalize(
    request: &super::super::models::OpenAIRequest,
    project_id: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    setup: &super::session_setup::SetupState,
    contents: &[serde_json::Value],
    inner_request: serde_json::Value,
) -> (serde_json::Value, String, usize, String) {
    // requestId: Official 5-segment form, shared by three adapters (including unixMs → idempotent isolation).
    // Lesson from history: reusing session/message-count IDs will pin subsequent requests to an earlier 429 result.
    let request_id =
        common_utils::build_official_request_id(&setup.session_id, setup.message_count as u64);

    // Official client fingerprint (enterprise/GCP account is jetski) - shared by three adapters to avoid fingerprint drift
    let (official_user_agent, _official_ide_type) =
        common_utils::resolve_official_fingerprint(token);

    // [NEW] Dynamically detect whether requests need to be marked as agent
    // The agent tag is only used when the request carries tools, or when the context contains tool call interactions.
    let has_tools = inner_request
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    let has_tool_interactions = inner_request
        .get("contents")
        .map(common_utils::contents_has_tool_interactions)
        .unwrap_or(false);
    let is_agent_request =
        setup.config.request_type != "image_gen" && (has_tools || has_tool_interactions);

    let mut final_body = json!({
        "project": project_id,
        // [CACHE] Use rearranged field order, stable prefix first
        "request": inner_request,
        "model": setup.config.final_model,
        "userAgent": official_user_agent,
        // [CACHE] requestId stays last so its per-attempt value does not disturb the stable prefix.
        "requestId": request_id,
    });

    if setup.config.request_type == "image_gen" {
        final_body["requestType"] = json!("image_gen");
    } else if is_agent_request {
        final_body["requestType"] = json!("agent");
        if let Some(obj) = final_body.as_object_mut() {
            obj.insert("enabledCreditTypes".to_string(), json!(["GOOGLE_ONE_AI"]));
        }
    }

    // [CACHE:L3] Compute combined hashes using compute_prefix_hash of multi-level cache
    // Independent hash combination of Layer 1 + Layer 2 → Layer 3 key
    let prefix_hash = {
        let si_json = final_body["request"]
            .get("systemInstruction")
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        let tools_json = final_body["request"]
            .get("tools")
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        let hash =
            crate::proxy::cache_manager::CacheManager::compute_prefix_hash(&si_json, &tools_json);
        tracing::info!(
            "[Cache-Opt:L3-Prefix] prefix_hash={} model={} sid={} tokens_in_msg={}",
            &hash[..hash.len().min(16)],
            setup.config.final_model,
            &setup.session_id[..setup.session_id.len().min(8)],
            setup.message_count
        );
        hash
    };

    // [CACHE:L3] Try to use explicit caching: query the Gemini cache_id corresponding to prefix_hash
    // If hit, inject the cachedContent parameter to tell the Gemini server to reuse the cached prefix.
    let cache_manager = crate::proxy::cache_manager::global_cache_manager();
    if let Some(cache_name) = cache_manager.lookup_prefix(&prefix_hash) {
        if let Some(req_obj) = final_body["request"].as_object_mut() {
            req_obj.insert("cachedContent".to_string(), json!(cache_name));
            tracing::info!(
                "[Cache-Opt] Explicit cache HIT: prefix_hash={} cache_name={}",
                &prefix_hash[..prefix_hash.len().min(16)],
                cache_name
            );
            cache_manager.record_explicit_hit(&prefix_hash);
        }
    }

    // [DEFENSE] Sanitize inlineData in all contents, filter or degrade empty/corrupted data
    if let Some(inner) = final_body.get_mut("request") {
        crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(inner);
    }

    (
        final_body,
        setup.session_id.clone(),
        setup.message_count,
        prefix_hash,
    )
}
