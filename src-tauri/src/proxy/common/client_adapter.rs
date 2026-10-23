use super::client_adapters::OpencodeAdapter;
use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use std::sync::Arc; // [NEW] Import Arc

/// client adapter trait
///
/// Provide customized protocol processing strategies for different clients (such as opencode, Cherry Studio).
/// Each client can implement its own adapter to handle specific needs.
///
/// # design principles
/// 1. **Complete isolation**: The adapter serves as an optional enhancement layer and does not modify the core logic of the existing protocol.
/// 2. **Backward Compatibility**: Requests that do not match the adapter will be processed completely according to the existing process.
/// 3. **Single file modification**: Client-specific logic is encapsulated in respective adapter files
pub trait ClientAdapter: Send + Sync {
    /// Determines whether this adapter matches the given request
    ///
    /// # Arguments
    /// * `headers` - request headers, usually identifying the client through fields such as User-Agent
    ///
    /// # Returns
    /// Returns true if there is a match, false otherwise
    fn matches(&self, headers: &HeaderMap) -> bool;

    /// Whether to bypass signature verification
    ///
    /// Some clients may not require strict thinking signature matching
    #[allow(dead_code)]
    fn bypass_signature_matching(&self) -> bool {
        false
    }

    /// Whether to adopt the "let it crash" philosophy
    ///
    /// Reduce unnecessary retries and recovery logic to quickly expose errors
    fn let_it_crash(&self) -> bool {
        false
    }

    /// Signature caching strategy
    ///
    /// Different clients may require different signature management methods (FIFO/LIFO)
    fn signature_buffer_strategy(&self) -> SignatureBufferStrategy {
        SignatureBufferStrategy::Default
    }

    /// Inject the missing Beta Header on the client
    ///
    /// Some clients may require specific Beta Headers to work properly
    fn inject_beta_headers(&self, _headers: &mut HeaderMap) {
        // Not injected by default
    }

    /// Declare supported protocols
    ///
    /// for multi-protocol clients (like opencode)
    #[allow(dead_code)]
    fn supported_protocols(&self) -> Vec<Protocol> {
        vec![Protocol::Anthropic] // Only Anthropic is supported by default
    }
}

/// Signature caching strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureBufferStrategy {
    /// Default strategy (current implementation)
    Default,
    /// FIFO (first in, first out) - suitable for multiple concurrent tool calls
    Fifo,
    /// LIFO (last in, first out) - suitable for nested calls
    #[allow(dead_code)]
    Lifo,
}

/// Supported protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Protocol {
    Anthropic,
    OpenAI,
    OACompatible,
    GoogleGemini,
}

/// Global client adapter registry
///
/// All registered adapters are checked during request processing
pub static CLIENT_ADAPTERS: Lazy<Vec<Arc<dyn ClientAdapter>>> = Lazy::new(|| {
    vec![
        Arc::new(OpencodeAdapter),
        // More adapters can be easily added in the future:
        // Arc::new(CherryStudioAdapter),
    ]
});

/// Helper function: extract User-Agent from HeaderMap
pub fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    struct TestAdapter;

    impl ClientAdapter for TestAdapter {
        fn matches(&self, headers: &HeaderMap) -> bool {
            get_user_agent(headers)
                .map(|ua| ua.contains("test-client"))
                .unwrap_or(false)
        }

        fn bypass_signature_matching(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_adapter_matches() {
        let adapter = TestAdapter;

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("test-client/1.0"));

        assert!(adapter.matches(&headers));
        assert!(adapter.bypass_signature_matching());
    }

    #[test]
    fn test_adapter_no_match() {
        let adapter = TestAdapter;

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("other-client/1.0"));

        assert!(!adapter.matches(&headers));
    }

    #[test]
    fn test_get_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("opencode/1.0"));

        assert_eq!(get_user_agent(&headers), Some("opencode/1.0".to_string()));
    }
}
