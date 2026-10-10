// Claude 协议处理器

use axum::{
    body::Body,
    extract::{Json, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures::StreamExt;
use serde_json::{json, Value};
use tokio::time::Duration;
use tracing::{debug, error, info};

use crate::proxy::common::client_adapter::CLIENT_ADAPTERS; // [NEW] Import Adapter Registry
use crate::proxy::debug_logger;
use crate::proxy::mappers::claude::{
    clean_cache_control_from_messages, create_claude_sse_stream,
    filter_invalid_thinking_blocks_with_family, merge_consecutive_messages,
    models::{Message, MessageContent},
    transform_response, ClaudeRequest,
};
use crate::proxy::mappers::context_manager::ContextManager;
use crate::proxy::mappers::estimation_calibrator::get_calibrator;
use crate::proxy::mappers::gemini::SUMMARY_REQUEST_TIMEOUT_SECS;
use crate::proxy::model_specs;
use crate::proxy::server::AppState;
use crate::proxy::upstream::client::mask_email;
use axum::http::HeaderMap;
use std::sync::{atomic::Ordering, Arc}; // [NEW]

// ===== Task #6: OpenCode variants thinking config mapping =====
// Helper structs for parsing thinking hints from raw JSON
// Claude protocol handler — facade module.
// The implementation is split into focused submodules; public paths are
// preserved via re-exports below.
mod apply_compression;
mod attempt;
mod attempt_setup;
mod background;
mod compression;
mod consts;
mod error_handling;
mod handler;
mod helpers;
#[cfg(test)]
mod opus_tests;
mod request_log;
mod response_handling;
mod setup_phase;
mod stream_handling;
mod thinking;
mod variant;
#[cfg(test)]
mod variant_tests;
mod warmup;
#[cfg(test)]
mod warmup_tests;

pub use handler::{handle_count_tokens, handle_list_models, handle_messages};
pub(crate) use attempt::{AttemptCall, AttemptState, ErrorOutcome, PrepOutcome, StreamOutcome};
pub(crate) use attempt_setup::prepare_attempt;
pub(crate) use setup_phase::preprocess_request;
