//! SSE Chunk Stream Boundary Validator
//!
//! Inspects raw streaming SSE chunks and validates event frame boundary integrity.
//! Identifies split line-breaks, unclosed data envelopes, and invalid UTF-8 trailing slices
//! before chunks are forwarded to client response pipelines.

use tracing::warn;

/// Result of evaluating a streaming chunk's boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundaryValidationResult {
    /// Chunk contains complete SSE event line(s) ending with newline.
    CompleteLines,
    /// Chunk ends mid-line (needs buffering with next chunk).
    IncompleteLine,
    /// Chunk contains an invalid or incomplete UTF-8 sequence at the boundary.
    IncompleteUtf8 { valid_up_to: usize },
    /// Chunk is empty.
    Empty,
}

/// Validates chunk boundaries against UTF-8 and line-break constraints.
pub fn validate_chunk_boundary(chunk: &[u8]) -> BoundaryValidationResult {
    if chunk.is_empty() {
        return BoundaryValidationResult::Empty;
    }

    // Step 1: Validate UTF-8 boundary
    match std::str::from_utf8(chunk) {
        Ok(text) => {
            // Step 2: Check if chunk concludes with newline
            if text.ends_with('\n') {
                BoundaryValidationResult::CompleteLines
            } else {
                BoundaryValidationResult::IncompleteLine
            }
        }
        Err(e) => {
            let valid_up_to = e.valid_up_to();
            warn!(
                valid_bytes = valid_up_to,
                total_bytes = chunk.len(),
                "Chunk boundary split multi-byte UTF-8 character"
            );
            BoundaryValidationResult::IncompleteUtf8 { valid_up_to }
        }
    }
}

/// Splits a raw byte buffer at the last safe UTF-8 boundary if it was split mid-character.
pub fn split_at_safe_utf8_boundary(chunk: &[u8]) -> (&[u8], &[u8]) {
    match std::str::from_utf8(chunk) {
        Ok(_) => (chunk, &[]),
        Err(e) => {
            let valid_len = e.valid_up_to();
            (&chunk[..valid_len], &chunk[valid_len..])
        }
    }
}
