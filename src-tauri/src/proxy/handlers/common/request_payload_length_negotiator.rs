//! Request Payload Length Negotiator (`Imp-615`)
//!
//! Negotiates whether client request payloads should be buffered into memory or
//! streamed directly downstream to upstream providers based on HTTP `Content-Length`,
//! `Transfer-Encoding: chunked`, and safe memory allocation bounds.
//! Bounding memory buffer allocations prevents Out-Of-Memory (OOM) vulnerabilities
//! from unbounded streaming uploads.

use axum::http::header::{CONTENT_LENGTH, TRANSFER_ENCODING};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Default upper ceiling on payload buffer memory allocation (10 MB).
pub const DEFAULT_MAX_BUFFERED_BYTES: usize = 10 * 1024 * 1024;

/// Default threshold above which payloads should be streamed directly rather than buffered (1 MB).
pub const DEFAULT_STREAM_THRESHOLD_BYTES: usize = 1024 * 1024;

/// Transfer encoding mode inferred from client request headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferEncodingMode {
    /// Explicit Content-Length header with known payload size in bytes.
    FixedContentLength(u64),
    /// Chunked transfer encoding without fixed prior size.
    ChunkedTransfer,
    /// Neither Content-Length nor chunked Transfer-Encoding is specified.
    AbsentLength,
}

impl TransferEncodingMode {
    /// Infers `TransferEncodingMode` from optional `content_length` and `is_chunked` flag.
    pub fn from_length_and_chunked(content_length: Option<u64>, is_chunked: bool) -> Self {
        if is_chunked {
            Self::ChunkedTransfer
        } else {
            match content_length {
                Some(len) => Self::FixedContentLength(len),
                None => Self::AbsentLength,
            }
        }
    }

    /// Infers `TransferEncodingMode` by inspecting HTTP headers.
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let is_chunked = headers
            .get(TRANSFER_ENCODING)
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                s.split(',')
                    .any(|part| part.trim().eq_ignore_ascii_case("chunked"))
            })
            .unwrap_or(false);

        if is_chunked {
            return Self::ChunkedTransfer;
        }

        let content_length = headers
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok());

        match content_length {
            Some(len) => Self::FixedContentLength(len),
            None => Self::AbsentLength,
        }
    }
}

/// Configuration bounds for request payload buffering vs streaming negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadNegotiationConfig {
    /// Maximum allowable bytes that may be buffered into memory (default: 10 MB).
    pub max_buffered_bytes: usize,
    /// Threshold size above which payload is streamed directly (default: 1 MB).
    pub stream_threshold_bytes: usize,
}

impl Default for PayloadNegotiationConfig {
    fn default() -> Self {
        Self {
            max_buffered_bytes: DEFAULT_MAX_BUFFERED_BYTES,
            stream_threshold_bytes: DEFAULT_STREAM_THRESHOLD_BYTES,
        }
    }
}

impl PayloadNegotiationConfig {
    pub fn new(max_buffered_bytes: usize, stream_threshold_bytes: usize) -> Self {
        Self {
            max_buffered_bytes,
            stream_threshold_bytes,
        }
    }
}

/// Result decision determining how the incoming request body should be processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadLengthDecision {
    /// Safe to buffer entire payload in memory with reserved capacity.
    BufferInMemory(usize),
    /// Payload exceeds buffer threshold or is chunked streaming; must be forwarded directly.
    StreamDirectly,
    /// Payload exceeds maximum allowable ceiling and must be rejected with HTTP 413.
    /// Format: `RejectOversized(declared_or_actual_bytes, max_allowed_bytes)`
    RejectOversized(u64, u64),
}

impl PayloadLengthDecision {
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::RejectOversized(..))
    }

    pub fn is_stream(&self) -> bool {
        matches!(self, Self::StreamDirectly)
    }

    pub fn is_buffered(&self) -> bool {
        matches!(self, Self::BufferInMemory(_))
    }
}

/// Detailed audit report detailing the payload negotiation rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiationReport {
    pub decision: PayloadLengthDecision,
    pub transfer_mode: TransferEncodingMode,
    pub reason: String,
}

/// Negotiates the payload strategy given an explicit or absent content_length,
/// chunked transfer indicator, and negotiation configuration bounds.
pub fn negotiate_payload_strategy(
    content_length: Option<u64>,
    is_chunked: bool,
    config: &PayloadNegotiationConfig,
) -> PayloadLengthDecision {
    let mode = TransferEncodingMode::from_length_and_chunked(content_length, is_chunked);
    match mode {
        TransferEncodingMode::ChunkedTransfer => {
            // Chunked transfers have indefinite size upfront; stream directly to avoid OOM.
            debug!("Chunked transfer encoding detected: streaming directly");
            PayloadLengthDecision::StreamDirectly
        }
        TransferEncodingMode::AbsentLength => {
            // Absent Content-Length without chunked transfer: stream directly to prevent unbounded buffering.
            debug!("Absent Content-Length detected: streaming directly");
            PayloadLengthDecision::StreamDirectly
        }
        TransferEncodingMode::FixedContentLength(len) => {
            let max_buffered = config.max_buffered_bytes as u64;
            let stream_threshold = config.stream_threshold_bytes as u64;

            if len > max_buffered {
                warn!(
                    declared = len,
                    max_allowed = max_buffered,
                    "Payload exceeds allowable ceiling; rejecting oversized request"
                );
                PayloadLengthDecision::RejectOversized(len, max_buffered)
            } else if len > stream_threshold {
                debug!(
                    declared = len,
                    stream_threshold, "Payload exceeds stream threshold; streaming directly"
                );
                PayloadLengthDecision::StreamDirectly
            } else {
                debug!(
                    declared = len,
                    "Payload within stream threshold; buffering in memory"
                );
                PayloadLengthDecision::BufferInMemory(len as usize)
            }
        }
    }
}

/// Helper to negotiate payload strategy directly from HTTP request HeaderMap.
pub fn negotiate_payload_strategy_from_headers(
    headers: &HeaderMap,
    config: &PayloadNegotiationConfig,
) -> PayloadLengthDecision {
    let mode = TransferEncodingMode::from_headers(headers);
    let (content_length, is_chunked) = match mode {
        TransferEncodingMode::FixedContentLength(len) => (Some(len), false),
        TransferEncodingMode::ChunkedTransfer => (None, true),
        TransferEncodingMode::AbsentLength => (None, false),
    };
    negotiate_payload_strategy(content_length, is_chunked, config)
}

/// Produces a comprehensive `NegotiationReport` including audit explanation.
pub fn negotiate_payload_with_report(
    content_length: Option<u64>,
    is_chunked: bool,
    config: &PayloadNegotiationConfig,
) -> NegotiationReport {
    let mode = TransferEncodingMode::from_length_and_chunked(content_length, is_chunked);
    let decision = negotiate_payload_strategy(content_length, is_chunked, config);
    let reason = match decision {
        PayloadLengthDecision::BufferInMemory(size) => {
            format!(
                "Payload size {} bytes is within stream threshold ({} bytes); buffering in memory",
                size, config.stream_threshold_bytes
            )
        }
        PayloadLengthDecision::StreamDirectly => match mode {
            TransferEncodingMode::ChunkedTransfer => {
                "Chunked transfer encoding stream: streaming directly to prevent memory exhaustion"
                    .to_string()
            }
            TransferEncodingMode::AbsentLength => {
                "Length header absent: streaming directly without preallocated memory buffer"
                    .to_string()
            }
            TransferEncodingMode::FixedContentLength(len) => {
                format!(
                    "Payload size {} bytes exceeds stream threshold {} bytes; streaming directly",
                    len, config.stream_threshold_bytes
                )
            }
        },
        PayloadLengthDecision::RejectOversized(size, limit) => {
            format!(
                "Payload size {} bytes exceeds allowable ceiling of {} bytes",
                size, limit
            )
        }
    };

    NegotiationReport {
        decision,
        transfer_mode: mode,
        reason,
    }
}
