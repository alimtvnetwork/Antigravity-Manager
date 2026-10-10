// Phase 3 of `handle_chat_completions`: error classification, account
// rotation / retry decisions.
use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::proxy::common::client_adapter::{ClientAdapter, CLIENT_ADAPTERS};
use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    apply_retry_strategy, should_rotate_account, FailureStatusTracker, RequestRetryState,
    RetryStrategy,
};
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::session_manager::SessionManager;
use crate::proxy::TokenManager;

use super::chat_completions::ChatAttemptOutcome;
use super::chat_completions_send::ChatSendOutput;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn chat_completions_error(
    send: ChatSendOutput,
    openai_req: &OpenAIRequest,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    client_session_id: String,
    failure_statuses: &mut FailureStatusTracker,
    force_rotate: &mut bool,
    last_error: &mut String,
    retry_credentials: &mut Option<(String, String, String, String, u64)>,
    retry_state: &mut RequestRetryState,
    retried_without_thinking: &mut bool,
    max_attempts: usize,
    token_manager: Arc<TokenManager>,
    client_adapter: Option<Arc<dyn ClientAdapter>>,
    pool_size: usize,
    image_permit: &mut Option<crate::proxy::server::ImagePermit>,
) -> Result<ChatAttemptOutcome, (StatusCode, String)> {
    let ChatSendOutput {
        response,
        status,
        upstream_url,
        session_id,
        config,
        mapped_model,
        email,
        account_id,
        access_token,
        project_id,
        ..
    } = send;
    // 处理特定错误并重试
    failure_statuses.record(status);
    let status_code = status.as_u16();
    let retry_after = response
        .headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| format!("HTTP {}", status_code));
    last_error = format!("HTTP {}: {}", status_code, error_text);

    // [New] 打印错误报文日志
    tracing::error!(
        "[OpenAI-Upstream] Error Response {}: {}",
        status_code,
        error_text
    );
    if debug_logger::is_enabled(&debug_cfg) {
        let payload = json!({
            "kind": "upstream_response_error",
            "protocol": "openai",
            "trace_id": trace_id,
            "original_model": openai_req.model,
            "mapped_model": mapped_model,
            "request_type": config.request_type,
            "attempt": attempt,
            "status": status_code,
            "upstream_url": upstream_url,
            "account": mask_email(&email),
            "error_text": error_text,
        });
        debug_logger::write_debug_payload(
            &debug_cfg,
            Some(&trace_id),
            "upstream_response_error",
            &payload,
        )
        .await;
    }

    let scheduling_mode = token_manager.get_scheduling_mode().await;
    let _allow_grace = match scheduling_mode {
        crate::proxy::sticky_config::SchedulingMode::Balance => token_manager.tokens_count() <= 1,
        crate::proxy::sticky_config::SchedulingMode::CacheFirst => true,
        crate::proxy::sticky_config::SchedulingMode::PerformanceFirst => false,
    };

    // 确定重试策略：传入当前 attempt 与 pool_size，执行智能自适应裁决
    let strategy = retry_state.determine_strategy_adaptive(
        &account_id,
        status_code,
        &error_text,
        retry_after.as_deref(),
        retried_without_thinking,
        attempt,
        pool_size,
    );
    // 统一流水线决策判定：协议无关的限流与错误判定
    let classification = crate::proxy::pipeline::UpstreamClassification::classify(
        status_code,
        &error_text,
        retry_after.as_deref(),
    );

    if classification.is_model_not_found() {
        tracing::warn!(
                "[{}] Pipeline: Target model [{}] not found on upstream (HTTP {}). Terminating retry loop without account lockout.",
                trace_id, mapped_model, status_code
            );
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            "openai",
            status_code,
            &mapped_model,
            &error_text,
        );
        return Ok(ChatAttemptOutcome::Respond(
            (
                StatusCode::from_u16(status_code).unwrap_or(StatusCode::NOT_FOUND),
                [
                    ("X-Account-Email", email.as_str()),
                    ("X-Mapped-Model", mapped_model.as_str()),
                ],
                Json(dual_err),
            )
                .into_response(),
        ));
    }

    if classification.is_thought_signature_error() {
        if !retried_without_thinking {
            retried_without_thinking = true;
            tracing::warn!(
                    "[{}] Pipeline: Thinking signature error detected on upstream (HTTP {}). Surgically purging corrupted signatures and retrying on same account.",
                    trace_id, status_code
                );
            // 1. 精准定向净化 ThinkingStore 中的异构污染签名（保留思考文本与健康签名）
            crate::proxy::thinking_store::ThinkingStore::global()
                .purge_corrupted_signatures(&session_id, &mapped_model);
            // 2. 清理当前 session 的 SignatureCache
            crate::proxy::SignatureCache::global().delete_session_signature(&client_session_id);
            // 3. 保持同一账号原地重试
            force_rotate = false;
            return Ok(ChatAttemptOutcome::Continue);
        } else {
            tracing::warn!(
                    "[{}] Pipeline: Thinking signature error persisted after retry without thinking. Terminating retry loop.",
                    trace_id
                );
        }
    }

    let should_mark_limited = classification.should_lock_account();
    let needs_quota_refresh = if config.request_type == "image_gen" && should_mark_limited {
        token_manager
            .mark_rate_limited_fast(
                &email,
                status_code,
                retry_after.as_deref(),
                &error_text,
                Some(&mapped_model),
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
            .refresh_quota_lock_after_fast_mark(&email, Some(&mapped_model))
            .await;
    }

    // 3. 标记限流状态(用于 UI 显示)
    if config.request_type != "image_gen" && should_mark_limited {
        // [FIX] Use async version with model parameter for fine-grained rate limiting
        token_manager
            .mark_rate_limited_async(
                &email,
                status_code,
                retry_after.as_deref(),
                &error_text,
                Some(&mapped_model),
            )
            .await;
    }

    if status_code == 429 || status_code == 529 {
        token_manager
            .unbind_session_and_clear_last_used(Some(&session_id))
            .await;
    }

    // [FIX] 403 时优先检测 VALIDATION_REQUIRED 并设置 is_forbidden / validation_block 状态，确保及时提取 URL 与更新 UI
    if status_code == 403 {
        if let Some(acc_id) = token_manager.get_account_id_by_email(&email) {
            if error_text.contains("VALIDATION_REQUIRED")
                || error_text.contains("verify your account")
                || error_text.contains("Verify your account")
                || error_text.contains("validation_url")
            {
                tracing::warn!(
                    "[OpenAI] VALIDATION_REQUIRED detected on account {}, temporarily blocking",
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
            retry_credentials = Some((
                access_token.clone(),
                project_id.clone(),
                email.clone(),
                account_id.clone(),
                0,
            ));
        }
        // [NEW] Apply Client Adapter "let_it_crash" strategy
        if let Some(adapter) = &client_adapter {
            if adapter.let_it_crash() && attempt > 0 {
                tracing::warn!(
                    "[OpenAI] let_it_crash active: Aborting retries after attempt {}",
                    attempt
                );
                return Ok(ChatAttemptOutcome::Break);
            }
        }

        // 判断是否需要轮换账号
        if !should_rotate_account(status_code, Some(&strategy)) {
            debug!(
                "[{}] Keeping same account for status {} (Grace Retry or Server Issue)",
                trace_id, status_code
            );
            force_rotate = false;
        } else {
            force_rotate = true;
        }

        tracing::warn!(
            "OpenAI Upstream {} on {} attempt {}/{}, rotating account",
            status_code,
            email,
            attempt + 1,
            max_attempts
        );
        return Ok(ChatAttemptOutcome::Continue);
    }

    // [FIX session-1M] 上游按 sessionId 在服务端累计会话输入,长工具循环会把累计推过 1M,
    // 之后该 sessionId 的所有请求都 400 "input token count exceeds ... 1048576"。
    // 给 (账号, 对话) 的 sessionId 升代并立即重试:新 sessionId = 上游全新会话,对话无感恢复。
    if status_code == 400 && error_text.contains("exceeds the maximum number of tokens") {
        let fingerprint = SessionManager::extract_openai_session_id(openai_req);
        let generation = crate::proxy::common::session::bump_session(&account_id, &fingerprint);
        tracing::warn!(
                "[OpenAI] Upstream session token accumulation exceeded 1M on account {}. sessionId bumped to generation {}, retrying with a fresh upstream session.",
                email, generation
            );
        return Ok(ChatAttemptOutcome::Continue); // 重试:下一轮 transform 时读取新代数,派生全新 sessionId
    }

    // 404 等由于模型配置或路径错误的 HTTP 异常，直接报错返回双轨制友好报文，不进行无效轮换
    error!(
        "OpenAI Upstream non-retryable error {} on account {}: {}",
        status_code, email, error_text
    );
    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
        "openai",
        status_code,
        &mapped_model,
        &error_text,
    );
    return Ok(ChatAttemptOutcome::Respond(
        (
            status,
            [
                ("X-Account-Email", email.as_str()),
                ("X-Mapped-Model", mapped_model.as_str()),
            ],
            Json(dual_err),
        )
            .into_response(),
    ));
}
