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

    /// Checks if a header name is permitted in forwarded requests.
    pub fn is_allowed_header(name: &str) -> bool {
        let name_lower = name.to_ascii_lowercase();
        Self::ALLOWED_HEADERS.iter().any(|&h| h == name_lower)
    }

    /// Filters a HeaderMap, stripping all headers not present on the allowlist.
    pub fn filter_headers(headers: &HeaderMap) -> HeaderMap {
        let mut clean = HeaderMap::new();
        for (key, value) in headers {
            if Self::is_allowed_header(key.as_str()) {
                clean.insert(key.clone(), value.clone());
            }
        }
        clean
    }
}
