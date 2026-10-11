use super::*;
use crate::proxy::monitor::CURRENT_UPSTREAM_CAPTURE;
use std::sync::Arc;

/// Build the initial `ProxyRequestLog` from response metadata and request context.
///
/// Extracted from `monitor_middleware` via extract-method (Phase 2: response metadata).
/// Pure code motion — no logic changes.
pub(crate) fn build_initial_log(
    ctx: &MonitorRequestContext,
    response: &Response,
    request_headers_json: String,
    response_headers_json: String,
    upstream_request_body: Option<String>,
    upstream_request_headers: Option<String>,
    account_email: Option<String>,
    mapped_model: Option<String>,
    protocol: Option<String>,
    session_id: Option<String>,
    duration: u64,
    status: u16,
) -> ProxyRequestLog {
    ProxyRequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        method: ctx.method.clone(),
        url: ctx.uri.clone(),
        status,
        duration,
        model: ctx.model.clone(),
        mapped_model,
        account_email,
        client_ip: ctx.client_ip.clone(),
        error: None,
        request_body: ctx.request_body_str.clone(),
        upstream_request_body,
        response_body: None,
        request_headers: Some(request_headers_json),
        upstream_request_headers,
        response_headers: Some(response_headers_json),
        input_tokens: None,
        output_tokens: None,
        cached_tokens: None,
        protocol,
        username: ctx
            .user_token_identity
            .as_ref()
            .map(|identity| identity.username.clone()),
        session_id,
    }
}

/// Extract account email, mapped model, protocol, and session ID from response headers.
fn extract_response_metadata(
    response: &Response,
    uri: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let account_email = response
        .headers()
        .get("X-Account-Email")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mapped_model = response
        .headers()
        .get("X-Mapped-Model")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let protocol = if uri.contains("/v1/messages") {
        Some("anthropic".to_string())
    } else if uri.contains("/v1beta/models") {
        Some("gemini".to_string())
    } else if uri.starts_with("/v1/") {
        Some("openai".to_string())
    } else {
        None
    };

    let session_id = response
        .headers()
        .get("X-Session-Id")
        .or_else(|| response.headers().get("X-Antigravity-Session-Id"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    (account_email, mapped_model, protocol, session_id)
}

/// HTTP middleware that logs proxied requests/responses for the Proxy Monitor UI.
///
/// Extracted via extract-method: request setup → `extract_request_context`,
/// SSE parsing → `SseParseAccumulator`, stream post-processing →
/// `process_collected_stream`. Pure code motion — no logic changes.
pub async fn monitor_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let _logging_enabled = state.monitor.is_enabled();

    let method = request.method().to_string();
    let uri = request.uri().to_string();

    if uri.contains("event_logging") || uri.contains("/api/") || uri.starts_with("/internal/") {
        return next.run(request).await;
    }

    let (request, ctx) = extract_request_context(request, method, uri.clone()).await;

    let upstream_holder = UpstreamRequestBodyHolder::new();
    let mut request = request;
    request.extensions_mut().insert(upstream_holder.clone());
    let request_headers_json =
        crate::proxy::payload_audit::headers_to_redacted_json(request.headers());

    let response = crate::proxy::monitor::CURRENT_UPSTREAM_CAPTURE
        .scope(upstream_holder.clone(), next.run(request))
        .await;
    let upstream_request_body = upstream_holder.take();
    let upstream_request_headers = upstream_holder.take_headers();
    let response_headers_json =
        crate::proxy::payload_audit::headers_to_redacted_json(response.headers());

    let duration = ctx.start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();

    // 过滤噪音请求，避免客户端探针刷屏淹没真实业务日志 (Issue #3498 + `/v1/models` 轮询刷屏)
    let capture_health_enabled =
        state.monitor.is_capture_health_logs() || should_log_health_checks();
    if should_skip_request_log(&ctx.method, response.status(), capture_health_enabled) {
        return response;
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let (account_email, mapped_model, protocol, session_id) =
        extract_response_metadata(&response, &ctx.uri);

    let monitor: Arc<ProxyMonitor> = state.monitor.clone();
    let mut log = build_initial_log(
        &ctx,
        &response,
        request_headers_json,
        response_headers_json,
        upstream_request_body,
        upstream_request_headers,
        account_email,
        mapped_model,
        protocol,
        session_id,
        duration,
        status,
    );

    let is_image_route = ctx.uri.contains("/v1/images/");
    let user_token_identity = ctx.user_token_identity.clone();
    let user_agent = ctx.user_agent.clone();

    if content_type.contains("text/event-stream") {
        let (parts, body) = response.into_parts();
        let mut stream = body.into_data_stream();
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let start_instant = ctx.start;

        tokio::spawn(async move {
            let stream_start = std::time::Instant::now();
            let mut all_stream_data = Vec::new();
            let mut last_few_bytes = Vec::new();

            while let Some(chunk_res) = next_chunk_while_receiver_open(&mut stream, &tx).await {
                if let Ok(chunk) = chunk_res {
                    all_stream_data.extend_from_slice(&chunk);

                    if chunk.len() > 8192 {
                        last_few_bytes = chunk.slice(chunk.len() - 8192..).to_vec();
                    } else {
                        last_few_bytes.extend_from_slice(&chunk);
                        if last_few_bytes.len() > 8192 {
                            last_few_bytes.drain(0..last_few_bytes.len() - 8192);
                        }
                    }
                    if tx.send(Ok::<_, axum::Error>(chunk)).await.is_err() {
                        break;
                    }
                } else if let Err(e) = chunk_res {
                    if tx.send(Err(axum::Error::new(e))).await.is_err() {
                        break;
                    }
                }
            }
            drop(stream);

            let stream_ms = stream_start.elapsed().as_micros() as f64 / 1000.0;
            let total_ms = start_instant.elapsed().as_micros() as f64 / 1000.0;
            log.duration = total_ms.round() as u64;

            process_collected_stream(
                all_stream_data,
                last_few_bytes,
                stream_ms,
                total_ms,
                log,
                user_token_identity,
                monitor,
                user_agent,
            )
            .await;
        });

        Response::from_parts(
            parts,
            Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx)),
        )
    } else if content_type.contains("application/json") || content_type.contains("text/") {
        handle_non_streaming_response(
            response,
            log,
            &ctx,
            is_image_route,
            user_token_identity,
            user_agent,
            monitor,
        )
        .await
    } else {
        log.response_body = Some(format!("[{}]", content_type));

        // Record User Token Usage
        record_user_token_usage(&user_token_identity, &log, user_agent);

        monitor.log_request(log).await;
        response
    }
}

/// Handle non-streaming JSON/text responses: parse, consolidate, extract tokens, persist.
///
/// Extracted from `monitor_middleware` via extract-method. Pure code motion.
async fn handle_non_streaming_response(
    response: Response,
    mut log: ProxyRequestLog,
    ctx: &MonitorRequestContext,
    is_image_route: bool,
    user_token_identity: Option<UserTokenIdentity>,
    user_agent: Option<String>,
    monitor: Arc<ProxyMonitor>,
) -> Response {
    let total_ms = ctx.start.elapsed().as_micros() as f64 / 1000.0;
    log.duration = total_ms.round() as u64;

    let mut headers_map: serde_json::Map<String, Value> = log
        .response_headers
        .as_ref()
        .and_then(|h| serde_json::from_str(h).ok())
        .unwrap_or_default();

    headers_map.insert(
        "x-timing-total-ms".to_string(),
        serde_json::json!(format!("{:.3}", total_ms)),
    );
    log.response_headers = serde_json::to_string(&Value::Object(headers_map.clone())).ok();

    let (parts, body) = response.into_parts();
    match axum::body::to_bytes(body, MAX_RESPONSE_LOG_SIZE).await {
        Ok(bytes) => {
            if let Ok(s) = std::str::from_utf8(&bytes) {
                if let Ok(json) = serde_json::from_str::<Value>(&s) {
                    // 支持 OpenAI "usage" 或 Gemini "usageMetadata"
                    if let Some(usage) = json
                        .get("usage")
                        .or(json.get("usageMetadata"))
                        .or(json.get("response").and_then(|r| r.get("usage")))
                        .or(json.get("response").and_then(|r| r.get("usageMetadata")))
                    {
                        log.input_tokens = extract_input_tokens(usage);
                        log.output_tokens = extract_output_tokens(usage);
                        log.cached_tokens =
                            log.cached_tokens.or_else(|| extract_cached_tokens(usage));

                        if log.input_tokens.is_none() && log.output_tokens.is_none() {
                            log.output_tokens = usage
                                .get("total_tokens")
                                .or(usage.get("totalTokenCount"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32);
                        }
                    }
                }
                if is_image_route {
                    log.response_body = serde_json::from_str::<Value>(&s)
                        .ok()
                        .and_then(|json| summarize_image_json_response(&json))
                        .or_else(|| Some(s.to_string()));
                } else if let Ok(json) = serde_json::from_str::<Value>(&s) {
                    // 🌟 非流式响应入库统一转换为满血简要版 (包含网关权威签名回填与防雪崩确定性 Tool ID)
                    if let Some(canonical) =
                        consolidate_non_streaming_response(&json, &log, &headers_map)
                    {
                        log.response_body = serde_json::to_string_pretty(&canonical)
                            .ok()
                            .or_else(|| Some(s.to_string()));
                    } else {
                        log.response_body = serde_json::to_string_pretty(&json)
                            .ok()
                            .or_else(|| Some(s.to_string()));
                    }
                } else {
                    log.response_body = Some(s.to_string());
                }
            } else {
                log.response_body = Some("[Binary Response Data]".to_string());
            }

            if log.status >= 400 {
                log.error = log.response_body.clone();
            }

            // Fallback input token estimation prefers the transit (upstream) body
            if log.input_tokens.is_none() {
                let estimated = log
                    .upstream_request_body
                    .as_ref()
                    .or(log.request_body.as_ref())
                    .map(|body| {
                        crate::proxy::mappers::context_manager::estimate_raw_tokens_from_payload(
                            body,
                        )
                    })
                    .unwrap_or(0);
                if estimated > 0 {
                    log.input_tokens = Some(estimated);
                }
            }

            // Record User Token Usage
            record_user_token_usage(&user_token_identity, &log, user_agent.clone());

            monitor.log_request(log).await;
            Response::from_parts(parts, Body::from(bytes))
        }
        Err(_) => {
            log.response_body = Some("[Response too large (>100MB)]".to_string());

            // Record User Token Usage (even if too large)
            record_user_token_usage(&user_token_identity, &log, user_agent.clone());

            monitor.log_request(log).await;
            Response::from_parts(parts, Body::empty())
        }
    }
}
