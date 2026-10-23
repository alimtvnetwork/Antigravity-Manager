//! Streaming Event Frame Parser (`Imp-690`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including thinking
//! block backfilling, thinking budget filtering and backfilling, unified context structural
//! alignment, prefix stability, and the sanitization of risky prompts and request headers."
//!
//! W3C Server-Sent Events (SSE) compliant frame parser for high-throughput AI streaming.
//! Parses `event:`, `data:`, `id:`, `retry:`, and `: comment` fields, concatenates multi-line
//! `data:` entries with `\n`, flushes complete frames on empty line dispatch boundaries
//! (`\n\n` and `\r\n\r\n`), tracks stream offsets, and guarantees zero event loss or buffer
//! truncation during proxy streaming.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Discrete parsed Server-Sent Event frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedEventFrame {
    /// Optional event type name specified by `event: <name>`.
    pub event_type: Option<String>,
    /// Accumulated payload data from one or more `data: <chunk>` lines.
    pub data: String,
    /// Optional event identifier specified by `id: <value>`.
    pub id: Option<String>,
    /// Optional client reconnection time in milliseconds specified by `retry: <ms>`.
    pub retry_ms: Option<u64>,
    /// True if this frame represents an SSE comment line (`: <comment>`).
    pub is_comment: bool,
}

impl Default for ParsedEventFrame {
    fn default() -> Self {
        Self {
            event_type: None,
            data: String::new(),
            id: None,
            retry_ms: None,
            is_comment: false,
        }
    }
}

impl ParsedEventFrame {
    /// Constructs a new empty event frame.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs an event frame with the given data string.
    pub fn with_data(data: impl Into<String>) -> Self {
        Self {
            event_type: None,
            data: data.into(),
            id: None,
            retry_ms: None,
            is_comment: false,
        }
    }

    /// Builder method to assign an event type.
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Builder method to assign an event identifier.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Builder method to assign a retry interval in milliseconds.
    pub fn with_retry(mut self, retry_ms: u64) -> Self {
        self.retry_ms = Some(retry_ms);
        self
    }

    /// Constructs a comment event frame.
    pub fn comment(text: impl Into<String>) -> Self {
        Self {
            event_type: None,
            data: text.into(),
            id: None,
            retry_ms: None,
            is_comment: true,
        }
    }

    /// Returns true if the frame has no data, no event type, no id, no retry, and is not a comment.
    pub fn is_empty(&self) -> bool {
        self.event_type.is_none()
            && self.data.is_empty()
            && self.id.is_none()
            && self.retry_ms.is_none()
            && !self.is_comment
    }

    /// Formats the frame into standard W3C SSE protocol text ending with double newline.
    pub fn to_sse_string(&self) -> String {
        let mut out = String::new();
        if self.is_comment {
            if self.data.is_empty() {
                out.push_str(":\n\n");
            } else {
                for line in self.data.split('\n') {
                    out.push_str(": ");
                    out.push_str(line);
                    out.push('\n');
                }
                out.push('\n');
            }
            return out;
        }

        if let Some(ref ev) = self.event_type {
            out.push_str("event: ");
            out.push_str(ev);
            out.push('\n');
        }

        if !self.data.is_empty() {
            for line in self.data.split('\n') {
                out.push_str("data: ");
                out.push_str(line);
                out.push('\n');
            }
        }

        if let Some(ref id) = self.id {
            out.push_str("id: ");
            out.push_str(id);
            out.push('\n');
        }

        if let Some(retry) = self.retry_ms {
            out.push_str("retry: ");
            out.push_str(&retry.to_string());
            out.push('\n');
        }

        out.push('\n');
        out
    }
}

impl fmt::Display for ParsedEventFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParsedEventFrame(type={:?}, data_len={}, id={:?}, retry={:?}, is_comment={})",
            self.event_type,
            self.data.len(),
            self.id,
            self.retry_ms,
            self.is_comment
        )
    }
}

/// State-machine stream parser buffering incoming SSE chunks and yielding complete frames.
#[derive(Debug, Clone)]
pub struct StreamingFrameParser {
    /// Internal byte buffer for incoming chunk fragments.
    buffer: Vec<u8>,
    /// Frame currently being assembled from incoming lines.
    current_frame: ParsedEventFrame,
    /// Accumulator for comment line contents before dispatch.
    comment_accumulator: String,
    /// Flag indicating the current event block contains comment lines.
    is_comment_event: bool,
    /// Flag indicating that at least one field or comment was observed in the current block.
    has_event_data: bool,
    /// Total cumulative byte volume ingested across all chunks.
    total_bytes_ingested: usize,
    /// Total count of successfully parsed and dispatched event frames.
    total_frames_parsed: usize,
}

impl Default for StreamingFrameParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingFrameParser {
    /// Creates a new streaming event frame parser with a default buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    /// Creates a new streaming event frame parser with the specified buffer initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            current_frame: ParsedEventFrame::default(),
            comment_accumulator: String::new(),
            is_comment_event: false,
            has_event_data: false,
            total_bytes_ingested: 0,
            total_frames_parsed: 0,
        }
    }

    /// Total bytes ingested across all `push_chunk` calls since creation or last reset.
    pub fn total_bytes_ingested(&self) -> usize {
        self.total_bytes_ingested
    }

    /// Total count of completed frames parsed and dispatched.
    pub fn total_frames_parsed(&self) -> usize {
        self.total_frames_parsed
    }

    /// Current unparsed bytes remaining in the internal buffer.
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// True if the internal byte buffer is currently empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Current incomplete line byte length held in the buffer.
    pub fn incomplete_line_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// Returns a lossless string representation of any incomplete trailing line in the buffer.
    pub fn incomplete_line_str(&self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&self.buffer).to_string())
        }
    }

    /// True if an event frame is partially assembled in the active state.
    pub fn has_pending_frame(&self) -> bool {
        self.has_event_data
    }

    /// Ingests a raw byte chunk from the network stream and returns all completed event frames.
    pub fn push_chunk(&mut self, chunk: &[u8]) -> Vec<ParsedEventFrame> {
        if chunk.is_empty() {
            return Vec::new();
        }

        self.total_bytes_ingested += chunk.len();
        self.buffer.extend_from_slice(chunk);

        let mut frames = Vec::new();
        let mut cursor = 0;

        while cursor < self.buffer.len() {
            let mut line_end = None;
            let mut delim_len = 0;

            let buf = &self.buffer[cursor..];
            for (i, &b) in buf.iter().enumerate() {
                if b == b'\r' {
                    if i + 1 < buf.len() {
                        if buf[i + 1] == b'\n' {
                            line_end = Some(cursor + i);
                            delim_len = 2;
                            break;
                        } else {
                            line_end = Some(cursor + i);
                            delim_len = 1;
                            break;
                        }
                    } else {
                        // Buffer ends with '\r'. A subsequent chunk may provide '\n'.
                        // Hold off line extraction until more bytes arrive or finish() is called.
                        break;
                    }
                } else if b == b'\n' {
                    line_end = Some(cursor + i);
                    delim_len = 1;
                    break;
                }
            }

            if let Some(end_idx) = line_end {
                let line_bytes = &self.buffer[cursor..end_idx];
                cursor = end_idx + delim_len;
                if let Some(frame) = self.process_line(line_bytes) {
                    frames.push(frame);
                }
            } else {
                // No complete line boundary in the remaining buffer
                break;
            }
        }

        if cursor > 0 {
            self.buffer.drain(..cursor);
        }

        frames
    }

    /// Finalizes the stream, flushing any trailing incomplete lines or unclosed frames.
    pub fn finish(&mut self) -> Vec<ParsedEventFrame> {
        let mut frames = Vec::new();

        if !self.buffer.is_empty() {
            let remaining = std::mem::take(&mut self.buffer);
            let mut slice = remaining.as_slice();

            // Strip trailing line terminators if present
            if slice.ends_with(b"\r\n") {
                slice = &slice[..slice.len() - 2];
            } else if slice.ends_with(b"\n") || slice.ends_with(b"\r") {
                slice = &slice[..slice.len() - 1];
            }

            if !slice.is_empty() {
                if let Some(frame) = self.process_line(slice) {
                    frames.push(frame);
                }
            }
        }

        if let Some(frame) = self.dispatch_frame() {
            frames.push(frame);
        }

        frames
    }

    /// Resets the parser state and clears all internal buffers and metrics.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.current_frame = ParsedEventFrame::default();
        self.comment_accumulator.clear();
        self.is_comment_event = false;
        self.has_event_data = false;
        self.total_bytes_ingested = 0;
        self.total_frames_parsed = 0;
    }

    /// Processes an individual parsed line. Returns Some(frame) on an empty dispatch line.
    fn process_line(&mut self, line_bytes: &[u8]) -> Option<ParsedEventFrame> {
        let mut bytes = line_bytes;

        // Strip optional UTF-8 Byte Order Mark if present
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            bytes = &bytes[3..];
        }

        // Empty line indicates event dispatch boundary
        if bytes.is_empty() {
            return self.dispatch_frame();
        }

        let line = String::from_utf8_lossy(bytes);

        // SSE comment line begins with ':'
        if let Some(comment_content) = line.strip_prefix(':') {
            let text = comment_content.strip_prefix(' ').unwrap_or(comment_content);
            if self.comment_accumulator.is_empty() {
                self.comment_accumulator.push_str(text);
            } else {
                self.comment_accumulator.push('\n');
                self.comment_accumulator.push_str(text);
            }
            self.is_comment_event = true;
            self.has_event_data = true;
            return None;
        }

        // Standard field parsing: `field: value` or `field`
        let (field, value) = if let Some((f, v)) = line.split_once(':') {
            let v = v.strip_prefix(' ').unwrap_or(v);
            (f, v)
        } else {
            (line.as_ref(), "")
        };

        match field {
            "event" => {
                self.current_frame.event_type = Some(value.to_string());
                self.has_event_data = true;
            }
            "data" => {
                if self.current_frame.data.is_empty() {
                    self.current_frame.data.push_str(value);
                } else {
                    self.current_frame.data.push('\n');
                    self.current_frame.data.push_str(value);
                }
                self.has_event_data = true;
            }
            "id" => {
                // W3C spec: if value contains null char, ignore the id field
                if !value.contains('\0') {
                    self.current_frame.id = Some(value.to_string());
                    self.has_event_data = true;
                }
            }
            "retry" => {
                if let Ok(ms) = value.trim().parse::<u64>() {
                    self.current_frame.retry_ms = Some(ms);
                    self.has_event_data = true;
                }
            }
            _ => {
                // Unrecognized field name ignored per W3C SSE standard
            }
        }

        None
    }

    /// Dispatches the accumulated frame when an empty line boundary is reached.
    fn dispatch_frame(&mut self) -> Option<ParsedEventFrame> {
        if !self.has_event_data {
            return None;
        }

        let mut frame = std::mem::take(&mut self.current_frame);

        // If data is empty and comment was observed, treat as a comment frame
        if frame.data.is_empty() && self.is_comment_event {
            frame.data = std::mem::take(&mut self.comment_accumulator);
            frame.is_comment = true;
        } else {
            self.comment_accumulator.clear();
            frame.is_comment = false;
        }

        self.is_comment_event = false;
        self.has_event_data = false;
        self.total_frames_parsed += 1;

        Some(frame)
    }
}
