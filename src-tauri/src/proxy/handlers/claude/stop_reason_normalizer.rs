//! Claude stop reason normalizer.
//! Bidirectionally maps completion termination reasons between Anthropic Claude,
//! OpenAI Chat Completions, and Gemini Canonical IR representations.

use serde::{Deserialize, Serialize};

/// Canonical termination reason representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalStopReason {
    Stop,
    Length,
    ToolUse,
    ContentFilter,
    Other,
}

pub struct StopReasonNormalizer;

impl StopReasonNormalizer {
    /// Converts Claude stop_reason string into CanonicalStopReason.
    pub fn claude_to_canonical(claude_reason: Option<&str>) -> CanonicalStopReason {
        match claude_reason {
            Some("end_turn") | Some("stop_sequence") => CanonicalStopReason::Stop,
            Some("max_tokens") => CanonicalStopReason::Length,
            Some("tool_use") => CanonicalStopReason::ToolUse,
            Some("content_filter") => CanonicalStopReason::ContentFilter,
            _ => CanonicalStopReason::Other,
        }
    }

    /// Converts CanonicalStopReason to OpenAI-compatible finish_reason string.
    pub fn canonical_to_openai(canonical: CanonicalStopReason) -> &'static str {
        match canonical {
            CanonicalStopReason::Stop => "stop",
            CanonicalStopReason::Length => "length",
            CanonicalStopReason::ToolUse => "tool_calls",
            CanonicalStopReason::ContentFilter => "content_filter",
            CanonicalStopReason::Other => "stop",
        }
    }

    /// Converts CanonicalStopReason to Claude-compatible stop_reason string.
    pub fn canonical_to_claude(canonical: CanonicalStopReason) -> &'static str {
        match canonical {
            CanonicalStopReason::Stop => "end_turn",
            CanonicalStopReason::Length => "max_tokens",
            CanonicalStopReason::ToolUse => "tool_use",
            CanonicalStopReason::ContentFilter => "content_filter",
            CanonicalStopReason::Other => "end_turn",
        }
    }

    /// Direct mapping from Claude to OpenAI finish_reason.
    pub fn claude_to_openai(claude_reason: Option<&str>) -> &'static str {
        Self::canonical_to_openai(Self::claude_to_canonical(claude_reason))
    }

    /// Direct mapping from OpenAI finish_reason to Claude stop_reason.
    pub fn openai_to_claude(openai_finish_reason: Option<&str>) -> &'static str {
        match openai_finish_reason {
            Some("stop") => "end_turn",
            Some("length") => "max_tokens",
            Some("tool_calls") | Some("function_call") => "tool_use",
            Some("content_filter") => "content_filter",
            _ => "end_turn",
        }
    }
}
