use super::*;

/// Handle a successful upstream response when streaming was requested:
/// heartbeat-skipping peek, combined-stream construction with idle timeout,
/// then either SSE passthrough or collect-to-JSON.
/// Extracted from `handle_messages`. Pure code move: no logic changes.
pub(crate) async fn handle_stream_success(
    st: &mut AttemptState,
    call: AttemptCall,
    attempt: usize,
) -> StreamOutcome {
    let AttemptCall {
        response,
        email,
        request_with_mapped,
        request_type,
        session_id_str,
        client_session_id,
        raw_estimated,
        client_wants_stream,
        upstream_req_start,
        upstream_url,
        status,
        ..
    } = call;
    let meta = json!({
        "protocol": "anthropic",
        "trace_id": st.trace_id,
        "original_model": st.request.model,
        "mapped_model": request_with_mapped.model,
        "request_type": request_type,
        "attempt": attempt,
        "status": status.as_u16(),
        "upstream_url": upstream_url,
    });
    let gemini_stream = debug_logger::wrap_stream_with_debug(
        Box::pin(response.bytes_stream()),
        st.debug_cfg.clone(),
        st.trace_id.clone(),
        "upstream_response",
        meta,
    );

    let current_message_count = request_with_mapped.messages.len();

    // Determine context limit based on model
    let context_limit = crate::proxy::mappers::claude::utils::get_context_limit_for_model(
        &request_with_mapped.model,
    );

    // [FIX #MCP] Extract registered tool names for MCP fuzzy matching
    let registered_tool_names: Vec<String> = request_with_mapped
        .tools
        .as_ref()
        .map(|tools| tools.iter().filter_map(|t| t.name.clone()).collect())
        .unwrap_or_default();

    // [FIX #530/#529/#859] Enhanced Peek logic to handle heartbeats and slow start
    // We must pre-read until we find a MEANINGFUL content block (like message_start).
    // If we only get heartbeats (ping) and then the stream dies, we should rotate account.
    let mut claude_stream = create_claude_sse_stream(
        gemini_stream,
        st.trace_id.clone(),
        email.clone(),
        Some(session_id_str.clone()),
        st.scaling_enabled,
        context_limit,
        Some(raw_estimated), // [FIX] Pass estimated tokens for calibrator learning
        current_message_count, // [NEW v4.0.0] Pass message count for rewind detection
        st.client_adapter.clone(), // [NEW] Pass client adapter
        registered_tool_names, // [FIX #MCP] Pass tool names for fuzzy matching
    );

    let mut first_data_chunk = None;
    let mut retry_this_account = false;

    // Loop to skip heartbeats during peek
    loop {
        match tokio::time::timeout(
            // [FIX #Bug1] Reduced from 300s to 30s.
            // Gemini sends first chunk within 5s normally; 30s allows for retries
            // without causing the 5-minute hang users observed.
            std::time::Duration::from_secs(30),
            claude_stream.next(),
        )
        .await
        {
            Ok(Some(Ok(bytes))) => {
                if bytes.is_empty() {
                    continue;
                }

                let text = String::from_utf8_lossy(&bytes);
                // Skip SSE comments/pings
                if text.trim().starts_with(":") {
                    debug!("[{}] Skipping peek heartbeat: {}", st.trace_id, text.trim());
                    continue;
                }

                // We found real data!
                st.ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
                first_data_chunk = Some(bytes);
                break;
            }
            Ok(Some(Err(e))) => {
                tracing::warn!(
                    "[{}] Stream error during peek: {}, retrying...",
                    st.trace_id,
                    e
                );
                st.last_error = format!("Stream error during peek: {}", e);
                retry_this_account = true;
                break;
            }
            Ok(None) => {
                tracing::warn!(
                    "[{}] Stream ended during peek (Empty Response), retrying...",
                    st.trace_id
                );
                st.last_error = "Empty response stream during peek".to_string();
                retry_this_account = true;
                break;
            }
            Err(_) => {
                tracing::warn!(
                    "[{}] Timeout waiting for first data (30s), retrying...",
                    st.trace_id
                );
                st.last_error = "Timeout waiting for first data".to_string();
                retry_this_account = true;
                break;
            }
        }
    }

    if retry_this_account {
        return StreamOutcome::Retry;
    }

    match first_data_chunk {
        Some(bytes) => {
            // We have data! Construct the combined stream
            let stream_rest = claude_stream;
            let combined_stream = futures::stream::once(async move { Ok(bytes) }).chain(
                stream_rest.map(|result| -> Result<Bytes, std::io::Error> {
                    match result {
                        Ok(b) => Ok(b),
                        Err(e) => Ok(Bytes::from(format!("data: {{\"error\":\"{}\"}}\n\n", e))),
                    }
                }),
            );

            // [FIX #Bug1] 针对 Claude 流增加空闲超时保护，从 300s 降至 120s
            // 300s 会导致客户端等待长达 5 分钟；120s 仍有足够容错余量
            let combined_stream = async_stream::stream! {
                let mut s = Box::pin(combined_stream);
                loop {
                    match tokio::time::timeout(std::time::Duration::from_secs(120), s.next()).await {
                        Ok(Some(item)) => yield item,
                        Ok(None) => break,
                        Err(_) => {
                            tracing::error!("[Claude-SSE] Idle timeout after 120s, terminating stream");
                            yield Ok::<Bytes, std::io::Error>(Bytes::from("data: {\"type\": \"message_stop\"}\n\ndata: [DONE]\n\n"));
                            break;
                        }
                    }
                }
            };

            // 判断客户端期望的格式
            if client_wants_stream {
                // 客户端本就要 Stream，直接返回 SSE
                return StreamOutcome::Respond(
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/event-stream")
                        .header(header::CACHE_CONTROL, "no-cache")
                        .header(header::CONNECTION, "keep-alive")
                        .header("X-Accel-Buffering", "no")
                        .header("X-Account-Email", &email)
                        .header("X-Mapped-Model", &request_with_mapped.model)
                        .header("X-Session-Id", &client_session_id)
                        .header("X-Antigravity-Session-Id", &client_session_id)
                        .header("X-Context-Purified", "false")
                        .header("X-Timing-Clean-Ms", format!("{:.3}", st.clean_ms))
                        .header("X-Timing-Norm-Ms", format!("{:.3}", st.norm_ms))
                        .header("X-Timing-Thinking-Ms", format!("{:.3}", st.think_fill_ms))
                        .header("X-Timing-Ttft-Ms", format!("{:.3}", st.ttft_ms))
                        .body(Body::from_stream(combined_stream))
                        .unwrap(),
                );
            } else {
                // 客户端要非 Stream，需要收集完整响应并转换为 JSON
                use crate::proxy::handlers::claude::attempt::AttemptCall;
                use crate::proxy::handlers::claude::attempt::AttemptState;
                use crate::proxy::handlers::claude::attempt::StreamOutcome;
                use crate::proxy::mappers::claude::collect_stream_to_json;

                match collect_stream_to_json(Box::pin(combined_stream)).await {
                    Ok(full_response) => {
                        info!("[{}] ✓ Stream collected and converted to JSON", st.trace_id);
                        return StreamOutcome::Respond(
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/json")
                                .header("X-Account-Email", &email)
                                .header("X-Mapped-Model", &request_with_mapped.model)
                                .header("X-Session-Id", &client_session_id)
                                .header("X-Antigravity-Session-Id", &client_session_id)
                                .header("X-Context-Purified", "false")
                                .header("X-Timing-Clean-Ms", format!("{:.3}", st.clean_ms))
                                .header("X-Timing-Norm-Ms", format!("{:.3}", st.norm_ms))
                                .header("X-Timing-Thinking-Ms", format!("{:.3}", st.think_fill_ms))
                                .header("X-Timing-Ttft-Ms", format!("{:.3}", st.ttft_ms))
                                .body(Body::from(serde_json::to_string(&full_response).unwrap()))
                                .unwrap(),
                        );
                    }
                    Err(e) => {
                        return StreamOutcome::Respond(
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

        None => {
            tracing::warn!(
                "[{}] Stream ended immediately (Empty Response), retrying...",
                st.trace_id
            );
            st.last_error = "Empty response stream (None)".to_string();
            return StreamOutcome::Retry;
        }
    }
}
