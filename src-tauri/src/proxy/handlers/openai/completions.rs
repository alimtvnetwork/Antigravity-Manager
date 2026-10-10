// OpenAI `/v1/completions` + `/v1/responses` handler (skeleton; prelude
// phases and attempt phases live in siblings).
use axum::http::HeaderMap;
use axum::{
    extract::Json, extract::State, http::StatusCode, response::IntoResponse, response::Response,
};
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    next_rotation_attempt, FailureStatusTracker, RequestRetryState,
};
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::server::AppState;

use super::completions_error::completions_handle_error;
use super::completions_prep_codex::completions_prep_codex;
use super::completions_prep_convert::completions_convert_payload;
use super::completions_prep_session::{completions_prep_session, CompletionsSessionPrep};
use super::completions_prep_setup::{completions_prep_setup, CompletionsSetup};
use super::completions_send::{completions_send, CompletionsSendOutcome};
use super::completions_success::completions_handle_success;
use super::responses_history::{debug_value_without_inline_data, serialized_json_len};

pub(crate) enum CompletionsOutcome {
    Respond(Response),
    ContinueLoop,
}

fn responses_store_enabled(body: &Value) -> bool {
    body.get("store").and_then(Value::as_bool) != Some(false)
}

pub async fn handle_completions(
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
    State(state): State<AppState>,
    headers: HeaderMap,
    upstream_recorder: Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
    Json(mut body): Json<Value>,
) -> Response {
    let clean_start = std::time::Instant::now();
    debug!(
        "Received /v1/completions or /v1/responses payload: {} bytes",
        serialized_json_len(&body)
    );
    let debug_cfg = state.debug_logging.read().await.clone();
    let original_body = debug_logger::is_enabled(&debug_cfg).then(|| {
        crate::proxy::payload_audit::reorder_payload_fields(&debug_value_without_inline_data(&body))
    });
    let is_responses_api = uri.path() == "/v1/responses";
    let is_codex_style = body.get("input").is_some() || body.get("instructions").is_some();
    let store_response = responses_store_enabled(&body);

    let sess = match completions_prep_session(&headers, body, is_codex_style, store_response).await
    {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let sess = completions_convert_payload(sess, is_codex_style, store_response);
    let codex = match completions_prep_codex(sess, is_codex_style, store_response) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let mut setup = match completions_prep_setup(
        &state,
        &headers,
        codex,
        is_responses_api,
        &debug_cfg,
        clean_start,
        &original_body,
    )
    .await
    {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    while let Some(attempt) = next_rotation_attempt(
        &mut setup.used_attempts,
        setup.max_attempts,
        setup.retry_credentials.is_some(),
    ) {
        let send = match completions_send(
            setup.upstream.clone(),
            setup.token_manager.clone(),
            &setup.openai_req,
            setup.mapped_model.clone(),
            setup.session_id_str.clone(),
            &upstream_recorder,
            attempt,
            debug_cfg.clone(),
            setup.trace_id.clone(),
            setup.client_session_id.clone(),
            is_responses_api,
            setup.signature_read_key.clone(),
            setup.max_attempts,
            &mut setup.force_rotate,
            &mut setup.last_error,
            &mut setup.last_email,
            &mut setup.retry_credentials,
            &mut setup.failure_statuses,
            &mut setup.norm_ms,
            &mut setup.think_fill_ms,
        )
        .await
        {
            CompletionsSendOutcome::Respond(r) => return r,
            CompletionsSendOutcome::ContinueLoop => continue,
            CompletionsSendOutcome::Sent(s) => s,
        };
        if send.status.is_success() {
            match completions_handle_success(
                send,
                &setup.openai_req,
                debug_cfg.clone(),
                setup.trace_id.clone(),
                original_body.clone(),
                setup.client_tool_names.clone(),
                &uri,
                &setup.response_id_for_save,
                setup.routing_session_id.clone(),
                &setup.session_id_str,
                setup.session_scope.clone(),
                &setup.signature_session_id_str,
                store_response,
                is_codex_style,
                setup.assistant_turn_index,
                &mut setup.session_parent,
                &mut setup.session_save_input,
                &mut setup.session_save_instructions,
                setup.clean_ms,
                &mut setup.norm_ms,
                &mut setup.think_fill_ms,
                &mut setup.ttft_ms,
                &mut setup.failure_statuses,
                &mut setup.last_error,
                attempt,
            )
            .await
            {
                CompletionsOutcome::Respond(r) => return r,
                CompletionsOutcome::ContinueLoop => continue,
            }
        } else {
            match completions_handle_error(
                send,
                &setup.openai_req,
                setup.trace_id.clone(),
                setup.session_id_str.clone(),
                is_responses_api,
                attempt,
                setup.max_attempts,
                setup.pool_size,
                setup.token_manager.clone(),
                &mut setup.failure_statuses,
                &mut setup.force_rotate,
                &mut setup.last_error,
                &mut setup.retry_credentials,
                &mut setup.retry_state,
            )
            .await
            {
                CompletionsOutcome::Respond(r) => return r,
                CompletionsOutcome::ContinueLoop => continue,
            }
        }
    }
    let final_status = failure_statuses.final_status();
    let headers = crate::proxy::handlers::common::build_token_error_headers(
        Some(mapped_model.as_str()),
        last_email.as_deref(),
        &last_error,
    );
    let protocol = if is_responses_api {
        "responses"
    } else {
        "openai"
    };
    let dual_err = crate::proxy::handlers::common::build_dual_track_error(
        protocol,
        final_status.as_u16(),
        &mapped_model,
        &last_error,
    );
    (final_status, headers, axum::Json(dual_err)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn responses_store_defaults_to_enabled_and_honors_explicit_false() {
        assert!(super::responses_store_enabled(&json!({})));
        assert!(super::responses_store_enabled(&json!({"store": true})));
        assert!(super::responses_store_enabled(&json!({"store": null})));
        assert!(!super::responses_store_enabled(&json!({"store": false})));
    }
}
