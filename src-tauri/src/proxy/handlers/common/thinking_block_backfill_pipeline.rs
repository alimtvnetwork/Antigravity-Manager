//! Thinking Block Backfill Pipeline (`Imp-252`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability, and the sanitization
//! of risky prompts and request headers."
//!
//! Handles detecting, backfilling, and standardizing thinking blocks into
//! Gemini-compatible `<thought>` tags and candidate part flags.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard reasoning prefixes used across OpenAI, Anthropic, and DeepSeek adapters.
pub const REASONING_PREFIXES: &[&str] = &[
    "Reasoning:",
    "Thinking:",
    "[Thought]",
    "[Thinking]",
    "reasoning_content:",
];

/// Known answer transition markers separating reasoning from final output.
pub const ANSWER_TRANSITION_MARKERS: &[&str] = &[
    "\n\nFinal Answer:",
    "\n\nAnswer:",
    "\n\nOutput:",
    "\n\nResponse:",
];

/// Format identification for thinking blocks within payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingPayloadFormat {
    /// Content enclosed within XML-style thought tags (e.g. `<thought>...</thought>`).
    TaggedThought { tag_open: String, tag_close: String },
    /// JSON part with candidate `"thought": true` flag.
    CandidatePartThoughtFlag,
    /// Raw un-tagged reasoning content.
    RawReasoningContent,
}

impl ThinkingPayloadFormat {
    /// Returns true if content uses tagged thought XML format.
    pub fn is_tagged(&self) -> bool {
        matches!(self, Self::TaggedThought { .. })
    }

    /// Returns true if candidate part flag format is used.
    pub fn is_candidate_part(&self) -> bool {
        matches!(self, Self::CandidatePartThoughtFlag)
    }

    /// Returns true if format is raw reasoning content.
    pub fn is_raw_reasoning(&self) -> bool {
        matches!(self, Self::RawReasoningContent)
    }
}

impl fmt::Display for ThinkingPayloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaggedThought {
                tag_open,
                tag_close,
            } => write!(f, "TaggedThought({tag_open}...{tag_close})"),
            Self::CandidatePartThoughtFlag => write!(f, "CandidatePartThoughtFlag"),
            Self::RawReasoningContent => write!(f, "RawReasoningContent"),
        }
    }
}

/// Backfill or normalization action performed on a payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackfillAction {
    /// No transformation was required; content is already properly formed.
    NoBackfillNeeded,
    /// Missing `<thought>` tags were injected at the specified character offset.
    ThoughtTagInjected { character_offset: usize },
    /// JSON candidate part was tagged with `"thought": true`.
    ThoughtPartFlagged { part_index: usize },
    /// An empty thinking block `<thought></thought>` was stripped.
    EmptyThinkingBlockStripped,
}

impl BackfillAction {
    /// Returns true if no backfill modification was performed.
    pub fn is_no_action(&self) -> bool {
        matches!(self, Self::NoBackfillNeeded)
    }

    /// Returns true if thought tags were injected.
    pub fn is_injected(&self) -> bool {
        matches!(self, Self::ThoughtTagInjected { .. })
    }

    /// Returns true if candidate part was flagged.
    pub fn is_flagged(&self) -> bool {
        matches!(self, Self::ThoughtPartFlagged { .. })
    }

    /// Returns true if empty thinking block was stripped.
    pub fn is_stripped(&self) -> bool {
        matches!(self, Self::EmptyThinkingBlockStripped)
    }
}

impl fmt::Display for BackfillAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBackfillNeeded => write!(f, "NoBackfillNeeded"),
            Self::ThoughtTagInjected { character_offset } => {
                write!(f, "ThoughtTagInjected at offset {character_offset}")
            }
            Self::ThoughtPartFlagged { part_index } => {
                write!(f, "ThoughtPartFlagged at index {part_index}")
            }
            Self::EmptyThinkingBlockStripped => write!(f, "EmptyThinkingBlockStripped"),
        }
    }
}

/// Comprehensive report on thinking block detection and backfill execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingBackfillReport {
    /// True if payload was transformed or backfilled.
    pub is_modified: bool,
    /// Detected or target payload format.
    pub format: ThinkingPayloadFormat,
    /// Concrete action applied.
    pub action: BackfillAction,
    /// Processed and normalized text content.
    pub processed_content: String,
    /// Estimated token count consumed by reasoning/thought.
    pub estimated_thinking_tokens: usize,
}

impl ThinkingBackfillReport {
    /// Returns true if content was modified during processing.
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Returns reference to processed content string.
    pub fn processed_content(&self) -> &str {
        &self.processed_content
    }

    /// Returns estimated thinking token budget.
    pub fn estimated_tokens(&self) -> usize {
        self.estimated_thinking_tokens
    }
}

impl fmt::Display for ThinkingBackfillReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThinkingBackfillReport(modified={}, format={}, action={}, tokens={})",
            self.is_modified, self.format, self.action, self.estimated_thinking_tokens
        )
    }
}

/// Estimates thinking token budget from text length (~4 chars per token).
pub fn estimate_thinking_tokens(text: &str) -> usize {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        0
    } else {
        (trimmed.chars().count() / 4).max(1)
    }
}

/// Extracts text enclosed within `<thought>` and `</thought>` tags.
pub fn extract_thought_content(content: &str) -> Option<&str> {
    let start_idx = content.find("<thought>")? + "<thought>".len();
    let end_idx = content[start_idx..].find("</thought>")? + start_idx;
    Some(&content[start_idx..end_idx])
}

/// Detects thinking payload format from raw string content.
pub fn detect_thinking_format(content: &str) -> ThinkingPayloadFormat {
    if content.contains("<thought>") || content.contains("</thought>") {
        ThinkingPayloadFormat::TaggedThought {
            tag_open: "<thought>".to_string(),
            tag_close: "</thought>".to_string(),
        }
    } else if content.contains("\"thought\": true") || content.contains("\"thought\":true") {
        ThinkingPayloadFormat::CandidatePartThoughtFlag
    } else {
        ThinkingPayloadFormat::RawReasoningContent
    }
}

/// Strips empty `<thought></thought>` blocks if present.
fn strip_empty_thought_blocks(content: &str) -> Option<String> {
    if let (Some(start), Some(end)) = (content.find("<thought>"), content.find("</thought>")) {
        let inner = &content[start + "<thought>".len()..end];
        if inner.trim().is_empty() {
            let before = &content[..start];
            let after = &content[end + "</thought>".len()..];
            let combined = format!("{}{}", before.trim_end(), after.trim_start());
            return Some(combined.trim().to_string());
        }
    }
    None
}

/// Strips reasoning prefix headers from raw reasoning string.
fn strip_reasoning_prefix(text: &str) -> (&str, bool) {
    let trimmed = text.trim_start();
    for prefix in REASONING_PREFIXES {
        if let Some(stripped) = trimmed.strip_prefix(prefix) {
            return (stripped.trim_start(), true);
        }
    }
    (trimmed, false)
}

/// Splits reasoning text and subsequent answer marker if present.
fn split_reasoning_and_answer(text: &str) -> (&str, &str) {
    for marker in ANSWER_TRANSITION_MARKERS {
        if let Some(pos) = text.find(marker) {
            let reasoning = &text[..pos];
            let answer = &text[pos + marker.len()..];
            return (reasoning.trim(), answer.trim());
        }
    }
    (text.trim(), "")
}

/// Wraps reasoning portion with `<thought>` tags and appends optional answer.
fn format_wrapped_thought(reasoning: &str, answer: &str) -> String {
    if answer.is_empty() {
        format!("<thought>\n{reasoning}\n</thought>")
    } else {
        format!("<thought>\n{reasoning}\n</thought>\n\n{answer}")
    }
}

/// Builds report for already tagged thinking content.
fn build_tagged_report(raw_content: &str) -> ThinkingBackfillReport {
    let thought_text = extract_thought_content(raw_content).unwrap_or("");
    ThinkingBackfillReport {
        is_modified: false,
        format: ThinkingPayloadFormat::TaggedThought {
            tag_open: "<thought>".to_string(),
            tag_close: "</thought>".to_string(),
        },
        action: BackfillAction::NoBackfillNeeded,
        processed_content: raw_content.to_string(),
        estimated_thinking_tokens: estimate_thinking_tokens(thought_text),
    }
}

/// Builds report when empty thinking blocks are stripped.
fn build_stripped_report(stripped_content: String) -> ThinkingBackfillReport {
    ThinkingBackfillReport {
        is_modified: true,
        format: ThinkingPayloadFormat::TaggedThought {
            tag_open: "<thought>".to_string(),
            tag_close: "</thought>".to_string(),
        },
        action: BackfillAction::EmptyThinkingBlockStripped,
        processed_content: stripped_content,
        estimated_thinking_tokens: 0,
    }
}

/// Backfills missing thought tags into reasoning text.
fn execute_thought_injection(raw_content: &str) -> ThinkingBackfillReport {
    let (stripped_text, _has_prefix) = strip_reasoning_prefix(raw_content);
    let (reasoning_part, answer_part) = split_reasoning_and_answer(stripped_text);
    let wrapped = format_wrapped_thought(reasoning_part, answer_part);
    let tokens = estimate_thinking_tokens(reasoning_part);

    ThinkingBackfillReport {
        is_modified: true,
        format: ThinkingPayloadFormat::TaggedThought {
            tag_open: "<thought>".to_string(),
            tag_close: "</thought>".to_string(),
        },
        action: BackfillAction::ThoughtTagInjected {
            character_offset: 0,
        },
        processed_content: wrapped,
        estimated_thinking_tokens: tokens,
    }
}

/// Backfills and standardizes Gemini thinking blocks into XML tags or candidate flags.
pub fn backfill_gemini_thinking_blocks(
    raw_content: &str,
    require_thought_tags: bool,
) -> ThinkingBackfillReport {
    if raw_content.is_empty() {
        return ThinkingBackfillReport {
            is_modified: false,
            format: ThinkingPayloadFormat::RawReasoningContent,
            action: BackfillAction::NoBackfillNeeded,
            processed_content: String::new(),
            estimated_thinking_tokens: 0,
        };
    }

    if let Some(stripped) = strip_empty_thought_blocks(raw_content) {
        return build_stripped_report(stripped);
    }

    if raw_content.contains("<thought>") {
        return build_tagged_report(raw_content);
    }

    if require_thought_tags {
        return execute_thought_injection(raw_content);
    }

    ThinkingBackfillReport {
        is_modified: false,
        format: detect_thinking_format(raw_content),
        action: BackfillAction::NoBackfillNeeded,
        processed_content: raw_content.to_string(),
        estimated_thinking_tokens: 0,
    }
}

/// Checks if a JSON part object represents reasoning/thought content.
fn is_candidate_thought_part(part: &serde_json::Value) -> bool {
    let obj = match part.as_object() {
        Some(o) => o,
        None => return false,
    };

    if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
        if text.trim_start().starts_with("<thought>") {
            return true;
        }
    }

    if let Some(part_type) = obj.get("type").and_then(|t| t.as_str()) {
        if part_type == "reasoning" || part_type == "thought" {
            return true;
        }
    }

    obj.contains_key("reasoning")
}

/// Flags candidate JSON parts with `"thought": true`, auditing applied actions.
pub fn audit_and_flag_thinking_parts(parts: &mut [serde_json::Value]) -> Vec<BackfillAction> {
    let mut actions = Vec::new();
    for (idx, part) in parts.iter_mut().enumerate() {
        if is_candidate_thought_part(part) {
            if let Some(obj) = part.as_object_mut() {
                let is_already_flagged = obj
                    .get("thought")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if !is_already_flagged {
                    obj.insert("thought".to_string(), serde_json::Value::Bool(true));
                    actions.push(BackfillAction::ThoughtPartFlagged { part_index: idx });
                }
            }
        }
    }
    actions
}

/// Ensures candidate reasoning parts have `"thought": true` set; returns count of modified parts.
pub fn ensure_thinking_part_flags(parts: &mut [serde_json::Value]) -> usize {
    audit_and_flag_thinking_parts(parts).len()
}
