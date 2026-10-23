//! Unit tests for Pipeline Response Header Sanitizer (`Imp-685`).

use super::pipeline_response_header_sanitizer::*;
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::HeaderMap;

#[test]
fn test_is_forbidden_response_header() {
    assert!(is_forbidden_response_header("Server"));
    assert!(is_forbidden_response_header("alt-svc"));
    assert!(is_forbidden_response_header("x-goog-safety-ratings"));
    assert!(is_forbidden_response_header("X-Goog-Ratelimit-Remaining"));
    assert!(is_forbidden_response_header("Set-Cookie"));
    assert!(is_forbidden_response_header("Transfer-Encoding"));
    assert!(is_forbidden_response_header("connection"));
    assert!(is_forbidden_response_header("upgrade"));
    assert!(is_forbidden_response_header("keep-alive"));
    assert!(is_forbidden_response_header("x-agm-debug"));
    assert!(is_forbidden_response_header("x-internal-route"));

    assert!(!is_forbidden_response_header("Date"));
    assert!(!is_forbidden_response_header("ETag"));
    assert!(!is_forbidden_response_header("Content-Length"));
}

#[test]
fn test_classify_header() {
    assert_eq!(
        classify_header("connection"),
        HeaderDisposition::StripHopByHop
    );
    assert_eq!(
        classify_header("alt-svc"),
        HeaderDisposition::StripHopByHop
    );
    assert_eq!(
        classify_header("transfer-encoding"),
        HeaderDisposition::StripHopByHop
    );
    assert_eq!(
        classify_header("server"),
        HeaderDisposition::StripInternal
    );
    assert_eq!(
        classify_header("set-cookie"),
        HeaderDisposition::StripInternal
    );
    assert_eq!(
        classify_header("x-goog-safety-ratings"),
        HeaderDisposition::StripInternal
    );
    assert_eq!(
        classify_header("x-goog-ratelimit-remaining"),
        HeaderDisposition::StripInternal
    );
    assert_eq!(
        classify_header("date"),
        HeaderDisposition::Keep
    );
    assert_eq!(
        classify_header("etag"),
        HeaderDisposition::Keep
    );
}

#[test]
fn test_sanitize_response_headers_strips_internal_and_hop_by_hop() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("server"),
        HeaderValue::from_static("ESF"),
    );
    headers.insert(
        HeaderName::from_static("alt-svc"),
        HeaderValue::from_static("h3=\":443\""),
    );
    headers.insert(
        HeaderName::from_static("x-goog-safety-ratings"),
        HeaderValue::from_static("HARM_CATEGORY_HARASSMENT: NEGLIGIBLE"),
    );
    headers.insert(
        HeaderName::from_static("x-goog-ratelimit-remaining"),
        HeaderValue::from_static("100"),
    );
    headers.insert(
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_static("session_id=secret123; Path=/"),
    );
    headers.insert(
        HeaderName::from_static("connection"),
        HeaderValue::from_static("close"),
    );
    headers.insert(
        HeaderName::from_static("date"),
        HeaderValue::from_static("Fri, 23 Oct 2026 12:00:00 GMT"),
    );
    headers.insert(
        HeaderName::from_static("etag"),
        HeaderValue::from_static("\"etag-hash-999\""),
    );

    let report = sanitize_response_headers(&headers);

    assert_eq!(report.stripped_headers.len(), 6);
    assert!(report.has_stripped());
    assert!(!report.is_clean());

    let stripped_names: Vec<&str> = report
        .stripped_headers
        .iter()
        .map(|f| f.header_name.as_str())
        .collect();
    assert!(stripped_names.contains(&"server"));
    assert!(stripped_names.contains(&"alt-svc"));
    assert!(stripped_names.contains(&"x-goog-safety-ratings"));
    assert!(stripped_names.contains(&"x-goog-ratelimit-remaining"));
    assert!(stripped_names.contains(&"set-cookie"));
    assert!(stripped_names.contains(&"connection"));

    assert!(!report.final_headers.contains_key("server"));
    assert!(!report.final_headers.contains_key("alt-svc"));
    assert!(!report.final_headers.contains_key("x-goog-safety-ratings"));
    assert!(!report.final_headers.contains_key("x-goog-ratelimit-remaining"));
    assert!(!report.final_headers.contains_key("set-cookie"));
    assert!(!report.final_headers.contains_key("connection"));

    assert_eq!(
        report.get_final_header_str("date").unwrap(),
        "Fri, 23 Oct 2026 12:00:00 GMT"
    );
    assert_eq!(
        report.get_final_header_str("etag").unwrap(),
        "\"etag-hash-999\""
    );
}

#[test]
fn test_sanitize_response_headers_normalizes_content_type_and_cache_control() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("public, max-age=3600"),
    );

    let report = sanitize_response_headers(&headers);

    assert_eq!(
        report.get_final_header_str("content-type").unwrap(),
        "text/event-stream; charset=utf-8"
    );
    assert_eq!(
        report.get_final_header_str("cache-control").unwrap(),
        "no-cache"
    );

    assert!(report.has_normalized());
    assert_eq!(report.normalized_count(), 2);
}

#[test]
fn test_sanitize_response_headers_injects_defaults_when_missing() {
    let headers = HeaderMap::new();
    let report = sanitize_response_headers(&headers);

    assert_eq!(
        report.get_final_header_str("content-type").unwrap(),
        DEFAULT_STREAM_CONTENT_TYPE
    );
    assert_eq!(
        report.get_final_header_str("cache-control").unwrap(),
        DEFAULT_CACHE_CONTROL
    );
    assert_eq!(report.normalized_headers.len(), 2);
    assert!(report.stripped_headers.is_empty());
    assert!(report.is_clean());
}

#[test]
fn test_sanitize_response_headers_preserves_already_normalized_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-cache"),
    );
    headers.insert(
        HeaderName::from_static("date"),
        HeaderValue::from_static("Fri, 23 Oct 2026 12:00:00 GMT"),
    );

    let report = sanitize_response_headers(&headers);

    assert!(report.is_clean());
    assert_eq!(report.stripped_count(), 0);
    assert_eq!(report.normalized_count(), 0);
    assert_eq!(
        report.get_final_header_str("content-type").unwrap(),
        "text/event-stream; charset=utf-8"
    );
    assert_eq!(
        report.get_final_header_str("cache-control").unwrap(),
        "no-cache"
    );
}

#[test]
fn test_sanitize_response_headers_json_normalization() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );

    let report = sanitize_response_headers(&headers);

    assert_eq!(
        report.get_final_header_str("content-type").unwrap(),
        "application/json; charset=utf-8"
    );
}

#[test]
fn test_report_serde_roundtrip() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("server"),
        HeaderValue::from_static("envoy"),
    );
    headers.insert(
        HeaderName::from_static("x-goog-ratelimit-remaining"),
        HeaderValue::from_static("50"),
    );
    headers.insert(
        HeaderName::from_static("date"),
        HeaderValue::from_static("Fri, 23 Oct 2026 12:00:00 GMT"),
    );

    let report = sanitize_response_headers(&headers);
    let serialized = serde_json::to_string(&report).expect("serialization failed");
    let deserialized: ResponseHeaderSanitizationReport =
        serde_json::from_str(&serialized).expect("deserialization failed");

    assert_eq!(report.stripped_headers, deserialized.stripped_headers);
    assert_eq!(report.normalized_headers, deserialized.normalized_headers);
    assert_eq!(
        report.get_final_header_str("content-type"),
        deserialized.get_final_header_str("content-type")
    );
    assert_eq!(
        report.get_final_header_str("cache-control"),
        deserialized.get_final_header_str("cache-control")
    );
    assert_eq!(
        report.get_final_header_str("date"),
        deserialized.get_final_header_str("date")
    );
}

#[test]
fn test_disposition_display() {
    assert_eq!(format!("{}", HeaderDisposition::Keep), "Keep");
    assert_eq!(
        format!("{}", HeaderDisposition::StripInternal),
        "StripInternal"
    );
    assert_eq!(
        format!("{}", HeaderDisposition::StripHopByHop),
        "StripHopByHop"
    );
    assert_eq!(
        format!("{}", HeaderDisposition::NormalizeStandard),
        "NormalizeStandard"
    );
}
