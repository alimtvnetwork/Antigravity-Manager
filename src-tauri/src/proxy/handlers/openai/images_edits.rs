// `POST /v1/images/edits`.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::Json, extract::State, http::StatusCode, response::IntoResponse, response::Response,
};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::monitor::UpstreamRequestBodyHolder;
use crate::proxy::server::AppState;
use crate::proxy::session_manager::SessionManager;
use crate::proxy::TokenManager;

use super::image_input::{build_image_edit_body, validate_input_image_limits};
use super::images_edits_dispatch::spawn_image_edit_tasks;

pub async fn handle_images_edits(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("[Images] Received edit request");

    let mut input_images: Vec<NormalizedInputImage> = Vec::new();
    let mut mask_data: Option<NormalizedInputImage> = None;
    let mut total_input_image_bytes = 0;
    let mut prompt = String::new();
    let mut n = 1;
    let mut size: Option<String> = None;
    let mut response_format = "b64_json".to_string();
    let mut model = "gemini-3.1-flash-image".to_string();
    let mut aspect_ratio: Option<String> = None;
    let mut image_size_param: Option<String> = None;
    let mut quality: Option<String> = None;
    let mut style: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if is_edit_image_field(&name) {
            let mime_type = field
                .content_type()
                .map(|content_type| content_type.to_string())
                .unwrap_or_else(|| "image/png".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Image read error: {}", e)))?;
            let image = normalized_image_from_bytes(
                &data,
                &mime_type,
                input_images.len() + 1,
                total_input_image_bytes,
            )
            .map_err(|message| (StatusCode::BAD_REQUEST, message))?;
            total_input_image_bytes = total_input_image_bytes.saturating_add(image.decoded_len);
            input_images.push(image);
        } else if name == "mask" {
            let mime_type = field
                .content_type()
                .map(|content_type| content_type.to_string())
                .unwrap_or_else(|| "image/png".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Mask read error: {}", e)))?;
            let mask = normalized_image_from_bytes(
                &data,
                &mime_type,
                input_images.len(),
                total_input_image_bytes,
            )
            .map_err(|message| (StatusCode::BAD_REQUEST, message))?;
            total_input_image_bytes = total_input_image_bytes.saturating_add(mask.decoded_len);
            mask_data = Some(mask);
        } else if name == "prompt" {
            prompt = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Prompt read error: {}", e)))?;
        } else if name == "n" {
            if let Ok(val) = field.text().await {
                n = val.parse().unwrap_or(1);
            }
        } else if name == "size" {
            let val = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Size read error: {}", e)))?;
            size = Some(val);
        } else if name == "image_size" || name == "imageSize" {
            if let Ok(val) = field.text().await {
                image_size_param = Some(val);
            }
        } else if name == "quality" {
            if let Ok(val) = field.text().await {
                quality = Some(val);
            }
        } else if name == "aspect_ratio" {
            if let Ok(val) = field.text().await {
                aspect_ratio = Some(val);
            }
        } else if name == "style" {
            if let Ok(val) = field.text().await {
                style = Some(val);
            }
        } else if name == "response_format" {
            if let Ok(val) = field.text().await {
                response_format = val;
            }
        } else if name == "model" {
            if let Ok(val) = field.text().await {
                if !val.is_empty() {
                    model = val;
                }
            }
        }
    }

    // Validation: Require either 'image' (standard edit) OR 'prompt' (generation)
    // If reference images are present, we treat it as generation with image context
    if prompt.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing prompt".to_string()));
    }

    tracing::info!(
        model = model,
        n = n,
        size = size.as_deref().unwrap_or("auto"),
        aspect_ratio = aspect_ratio.as_deref().unwrap_or("auto"),
        image_size = image_size_param.as_deref().unwrap_or("auto"),
        quality = quality.as_deref().unwrap_or("auto"),
        style = style.as_deref().unwrap_or("auto"),
        image_count = input_images.len(),
        has_mask = mask_data.is_some(),
        "[Images] Received edit request metadata"
    );

    // 2. Prepare Config (Aspect Ratio / Size)
    // Priority: aspect_ratio param > size param
    // Priority: image_size param > quality param (derived from model suffix or default)

    // We reuse parse_image_config_with_params but need to adapt the inputs
    let size_input = edit_size_input(aspect_ratio.as_deref(), size.as_deref());

    let (image_config, clean_model_name) =
        crate::proxy::mappers::common_utils::try_parse_image_config_with_params(
            &model,
            size_input,
            quality.as_deref(),
            image_size_param.as_deref(),
        )
        .map_err(|message| (StatusCode::BAD_REQUEST, message))?;

    // 3. Construct Contents
    let mut final_prompt = prompt.clone();
    if let Some(s) = style {
        final_prompt.push_str(&format!(", style: {}", s));
    }
    let contents_parts = build_image_contents(final_prompt, &input_images, mask_data.as_ref());

    // Prepare task spawning context (reconstructed after module split)
    let mut tasks: tokio::task::JoinSet<
        Result<(serde_json::Value, String, String), (axum::http::StatusCode, String)>,
    > = tokio::task::JoinSet::new();
    let upstream = state.upstream_client.clone();
    let token_manager = state.token_manager.clone();
    let client_adapter: Option<
        std::sync::Arc<dyn crate::proxy::common::client_adapter::ClientAdapter>,
    > = None;
    let openai_req = serde_json::json!({"model": model, "prompt": prompt});
    let selected: Vec<(usize, String, String, String, String, u64)> = Vec::new();
    let extra_headers = axum::http::HeaderMap::new();
    let debug_cfg = state.debug_logging.clone();
    let debug_cfg_read = debug_cfg.read().unwrap();
    let trace_id = uuid::Uuid::new_v4().to_string();
    let attempt_no: usize = 0;

    spawn_image_edit_tasks(
        &mut tasks,
        &upstream,
        &token_manager,
        &client_adapter,
        &openai_req,
        &input_images,
        &selected,
        &extra_headers,
        &debug_cfg_read,
        &trace_id,
        attempt_no,
        &state,
        n as usize,
        contents_parts,
        image_config,
        response_format,
        clean_model_name,
    );

    // 5. Collect Results
    let mut images: Vec<Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut used_email: Option<String> = None;
    let mut failure_statuses = FailureStatusTracker::default();

    while let Some(task) = tasks.join_next().await {
        match task {
            Ok(result) => match result {
                Ok((gemini_resp, response_format, email_used)) => {
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
        tracing::error!(
            "[Images] All {} edit requests failed. Errors: {}",
            n,
            error_msg
        );
        let status = failure_statuses.final_status();

        return Err((status, error_msg));
    }

    if !errors.is_empty() {
        tracing::warn!(
            "[Images] Partial success: {} out of {} requests succeeded. Errors: {}",
            images.len(),
            n,
            errors.join("; ")
        );
    }

    tracing::info!(
        "[Images] Successfully generated {} out of {} requested edited image(s)",
        images.len(),
        n
    );

    let openai_response = json!({
        "created": chrono::Utc::now().timestamp(),
        "data": images
    });

    tokio::spawn(async move {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::account::refresh::refresh_all_quotas_logic().await,
            "refresh_all_quotas_logic",
        );
    });

    let email_header = used_email.unwrap_or_default();
    Ok((
        StatusCode::OK,
        [
            ("X-Mapped-Model", clean_model_name.as_str()),
            ("X-Account-Email", email_header.as_str()),
        ],
        Json(openai_response),
    )
        .into_response())
}

// ==========================================
// CODE INTEGRATION: Codex WebSocket Handler
// ==========================================

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

// ==========================================

// CODE INTEGRATION: Global Tool Call Cache

// ==========================================

use std::sync::OnceLock;

use crate::proxy::handlers::openai::image_input::build_image_contents;
use crate::proxy::handlers::openai::image_input::edit_size_input;
use crate::proxy::handlers::openai::image_input::is_edit_image_field;
use crate::proxy::handlers::openai::image_input::normalized_image_from_bytes;
use crate::proxy::handlers::openai::image_input::NormalizedInputImage;
use tokio::sync::RwLock as TokioRwLock;

pub(crate) static WEBSOCKET_TOOL_CALL_CACHE: OnceLock<TokioRwLock<HashMap<String, Value>>> =
    OnceLock::new();
