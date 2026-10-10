// Task spawner for `handle_images_edits`: one upstream attempt per account.
use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::Value;
use tokio::task::JoinSet;

use crate::proxy::server::UpstreamClient;
use crate::proxy::TokenManager;
use crate::proxy::handlers::common::retrystrategy::should_rotate_account;
use crate::proxy::handlers::openai::image_input::build_image_edit_body;
use crate::proxy::handlers::openai::image_input::NormalizedInputImage;
use crate::proxy::handlers::openai::image_input::image_account_selection_target;
use crate::proxy::handlers::openai::responses_media::response_has_inline_image_data;
use crate::proxy::handlers::common::retrystrategy::next_rotation_attempt;
use crate::proxy::handlers::common::apply_retry_strategy;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_image_edit_tasks(
    tasks: &mut JoinSet<Result<(Value, String, String), (StatusCode, String)>>,
    upstream: &Arc<UpstreamClient>,
    token_manager: &Arc<TokenManager>,
    client_adapter: &Option<Arc<dyn crate::proxy::common::client_adapter::ClientAdapter>>,
    openai_req: &serde_json::Value,
    input_images: &[super::image_input::NormalizedInputImage],
    selected: &[(usize, String, String, String, String, u64)],
    extra_headers: &axum::http::HeaderMap,
    debug_cfg: &crate::proxy::config::DebugLoggingConfig,
    trace_id: &str,
    attempt_no: usize,
) {
    // 4. 并发发送请求
    // 注意：不再在外部获取 Token，而是移入 Task 内部
    let upstream = upstream.clone();
    let token_manager = token_manager.clone();
    let image_scheduler = state.image_scheduler.clone();
    let request_timeout = state.request_timeout;
    let max_pool_size = token_manager.len();
    // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
    let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(max_pool_size);

    let mut tasks = JoinSet::new();
    for _ in 0..n {
        let upstream = upstream.clone();
        let token_manager = token_manager.clone();
        let contents_parts = contents_parts.clone();
        let image_config = image_config.clone();
        let response_format = response_format.clone();
        let model_to_use = clean_model_name.clone();
        let image_scheduler = image_scheduler.clone();

        tasks.spawn(async move {
            let mut image_permit = None;
            let mut last_error = String::new();
            let mut force_rotate = false;
            let mut retry_state = RequestRetryState::default();
            let mut retry_credentials: Option<(String, String, String, String, u64)> = None;
            let mut failure_statuses = FailureStatusTracker::default();
            let mut used_attempts = 0;

            while let Some(attempt) = next_rotation_attempt(
                &mut used_attempts,
                max_attempts,
                retry_credentials.is_some(),
            ) {
                // 4.1 获取 Token
                let (access_token, project_id, email, account_id, _wait_ms) =
                    if let Some(credentials) = retry_credentials.take() {
                        credentials
                    } else {
                        drop(image_permit.take());
                        match token_manager
                            .get_image_token(
                                force_rotate,
                                None,
                                image_account_selection_target(&model_to_use),
                                &image_scheduler,
                                request_timeout,
                            )
                            .await
                        {
                            Ok((access_token, project_id, email, account_id, wait_ms, permit)) => {
                                image_permit = Some(permit);
                                (access_token, project_id, email, account_id, wait_ms)
                            }
                            Err((status, e)) => {
                                last_error = format!("Token error: {}", e);
                                failure_statuses.record(status);
                                if status == StatusCode::TOO_MANY_REQUESTS {
                                    return Err((status, e));
                                }
                                if attempt < max_attempts - 1 {
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                    continue;
                                }
                                break;
                            }
                        }
                    };

                let resolved_model = token_manager
                    .resolve_dynamic_model_for_account(&account_id, &model_to_use)
                    .await;

                // 4.2 Construct Request Body (Need project_id and account-resolved model)
                let gemini_body = build_image_edit_body(
                    project_id.clone(),
                    &resolved_model,
                    contents_parts.clone(),
                    image_config.clone(),
                );

                match upstream
                    .call_v1_internal(
                        "generateContent",
                        &access_token,
                        gemini_body,
                        None,
                        Some(account_id.as_str()),
                    )
                    .await
                {
                    Ok(call_result) => {
                        let response = call_result.response;
                        let status = response.status();
                        if !status.is_success() {
                            failure_statuses.record(status);
                            let retry_after = response
                                .headers()
                                .get("Retry-After")
                                .and_then(|header| header.to_str().ok())
                                .map(str::to_string);
                            let err_text = response.text().await.unwrap_or_default();
                            let status_code = status.as_u16();
                            last_error = format!("Upstream error {}: {}", status, err_text);
                            let strategy = (status_code == 429).then(|| {
                                retry_state.determine_strategy(
                                    &account_id,
                                    status_code,
                                    &err_text,
                                    retry_after.as_deref(),
                                    false,
                                )
                            });
                            // 统一流水线限流裁决：500/503等服务异常绝不打入限流
                            let classification =
                                crate::proxy::pipeline::UpstreamClassification::classify(
                                    status_code,
                                    &err_text,
                                    retry_after.as_deref(),
                                );
                            let should_mark_limited = classification.should_lock_account();
                            let needs_quota_refresh = if should_mark_limited {
                                tracing::warn!(
                                    "[Images] Account {} rate limited/error ({}), rotating...",
                                    email,
                                    status_code
                                );
                                token_manager
                                    .mark_rate_limited_fast(
                                        &email,
                                        status_code,
                                        retry_after.as_deref(),
                                        &err_text,
                                        Some(&resolved_model),
                                    )
                                    .await
                            } else {
                                false
                            };
                            if !matches!(strategy.as_ref(), Some(RetryStrategy::GraceRetry(_))) {
                                drop(image_permit.take());
                            }
                            if needs_quota_refresh {
                                token_manager
                                    .refresh_quota_lock_after_fast_mark(
                                        &email,
                                        Some(&resolved_model),
                                    )
                                    .await;
                            }

                            if let Some(strategy) = strategy {
                                if apply_retry_strategy(
                                    strategy.clone(),
                                    attempt,
                                    max_attempts,
                                    status_code,
                                    "image_edit",
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
                                    force_rotate =
                                        should_rotate_account(status_code, Some(&strategy));
                                    continue;
                                }
                            }

                            if status_code == 503 || status_code == 500 {
                                continue; // Retry loop
                            }
                            return Err((failure_statuses.final_status(), last_error));
                        }
                        match response.json::<Value>().await {
                            Ok(json) => {
                                if response_has_inline_image_data(&json) {
                                    token_manager.mark_account_success(&account_id);
                                    token_manager.clear_persisted_live_limit(
                                        &account_id,
                                        Some(&model_to_use),
                                    );
                                }
                                return Ok((json, response_format.clone(), email));
                            }
                            Err(e) => {
                                return Err((
                                    StatusCode::BAD_GATEWAY,
                                    format!("Parse error: {}", e),
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        last_error = format!("Network error: {}", e);
                        failure_statuses.record(StatusCode::BAD_GATEWAY);
                        drop(image_permit.take());
                        continue;
                    }
                }
            }
            Err((
                failure_statuses.final_status(),
                format!("Max retries exhausted. Last error: {}", last_error),
            ))
        });
    }
}
