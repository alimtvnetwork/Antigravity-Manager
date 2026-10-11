// Finalize phase (split from request.rs).
// OpenAI → Gemini 请求转换
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
    // requestId：官方 5 段形态，三适配器共用（含 unixMs → 幂等隔离）。
    // 历史教训：复用 session / message-count 的 ID 会把后续请求 pin 到一次更早的 429 结果。
    let request_id =
        common_utils::build_official_request_id(&setup.session_id, setup.message_count as u64);

    // 官方客户端指纹（企业 / GCP 账号为 jetski）—— 三适配器共用，避免指纹漂移
    let (official_user_agent, _official_ide_type) =
        common_utils::resolve_official_fingerprint(token);

    // [NEW] 动态检测是否需要标记为 agent 请求
    // 只有在请求携带 tools，或上下文包含工具调用交互时才打上 agent 标签
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
        // [CACHE] 使用重排后的字段顺序，稳定前缀在前
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

    // [CACHE:L3] 使用多层级缓存的 compute_prefix_hash 计算组合哈希
    // Layer 1 + Layer 2 的独立 hash 组合 → Layer 3 key
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

    // [CACHE:L3] 尝试利用显式缓存：查询 prefix_hash 对应的 Gemini cache_id
    // 若命中，注入 cachedContent 参数，告知 Gemini 服务端复用已缓存的前缀
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

    // [DEFENSE] 净化所有 contents 中的 inlineData，过滤或降级空数据/损坏数据
    if let Some(inner) = final_body.get_mut("request") {
        crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(inner);
    }

    (
        final_body,
        setup.session_id,
        setup.message_count,
        prefix_hash,
    )
}
