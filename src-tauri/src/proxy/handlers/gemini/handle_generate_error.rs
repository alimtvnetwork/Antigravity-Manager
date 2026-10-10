use super::*;

/// Handle an upstream error: classify, decide retry strategy, update retry state.
/// Extracted from `handle_generate` to keep file sizes manageable.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_generate_error(
    status: StatusCode,
    response: rquest::Response,
    debug_cfg: &crate::proxy::config::DebugLoggingConfig,
    trace_id: &str,
    model_name: &str,
    mapped_model: &str,
    session_id: &str,
    client_session_id: &str,
    account_id: &str,
    email: &str,
    upstream_url: &str,
    attempt: usize,
    max_attempts: usize,
    pool_size: usize,
    token_manager: &std::sync::Arc<crate::proxy::TokenManager>,
    _headers: &axum::http::HeaderMap,
    client_adapter: &Option<std::sync::Arc<dyn crate::proxy::common::client_adapter::ClientAdapter>>,
    failure_statuses: &mut crate::proxy::handlers::common::FailureStatusTracker,
    last_error: &mut String,
    force_rotate: &mut bool,
    retried_without_thinking: &mut bool,
    retry_credentials: &mut Option<(String, String, String, String, u64)>,
    retry_state: &mut crate::proxy::handlers::common::RequestRetryState,
    image_permit: &mut Option<crate::proxy::server::image_scheduler::ImagePermit>,
    config: &crate::proxy::mappers::common_utils::RequestConfig,
    body: &mut serde_json::Value,
    access_token: &str,
    project_id: &str,
) -> Result<ErrorOutcome, (StatusCode, String)> {
    failure_statuses.record(status);
    let status_code = status.as_u16();
    let retry_after = response
        .headers()
        .get("Retry-After")
        .and_then(|header| header.to_str().ok())
        .map(str::to_string);
    let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| format!("HTTP {}", status_code));
    *last_error = format!("HTTP {}: {}", status_code, error_text);
    if debug_logger::is_enabled(&debug_cfg) {
        let payload = json!({
            "kind": "upstream_response_error",
            "protocol": "gemini",
            "trace_id": trace_id,
            "original_model": model_name,
            "mapped_model": mapped_model,
            "request_type": config.request_type,
            "attempt": attempt,
            "status": status_code,
            "upstream_url": upstream_url,
            "account": mask_email(email),
            "error_text": error_text,
        });
        debug_logger::write_debug_payload(
            &debug_cfg,
            Some(trace_id),
            "upstream_response_error",
            &payload,
        )
        .await;
    }

    // [FIX] 403 时优先检测 VALIDATION_REQUIRED 并设置 is_forbidden / validation_block 状态，确保及时提取 URL 与更新 UI
    if status_code == 403 {
        if let Some(acc_id) = token_manager.get_account_id_by_email(email) {
            if error_text.contains("VALIDATION_REQUIRED")
                || error_text.contains("verify your account")
                || error_text.contains("Verify your account")
                || error_text.contains("validation_url")
            {
                tracing::warn!(
                    "[Gemini] VALIDATION_REQUIRED detected on account {}, temporarily blocking",
                    email
                );
                let block_minutes = 10i64;
                let block_until = chrono::Utc::now().timestamp() + (block_minutes * 60);

                if let Err(e) = token_manager
                    .set_validation_block_public(&acc_id, block_until, &error_text)
                    .await
                {
                    tracing::error!("Failed to set validation block: {}", e);
                }
            }

            // 设置 is_forbidden 状态并持久化
            if let Err(e) = token_manager.set_forbidden(&acc_id, &error_text).await {
                tracing::error!("Failed to set forbidden status: {}", e);
            }
        }
    }

    // [FIX] 429 时立即解绑当前会话，确保换号重试与后续请求不会死锁在受限账号上
    if status_code == 429 || status_code == 529 {
        token_manager.clear_session_binding(session_id);
        tracing::debug!(
            "[Gemini] Unbound session {} from account {} due to status {}",
            session_id,
            email,
            status_code
        );
    }

    let scheduling_mode = token_manager.get_scheduling_mode().await;
    let _allow_grace = match scheduling_mode {
        crate::proxy::sticky_config::SchedulingMode::Balance => token_manager.tokens_count() <= 1,
        crate::proxy::sticky_config::SchedulingMode::CacheFirst => true,
        crate::proxy::sticky_config::SchedulingMode::PerformanceFirst => false,
    };

    // Determine retry strategy: pass current attempt and pool_size for adaptive arbitration
    let strategy = retry_state.determine_strategy_adaptive(
        account_id,
        status_code,
        &error_text,
        retry_after.as_deref(),
        *retried_without_thinking,
        attempt,
        pool_size,
    );
    // Unified pipeline classification: protocol-agnostic rate limit and error classification
    let classification = crate::proxy::pipeline::UpstreamClassification::classify(
        status_code,
        &error_text,
        retry_after.as_deref(),
    );

    if classification.is_model_not_found() {
        tracing::warn!(
            "[Gemini] Target model [{}] not found on upstream (HTTP {}). Terminating retry loop without account lockout.",
            mapped_model, status_code
        );
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "gemini",
            status_code,
            mapped_model,
            &error_text,
        );
        return Ok(ErrorOutcome::Respond(
            (
                StatusCode::from_u16(status_code).unwrap_or(StatusCode::NOT_FOUND),
                [
                    ("X-Account-Email", email),
                    ("X-Mapped-Model", mapped_model),
                ],
                Json(dual_err),
            )
                .into_response(),
        ));
    }

    if classification.is_thought_signature_error() {
        if !*retried_without_thinking {
            *retried_without_thinking = true;
            tracing::warn!(
                "[Gemini] Pipeline: Thinking signature error detected on upstream (HTTP {}). Surgically purging corrupted signatures and retrying on same account.",
                status_code
            );
            // 1. Purge corrupted signatures in ThinkingStore
            crate::proxy::thinking_store::ThinkingStore::global()
                .purge_corrupted_signatures(session_id, mapped_model);
            // 2. Clear session signature cache
            crate::proxy::SignatureCache::global().delete_session_signature(client_session_id);
            // 3. Append recovery prompt to the last content item
            if let Some(contents) = body.get_mut("contents").and_then(|v| v.as_array_mut()) {
                if let Some(last_content) = contents.last_mut() {
                    if let Some(parts) =
                        last_content.get_mut("parts").and_then(|v| v.as_array_mut())
                    {
                        parts.push(json!({
                            "text": "\n\n[System Recovery] Your previous output contained an invalid signature. Please regenerate the response without the corrupted signature block."
                        }));
                        tracing::debug!("[Gemini] Appended repair prompt to last content");
                    }
                }
            }
            // 4. Retry on same account
            *force_rotate = false;
            return Ok(ErrorOutcome::Continue);
        } else {
            tracing::warn!(
                "[Gemini] Pipeline: Thinking signature error persisted after retry without thinking. Terminating retry loop."
            );
        }
    }

    let should_mark_limited = classification.should_lock_account();
    let needs_quota_refresh = if should_mark_limited {
        token_manager
            .mark_rate_limited_fast(
                email,
                status_code,
                retry_after.as_deref(),
                &error_text,
                Some(mapped_model),
            )
            .await
    } else {
        false
    };
    if !matches!(&strategy, RetryStrategy::GraceRetry(_)) {
        drop(image_permit.take());
    }
    if needs_quota_refresh {
        token_manager
            .refresh_quota_lock_after_fast_mark(email, Some(mapped_model))
            .await;
    }
    let trace_id = format!("gemini_{}", session_id);

    // 执行退避
    if apply_retry_strategy(
        strategy.clone(),
        attempt,
        max_attempts,
        status_code,
        &trace_id,
    )
    .await
    {
        if matches!(strategy, RetryStrategy::GraceRetry(_)) {
            *retry_credentials = Some((
                access_token.to_string(),
                project_id.to_string(),
                email.to_string(),
                account_id.to_string(),
                0,
            ));
        }
        // [NEW] Apply Client Adapter "let_it_crash" strategy
        if let Some(adapter) = &client_adapter {
            if adapter.let_it_crash() && attempt > 0 {
                tracing::warn!(
                    "[Gemini] let_it_crash active: Aborting retries after attempt {}",
                    attempt
                );
                return Ok(ErrorOutcome::Break);
            }
        }

        // 判断是否需要轮换账号
        if !should_rotate_account(status_code, Some(&strategy)) {
            debug!(
                "[{}] Keeping same account for status {} (Gemini server-side issue or Grace Retry)",
                trace_id, status_code
            );
            *force_rotate = false;
        } else {
            *force_rotate = true;
        }

        return Ok(ErrorOutcome::Continue);
    }

    // 404 等由于模型配置或路径错误的 HTTP 异常，直接报错，不进行无效轮换
    error!(
        "Gemini Upstream non-retryable error {}: {}",
        status_code, error_text
    );
    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
        "gemini",
        status_code,
        mapped_model,
        &error_text,
    );
    return Ok(ErrorOutcome::Respond(
        (
            status,
            [
                ("X-Account-Email", email),
                ("X-Mapped-Model", mapped_model),
            ],
            Json(dual_err),
        )
            .into_response(),
    ));
}
