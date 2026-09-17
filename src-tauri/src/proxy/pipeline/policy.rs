use serde::{Deserialize, Serialize};

/// Client protocol types served by the proxy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyProtocol {
    OpenAIChat,
    OpenAIResponses,
    AnthropicClaude,
    GeminiNative,
}

impl ProxyProtocol {
    /// Whether this protocol trusts client-provided thought signatures.
    /// - OpenAIChat: false (official specification has no concept of signature; untrusted in client, stripped inbound and server-authoritative backfilled)
    /// - OpenAIResponses / AnthropicClaude / GeminiNative: true (protocols natively support signatures; accepted after length and compatibility validation)
    pub fn trusts_client_signature(&self) -> bool {
        match self {
            ProxyProtocol::OpenAIChat => false,
            ProxyProtocol::OpenAIResponses
            | ProxyProtocol::AnthropicClaude
            | ProxyProtocol::GeminiNative => true,
        }
    }

    /// Whether this protocol emits / echoes thought signatures outbound to the client.
    /// - OpenAIChat: false (OpenAI Chat API only accepts reasoning_content, never emits signatures)
    /// - OpenAIResponses / AnthropicClaude / GeminiNative: true (emits native signature or encrypted field)
    pub fn emits_signature_to_client(&self) -> bool {
        match self {
            ProxyProtocol::OpenAIChat => false,
            ProxyProtocol::OpenAIResponses
            | ProxyProtocol::AnthropicClaude
            | ProxyProtocol::GeminiNative => true,
        }
    }

    /// Protocol display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ProxyProtocol::OpenAIChat => "OPENAI_CHAT",
            ProxyProtocol::OpenAIResponses => "OPENAI_RESPONSES",
            ProxyProtocol::AnthropicClaude => "ANTHROPIC",
            ProxyProtocol::GeminiNative => "GEMINI",
        }
    }
}
