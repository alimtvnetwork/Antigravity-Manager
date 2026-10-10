// `handle_completions` prelude phase 4: context management, session scope,
// model routing, retry-state setup.
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    calculate_max_retry_attempts, FailureStatusTracker, RequestRetryState,
};
use crate::proxy::http_session_store::SessionParent;
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::server::{AppState, UpstreamClient};
use crate::proxy::thinking_store::SessionScope;
use crate::proxy::TokenManager;

use super::completions_prep_codex::CompletionsCodexPrep;
use super::websocket_compress::try_compress_openai_with_summary;
use crate::proxy::session_manager::SessionManager;

/// Everything the retry loop needs, prepared before the first attempt.
pub(crate) struct CompletionsSetup {
    pub openai_req: OpenAIRequest,
    pub mapped_model: String,
    pub trace_id: String,
    pub session_scope: SessionScope,
    pub session_id_str: String,
    pub client_session_id: String,
    pub signature_session_id_str: String,
    pub signature_read_key: Option<String>,
    pub client_tool_names: HashSet<String>,
    pub token_manager: Arc<TokenManager>,
    pub upstream: Arc<UpstreamClient>,
    pub pool_size: usize,
    pub max_attempts: usize,
    pub last_error: String,
    pub last_email: Option<String>,
    pub retry_state: RequestRetryState,
    pub retry_credentials: Option<(String, String, String, String, u64)>,
    pub failure_statuses: FailureStatusTracker,
    pub used_attempts: usize,
    pub force_rotate: bool,
    pub clean_ms: f64,
    pub norm_ms: f64,
    pub think_fill_ms: f64,
    pub ttft_ms: f64,
    pub previous_response_id: Option<String>,
    pub explicit_session_id: Option<String>,
    pub response_id_for_save: String,
    pub session_parent: Option<SessionParent>,
    pub routing_session_id: String,
    pub session_save_input: Vec<Value>,
    pub session_save_instructions: String,
    pub assistant_turn_index: usize,
}

pub(crate) async fn completions_prep_setup(
    state: &AppState,
    headers: &HeaderMap,
    codex: CompletionsCodexPrep,
    is_responses_api: bool,
    debug_cfg: &DebugLoggingConfig,
    clean_start: Instant,
    original_body: &Option<Value>,
) -> Result<CompletionsSetup, Response> {
    let CompletionsCodexPrep {
        previous_response_id,
        explicit_session_id,
        response_id_for_save,
        session_parent,
        routing_session_id,
        signature_read_key,
        normalized_interaction_ledger,
        session_save_input,
        session_save_instructions,
        openai_req,
    } = codex;
    let mut openai_req = openai_req;
    // [NEW v4.2.0] Context Management & Reasoning Replay
    let explicit_sid = headers
        .get("x-session-id")
        .or_else(|| headers.get("x-jeikcode-session-id"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let fallback_sid = if let Some(sid) = explicit_sid {
        sid.to_string()
    } else if is_responses_api {
        if explicit_session_id.is_some() || previous_response_id.is_some() {
            routing_session_id.clone()
        } else {
            SessionManager::extract_openai_session_id(&openai_req)
        }
    } else {
        SessionManager::extract_openai_session_id(&openai_req)
    };
    let session_scope = crate::proxy::thinking_store::SessionScope::from_headers_and_body(
        &headers,
        original_body.as_ref(),
        fallback_sid,
    );
    openai_req.session_id = Some(session_scope.store_key.clone());
    let session_id_str = session_scope.store_key.clone();
    let client_session_id = session_scope.client_id.clone();
    let signature_session_id_str = if is_responses_api {
        previous_response_id
            .clone()
            .unwrap_or_else(|| response_id_for_save.clone())
    } else {
        session_id_str.clone()
    };

    let client_tool_names =
        crate::proxy::mappers::openai::request::extract_client_tool_names(&openai_req.tools);

    // Server-authoritative thinking: do NOT prefill messages.reasoning_content from
    // SignatureCache. OpenAI mapping ignores client/cached reasoning text and fills
    // placeholders via ThinkingStore hydrate + finalize instead.

    let experimental_cfg = state.experimental.read().await;
    let compression_level = if experimental_cfg.compression_level == "disabled" {
        if experimental_cfg.enable_usage_scaling {
            "high".to_string()
        } else {
            "disabled".to_string()
        }
    } else {
        experimental_cfg.compression_level.clone()
    };

    let mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        &openai_req.model,
        &*state.custom_mapping.read().await,
    );
    let trace_id = format!("req_{}", chrono::Utc::now().timestamp_subsec_millis());
    if debug_logger::is_enabled(&debug_cfg) {
        if let Some(ledger) = normalized_interaction_ledger {
            let payload = json!({
                "kind": "normalized_interaction_ledger",
                "protocol": "openai",
                "trace_id": trace_id.clone(),
                "request_path": uri.path(),
                "original_model": openai_req.model.clone(),
                "interaction_ledger": ledger,
            });
            debug_logger::write_exchange_payload(
                &debug_cfg,
                Some(&trace_id),
                "normalized_interaction_ledger",
                &payload,
            )
            .await;
        }
    }
    let token_manager = state.token_manager.clone();

    let mut compression_applied = false;
    let mut is_purified = false;

    if compression_level == "high" {
        let context_limit = if mapped_model.contains("flash") {
            1_000_000
        } else {
            2_000_000
        };

        let raw_estimated =
            crate::proxy::mappers::context_manager::ContextManager::estimate_openai_token_usage(
                &openai_req,
            );
        let calibrator = crate::proxy::mappers::estimation_calibrator::get_calibrator();
        let mut estimated_usage = calibrator.calibrate(raw_estimated);
        let mut usage_ratio = estimated_usage as f32 / context_limit as f32;

        let threshold_l1 = experimental_cfg.context_compression_threshold_l1;
        let threshold_l3 = experimental_cfg.context_compression_threshold_l3;

        tracing::info!(
            "[{}] [ContextManager] [OpenAI] Context pressure: {:.1}% (raw: {}, calibrated: {} / {}), Calibration factor: {:.2}",
            trace_id, usage_ratio * 100.0, raw_estimated, estimated_usage, context_limit, calibrator.get_factor()
        );

        // ===== Layer 1: Tool Message Trimming =====
        if usage_ratio > threshold_l1 && !compression_applied {
            if crate::proxy::mappers::context_manager::ContextManager::trim_openai_tool_messages(
                &mut openai_req.messages,
                5,
            ) {
                tracing::info!(
                    "[{}] [Layer-1] [OpenAI] Tool trimming triggered (usage: {:.1}%, threshold: {:.1}%)",
                    trace_id, usage_ratio * 100.0, threshold_l1 * 100.0
                );
                compression_applied = true;

                let new_raw = crate::proxy::mappers::context_manager::ContextManager::estimate_openai_token_usage(&openai_req);
                let new_usage = calibrator.calibrate(new_raw);
                let new_ratio = new_usage as f32 / context_limit as f32;

                tracing::info!(
                    "[{}] [Layer-1] [OpenAI] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                    trace_id,
                    usage_ratio * 100.0,
                    new_ratio * 100.0,
                    estimated_usage - new_usage
                );

                if new_ratio < 0.7 {
                    estimated_usage = new_usage;
                    usage_ratio = new_ratio;
                } else {
                    usage_ratio = new_ratio;
                    compression_applied = false;
                }
            }
        }

        // ===== Layer 3: Fork Conversation + XML Summary =====
        if usage_ratio > threshold_l3 && !compression_applied {
            tracing::info!(
                "[{}] [Layer-3] [OpenAI] Context pressure ({:.1}%) exceeded threshold ({:.1}%), attempting Fork+Summary",
                trace_id, usage_ratio * 100.0, threshold_l3 * 100.0
            );

            let token_manager_clone = token_manager.clone();

            match try_compress_openai_with_summary(
                &openai_req,
                &trace_id,
                &token_manager_clone,
                &state.upstream,
                &signature_session_id_str,
            )
            .await
            {
                Ok(forked_req) => {
                    tracing::info!(
                        "[{}] [Layer-3] [OpenAI] Fork successful: {} → {} messages",
                        trace_id,
                        openai_req.messages.len(),
                        forked_req.messages.len()
                    );

                    openai_req = forked_req;
                    is_purified = false;

                    let new_raw = crate::proxy::mappers::context_manager::ContextManager::estimate_openai_token_usage(&openai_req);
                    let new_usage = calibrator.calibrate(new_raw);
                    let new_ratio = new_usage as f32 / context_limit as f32;

                    tracing::info!(
                        "[{}] [Layer-3] [OpenAI] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                        trace_id, usage_ratio * 100.0, new_ratio * 100.0, estimated_usage - new_usage
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "[{}] [Layer-3] [OpenAI] Fork+Summary failed: {}, falling back to error response",
                        trace_id, e
                    );
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("Context too long and automatic compression failed: {}", e),
                    )
                        .into_response());
                }
            }
        }
    } else if compression_level != "disabled" {
        if crate::proxy::mappers::context_manager::ContextManager::trim_openai_tool_messages(
            &mut openai_req.messages,
            5,
        ) {
            tracing::info!("[Codex-Context] Trimmed old tool messages to keep last 5 rounds");
        }

        if compression_level == "medium" {
            if crate::proxy::mappers::context_manager::ContextManager::purify_openai_history(
                &mut openai_req.messages,
                crate::proxy::mappers::context_manager::PurificationStrategy::Soft,
            ) {
                tracing::info!("[Codex-Context] Purified older assistant reasoning_content and natural language history");
            }
        }
    }

    let assistant_turn_index = openai_req
        .messages
        .iter()
        .filter(|m| m.role == "assistant")
        .count();

    let upstream = state.upstream.clone();
    let pool_size = token_manager.len();
    // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
    let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(pool_size);

    let mut last_error = String::new();
    let mut last_email: Option<String> = None;
    let mut retry_state = RequestRetryState::default();
    let mut retry_credentials: Option<(String, String, String, String, u64)> = None;
    let mut failure_statuses = FailureStatusTracker::default();
    let mut used_attempts = 0;

    let clean_ms = clean_start.elapsed().as_micros() as f64 / 1000.0;
    let mut norm_ms = 0.0f64;
    let mut think_fill_ms = 0.0f64;
    let mut ttft_ms = 0.0f64;

    if debug_logger::is_enabled(&debug_cfg) {
        let payload = json!({
            "kind": "original_request",
            "protocol": "openai",
            "trace_id": trace_id,
            "request_path": uri.path(),
            "request": original_body.as_ref(),
        });
        debug_logger::write_exchange_payload(
            &debug_cfg,
            Some(&trace_id),
            "original_request",
            &payload,
        )
        .await;
    }

    let mut force_rotate = false;

    Ok(CompletionsSetup {
        openai_req,
        mapped_model,
        trace_id,
        session_scope,
        session_id_str,
        client_session_id,
        signature_session_id_str,
        signature_read_key,
        client_tool_names,
        token_manager,
        upstream,
        pool_size,
        max_attempts,
        last_error,
        last_email,
        retry_state,
        retry_credentials,
        failure_statuses,
        used_attempts,
        force_rotate,
        clean_ms,
        norm_ms,
        think_fill_ms,
        ttft_ms,
        previous_response_id,
        explicit_session_id,
        response_id_for_save,
        session_parent,
        routing_session_id,
        session_save_input,
        session_save_instructions,
        assistant_turn_index,
    })
}
