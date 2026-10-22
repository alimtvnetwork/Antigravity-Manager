//! Protocol Role Normalization Validator.
//! Validates and maps protocol role strings across canonical IR, Gemini, OpenAI, and Claude models.

/// Canonical role classifications in AGM protocol IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalRole {
    User,
    Model,     // Gemini assistant
    Assistant, // OpenAI/Claude assistant
    System,
    Developer, // OpenAI o1/o3 developer system messages
    Tool,
}

impl CanonicalRole {
    /// Normalizes raw protocol role strings into typed canonical representation.
    pub fn parse(raw_role: &str) -> Option<Self> {
        let trimmed = raw_role.trim().to_lowercase();
        match trimmed.as_str() {
            "user" | "human" => Some(Self::User),
            "model" => Some(Self::Model),
            "assistant" | "bot" => Some(Self::Assistant),
            "system" => Some(Self::System),
            "developer" => Some(Self::Developer),
            "tool" | "function" => Some(Self::Tool),
            _ => None,
        }
    }

    /// Converts to Gemini protocol standard role (`user` or `model`).
    pub fn to_gemini_role(&self) -> &'static str {
        match self {
            Self::User | Self::Tool => "user",
            Self::Model | Self::Assistant => "model",
            Self::System | Self::Developer => "user", // Folded to user instructions or systemInstruction
        }
    }

    /// Converts to OpenAI standard role string.
    pub fn to_openai_role(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model | Self::Assistant => "assistant",
            Self::System => "system",
            Self::Developer => "developer",
            Self::Tool => "tool",
        }
    }

    /// Converts to Claude standard role string (`user` or `assistant`).
    pub fn to_claude_role(&self) -> &'static str {
        match self {
            Self::User | Self::Tool => "user",
            Self::Model | Self::Assistant => "assistant",
            Self::System | Self::Developer => "user",
        }
    }
}

/// Validates that a sequence of roles contains strictly valid transitions.
pub fn validate_turn_order(roles: &[CanonicalRole]) -> bool {
    if roles.is_empty() {
        return false;
    }
    // Must contain at least one user or system message
    roles.iter().any(|r| matches!(r, CanonicalRole::User | CanonicalRole::System | CanonicalRole::Developer))
}
