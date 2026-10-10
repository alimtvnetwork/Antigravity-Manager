use super::*;
use crate::proxy::handlers::claude::attempt::AttemptState;
use crate::proxy::handlers::claude::attempt::AttemptCall;

/// Handle a successful upstream response for non-streaming requests:
/// read body, parse, transform to Claude format and return JSON.
/// Extracted from `handle_messages`. Pure code move: no logic changes.
pub(crate) async fn handle_nonstream_success(st: &mut AttemptState, call: AttemptCall) -> Response {
    let AttemptCall {
        response,
        email,
        request_with_mapped,
        session_id_str,
        ..
    } = call;
    let s_id_owned: Option<String> = Some(session_id_str);
    // 处理非流式响应
    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to read body: {}", e),
            )
                .into_response()
        }
    };

    // Debug print
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        debug!("Upstream Response for Claude st.request: {}", text);
    }

    let gemini_resp: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("Parse error: {}", e)).into_response(),
    };

    // 解包 response 字段（v1internal 格式）
    let raw = gemini_resp.get("response").unwrap_or(&gemini_resp);

    // 转换为 Gemini Response 结构
    let gemini_response: crate::proxy::mappers::claude::models::GeminiResponse =
        match serde_json::from_value(raw.clone()) {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Convert error: {}", e),
                )
                    .into_response()
            }
        };

    // Determine context limit based on model
    let context_limit = crate::proxy::mappers::claude::utils::get_context_limit_for_model(
        &request_with_mapped.model,
    );

    // 转换
    // [FIX #765] Pass session_id and model_name for signature caching
    let s_id_owned = session_id.map(|s| s.to_string());
    // [FIX #3379] Extract registered tool names for non-streaming leakage recovery
    let ns_registered_tool_names: Vec<String> = request_with_mapped
        .tools
        .as_ref()
        .map(|tools| tools.iter().filter_map(|t| t.name.clone()).collect())
        .unwrap_or_default();
    // 转换
    let claude_response = match transform_response(
        &gemini_response,
        st.scaling_enabled,
        context_limit,
        s_id_owned,
        request_with_mapped.model.clone(),
        request_with_mapped.messages.len(), // [NEW v4.0.0] Pass message count for rewind detection
        ns_registered_tool_names,           // [FIX #3379] For call:default_api leakage recovery
    ) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Transform error: {}", e),
            )
                .into_response()
        }
    };

    // [Optimization] 记录闭环日志：消耗情况
    let cache_info = if let Some(cached) = claude_response.usage.cache_read_input_tokens {
        format!(", Cached: {}", cached)
    } else {
        String::new()
    };

    tracing::info!(
        "[{}] Request finished. Model: {}, Tokens: In {}, Out {}{}",
        st.trace_id,
        request_with_mapped.model,
        claude_response.usage.input_tokens,
        claude_response.usage.output_tokens,
        cache_info
    );

    return (
        StatusCode::OK,
        [
            ("X-Account-Email", email.as_str()),
            ("X-Mapped-Model", request_with_mapped.model.as_str()),
        ],
        Json(claude_response),
    )
        .into_response();
}
