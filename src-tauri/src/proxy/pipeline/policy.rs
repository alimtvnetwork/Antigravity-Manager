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

/// Upstream response and error classification in the unified pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpstreamClassification {
    /// Authentic upstream rate limit / quota exhaustion (429 or 529 only)
    RateLimited { retry_after: Option<u64> },
    /// Model not found or unsupported (404 or model not found payload cue)
    ModelNotFound,
    /// Transient server error (500/503; switch account/backoff, never lock account)
    TransientServerError,
    /// Internal gateway self-generated message (e.g. All accounts limited; never lock)
    InternalGatewayMessage,
    /// Thought signature error or cross-model signature corruption (400 Invalid signature in thinking block)
    ThoughtSignatureError,
    /// Other generic client error
    OtherClientError(u16),
}

impl UpstreamClassification {
    /// Unified classifier: protocol agnostic
    pub fn classify(status: u16, body: &str, retry_after_header: Option<&str>) -> Self {
        let lower = body.to_lowercase();
        // 1. Exclude gateway internal errors first
        if lower.contains("all accounts limited")
            || lower.contains("no accounts available")
            || lower.contains("all accounts failed")
            || lower.contains("token pool is empty")
            || lower.contains("all accounts exhausted")
            || lower.contains("all accounts unhealthy")
        {
            return UpstreamClassification::InternalGatewayMessage;
        }

        // 2. Determine if model does not exist
        let has_model_not_found_cue = lower.contains("model not found")
            || lower.contains("unknown model")
            || lower.contains("does not exist")
            || lower.contains("is not found")
            || lower.contains("unsupported model")
            || lower.contains("not found for api version")
            || lower.contains("publisher model")
            || lower.contains("model_not_found")
            || lower.contains("no such model")
            || lower.contains("invalid model")
            || lower.contains("model is not available");

        if status == 404 || has_model_not_found_cue {
            return UpstreamClassification::ModelNotFound;
        }

        // 3. Determine if thought signature is invalid or foreign
        let has_signature_error_cue = lower.contains("invalid thought signature")
            || lower.contains("invalid `signature`")
            || lower.contains("invalid signature")
            || lower.contains("thought_signature")
            || lower.contains("thoughtsignature")
            || lower.contains("thinking.signature")
            || lower.contains("thinking.thinking")
            || lower.contains("corrupted thought signature");

        if status == 400 && has_signature_error_cue {
            return UpstreamClassification::ThoughtSignatureError;
        }

        // 4. Rate limiting (429 and 529 only)
        if status == 429 || status == 529 {
            let delay = crate::proxy::upstream::retry::parse_retry_delay(body, retry_after_header)
                .map(|ms| ms.saturating_add(999) / 1000);
            return UpstreamClassification::RateLimited { retry_after: delay };
        }

        // 5. Transient server errors (500 / 503)
        if status == 500 || status == 503 {
            return UpstreamClassification::TransientServerError;
        }

        UpstreamClassification::OtherClientError(status)
    }

    /// Whether this classification should trigger account circuit breaking in TokenManager
    pub fn should_lock_account(&self) -> bool {
        matches!(self, UpstreamClassification::RateLimited { .. })
    }

    /// Whether this classification indicates model not found
    pub fn is_model_not_found(&self) -> bool {
        matches!(self, UpstreamClassification::ModelNotFound)
    }

    /// Whether this classification indicates thought signature error
    pub fn is_thought_signature_error(&self) -> bool {
        matches!(self, UpstreamClassification::ThoughtSignatureError)
    }

    /// Whether this classification indicates an internal gateway message
    pub fn is_internal_gateway_message(&self) -> bool {
        matches!(self, UpstreamClassification::InternalGatewayMessage)
    }
}
