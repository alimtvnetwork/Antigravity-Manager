//! Compression & Content-Encoding Guard
//! Manages Content-Encoding negotiation and prevents double-decompression bugs
//! when edge proxies return compressed payloads.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    Gzip,
    Deflate,
    Brotli,
    Identity,
}

impl ContentEncoding {
    pub fn from_header(header_val: &str) -> Self {
        let val_lower = header_val.to_ascii_lowercase();
        if val_lower.contains("gzip") {
            ContentEncoding::Gzip
        } else if val_lower.contains("br") {
            ContentEncoding::Brotli
        } else if val_lower.contains("deflate") {
            ContentEncoding::Deflate
        } else {
            ContentEncoding::Identity
        }
    }

    pub fn is_compressed(&self) -> bool {
        !matches!(self, ContentEncoding::Identity)
    }
}

pub struct CompressionGuard;

impl CompressionGuard {
    /// Determines whether Content-Encoding header should be stripped
    /// when the body has already been decompressed by the HTTP client layer.
    pub fn should_strip_encoding_header(already_decompressed: bool) -> bool {
        already_decompressed
    }
}
