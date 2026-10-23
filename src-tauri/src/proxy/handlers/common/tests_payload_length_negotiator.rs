//! Unit tests for Request Payload Length Negotiator (`Imp-615`)

use super::request_payload_length_negotiator::{
    negotiate_payload_strategy, negotiate_payload_strategy_from_headers,
    negotiate_payload_with_report, PayloadLengthDecision, PayloadNegotiationConfig,
    TransferEncodingMode, DEFAULT_MAX_BUFFERED_BYTES, DEFAULT_STREAM_THRESHOLD_BYTES,
};
use axum::http::header::{CONTENT_LENGTH, TRANSFER_ENCODING};
use axum::http::{HeaderMap, HeaderValue};

#[test]
fn test_chunked_transfer_streams_directly() {
    let config = PayloadNegotiationConfig::default();

    // Chunked without content length
    let decision = negotiate_payload_strategy(None, true, &config);
    assert_eq!(decision, PayloadLengthDecision::StreamDirectly);
    assert!(decision.is_stream());

    // Chunked with content length provided (chunked overrides)
    let decision_with_len = negotiate_payload_strategy(Some(4096), true, &config);
    assert_eq!(decision_with_len, PayloadLengthDecision::StreamDirectly);
}

#[test]
fn test_absent_content_length_streams_directly() {
    let config = PayloadNegotiationConfig::default();

    let decision = negotiate_payload_strategy(None, false, &config);
    assert_eq!(decision, PayloadLengthDecision::StreamDirectly);
    assert!(!decision.is_rejected());
    assert!(!decision.is_buffered());
}

#[test]
fn test_small_payload_buffers_in_memory() {
    let config = PayloadNegotiationConfig::default();

    // Zero-byte request
    let zero = negotiate_payload_strategy(Some(0), false, &config);
    assert_eq!(zero, PayloadLengthDecision::BufferInMemory(0));
    assert!(zero.is_buffered());

    // 512 KB request (below 1 MB stream threshold)
    let small = negotiate_payload_strategy(Some(512 * 1024), false, &config);
    assert_eq!(small, PayloadLengthDecision::BufferInMemory(524_288));

    // Exactly at stream threshold (1 MB)
    let threshold = negotiate_payload_strategy(Some(1024 * 1024), false, &config);
    assert_eq!(threshold, PayloadLengthDecision::BufferInMemory(1_048_576));
}

#[test]
fn test_payload_above_stream_threshold_streams_directly() {
    let config = PayloadNegotiationConfig::default();

    // 1 MB + 1 byte
    let just_above = negotiate_payload_strategy(Some((1024 * 1024) + 1), false, &config);
    assert_eq!(just_above, PayloadLengthDecision::StreamDirectly);

    // 5 MB payload
    let medium = negotiate_payload_strategy(Some(5 * 1024 * 1024), false, &config);
    assert_eq!(medium, PayloadLengthDecision::StreamDirectly);

    // Exactly at max buffered ceiling (10 MB)
    let at_max = negotiate_payload_strategy(Some(10 * 1024 * 1024), false, &config);
    assert_eq!(at_max, PayloadLengthDecision::StreamDirectly);
}

#[test]
fn test_payload_above_max_buffered_rejected() {
    let config = PayloadNegotiationConfig::default();

    // 10 MB + 1 byte
    let oversize = (DEFAULT_MAX_BUFFERED_BYTES as u64) + 1;
    let decision = negotiate_payload_strategy(Some(oversize), false, &config);
    assert_eq!(
        decision,
        PayloadLengthDecision::RejectOversized(oversize, DEFAULT_MAX_BUFFERED_BYTES as u64)
    );
    assert!(decision.is_rejected());

    // 50 MB payload
    let huge = 50 * 1024 * 1024;
    let decision_huge = negotiate_payload_strategy(Some(huge), false, &config);
    assert_eq!(
        decision_huge,
        PayloadLengthDecision::RejectOversized(huge, DEFAULT_MAX_BUFFERED_BYTES as u64)
    );
}

#[test]
fn test_custom_config_thresholds() {
    let custom = PayloadNegotiationConfig::new(4000, 1000);

    assert_eq!(
        negotiate_payload_strategy(Some(500), false, &custom),
        PayloadLengthDecision::BufferInMemory(500)
    );
    assert_eq!(
        negotiate_payload_strategy(Some(1000), false, &custom),
        PayloadLengthDecision::BufferInMemory(1000)
    );
    assert_eq!(
        negotiate_payload_strategy(Some(1001), false, &custom),
        PayloadLengthDecision::StreamDirectly
    );
    assert_eq!(
        negotiate_payload_strategy(Some(4000), false, &custom),
        PayloadLengthDecision::StreamDirectly
    );
    assert_eq!(
        negotiate_payload_strategy(Some(4001), false, &custom),
        PayloadLengthDecision::RejectOversized(4001, 4000)
    );
}

#[test]
fn test_headers_parsing_transfer_encoding_chunked() {
    let config = PayloadNegotiationConfig::default();

    let mut headers = HeaderMap::new();
    headers.insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    assert_eq!(
        negotiate_payload_strategy_from_headers(&headers, &config),
        PayloadLengthDecision::StreamDirectly
    );

    // Multi-value or uppercase chunked
    let mut multi_headers = HeaderMap::new();
    multi_headers.insert(TRANSFER_ENCODING, HeaderValue::from_static("gzip, Chunked"));
    assert_eq!(
        negotiate_payload_strategy_from_headers(&multi_headers, &config),
        PayloadLengthDecision::StreamDirectly
    );
}

#[test]
fn test_headers_parsing_content_length() {
    let config = PayloadNegotiationConfig::default();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("4096"));
    assert_eq!(
        negotiate_payload_strategy_from_headers(&headers, &config),
        PayloadLengthDecision::BufferInMemory(4096)
    );

    let mut headers_oversize = HeaderMap::new();
    let limit_str = (DEFAULT_MAX_BUFFERED_BYTES + 1024).to_string();
    headers_oversize.insert(CONTENT_LENGTH, HeaderValue::from_str(&limit_str).unwrap());
    assert_eq!(
        negotiate_payload_strategy_from_headers(&headers_oversize, &config),
        PayloadLengthDecision::RejectOversized(
            DEFAULT_MAX_BUFFERED_BYTES as u64 + 1024,
            DEFAULT_MAX_BUFFERED_BYTES as u64
        )
    );
}

#[test]
fn test_headers_parsing_empty_and_malformed() {
    let config = PayloadNegotiationConfig::default();

    let headers_empty = HeaderMap::new();
    assert_eq!(
        negotiate_payload_strategy_from_headers(&headers_empty, &config),
        PayloadLengthDecision::StreamDirectly
    );

    let mut headers_malformed = HeaderMap::new();
    headers_malformed.insert(CONTENT_LENGTH, HeaderValue::from_static("not-a-number"));
    assert_eq!(
        negotiate_payload_strategy_from_headers(&headers_malformed, &config),
        PayloadLengthDecision::StreamDirectly
    );
}

#[test]
fn test_negotiate_payload_with_report() {
    let config = PayloadNegotiationConfig::default();

    let rep_buf = negotiate_payload_with_report(Some(2048), false, &config);
    assert_eq!(
        rep_buf.decision,
        PayloadLengthDecision::BufferInMemory(2048)
    );
    assert_eq!(
        rep_buf.transfer_mode,
        TransferEncodingMode::FixedContentLength(2048)
    );
    assert!(rep_buf.reason.contains("buffering in memory"));

    let rep_stream = negotiate_payload_with_report(None, true, &config);
    assert_eq!(rep_stream.decision, PayloadLengthDecision::StreamDirectly);
    assert_eq!(
        rep_stream.transfer_mode,
        TransferEncodingMode::ChunkedTransfer
    );
    assert!(rep_stream.reason.contains("streaming directly"));

    let rep_reject = negotiate_payload_with_report(Some(20 * 1024 * 1024), false, &config);
    assert!(rep_reject.decision.is_rejected());
    assert!(rep_reject.reason.contains("exceeds allowable ceiling"));
}
