// OpenAI `/v1/chat/completions` handler (skeleton; phases live in siblings).
use axum::http::HeaderMap;
use axum::{
    body::Body, extract::Json, extract::State, http::StatusCode, response::IntoResponse,
    response::Response,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::proxy::common::client_adapter::CLIENT_ADAPTERS;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    next_rotation_attempt, FailureStatusTracker, RequestRetryState,
};
use crate::proxy::mappers::openai::{OpenAIContent, OpenAIMessage, OpenAIRequest};
use crate::proxy::server::AppState;
use crate::proxy::session_manager::SessionManager;

use super::chat_completions_error::chat_completions_error;
use super::chat_completions_normalize::apply_responses_format_fallback;
use super::chat_completions_send::{chat_completions_send, ChatSendOutcome};
use super::chat_completions_success::chat_completions_success;
use super::images_intercept::intercept_chat_to_image;
use super::responses_history::{debug_value_without_inline_data, serialized_json_len};

pub(crate) enum ChatAttemptOutcome {
    Respond(Response),
    Continue,
    Break,
}

pub async fn handle_chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap, // [CHANGED] Extract headers
    upstream_recorder: Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let clean_start = std::time::Instant::now();
    // [NEW] Check for Image Model Redirection
    let model_name = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    // [FIX] Only redirect non-native image aliases (dall-e / midjourney) to the
    // images-generations shim. Native Gemini image models (gemini-3-pro-image*) must
    // flow through the normal pipeline (transform_openai_request -> resolve_request_config),
    // which correctly sets requestType=image_gen, imageConfig (size/aspect ratio), sessionId,
    // structured requestId and per-account dynamic model resolution — matching the official
    // Antigravity client. The old shim dropped `size` and built a divergent upstream body,
    // which caused image generation to silently fail for gemini-3-pro-image.
    if (model_name.contains("image")
        || model_name.contains("dall-e")
        || model_name.contains("midjourney"))
        && !model_name.contains("gemini")
    {
        tracing::info!(
            "[ChatRedirection] Redirecting model {} to image generations",
            model_name
        );
        return intercept_chat_to_image(state, body, &model_name).await;
    }

    let debug_cfg = state.debug_logging.read().await.clone();
    let original_body = debug_logger::is_enabled(&debug_cfg).then(|| {
        crate::proxy::payload_audit::reorder_payload_fields(&debug_value_without_inline_data(&body))
    });

    apply_responses_format_fallback(&mut body);

    let normalized_interaction_ledger = body.get("_interaction_ledger").cloned();
    let mut openai_req: OpenAIRequest = serde_json::from_value(body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid request: {}", e)))?;

    // Safety: Ensure messages is not empty
    if openai_req.messages.is_empty() {
        debug!("Received request with empty messages, injecting fallback...");
        openai_req
            .messages
            .push(crate::proxy::mappers::openai::OpenAIMessage {
                role: "user".to_string(),
                content: Some(crate::proxy::mappers::openai::OpenAIContent::String(
                    " ".to_string(),
                )),
                reasoning_content: None,
                signature: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
                refusal: None,
            });
    }

    let clean_ms = clean_start.elapsed().as_micros() as f64 / 1000.0;
    let mut norm_ms = 0.0f64;
    let mut think_fill_ms = 0.0f64;
    let mut ttft_ms = 0.0f64;

    let trace_id = format!("req_{}", chrono::Utc::now().timestamp_subsec_millis());
    info!(
        "[{}] OpenAI Chat Request: {} | {} messages | stream: {}",
        trace_id,
        openai_req.model,
        openai_req.messages.len(),
        openai_req.stream
    );
    let mut force_rotate = false;

    if debug_logger::is_enabled(&debug_cfg) {
        if let Some(ledger) = normalized_interaction_ledger {
            let payload = json!({
                "kind": "normalized_interaction_ledger",
                "protocol": "openai",
                "trace_id": trace_id.clone(),
                "original_model": openai_req.model.clone(),
                "interaction_ledger": ledger,
            });
            debug_logger::write_exchange_payload(
                &debug_cfg,
                Some(&trace_id),
                "normalized_interaction_ledger",
                &payload,
            )
            .await;
        }

        // [FIX] 使用原始 body 副本记录日志，确保不丢失任何字段
        let original_payload = json!({
            "kind": "original_request",
            "protocol": "openai",
            "trace_id": trace_id,
            "original_model": openai_req.model,
            "request": original_body.as_ref(),
        });
        debug_logger::write_exchange_payload(
            &debug_cfg,
            Some(&trace_id),
            "original_request",
            &original_payload,
        )
        .await;
    }

    // [NEW] Detect Client Adapter
    let client_adapter = CLIENT_ADAPTERS
        .iter()
        .find(|a| a.matches(&headers))
        .cloned();
    if client_adapter.is_some() {
        debug!("[{}] Client Adapter detected", trace_id);
    }

    // [Variant] Resolve canonical model + variant → real model + real params.
    // Replace the client's model/thinking/max_tokens with verified real values so the
    // forwarded request matches the expected upstream format. OpenCode encodes the variant as
    // thinking.budget_tokens; we infer the tier from its magnitude.
    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;

    let model_lower = openai_req.model.to_lowercase();
    let is_v3_or_above = crate::proxy::model_specs::is_gemini_v3_or_above(&openai_req.model);
    let is_explicit_tier_model = model_lower.ends_with("-high")
        || model_lower.ends_with("-medium")
        || model_lower.ends_with("-low")
        || model_lower.ends_with("-extra-low");

    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        openai_req
            .thinking
            .as_ref()
            .and_then(|t| t.thinking_type.as_deref()),
        openai_req
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens.map(|b| b as u64))
            .or_else(|| {
                openai_req
                    .reasoning
                    .as_ref()
                    .and_then(|r| r.max_tokens.map(|b| b as u64))
            }),
        openai_req
            .reasoning_effort
            .as_deref()
            .or_else(|| {
                openai_req
                    .reasoning
                    .as_ref()
                    .and_then(|r| r.effort.as_deref())
            })
            .or_else(|| {
                openai_req
                    .thinking
                    .as_ref()
                    .and_then(|t| t.effort.as_deref())
            }),
    );
    let client_explicit_disabled = client_switch.is_disabled();

    let raw_client_budget = openai_req
        .thinking
        .as_ref()
        .and_then(|t| t.budget_tokens)
        .or_else(|| openai_req.reasoning.as_ref().and_then(|r| r.max_tokens));

    let client_budget = if is_client_control {
        raw_client_budget
    } else if is_v3_or_above || is_explicit_tier_model {
        if let Some(ref mut t) = openai_req.thinking {
            t.budget_tokens = None; // 清理客户端 budget_tokens，防止污染
        }
        if let Some(ref mut r) = openai_req.reasoning {
            r.max_tokens = None;
        }
        None
    } else {
        raw_client_budget
    };
    let effective_budget_hint = if !is_client_control && (is_explicit_tier_model || is_v3_or_above)
    {
        None
    } else {
        client_budget
    };

    let effort_hint = openai_req
        .reasoning_effort
        .as_deref()
        .or_else(|| {
            openai_req
                .reasoning
                .as_ref()
                .and_then(|r| r.effort.as_deref())
        })
        .or_else(|| {
            openai_req
                .thinking
                .as_ref()
                .and_then(|t| t.effort.as_deref())
        });
    let effort_tier = crate::proxy::common::variant_mapping::tier_from_effort(effort_hint);

    let variant_spec =
        if crate::proxy::mappers::openai::request::is_tiered_flash_model(&openai_req.model) {
            None
        } else {
            crate::proxy::common::variant_mapping::resolve_with_tier(
                &openai_req.model,
                effort_tier,
                effective_budget_hint,
            )
        };
    if let Some(spec) = variant_spec {
        tracing::info!(
            "[{}] [Variant] canonical='{}' effort={:?} budget_hint={:?} -> real_model='{}' budget={} maxOut={}",
            trace_id,
            openai_req.model,
            effort_hint,
            effective_budget_hint,
            spec.id,
            spec.thinking_budget,
            spec.max_output_tokens
        );
        openai_req.model = spec.id.to_string();
        if is_client_control && client_explicit_disabled {
            openai_req.thinking = Some(crate::proxy::mappers::openai::models::ThinkingConfig {
                thinking_type: Some("disabled".to_string()),
                budget_tokens: Some(0),
                effort: None,
            });
        } else if is_client_control && raw_client_budget.is_some() {
            openai_req.thinking = Some(crate::proxy::mappers::openai::models::ThinkingConfig {
                thinking_type: Some("enabled".to_string()),
                budget_tokens: raw_client_budget,
                effort: effort_hint.map(|s| s.to_string()),
            });
        } else if is_client_control {
            // [CRITICAL FIX] 客户端控制模式下，客户端未传数字预算（全缺省或仅传等级）
            // 严禁伪造并塞入 spec.thinking_budget (4000)！保持真实客户端状态
            if let Some(ref mut t) = openai_req.thinking {
                t.budget_tokens = None;
            }
            if let Some(ref mut r) = openai_req.reasoning {
                r.max_tokens = None;
            }
        } else if spec.thinking_budget == 0 {
            // Non-thinking checkpoint model (e.g. gemini-3.1-flash-lite): disable thinking
            // AND strip tools/tool_choice — per upstream spec §3 checkpoint requests carry
            // no tools.
            openai_req.thinking = None;
            openai_req.tools = None;
            openai_req.tool_choice = None;
        } else {
            // 网关控制模式继续保留 4000 (Medium) 权威回填
            openai_req.thinking = Some(crate::proxy::mappers::openai::models::ThinkingConfig {
                thinking_type: Some("enabled".to_string()),
                budget_tokens: Some(spec.thinking_budget),
                effort: effort_hint.map(|s| s.to_string()),
            });
        }
        openai_req.max_tokens = Some(spec.max_output_tokens);
    }

    let client_tool_names =
        crate::proxy::mappers::openai::request::extract_client_tool_names(&openai_req.tools);

    // 1. 获取 UpstreamClient (Clone handle)
    let upstream = state.upstream.clone();
    let image_scheduler = state.image_scheduler.clone();
    let request_timeout = state.request_timeout;
    let token_manager = state.token_manager;
    let pool_size = token_manager.len();
    // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
    let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(pool_size);

    let mut last_error = String::new();
    let mut last_email: Option<String> = None;
    let mut retry_state = RequestRetryState::default();
    let mut retry_credentials: Option<(String, String, String, String, u64)> = None;
    let mut image_permit = None;
    let mut failure_statuses = FailureStatusTracker::default();
    let mut used_attempts = 0;
    let mut retried_without_thinking = false;

    // 2. 模型路由解析 (移到循环外以支持在所有路径返回 X-Mapped-Model)
    let mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        &openai_req.model,
        &*state.custom_mapping.read().await,
    );
    let explicit_sid = headers
        .get("x-session-id")
        .or_else(|| headers.get("x-jeikcode-session-id"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let fallback_sid = if let Some(sid) = explicit_sid {
        sid.to_string()
    } else {
        SessionManager::extract_openai_session_id(&openai_req)
    };
    let session_scope = crate::proxy::thinking_store::SessionScope::from_headers_and_body(
        &headers,
        original_body.as_ref(),
        fallback_sid,
    );
    openai_req.session_id = Some(session_scope.store_key.clone());
    let client_session_id = session_scope.client_id.clone();

    while let Some(attempt) = next_rotation_attempt(
        &mut used_attempts,
        max_attempts,
        retry_credentials.is_some(),
    ) {
        let send = match chat_completions_send(
            upstream.clone(),
            token_manager.clone(),
            image_scheduler.clone(),
            request_timeout,
            &openai_req,
            mapped_model.clone(),
            &session_scope,
            &upstream_recorder,
            attempt,
            debug_cfg.clone(),
            trace_id.clone(),
            client_session_id.clone(),
            max_attempts,
            &mut force_rotate,
            &mut last_error,
            &mut last_email,
            &mut retry_credentials,
            &mut failure_statuses,
            &mut image_permit,
            &mut norm_ms,
            &mut think_fill_ms,
        )
        .await?
        {
            ChatSendOutcome::Respond(response) => return Ok(response),
            ChatSendOutcome::BreakLoop => break,
            ChatSendOutcome::ContinueLoop => continue,
            ChatSendOutcome::Sent(send) => send,
        };
        if send.status.is_success() {
            match chat_completions_success(
                send,
                &openai_req,
                debug_cfg.clone(),
                trace_id.clone(),
                client_session_id.clone(),
                client_tool_names.clone(),
                original_body.clone(),
                token_manager.clone(),
                clean_ms,
                &mut norm_ms,
                &mut think_fill_ms,
                &mut ttft_ms,
                &mut failure_statuses,
                &mut last_error,
                &mut image_permit,
                0,
            )
            .await?
            {
                ChatAttemptOutcome::Respond(response) => return Ok(response),
                ChatAttemptOutcome::Continue => continue,
                ChatAttemptOutcome::Break => break,
            }
        } else {
            match chat_completions_error(
                send,
                &openai_req,
                debug_cfg.clone(),
                trace_id.clone(),
                client_session_id.clone(),
                &mut failure_statuses,
                &mut force_rotate,
                &mut last_error,
                &mut retry_credentials,
                &mut retry_state,
                &mut retried_without_thinking,
                max_attempts,
                0,
                token_manager.clone(),
                client_adapter.clone(),
                pool_size,
                &mut image_permit,
            )
            .await?
            {
                ChatAttemptOutcome::Respond(response) => return Ok(response),
                ChatAttemptOutcome::Continue => continue,
                ChatAttemptOutcome::Break => break,
            }
        }
    }

    // 所有尝试均失败：仅当全部结构化失败状态均为 429 时返回 429
    let final_status = failure_statuses.final_status();
    let headers = crate::proxy::handlers::common::build_token_error_headers(
        Some(mapped_model.as_str()),
        last_email.as_deref(),
        &last_error,
    );

    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
        "openai",
        final_status.as_u16(),
        &mapped_model,
        &last_error,
    );

    Ok((final_status, headers, Json(dual_err)).into_response())
}
