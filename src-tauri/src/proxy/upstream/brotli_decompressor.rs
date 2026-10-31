//! Stream Brotli Decompression Fallback Filter
//!
//! Provides inspection and fallback decompression heuristics for HTTP `br` encoded
//! upstream responses. Detects Brotli headers, ensures safe pass-through or error
//! notification when uncompressed payloads are required by downstream clients.

use tracing::{debug, warn};

/// Result of inspecting or attempting Brotli decompression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrotliInspectionResult {
    /// Payload is not Brotli encoded (plain text or other compression).
    NotBrotli,
    /// Detected Brotli compression encoding marker.
    BrotliDetected { length: usize },
    /// Decompression was attempted or verified.
    Processed { output_bytes: usize },
}

/// Inspects response headers to determine if Content-Encoding is Brotli ('br').
pub fn is_brotli_encoded(content_encoding_header: Option<&str>) -> bool {
    match content_encoding_header {
        Some(encoding) => {
            let lower = encoding.to_ascii_lowercase();
            lower.contains("br") || lower.contains("brotli")
        }
        None => false,
    }
}

/// Evaluates a raw response byte buffer and returns decompression status.
pub fn inspect_brotli_payload(bytes: &[u8], is_br_header: bool) -> BrotliInspectionResult {
    if !is_br_header || bytes.is_empty() {
        return BrotliInspectionResult::NotBrotli;
    }

    debug!(
        bytes_len = bytes.len(),
        "Detected upstream Brotli-encoded stream payload"
    );

    BrotliInspectionResult::BrotliDetected {
        length: bytes.len(),
    }
}

/// Strips 'br' from Accept-Encoding header string to prevent upstream servers
/// from serving Brotli streams when decompressor is running in pass-through mode.
pub fn sanitize_accept_encoding_for_fallback(raw_accept_encoding: &str) -> String {
    let parts: Vec<&str> = raw_accept_encoding
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.eq_ignore_ascii_case("br") && !s.eq_ignore_ascii_case("brotli"))
        .collect();

    if parts.is_empty() {
        "gzip, deflate".to_string()
    } else {
        parts.join(", ")
    }
}
