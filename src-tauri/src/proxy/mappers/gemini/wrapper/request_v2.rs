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
    token: Option<&crate::proxy::token_manager::ProxyToken>, // [NEW] 动态规格注入
    token_manager: Option<&std::sync::Arc<crate::proxy::TokenManager>>,
    // [NEW] Layer-3 后台摘要专用的上游客户端。
    // 必须由 handler 传入 `&state.upstream`，它才能按账号解析代理池（`client_cache`）
    // 并随上游代理热更新生效；传 `None` 时 Layer-3 不触发。
    upstream: Option<&std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>>,
) -> Value {
    // 优先使用传入的 mapped_model，其次尝试从 body 获取
    let original_model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(mapped_model);

    // 如果 mapped_model 是空的，则使用 original_model
    let final_model_name = if !mapped_model.is_empty() {
        mapped_model
    } else {
        original_model
    };

    // [ADDED v4.1.24] 计算 message_count 供 requestId 使用
    let _message_count = body
        .get("contents")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .unwrap_or(1);

    // 复制 body 以便修改
    let mut inner_request = body.clone();

    // 深度清理 [undefined] 字符串 (Cherry Studio 等客户端常见注入)
    crate::proxy::mappers::common_utils::deep_clean_undefined(&mut inner_request, 0);

    // [PIPELINE] 统一清洗提示词与风控伪 Header（兼容第三方聚合器如 New API 以 Gemini 原生协议转入时的特征残留）
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
