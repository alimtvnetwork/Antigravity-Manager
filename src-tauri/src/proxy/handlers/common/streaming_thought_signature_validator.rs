//! Streaming Thought Signature & Delimiter Validator (`Imp-710`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability, and the sanitization
//! of risky prompts and request headers."
//!
//! Validates SSE streaming payloads containing thinking blocks. Ensures `<thought>`
//! and `</thought>` tags, `thought: true` attributes, and delta buffers maintain
//! balanced delimiters and do not leak internal marker tokens across SSE frames.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard thought opening tag delimiter.
pub const THOUGHT_TAG_OPEN: &str = "<thought>";

/// Standard thought closing tag delimiter.
pub const THOUGHT_TAG_CLOSE: &str = "</thought>";

/// Alternative thought opening tag delimiter to normalize.
pub const ALT_THINKING_TAG_OPEN: &str = "<thinking>";

/// Alternative thought closing tag delimiter to normalize.
pub const ALT_THINKING_TAG_CLOSE: &str = "</thinking>";

/// Internal marker prefixes that must not leak across SSE frames to downstream clients.
pub const INTERNAL_LEAKING_MARKERS: &[&str] = &[
    "[Thought]",
    "[Thinking]",
    "[Thought:",
    "[Thinking:",
    "[REASONING]",
    "reasoning_content:",
];

/// State of thought tag delimiter processing within the streaming pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThoughtTagState {
    /// Outside any thinking block.
    #[default]
    Inactive,
    /// An opening `<thought>` tag has just been encountered in the current chunk.
    OpenTagEncountered,
    /// Currently accumulating tokens within a `<thought>` reasoning block.
    InsideThoughtBlock,
    /// A closing `</thought>` tag has just been encountered in the current chunk.
    ClosingTagEncountered,
}

impl ThoughtTagState {
    /// Returns true if currently within an open thinking block.
    pub fn is_inside(&self) -> bool {
        matches!(self, Self::OpenTagEncountered | Self::InsideThoughtBlock)
    }

    /// Returns true if inactive (outside of any thinking block).
    pub fn is_inactive(&self) -> bool {
        matches!(self, Self::Inactive)
    }
}

impl fmt::Display for ThoughtTagState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inactive => write!(f, "Inactive"),
            Self::OpenTagEncountered => write!(f, "OpenTagEncountered"),
            Self::InsideThoughtBlock => write!(f, "InsideThoughtBlock"),
            Self::ClosingTagEncountered => write!(f, "ClosingTagEncountered"),
        }
    }
}

/// Findings produced when validating thought delimiters and flags in a stream chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThoughtValidationFinding {
    /// Thought delimiters are properly paired and balanced.
    Balanced,
    /// An unclosed `<thought>` tag was encountered in the stream chunk.
    UnclosedThoughtTag { open_epoch_ms: u64 },
    /// A closing `</thought>` tag was encountered without a matching opening tag.
    UnexpectedClosingTag,
    /// Mismatch between thought block delimiters and candidate part flags (e.g. `thought: true`).
    MismatchedThoughtFlag,
}

impl ThoughtValidationFinding {
    /// Returns true if the finding represents a balanced thought stream.
    pub fn is_balanced(&self) -> bool {
        matches!(self, Self::Balanced)
    }

    /// Returns true if the finding represents an unclosed thought tag.
    pub fn is_unclosed(&self) -> bool {
        matches!(self, Self::UnclosedThoughtTag { .. })
    }

    /// Returns true if the finding represents an unexpected closing tag.
    pub fn is_unexpected_closing(&self) -> bool {
        matches!(self, Self::UnexpectedClosingTag)
    }

    /// Returns true if the finding represents a mismatched thought flag.
    pub fn is_mismatched_flag(&self) -> bool {
        matches!(self, Self::MismatchedThoughtFlag)
    }
}

impl fmt::Display for ThoughtValidationFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Balanced => write!(f, "Balanced: thought delimiters properly formed"),
            Self::UnclosedThoughtTag { open_epoch_ms } => {
                write!(
                    f,
                    "UnclosedThoughtTag: open tag encountered at epoch {open_epoch_ms} ms without matching closing tag"
                )
            }
            Self::UnexpectedClosingTag => {
                write!(
                    f,
                    "UnexpectedClosingTag: closing tag encountered outside active thought block"
                )
            }
            Self::MismatchedThoughtFlag => {
                write!(
                    f,
                    "MismatchedThoughtFlag: thought boolean flag conflicts with tag state"
                )
            }
        }
    }
}

/// Detailed validation report for a streaming chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThoughtStreamValidationReport {
    /// True if chunk conforms to thought delimiter and marker invariants.
    pub is_compliant: bool,
    /// Byte length of reasoning/thought text contained within this chunk.
    pub total_thought_bytes: usize,
    /// Findings cataloging balance, tag state, and flag consistency.
    pub findings: Vec<ThoughtValidationFinding>,
    /// Chunk payload after sanitizing internal leaking markers.
    pub sanitized_payload: String,
}

impl ThoughtStreamValidationReport {
    /// Constructs a new report.
    pub fn new(
        is_compliant: bool,
        total_thought_bytes: usize,
        findings: Vec<ThoughtValidationFinding>,
        sanitized_payload: String,
    ) -> Self {
        Self {
            is_compliant,
            total_thought_bytes,
            findings,
            sanitized_payload,
        }
    }

    /// Returns true if the report is compliant.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Returns true if any unclosed thought tags were found.
    pub fn has_unclosed(&self) -> bool {
        self.findings.iter().any(|f| f.is_unclosed())
    }

    /// Returns true if any unexpected closing tags were found.
    pub fn has_unexpected_closing(&self) -> bool {
        self.findings.iter().any(|f| f.is_unexpected_closing())
    }

    /// Returns true if any mismatched thought flags were found.
    pub fn has_mismatched_flag(&self) -> bool {
        self.findings.iter().any(|f| f.is_mismatched_flag())
    }
}

impl fmt::Display for ThoughtStreamValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThoughtStreamValidationReport(compliant={}, thought_bytes={}, findings={})",
            self.is_compliant,
            self.total_thought_bytes,
            self.findings.len()
        )
    }
}

/// Helper to obtain current timestamp in milliseconds.
fn current_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Sanitizes delta chunk by removing leaking internal markers and normalizing alternative tags.
fn sanitize_chunk_markers(delta: &str) -> String {
    let mut sanitized = delta.to_string();

    // Normalize alternative <thinking> and </thinking> tags to canonical <thought> and </thought>
    if sanitized.contains(ALT_THINKING_TAG_OPEN) {
        sanitized = sanitized.replace(ALT_THINKING_TAG_OPEN, THOUGHT_TAG_OPEN);
    }
    if sanitized.contains(ALT_THINKING_TAG_CLOSE) {
        sanitized = sanitized.replace(ALT_THINKING_TAG_CLOSE, THOUGHT_TAG_CLOSE);
    }

    // Strip internal leaking marker prefixes
    for marker in INTERNAL_LEAKING_MARKERS {
        if sanitized.contains(marker) {
            sanitized = sanitized.replace(marker, "");
        }
    }

    sanitized
}

/// Validates an incoming SSE delta chunk against thought delimiters and updates tag state.
pub fn validate_thought_stream_chunk(
    delta: &str,
    current_state: &mut ThoughtTagState,
) -> ThoughtStreamValidationReport {
    let sanitized = sanitize_chunk_markers(delta);
    let mut findings = Vec::new();
    let mut total_thought_bytes: usize = 0;

    let has_explicit_thought_true = sanitized.contains(r#""thought":true"#)
        || sanitized.contains(r#""thought": true"#);
    let has_explicit_thought_false = sanitized.contains(r#""thought":false"#)
        || sanitized.contains(r#""thought": false"#);

    let has_open_tag = sanitized.contains(THOUGHT_TAG_OPEN);
    let has_close_tag = sanitized.contains(THOUGHT_TAG_CLOSE);

    // Check for unexpected closing tags when inactive and no open tag in this delta
    if has_close_tag && !has_open_tag && *current_state == ThoughtTagState::Inactive {
        findings.push(ThoughtValidationFinding::UnexpectedClosingTag);
    }

    // Check flag mismatches
    if has_explicit_thought_false
        && (*current_state == ThoughtTagState::InsideThoughtBlock || has_open_tag)
    {
        findings.push(ThoughtValidationFinding::MismatchedThoughtFlag);
    }
    if has_explicit_thought_true && !has_open_tag && *current_state == ThoughtTagState::Inactive {
        findings.push(ThoughtValidationFinding::MismatchedThoughtFlag);
    }

    // Calculate thought bytes and state transitions
    match *current_state {
        ThoughtTagState::Inactive => {
            if has_open_tag && has_close_tag {
                // Self-contained thought block in a single chunk
                if let Some(open_idx) = sanitized.find(THOUGHT_TAG_OPEN) {
                    let after_open = open_idx + THOUGHT_TAG_OPEN.len();
                    if let Some(close_idx) = sanitized[after_open..].find(THOUGHT_TAG_CLOSE) {
                        total_thought_bytes = close_idx;
                    }
                }
                *current_state = ThoughtTagState::ClosingTagEncountered;
                findings.push(ThoughtValidationFinding::Balanced);
            } else if has_open_tag {
                // Opened but not closed in this chunk
                if let Some(open_idx) = sanitized.find(THOUGHT_TAG_OPEN) {
                    let after_open = open_idx + THOUGHT_TAG_OPEN.len();
                    total_thought_bytes = sanitized.len().saturating_sub(after_open);
                }
                *current_state = ThoughtTagState::InsideThoughtBlock;
                findings.push(ThoughtValidationFinding::UnclosedThoughtTag {
                    open_epoch_ms: current_epoch_ms(),
                });
            } else {
                // Outside thought block, no tags
                total_thought_bytes = 0;
                if findings.is_empty() {
                    findings.push(ThoughtValidationFinding::Balanced);
                }
            }
        }
        ThoughtTagState::InsideThoughtBlock | ThoughtTagState::OpenTagEncountered => {
            if has_close_tag {
                // Closes an existing thought block
                if let Some(close_idx) = sanitized.find(THOUGHT_TAG_CLOSE) {
                    total_thought_bytes = close_idx;
                } else {
                    total_thought_bytes = sanitized.len();
                }
                *current_state = ThoughtTagState::ClosingTagEncountered;
                findings.push(ThoughtValidationFinding::Balanced);
            } else {
                // Still inside thought block
                total_thought_bytes = sanitized.len();
                *current_state = ThoughtTagState::InsideThoughtBlock;
                findings.push(ThoughtValidationFinding::UnclosedThoughtTag {
                    open_epoch_ms: current_epoch_ms(),
                });
            }
        }
        ThoughtTagState::ClosingTagEncountered => {
            // Tag was closed in previous step, reset or process new open tag
            if has_open_tag {
                if has_close_tag {
                    if let Some(open_idx) = sanitized.find(THOUGHT_TAG_OPEN) {
                        let after_open = open_idx + THOUGHT_TAG_OPEN.len();
                        if let Some(close_idx) = sanitized[after_open..].find(THOUGHT_TAG_CLOSE) {
                            total_thought_bytes = close_idx;
                        }
                    }
                    *current_state = ThoughtTagState::ClosingTagEncountered;
                    findings.push(ThoughtValidationFinding::Balanced);
                } else {
                    if let Some(open_idx) = sanitized.find(THOUGHT_TAG_OPEN) {
                        let after_open = open_idx + THOUGHT_TAG_OPEN.len();
                        total_thought_bytes = sanitized.len().saturating_sub(after_open);
                    }
                    *current_state = ThoughtTagState::InsideThoughtBlock;
                    findings.push(ThoughtValidationFinding::UnclosedThoughtTag {
                        open_epoch_ms: current_epoch_ms(),
                    });
                }
            } else {
                *current_state = ThoughtTagState::Inactive;
                if findings.is_empty() {
                    findings.push(ThoughtValidationFinding::Balanced);
                }
            }
        }
    }

    let is_compliant = !findings.iter().any(|f| {
        matches!(
            f,
            ThoughtValidationFinding::UnexpectedClosingTag
                | ThoughtValidationFinding::MismatchedThoughtFlag
                | ThoughtValidationFinding::UnclosedThoughtTag { .. }
        )
    });

    ThoughtStreamValidationReport {
        is_compliant,
        total_thought_bytes,
        findings,
        sanitized_payload: sanitized,
    }
}

/// Repairs truncated or malformed thought blocks in streamed text.
///
/// Ensures balanced `<thought>` and `</thought>` delimiters when the stream ends abruptly,
/// finishes mid-tag, or misses opening/closing pairs.
pub fn repair_truncated_thought_block(text: &str) -> String {
    let mut repaired = text.to_string();

    if repaired.is_empty() {
        return repaired;
    }

    // Check for partial closing tag truncated at the end
    let partial_closing_prefixes = [
        "</thought", "</thoug", "</thou", "</tho", "</th", "</t", "</",
    ];

    for prefix in &partial_closing_prefixes {
        if repaired.ends_with(prefix) && !repaired.ends_with(THOUGHT_TAG_CLOSE) {
            let trim_len = repaired.len() - prefix.len();
            repaired.truncate(trim_len);
            repaired.push_str(THOUGHT_TAG_CLOSE);
            break;
        }
    }

    // Check for partial opening tag truncated at the end
    let partial_opening_prefixes = [
        "<thought", "<thoug", "<thou", "<tho", "<th", "<t", "<",
    ];
    for prefix in &partial_opening_prefixes {
        if repaired.ends_with(prefix) && !repaired.ends_with(THOUGHT_TAG_OPEN) {
            let trim_len = repaired.len() - prefix.len();
            repaired.truncate(trim_len);
            repaired.push_str(THOUGHT_TAG_OPEN);
            break;
        }
    }

    // Count open and close tags
    let open_count = repaired.matches(THOUGHT_TAG_OPEN).count();
    let close_count = repaired.matches(THOUGHT_TAG_CLOSE).count();

    if open_count > close_count {
        // Missing closing tag(s) - append closing tag(s)
        for _ in 0..(open_count - close_count) {
            repaired.push_str(THOUGHT_TAG_CLOSE);
        }
    } else if close_count > open_count {
        // Missing opening tag(s) - prepend opening tag(s)
        let mut prefix = String::new();
        for _ in 0..(close_count - open_count) {
            prefix.push_str(THOUGHT_TAG_OPEN);
        }
        repaired = format!("{}{}", prefix, repaired);
    }

    repaired
}
