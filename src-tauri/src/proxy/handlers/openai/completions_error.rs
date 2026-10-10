// `handle_completions` error path: classification, account marking,
// retry/backoff decisions.
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::Response;
use serde_json::Value;
use tracing::error;
use axum::body::Body;

use crate::proxy::handlers::common::{
    apply_retry_strategy, should_rotate_account, FailureStatusTracker, RequestRetryState,
    RetryStrategy,
};
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::server::UpstreamClient;
use crate::proxy::TokenManager;

use super::completions::CompletionsOutcome;
use super::completions_send::CompletionsSendOutput;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn completions_handle_error(
    send: CompletionsSendOutput,
    openai_req: &OpenAIRequest,
    trace_id: String,
    session_id_str: String,
    is_responses_api: bool,
    attempt: usize,
    max_attempts: usize,
    pool_size: usize,
    token_manager: Arc<TokenManager>,
    failure_statuses: &mut FailureStatusTracker,
    force_rotate: &mut bool,
    last_error: &mut String,
    retry_credentials: &mut Option<(String, String, String, String, u64)>,
    retry_state: &mut RequestRetryState,
) -> CompletionsOutcome {
    let CompletionsSendOutput {
        response,
        status,
        upstream_url: _,
        session_id: _,
        message_count: _,
        config,
        mapped_model,
        email,
        account_id,
        access_token,
        project_id,
        ..
    } = send;
    // Handle errors and retry
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
    *last_error = format!("HTTP {}: {}", status_code, error_text);

    tracing::error!(
        "[Codex-Upstream] Error Response {}: {}",
        status_code,
        error_text
    );

    // 3. 统一流水线判定与标记限流状态(用于 UI 显示)
    let classification = crate::proxy::pipeline::UpstreamClassification::classify(
        status_code,
        &error_text,
        retry_after.as_deref(),
    );

    if classification.is_model_not_found() {
        tracing::warn!(
                "[{}] Pipeline: Target model [{}] not found on upstream (HTTP {}). Terminating completions retry loop without account lockout.",
                trace_id, mapped_model, status_code
            );
        let protocol = if is_responses_api {
            "responses"
        } else {
            "openai"
        };
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            protocol,
            status_code,
            &mapped_model,
            &error_text,
        );
        return CompletionsOutcome::Respond(
            Response::builder()
                .status(StatusCode::from_u16(status_code).unwrap_or(StatusCode::NOT_FOUND))
                .header("X-Account-Email", email.as_str())
                .header("X-Mapped-Model", mapped_model.as_str())
                .body(Body::from(
                    serde_json::to_string(&dual_err).unwrap_or_default(),
                ))
                .unwrap()
                .into_response(),
        );
    }

    if classification.should_lock_account() {
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
            .unbind_session_and_clear_last_used(Some(&session_id_str))
            .await;
    }

    let scheduling_mode = token_manager.get_scheduling_mode().await;
    let _allow_grace = match scheduling_mode {
        crate::proxy::sticky_config::SchedulingMode::Balance => token_manager.tokens_count() <= 1,
        crate::proxy::sticky_config::SchedulingMode::CacheFirst => true,
        crate::proxy::sticky_config::SchedulingMode::PerformanceFirst => false,
    };

    let strategy = retry_state.determine_strategy_adaptive(
        &account_id,
        status_code,
        &error_text,
        retry_after.as_deref(),
        false,
        attempt,
        pool_size,
    );

    // 执行退备
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
                access_token.clone(),
                project_id.clone(),
                email.clone(),
                account_id.clone(),
                0,
            ));
        }
        *force_rotate = should_rotate_account(status_code, Some(&strategy));
        return CompletionsOutcome::ContinueLoop;
    } else {
        // 不可重试
        let protocol = if is_responses_api {
            "responses"
        } else {
            "openai"
        };
        let dual_err = crate::proxy::handlers::common::build_dual_track_error(
            protocol,
            status_code,
            &mapped_model,
            &error_text,
        );
        return CompletionsOutcome::Respond(
            (
                status,
                [
                    ("X-Account-Email", email.as_str()),
                    ("X-Mapped-Model", mapped_model.as_str()),
                ],
                axum::Json(dual_err),
            )
                .into_response(),
        );
    }
}
