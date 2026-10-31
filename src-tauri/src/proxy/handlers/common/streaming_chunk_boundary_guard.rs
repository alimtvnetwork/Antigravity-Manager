//! Streaming Chunk Boundary Guard (`Imp-682`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including thinking
//! block backfilling, thinking budget filtering and backfilling, unified context structural
//! alignment, prefix stability, and the sanitization of risky prompts and request headers."
//!
//! Enforces strict frame capacity ceilings (1MB default) and frame delimiter validation across
//! Server-Sent Event (SSE) streaming chunks, preventing unbounded memory accumulation and buffer
//! exhaustion during prolonged token generation turns.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Default maximum allowable byte size for an individual raw streaming chunk (1 MB).
pub const DEFAULT_MAX_CHUNK_BYTES: usize = 1024 * 1024;

/// SSE frame boundary delimiter tokens.
pub const SSE_DELIMITER_LF: &[u8] = b"\n\n";
pub const SSE_DELIMITER_CRLF: &[u8] = b"\r\n\r\n";

/// Configuration options governing streaming chunk capacity and framing rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkBoundaryConfig {
    /// Maximum permitted byte length for a single raw SSE chunk.
    pub max_chunk_bytes: usize,
    /// Whether an incomplete partial frame is permitted at the stream tail.
    pub allow_incomplete_tail: bool,
}

impl Default for ChunkBoundaryConfig {
    fn default() -> Self {
        Self {
            max_chunk_bytes: DEFAULT_MAX_CHUNK_BYTES,
            allow_incomplete_tail: true,
        }
    }
}

/// Violations detected when inspecting streaming chunk boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkBoundaryViolation {
    /// A single raw chunk exceeds the configured maximum byte ceiling.
    OversizedChunk {
        chunk_len: usize,
        max_allowed: usize,
    },
    /// Chunk data is completely empty.
    EmptyChunk,
    /// Terminal stream chunk ended abruptly without proper SSE boundary completion.
    UnclosedStreamFrame { trailing_bytes: usize },
}

impl fmt::Display for ChunkBoundaryViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OversizedChunk { chunk_len, max_allowed } => write!(
                f,
                "Oversized streaming chunk detected: {chunk_len} bytes exceeds max allowed {max_allowed} bytes"
            ),
            Self::EmptyChunk => write!(f, "Encountered zero-byte empty streaming chunk"),
            Self::UnclosedStreamFrame { trailing_bytes } => write!(
                f,
                "Stream terminated with {trailing_bytes} unclosed dangling bytes missing SSE delimiter"
            ),
        }
    }
}

/// Audit report produced after verifying streaming chunk boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkBoundaryReport {
    /// True if all inspected chunks satisfied capacity and framing constraints.
    pub is_valid: bool,
    /// Total count of streaming chunks inspected.
    pub total_chunks_count: usize,
    /// Total accumulated byte volume across all chunks.
    pub total_bytes_accumulated: usize,
    /// Count of fully completed `\n\n` or `\r\n\r\n` SSE frames.
    pub completed_frames_count: usize,
    /// Granular list of detected boundary violations.
    pub violations: Vec<ChunkBoundaryViolation>,
}

/// Checks if a slice of bytes contains a compliant SSE frame delimiter.
pub fn contains_sse_frame_delimiter(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|w| w == b"\n\n") || bytes.windows(4).any(|w| w == b"\r\n\r\n")
}

/// Validates an individual streaming chunk against boundary capacity constraints.
pub fn validate_streaming_chunk(
    chunk: &[u8],
    config: &ChunkBoundaryConfig,
) -> Result<(), ChunkBoundaryViolation> {
    if chunk.is_empty() {
        return Err(ChunkBoundaryViolation::EmptyChunk);
    }

    if chunk.len() > config.max_chunk_bytes {
        return Err(ChunkBoundaryViolation::OversizedChunk {
            chunk_len: chunk.len(),
            max_allowed: config.max_chunk_bytes,
        });
    }

    Ok(())
}

/// Audits a sequence of streaming chunks forming a complete or ongoing response stream.
pub fn audit_streaming_stream(
    chunks: &[&[u8]],
    config: &ChunkBoundaryConfig,
) -> ChunkBoundaryReport {
    let mut violations = Vec::new();
    let mut total_bytes = 0;
    let mut completed_frames_count = 0;

    for chunk in chunks {
        total_bytes += chunk.len();

        if let Err(violation) = validate_streaming_chunk(chunk, config) {
            violations.push(violation);
        }

        if contains_sse_frame_delimiter(chunk) {
            completed_frames_count += 1;
        }
    }

    // Check if the stream tail ended without completing its frame when required
    if !config.allow_incomplete_tail {
        if let Some(last_chunk) = chunks.last() {
            if !last_chunk.ends_with(b"\n\n") && !last_chunk.ends_with(b"\r\n\r\n") {
                violations.push(ChunkBoundaryViolation::UnclosedStreamFrame {
                    trailing_bytes: last_chunk.len(),
                });
            }
        }
    }

    ChunkBoundaryReport {
        is_valid: violations.is_empty(),
        total_chunks_count: chunks.len(),
        total_bytes_accumulated: total_bytes,
        completed_frames_count,
        violations,
    }
}
