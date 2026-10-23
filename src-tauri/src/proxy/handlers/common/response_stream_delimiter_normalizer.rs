//! Multi-Protocol Response Stream Delimiter Normalizer (Imp-633).
//!
//! Provides protocol-agnostic SSE delimiter normalization across OpenAI, Claude,
//! and Gemini protocols per AGENTS.md:
//! - Normalizes CRLF delimiters (`\r\n\r\n`) to canonical LF (`\n\n`).
//! - Reassembles split SSE frames across chunk boundaries.
//! - Strips malformed trailing delimiters and preserves unclosed fragments.

use serde::{Deserialize, Serialize};

/// Represents an individual normalized Server-Sent Events (SSE) frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamFrame {
    pub event_type: Option<String>,
    pub data_payload: String,
    pub raw_bytes_count: usize,
    pub is_terminal: bool,
}

impl StreamFrame {
    /// Constructs a new normalized stream frame.
    pub fn new(
        event_type: Option<String>,
        data_payload: String,
        raw_bytes_count: usize,
        is_terminal: bool,
    ) -> Self {
        Self {
            event_type,
            data_payload,
            raw_bytes_count,
            is_terminal,
        }
    }

    /// Returns true if this frame marks stream completion.
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Returns true if this frame carries a specific event type.
    pub fn has_event_type(&self) -> bool {
        self.event_type.is_some()
    }

    /// Returns the optional event type string slice.
    pub fn event_type(&self) -> Option<&str> {
        self.event_type.as_deref()
    }

    /// Returns the payload content string slice.
    pub fn data_payload(&self) -> &str {
        &self.data_payload
    }

    /// Returns the raw byte count of the frame prior to normalization.
    pub fn raw_bytes_count(&self) -> usize {
        self.raw_bytes_count
    }
}

/// Normalizes mixed stream delimiters, reassembles split chunks, and extracts frames.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StreamDelimiterNormalizer {
    pub buffer: Vec<u8>,
    pub frames_normalized_count: usize,
}

impl StreamDelimiterNormalizer {
    /// Creates a new normalizer instance with empty carryover buffer.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            frames_normalized_count: 0,
        }
    }

    /// Returns true if there are unclosed frame bytes remaining in the buffer.
    pub fn has_pending_bytes(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Returns the current count of buffered pending bytes.
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the total count of normalized frames produced so far.
    pub fn frames_normalized_count(&self) -> usize {
        self.frames_normalized_count
    }

    /// Resets the normalizer buffer and frame counters to initial state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.frames_normalized_count = 0;
    }

    /// Feeds an incoming byte chunk, normalizing delimiters and returning completed frames.
    pub fn feed_chunk(&mut self, chunk: &[u8]) -> Vec<StreamFrame> {
        self.buffer.extend_from_slice(chunk);
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let (text, incomplete_tail) = self.decode_valid_utf8_prefix();
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");

        if !normalized.contains("\n\n") {
            return Vec::new();
        }

        self.extract_frames_from_normalized(&normalized, incomplete_tail)
    }

    /// Flushes any pending un-delimited bytes as a final frame if non-empty.
    pub fn flush_remaining(&mut self) -> Option<StreamFrame> {
        if self.buffer.is_empty() {
            return None;
        }

        let text = String::from_utf8_lossy(&self.buffer).to_string();
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        self.buffer.clear();

        let trimmed = normalized.trim();
        if trimmed.is_empty() {
            return None;
        }

        let frame = Self::parse_frame_block(trimmed, trimmed.as_bytes().len())?;
        self.frames_normalized_count += 1;
        Some(frame)
    }

    /// Decodes valid UTF-8 prefix from buffer and separates any trailing partial UTF-8 bytes.
    fn decode_valid_utf8_prefix(&self) -> (String, Vec<u8>) {
        match std::str::from_utf8(&self.buffer) {
            Ok(valid_str) => (valid_str.to_string(), Vec::new()),
            Err(e) if e.error_len().is_none() => {
                let valid_up_to = e.valid_up_to();
                let valid_str = String::from_utf8_lossy(&self.buffer[..valid_up_to]).to_string();
                let tail = self.buffer[valid_up_to..].to_vec();
                (valid_str, tail)
            }
            Err(_) => (
                String::from_utf8_lossy(&self.buffer).to_string(),
                Vec::new(),
            ),
        }
    }

    /// Extracts completed SSE frames from normalized double-newline text.
    fn extract_frames_from_normalized(
        &mut self,
        normalized: &str,
        incomplete_tail: Vec<u8>,
    ) -> Vec<StreamFrame> {
        let mut frames = Vec::new();
        let parts: Vec<&str> = normalized.split("\n\n").collect();
        let is_trailing_delimited = normalized.ends_with("\n\n");

        let (completed, remainder) = if is_trailing_delimited {
            (&parts[..parts.len() - 1], "")
        } else {
            (&parts[..parts.len() - 1], parts[parts.len() - 1])
        };

        for part in completed {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(frame) = Self::parse_frame_block(trimmed, part.as_bytes().len()) {
                self.frames_normalized_count += 1;
                frames.push(frame);
            }
        }

        let mut next_buffer = remainder.as_bytes().to_vec();
        next_buffer.extend_from_slice(&incomplete_tail);
        self.buffer = next_buffer;

        frames
    }

    /// Parses a single block into a `StreamFrame`, extracting event and data fields.
    fn parse_frame_block(block_text: &str, raw_bytes_count: usize) -> Option<StreamFrame> {
        let mut event_type: Option<String> = None;
        let mut data_lines: Vec<String> = Vec::new();

        for line in block_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }

            if let Some(event_val) = line.strip_prefix("event:") {
                event_type = Some(event_val.trim().to_string());
            } else if let Some(data_val) = line.strip_prefix("data:") {
                let stripped = data_val.strip_prefix(' ').unwrap_or(data_val);
                data_lines.push(stripped.to_string());
            } else {
                data_lines.push(line.to_string());
            }
        }

        if data_lines.is_empty() && event_type.is_none() {
            return None;
        }

        let data_payload = data_lines.join("\n");
        let is_terminal = Self::check_is_terminal(&data_payload, event_type.as_deref());

        Some(StreamFrame {
            event_type,
            data_payload,
            raw_bytes_count,
            is_terminal,
        })
    }

    /// Determines whether a frame represents a stream-terminating sentinel.
    fn check_is_terminal(data_payload: &str, event_type: Option<&str>) -> bool {
        data_payload.trim() == "[DONE]"
            || matches!(event_type, Some("done") | Some("finish") | Some("end"))
    }
}
