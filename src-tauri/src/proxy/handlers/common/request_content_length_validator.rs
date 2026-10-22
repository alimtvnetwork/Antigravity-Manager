//! Request Content-Length & Body Size Bounds Validator
//!
//! Validates client HTTP request `Content-Length` headers, enforcing upper and lower
//! payload bounds (default max 32MB) to prevent buffer overflows, denial-of-service,
//! and mismatched streaming length anomalies.

use axum::http::HeaderMap;
use tracing::warn;

/// Maximum allowable request body size (32MB).
pub const DEFAULT_MAX_REQUEST_BODY_BYTES: usize = 32 * 1024 * 1024;

/// Result of request size validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentLengthValidation {
    /// Valid explicit content length within safe bounds.
    Valid(usize),
    /// Content length header is absent (e.g. chunked transfer encoding).
    ChunkedOrAbsent,
    /// Header value was malformed or negative.
    Malformed(String),
    /// Declared content length exceeds allowable ceiling.
    ExceedsLimit { declared: usize, limit: usize },
}

/// Inspects request headers and validates the declared Content-Length.
pub fn validate_content_length(
    headers: &HeaderMap,
    max_allowed_bytes: Option<usize>,
) -> ContentLengthValidation {
    let limit = max_allowed_bytes.unwrap_or(DEFAULT_MAX_REQUEST_BODY_BYTES);

    let header_val = match headers.get(axum::http::header::CONTENT_LENGTH) {
        Some(val) => val,
        None => return ContentLengthValidation::ChunkedOrAbsent,
    };

    let s = match header_val.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return ContentLengthValidation::Malformed("Non-ASCII Content-Length header".to_string()),
    };

    match s.parse::<usize>() {
        Ok(declared_len) => {
            if declared_len > limit {
                warn!(
                    declared = declared_len,
                    limit = limit,
                    "Client request rejected: Content-Length exceeds allowable ceiling"
                );
                ContentLengthValidation::ExceedsLimit {
                    declared: declared_len,
                    limit,
                }
            } else {
                ContentLengthValidation::Valid(declared_len)
            }
        }
        Err(e) => {
            warn!(error = %e, raw = %s, "Malformed Content-Length header in request");
            ContentLengthValidation::Malformed(format!("Invalid integer: {}", s))
        }
    }
}

/// Checks if actual accumulated byte stream size conforms to declared Content-Length.
pub fn check_stream_size_bounds(
    accumulated_bytes: usize,
    declared_len: Option<usize>,
    max_limit: usize,
) -> Result<(), String> {
    if accumulated_bytes > max_limit {
        return Err(format!(
            "Payload size {} bytes exceeds maximum limit of {} bytes",
            accumulated_bytes, max_limit
        ));
    }

    if let Some(expected) = declared_len {
        if accumulated_bytes > expected {
            return Err(format!(
                "Payload size {} bytes exceeded declared Content-Length of {} bytes",
                accumulated_bytes, expected
            ));
        }
    }

    Ok(())
}
