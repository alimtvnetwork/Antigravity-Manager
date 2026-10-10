use super::*;

/// Handle a non-success upstream response: status extraction, pipeline
/// classification, account lockout, thinking-signature retry, 403 handling,
/// session-token-exceeded retry and the adaptive retry-strategy decision.
/// Extracted from `handle_messages`. Pure code move: no logic changes.
///
/// `Respond` carries the final HTTP response, `Retry` means `continue` the
/// retry loop. Internal fall-throughs (e.g. thinking-retry declining the
/// backoff) are preserved verbatim inside.
pub(crate) async fn handle_upstream_error(
    st: &mut AttemptState,
    call: AttemptCall,
    attempt: usize,
) -> ErrorOutcome {
    let AttemptCall {
        response,
        upstream_url,
        status,
        email,
        account_id,
        mapped_model,
        request_type,
        request_with_mapped,
        session_id_str,
        client_session_id,
        ..
    } = call;
    let session_id = Some(session_id_str.as_str());
    // 1. 立即提取状态码和 st.headers（防止 response 被 move）
    let status_code = status.as_u16();
    st.last_status = status;
    let retry_after = response
        .st
        .headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 2. 获取错误文本并转移 Response 所有权
    let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| format!("HTTP {}", status));
    st.last_error = format!("HTTP {}: {}", status_code, error_text);
    debug!("[{}] Upstream Error Response: {}", st.trace_id, error_text);
    if debug_logger::is_enabled(&st.debug_cfg) {
        let payload = json!({
            "kind": "upstream_response_error",
            "protocol": "anthropic",
            "trace_id": st.trace_id,
            "original_model": st.request.model,
            "mapped_model": request_with_mapped.model,
            "request_type": request_type,
            "attempt": attempt,
            "status": status_code,
            "upstream_url": upstream_url,
            "account": mask_email(&email),
            "error_text": error_text,
        });
        debug_logger::write_debug_payload(
            &st.debug_cfg,
            Some(&st.trace_id),
            "upstream_response_error",
            &payload,
        )
        .await;
    }

    // 3. 统一流水线决策判定（协议无关的唯一真理）
    let classification = crate::proxy::pipeline::UpstreamClassification::classify(
        status_code,
        &error_text,
        retry_after.as_deref(),
    );

    if classification.is_model_not_found() {
        tracing::warn!(
        "[{}] Pipeline: Target model [{}] not found on st.upstream (HTTP {}). Terminating retry loop without account lockout.",
        st.trace_id, request_with_mapped.model, status_code
    );
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "claude",
            status_code,
            &request_with_mapped.model,
            &error_text,
        );
        return ErrorOutcome::Respond(
            (
                StatusCode::from_u16(status_code).unwrap_or(StatusCode::NOT_FOUND),
                [
                    ("X-Account-Email", email.as_str()),
                    ("X-Mapped-Model", request_with_mapped.model.as_str()),
                ],
                Json(dual_err),
            )
                .into_response(),
        );
    }

    if classification.should_lock_account() {
        st.token_manager
            .mark_rate_limited_async_baseline(
                &email,
                status_code,
                retry_after.as_deref(),
                &error_text,
                Some(&request_with_mapped.model),
            )
            .await;

        st.token_manager
            .unbind_session_and_clear_last_used(session_id)
            .await;
        if let Some(sid) = session_id {
            debug!(
                "[{}] Unbound session {} from account {} due to status {}",
                st.trace_id, sid, email, status_code
            );
        }
    }

    // 4. 处理 400 错误 (Thinking 签名失效 或 块顺序错误)
    // [FIX 2026-08-28] Use case-insensitive matching and cover Google's exact phrasing:
    // "Invalid thought signature." / "thoughtSignature" / "thought_signature"
    let lower_err = error_text.to_lowercase();
    if status_code == 400
        && !st.retried_without_thinking
        && (lower_err.contains("invalid thought signature")
            || lower_err.contains("invalid `signature`")
            || lower_err.contains("invalid signature")
            || lower_err.contains("thought_signature")
            || lower_err.contains("thoughtsignature")
            || lower_err.contains("thinking.signature: field required")
            || lower_err.contains("thinking.thinking: field required")
            || lower_err.contains("thinking.signature")
            || lower_err.contains("thinking.thinking")
            || lower_err.contains("corrupted thought signature")
            || lower_err.contains("failed to deserialise")
            || lower_err.contains("thinking block")
            || lower_err.contains("found `text`")
            || lower_err.contains("found 'text'")
            || lower_err.contains("must be `thinking`")
            || lower_err.contains("must be 'thinking'"))
    {
        // Existing logic for thinking signature.
        st.retried_without_thinking = true;

        // 使用 WARN 级别,因为这不应该经常发生(已经主动过滤过)
        tracing::warn!(
            "[{}] Unexpected thinking signature error (should have been filtered). \
         Retrying with all thinking blocks removed.",
            st.trace_id
        );

        // [IMPROVED] 不再禁用 Thinking 模式！
        // 既然我们已经将历史 Thinking Block 转换为 Text，那么当前请求可以视为一个新的 Thinking 会话
        // 保持 thinking 配置开启，让模型重新生成思维，避免退化为简单的 "OK" 回复
        // st.request_for_body.thinking = None;

        // 清理历史消息中的所有 Thinking Block，将其转换为 Text 以保留上下文
        for msg in st.request_for_body.messages.iter_mut() {
            if let crate::proxy::mappers::claude::models::MessageContent::Array(blocks) =
                &mut msg.content
            {
                let mut new_blocks = Vec::with_capacity(blocks.len());
                for block in blocks.drain(..) {
                    match block {
                        crate::proxy::mappers::claude::models::ContentBlock::Thinking {
                            thinking,
                            ..
                        } => {
                            // 降级为 text
                            if !thinking.is_empty() {
                                tracing::debug!(
                                    "[Fallback] Converting thinking block to text (len={})",
                                    thinking.len()
                                );
                                new_blocks.push(
                                    crate::proxy::mappers::claude::models::ContentBlock::Text {
                                        text: thinking,
                                    },
                                );
                            }
                        }
                        crate::proxy::mappers::claude::models::ContentBlock::RedactedThinking {
                            ..
                        } => {
                            // Redacted thinking 没什么用，直接丢弃
                        }
                        _ => new_blocks.push(block),
                    }
                }
                *blocks = new_blocks;
            }
        }

        // Target-purify ThinkingStore corrupted heterogeneous signatures for the current session,
        // preserving thoughts and healthy history signatures to prevent corrupted signatures from reappearing in contents.
        crate::proxy::thinking_store::ThinkingStore::global()
            .purge_corrupted_signatures(&session_id_str, &mapped_model);
        crate::proxy::SignatureCache::global().delete_session_signature(&client_session_id);

        // [FIX Prompt-Cache] Strictly avoid injecting synthetic messages (close_tool_loop_for_thinking) in retry path!
        // Maintain pure historical messages, delegated to InboundThinkingPipeline and finalize_gemini_contents_thinking.

        // 清理模型名中的 -thinking 后缀
        if st.request_for_body.model.contains("claude-") {
            let mut m = st.request_for_body.model.clone();
            m = m.replace("-thinking", "");
            if m.contains("claude-sonnet-4-6-") {
                m = "claude-sonnet-4-6".to_string();
            } else if m.contains("claude-sonnet-4-5-") {
                m = "claude-sonnet-4-6".to_string();
            } else if m.contains("claude-opus-4-6-") {
                m = "claude-opus-4-6".to_string();
            } else if m.contains("claude-opus-4-5-") || m.contains("claude-opus-4-") {
                m = "claude-opus-4-5".to_string();
            }
            st.request_for_body.model = m;
        }

        // [FIX] 强制重试：因为我们已经清理了 thinking block，所以这是一个新的、可以重试的请求
        // 不要使用 determine_retry_strategy，因为它会因为 st.retried_without_thinking=true 而返回 NoRetry
        if apply_retry_strategy(
            RetryStrategy::FixedDelay(Duration::from_millis(200)),
            attempt,
            st.max_attempts,
            status_code,
            &st.trace_id,
        )
        .await
        {
            return ErrorOutcome::Retry;
        }
    }

    // 5. 统一处理所有可重试错误
    // [REMOVED] 不再特殊处理 QUOTA_EXHAUSTED,允许账号轮换
    // 原逻辑会在第一个账号配额耗尽时直接返回,导致"平衡"模式无法切换账号

    // [FIX] 403 时设置 is_forbidden 状态，避免账号被重复选中
    if status_code == 403 {
        // Check for VALIDATION_REQUIRED error - temporarily block account
        if error_text.contains("VALIDATION_REQUIRED")
            || error_text.contains("verify your account")
            || error_text.contains("validation_url")
        {
            tracing::warn!(
                "[Claude] VALIDATION_REQUIRED detected on account {}, temporarily blocking",
                email
            );
            let block_minutes = 10i64;
            let block_until = chrono::Utc::now().timestamp() + (block_minutes * 60);
            if let Err(e) = st
                .token_manager
                .set_validation_block_public(&account_id, block_until, &error_text)
                .await
            {
                tracing::error!("Failed to set validation block: {}", e);
            }
        }

        // 设置 is_forbidden 状态
        if let Err(e) = st
            .token_manager
            .set_forbidden(&account_id, &error_text)
            .await
        {
            tracing::error!("Failed to set forbidden status for {}: {}", email, e);
        } else {
            tracing::warn!("[Claude] Account {} marked as forbidden due to 403", email);
        }
    }

    // [FIX session-1M] Handle st.upstream session token accumulation > 1M
    if status_code == 400 && error_text.contains("exceeds the maximum number of tokens") {
        let fingerprint = session_id_str.as_str();
        let generation = crate::proxy::common::session::bump_session(&account_id, fingerprint);
        tracing::warn!(
        "[Claude] Upstream session token accumulation exceeded 1M on account {}. sessionId bumped to generation {}, retrying with a fresh st.upstream session.",
        email, generation
    );
        return ErrorOutcome::Retry;
    }

    let scheduling_mode = st.token_manager.get_scheduling_mode().await;
    let allow_grace = match scheduling_mode {
        crate::proxy::sticky_config::SchedulingMode::Balance => {
            st.token_manager.tokens_count() <= 1
        }
        crate::proxy::sticky_config::SchedulingMode::CacheFirst => true,
        crate::proxy::sticky_config::SchedulingMode::PerformanceFirst => false,
    };

    // 确定重试策略：传入当前 attempt 与 st.pool_size，执行智能自适应裁决
    let retry_strategy = super::common::determine_retry_strategy_adaptive(
        status_code,
        &error_text,
        retry_after.as_deref(),
        st.retried_without_thinking,
        allow_grace,
        attempt,
        st.pool_size,
    );

    // 执行退避
    if apply_retry_strategy(
        retry_strategy.clone(),
        attempt,
        st.max_attempts,
        status_code,
        &st.trace_id,
    )
    .await
    {
        // 判断是否需要轮换账号
        if !should_rotate_account(status_code, Some(&retry_strategy)) {
            debug!(
                "[{}] Keeping same account for status {} (Grace Retry or Server Issue)",
                st.trace_id, status_code
            );
            st.force_rotate = false;
        } else {
            st.force_rotate = true;
        }
        return ErrorOutcome::Retry;
    } else {
        // 不可重试的错误，直接返回双轨制友好报文
        error!(
            "[{}] Non-retryable error {}: {}",
            st.trace_id, status_code, error_text
        );
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "claude",
            status_code,
            &request_with_mapped.model,
            &error_text,
        );
        return ErrorOutcome::Respond(
            (
                status,
                [
                    ("X-Account-Email", email.as_str()),
                    ("X-Mapped-Model", request_with_mapped.model.as_str()),
                ],
                Json(dual_err),
            )
                .into_response(),
        );
    }
}

/// Build the final error response when all retry attempts are exhausted.
/// Extracted from the tail of `handle_messages`. Pure code move: no logic changes.
pub(crate) fn build_exhaustion_error(
    last_email: Option<String>,
    last_mapped_model: Option<String>,
    last_status: StatusCode,
    last_error: &str,
) -> Response {
    if let Some(email) = last_email {
        // [FIX] Include X-Mapped-Model in exhaustion error
        let mut headers = HeaderMap::new();
        if let Ok(email_value) = header::HeaderValue::from_str(&email) {
            headers.insert("X-Account-Email", email_value);
        }
        if let Some(ref model) = last_mapped_model {
            if let Ok(v) = header::HeaderValue::from_str(model) {
                headers.insert("X-Mapped-Model", v);
            }
        }

        let _error_type = match last_status.as_u16() {
            400 => "invalid_request_error",
            401 => "authentication_error",
            403 => "permission_error",
            429 => "rate_limit_error",
            529 => "overloaded_error",
            _ => "api_error",
        };

        // [FIX] 403 时返回 503，避免 Claude Code 客户端退出到登录页
        let response_status = if last_status.as_u16() == 403 {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            last_status
        };

        if let Some(sec) = crate::proxy::handlers::common::extract_retry_after_seconds(&last_error)
        {
            if let Ok(val) = header::HeaderValue::from_str(&sec.to_string()) {
                headers.insert(axum::http::header::RETRY_AFTER, val);
            }
        }

        let model_str = last_mapped_model.as_deref().unwrap_or("unknown");
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "claude",
            response_status.as_u16(),
            model_str,
            &last_error,
        );

        (response_status, headers, Json(dual_err)).into_response()
    } else {
        // Fallback if no email (e.g. mapping error before token)
        let mut headers = HeaderMap::new();
        if let Some(ref model) = last_mapped_model {
            if let Ok(v) = header::HeaderValue::from_str(model) {
                headers.insert("X-Mapped-Model", v);
            }
        }
        if let Some(sec) = crate::proxy::handlers::common::extract_retry_after_seconds(&last_error)
        {
            if let Ok(val) = header::HeaderValue::from_str(&sec.to_string()) {
                headers.insert(axum::http::header::RETRY_AFTER, val);
            }
        }

        let _error_type = match last_status.as_u16() {
            400 => "invalid_request_error",
            401 => "authentication_error",
            403 => "permission_error",
            429 => "rate_limit_error",
            529 => "overloaded_error",
            _ => "api_error",
        };

        // [FIX] 403 时返回 503，避免 Claude Code 客户端退出到登录页
        let response_status = if last_status.as_u16() == 403 {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            last_status
        };

        let model_str = last_mapped_model.as_deref().unwrap_or("unknown");
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "claude",
            response_status.as_u16(),
            model_str,
            &last_error,
        );

        (response_status, headers, Json(dual_err)).into_response()
    }
}
