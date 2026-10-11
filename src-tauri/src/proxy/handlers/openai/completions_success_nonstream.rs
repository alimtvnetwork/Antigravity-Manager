// `handle_completions` success path, non-streaming arm.
use std::collections::HashSet;

use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::mappers::openai::{transform_openai_response, OpenAIRequest};
use crate::proxy::thinking_store::SessionScope;

use super::completions::CompletionsOutcome;
use crate::proxy::handlers::openai::chat_conversion::convert_chat_response_to_responses;
use axum::body::Body;
use axum::response::IntoResponse;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn completions_success_nonstream(
    response: rquest::Response,
    mapped_model: String,
    email: String,
    upstream_req_start: std::time::Instant,
    openai_req: &OpenAIRequest,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    original_body: Option<Value>,
    client_tool_names: HashSet<String>,
    uri: &OriginalUri,
    session_id_str: &String,
    session_scope: SessionScope,
    signature_session_id_str: &String,
    clean_ms: f64,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
    ttft_ms: &mut f64,
    gemini_body_for_debug: &Option<Value>,
) -> CompletionsOutcome {
    *ttft_ms = upstream_req_start.elapsed().as_micros() as f64 / 1000.0;
    let gemini_resp: Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            return CompletionsOutcome::Respond(
                (
                    StatusCode::BAD_GATEWAY,
                    [("X-Mapped-Model", mapped_model.as_str())],
                    format!("Parse error: {}", e),
                )
                    .into_response(),
            );
        }
    };

    crate::proxy::thinking_store::capture_gemini_response(&session_id_str, &gemini_resp);

    let chat_resp = transform_openai_response(
        &gemini_resp,
        Some(&signature_session_id_str),
        1,
        Some(&client_tool_names),
    );

    let is_responses_api = uri.path() == "/v1/responses";

    if is_responses_api {
        let resp = convert_chat_response_to_responses(&chat_resp);
        if debug_logger::is_enabled(&debug_cfg) {
            let payload = json!({
                "kind": "exchange_summary",
                "protocol": "openai",
                "trace_id": trace_id,
                "request_path": uri.path(),
                "original_codex_request": original_body.as_ref(),
                "gemini_request": gemini_body_for_debug.as_ref(),
                "gemini_raw_response": gemini_resp.clone(),
                "converted_codex_response": resp.clone(),
            });
            debug_logger::write_exchange_payload(
                &debug_cfg,
                Some(&trace_id),
                "exchange_summary",
                &payload,
            )
            .await;
        }

        return CompletionsOutcome::Respond(
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("X-Account-Email", email.as_str())
                .header("X-Mapped-Model", mapped_model.as_str())
                .header("X-Session-Id", session_scope.client_id.as_str())
                .header("X-Antigravity-Session-Id", session_scope.client_id.as_str())
                .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
                .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
                .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
                .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
                .body(Body::from(serde_json::to_string(&resp).unwrap_or_default()))
                .unwrap()
                .into_response(),
        );
    }

    // Map Chat Response -> Legacy Completions Response
    let choices = chat_resp
        .choices
        .iter()
        .map(|c| {
            json!({
                "text": match &c.message.content {
                    Some(crate::proxy::mappers::openai::OpenAIContent::String(s)) => s.clone(),
                    _ => "".to_string()
                },
                "index": c.index,
                "logprobs": null,
                "finish_reason": c.finish_reason
            })
        })
        .collect::<Vec<_>>();

    let legacy_resp = json!({
        "id": chat_resp.id,
        "object": "text_completion",
        "created": chat_resp.created,
        "model": chat_resp.model,
        "choices": choices,
        "usage": chat_resp.usage
    });
    if debug_logger::is_enabled(&debug_cfg) {
        let payload = json!({
            "kind": "exchange_summary",
            "protocol": "openai",
            "trace_id": trace_id,
            "request_path": uri.path(),
            "original_codex_request": original_body.as_ref(),
            "gemini_request": gemini_body_for_debug.as_ref(),
            "gemini_raw_response": gemini_resp.clone(),
            "converted_codex_response": legacy_resp.clone(),
        });
        debug_logger::write_exchange_payload(
            &debug_cfg,
            Some(&trace_id),
            "exchange_summary",
            &payload,
        )
        .await;
    }

    return CompletionsOutcome::Respond(
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("X-Account-Email", email.as_str())
            .header("X-Mapped-Model", mapped_model.as_str())
            .header("X-Session-Id", session_scope.client_id.as_str())
            .header("X-Antigravity-Session-Id", session_scope.client_id.as_str())
            .header("X-Timing-Clean-Ms", format!("{:.3}", clean_ms))
            .header("X-Timing-Norm-Ms", format!("{:.3}", norm_ms))
            .header("X-Timing-Thinking-Ms", format!("{:.3}", think_fill_ms))
            .header("X-Timing-Ttft-Ms", format!("{:.3}", ttft_ms))
            .body(Body::from(
                serde_json::to_string(&legacy_resp).unwrap_or_default(),
            ))
            .unwrap()
            .into_response(),
    );
}
