//! Proxy Request Header Fingerprint Anonymizer.
//! Strips intermediary client IP trackers and internal hop headers before forwarding to upstream models.

use axum::http::HeaderMap;

pub const ANONYMIZED_HEADER_KEYS: &[&str] = &[
    "x-forwarded-for",
    "x-forwarded-host",
    "x-forwarded-proto",
    "x-real-ip",
    "via",
    "forwarded",
    "client-ip",
];

/// Strips client identity headers from an outgoing HeaderMap.
pub fn anonymize_forwarded_headers(headers: &mut HeaderMap) {
    for key in ANONYMIZED_HEADER_KEYS {
        headers.remove(*key);
    }
}
