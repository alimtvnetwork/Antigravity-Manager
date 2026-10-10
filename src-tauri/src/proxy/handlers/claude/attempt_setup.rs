use super::*;

/// Run one retry-loop attempt up to the upstream call: model routing, token
/// acquisition, request transform, upstream invocation and fallback logging.
/// Extracted from `handle_messages`. Pure code move: no logic changes.
///
/// `Respond` carries an early HTTP response, `Retry` means `continue` the
/// retry loop, `Proceed` carries the data for the response handlers.
pub(crate) async fn prepare_attempt(st: &mut AttemptState, attempt: usize) -> PrepOutcome {
    // [Stage 2 Timing] Normalization start time
    let norm_start = std::time::Instant::now();

    // 2. 模型路由解析
    let mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        &st.request_for_body.model,
        &*st.custom_mapping.read().await,
    );
    st.last_mapped_model = Some(mapped_model.clone());

    // 将 Claude 工具转为 Value 数组以便探测联网
    let tools_val: Option<Vec<Value>> = st.request_for_body.tools.as_ref().map(|list| {
        list.iter()
            .map(|t| serde_json::to_value(t).unwrap_or(json!({})))
            .collect()
    });

    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &st.request_for_body.model,
        &mapped_model,
        &tools_val,
        st.request.size.as_deref(),    // [NEW] Pass size parameter
        st.request.quality.as_deref(), // [NEW] Pass quality parameter
        None,                          // image_size
        None,                          // body
    );

    // 0. 尝试提取 session_id 用于粘性调度 (Phase 2/3)
    // 使用 SessionManager 生成稳定的会话指纹，优先以显式会话头对齐跨协议 store_key
    let explicit_sid = st
        .headers
        .get("x-session-id")
        .or_else(|| st.headers.get("x-jeikcode-session-id"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let fallback_sid = if let Some(sid) = explicit_sid {
        sid.to_string()
    } else {
        crate::proxy::session_manager::SessionManager::extract_session_id(&st.request_for_body)
    };
    let session_scope = crate::proxy::thinking_store::SessionScope::from_headers_and_body(
        &st.headers,
        Some(&st.original_body),
        fallback_sid,
    );
    let session_id_str = session_scope.store_key.clone();
    let client_session_id = session_scope.client_id.clone();
    let session_id = Some(session_id_str.as_str());

    let (access_token, project_id, email, account_id, _wait_ms) = match st
        .token_manager
        .get_token(
            &config.request_type,
            st.force_rotate,
            session_id,
            &config.final_model,
        )
        .await
    {
        Ok(t) => t,
        Err(e) => {
            let safe_message = if e.contains("invalid_grant") {
                "OAuth refresh failed (invalid_grant): refresh_token likely revoked/expired; reauthorize account(s) to restore service.".to_string()
            } else {
                e
            };
            let tok_err_headers = crate::proxy::handlers::common::build_token_error_headers(
                Some(mapped_model.as_str()),
                None,
                &safe_message,
            );
            let dual_err = crate::proxy::handlers::common::build_dual_track_error(
                "claude",
                StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                mapped_model.as_str(),
                &safe_message,
            );
            return PrepOutcome::Respond(
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    tok_err_headers,
                    Json(dual_err),
                )
                    .into_response(),
            );
        }
    };

    st.last_email = Some(email.clone());
    info!("✓ Using account: {} (type: {})", email, config.request_type);

    let mut request_with_mapped =
        match apply_compression(&st.st.request_for_body, &mapped_model, st).await {
            Ok(r) => r,
            Err(resp) => return PrepOutcome::Respond(resp),
        };

    // [FIX] Estimate AFTER purification to get accurate token count for calibrator learning
    let raw_estimated = ContextManager::estimate_token_usage(&request_with_mapped);

    request_with_mapped.model = mapped_model.clone();

    // 生成 Trace ID (简单用时间戳后缀)
    // let _trace_id = format!("req_{}", chrono::Utc::now().timestamp_subsec_millis());

    let token_obj = st.token_manager.get_token_by_id(&account_id);
    let (mut gemini_body, transform_timing) =
        match crate::proxy::mappers::claude::transform_claude_request_in_timed(
            &request_with_mapped,
            &project_id,
            st.retried_without_thinking,
            Some(account_id.as_str()),
            &session_id_str,
            token_obj.as_ref(),
        ) {
            Ok((b, timing)) => {
                debug!(
                    "[{}] Transformed Gemini Body: {}",
                    st.trace_id,
                    serde_json::to_string_pretty(&b).unwrap_or_default()
                );
                (b, timing)
            }
            Err(e) => {
                let xform_headers = [
                    ("X-Mapped-Model", request_with_mapped.model.as_str()),
                    ("X-Account-Email", email.as_str()),
                ];
                return PrepOutcome::Respond(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        st.headers,
                        Json(json!({
                            "type": "error",
                            "error": {
                                "type": "api_error",
                                "message": format!("Transform error: {}", e)
                            }
                        })),
                    )
                        .into_response(),
                );
            }
        };

    // Justification: non-Result return value intentionally discarded — no error channel to track
    let _ = crate::proxy::mappers::context_manager::ContextManager::apply_post_transit_context_mgmt(
        &mut gemini_body,
        &mapped_model,
    );
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
        &mut gemini_body,
    );
    crate::proxy::mappers::common_utils::ensure_gemini_payload_ends_with_user(&mut gemini_body);

    let norm_total_micros = norm_start.elapsed().as_micros() as u64;
    let tf_micros = transform_timing.think_fill_micros;
    st.norm_ms = norm_total_micros.saturating_sub(tf_micros) as f64 / 1000.0;
    st.think_fill_ms = tf_micros as f64 / 1000.0;

    if let Some(ref recorder) = st.upstream_recorder {
        recorder.set_value(&gemini_body);
    }

    if debug_logger::is_enabled(&st.debug_cfg) {
        let payload = json!({
            "kind": "v1internal_request",
            "protocol": "anthropic",
            "trace_id": st.trace_id,
            "original_model": st.request.model,
            "mapped_model": request_with_mapped.model,
            "request_type": config.request_type,
            "attempt": attempt,
            "v1internal_request": gemini_body.clone(),
        });
        debug_logger::write_debug_payload(
            &st.debug_cfg,
            Some(&st.trace_id),
            "v1internal_request",
            &payload,
        )
        .await;
    }

    // 4. 上游调用 - 自动转换逻辑
    let client_wants_stream = st.request.stream;
    // [AUTO-CONVERSION] 非 Stream 请求自动转换为 Stream 以享受更宽松的配额
    let force_stream_internally = !client_wants_stream;
    let actual_stream = client_wants_stream || force_stream_internally;

    if force_stream_internally {
        info!(
            "[{}] 🔄 Auto-converting non-stream st.request to stream for better quota",
            st.trace_id
        );
    }

    let method = if actual_stream {
        "streamGenerateContent"
    } else {
        "generateContent"
    };
    let query = if actual_stream { Some("alt=sse") } else { None };
    // [FIX #765/1522] Prepare Robust Beta Headers for Claude models
    let mut extra_headers = std::collections::HashMap::new();
    extra_headers.insert("x-session-id".to_string(), client_session_id.clone());
    if mapped_model.to_lowercase().contains("claude") {
        extra_headers.insert(
            "anthropic-beta".to_string(),
            "claude-code-20250219".to_string(),
        );
        tracing::debug!(
            "[{}] Added Comprehensive Beta Headers for Claude model",
            st.trace_id
        );
    }

    // [NEW] Inject Beta Headers from Client Adapter
    if let Some(adapter) = &st.client_adapter {
        let mut temp_headers = HeaderMap::new();
        adapter.inject_beta_headers(&mut temp_headers);
        for (k, v) in temp_headers {
            if let Some(name) = k {
                if let Ok(v_str) = v.to_str() {
                    extra_headers.insert(name.to_string(), v_str.to_string());
                    tracing::debug!(
                        "[{}] Added Adapter Header: {}: {}",
                        st.trace_id,
                        name,
                        v_str
                    );
                }
            }
        }
    }

    // [Stage 4 Timing] Waiting for Google st.upstream TTFT start time
    let upstream_req_start = std::time::Instant::now();

    let call_result = match st
        .upstream
        .call_v1_internal_with_headers(
            method,
            &access_token,
            gemini_body,
            query,
            extra_headers.clone(),
            Some(account_id.as_str()),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            st.last_error = e.clone();
            debug!(
                "Request failed on attempt {}/{}: {}",
                attempt + 1,
                st.max_attempts,
                e
            );
            return PrepOutcome::Retry;
        }
    };

    // [NEW] 记录端点降级日志到 debug 文件
    if !call_result.fallback_attempts.is_empty() && debug_logger::is_enabled(&st.debug_cfg) {
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
            "protocol": "anthropic",
            "trace_id": st.trace_id,
            "original_model": st.request.model,
            "mapped_model": request_with_mapped.model,
            "attempt": attempt,
            "account": mask_email(&email),
            "fallback_attempts": fallback_entries,
        });
        debug_logger::write_debug_payload(
            &st.debug_cfg,
            Some(&st.trace_id),
            "endpoint_fallback",
            &payload,
        )
        .await;
    }

    let response = call_result.response;
    // [NEW] 提取实际请求的上游端点 URL，用于日志记录和排查
    let upstream_url = response.url().to_string();
    let status = response.status();
    st.last_status = status;

    PrepOutcome::Proceed(AttemptCall {
        response,
        upstream_url,
        status,
        email,
        account_id,
        mapped_model,
        request_type: config.request_type,
        request_with_mapped,
        session_id_str,
        client_session_id,
        raw_estimated,
        client_wants_stream,
        actual_stream,
        upstream_req_start,
    })
}
