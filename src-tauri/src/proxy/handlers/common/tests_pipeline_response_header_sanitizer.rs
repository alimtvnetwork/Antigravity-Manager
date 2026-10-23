//! Unit tests for Pipeline Response Header Sanitizer (`Imp-685`).

use super::pipeline_response_header_sanitizer::*;
use std::collections::BTreeMap;

#[test]
fn test_is_forbidden_response_header() {
    assert!(is_forbidden_response_header("Server"));
    assert!(is_forbidden_response_header("alt-svc"));
    assert!(is_forbidden_response_header("x-goog-safety-ratings"));
    assert!(is_forbidden_response_header("X-Goog-Ratelimit-Remaining"));
    assert!(is_forbidden_response_header("Set-Cookie"));
    assert!(is_forbidden_response_header("Transfer-Encoding"));
    assert!(!is_forbidden_response_header("Date"));
    assert!(!is_forbidden_response_header("ETag"));
}

#[test]
fn test_sanitize_response_headers_streaming_mode() {
    let mut raw = BTreeMap::new();
    raw.insert("Server".to_string(), "ESF".to_string());
    raw.insert("alt-svc".to_string(), "h3=\":443\"".to_string());
    raw.insert("Date".to_string(), "Fri, 23 Oct 2026 12:00:00 GMT".to_string());
    raw.insert("x-goog-safety-ratings".to_string(), "HARM_CATEGORY_HARASSMENT: NEGLIGIBLE".to_string());

    let report = sanitize_response_headers(&raw, ResponseSanitizeMode::StreamingEventStream);
    assert!(report.is_sanitized);

    // Stripped headers
    assert!(report.stripped_headers.contains(&"Server".to_string()));
    assert!(report.stripped_headers.contains(&"alt-svc".to_string()));
    assert!(report.stripped_headers.contains(&"x-goog-safety-ratings".to_string()));

    // Injected headers
    assert_eq!(
        report.sanitized_headers_map.get("content-type").unwrap(),
        "text/event-stream; charset=utf-8"
    );
    assert_eq!(
        report.sanitized_headers_map.get("cache-control").unwrap(),
        "no-cache, no-transform"
    );

    // Preserved safe header
    assert_eq!(
        report.sanitized_headers_map.get("date").unwrap(),
        "Fri, 23 Oct 2026 12:00:00 GMT"
    );
}

#[test]
fn test_sanitize_response_headers_unary_mode() {
    let mut raw = BTreeMap::new();
    raw.insert("Content-Length".to_string(), "42".to_string());
    raw.insert("Set-Cookie".to_string(), "session=abc".to_string());

    let report = sanitize_response_headers(&raw, ResponseSanitizeMode::UnaryJson);
    assert!(!report.sanitized_headers_map.contains_key("set-cookie"));
    assert_eq!(
        report.sanitized_headers_map.get("content-type").unwrap(),
        "application/json; charset=utf-8"
    );
    assert_eq!(
        report.sanitized_headers_map.get("content-length").unwrap(),
        "42"
    );
}
