use super::*;

/// 处理 generateContent 和 streamGenerateContent
/// 路径参数: model_name, method (e.g. "gemini-pro", "generateContent")
pub async fn handle_generate(
    State(state): State<AppState>,
    Path(model_action): Path<String>,
    headers: HeaderMap, // [NEW] Extract headers for adapter detection
    upstream_recorder: Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
    Json(mut body): Json<Value>, // 改为 mut 以支持修复提示词注入
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let clean_start = std::time::Instant::now();

    // 解析 model:method
    let (model_name, method) = if let Some((m, action)) = model_action.rsplit_once(':') {
        (m.to_string(), action.to_string())
    } else {
        (model_action, "generateContent".to_string())
    };

    crate::modules::logger::log_info(&format!(
        "Received Gemini request: {}/{}",
        model_name, method
    ));
    let trace_id = format!("req_{}", chrono::Utc::now().timestamp_subsec_millis());
    let debug_cfg = state.debug_logging.read().await.clone();

    // [NEW] Detect Client Adapter
    let client_adapter = CLIENT_ADAPTERS
        .iter()
        .find(|a| a.matches(&headers))
        .cloned();
    if client_adapter.is_some() {
        debug!("[{}] Client Adapter detected", trace_id);
    }

    // [DEFENSE] Sanitize all inlineData in Gemini request bodies (filter empty or corrupted images)
    crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(&mut body);

    // [Stage Timing] Stage timing measurement variables (ms)
    let clean_micros = clean_start.elapsed().as_micros() as u64;
    let clean_ms: f64 = clean_micros as f64 / 1000.0;
    #[allow(unused_assignments)]
    let mut norm_ms: f64 = 0.0;
    #[allow(unused_assignments)]
    let mut think_fill_ms: f64 = 0.0;
    let mut ttft_ms: f64 = 0.0;

    // 1. 验证方法
    // [NEW] :countTokens 冒号语法，直接代理到上游 v1internal:countTokens
    if method == "countTokens" {
        return Ok(execute_count_tokens(state, model_name, body).await);
    }

    if method != "generateContent" && method != "streamGenerateContent" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Unsupported method: {}", method),
        ));
    }
    if debug_logger::is_enabled(&debug_cfg) {
        let original_payload = json!({
            "kind": "original_request",
            "protocol": "gemini",
            "trace_id": trace_id,
            "original_model": model_name,
            "method": method,
            "request": body.clone(),
        });
        debug_logger::write_debug_payload(
            &debug_cfg,
            Some(&trace_id),
            "original_request",
            &original_payload,
        )
        .await;
    }
    let client_wants_stream = method == "streamGenerateContent";
    // [AUTO-CONVERSION] 强制内部流式化
    let force_stream_internally = !client_wants_stream;
    let is_stream = client_wants_stream || force_stream_internally;

    if force_stream_internally {
        // debug!("[AutoConverter] Converting non-stream request to stream");
    }

    // 2. 获取 UpstreamClient 和 TokenManager
    let upstream = state.upstream.clone();
    let image_scheduler = state.image_scheduler.clone();
    let request_timeout = state.request_timeout;
    let token_manager = state.token_manager;
    let pool_size = token_manager.len();
    // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
    let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(pool_size);

    let mut last_error = String::new();
    let mut last_email: Option<String> = None;
    let mut force_rotate = false;
    let mut retry_state = RequestRetryState::default();
    let mut retry_credentials: Option<(String, String, String, String, u64)> = None;
    let mut image_permit = None;
    let mut failure_statuses = FailureStatusTracker::default();
    let mut used_attempts = 0;
    let mut retried_without_thinking = false;

    let initial_mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        &model_name,
        &*state.custom_mapping.read().await,
    );

    while let Some(attempt) = next_rotation_attempt(
        &mut used_attempts,
        max_attempts,
        retry_credentials.is_some(),
    ) {
        // [Stage Timing] Normalization start time
        let norm_start = std::time::Instant::now();

        // 3. 模型路由解析
        let mapped_model = initial_mapped_model.clone();
        // 提取 tools 列表以进行联网探测 (Gemini 风格可能是嵌套的)
        let tools_val: Option<Vec<Value>> =
            body.get("tools").and_then(|t| t.as_array()).map(|arr| {
                let mut flattened = Vec::new();
                for tool_entry in arr {
                    if let Some(decls) = tool_entry
                        .get("functionDeclarations")
                        .and_then(|v| v.as_array())
                    {
                        flattened.extend(decls.iter().cloned());
                    } else {
                        flattened.push(tool_entry.clone());
                    }
                }
                flattened
            });

        let config = crate::proxy::mappers::common_utils::resolve_request_config(
            &model_name,
            &mapped_model,
            &tools_val,
            None,        // size (not applicable for Gemini native protocol)
            None,        // quality
            None,        // [NEW] image_size
            Some(&body), // [NEW] Pass request body for imageConfig parsing
        );

        // 4. 获取 Token (使用准确的 request_type)
        // 提取 SessionId (粘性指纹，优先以显式会话头对齐跨协议 store_key)
        let explicit_sid = headers
            .get("x-session-id")
            .or_else(|| headers.get("x-jeikcode-session-id"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let fallback_sid = if let Some(sid) = explicit_sid {
            sid.to_string()
        } else {
            SessionManager::extract_gemini_session_id(&body, &model_name)
        };
        let session_scope = crate::proxy::thinking_store::SessionScope::from_headers_and_body(
            &headers,
            Some(&body),
            fallback_sid,
        );
        let session_id = session_scope.store_key.clone();
        let client_session_id = session_scope.client_id.clone();

        // 关键：根据 force_rotate 标志决定是否轮换账号（支持 Grace Retry 原地重试）
        let (access_token, project_id, email, account_id, _wait_ms) =
            if let Some(credentials) = retry_credentials.take() {
                credentials
            } else if config.request_type == "image_gen" {
                drop(image_permit.take());
                match token_manager
                    .get_image_token(
                        force_rotate,
                        Some(&session_id),
                        &config.final_model,
                        &image_scheduler,
                        request_timeout,
                    )
                    .await
                {
                    Ok((access_token, project_id, email, account_id, wait_ms, permit)) => {
                        image_permit = Some(permit);
                        (access_token, project_id, email, account_id, wait_ms)
                    }
                    Err((status, message)) => {
                        failure_statuses.record(status);
                        last_error = message;
                        break;
                    }
                }
            } else {
                match token_manager
                    .get_token(
                        &config.request_type,
                        force_rotate,
                        Some(&session_id),
                        &config.final_model,
                    )
                    .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        let headers = build_token_error_headers(
                            Some(mapped_model.as_str()),
                            last_email.as_deref(),
                            &e,
                        );
                        return Ok((
                            StatusCode::SERVICE_UNAVAILABLE,
                            headers,
                            format!("Token error: {}", e),
                        )
                            .into_response());
                    }
                }
            };

        let mapped_model = token_manager
            .resolve_dynamic_model_for_account(&account_id, &mapped_model)
            .await;

        last_email = Some(email.clone());
        info!("✓ Using account: {} (type: {})", email, config.request_type);

        // 5. 包装请求 (project injection)
        // [FIX #765] Pass session_id to wrap_request for signature injection
        // [NEW] 获取完整 Token 对象以注入动态规格 (dynamic > static default > 65535)
        let token_obj = token_manager.get_token_by_id(&account_id);
        let tf_start = std::time::Instant::now();
        let mut wrapped_body = wrap_request_v2(
            &body,
            &project_id,
            &mapped_model,
            Some(account_id.as_str()),
            Some(&session_id),
            token_obj.as_ref(),
            Some(&token_manager),
            Some(&state.upstream),
        );
        let tf_micros = tf_start.elapsed().as_micros() as u64;
        let norm_total_micros = norm_start.elapsed().as_micros() as u64;
        norm_ms = norm_total_micros.saturating_sub(tf_micros) as f64 / 1000.0;
        think_fill_ms = tf_micros as f64 / 1000.0;

        // Justification: non-Result return value intentionally discarded — no error channel to track
        let _ =
            crate::proxy::mappers::context_manager::ContextManager::apply_post_transit_context_mgmt(
                &mut wrapped_body,
                &mapped_model,
            );

        if let Some(ref recorder) = upstream_recorder {
            recorder.set_value(&wrapped_body);
        }

        if debug_logger::is_enabled(&debug_cfg) {
            let payload = json!({
                "kind": "v1internal_request",
                "protocol": "gemini",
                "trace_id": trace_id,
                "original_model": model_name,
                "mapped_model": mapped_model,
                "request_type": config.request_type,
                "attempt": attempt,
                "v1internal_request": wrapped_body.clone(),
            });
            debug_logger::write_debug_payload(
                &debug_cfg,
                Some(&trace_id),
                "v1internal_request",
                &payload,
            )
            .await;
        }

        // 5. 上游调用
        let query_string = if is_stream { Some("alt=sse") } else { None };
        let upstream_method = if is_stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };

        // [FIX #1522] Inject Anthropic Beta Headers for Claude models
        let mut extra_headers = std::collections::HashMap::new();
        if mapped_model.to_lowercase().contains("claude") {
            extra_headers.insert("anthropic-beta".to_string(), "claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14".to_string());
            tracing::debug!(
                "[Gemini] Injected Anthropic beta headers for Claude model: {}",
                mapped_model
            );
        }

        let preceding_turn_anchor = wrapped_body
            .get("request")
            .and_then(|r| r.get("contents"))
            .or_else(|| wrapped_body.get("contents"))
            .and_then(|c| c.as_array())
            .and_then(|a| a.last())
            .cloned();
        let causal_anchor =
            crate::proxy::thinking_store::compute_causal_anchor(preceding_turn_anchor.as_ref());
        let upstream_req_start = std::time::Instant::now();
        let call_result = match upstream
            .call_v1_internal_with_headers(
                upstream_method,
                &access_token,
                wrapped_body,
                query_string,
                extra_headers.clone(),
                Some(account_id.as_str()),
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_error = e.clone();
                failure_statuses.record(StatusCode::BAD_GATEWAY);
                drop(image_permit.take());
                debug!(
                    "Gemini Request failed on attempt {}/{}: {}",
                    attempt + 1,
                    max_attempts,
                    e
                );
                continue;
            }
        };

        // [NEW] 记录端点降级日志到 debug 文件
        if !call_result.fallback_attempts.is_empty() && debug_logger::is_enabled(&debug_cfg) {
            let fallback_entries: Vec<serde_json::Value> = call_result
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
                "protocol": "gemini",
                "trace_id": trace_id,
                "original_model": model_name,
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

        // [NEW] 提取官方 TraceID
        let cloud_code_trace_id = response
            .headers()
            .get("x-cloudaicompanion-trace-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        if status.is_success() {
            match handle_generate_success(
                response,
                is_stream,
                &debug_cfg,
                &trace_id,
                &model_name,
                &mapped_model,
                &config.request_type,
                attempt,
                status,
                &upstream_url,
                &session_id,
                &client_session_id,
                cloud_code_trace_id,
                upstream_req_start,
                &token_manager,
                &account_id,
                &email,
                clean_ms,
                norm_ms,
                think_fill_ms,
                &mut ttft_ms,
                &mut image_permit,
                &mut last_error,
            )
            .await
            {
                Ok(HandleSuccessOutcome::Respond(r)) => return Ok(r),
                Ok(HandleSuccessOutcome::Retry) => {
                    failure_statuses.record(StatusCode::BAD_GATEWAY);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        // 处理错误并重试
        match handle_generate_error(
            status,
            response,
            &debug_cfg,
            &trace_id,
            &model_name,
            &mapped_model,
            &session_id,
            &client_session_id,
            &account_id,
            &email,
            &upstream_url,
            attempt,
            max_attempts,
            pool_size,
            &token_manager,
            &headers,
            &client_adapter,
            &mut failure_statuses,
            &mut last_error,
            &mut force_rotate,
            &mut retried_without_thinking,
            &mut retry_credentials,
            &mut retry_state,
            &mut image_permit,
            &config,
            &mut body,
            &access_token,
            &project_id,
        )
        .await
        {
            Ok(ErrorOutcome::Respond(r)) => return Ok(r),
            Ok(ErrorOutcome::Continue) => continue,
            Ok(ErrorOutcome::Break) => break,
            Err(e) => return Err(e),
        }
    }

    // 所有尝试均失败：仅当全部结构化失败状态均为 429 时返回 429
    let final_status = failure_statuses.final_status();
    let headers = build_token_error_headers(
        Some(initial_mapped_model.as_str()),
        last_email.as_deref(),
        &last_error,
    );

    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
        "gemini",
        final_status.as_u16(),
        initial_mapped_model.as_str(),
        &last_error,
    );

    Ok((final_status, headers, Json(dual_err)).into_response())
}
