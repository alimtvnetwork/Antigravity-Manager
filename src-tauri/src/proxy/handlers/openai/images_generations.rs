// `POST /v1/images/generations`.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::Body, extract::Json, extract::State, http::StatusCode, response::IntoResponse,
    response::Response,
};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    next_rotation_attempt, FailureStatusTracker, RequestRetryState, RetryStrategy,
};
use crate::proxy::monitor::UpstreamRequestBodyHolder;
use crate::proxy::server::{AppState, UpstreamClient};
use crate::proxy::session_manager::SessionManager;
use crate::proxy::TokenManager;

use super::image_input::{build_image_edit_body, NormalizedInputImage};
use super::images_intercept::intercept_chat_to_image;
use crate::proxy::handlers::common::apply_retry_strategy;
use crate::proxy::handlers::common::retrystrategy::should_rotate_account;
use crate::proxy::handlers::openai::image_input::build_image_contents;
use crate::proxy::handlers::openai::image_input::generation_image_size_param;
use crate::proxy::handlers::openai::image_input::parse_generation_input_images;
use crate::proxy::handlers::openai::responses_media::response_has_inline_image_data;
use std::time::Duration;

pub async fn handle_images_generations(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match handle_images_generations_internal(state, body).await {
        Ok((email_header, openai_response)) => Ok((
            StatusCode::OK,
            [("X-Account-Email", email_header.as_str())],
            Json(openai_response),
        )
            .into_response()),
        // Attach the attempted account to error responses too, so the traffic log shows
        // which account the failed (e.g. 502/503) image request used.
        Err((status, msg, email_opt)) => {
            let email = email_opt.unwrap_or_default();
            Ok((status, [("X-Account-Email", email)], msg).into_response())
        }
    }
}

pub async fn handle_images_generations_internal(
    state: AppState,
    body: Value,
) -> Result<(String, Value), (StatusCode, String, Option<String>)> {
    // 1. 解析请求参数
    let prompt = body.get("prompt").and_then(|v| v.as_str()).ok_or((
        StatusCode::BAD_REQUEST,
        "Missing 'prompt' field".to_string(),
        None,
    ))?;

    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-3.1-flash-image");

    let n = body.get("n").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

    let size = body.get("size").and_then(|v| v.as_str());

    let response_format = body
        .get("response_format")
        .and_then(|v| v.as_str())
        .unwrap_or("b64_json");

    let quality = body.get("quality").and_then(|v| v.as_str());

    let image_size = generation_image_size_param(&body)
        .map_err(|message| (StatusCode::BAD_REQUEST, message, None))?;

    let style = body
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("vivid");

    // Canvas compatibility extension: OpenAI's standard generations endpoint does not define
    // this top-level field. Accept only inline data:image URLs and never fetch remote URLs.
    let input_images = parse_generation_input_images(body.get("image"))
        .map_err(|message| (StatusCode::BAD_REQUEST, message, None))?;

    info!(
        model = model,
        image_count = input_images.len(),
        n = n,
        size = size.unwrap_or("auto"),
        quality = quality.unwrap_or("auto"),
        style = style,
        "[Images] Received generation request"
    );

    // 2. 使用 common_utils 解析图片配置（统一逻辑，支持动态计算宽高比和 quality 映射）
    let (image_config, clean_model_name) =
        crate::proxy::mappers::common_utils::try_parse_image_config_with_params(
            model, size, quality, image_size,
        )
        .map_err(|message| (StatusCode::BAD_REQUEST, message, None))?;

    // 3. Prompt Enhancement（保留原有逻辑）
    let mut final_prompt = prompt.to_string();
    if quality == Some("hd") {
        final_prompt.push_str(", (high quality, highly detailed, 4k resolution, hdr)");
    }
    match style {
        "vivid" => final_prompt.push_str(", (vivid colors, dramatic lighting, rich details)"),
        "natural" => final_prompt.push_str(", (natural lighting, realistic, photorealistic)"),
        _ => {}
    }
    let contents_parts = build_image_contents(final_prompt, &input_images, None);

    // 4. 并发发送请求
    // 注意：不再在外部获取 Token，而是移入 Task 内部并在重试时获取
    let upstream = state.upstream.clone();
    let token_manager = state.token_manager.clone();
    let image_scheduler = state.image_scheduler.clone();
    let request_timeout = state.request_timeout;
    let max_pool_size = token_manager.len();
    // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
    let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(max_pool_size);

    let mut tasks = JoinSet::new();

    // Track the last account actually attempted, so error responses (502/503) can be
    // attributed to an account in the traffic log instead of showing "(none)".
    let attempted_account = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));

    for _ in 0..n {
        let upstream = upstream.clone();
        let token_manager = token_manager.clone();
        let contents_parts = contents_parts.clone();
        let image_config = image_config.clone(); // 使用解析后的完整配置
        let _response_format = response_format.to_string();

        let model_to_use = clean_model_name.clone();
        let attempted_account = attempted_account.clone();
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
                let (access_token, project_id, email, account_id, _wait_ms) =
                    if let Some(credentials) = retry_credentials.take() {
                        credentials
                    } else {
                        drop(image_permit.take());
                        match token_manager
                            .get_image_token(
                                force_rotate,
                                None,
                                &model_to_use,
                                &image_scheduler,
                                request_timeout,
                            )
                            .await
                        {
                            Ok((
                                access_token,
                                project_id,
                                email,
                                account_id,
                                wait_ms,
                                permit,
                            )) => {
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
                if let Ok(mut g) = attempted_account.lock() {
                    *g = Some(email.clone());
                }

                // [FIX] Resolve to the account-specific dynamic image model, exactly like the
                // chat (openai.rs:232) and gemini (gemini.rs:155) handlers do. Sending the static
                // alias (e.g. "gemini-3-pro-image") made upstream return 404 "Requested entity was
                // not found" because each account exposes its own concrete image model id.
                let resolved_model = token_manager
                    .resolve_dynamic_model_for_account(&account_id, &model_to_use)
                    .await;

                let gemini_body = json!({
                    "project": project_id,
                    "requestId": format!("agent-{}", uuid::Uuid::new_v4()),
                    "model": resolved_model,
                    "userAgent": "antigravity",
                    "requestType": "image_gen",
                    "request": {
                        "contents": [{
                            "role": "user",
                            "parts": contents_parts
                        }],
                        "generationConfig": {
                            "candidateCount": 1, // 强制单张
                            "imageConfig": image_config // ✅ 使用完整配置（包含 aspectRatio 和 imageSize）
                        },
                        "safetySettings": [
                            { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
                            { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
                            { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
                            { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
                        ]
                    }
                });

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
                            let classification = crate::proxy::pipeline::UpstreamClassification::classify(
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
                                    "image_generation",
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
                                force_rotate = true;
                                continue; // Retry loop
                            }

                            // [FIX] 403/404 usually mean THIS account lacks the image model or
                            // project access. Rotate to another account instead of failing the
                            // whole request, so an image-capable account can serve it.
                            if (status_code == 403 || status_code == 404)
                                && attempt < max_attempts - 1
                            {
                                tracing::warn!(
                                    "[Images] Account {} returned {} for image gen, rotating to another account",
                                    email,
                                    status_code
                                );
                                force_rotate = true;
                                continue;
                            }

                            // Other errors: return
                            return Err((failure_statuses.final_status(), last_error));
                        }
                        match response.json::<Value>().await {
                            Ok(json) => {
                                if response_has_inline_image_data(&json) {
                                    token_manager.mark_account_success(&account_id);
                                    token_manager
                                        .clear_persisted_live_limit(
                                            &account_id,
                                            Some(&model_to_use),
                                        );
                                }
                                return Ok((json, email));
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

            // All attempts failed
            Err((
                failure_statuses.final_status(),
                format!("Max retries exhausted. Last error: {}", last_error),
            ))
        });
    }

    // 5. 收集结果
    let mut images: Vec<Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut used_email: Option<String> = None;
    let mut failure_statuses = FailureStatusTracker::default();

    while let Some(task) = tasks.join_next().await {
        match task {
            Ok(result) => match result {
                Ok((gemini_resp, email_used)) => {
                    // Capture the email from the first successful task for logging
                    if used_email.is_none() {
                        used_email = Some(email_used);
                    }
                    let raw = gemini_resp.get("response").unwrap_or(&gemini_resp);
                    if let Some(parts) = raw
                        .get("candidates")
                        .and_then(|c| c.get(0))
                        .and_then(|cand| cand.get("content"))
                        .and_then(|content| content.get("parts"))
                        .and_then(|p| p.as_array())
                    {
                        for part in parts {
                            if let Some(img) = part.get("inlineData") {
                                let data = img.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                if !data.is_empty() {
                                    if response_format == "url" {
                                        let mime_type = img
                                            .get("mimeType")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("image/png");
                                        images.push(json!({
                                            "url": format!("data:{};base64,{}", mime_type, data)
                                        }));
                                    } else {
                                        images.push(json!({
                                            "b64_json": data
                                        }));
                                    }
                                    tracing::debug!("[Images] Task succeeded");
                                }
                            }
                        }
                    }
                }
                Err((status, e)) => {
                    tracing::error!("[Images] Task failed: {}", e);
                    failure_statuses.record(status);
                    errors.push(e);
                }
            },
            Err(e) => {
                let err_msg = format!("Task join error: {}", e);
                tracing::error!("[Images] Task join error: {}", e);
                failure_statuses.record(StatusCode::BAD_GATEWAY);
                errors.push(err_msg);
            }
        }
    }

    if images.is_empty() {
        let error_msg = if !errors.is_empty() {
            errors.join("; ")
        } else {
            "No images generated".to_string()
        };
        tracing::error!("[Images] All {} requests failed. Errors: {}", n, error_msg);

        let status = failure_statuses.final_status();

        let attempted = used_email
            .clone()
            .or_else(|| attempted_account.lock().ok().and_then(|g| g.clone()));
        return Err((status, error_msg, attempted));
    }

    // 部分成功时记录警告
    if !errors.is_empty() {
        tracing::warn!(
            "[Images] Partial success: {} out of {} requests succeeded. Errors: {}",
            images.len(),
            n,
            errors.join("; ")
        );
    }

    tracing::info!(
        "[Images] Successfully generated {} out of {} requested image(s)",
        images.len(),
        n
    );

    // 6. 构建 OpenAI 格式响应
    let openai_response = json!({
        "created": chrono::Utc::now().timestamp(),
        "data": images
    });

    // [FIX] 图像生成成功后触发配额刷新 (Issue #1995)
    tokio::spawn(async move {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::account::refresh::refresh_all_quotas_logic().await,
            "refresh_all_quotas_logic",
        );
    });

    let email_header = used_email.unwrap_or_default();
    Ok((email_header, openai_response))
}
