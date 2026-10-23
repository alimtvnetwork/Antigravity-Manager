//! Header Allowlist Filter
//! Filters client request headers against a strict security allowlist,
//! preventing credential leakage, internal cookies, and tracking headers from reaching upstream.

use axum::http::HeaderMap;

pub struct HeaderAllowlistFilter;

impl HeaderAllowlistFilter {
    /// Canonical list of permitted headers forwarded to upstream AI providers.
    pub const ALLOWED_HEADERS: &'static [&'static str] = &[
        "content-type",
        "accept",
        "accept-encoding",
        "authorization",
        "user-agent",
        "x-goog-api-client",
        "x-goog-api-key",
        "x-api-key",
        "anthropic-version",
        "anthropic-beta",
        "anthropic-client",
    ];

    /// RFC 2616 / RFC 7230 hop-by-hop headers that must never be forwarded.
    pub const HOP_BY_HOP_HEADERS: &'static [&'static str] = &[
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
        "upgrade",
    ];

    /// Client tracing, IP forwarding, and session state headers.
    pub const CLIENT_TRACE_HEADERS: &'static [&'static str] = &[
        "x-forwarded-for",
        "x-real-ip",
        "x-request-id",
        "cookie",
        "set-cookie",
    ];

    /// Checks if a header is an RFC hop-by-hop transport header.
    pub fn is_hop_by_hop_header(name: &str) -> bool {
        let name_lower = name.to_ascii_lowercase();
        Self::HOP_BY_HOP_HEADERS.iter().any(|&h| h == name_lower)
    }

    /// Checks if a header contains client tracking or cookie state.
    pub fn is_client_trace_header(name: &str) -> bool {
        let name_lower = name.to_ascii_lowercase();
        Self::CLIENT_TRACE_HEADERS.iter().any(|&h| h == name_lower)
    }

    /// Checks if a header name is permitted in forwarded requests.
    pub fn is_allowed_header(name: &str) -> bool {
        let name_lower = name.to_ascii_lowercase();
        Self::ALLOWED_HEADERS.iter().any(|&h| h == name_lower)
    }

    /// Affirmative predicate determining if a header is safe and eligible for forwarding.
    pub fn is_forwardable_header(name: &str) -> bool {
        Self::is_allowed_header(name) && !Self::is_hop_by_hop_header(name)
    }

    /// Filters a HeaderMap, stripping all headers not present on the allowlist or flagged as hop-by-hop.
    pub fn filter_headers(headers: &HeaderMap) -> HeaderMap {
        let mut clean = HeaderMap::new();
        for (key, value) in headers {
            if Self::is_forwardable_header(key.as_str()) {
                clean.insert(key.clone(), value.clone());
            }
        }
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_is_allowed_header() {
        assert!(HeaderAllowlistFilter::is_allowed_header("content-type"));
        assert!(HeaderAllowlistFilter::is_allowed_header("Content-Type"));
        assert!(HeaderAllowlistFilter::is_allowed_header("AUTHORIZATION"));
        assert!(HeaderAllowlistFilter::is_allowed_header(
            "anthropic-version"
        ));
        assert!(!HeaderAllowlistFilter::is_allowed_header("x-custom-secret"));
    }

    #[test]
    fn test_is_hop_by_hop_header() {
        assert!(HeaderAllowlistFilter::is_hop_by_hop_header("connection"));
        assert!(HeaderAllowlistFilter::is_hop_by_hop_header("Upgrade"));
        assert!(HeaderAllowlistFilter::is_hop_by_hop_header(
            "transfer-encoding"
        ));
        assert!(!HeaderAllowlistFilter::is_hop_by_hop_header("content-type"));
    }

    #[test]
    fn test_is_client_trace_header() {
        assert!(HeaderAllowlistFilter::is_client_trace_header(
            "x-forwarded-for"
        ));
        assert!(HeaderAllowlistFilter::is_client_trace_header("Cookie"));
        assert!(HeaderAllowlistFilter::is_client_trace_header("x-real-ip"));
        assert!(!HeaderAllowlistFilter::is_client_trace_header(
            "authorization"
        ));
    }

    #[test]
    fn test_is_forwardable_header() {
        assert!(HeaderAllowlistFilter::is_forwardable_header("content-type"));
        assert!(HeaderAllowlistFilter::is_forwardable_header(
            "authorization"
        ));
        assert!(!HeaderAllowlistFilter::is_forwardable_header("connection"));
        assert!(!HeaderAllowlistFilter::is_forwardable_header("cookie"));
    }

    #[test]
    fn test_filter_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("cookie", HeaderValue::from_static("session_id=secret"));
        headers.insert("connection", HeaderValue::from_static("keep-alive"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        let clean = HeaderAllowlistFilter::filter_headers(&headers);
        assert!(clean.contains_key("content-type"));
        assert!(clean.contains_key("anthropic-version"));
        assert!(!clean.contains_key("cookie"));
        assert!(!clean.contains_key("connection"));
    }
}
