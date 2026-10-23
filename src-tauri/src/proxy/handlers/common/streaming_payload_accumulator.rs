//! Streaming Payload Accumulator (`Imp-695`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including thinking
//! block backfilling, thinking budget filtering and backfilling, unified context structural
//! alignment, prefix stability, and the sanitization of risky prompts and request headers."
//!
//! Accumulates and validates streaming response chunks into cohesive Gemini candidate turns,
//! tracking token boundaries, thinking block delimiters (`<thought>...</thought>`), and tool
//! invocation fragments, bounding maximum stream buffer size to prevent memory bloat while
//! ensuring complete turn delivery.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Default maximum allowable byte size for accumulated turn state buffer (10 MB).
pub const DEFAULT_MAX_BUFFER_BYTES: usize = 10 * 1024 * 1024;

/// Default maximum chunk count for a single generation turn.
pub const DEFAULT_MAX_CHUNKS: usize = 100_000;

/// Opening tag for model thinking blocks.
pub const THOUGHT_TAG_OPEN: &str = "<thought>";
/// Closing tag for model thinking blocks.
pub const THOUGHT_TAG_CLOSE: &str = "</thought>";

/// Individual accumulated part within a candidate turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccumulatedPart {
    /// Standard model generated text content.
    Text(String),
    /// Isolated model internal chain-of-thought block.
    Thought(String),
    /// Model tool or function call invocation with streaming arguments.
    FunctionCall {
        name: String,
        args: String,
        is_complete: bool,
    },
    /// Inlined binary or multimodal payload data.
    InlineData {
        mime_type: String,
        data: String,
    },
}

impl AccumulatedPart {
    /// Returns true if this part is standard text content.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns true if this part is a thinking block.
    pub fn is_thought(&self) -> bool {
        matches!(self, Self::Thought(_))
    }

    /// Returns true if this part is a function call.
    pub fn is_function_call(&self) -> bool {
        matches!(self, Self::FunctionCall { .. })
    }

    /// Returns true if this part is inline data.
    pub fn is_inline_data(&self) -> bool {
        matches!(self, Self::InlineData { .. })
    }

    /// Returns the text slice if this part is standard text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the thought slice if this part is a thinking block.
    pub fn as_thought(&self) -> Option<&str> {
        match self {
            Self::Thought(s) => Some(s),
            _ => None,
        }
    }
}

/// Structured state of an accumulated candidate turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateTurnState {
    /// Author role of the candidate turn (e.g., "model", "assistant").
    pub role: String,
    /// Ordered parts accumulated within this turn.
    pub parts: Vec<AccumulatedPart>,
    /// Turn termination or stop reason when finished.
    pub finish_reason: Option<String>,
    /// Total bytes buffered across all parts in this turn.
    pub total_bytes: usize,
    /// True if the turn has reached terminal completion.
    pub is_turn_complete: bool,
}

impl CandidateTurnState {
    /// Constructs a new empty turn state for the given role.
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            parts: Vec::new(),
            finish_reason: None,
            total_bytes: 0,
            is_turn_complete: false,
        }
    }

    /// Constructs a default turn state for the "model" role.
    pub fn default_model() -> Self {
        Self::new("model")
    }

    /// Returns the turn role.
    pub fn role(&self) -> &str {
        &self.role
    }

    /// Returns a slice of the accumulated parts.
    pub fn parts(&self) -> &[AccumulatedPart] {
        &self.parts
    }

    /// Returns the finish reason if set.
    pub fn finish_reason(&self) -> Option<&str> {
        self.finish_reason.as_deref()
    }

    /// Returns total bytes accumulated in this turn.
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Returns true if this turn has been marked complete.
    pub fn is_turn_complete(&self) -> bool {
        self.is_turn_complete
    }

    /// Concatenates and returns all text parts in sequence.
    pub fn full_text(&self) -> String {
        let mut out = String::new();
        for part in &self.parts {
            if let AccumulatedPart::Text(s) = part {
                out.push_str(s);
            }
        }
        out
    }

    /// Concatenates and returns all thought parts in sequence.
    pub fn full_thought(&self) -> String {
        let mut out = String::new();
        for part in &self.parts {
            if let AccumulatedPart::Thought(s) = part {
                out.push_str(s);
            }
        }
        out
    }

    /// True if any thought parts exist in this turn.
    pub fn has_thought(&self) -> bool {
        self.parts.iter().any(|p| p.is_thought())
    }

    /// True if any function call parts exist in this turn.
    pub fn has_function_calls(&self) -> bool {
        self.parts.iter().any(|p| p.is_function_call())
    }

    /// Count of parts accumulated in this turn.
    pub fn parts_count(&self) -> usize {
        self.parts.len()
    }

    /// True if no parts have been accumulated.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

/// Configuration governing streaming accumulator behavior and limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamingAccumulatorConfig {
    /// Maximum allowed buffer byte capacity before rejecting chunks (default 10 MB).
    pub max_buffer_bytes: usize,
    /// Maximum allowable chunk count before aborting turn accumulation.
    pub max_chunks: usize,
    /// When true, extracts `<thought>...</thought>` delimiters into dedicated `Thought` parts.
    pub detect_thought_tags: bool,
    /// When true, attempts auto-repair of unclosed JSON arguments on turn finish.
    pub auto_repair_json: bool,
}

impl Default for StreamingAccumulatorConfig {
    fn default() -> Self {
        Self {
            max_buffer_bytes: DEFAULT_MAX_BUFFER_BYTES,
            max_chunks: DEFAULT_MAX_CHUNKS,
            detect_thought_tags: true,
            auto_repair_json: true,
        }
    }
}

impl StreamingAccumulatorConfig {
    /// Creates a custom configuration.
    pub fn new(max_buffer_bytes: usize, detect_thought_tags: bool, auto_repair_json: bool) -> Self {
        Self {
            max_buffer_bytes,
            max_chunks: DEFAULT_MAX_CHUNKS,
            detect_thought_tags,
            auto_repair_json,
        }
    }

    /// Constructs a configuration with max buffer bytes and chunk ceiling.
    pub fn with_limits(max_buffer_bytes: usize, max_chunks: usize) -> Self {
        Self {
            max_buffer_bytes,
            max_chunks,
            detect_thought_tags: true,
            auto_repair_json: true,
        }
    }

    /// Fluent builder method to set maximum buffer capacity.
    pub fn with_max_buffer_bytes(mut self, bytes: usize) -> Self {
        self.max_buffer_bytes = bytes;
        self
    }

    /// Fluent builder method to set maximum chunk ceiling.
    pub fn with_max_chunks(mut self, chunks: usize) -> Self {
        self.max_chunks = chunks;
        self
    }

    /// Fluent builder method to enable or disable thought tag detection.
    pub fn with_detect_thought_tags(mut self, detect: bool) -> Self {
        self.detect_thought_tags = detect;
        self
    }

    /// Fluent builder method to enable or disable JSON argument auto-repair.
    pub fn with_auto_repair_json(mut self, auto_repair: bool) -> Self {
        self.auto_repair_json = auto_repair;
        self
    }
}

/// Backward compatibility alias for payload configuration.
pub type PayloadAccumulatorConfig = StreamingAccumulatorConfig;

/// Action classification resulting from chunk ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccumulatorAction {
    /// Content was appended to an existing or new open part.
    Appended,
    /// An individual part (e.g. thought block or complete function call) was completed.
    PartCompleted,
    /// The entire candidate turn has terminated.
    TurnFinished,
}

impl fmt::Display for AccumulatorAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Appended => write!(f, "Appended"),
            Self::PartCompleted => write!(f, "PartCompleted"),
            Self::TurnFinished => write!(f, "TurnFinished"),
        }
    }
}

/// Result returned after ingesting a streaming chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccumulatorChunkResult {
    /// Ingestion action summary.
    pub action: AccumulatorAction,
    /// Current snapshot of turn state.
    pub current_turn: CandidateTurnState,
    /// Current total bytes buffered in turn.
    pub bytes_buffered: usize,
}

/// An individual accumulated tool call with reconstructed arguments (backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccumulatedToolCall {
    /// Unique tool call identifier.
    pub id: String,
    /// Function or tool name.
    pub name: String,
    /// Fully accumulated JSON arguments string.
    pub arguments: String,
}

impl AccumulatedToolCall {
    /// Constructs a new tool call record.
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

/// A complete candidate turn aggregated across streaming response chunks (backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccumulatedCandidateTurn {
    /// Final assembled textual response (with `<thought>` tags cleanly extracted).
    pub text: String,
    /// Extracted or backfilled thinking block, if present.
    pub thought: Option<String>,
    /// Accumulated tool call invocations.
    pub tool_calls: Vec<AccumulatedToolCall>,
    /// Generation termination reason (e.g. "STOP", "MAX_TOKENS", "tool_calls").
    pub finish_reason: Option<String>,
    /// Total number of streaming chunks ingested.
    pub total_chunks: usize,
    /// Total byte length of all ingested chunks.
    pub total_bytes: usize,
}

impl AccumulatedCandidateTurn {
    /// Returns true if the turn produced non-empty text, a thought, or at least one tool call.
    pub fn has_content(&self) -> bool {
        !self.text.is_empty() || self.thought.is_some() || !self.tool_calls.is_empty()
    }
}

/// Errors returned during streaming chunk ingestion (backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccumulatorError {
    /// Ingested payload exceeded configured byte ceiling.
    BufferExceeded {
        current_bytes: usize,
        max_bytes: usize,
    },
    /// Stream exceeded configured maximum chunk limit.
    ChunkLimitExceeded {
        chunk_count: usize,
        max_chunks: usize,
    },
    /// A structured chunk failed to parse as valid JSON.
    MalformedJsonChunk(String),
}

impl fmt::Display for AccumulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferExceeded {
                current_bytes,
                max_bytes,
            } => write!(
                f,
                "Streaming accumulator buffer ceiling exceeded: {current_bytes} bytes exceeds limit of {max_bytes} bytes"
            ),
            Self::ChunkLimitExceeded {
                chunk_count,
                max_chunks,
            } => write!(
                f,
                "Streaming chunk count limit exceeded: {chunk_count} chunks exceeds maximum of {max_chunks}"
            ),
            Self::MalformedJsonChunk(err) => {
                write!(f, "Failed to parse structured streaming JSON chunk: {err}")
            }
        }
    }
}

impl std::error::Error for AccumulatorError {}

/// Accumulator tracking streaming response deltas and constructing unified turn payloads.
#[derive(Debug, Clone)]
pub struct StreamingPayloadAccumulator {
    config: StreamingAccumulatorConfig,
    current_turn: CandidateTurnState,
    default_role: String,
    in_thought: bool,
    new_thought_part_needed: bool,
    pending_tag_buffer: String,
    chunk_count: usize,
    legacy_tool_calls: Vec<AccumulatedToolCall>,
}

impl Default for StreamingPayloadAccumulator {
    fn default() -> Self {
        Self::new(StreamingAccumulatorConfig::default())
    }
}

impl StreamingPayloadAccumulator {
    /// Creates a new streaming payload accumulator with the specified configuration.
    pub fn new(config: StreamingAccumulatorConfig) -> Self {
        let default_role = "model".to_string();
        let current_turn = CandidateTurnState::new(&default_role);
        Self {
            config,
            current_turn,
            default_role,
            in_thought: false,
            new_thought_part_needed: false,
            pending_tag_buffer: String::new(),
            chunk_count: 0,
            legacy_tool_calls: Vec::new(),
        }
    }

    /// Sets the candidate turn role fluently.
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        let r = role.into();
        self.default_role = r.clone();
        self.current_turn.role = r;
        self
    }

    /// Overwrites the default and active turn role.
    pub fn set_role(&mut self, role: impl Into<String>) {
        let r = role.into();
        self.default_role = r.clone();
        self.current_turn.role = r;
    }

    /// Accessor for accumulator configuration.
    pub fn config(&self) -> &StreamingAccumulatorConfig {
        &self.config
    }

    /// Accessor for current turn state snapshot.
    pub fn current_turn(&self) -> &CandidateTurnState {
        &self.current_turn
    }

    /// Total bytes buffered in the current turn.
    pub fn bytes_buffered(&self) -> usize {
        self.current_turn.total_bytes
    }

    /// Total bytes ingested so far (backward compatibility).
    pub fn current_bytes(&self) -> usize {
        self.current_turn.total_bytes
    }

    /// Total chunk count ingested so far.
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Returns true if no chunks have been ingested.
    pub fn is_empty(&self) -> bool {
        self.chunk_count == 0 && self.current_turn.total_bytes == 0
    }

    /// Returns true if currently inside an open `<thought>` block.
    pub fn is_in_thought(&self) -> bool {
        self.in_thought
    }

    /// Ingests a raw text delta chunk, isolating thinking blocks and appending content.
    pub fn push_text_delta(&mut self, delta: &str) -> Result<AccumulatorChunkResult, String> {
        let delta_len = delta.len();
        let new_total = self.current_turn.total_bytes + delta_len;
        if new_total > self.config.max_buffer_bytes {
            return Err(format!(
                "Streaming buffer overflow: total bytes {} exceeds maximum allowable {}",
                new_total, self.config.max_buffer_bytes
            ));
        }
        self.current_turn.total_bytes = new_total;
        self.chunk_count += 1;

        if !self.config.detect_thought_tags {
            if !delta.is_empty() {
                Self::append_text(&mut self.current_turn.parts, delta);
            }
            return Ok(AccumulatorChunkResult {
                action: AccumulatorAction::Appended,
                current_turn: self.current_turn.clone(),
                bytes_buffered: self.current_turn.total_bytes,
            });
        }

        let combined = format!("{}{}", self.pending_tag_buffer, delta);
        self.pending_tag_buffer.clear();

        let mut cursor = 0;
        let mut part_completed = false;

        while cursor < combined.len() {
            if !self.in_thought {
                let rem = &combined[cursor..];
                let rem_lower = rem.to_ascii_lowercase();

                if let Some(pos) = rem_lower.find(THOUGHT_TAG_OPEN) {
                    let before = &rem[..pos];
                    Self::append_text(&mut self.current_turn.parts, before);
                    self.in_thought = true;
                    self.new_thought_part_needed = true;
                    cursor += pos + THOUGHT_TAG_OPEN.len();
                } else {
                    let mut matched_prefix_len = 0;
                    let max_k = rem.len().min(THOUGHT_TAG_OPEN.len() - 1);
                    for k in (1..=max_k).rev() {
                        let suffix = &rem[rem.len() - k..];
                        if THOUGHT_TAG_OPEN.starts_with(&suffix.to_ascii_lowercase()) {
                            matched_prefix_len = k;
                            break;
                        }
                    }

                    if matched_prefix_len > 0 {
                        let before = &rem[..rem.len() - matched_prefix_len];
                        Self::append_text(&mut self.current_turn.parts, before);
                        self.pending_tag_buffer = rem[rem.len() - matched_prefix_len..].to_string();
                    } else {
                        Self::append_text(&mut self.current_turn.parts, rem);
                    }
                    cursor = combined.len();
                }
            } else {
                let rem = &combined[cursor..];
                let rem_lower = rem.to_ascii_lowercase();

                if let Some(pos) = rem_lower.find(THOUGHT_TAG_CLOSE) {
                    let before = &rem[..pos];
                    Self::append_or_create_thought(
                        &mut self.current_turn.parts,
                        before,
                        &mut self.new_thought_part_needed,
                    );
                    self.in_thought = false;
                    part_completed = true;
                    cursor += pos + THOUGHT_TAG_CLOSE.len();
                } else {
                    let mut matched_prefix_len = 0;
                    let max_k = rem.len().min(THOUGHT_TAG_CLOSE.len() - 1);
                    for k in (1..=max_k).rev() {
                        let suffix = &rem[rem.len() - k..];
                        if THOUGHT_TAG_CLOSE.starts_with(&suffix.to_ascii_lowercase()) {
                            matched_prefix_len = k;
                            break;
                        }
                    }

                    if matched_prefix_len > 0 {
                        let before = &rem[..rem.len() - matched_prefix_len];
                        Self::append_or_create_thought(
                            &mut self.current_turn.parts,
                            before,
                            &mut self.new_thought_part_needed,
                        );
                        self.pending_tag_buffer = rem[rem.len() - matched_prefix_len..].to_string();
                    } else {
                        Self::append_or_create_thought(
                            &mut self.current_turn.parts,
                            rem,
                            &mut self.new_thought_part_needed,
                        );
                    }
                    cursor = combined.len();
                }
            }
        }

        let action = if part_completed {
            AccumulatorAction::PartCompleted
        } else {
            AccumulatorAction::Appended
        };

        Ok(AccumulatorChunkResult {
            action,
            current_turn: self.current_turn.clone(),
            bytes_buffered: self.current_turn.total_bytes,
        })
    }

    /// Ingests a streaming fragment of function call arguments.
    pub fn push_function_call_chunk(
        &mut self,
        call_name: &str,
        args_chunk: &str,
    ) -> Result<AccumulatorChunkResult, String> {
        let chunk_bytes = call_name.len() + args_chunk.len();
        let new_total = self.current_turn.total_bytes + chunk_bytes;
        if new_total > self.config.max_buffer_bytes {
            return Err(format!(
                "Streaming buffer overflow: total bytes {} exceeds maximum allowable {}",
                new_total, self.config.max_buffer_bytes
            ));
        }
        self.current_turn.total_bytes = new_total;
        self.chunk_count += 1;

        let mut is_completed_now = false;

        match self.current_turn.parts.last_mut() {
            Some(AccumulatedPart::FunctionCall {
                name,
                args,
                is_complete,
            }) if *name == call_name && !*is_complete => {
                args.push_str(args_chunk);
                if serde_json::from_str::<serde_json::Value>(args).is_ok() {
                    *is_complete = true;
                    is_completed_now = true;
                }
            }
            _ => {
                let is_parseable = serde_json::from_str::<serde_json::Value>(args_chunk).is_ok();
                if is_parseable {
                    is_completed_now = true;
                }
                self.current_turn.parts.push(AccumulatedPart::FunctionCall {
                    name: call_name.to_string(),
                    args: args_chunk.to_string(),
                    is_complete: is_parseable,
                });
            }
        }

        let action = if is_completed_now {
            AccumulatorAction::PartCompleted
        } else {
            AccumulatorAction::Appended
        };

        Ok(AccumulatorChunkResult {
            action,
            current_turn: self.current_turn.clone(),
            bytes_buffered: self.current_turn.total_bytes,
        })
    }

    /// Pushes an inline multimodal binary payload into the candidate turn.
    pub fn push_inline_data(
        &mut self,
        mime_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Result<AccumulatorChunkResult, String> {
        let m = mime_type.into();
        let d = data.into();
        let byte_len = m.len() + d.len();
        let new_total = self.current_turn.total_bytes + byte_len;
        if new_total > self.config.max_buffer_bytes {
            return Err(format!(
                "Streaming buffer overflow: total bytes {} exceeds maximum allowable {}",
                new_total, self.config.max_buffer_bytes
            ));
        }
        self.current_turn.total_bytes = new_total;
        self.chunk_count += 1;
        self.current_turn.parts.push(AccumulatedPart::InlineData {
            mime_type: m,
            data: d,
        });

        Ok(AccumulatorChunkResult {
            action: AccumulatorAction::PartCompleted,
            current_turn: self.current_turn.clone(),
            bytes_buffered: self.current_turn.total_bytes,
        })
    }

    /// Finalizes the current candidate turn, applying finish reasons and JSON argument auto-repairs.
    pub fn finish_turn(&mut self, finish_reason: Option<String>) -> CandidateTurnState {
        if !self.pending_tag_buffer.is_empty() {
            let pending = std::mem::take(&mut self.pending_tag_buffer);
            if self.in_thought {
                Self::append_or_create_thought(
                    &mut self.current_turn.parts,
                    &pending,
                    &mut self.new_thought_part_needed,
                );
            } else {
                Self::append_text(&mut self.current_turn.parts, &pending);
            }
        }

        if self.config.auto_repair_json {
            if let Some(AccumulatedPart::FunctionCall {
                args,
                is_complete,
                ..
            }) = self.current_turn.parts.last_mut()
            {
                if !*is_complete {
                    if let Some(repaired) = Self::try_repair_json(args) {
                        *args = repaired;
                        *is_complete = true;
                    }
                }
            }
        }

        self.current_turn.finish_reason = finish_reason;
        self.current_turn.is_turn_complete = true;
        self.current_turn.clone()
    }

    /// Resets the accumulator state for a new turn.
    pub fn reset(&mut self) {
        self.current_turn = CandidateTurnState::new(&self.default_role);
        self.in_thought = false;
        self.new_thought_part_needed = false;
        self.pending_tag_buffer.clear();
        self.chunk_count = 0;
        self.legacy_tool_calls.clear();
    }

    fn append_text(parts: &mut Vec<AccumulatedPart>, text: &str) {
        if text.is_empty() {
            return;
        }
        match parts.last_mut() {
            Some(AccumulatedPart::Text(existing)) => existing.push_str(text),
            _ => parts.push(AccumulatedPart::Text(text.to_string())),
        }
    }

    fn append_or_create_thought(
        parts: &mut Vec<AccumulatedPart>,
        thought: &str,
        new_needed: &mut bool,
    ) {
        if thought.is_empty() && !*new_needed {
            return;
        }
        if *new_needed {
            parts.push(AccumulatedPart::Thought(thought.to_string()));
            *new_needed = false;
        } else {
            match parts.last_mut() {
                Some(AccumulatedPart::Thought(existing)) => existing.push_str(thought),
                _ => parts.push(AccumulatedPart::Thought(thought.to_string())),
            }
        }
    }

    /// Attempts auto-repair of unclosed JSON structures (strings, objects, arrays).
    pub fn try_repair_json(s: &str) -> Option<String> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return Some(trimmed.to_string());
        }

        let mut repaired = trimmed.to_string();
        let mut in_string = false;
        let mut escaped = false;
        let mut stack = Vec::new();

        for c in repaired.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            match c {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '{' if !in_string => stack.push('}'),
                '[' if !in_string => stack.push(']'),
                '}' if !in_string => {
                    if stack.last() == Some(&'}') {
                        stack.pop();
                    }
                }
                ']' if !in_string => {
                    if stack.last() == Some(&']') {
                        stack.pop();
                    }
                }
                _ => {}
            }
        }

        if in_string {
            repaired.push('"');
        }
        while let Some(closing) = stack.pop() {
            repaired.push(closing);
        }

        if serde_json::from_str::<serde_json::Value>(&repaired).is_ok() {
            Some(repaired)
        } else {
            None
        }
    }

    // =========================================================================
    // Backward Compatibility Layer (ingest_chunk, finalize_turn)
    // =========================================================================

    /// Ingests a raw streaming chunk (SSE line or JSON chunk), updating internal state.
    pub fn ingest_chunk(&mut self, chunk: &str) -> Result<(), AccumulatorError> {
        let chunk_len = chunk.len();

        if self.chunk_count + 1 > self.config.max_chunks {
            return Err(AccumulatorError::ChunkLimitExceeded {
                chunk_count: self.chunk_count + 1,
                max_chunks: self.config.max_chunks,
            });
        }

        if self.current_turn.total_bytes + chunk_len > self.config.max_buffer_bytes {
            return Err(AccumulatorError::BufferExceeded {
                current_bytes: self.current_turn.total_bytes + chunk_len,
                max_bytes: self.config.max_buffer_bytes,
            });
        }

        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            self.chunk_count += 1;
            self.current_turn.total_bytes += chunk_len;
            return Ok(());
        }

        if trimmed == "data: [DONE]" || trimmed == "[DONE]" {
            self.chunk_count += 1;
            self.current_turn.total_bytes += chunk_len;
            if self.current_turn.finish_reason.is_none() {
                self.current_turn.finish_reason = Some("STOP".to_string());
            }
            return Ok(());
        }

        if let Some(stripped) = trimmed.strip_prefix("data:") {
            let data_payload = stripped.trim();
            if data_payload == "[DONE]" {
                self.chunk_count += 1;
                self.current_turn.total_bytes += chunk_len;
                if self.current_turn.finish_reason.is_none() {
                    self.current_turn.finish_reason = Some("STOP".to_string());
                }
                return Ok(());
            }

            if data_payload.starts_with('{') || data_payload.starts_with('[') {
                self.parse_json_payload(data_payload, chunk_len)?;
            } else {
                let _ = self.push_text_delta(data_payload);
            }
            return Ok(());
        }

        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            self.parse_json_payload(trimmed, chunk_len)?;
            return Ok(());
        }

        // Direct raw text chunk
        let _ = self.push_text_delta(chunk);
        Ok(())
    }

    fn parse_json_payload(&mut self, json_str: &str, chunk_len: usize) -> Result<(), AccumulatorError> {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AccumulatorError::MalformedJsonChunk(e.to_string()))?;

        self.chunk_count += 1;
        self.current_turn.total_bytes += chunk_len;

        // 1. Gemini candidate structure
        if let Some(candidates) = value.get("candidates").and_then(|c| c.as_array()) {
            if let Some(candidate) = candidates.first() {
                if let Some(fr) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                    self.current_turn.finish_reason = Some(fr.to_string());
                }
                if let Some(parts) = candidate
                    .get("content")
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                {
                    for part in parts {
                        let is_thought = part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false);
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            if is_thought {
                                Self::append_or_create_thought(
                                    &mut self.current_turn.parts,
                                    text,
                                    &mut self.new_thought_part_needed,
                                );
                            } else {
                                Self::append_text(&mut self.current_turn.parts, text);
                            }
                        }
                        if let Some(fc) = part.get("functionCall") {
                            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                            let args = if let Some(args_val) = fc.get("args") {
                                if args_val.is_string() {
                                    args_val.as_str().unwrap().to_string()
                                } else {
                                    serde_json::to_string(args_val).unwrap_or_default()
                                }
                            } else {
                                String::new()
                            };
                            let id = format!("call_{}", self.legacy_tool_calls.len() + 1);
                            self.legacy_tool_calls.push(AccumulatedToolCall {
                                id,
                                name: name.clone(),
                                arguments: args.clone(),
                            });
                            self.current_turn.parts.push(AccumulatedPart::FunctionCall {
                                name,
                                args,
                                is_complete: true,
                            });
                        }
                    }
                }
            }
            return Ok(());
        }

        // 2. OpenAI streaming choices structure
        if let Some(choices) = value.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(fr) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                    self.current_turn.finish_reason = Some(fr.to_string());
                }
                if let Some(delta) = choice.get("delta") {
                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                        Self::append_text(&mut self.current_turn.parts, content);
                    }
                    if let Some(reasoning) = delta
                        .get("reasoning_content")
                        .or_else(|| delta.get("thinking"))
                        .and_then(|r| r.as_str())
                    {
                        Self::append_or_create_thought(
                            &mut self.current_turn.parts,
                            reasoning,
                            &mut self.new_thought_part_needed,
                        );
                    }
                    if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                        for tc in tcs {
                            let idx = tc.get("index").and_then(|i| i.as_u64()).map(|i| i as usize);
                            let id = tc.get("id").and_then(|i| i.as_str());
                            let fn_obj = tc.get("function");
                            let name = fn_obj.and_then(|f| f.get("name")).and_then(|n| n.as_str());
                            let args = fn_obj.and_then(|f| f.get("arguments")).and_then(|a| a.as_str());

                            self.ingest_tool_call_delta(idx, id, name, args);
                        }
                    }
                }
            }
            return Ok(());
        }

        // 3. Fallback text/thought
        if let Some(txt) = value.get("text").and_then(|t| t.as_str()) {
            Self::append_text(&mut self.current_turn.parts, txt);
        }
        if let Some(th) = value.get("thought").and_then(|t| t.as_str()) {
            Self::append_or_create_thought(
                &mut self.current_turn.parts,
                th,
                &mut self.new_thought_part_needed,
            );
        }

        Ok(())
    }

    fn ingest_tool_call_delta(
        &mut self,
        index: Option<usize>,
        id: Option<&str>,
        name: Option<&str>,
        args: Option<&str>,
    ) {
        if let Some(idx) = index {
            while self.legacy_tool_calls.len() <= idx {
                self.legacy_tool_calls.push(AccumulatedToolCall {
                    id: format!("call_{}", self.legacy_tool_calls.len() + 1),
                    name: String::new(),
                    arguments: String::new(),
                });
            }
            let target = &mut self.legacy_tool_calls[idx];
            if let Some(new_id) = id {
                if !new_id.is_empty() {
                    target.id = new_id.to_string();
                }
            }
            if let Some(new_name) = name {
                if !new_name.is_empty() {
                    target.name.push_str(new_name);
                }
            }
            if let Some(new_args) = args {
                target.arguments.push_str(new_args);
            }
            return;
        }

        if let Some(id_str) = id {
            if let Some(existing) = self.legacy_tool_calls.iter_mut().find(|tc| tc.id == id_str) {
                if let Some(new_name) = name {
                    existing.name.push_str(new_name);
                }
                if let Some(new_args) = args {
                    existing.arguments.push_str(new_args);
                }
                return;
            }
        }

        self.legacy_tool_calls.push(AccumulatedToolCall {
            id: id.unwrap_or(&format!("call_{}", self.legacy_tool_calls.len() + 1)).to_string(),
            name: name.unwrap_or("").to_string(),
            arguments: args.unwrap_or("").to_string(),
        });
    }

    /// Finalizes and returns the accumulated candidate turn as `AccumulatedCandidateTurn`.
    pub fn finalize_turn(&self) -> AccumulatedCandidateTurn {
        let thought = if self.current_turn.has_thought() {
            Some(self.current_turn.full_thought())
        } else {
            None
        };

        let mut tool_calls = self.legacy_tool_calls.clone();
        if tool_calls.is_empty() {
            for (idx, part) in self.current_turn.parts.iter().enumerate() {
                if let AccumulatedPart::FunctionCall { name, args, .. } = part {
                    tool_calls.push(AccumulatedToolCall {
                        id: format!("call_{}", idx + 1),
                        name: name.clone(),
                        arguments: args.clone(),
                    });
                }
            }
        }

        AccumulatedCandidateTurn {
            text: self.current_turn.full_text(),
            thought,
            tool_calls,
            finish_reason: self.current_turn.finish_reason.clone(),
            total_chunks: self.chunk_count,
            total_bytes: self.current_turn.total_bytes,
        }
    }
}
