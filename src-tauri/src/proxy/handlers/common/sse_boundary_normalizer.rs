//! SSE boundary delimiter normalizer.
//! Cleanses and reformats outgoing SSE events to ensure strict RFC-compliant
//! formatting with uniform Unix line endings (`\n`) and clean trailing double-newlines (`\n\n`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSseFrame {
    pub raw_bytes_len: usize,
    pub payload: String,
    pub is_terminal: bool,
}

pub struct SseBoundaryNormalizer;

impl SseBoundaryNormalizer {
    /// Formats a raw SSE payload string into a canonical SSE frame ending with `\n\n`.
    /// Normalizes Windows CRLF (`\r\n`) to LF (`\n`) and strips extraneous trailing whitespace.
    pub fn normalize_frame(event_name: Option<&str>, data: &str) -> String {
        let clean_data = data.replace("\r\n", "\n").replace('\r', "\n");
        let mut frame = String::with_capacity(clean_data.len() + 32);

        if let Some(event) = event_name {
            let clean_event = event.trim();
            if !clean_event.is_empty() {
                frame.push_str("event: ");
                frame.push_str(clean_event);
                frame.push('\n');
            }
        }

        // Multi-line data handling: each line must start with "data: "
        for line in clean_data.lines() {
            frame.push_str("data: ");
            frame.push_str(line);
            frame.push('\n');
        }

        // Terminal delimiter for the SSE event
        frame.push('\n');
        frame
    }

    /// Checks if the payload represents a terminal stream marker (`[DONE]`).
    pub fn is_done_marker(data: &str) -> bool {
        data.trim() == "[DONE]" || data.contains("\"[DONE]\"")
    }
}
