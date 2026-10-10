// Phase 2 of `handle_chat_completions`: success-path response handling
// (streaming SSE assembly and non-stream JSON aggregation).
use std::collections::HashSet;
use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::mappers::openai::{transform_openai_response, OpenAIRequest};
use crate::proxy::TokenManager;

use super::chat_completions::ChatAttemptOutcome;
use super::chat_completions_send::ChatSendOutput;
use super::responses_media::{stream_chunk_has_error_event, stream_chunk_has_image_data};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn chat_completions_success(
    send: ChatSendOutput,
    openai_req: &OpenAIRequest,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    client_session_id: String,
    client_tool_names: HashSet<String>,
    original_body: Option<Value>,
    token_manager: Arc<TokenManager>,
    clean_ms: f64,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
    ttft_ms: &mut f64,
    failure_statuses: &mut FailureStatusTracker,
    last_error: &mut String,
    image_permit: &mut Option<crate::proxy::server::ImagePermit>,
) -> Result<ChatAttemptOutcome, (StatusCode, String)> {
    let ChatSendOutput {
        response,
        status,
        upstream_url,
        session_id,
        message_count,
        config,
        mapped_model,
        email,
        account_id,
        client_wants_stream,
        actual_stream,
        causal_anchor,
        upstream_req_start,
        gemini_body_for_debug,
        prefix_hash: _prefix_hash,
        ..
    } = send;
    // 5. 处理流式 vs 非流式
    if actual_stream {
        use axum::body::Body;
        use axum::response::Response;
        use futures::StreamExt;

        let meta = json!({
            "protocol": "openai",
            "trace_id": trace_id,
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
            meta,
        );

        // [P1 FIX] Enhanced Peek logic to handle heartbeats and slow start
        // Pre-read until we find meaningful content, skip heartbeats
        use crate::proxy::mappers::openai::streaming::create_openai_sse_stream_with_anchor;
        let include_usage = openai_req
            .stream_options
            .as_ref()
            .map(|o| o.include_usage)
            .unwrap_or(false);
        let mut openai_stream = create_openai_sse_stream_with_anchor(
            gemini_stream,
            openai_req.model.clone(),
            session_id,
            message_count,
            Some(client_tool_names.clone()),
            include_usage,
            Some(causal_anchor),
        );

        let mut first_data_chunk = None;
        let mut retry_this_account = false;

        // Loop to skip heartbeats during peek
        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(300), openai_stream.next())
                .await
            {
                Ok(Some(Ok(bytes))) => {
                    if bytes.is_empty() {
                        continue;
                    }

                    let text = String::from_utf8_lossy(&bytes);
                    // Skip SSE comments/pings (heartbeats)
                    if text.trim().starts_with(":") || text.trim().starts_with("data: :") {
                        tracing::debug!("[OpenAI] Skipping peek heartbeat");
                        continue;
                    }

                    // Check for error events
                    if stream_chunk_has_error_event(&bytes) {
                        tracing::warn!("[OpenAI] Error detected during peek, retrying...");
                        *last_error = "Error event during peek".to_string();
                        retry_this_account = true;
                        break;
                    }

                    // We found real data!
                    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
                    first_data_chunk = Some(bytes);
                    break;
                }
                Ok(Some(Err(e))) => {
                    tracing::warn!("[OpenAI] Stream error during peek: {}, retrying...", e);
                    *last_error = format!("Stream error during peek: {}", e);
                    retry_this_account = true;
                    break;
                }
                Ok(None) => {
                    tracing::warn!(
                        "[OpenAI] Stream ended during peek (Empty Response), retrying..."
                    );
                    *last_error = "Empty response stream during peek".to_string();
                    retry_this_account = true;
                    break;
                }
                Err(_) => {
                    tracing::warn!("[OpenAI] First chunk timeout after 300s, retrying...");
                    *last_error = "First chunk timeout".to_string();
                    retry_this_account = true;
                    break;
                }
            }
        }

        if retry_this_account {
            failure_statuses.record(StatusCode::BAD_GATEWAY);
            return Ok(ChatAttemptOutcome::Continue); // Rotate to next account
        }
        // Combine first chunk with remaining stream
        let combined_stream = futures::stream::once(async move {
            Ok::<Bytes, String>(first_data_chunk.unwrap_or_default())
        })
        .chain(openai_stream);

        // [NEW] 针对 OpenAI 流增加 300 秒空闲超时保护
        let image_permit_for_stream = image_permit.take();
        let track_image_success = config.request_type == "image_gen";
        let image_success_manager = token_manager.clone();
        let image_success_account = account_id.clone();
        let image_success_model = mapped_model.clone();
        let combined_stream = async_stream::stream! {
            let _image_permit = image_permit_for_stream;
            let mut s = Box::pin(combined_stream);
            let mut saw_image_data = false;
            let mut stream_failed = false;

            loop {
                match tokio::time::timeout(std::time::Duration::from_secs(300), s.next()).await {
                    Ok(Some(Ok(bytes))) => {
                        if stream_chunk_has_error_event(&bytes) {
                            stream_failed = true;
                        }
                        if track_image_success && stream_chunk_has_image_data(&bytes) {
                            saw_image_data = true;
                        }
                        yield Ok::<Bytes, String>(bytes);
                    }
                    Ok(Some(Err(error))) => {
                        stream_failed = true;
                        yield Err::<Bytes, String>(error);
                        break;
                    }
                    Ok(None) => break,
                    Err(_) => {
                        tracing::error!("[OpenAI-SSE] Idle timeout after 300s, terminating stream");
                        stream_failed = true;
                        yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
                        break;
                    }
                }
            }

            if track_image_success && saw_image_data && !stream_failed {
                image_success_manager.mark_account_success(&image_success_account);
                image_success_manager
                    .clear_persisted_live_limit(
                        &image_success_account,
                        Some(&image_success_model),
                    );
            }
        };
        let converted_meta = json!({
            "protocol": "openai",
            "trace_id": trace_id,
            "stage": "converted_codex_response",
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

        if client_wants_stream {
            // [MULTI-TURN] 保存本次对话的 messages 到 session store（/v1/chat/completions）
            {
                let save_msgs = openai_req
                    .messages
                    .iter()
                    .map(|m| {
                        let content_str = match &m.content {
                            Some(crate::proxy::mappers::openai::OpenAIContent::String(s)) => {
                                s.clone()
                            }
                            _ => String::new(),
                        };
                        json!({"role": m.role, "content": content_str})
                    })
                    .collect::<Vec<_>>();
                let chat_response_id = format!("chatcmpl-{}", uuid::Uuid::new_v4().simple());
                let entry = crate::proxy::http_session_store::HttpSessionEntry {
                    input_items: save_msgs,
                    instructions: String::new(),
                    model: openai_req.model.clone(),
                    last_accessed: std::time::Instant::now(),
                };
                let rid = chat_response_id.clone();
                tokio::spawn(async move {
                    crate::proxy::http_session_store::save_session(rid, entry).await;
                });
            }
            // 客户端请求流式，返回 SSE
            let body = Body::from_stream(combined_stream);
            return Ok(ChatAttemptOutcome::Respond(
                Response::builder()
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive")
                    .header("X-Accel-Buffering", "no")
                    .header("X-Account-Email", &email)
                    .header("X-Mapped-Model", &mapped_model)
                    .header("X-Session-Id", &client_session_id)
                    .header("X-Antigravity-Session-Id", &client_session_id)
                    .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                    .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                    .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                    .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                    .body(body)
                    .unwrap()
                    .into_response(),
            ));
        } else {
            // 客户端请求非流式，但内部强制转为流式
            // 收集流数据并聚合为 JSON
            use crate::proxy::mappers::openai::collector::collect_stream_to_json;
use bytes::Bytes;

            match collect_stream_to_json(combined_stream).await {
                Ok(full_response) => {
                    info!("[{}] ✓ Stream collected and converted to JSON", trace_id);
                    if debug_logger::is_enabled(&debug_cfg) {
                        let converted_response = serde_json::to_value(&full_response)
                            .unwrap_or_else(|e| json!({ "serialization_error": e.to_string() }));
                        let payload = json!({
                            "kind": "exchange_summary",
                            "protocol": "openai",
                            "trace_id": trace_id,
                            "original_codex_request": original_body.as_ref(),
                            "gemini_request": gemini_body_for_debug.as_ref(),
                            "converted_codex_response": converted_response,
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
                    return Ok(ChatAttemptOutcome::Respond(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .header("X-Account-Email", email.as_str())
                            .header("X-Mapped-Model", mapped_model.as_str())
                            .header("X-Session-Id", client_session_id.as_str())
                            .header("X-Antigravity-Session-Id", client_session_id.as_str())
                            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                            .body(Body::from(
                                serde_json::to_string(&full_response).unwrap_or_default(),
                            ))
                            .unwrap()
                            .into_response(),
                    ));
                }
                Err(e) => {
                    error!("[{}] Stream collection error: {}", trace_id, e);
                    return Ok(ChatAttemptOutcome::Respond(
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Stream collection error: {}", e),
                        )
                            .into_response(),
                    ));
                }
            }
        }
    }

    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
    let gemini_resp: Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Parse error: {}", e)))?;

    crate::proxy::thinking_store::capture_gemini_response(&session_id, &gemini_resp);

    // [CACHE] 从 Gemini 响应中提取缓存信息，关闭反馈循环
    // 兼容两种格式: cachedContentTokenCount (旧), total_cached_tokens (新)
    if let Some(usage) = gemini_resp.get("usageMetadata") {
        let cached = usage
            .get("total_cached_tokens")
            .or_else(|| usage.get("cachedContentTokenCount"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if cached > 0 {
            let cm = crate::proxy::cache_manager::global_cache_manager();
            cm.record_implicit_hit(&_prefix_hash);
            // [CACHE] 分层统计日志
            let stats = cm.get_layer_stats();
            tracing::info!(
                        "[Cache-Opt] Implicit cache HIT: prefix_hash={} cached_tokens={} | L1(SI): {}/{}, L2(Tools): {}/{}, L3(Prefix): {}/{}",
                        &_prefix_hash[.._prefix_hash.len().min(16)],
                        cached,
                        stats.si_hits, stats.si_total,
                        stats.tools_hits, stats.tools_total,
                        stats.prefix_hits, stats.prefix_total,
                    );
        }
    }

    let openai_response = transform_openai_response(
        &gemini_resp,
        Some(&session_id),
        message_count,
        Some(&client_tool_names),
    );
    if debug_logger::is_enabled(&debug_cfg) {
        let converted_response = serde_json::to_value(&openai_response)
            .unwrap_or_else(|e| json!({ "serialization_error": e.to_string() }));
        let payload = json!({
            "kind": "exchange_summary",
            "protocol": "openai",
            "trace_id": trace_id,
            "original_codex_request": original_body.as_ref(),
            "gemini_request": gemini_body_for_debug.as_ref(),
            "gemini_raw_response": gemini_resp,
            "converted_codex_response": converted_response,
        });
        debug_logger::write_exchange_payload(
            &debug_cfg,
            Some(&trace_id),
            "exchange_summary",
            &payload,
        )
        .await;
    }
    return Ok(ChatAttemptOutcome::Respond(
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("X-Account-Email", email.as_str())
            .header("X-Mapped-Model", mapped_model.as_str())
            .header("X-Session-Id", client_session_id.as_str())
            .header("X-Antigravity-Session-Id", client_session_id.as_str())
            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
            .body(Body::from(
                serde_json::to_string(&openai_response).unwrap_or_default(),
            ))
            .unwrap()
            .into_response(),
    ));
}
