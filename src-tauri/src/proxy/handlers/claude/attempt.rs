use super::*;
use crate::proxy::common::client_adapter::ClientAdapter;
use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::upstream::client::UpstreamClient;
use crate::proxy::TokenManager;
use std::sync::Arc;

/// Mutable per-request retry state shared by the extracted attempt-phase
/// functions. Fields map 1:1 to the locals previously held inline in
/// `handle_messages`. Pure code move: no logic changes.
pub(crate) struct AttemptState {
    // ---- request-scoped (immutable after setup) ----
    pub trace_id: String,
    pub debug_cfg: DebugLoggingConfig,
    pub client_adapter: Option<Arc<dyn ClientAdapter>>,
    pub request: crate::proxy::mappers::claude::models::ClaudeRequest,
    pub original_body: Value,
    pub compression_level: String,
    pub scaling_enabled: bool,
    pub threshold_l1: f32,
    pub threshold_l3: f32,
    pub clean_ms: f64,
    pub upstream: Arc<UpstreamClient>,
    pub token_manager: Arc<TokenManager>,
    pub headers: HeaderMap,
    pub upstream_recorder:
        Option<axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>>,
    pub custom_mapping: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    pub max_attempts: usize,
    pub pool_size: usize,
    // ---- mutable attempt state ----
    pub request_for_body: crate::proxy::mappers::claude::models::ClaudeRequest,
    pub last_error: String,
    pub retried_without_thinking: bool,
    pub last_email: Option<String>,
    pub last_mapped_model: Option<String>,
    pub last_status: StatusCode,
    pub force_rotate: bool,
    pub norm_ms: f64,
    pub think_fill_ms: f64,
    pub ttft_ms: f64,
}

/// Data handed from attempt preparation to the response handlers.
pub(crate) struct AttemptCall {
    pub response: rquest::Response,
    pub upstream_url: String,
    pub status: StatusCode,
    pub email: String,
    pub account_id: String,
    pub mapped_model: String,
    pub request_type: String,
    pub request_with_mapped: crate::proxy::mappers::claude::models::ClaudeRequest,
    pub session_id_str: String,
    pub client_session_id: String,
    pub raw_estimated: u32,
    pub client_wants_stream: bool,
    pub actual_stream: bool,
    pub upstream_req_start: std::time::Instant,
}

/// Control flow out of attempt preparation.
pub(crate) enum PrepOutcome {
    Respond(Response),
    Retry,
    Proceed(AttemptCall),
}

/// Control flow out of stream response handling.
pub(crate) enum StreamOutcome {
    Respond(Response),
    Retry,
}

/// Control flow out of upstream error handling.
pub(crate) enum ErrorOutcome {
    Respond(Response),
    Retry,
}

impl AttemptState {
    /// Build the initial retry state from the preprocessed request.
    /// Mirrors the pre-loop initialization previously inline in `handle_messages`.
    pub(crate) fn from_setup(
        setup: super::setup_phase::SetupOutput,
        token_manager: Arc<TokenManager>,
        upstream: Arc<UpstreamClient>,
        custom_mapping: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
        headers: HeaderMap,
        upstream_recorder: Option<
            axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
        >,
    ) -> Self {
        let pool_size = token_manager.len();
        // [FIX #3485] 自适应多账号池与单账号退避最大重试次数 (单账号3次，多账号整池两轮)
        let max_attempts = crate::proxy::handlers::common::calculate_max_retry_attempts(pool_size);
        let request_for_body = setup.request.clone();
        Self {
            trace_id: setup.trace_id,
            debug_cfg: setup.debug_cfg,
            client_adapter: setup.client_adapter,
            request: setup.request,
            original_body: setup.original_body,
            compression_level: setup.compression_level,
            scaling_enabled: setup.scaling_enabled,
            threshold_l1: setup.threshold_l1,
            threshold_l3: setup.threshold_l3,
            clean_ms: setup.clean_ms,
            upstream,
            token_manager,
            headers,
            upstream_recorder,
            custom_mapping,
            max_attempts,
            pool_size,
            request_for_body,
            last_error: String::new(),
            retried_without_thinking: false,
            last_email: None,
            last_mapped_model: None,
            last_status: StatusCode::SERVICE_UNAVAILABLE, // Default to 503 if no response reached
            force_rotate: false,
            norm_ms: 0.0,
            think_fill_ms: 0.0,
            ttft_ms: 0.0,
        }
    }
}
