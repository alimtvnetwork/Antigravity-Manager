use crate::proxy::middleware::auth::UserTokenIdentity;
use crate::proxy::monitor::{ProxyMonitor, ProxyRequestLog, UpstreamRequestBodyHolder};
use crate::proxy::server::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use base64::Engine as _;
use futures::{Stream, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

const MAX_REQUEST_LOG_SIZE: usize = 100 * 1024 * 1024; // 100MB
const MAX_RESPONSE_LOG_SIZE: usize = 100 * 1024 * 1024; // 100MB for image responses
const MAX_LOGGED_FIELD_CHARS: usize = 500;

mod consolidation;
mod helpers;
mod middleware;
mod request_context;
mod stream_accumulator;
mod stream_processor;
mod summarization;

pub use middleware::monitor_middleware;

pub(crate) use consolidation::{
    build_canonical_consolidated_response, consolidate_non_streaming_response,
};
pub(crate) use helpers::{
    extract_boundary, extract_cached_tokens, extract_input_tokens, extract_output_tokens,
    extract_quoted_param, extract_reasoning_tokens, find_subslice, next_chunk_while_receiver_open,
    record_user_token_usage, should_log_health_checks, should_skip_request_log, trim_part_tail,
    truncate_for_log,
};
pub(crate) use request_context::{extract_request_context, MonitorRequestContext};
pub(crate) use stream_accumulator::SseParseAccumulator;
pub(crate) use stream_processor::process_collected_stream;
pub(crate) use summarization::{
    image_mime_and_dimensions, summarize_image_json_response, summarize_multipart_request,
};

#[cfg(test)]
mod tests;
