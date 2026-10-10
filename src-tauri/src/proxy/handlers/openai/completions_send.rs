// Phase 1 of `handle_completions`: model config, token acquisition,
// request transform, upstream send.
use std::sync::Arc;

use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::mappers::common_utils::RequestConfig;
use crate::proxy::mappers::openai::{
    transform_openai_request, transform_openai_request_with_session, OpenAIRequest,
};
use crate::proxy::monitor::UpstreamRequestBodyHolder;
use crate::proxy::server::UpstreamClient;
use crate::proxy::TokenManager;

use super::responses_history::{debug_value_without_inline_data, serialized_json_len};

/// Values produced by the send phase for the success/error phases.
pub(crate) struct CompletionsSendOutput {
    pub response: rquest::Response,
    pub status: StatusCode,
    pub upstream_url: String,
    pub session_id: String,
    pub message_count: usize,
    pub config: RequestConfig,
    pub mapped_model: String,
    pub email: String,
    pub account_id: String,
    pub access_token: String,
    pub project_id: String,
    pub client_wants_stream: bool,
    pub list_response: bool,
    pub causal_anchor: Option<String>,
    pub upstream_req_start: std::time::Instant,
    pub gemini_body_for_debug: Option<Value>,
}

pub(crate) enum CompletionsSendOutcome {
    Respond(Response),
    ContinueLoop,
    Sent(CompletionsSendOutput),
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn completions_send(
    upstream: Arc<UpstreamClient>,
    token_manager: Arc<TokenManager>,
    openai_req: &OpenAIRequest,
    mapped_model: String,
    session_id_str: String,
    upstream_recorder: &Option<Extension<UpstreamRequestBodyHolder>>,
    attempt: usize,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    client_session_id: String,
    is_responses_api: bool,
    signature_read_key: Option<String>,
    max_attempts: usize,
    force_rotate: &mut bool,
    last_error: &mut String,
    last_email: &mut Option<String>,
    retry_credentials: &mut Option<(String, String, String, String, u64)>,
    failure_statuses: &mut FailureStatusTracker,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
    uri: &axum::extract::OriginalUri,
) -> CompletionsSendOutcome {
    let norm_start = std::time::Instant::now();
    // 3. 模型配置解析
    // 将 OpenAI 工具转为 Value 数组以便探测联网
    let tools_val: Option<Vec<Value>> = openai_req
        .tools
        .as_ref()
        .map(|list| list.iter().cloned().collect());
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &openai_req.model,
        &mapped_model,
        &tools_val,
        None, // size
        None, // quality
        None, // image_size
        None, // body
    );

    // 3. 提取 SessionId (复用)
    // [New] 使用 TokenManager 内部逻辑提取 session_id，支持粘性调度
    let session_id_str = session_id_str.clone();
    let session_id = Some(session_id_str.as_str());

    let (access_token, project_id, email, account_id, _wait_ms) =
        if let Some(credentials) = retry_credentials.take() {
            credentials
        } else {
            match token_manager
                .get_token(
                    &config.request_type,
                    force_rotate,
                    session_id,
                    &mapped_model,
                )
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    let headers = crate::proxy::handlers::common::build_token_error_headers(
                        Some(mapped_model.as_str()),
                        None,
                        &e,
                    );
                    return CompletionsSendOutcome::Respond(
                        (
                            StatusCode::SERVICE_UNAVAILABLE,
                            headers,
                            format!("Token error: {}", e),
                        )
                            .into_response(),
                    );
                }
            }
        };

    let mapped_model = token_manager
        .resolve_dynamic_model_for_account(&account_id, &mapped_model)
        .await;

    *last_email = Some(email.clone());

    info!("✓ Using account: {} (type: {})", email, config.request_type);

    let proxy_token = token_manager.get_token_by_id(&account_id);
    let tf_start = std::time::Instant::now();
    let (mut gemini_body, session_id, message_count, _prefix_hash) = if is_responses_api {
        transform_openai_request_with_session(
            openai_req,
            &project_id,
            &mapped_model,
            proxy_token.as_ref(),
            &session_id_str,
            signature_read_key.as_deref(),
            true, // is_responses_api
        )
    } else {
        transform_openai_request(openai_req, &project_id, &mapped_model, proxy_token.as_ref())
    };
    let tf_micros = tf_start.elapsed().as_micros() as u64;
    let norm_total_micros = norm_start.elapsed().as_micros() as u64;
    *norm_ms = norm_total_micros.saturating_sub(tf_micros) as f64 / 1000.0;
    *think_fill_ms = tf_micros as f64 / 1000.0;
    // Justification: non-Result return value intentionally discarded — no error channel to track
    let _ = crate::proxy::mappers::context_manager::ContextManager::apply_post_transit_context_mgmt(
        &mut gemini_body,
        &mapped_model,
    );
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
        &mut gemini_body,
    );
    crate::proxy::mappers::common_utils::ensure_gemini_payload_ends_with_user(&mut gemini_body);
    if let Some(ref recorder) = upstream_recorder {
        recorder.set_value(&gemini_body);
    }
    let gemini_body_for_debug =
        debug_logger::is_enabled(&debug_cfg).then(|| debug_value_without_inline_data(&gemini_body));
    if debug_logger::is_enabled(&debug_cfg) {
        let payload = json!({
            "kind": "v1internal_request",
            "protocol": "openai",
            "trace_id": trace_id,
            "request_path": uri.path(),
            "original_model": openai_req.model,
            "mapped_model": mapped_model,
            "request_type": config.request_type,
            "attempt": attempt,
            "v1internal_request": gemini_body_for_debug.as_ref(),
        });
        debug_logger::write_exchange_payload(
            &debug_cfg,
            Some(&trace_id),
            "v1internal_request",
            &payload,
        )
        .await;
    }

    // [DEBUG v4.2.0] Detailed size analysis of Gemini request body
    if let Some(contents) = gemini_body.get("contents").and_then(|c| c.as_array()) {
        let mut sizes = Vec::new();
        for (idx, msg) in contents.iter().enumerate() {
            let role = msg
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("unknown");
            sizes.push(format!(
                "msg_{}[{}]: {} chars",
                idx,
                role,
                serialized_json_len(msg)
            ));
        }

        let system_instruction_len = gemini_body
            .get("request")
            .and_then(|r| r.get("systemInstruction"))
            .map(serialized_json_len)
            .unwrap_or(0);

        let tools_len = gemini_body
            .get("request")
            .and_then(|r| r.get("tools"))
            .map(serialized_json_len)
            .unwrap_or(0);

        tracing::info!(
                "[Codex-Token-Analysis] Total parts: {}. SystemInstruction: {} chars, Tools: {} chars. Content sizes: {:?}",
                contents.len(),
                system_instruction_len,
                tools_len,
                sizes
            );
    }

    // [AUTO-CONVERSION] For Legacy/Codex as well
    let client_wants_stream = openai_req.stream;
    let force_stream_internally = !client_wants_stream;
    let list_response = client_wants_stream || force_stream_internally;
    let method = if list_response {
        "streamGenerateContent"
    } else {
        "generateContent"
    };
    let query_string = if list_response { Some("alt=sse") } else { None };

    let mut extra_headers = std::collections::HashMap::new();
    extra_headers.insert("x-session-id".to_string(), client_session_id.clone());

    let preceding_turn_anchor = gemini_body
        .get("request")
        .and_then(|r| r.get("contents"))
        .or_else(|| gemini_body.get("contents"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.last())
        .cloned();
    let causal_anchor =
        crate::proxy::thinking_store::compute_causal_anchor(preceding_turn_anchor.as_ref());

    let upstream_req_start = std::time::Instant::now();
    let call_result = match upstream
        .call_v1_internal_with_headers(
            method,
            &access_token,
            gemini_body.clone(),
            query_string,
            extra_headers,
            Some(account_id.as_str()),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            *last_error = e.clone();
            failure_statuses.record(StatusCode::BAD_GATEWAY);
            debug!(
                "Codex Request failed on attempt {}/{}: {}",
                attempt + 1,
                max_attempts,
                e
            );
            return CompletionsSendOutcome::ContinueLoop;
        }
    };

    let response = call_result.response;
    let upstream_url = response.url().to_string();
    let status = response.status();
    CompletionsSendOutcome::Sent(CompletionsSendOutput {
        response,
        status,
        upstream_url,
        session_id,
        message_count,
        config,
        mapped_model,
        email,
        account_id,
        access_token,
        project_id,
        client_wants_stream,
        list_response,
        causal_anchor,
        upstream_req_start,
        gemini_body_for_debug,
    })
}
