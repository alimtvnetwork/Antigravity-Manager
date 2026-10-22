//! Compression negotiator for upstream responses and client streaming payloads.
//! Determines whether streaming responses should preserve identity encoding
//! or negotiate gzip/brotli based on client headers and payload size.

use serde::{Deserialize, Serialize};

/// Compression format selected for the response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectedEncoding {
    Identity,
    Gzip,
    Brotli,
    Deflate,
}

impl SelectedEncoding {
    pub fn as_header_str(&self) -> Option<&'static str> {
        match self {
            SelectedEncoding::Identity => None,
            SelectedEncoding::Gzip => Some("gzip"),
            SelectedEncoding::Brotli => Some("br"),
            SelectedEncoding::Deflate => Some("deflate"),
        }
    }
}

/// Decision result produced by the compression negotiator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionDecision {
    pub encoding: SelectedEncoding,
    pub should_compress: bool,
    pub reason: String,
}

/// Evaluates client headers and payload characteristics to determine compression format.
pub struct CompressionNegotiator;

impl CompressionNegotiator {
    /// Determines the best compression format.
    /// Streaming SSE connections should default to Identity unless client explicitly demands compression
    /// to avoid chunk buffer lag in frontend consumers.
    pub fn negotiate(
        accept_encoding: Option<&str>,
        is_streaming: bool,
        content_length_hint: Option<usize>,
    ) -> CompressionDecision {
        // Minimum payload size for compression efficiency (1 KB)
        const MIN_COMPRESSION_BYTES: usize = 1024;

        if is_streaming {
            // For SSE streams, prefer identity to minimize latency and chunk buffering
            return CompressionDecision {
                encoding: SelectedEncoding::Identity,
                should_compress: false,
                reason: "SSE streaming prefers uncompressed identity frames for real-time delivery"
                    .to_string(),
            };
        }

        if let Some(len) = content_length_hint {
            if len < MIN_COMPRESSION_BYTES {
                return CompressionDecision {
                    encoding: SelectedEncoding::Identity,
                    should_compress: false,
                    reason: format!("Payload size {}B is below compression threshold", len),
                };
            }
        }

        let header = match accept_encoding {
            Some(h) if !h.trim().is_empty() => h.to_ascii_lowercase(),
            _ => {
                return CompressionDecision {
                    encoding: SelectedEncoding::Identity,
                    should_compress: false,
                    reason: "No Accept-Encoding header provided".to_string(),
                };
            }
        };

        // Prioritize Brotli, then Gzip, then Deflate
        if header.contains("br") {
            CompressionDecision {
                encoding: SelectedEncoding::Brotli,
                should_compress: true,
                reason: "Client supports Brotli (br)".to_string(),
            }
        } else if header.contains("gzip") {
            CompressionDecision {
                encoding: SelectedEncoding::Gzip,
                should_compress: true,
                reason: "Client supports Gzip".to_string(),
            }
        } else if header.contains("deflate") {
            CompressionDecision {
                encoding: SelectedEncoding::Deflate,
                should_compress: true,
                reason: "Client supports Deflate".to_string(),
            }
        } else {
            CompressionDecision {
                encoding: SelectedEncoding::Identity,
                should_compress: false,
                reason: "No matching supported compression algorithm".to_string(),
            }
        }
    }
}
