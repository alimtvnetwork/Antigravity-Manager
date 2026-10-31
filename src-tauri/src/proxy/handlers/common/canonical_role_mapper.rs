//! Canonical Role Token Mapper
//! Normalizes protocol-divergent role tokens (developer, assistant, model, system, user, tool)
//! across OpenAI, Claude, and Gemini formats into unified canonical roles.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalRole {
    User,
    Model,
    System,
    Tool,
}

impl CanonicalRole {
    /// Maps a raw role string from any supported AI protocol to a CanonicalRole.
    pub fn from_raw(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.eq_ignore_ascii_case("user") {
            CanonicalRole::User
        } else if trimmed.eq_ignore_ascii_case("assistant")
            || trimmed.eq_ignore_ascii_case("model")
            || trimmed.eq_ignore_ascii_case("bot")
        {
            CanonicalRole::Model
        } else if trimmed.eq_ignore_ascii_case("system")
            || trimmed.eq_ignore_ascii_case("developer")
        {
            CanonicalRole::System
        } else if trimmed.eq_ignore_ascii_case("tool") || trimmed.eq_ignore_ascii_case("function") {
            CanonicalRole::Tool
        } else {
            // Safe fallback to User
            CanonicalRole::User
        }
    }

    /// Converts canonical role to OpenAI protocol role string.
    pub fn to_openai_role(&self) -> &'static str {
        match self {
            CanonicalRole::User => "user",
            CanonicalRole::Model => "assistant",
            CanonicalRole::System => "system",
            CanonicalRole::Tool => "tool",
        }
    }

    /// Converts canonical role to Anthropic Claude protocol role string.
    pub fn to_claude_role(&self) -> &'static str {
        match self {
            CanonicalRole::User => "user",
            CanonicalRole::Model => "assistant",
            CanonicalRole::System => "system",
            CanonicalRole::Tool => "user", // Claude wraps tool results inside user role messages
        }
    }

    /// Converts canonical role to Google Gemini protocol role string.
    pub fn to_gemini_role(&self) -> &'static str {
        match self {
            CanonicalRole::User => "user",
            CanonicalRole::Model => "model",
            CanonicalRole::System => "system",
            CanonicalRole::Tool => "function",
        }
    }
}
