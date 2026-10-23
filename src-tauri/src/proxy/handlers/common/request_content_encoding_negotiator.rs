//! Request Content-Encoding Negotiator
//!
//! Validates and negotiates client HTTP request `Content-Encoding` headers, verifying that
//! incoming compressed payloads (gzip, deflate) are recognized, decompressible, or converted
//! before hitting upstream API endpoints.

use axum::http::HeaderMap;
use tracing::debug;

/// Supported content encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedEncoding {
    Identity,
    Gzip,
    Deflate,
    Unsupported,
}

/// Negotiates and identifies the content encoding from incoming request headers.
pub fn negotiate_content_encoding(headers: &HeaderMap) -> SupportedEncoding {
    let encoding_hdr = headers
        .get(axum::http::header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok());

    match encoding_hdr {
        Some(s) => {
            let lower = s.to_ascii_lowercase();
            if lower.contains("gzip") {
                SupportedEncoding::Gzip
            } else if lower.contains("deflate") {
                SupportedEncoding::Deflate
            } else if lower.contains("identity") || lower.is_empty() {
                SupportedEncoding::Identity
            } else {
                SupportedEncoding::Unsupported
            }
        }
        None => SupportedEncoding::Identity,
    }
}

/// Checks if an encoding is supported for transparent server-side decompression.
pub fn is_encoding_supported(encoding: SupportedEncoding) -> bool {
    matches!(
        encoding,
        SupportedEncoding::Identity | SupportedEncoding::Gzip | SupportedEncoding::Deflate
    )
}
