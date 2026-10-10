// `handle_completions` success-path dispatcher: streaming arm vs
// non-streaming arm. The streaming arm always diverges, so the trailing
// non-streaming arm is only reached when `list_response` is false.
use std::collections::HashSet;
use std::sync::Arc;
use crate::proxy::TokenManager;

use axum::extract::OriginalUri;
use serde_json::Value;

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::http_session_store::SessionParent;
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::thinking_store::SessionScope;

use super::completions::CompletionsOutcome;
use super::completions_send::CompletionsSendOutput;
use super::completions_success_nonstream::completions_success_nonstream;
use super::completions_success_stream::{completions_success_stream, CompletionsStreamOutcome};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn completions_handle_success(
    send: CompletionsSendOutput,
    openai_req: &OpenAIRequest,
    debug_cfg: DebugLoggingConfig,
    trace_id: String,
    original_body: Option<Value>,
    client_tool_names: HashSet<String>,
    uri: &OriginalUri,
    response_id_for_save: &String,
    routing_session_id: String,
    session_id_str: &String,
    session_scope: SessionScope,
    signature_session_id_str: &String,
    store_response: bool,
    is_codex_style: bool,
    assistant_turn_index: usize,
    session_parent: &mut Option<SessionParent>,
    session_save_input: &mut Vec<Value>,
    session_save_instructions: &mut String,
    clean_ms: f64,
    norm_ms: &mut f64,
    think_fill_ms: &mut f64,
    ttft_ms: &mut f64,
    failure_statuses: &mut FailureStatusTracker,
    last_error: &mut String,
    attempt: usize,
    token_manager: Arc<TokenManager>,
) -> CompletionsOutcome {
    let CompletionsSendOutput {
        response,
        status,
        upstream_url,
        session_id,
        message_count,
        config,
        mapped_model,
        email,
        account_id: _,
        access_token: _,
        project_id: _,
        client_wants_stream,
        list_response,
        causal_anchor,
        upstream_req_start,
        gemini_body_for_debug,
    } = send;
    // [智能限流] 请求成功，重置该账号的连续失败计数
    token_manager.mark_account_success(&email);

    if list_response {
        return match completions_success_stream(
            response,
            mapped_model,
            email,
            upstream_req_start,
            session_id,
            causal_anchor,
            upstream_url,
            message_count,
            status,
            &config,
            client_wants_stream,
            openai_req,
            debug_cfg,
            trace_id,
            original_body,
            client_tool_names,
            uri,
            response_id_for_save,
            routing_session_id,
            session_id_str,
            session_scope,
            store_response,
            is_codex_style,
            assistant_turn_index,
            session_parent,
            session_save_input,
            session_save_instructions,
            clean_ms,
            norm_ms,
            think_fill_ms,
            ttft_ms,
            failure_statuses,
            last_error,
            &gemini_body_for_debug,
            attempt,
        )
        .await
        {
            CompletionsStreamOutcome::Respond(r) => CompletionsOutcome::Respond(r),
            CompletionsStreamOutcome::ContinueLoop => CompletionsOutcome::ContinueLoop,
        };
    }
    completions_success_nonstream(
        response,
        mapped_model,
        email,
        upstream_req_start,
        openai_req,
        debug_cfg,
        trace_id,
        original_body,
        client_tool_names,
        uri,
        session_id_str,
        session_scope,
        signature_session_id_str,
        clean_ms,
        norm_ms,
        think_fill_ms,
        ttft_ms,
        &gemini_body_for_debug,
    )
    .await
}
