//! Response Content-Type Header Normalizer
//!
//! Inspects upstream response Content-Type headers, detects stream vs non-stream formats,
//! strips non-standard vendor extensions, and normalizes headers to downstream standards
//! (`text/event-stream; charset=utf-8`, `application/json; charset=utf-8`).

use axum::http::HeaderMap;
use tracing::debug;

/// Canonical classification of response content types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalContentType {
    ServerSentEvents,
    Json,
    PlainText,
    Binary,
    Other,
}

impl CanonicalContentType {
    /// Returns the canonical standard header value for downstream responses.
    pub fn standard_header_value(&self) -> &'static str {
        match self {
            CanonicalContentType::ServerSentEvents => "text/event-stream; charset=utf-8",
            CanonicalContentType::Json => "application/json; charset=utf-8",
            CanonicalContentType::PlainText => "text/plain; charset=utf-8",
            CanonicalContentType::Binary => "application/octet-stream",
            CanonicalContentType::Other => "application/octet-stream",
        }
    }
}

/// Matches and classifies a raw Content-Type header string.
pub fn classify_content_type(header_val: Option<&str>) -> CanonicalContentType {
    match header_val {
        Some(val) => {
            let lower = val.to_ascii_lowercase();
            if lower.contains("text/event-stream") {
                CanonicalContentType::ServerSentEvents
            } else if lower.contains("application/json") || lower.contains("+json") {
                CanonicalContentType::Json
            } else if lower.contains("text/plain") {
                CanonicalContentType::PlainText
            } else if lower.contains("application/octet-stream") || lower.contains("image/") {
                CanonicalContentType::Binary
            } else {
                CanonicalContentType::Other
            }
        }
        None => CanonicalContentType::Json, // Default fallback
    }
}

/// Normalizes the Content-Type header in an outgoing response HeaderMap.
pub fn normalize_content_type(headers: &mut HeaderMap, fallback: CanonicalContentType) {
    let current_val = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let canonical = match current_val {
        Some(v) => classify_content_type(Some(v)),
        None => fallback,
    };

    if let Ok(hdr) = axum::http::HeaderValue::from_str(canonical.standard_header_value()) {
        headers.insert(axum::http::header::CONTENT_TYPE, hdr);
        debug!(
            canonical = ?canonical,
            "Normalized downstream response Content-Type"
        );
    }
}
