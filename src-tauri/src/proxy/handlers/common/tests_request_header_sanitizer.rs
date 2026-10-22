#[cfg(test)]
mod tests {
    use super::super::request_header_sanitizer::RequestHeaderSanitizer;
    use axum::http::{HeaderMap, HeaderName, HeaderValue};

    #[test]
    fn test_sanitize_strips_hop_by_hop_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("connection"), HeaderValue::from_static("close"));
        headers.insert(HeaderName::from_static("upgrade"), HeaderValue::from_static("websocket"));
        headers.insert(HeaderName::from_static("keep-alive"), HeaderValue::from_static("timeout=5"));
        headers.insert(HeaderName::from_static("transfer-encoding"), HeaderValue::from_static("chunked"));
        headers.insert(HeaderName::from_static("host"), HeaderValue::from_static("api.anthropic.com"));
        headers.insert(HeaderName::from_static("content-length"), HeaderValue::from_static("1234"));

        let clean = RequestHeaderSanitizer::sanitize(&headers);
        assert!(clean.is_empty(), "All hop-by-hop headers should be stripped");
    }

    #[test]
    fn test_sanitize_strips_internal_agm_and_identity_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("x-agm-instance-id"), HeaderValue::from_static("inst-1"));
        headers.insert(HeaderName::from_static("x-agm-trace-route"), HeaderValue::from_static("fast-forward"));
        headers.insert(HeaderName::from_static("cookie"), HeaderValue::from_static("session=secret"));
        headers.insert(HeaderName::from_static("x-real-ip"), HeaderValue::from_static("192.168.1.10"));
        headers.insert(HeaderName::from_static("x-forwarded-for"), HeaderValue::from_static("10.0.0.1"));

        let clean = RequestHeaderSanitizer::sanitize(&headers);
        assert!(clean.is_empty(), "Internal AGM and sensitive client identity headers must be stripped");
    }

    #[test]
    fn test_sanitize_preserves_safe_payload_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("authorization"), HeaderValue::from_static("Bearer sk-test"));
        headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
        headers.insert(HeaderName::from_static("accept"), HeaderValue::from_static("text/event-stream"));
        headers.insert(HeaderName::from_static("x-custom-tenant"), HeaderValue::from_static("alpha"));
        // Mix in blocked header
        headers.insert(HeaderName::from_static("keep-alive"), HeaderValue::from_static("timeout=10"));

        let clean = RequestHeaderSanitizer::sanitize(&headers);
        assert_eq!(clean.len(), 4);
        assert_eq!(clean.get("authorization").unwrap(), "Bearer sk-test");
        assert_eq!(clean.get("content-type").unwrap(), "application/json");
        assert_eq!(clean.get("accept").unwrap(), "text/event-stream");
        assert_eq!(clean.get("x-custom-tenant").unwrap(), "alpha");
        assert!(!clean.contains_key("keep-alive"));
    }

    #[test]
    fn test_sanitize_empty_headers() {
        let headers = HeaderMap::new();
        let clean = RequestHeaderSanitizer::sanitize(&headers);
        assert!(clean.is_empty());
    }
}
