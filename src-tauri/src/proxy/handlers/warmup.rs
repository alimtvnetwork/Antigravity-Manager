// Warmup Processor - Internal Warmup API
//
// Provides /internal/warmup endpoint, supports:
// - Specify account (via email)
// - Specify the model (no mapping, use the original model name directly)
// - Reuse all infrastructure of the proxy (UpstreamClient, TokenManager)

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::proxy::mappers::gemini::wrapper::wrap_request;
use crate::proxy::monitor::ProxyRequestLog;
use crate::proxy::server::AppState;

/// Warm up request body
#[derive(Debug, Deserialize)]
pub struct WarmupRequest {
    /// Account email
    pub email: String,
    /// Model name (original name, no mapping)
    pub model: String,
    /// Optional: Provide Access Token directly (for accounts not in TokenManager)
    pub access_token: Option<String>,
    /// Optional: Provide the Project ID directly
    pub project_id: Option<String>,
}

/// warm-up response
#[derive(Debug, Serialize)]
pub struct WarmupResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Handle warm-up requests
pub async fn handle_warmup(
    State(state): State<AppState>,
    Json(req): Json<WarmupRequest>,
) -> Response {
    let start_time = std::time::Instant::now();

    info!(
        "[Warmup-API] ========== START: email={}, model={} ==========",
        req.email, req.model
    );

    // ===== Step 1: Obtain Token =====
    let (access_token, project_id, account_id) =
        if let (Some(at), Some(pid)) = (&req.access_token, &req.project_id) {
            (at.clone(), pid.clone(), String::new())
        } else {
            match state.token_manager.get_token_by_email(&req.email).await {
                Ok((at, pid, _, acc_id, _wait_ms)) => (at, pid, acc_id),
                Err(e) => {
                    warn!(
                        "[Warmup-API] Step 1 FAILED: Token error for {}: {}",
                        req.email, e
                    );
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(WarmupResponse {
                            success: false,
                            message: format!("Failed to get token for {}", req.email),
                            error: Some(e),
                        }),
                    )
                        .into_response();
                }
            }
        };

    // ===== Step 2: Construct the request body according to the model type =====
    let is_claude = req.model.to_lowercase().contains("claude");
    let is_image = req.model.to_lowercase().contains("image");

    let body: Value = if is_claude {
        // Claude model: Transform using transform_claude_request_in
        let session_id = format!(
            "warmup_{}_{}",
            chrono::Utc::now().timestamp_millis(),
            &uuid::Uuid::new_v4().to_string()[..8]
        );
        let claude_request = crate::proxy::mappers::claude::models::ClaudeRequest {
            model: req.model.clone(),
            messages: vec![crate::proxy::mappers::claude::models::Message {
                role: "user".to_string(),
                content: crate::proxy::mappers::claude::models::MessageContent::String(
                    "ping".to_string(),
                ),
            }],
            max_tokens: Some(1),
            stream: false,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            tools: None,
            metadata: Some(crate::proxy::mappers::claude::models::Metadata {
                user_id: Some(session_id),
            }),
            thinking: None,
            output_config: None,
            size: None,
            quality: None,
        };

        match crate::proxy::mappers::claude::transform_claude_request_in(
            &claude_request,
            &project_id,
            false,
            None,
            "warmup",
            None, // [NEW] No token for warmup
        ) {
            Ok(transformed) => transformed,
            Err(e) => {
                warn!("[Warmup-API] Step 2 FAILED: Claude transform error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(WarmupResponse {
                        success: false,
                        message: format!("Transform error: {}", e),
                        error: Some(e),
                    }),
                )
                    .into_response();
            }
        }
    } else {
        // Gemini model: using wrap_request
        let session_id = format!(
            "warmup_{}_{}",
            chrono::Utc::now().timestamp_millis(),
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        let base_request = if is_image {
            json!({
                "model": req.model,
                "contents": [{"role": "user", "parts": [{"text": "Say hi"}]}],
                "generationConfig": {
                    "maxOutputTokens": 10,
                    "temperature": 0,
                    "responseModalities": ["TEXT"]
                },
                "session_id": session_id
            })
        } else {
            json!({
                "model": req.model,
                "contents": [{"role": "user", "parts": [{"text": "Say hi"}]}],
                "generationConfig": {
                    "temperature": 0
                },
                "session_id": session_id
            })
        };

        wrap_request(
            &base_request,
            &project_id,
            &req.model,
            None,
            Some(&session_id),
            None,
        ) // [FIX] Added None for token param
    };

    // ===== Step 3: Call UpstreamClient =====
    let model_lower = req.model.to_lowercase();
    let prefer_non_stream = model_lower.contains("flash-lite") || model_lower.contains("2.5-pro");

    let (method, query) = if prefer_non_stream {
        ("generateContent", None)
    } else {
        ("streamGenerateContent", Some("alt=sse"))
    };

    let mut result = state
        .upstream
        .call_v1_internal(
            method,
            &access_token,
            body.clone(),
            query,
            Some(account_id.as_str()),
        )
        .await;

    // If streaming request fails, try non-streaming request
    if result.is_err() && !prefer_non_stream {
        result = state
            .upstream
            .call_v1_internal(
                "generateContent",
                &access_token,
                body,
                None,
                Some(account_id.as_str()),
            )
            .await;
    }

    let duration = start_time.elapsed().as_millis() as u64;

    // ===== Step 4: Process response and log traffic =====
    match result {
        Ok(call_result) => {
            let response = call_result.response;
            let status = response.status();
            let status_code = status.as_u16();

            // Record warm-up requests to traffic logs
            let log = ProxyRequestLog {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                method: "POST".to_string(),
                url: format!("/internal/warmup -> {}", req.model),
                status: status_code,
                duration,
                model: Some(req.model.clone()),
                mapped_model: Some(req.model.clone()),
                account_email: Some(req.email.clone()),
                client_ip: Some("127.0.0.1".to_string()),
                error: if status.is_success() {
                    None
                } else {
                    Some(format!("HTTP {}", status_code))
                },
                request_body: Some(format!(
                    "{{\"type\": \"warmup\", \"model\": \"{}\"}}",
                    req.model
                )),
                upstream_request_body: None,
                response_body: None,
                input_tokens: Some(0),
                output_tokens: Some(0),
                cached_tokens: None,
                protocol: Some("warmup".to_string()),
                username: None,
                request_headers: None,
                upstream_request_headers: None,
                response_headers: None,
                session_id: None,
            };
            state.monitor.log_request(log).await;

            let mut response = if status.is_success() {
                info!(
                    "[Warmup-API] ========== SUCCESS: {} / {} ({}ms) ==========",
                    req.email, req.model, duration
                );
                (
                    StatusCode::OK,
                    Json(WarmupResponse {
                        success: true,
                        message: format!("Warmup triggered for {}", req.model),
                        error: None,
                    }),
                )
                    .into_response()
            } else {
                let error_text = response.text().await.unwrap_or_default();

                // [FIX] When 403 is detected during the warm-up phase, mark the account as forbidden to prevent invalid accounts from continuing to participate in polling
                // If account_id is empty (scenario where access_token is passed in directly), find the ID from the index through email
                if status_code == 403 {
                    let resolved_account_id = if !account_id.is_empty() {
                        account_id.clone()
                    } else {
                        // Try to find the account ID via email
                        crate::modules::account::find_account_id_by_email(&req.email)
                            .unwrap_or_default()
                    };

                    if !resolved_account_id.is_empty() {
                        warn!(
                            "[Warmup-API] 403 Forbidden detected for {}, marking account as forbidden",
                            req.email
                        );
                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                        crate::error::record_ignored(
                            crate::modules::account::mark_account_forbidden(
                                &resolved_account_id,
                                &error_text,
                            ),
                            "mark_account_forbidden",
                        );
                    } else {
                        warn!(
                            "[Warmup-API] 403 Forbidden detected for {} but could not resolve account_id, skipping mark",
                            req.email
                        );
                    }
                }

                (
                    StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(WarmupResponse {
                        success: false,
                        message: format!("Warmup failed: HTTP {}", status_code),
                        error: Some(error_text),
                    }),
                )
                    .into_response()
            };

            // Add a response header to let the monitoring middleware capture account information
            if let Ok(email_val) = axum::http::HeaderValue::from_str(&req.email) {
                response.headers_mut().insert("X-Account-Email", email_val);
            }
            if let Ok(model_val) = axum::http::HeaderValue::from_str(&req.model) {
                response.headers_mut().insert("X-Mapped-Model", model_val);
            }

            response
        }
        Err(e) => {
            warn!(
                "[Warmup-API] ========== ERROR: {} / {} - {} ({}ms) ==========",
                req.email, req.model, e, duration
            );

            // Record failed warm-up requests to the traffic log
            let log = ProxyRequestLog {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                method: "POST".to_string(),
                url: format!("/internal/warmup -> {}", req.model),
                status: 500,
                duration,
                model: Some(req.model.clone()),
                mapped_model: Some(req.model.clone()),
                account_email: Some(req.email.clone()),
                client_ip: Some("127.0.0.1".to_string()),
                error: Some(e.clone()),
                request_body: Some(format!(
                    "{{\"type\": \"warmup\", \"model\": \"{}\"}}",
                    req.model
                )),
                upstream_request_body: None,
                response_body: Some(e.clone()),
                input_tokens: None,
                output_tokens: None,
                cached_tokens: None,
                protocol: Some("warmup".to_string()),
                username: None,
                request_headers: None,
                upstream_request_headers: None,
                response_headers: None,
                session_id: None,
            };
            state.monitor.log_request(log).await;

            let mut response = (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WarmupResponse {
                    success: false,
                    message: "Warmup request failed".to_string(),
                    error: Some(e),
                }),
            )
                .into_response();

            // Add response headers even on failure for easy monitoring
            if let Ok(email_val) = axum::http::HeaderValue::from_str(&req.email) {
                response.headers_mut().insert("X-Account-Email", email_val);
            }
            if let Ok(model_val) = axum::http::HeaderValue::from_str(&req.model) {
                response.headers_mut().insert("X-Mapped-Model", model_val);
            }

            response
        }
    }
}
