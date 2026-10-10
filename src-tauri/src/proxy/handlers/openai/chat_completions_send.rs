// Phase 1 of `handle_chat_completions`: account/token acquisition,
// request transform, upstream send. Pure move of the original loop prologue.
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
use crate::proxy::mappers::openai::{transform_openai_request, OpenAIRequest};
use crate::proxy::monitor::UpstreamRequestBodyHolder;
use crate::proxy::server::{ImagePermit, ImageScheduler, UpstreamClient};
use crate::proxy::thinking_store::SessionScope;
use crate::proxy::upstream::client::mask_email;
use crate::proxy::TokenManager;

use super::responses_history::{debug_value_without_inline_data, serialized_json_len};
use axum::response::IntoResponse;
use axum::Json;

/// Values produced by the send phase for the success/error phases.
pub(crate) struct ChatSendOutput {
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
    pub actual_stream: bool,
    pub causal_anchor: Option<String>,
    pub upstream_req_start: std::time::Instant,
    pub gemini_body_for_debug: Option<Value>,
    pub prefix_hash: String,
}

pub(crate) enum ChatSendOutcome {
    Respond(Response),
    BreakLoop,
    ContinueLoop,
    Sent(ChatSendOutput),
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn chat_completions_send(
    upstream: Arc<UpstreamClient>,
    token_manager: Arc<TokenManager>,
    image_scheduler: Arc<ImageScheduler>,
    request_timeout: u64,
    openai_req: &OpenAIRequest,
    mapped_model: String,
    session_scope: &SessionScope,
    upstream_recorder: &Option<Extension<UpstreamRequestBodyHolder>>,
    attempt: usize,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    client_session_id: String,
    max_attempts: usize,
    force_rotate: &mut bool,
    last_error: &mut String,
    last_email: &mut Option<String>,
    retry_credentials: &mut Option<(String, String, String, String, u64)>,
    failure_statuses: &mut FailureStatusTracker,
    image_permit: &mut Option<ImagePermit>,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
) -> Result<ChatSendOutcome, (StatusCode, String)> {
    let norm_start = std::time::Instant::now();
    // 将 OpenAI 工具转为 Value 数组以便探测联网
    let tools_val: Option<Vec<Value>> = openai_req
        .tools
        .as_ref()
        .map(|list| list.iter().cloned().collect());
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &openai_req.model,
        &mapped_model,
        &tools_val,
        None, // size (not used in handler, transform_openai_request handles it)
        None, // quality
        None, // image_size
        None, // body
    );

    // 3. 提取 SessionId (粘性指纹)
    let session_id = session_scope.store_key.clone();

    // 4. 获取 Token (使用准确的 request_type)
    // 关键：在重试尝试时根据 force_rotate 决定是否轮换账号
    let (access_token, project_id, email, account_id, _wait_ms) =
        if let Some(credentials) = retry_credentials.take() {
            credentials
        } else if config.request_type == "image_gen" {
            drop(image_permit.take());
            match token_manager
                .get_image_token(
                    *force_rotate,
                    Some(&session_id),
                    &mapped_model,
                    &image_scheduler,
                    request_timeout,
                )
                .await
            {
                Ok((access_token, project_id, email, account_id, wait_ms, permit)) => {
                    *image_permit = Some(permit);
                    (access_token, project_id, email, account_id, wait_ms)
                }
                Err((status, message)) => {
                    failure_statuses.record(status);
                    *last_error = message;
                    return Ok(ChatSendOutcome::BreakLoop);
                }
            }
        } else {
            match token_manager
                .get_token(
                    &config.request_type,
                    *force_rotate,
                    Some(&session_id),
                    &mapped_model,
                )
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    // [Issue #3414] Attach headers with Retry-After if temporary cooldown exists
                    let headers = crate::proxy::handlers::common::build_token_error_headers(
                        Some(mapped_model.as_str()),
                        None,
                        &e,
                    );
                    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
                        "openai",
                        StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                        mapped_model.as_str(),
                        &e,
                    );
                    return Ok(ChatSendOutcome::Respond(
                        (StatusCode::SERVICE_UNAVAILABLE, headers, Json(dual_err)).into_response(),
                    ));
                }
            }
        };

    // [NEW v4.1.29] 获取完整 Token 对象用于动态规格查询
    let proxy_token = token_manager.get_token_by_id(&account_id);
    let mapped_model = token_manager
        .resolve_dynamic_model_for_account(&account_id, &mapped_model)
        .await;

    *last_email = Some(email.clone());
    info!("✓ Using account: {} (type: {})", email, config.request_type);

    // 4. 转换请求 (返回内容包含 session_id, message_count, prefix_hash)
    let tf_start = std::time::Instant::now();
    let (mut gemini_body, session_id, message_count, _prefix_hash) =
        transform_openai_request(openai_req, &project_id, &mapped_model, proxy_token.as_ref());
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

    let actual_request_type = gemini_body
        .get("requestType")
        .and_then(|v| v.as_str())
        .unwrap_or("none (standard)");
    info!(
        "[{}] Upstream request ready -> model: {}, requestType: {}",
        trace_id, mapped_model, actual_request_type
    );
    debug!(
        "[OpenAI-Request] Transformed Gemini body: {} bytes",
        serialized_json_len(&gemini_body)
    );

    // 5. 发送请求
    let client_wants_stream = openai_req.stream;
    let force_stream_internally = !client_wants_stream;
    let actual_stream = client_wants_stream || force_stream_internally;

    if force_stream_internally {
        debug!(
            "[{}] 🔄 Auto-converting non-stream request to stream for better quota",
            trace_id
        );
    }

    let method = if actual_stream {
        "streamGenerateContent"
    } else {
        "generateContent"
    };
    let query_string = if actual_stream { Some("alt=sse") } else { None };

    // [FIX #1522] Inject Anthropic Beta Headers for Claude models (OpenAI path)
    let mut extra_headers = std::collections::HashMap::new();
    extra_headers.insert("x-session-id".to_string(), client_session_id.clone());
    if mapped_model.to_lowercase().contains("claude") {
        extra_headers.insert(
            "anthropic-beta".to_string(),
            "claude-code-20250219".to_string(),
        );
        tracing::debug!(
            "[{}] Injected Anthropic beta headers for Claude model (via OpenAI)",
            trace_id
        );
    }

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
            extra_headers.clone(),
            Some(account_id.as_str()),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            *last_error = e.clone();
            failure_statuses.record(StatusCode::BAD_GATEWAY);
            drop(image_permit.take());
            debug!(
                "OpenAI Request failed on attempt {}/{}: {}",
                attempt + 1,
                max_attempts,
                e
            );
            return Ok(ChatSendOutcome::ContinueLoop);
        }
    };

    // [NEW] 记录端点降级日志到 debug 文件
    if !call_result.fallback_attempts.is_empty() && debug_logger::is_enabled(&debug_cfg) {
        let fallback_entries: Vec<Value> = call_result
            .fallback_attempts
            .iter()
            .map(|a| {
                json!({
                    "endpoint_url": a.endpoint_url,
                    "status": a.status,
                    "error": a.error,
                })
            })
            .collect();
        let payload = json!({
            "kind": "endpoint_fallback",
            "protocol": "openai",
            "trace_id": trace_id,
            "original_model": openai_req.model,
            "mapped_model": mapped_model,
            "attempt": attempt,
            "account": mask_email(&email),
            "fallback_attempts": fallback_entries,
        });
        debug_logger::write_debug_payload(
            &debug_cfg,
            Some(&trace_id),
            "endpoint_fallback",
            &payload,
        )
        .await;
    }

    let response = call_result.response;
    // [NEW] 提取实际请求的上游端点 URL，用于日志记录和排查
    let upstream_url = response.url().to_string();
    let status = response.status();
    Ok(ChatSendOutcome::Sent(ChatSendOutput {
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
        actual_stream,
        causal_anchor,
        upstream_req_start,
        gemini_body_for_debug,
        prefix_hash: _prefix_hash,
    }))
}
