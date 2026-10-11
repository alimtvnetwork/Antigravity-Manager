// `handle_completions` success path, streaming arm (`if list_response`).
// NOTE: this arm always diverges (it consumes `response` via `bytes_stream()`
// yet the original code uses `response` afterwards, which only compiles if
// every path diverges), so there is no fallthrough variant.
use std::collections::HashSet;
use std::sync::Arc;

use super::responses_history::save_session_unless_response_cancelled;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::Response;
use bytes::Bytes;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::http_session_store::SessionParent;
use crate::proxy::mappers::common_utils::RequestConfig;
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::thinking_store::SessionScope;

use super::chat_conversion::convert_chat_response_to_responses;
use super::responses_history::into_history_without_inline_media;
use super::responses_media::stream_chunk_has_error_event;
use axum::response::IntoResponse;

pub(crate) enum CompletionsStreamOutcome {
    Respond(Response),
    ContinueLoop,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn completions_success_stream(
    response: rquest::Response,
    mapped_model: String,
    email: String,
    upstream_req_start: std::time::Instant,
    session_id: String,
    causal_anchor: Option<String>,
    upstream_url: String,
    message_count: usize,
    status: StatusCode,
    config: &RequestConfig,
    client_wants_stream: bool,
    openai_req: &OpenAIRequest,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    original_body: Option<Value>,
    client_tool_names: HashSet<String>,
    uri: &OriginalUri,
    response_id_for_save: &String,
    routing_session_id: String,
    session_id_str: &String,
    session_scope: SessionScope,
    store_response: bool,
    is_codex_style: bool,
    assistant_turn_index: usize,
    session_parent: &mut Option<SessionParent>,
    session_save_input: &mut Vec<Value>,
    session_save_instructions: &mut String,
    clean_ms: f64,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
    ttft_ms: &mut f64,
    failure_statuses: &mut FailureStatusTracker,
    last_error: &mut String,
    gemini_body_for_debug: &Option<Value>,
    attempt: usize,
) -> CompletionsStreamOutcome {
    let is_responses_api = uri.path() == "/v1/responses";
    use axum::body::Body;
    use axum::response::Response;
    use futures::StreamExt;

    let upstream_meta = json!({
        "protocol": "openai",
        "trace_id": trace_id,
        "request_path": uri.path(),
        "original_model": openai_req.model,
        "mapped_model": mapped_model,
        "request_type": config.request_type,
        "attempt": attempt,
        "status": status.as_u16(),
        "upstream_url": upstream_url,
    });
    let gemini_stream = debug_logger::wrap_stream_with_debug(
        Box::pin(response.bytes_stream()),
        debug_cfg.clone(),
        trace_id.clone(),
        "upstream_response",
        upstream_meta,
    );

    // DECISION: Which stream to create?
    // If client wants stream: give them what they asked (Legacy/Codex SSE).
    // If forced stream: use Chat SSE + Collector, because our collector works on Chat format
    // and we already have logic to convert Chat JSON -> Legacy JSON.

    if client_wants_stream {
        let mut session_completion_rx = None;
        let mut openai_stream = if is_codex_style {
            use crate::proxy::mappers::openai::streaming::create_codex_sse_stream;
            let completion_tx = if store_response {
                let (tx, rx) = tokio::sync::oneshot::channel();
                session_completion_rx = Some(rx);
                Some(tx)
            } else {
                None
            };
            create_codex_sse_stream(
                gemini_stream,
                openai_req.model.clone(),
                session_id_str.clone(),
                message_count,
                assistant_turn_index,
                response_id_for_save.clone(),
                completion_tx,
                store_response,
            )
        } else {
            use crate::proxy::mappers::openai::streaming::create_legacy_sse_stream;
            create_legacy_sse_stream(
                gemini_stream,
                openai_req.model.clone(),
                session_id,
                message_count,
            )
        };

        // [P1 FIX] Enhanced Peek logic (Reused from above/standard)
        let mut first_data_chunk = None;
        let mut retry_this_account = false;

        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(60), openai_stream.next())
                .await
            {
                Ok(Some(Ok(bytes))) => {
                    if bytes.is_empty() {
                        continue;
                    }
                    let text = String::from_utf8_lossy(&bytes);
                    if text.trim().starts_with(":") || text.trim().starts_with("data: :") {
                        continue;
                    }
                    if stream_chunk_has_error_event(&bytes) {
                        *last_error = "Error event during peek".to_string();
                        retry_this_account = true;
                        break;
                    }
                    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
                    first_data_chunk = Some(bytes);
                    break;
                }
                Ok(Some(Err(e))) => {
                    *last_error = format!("Stream error during peek: {}", e);
                    retry_this_account = true;
                    break;
                }
                Ok(None) => {
                    *last_error = "Empty response stream".to_string();
                    retry_this_account = true;
                    break;
                }
                Err(_) => {
                    *last_error = "Timeout waiting for first data".to_string();
                    retry_this_account = true;
                    break;
                }
            }
        }

        if retry_this_account {
            failure_statuses.record(StatusCode::BAD_GATEWAY);
            return CompletionsStreamOutcome::ContinueLoop;
        }

        let combined_stream = futures::stream::once(async move {
            Ok::<Bytes, String>(first_data_chunk.unwrap_or_default())
        })
        .chain(openai_stream);
        let converted_meta = json!({
            "protocol": "openai",
            "trace_id": trace_id,
            "stage": "converted_codex_response",
            "request_path": uri.path(),
            "original_model": openai_req.model,
            "mapped_model": mapped_model,
            "request_type": config.request_type,
            "attempt": attempt,
            "status": status.as_u16(),
            "upstream_url": upstream_url,
        });
        let combined_stream = debug_logger::wrap_stream_with_debug(
            Box::pin(combined_stream),
            debug_cfg.clone(),
            trace_id.clone(),
            "converted_codex_response",
            converted_meta,
        );

        // 仅当转换器产生 response.completed 时保存本轮增量及必要输出。
        if let Some(completion_rx) = session_completion_rx {
            let save_parent = session_parent.take();
            let save_input = std::mem::take(session_save_input);
            let save_instructions = std::mem::take(session_save_instructions);
            let save_model = openai_req.model.clone();
            let rid = response_id_for_save.clone();
            tokio::spawn(async move {
                if let Ok((outputs, ack_tx)) = completion_rx.await {
                    let outputs = outputs
                        .into_iter()
                        .filter_map(into_history_without_inline_media)
                        .collect();
                    save_session_unless_response_cancelled(
                        ack_tx,
                        crate::proxy::http_session_store::save_session_delta(
                            rid,
                            save_parent,
                            save_input,
                            outputs,
                            save_instructions,
                            save_model,
                            routing_session_id,
                        ),
                    )
                    .await;
                }
            });
        }
        return CompletionsStreamOutcome::Respond(
            Response::builder()
                .header("Content-Type", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .header("X-Account-Email", &email)
                .header("X-Mapped-Model", &mapped_model)
                .header("X-Session-Id", &session_scope.client_id)
                .header("X-Antigravity-Session-Id", &session_scope.client_id)
                .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                .body(Body::from_stream(combined_stream))
                .unwrap()
                .into_response(),
        );
    } else {
        // Forced Stream Internal -> Convert to Legacy JSON
        // Use CHAT SSE Stream (so Collector can parse it)
        use crate::proxy::mappers::openai::streaming::create_openai_sse_stream_with_anchor;
        // Note: We use create_openai_sse_stream regardless of is_codex_style here,
        // because we just want the content aggregation which chat stream does well.
        let mut openai_stream = create_openai_sse_stream_with_anchor(
            gemini_stream,
            openai_req.model.clone(),
            if is_responses_api {
                response_id_for_save.clone()
            } else {
                session_id
            },
            message_count,
            Some(client_tool_names.clone()),
            true,
            causal_anchor,
        );

        // Peek Logic (Repeated for safety/correctness on this stream type)
        let mut first_data_chunk = None;
        let mut retry_this_account = false;
        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(60), openai_stream.next())
                .await
            {
                Ok(Some(Ok(bytes))) => {
                    if bytes.is_empty() {
                        continue;
                    }
                    let text = String::from_utf8_lossy(&bytes);
                    if text.trim().starts_with(":") || text.trim().starts_with("data: :") {
                        continue;
                    }
                    if stream_chunk_has_error_event(&bytes) {
                        *last_error = "Error event in internal stream".to_string();
                        retry_this_account = true;
                        break;
                    }
                    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
                    first_data_chunk = Some(bytes);
                    break;
                }
                Ok(Some(Err(e))) => {
                    *last_error = format!("Internal stream error: {}", e);
                    retry_this_account = true;
                    break;
                }
                Ok(None) => {
                    *last_error = "Empty internal stream".to_string();
                    retry_this_account = true;
                    break;
                }
                Err(_) => {
                    *last_error = "Timeout peek internal".to_string();
                    retry_this_account = true;
                    break;
                }
            }
        }
        if retry_this_account {
            failure_statuses.record(StatusCode::BAD_GATEWAY);
            return CompletionsStreamOutcome::ContinueLoop;
        }

        let combined_stream = futures::stream::once(async move {
            Ok::<Bytes, String>(first_data_chunk.unwrap_or_default())
        })
        .chain(openai_stream);
        let converted_meta = json!({
            "protocol": "openai",
            "trace_id": trace_id,
            "stage": "converted_codex_response",
            "request_path": uri.path(),
            "original_model": openai_req.model,
            "mapped_model": mapped_model,
            "request_type": config.request_type,
            "attempt": attempt,
            "status": status.as_u16(),
            "upstream_url": upstream_url,
        });
        let combined_stream = debug_logger::wrap_stream_with_debug(
            Box::pin(combined_stream),
            debug_cfg.clone(),
            trace_id.clone(),
            "converted_codex_response",
            converted_meta,
        );

        // Collect
        use crate::proxy::mappers::openai::collector::collect_stream_to_json;
        match collect_stream_to_json(combined_stream).await {
            Ok(chat_resp) => {
                if is_responses_api {
                    let mut resp = convert_chat_response_to_responses(&chat_resp);
                    resp["id"] = json!(response_id_for_save.clone());
                    let outputs = resp
                        .get("output")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(into_history_without_inline_media)
                        .collect();
                    if store_response {
                        crate::proxy::http_session_store::save_session_delta(
                            response_id_for_save.clone(),
                            session_parent,
                            session_save_input,
                            outputs,
                            session_save_instructions,
                            openai_req.model.clone(),
                            routing_session_id.clone(),
                        )
                        .await;
                    }
                    if debug_logger::is_enabled(&debug_cfg) {
                        let payload = json!({
                            "kind": "exchange_summary",
                            "protocol": "openai",
                            "trace_id": trace_id,
                            "request_path": uri.path(),
                            "original_codex_request": original_body.as_ref(),
                            "gemini_request": gemini_body_for_debug.as_ref(),
                            "converted_codex_response": resp.clone(),
                            "gemini_raw_response_ref": "see upstream_response file with the same trace_id",
                        });
                        debug_logger::write_exchange_payload(
                            &debug_cfg,
                            Some(&trace_id),
                            "exchange_summary",
                            &payload,
                        )
                        .await;
                    }

                    return CompletionsStreamOutcome::Respond(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .header("X-Account-Email", email.as_str())
                            .header("X-Mapped-Model", mapped_model.as_str())
                            .header("X-Session-Id", session_scope.client_id.as_str())
                            .header("X-Antigravity-Session-Id", session_scope.client_id.as_str())
                            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                            .body(Body::from(serde_json::to_string(&resp).unwrap_or_default()))
                            .unwrap()
                            .into_response(),
                    );
                }

                // NOW: Convert Chat Response -> Legacy Response (Same logic as below)
                let choices = chat_resp
                    .choices
                    .iter()
                    .map(|c| {
                        let mut text = match &c.message.content {
                            Some(crate::proxy::mappers::openai::OpenAIContent::String(s)) => {
                                s.clone()
                            }
                            _ => "".to_string(),
                        };
                        if let Some(ref reasoning) = c.message.reasoning_content {
                            if !reasoning.is_empty() {
                                text = format!("{}\n\n{}", reasoning, text);
                            }
                        }
                        json!({
                            "text": text,
                            "index": c.index,
                            "logprobs": null,
                            "finish_reason": c.finish_reason
                        })
                    })
                    .collect::<Vec<_>>();

                let legacy_resp = json!({
                    "id": chat_resp.id,
                    "object": "text_completion",
                    "created": chat_resp.created,
                    "model": chat_resp.model,
                    "choices": choices,
                    "usage": chat_resp.usage
                });
                if debug_logger::is_enabled(&debug_cfg) {
                    let payload = json!({
                        "kind": "exchange_summary",
                        "protocol": "openai",
                        "trace_id": trace_id,
                        "request_path": uri.path(),
                        "original_codex_request": original_body.as_ref(),
                        "gemini_request": gemini_body_for_debug.as_ref(),
                        "converted_codex_response": legacy_resp.clone(),
                        "gemini_raw_response_ref": "see upstream_response file with the same trace_id",
                    });
                    debug_logger::write_exchange_payload(
                        &debug_cfg,
                        Some(&trace_id),
                        "exchange_summary",
                        &payload,
                    )
                    .await;
                }

                return CompletionsStreamOutcome::Respond(
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .header("X-Account-Email", email.as_str())
                        .header("X-Mapped-Model", mapped_model.as_str())
                        .header("X-Session-Id", session_scope.client_id.as_str())
                        .header("X-Antigravity-Session-Id", session_scope.client_id.as_str())
                        .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                        .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                        .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                        .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                        .body(Body::from(
                            serde_json::to_string(&legacy_resp).unwrap_or_default(),
                        ))
                        .unwrap()
                        .into_response(),
                );
            }
            Err(e) => {
                return CompletionsStreamOutcome::Respond(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Stream collection error: {}", e),
                    )
                        .into_response(),
                );
            }
        }
    }
}
