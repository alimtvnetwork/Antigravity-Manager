use super::usage::CanonicalUsage;
use serde::{Deserialize, Serialize};

/// Unified domain event abstraction for streaming SSE chunks
/// Upstream Google Gemini SSE chunks are extracted into this event before protocol-specific transformation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalStreamEvent {
    /// Thought reasoning delta
    ThoughtDelta(String),
    /// Thought signature (Google-issued cryptographic signature)
    ThoughtSignature(String),
    /// User-visible text delta
    TextDelta(String),
    /// Tool call delta
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        args_delta: String,
    },
    /// Incremental usage update
    UsageUpdate(CanonicalUsage),
    /// Stream finish event
    Finish {
        reason: String,
        usage: Option<CanonicalUsage>,
    },
}
