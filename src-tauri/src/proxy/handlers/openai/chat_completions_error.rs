// Phase 3 of `handle_chat_completions`: error classification, account
// rotation / retry decisions.
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
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
use crate::proxy::upstream::client::types::mask_email;
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
    attempt: usize,
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
    // Handle specific errors and try again
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

    // [New] Print error message log
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

    // Determine the retry strategy: pass in the current attempt and pool_size, and perform intelligent adaptive decision-making
    let strategy = retry_state.determine_strategy_adaptive(
        &account_id,
        status_code,
        &error_text,
        retry_after.as_deref(),
        *retried_without_thinking,
        attempt,
        pool_size,
    );
    // Unified pipeline decision-making: protocol-independent current limiting and error determination
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
        if !*retried_without_thinking {
            *retried_without_thinking = true;
            tracing::warn!(
                    "[{}] Pipeline: Thinking signature error detected on upstream (HTTP {}). Surgically purging corrupted signatures and retrying on same account.",
                    trace_id, status_code
                );
            // 1. Precisely targeted purification of heterogeneous pollution signatures in ThinkingStore (retaining thinking text and healthy signatures)
            crate::proxy::thinking_store::ThinkingStore::global()
                .purge_corrupted_signatures(&session_id, &mapped_model);
            // 2. Clear the SignatureCache of the current session
            crate::proxy::SignatureCache::global().delete_session_signature(&client_session_id);
            // 3. Keep the same account and try again.
            *force_rotate = false;
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

    // 3. Mark the current limiting status (for UI display)
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

    // [FIX] When 403 is detected, VALIDATION_REQUIRED is detected first and the is_forbidden / validation_block status is set to ensure timely extraction of URLs and UI updates.
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

            // Set is_forbidden state and persist it
            if let Err(e) = token_manager.set_forbidden(&acc_id, &error_text).await {
                tracing::error!("Failed to set forbidden status: {}", e);
            }
        }
    }

    // Perform backoff
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

        // Determine whether accounts need to be rotated
        if !should_rotate_account(status_code, Some(&strategy)) {
            debug!(
                "[{}] Keeping same account for status {} (Grace Retry or Server Issue)",
                trace_id, status_code
            );
            *force_rotate = false;
        } else {
            *force_rotate = true;
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

    // [FIX session-1M] The upstream accumulates session input on the server side by sessionId. A long tool loop will push the accumulation past 1M.
    // All subsequent requests for this sessionId are 400 "input token count exceeds ... 1048576".
    // Upgrade the sessionId of (account, conversation) and try again immediately: new sessionId = new upstream session, conversation will be restored without any sense.
    if status_code == 400 && error_text.contains("exceeds the maximum number of tokens") {
        let fingerprint = SessionManager::extract_openai_session_id(openai_req);
        let generation = crate::proxy::common::session::bump_session(&account_id, &fingerprint);
        tracing::warn!(
                "[OpenAI] Upstream session token accumulation exceeded 1M on account {}. sessionId bumped to generation {}, retrying with a fresh upstream session.",
                email, generation
            );
        return Ok(ChatAttemptOutcome::Continue); // Retry: Read the new algebra in the next round of transform and derive a new sessionId
    }

    // For HTTP exceptions such as 404 due to incorrect model configuration or path, an error will be reported directly and a dual-track friendly message will be returned without invalid rotation.
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
