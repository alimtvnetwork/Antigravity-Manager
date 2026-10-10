use super::*;

const MAX_RETRY_ATTEMPTS: usize = 3;

pub(crate) fn response_has_inline_image_data(value: &Value) -> bool {
    let response = value.get("response").unwrap_or(value);
    response
        .get("candidates")
        .and_then(Value::as_array)
        .is_some_and(|candidates| {
            candidates.iter().any(|candidate| {
                candidate
                    .get("content")
                    .and_then(|content| content.get("parts"))
                    .and_then(Value::as_array)
                    .is_some_and(|parts| {
                        parts.iter().any(|part| {
                            part.get("inlineData")
                                .or_else(|| part.get("inline_data"))
                                .and_then(|image| image.get("data"))
                                .and_then(Value::as_str)
                                .is_some_and(|data| !data.is_empty())
                        })
                    })
            })
        })
}

#[cfg(test)]
pub(crate) mod image_success_tests {
    use super::response_has_inline_image_data;
    use serde_json::json;

    #[test]
    fn task_gemini_image_success_requires_nonempty_payload() {
        let empty = json!({
            "response": {"candidates": [{"content": {"parts": [{"inlineData": {"data": ""}}]}}]}
        });
        let image = json!({
            "response": {"candidates": [{"content": {"parts": [{"inlineData": {"data": "AQ=="}}]}}]}
        });
        assert!(!response_has_inline_image_data(&empty));
        assert!(response_has_inline_image_data(&image));
    }
}

/// Outcome of handling a successful upstream response in `handle_generate`.
pub(crate) enum HandleSuccessOutcome {
    /// Send this response to the client.
    Respond(axum::response::Response),
    /// Stream had issues; caller should record BAD_GATEWAY and retry.
    Retry,
}

/// Handle a successful upstream response (streaming or non-streaming).
/// Extracted from `handle_generate` to keep file sizes manageable.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_generate_success(
    response: rquest::Response,
    is_stream: bool,
    debug_cfg: &crate::proxy::config::DebugLoggingConfig,
    trace_id: &str,
    model_name: &str,
    mapped_model: &str,
    request_type: &str,
    attempt: usize,
    status: StatusCode,
    upstream_url: &str,
    session_id: &str,
    client_session_id: &str,
    cloud_code_trace_id: Option<String>,
    upstream_req_start: std::time::Instant,
    token_manager: &std::sync::Arc<crate::proxy::TokenManager>,
    account_id: &str,
    email: &str,
    clean_ms: f64,
    norm_ms: f64,
    think_fill_ms: f64,
    ttft_ms: &mut f64,
    image_permit: &mut Option<crate::proxy::server::image_scheduler::ImagePermit>,
    last_error: &mut String,
) -> Result<HandleSuccessOutcome, (StatusCode, String)> {
    // 6. 响应处理
    if is_stream {
        use axum::body::Body;
        use axum::response::Response;
        use bytes::{Bytes, BytesMut};
        use futures::StreamExt;

        let meta = json!({
            "protocol": "gemini",
            "trace_id": trace_id,
            "original_model": model_name,
            "mapped_model": mapped_model,
            "request_type": request_type,
            "attempt": attempt,
            "status": status.as_u16(),
            "upstream_url": upstream_url,
        });
        let mut response_stream = debug_logger::wrap_stream_with_debug(
            Box::pin(response.bytes_stream()),
            debug_cfg.clone(),
            trace_id.to_string(),
            "upstream_response",
            meta,
        );
        let mut buffer = BytesMut::new();
        let s_id = session_id.clone(); // Clone for stream closure

        // [FIX #859] Implement peek logic for Gemini stream to prevent 0-token 200 OK
        let mut first_chunk = None;
        let mut retry_gemini = false;

        // [NEW] 实施双阶段超时：第一阶段为 FirstChunkTimeout (300s / 5min)
        // 这精准对齐了官方 Worker 在模型冷启动（Initialization）阶段的极度耐心
        match tokio::time::timeout(std::time::Duration::from_secs(300), response_stream.next())
            .await
        {
            Ok(Some(Ok(bytes))) => {
                if bytes.is_empty() {
                    tracing::warn!("[Gemini] Empty first chunk received, retrying...");
                    retry_gemini = true;
                } else {
                    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
                    first_chunk = Some(bytes);
                }
            }
            Ok(Some(Err(e))) => {
                tracing::warn!("[Gemini] Stream error during peek: {}, retrying...", e);
                *last_error = format!("Stream error: {}", e);
                retry_gemini = true;
            }
            Ok(None) => {
                tracing::warn!("[Gemini] Stream ended immediately, retrying...");
                *last_error = "Empty response".to_string();
                retry_gemini = true;
            }
            Err(_) => {
                tracing::warn!("[Gemini] First chunk timeout after 300s, retrying...");
                *last_error = "First chunk timeout".to_string();
                retry_gemini = true;
            }
        }

        if retry_gemini {
            return Ok(HandleSuccessOutcome::Retry);
        }
        let s_id_for_stream = s_id.clone();
        let model_name_for_stream = mapped_model.clone();
        let image_permit_for_stream = image_permit.take();
        let track_image_success = request_type == "image_gen";
        let image_success_manager = token_manager.clone();
        let image_success_account = account_id.clone();
        let image_success_model = mapped_model.clone();
        let stream = async_stream::stream! {
            let _image_permit = image_permit_for_stream;
            let mut first_data = first_chunk;
            let mut meta_sent = false;
            let mut saw_image_data = false;
            let mut stream_failed = false;
            let mut thinking_acc = crate::proxy::thinking_store::TurnAccumulator::new();

            loop {
                // [NEW] 阶段 6.2: 补全 __cloudCodeMeta 响应元数据透传
                // 官方 Worker 会将 TraceID 作为 SSE 流的第 0 个数据包下发
                if !meta_sent {
                    if let Some(tid) = &cloud_code_trace_id {
                        let meta_pkg = serde_json::json!({
                            "__cloudCodeMeta": {
                                "traceId": tid
                            }
                        });
                        yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&meta_pkg).unwrap())));
                    }
                    meta_sent = true;
                }

                let item = if let Some(fd) = first_data.take() {
                    Some(Ok(fd))
                } else {
                    // [NEW] 第二阶段为 StreamIdleTimeout (300s / 5min)
                    match tokio::time::timeout(std::time::Duration::from_secs(300), response_stream.next()).await {
                        Ok(next_item) => next_item,
                        Err(_) => {
                            error!("[Gemini-SSE] Idle timeout after 300s, terminating stream");
                            stream_failed = true;
                            None
                        }
                    }
                };

                let bytes = match item {
                    Some(Ok(b)) => b,
                    Some(Err(e)) => {
                        error!("[Gemini-SSE] Stream error: {}", e);
                        stream_failed = true;
                        let error_json = serde_json::json!({
                            "id": &s_id_for_stream,
                            "object": "chat.completion.chunk",
                            "model": &model_name_for_stream,
                            "choices": [
                                {
                                    "index": 0,
                                    "delta": {
                                        "content": format!("\n[Stream Error] {}", e)
                                    },
                                    "finish_reason": "error"
                                }
                            ]
                        });
                        yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error_json).unwrap_or_default())));
                        yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
                        break;
                    }
                    None => break,
                };

                debug!("[Gemini-SSE] Received chunk: {} bytes", bytes.len());
                buffer.extend_from_slice(&bytes);
                while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line_raw = buffer.split_to(pos + 1);
                    if let Ok(line_str) = std::str::from_utf8(&line_raw) {
                        let line = line_str.trim();
                        if line.is_empty() { continue; }

                        if line.starts_with("data: ") {
                            let json_part = line.trim_start_matches("data: ").trim();
                            if json_part == "[DONE]" {
                                yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
                                continue;
                            }

                            match serde_json::from_str::<Value>(json_part) {
                                Ok(mut json) => {
                                    if track_image_success && response_has_inline_image_data(&json) {
                                        saw_image_data = true;
                                    }
                                    // [FIX #765] Extract thoughtSignature from stream
                                    let inner_val = if json.get("response").is_some() {
                                        json.get("response")
                                    } else {
                                        Some(&json)
                                    };

                                    if let Some(resp) = inner_val {
                                        if let Some(candidates) = resp.get("candidates").and_then(|c| c.as_array()) {
                                            for cand in candidates {
                                                if let Some(parts) = cand.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                                    for part in parts {
                                                        thinking_acc.ingest_part(part);
                                                        if let Some(sig) = part.get("thoughtSignature").and_then(|s| s.as_str()) {
                                                            crate::proxy::SignatureCache::global()
                                                                .cache_session_signature(&s_id_for_stream, sig.to_string(), 1);
                                                            debug!("[Gemini-SSE] Cached session signature (len: {}) for session: {}", sig.len(), s_id_for_stream);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // [FIX #1522] Inject Tool ID into Stream Response
                                    crate::proxy::mappers::gemini::wrapper::inject_ids_to_response(&mut json, &model_name_for_stream);

                                    // Unwrap v1internal response wrapper
                                    if let Some(inner) = json.get_mut("response").map(|v| v.take()) {
                                        let new_line = format!("data: {}\n\n", serde_json::to_string(&inner).unwrap_or_default());
                                        yield Ok::<Bytes, String>(Bytes::from(new_line));
                                    } else {
                                        yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&json).unwrap_or_default())));
                                    }
                                }
                                Err(e) => {
                                    debug!("[Gemini-SSE] JSON parse error: {}, passing raw line", e);
                                    stream_failed = true;
                                    yield Ok::<Bytes, String>(Bytes::from(format!("{}\n\n", line)));
                                }
                            }
                        } else {
                            // Non-data lines (comments, etc.)
                            yield Ok::<Bytes, String>(Bytes::from(format!("{}\n\n", line)));
                        }
                    } else {
                        // Non-UTF8 data? Just pass it through or skip
                        debug!("[Gemini-SSE] Non-UTF8 line encountered");
                        yield Ok::<Bytes, String>(line_raw.freeze());
                    }
                }
            }

            // 仅在流式完整传输、没有发生网络中断或异常失败时，才原子提交思维链到持久化存储
            // 防止因 connection reset by peer / unexpected EOF 导致半截残废思维块污染历史记忆
            if !stream_failed {
                thinking_acc.commit(&s_id_for_stream);
            } else {
                warn!("[Gemini-SSE] Stream terminated prematurely or failed, discarding partial thinking block to prevent context poisoning: session={}", s_id_for_stream);
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

        if is_stream {
            let body = Body::from_stream(stream);
            return Ok(HandleSuccessOutcome::Respond(
                Response::builder()
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive")
                    .header("X-Accel-Buffering", "no")
                    .header("X-Account-Email", email)
                    .header("X-Mapped-Model", mapped_model)
                    .header("X-Session-Id", client_session_id)
                    .header("X-Antigravity-Session-Id", client_session_id)
                    .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                    .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                    .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                    .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                    .body(body)
                    .unwrap()
                    .into_response(),
            ));
        } else {
            // Collect to JSON
            use crate::proxy::mappers::gemini::collector::collect_stream_to_json;
            match collect_stream_to_json(Box::pin(stream), &s_id).await {
                Ok(gemini_resp) => {
                    info!(
                        "[{}] ✓ Stream collected and converted to JSON (Gemini)",
                        session_id
                    );
                    let unwrapped = unwrap_response(&gemini_resp);
                    return Ok(HandleSuccessOutcome::Respond(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .header("X-Account-Email", email)
                            .header("X-Mapped-Model", mapped_model)
                            .header("X-Session-Id", client_session_id)
                            .header("X-Antigravity-Session-Id", client_session_id)
                            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                            .body(Body::from(serde_json::to_string(&unwrapped).unwrap()))
                            .unwrap()
                            .into_response(),
                    ));
                }
                Err(e) => {
                    error!("Stream collection error: {}", e);
                    return Ok(HandleSuccessOutcome::Respond(
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
    let mut gemini_resp: Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Parse error: {}", e)))?;

    // [FIX #1522] Inject Tool ID into Non-streaming Response
    crate::proxy::mappers::gemini::wrapper::inject_ids_to_response(&mut gemini_resp, mapped_model);

    // [FIX #765] Extract thoughtSignature from non-streaming response
    let inner_val = if gemini_resp.get("response").is_some() {
        gemini_resp.get("response")
    } else {
        Some(&gemini_resp)
    };

    if let Some(resp) = inner_val {
        if let Some(candidates) = resp.get("candidates").and_then(|c| c.as_array()) {
            for cand in candidates {
                if let Some(parts) = cand
                    .get("content")
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                {
                    for part in parts {
                        if let Some(sig) = part.get("thoughtSignature").and_then(|s| s.as_str()) {
                            crate::proxy::SignatureCache::global().cache_session_signature(
                                session_id,
                                sig.to_string(),
                                1,
                            );
                            debug!("[Gemini-Response] Cached session signature (len: {}) for session: {}", sig.len(), session_id);
                        }
                    }
                }
            }
        }
    }

    crate::proxy::thinking_store::capture_gemini_response(session_id, &gemini_resp);
    let unwrapped = unwrap_response(&gemini_resp);
    return Ok(HandleSuccessOutcome::Respond(
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("X-Account-Email", email)
            .header("X-Mapped-Model", mapped_model)
            .header("X-Session-Id", client_session_id)
            .header("X-Antigravity-Session-Id", client_session_id)
            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
            .body(Body::from(serde_json::to_string(&unwrapped).unwrap()))
            .unwrap()
            .into_response(),
    ));
}

/// Outcome of handling an upstream error in `handle_generate`.
pub(crate) enum ErrorOutcome {
    /// Send this response to the client immediately.
    Respond(axum::response::Response),
    /// Continue the retry loop.
    Continue,
    /// Break the retry loop (proceed to final failure handling).
    Break,
}
