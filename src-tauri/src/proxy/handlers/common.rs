use crate::proxy::server::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

pub mod abort_sentinel;
pub mod boundary_normalizer;
pub mod canonical_role_mapper;
pub mod chunk_boundary_validator;
pub mod compression_guard;
pub mod compression_negotiator;
pub mod context_guard;
pub mod curl_gen;
pub mod decompressor;
pub mod frame_clamper;
pub mod frame_length_clamper;
mod handle_detect_model;
pub mod header_allowlist_filter;
pub mod header_anonymizer;
pub mod json_payload_sanitizer;
pub mod pipeline_request_header_sanitizer;
pub mod prefix_stability_pipeline_guard;
pub mod prefix_stability_sanitizer;
pub mod request_cancellation_watcher;
pub mod request_content_encoding_negotiator;
pub mod request_content_length_validator;
pub mod request_gzip_sniffer;
pub mod request_header_sanitizer;
pub mod request_payload_length_negotiator;
pub mod request_size_guard;
pub mod response_content_type_matcher;
pub mod response_stream_delimiter_normalizer;
pub(crate) mod retrystrategy;
pub mod role_normalization_validator;
pub mod sse_boundary_normalizer;
pub mod sse_chunk_coalescer;
pub mod sse_comment_filter;
pub mod sse_delimiter_normalizer;
pub mod sse_framing_validator;
pub mod sse_heartbeat;
pub mod sse_heartbeat_generator;
pub mod sse_ping_heartbeat;
pub mod sse_stream_padder;
pub mod sse_stream_splitter;
pub mod status_mapper;
pub mod stream_abort;
pub mod stream_boundary_guard;
pub mod stream_drain_guard;
pub mod stream_early_abort_sentinel;
pub mod stream_flusher;
pub mod thinking_budget_filter_pipeline;
pub mod thinking_budget_headroom_evaluator;
pub mod token_budget_decider;
pub mod tool_response_formatter;
pub mod unified_context_structural_aligner;
pub mod user_agent_normalizer;
#[cfg(test)]
pub mod tests_request_gzip_sniffer;
#[cfg(test)]
pub mod tests_sse_splitter_padder;
#[cfg(test)]
pub mod tests_sse_delimiter_normalizer;
#[cfg(test)]
pub mod tests_request_cancellation_watcher;
#[cfg(test)]
pub mod tests_request_size_guard;
#[cfg(test)]
pub mod tests_request_header_sanitizer;
#[cfg(test)]
pub mod tests_request_content_length_validator;
#[cfg(test)]
pub mod tests_response_content_type_matcher;
#[cfg(test)]
pub mod tests_response_stream_delimiter_normalizer;
#[cfg(test)]
pub mod tests_compression_negotiator;
#[cfg(test)]
pub mod tests_stream_flusher;
#[cfg(test)]
pub mod tests_stream_early_abort_sentinel;
#[cfg(test)]
pub mod tests_sse_chunk_coalescer;
#[cfg(test)]
pub mod tests_sse_comment_filter;
#[cfg(test)]
pub mod tests_role_normalization_validator;
#[cfg(test)]
pub mod tests_request_content_encoding_negotiator;
#[cfg(test)]
pub mod tests_decompressor;
#[cfg(test)]
pub mod tests_boundary_normalizer;
#[cfg(test)]
pub mod tests_chunk_boundary_validator;
#[cfg(test)]
pub mod tests_json_payload_sanitizer;
#[cfg(test)]
pub mod tests_payload_length_negotiator;
#[cfg(test)]
pub mod tests_pipeline_request_header_sanitizer;
#[cfg(test)]
pub mod tests_sse_heartbeat_generator;
#[cfg(test)]
pub mod tests_token_budget_decider;
#[cfg(test)]
pub mod tests_thinking_budget_filter_pipeline;
#[cfg(test)]
pub mod tests_thinking_budget_headroom_evaluator;
#[cfg(test)]
pub mod tests_prefix_stability_pipeline_guard;
#[cfg(test)]
pub mod tests_prefix_stability_sanitizer;
#[cfg(test)]
pub mod tests_unified_context_structural_aligner;

pub use abort_sentinel::*;
pub use boundary_normalizer::StreamBoundaryNormalizer;
pub use canonical_role_mapper::*;
pub use chunk_boundary_validator::*;
pub use compression_guard::*;
pub use compression_negotiator::*;
pub use context_guard::*;
pub use curl_gen::generate_reproducible_curl;
pub use decompressor::decompress_payload;
pub use frame_clamper::*;
pub use frame_length_clamper::*;
pub use header_allowlist_filter::*;
pub use header_anonymizer::anonymize_forwarded_headers;
pub use json_payload_sanitizer::*;
pub use pipeline_request_header_sanitizer::*;
pub use prefix_stability_pipeline_guard::*;
pub use prefix_stability_sanitizer::*;
pub use request_cancellation_watcher::*;
pub use request_content_encoding_negotiator::*;
pub use request_content_length_validator::*;
pub use request_gzip_sniffer::*;
pub use request_header_sanitizer::*;
pub use request_payload_length_negotiator::*;
pub use request_size_guard::*;
pub use response_content_type_matcher::*;
pub use response_stream_delimiter_normalizer::*;
pub use role_normalization_validator::*;
pub use sse_boundary_normalizer::*;
pub use sse_chunk_coalescer::*;
pub use sse_comment_filter::*;
pub use sse_delimiter_normalizer::*;
pub use sse_framing_validator::*;
pub use sse_heartbeat::*;
pub use sse_heartbeat_generator::*;
pub use sse_ping_heartbeat::*;
pub use sse_stream_padder::*;
pub use sse_stream_splitter::*;
pub use status_mapper::{map_upstream_status_to_canonical, CanonicalErrorPayload};
pub use stream_abort::{StreamAbortHandle, StreamAbortReceiver};
pub use stream_boundary_guard::*;
pub use stream_drain_guard::*;
pub use stream_early_abort_sentinel::*;
pub use stream_flusher::{StreamFlusher, SSE_KEEPALIVE_BYTES};
pub use thinking_budget_filter_pipeline::*;
pub use thinking_budget_headroom_evaluator::*;
pub use token_budget_decider::*;
pub use tool_response_formatter::*;
pub use unified_context_structural_aligner::*;
pub use user_agent_normalizer::*;

pub use handle_detect_model::build_dual_track_error;
pub use handle_detect_model::build_token_error_headers;
pub use handle_detect_model::extract_retry_after_seconds;
pub use handle_detect_model::handle_detect_model;
pub use handle_detect_model::is_model_not_found_error;
pub(crate) use handle_detect_model::map_status_code_to_gemini_status;
pub use handle_detect_model::parse_raw_upstream_error;
#[cfg(test)]
pub(crate) use handle_detect_model::retry_after_tests;
pub use retrystrategy::apply_retry_strategy;
pub use retrystrategy::calculate_max_retry_attempts;
pub use retrystrategy::determine_retry_strategy;
pub use retrystrategy::determine_retry_strategy_adaptive;
pub use retrystrategy::determine_retry_strategy_with_grace;
pub use retrystrategy::next_rotation_attempt;
pub use retrystrategy::should_rotate_account;
#[cfg(test)]
pub(crate) use retrystrategy::tests;
pub use retrystrategy::FailureStatusTracker;
pub use retrystrategy::RequestRetryState;
pub use retrystrategy::RetryStrategy;

/// Imp-49: Fast zero-allocation case-insensitive header comparison
pub fn header_eq_ignore_case(headers: &axum::http::HeaderMap, key: &str, expected: &str) -> bool {
    if let Some(val) = headers.get(key) {
        if let Ok(val_str) = val.to_str() {
            return val_str.trim().eq_ignore_ascii_case(expected.trim());
        }
    }
    false
}

/// Imp-49: Fast zero-allocation case-insensitive header substring matching
pub fn header_contains_ignore_case(headers: &axum::http::HeaderMap, key: &str, needle: &str) -> bool {
    if let Some(val) = headers.get(key) {
        if let Ok(val_str) = val.to_str() {
            let v_bytes = val_str.as_bytes();
            let n_bytes = needle.as_bytes();
            if n_bytes.is_empty() {
                return true;
            }
            if v_bytes.len() < n_bytes.len() {
                return false;
            }
            return v_bytes
                .windows(n_bytes.len())
                .any(|window| window.eq_ignore_ascii_case(n_bytes));
        }
    }
    false
}
