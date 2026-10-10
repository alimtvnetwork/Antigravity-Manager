// V1 request wrapper (split from wrapper.rs).
use bytes::Bytes;
use serde_json::{json, Value};

pub fn wrap_request(
    body: &Value,
    project_id: &str,
    mapped_model: &str,
    account_id: Option<&str>,
    session_id: Option<&str>,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
) -> Value {
    wrap_request_v2(
        body,
        project_id,
        mapped_model,
        account_id,
        session_id,
        token,
        None, // token_manager：不带 → Layer-3 本就不会触发
        None, // upstream：同上，无需客户端
    )
}

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod test_fixes {
    use crate::proxy::mappers::gemini::wrapper::request_v2::wrap_request_v2;
    use serde_json::json;

    #[test]
    fn test_wrap_request_with_signature() {
        let session_id = "test-session-sig";
        let signature = "test-signature-must-be-longer-than-fifty-characters-to-be-cached-by-signature-cache-12345"; // > 50 chars
        crate::proxy::SignatureCache::global().cache_session_signature(
            session_id,
            signature.to_string(),
            1,
        );

        let body = json!({
            "model": "gemini-pro",
            "contents": [{
                "role": "user",
                "parts": [{
                    "functionCall": {
                        "name": "get_weather",
                        "args": {"location": "London"}
                    }
                }]
            }]
        });

        let result = wrap_request(&body, "proj", "gemini-pro", None, Some(session_id), None);
        let injected_sig = result["request"]["contents"][0]["parts"][0]["thoughtSignature"]
            .as_str()
            .unwrap();
        assert!(
            injected_sig == signature
                || injected_sig == crate::proxy::thinking_store::SENTINEL_SIGNATURE
        );
    }

    #[test]
    fn test_wrap_request_with_snake_case_signature() {
        let body = json!({
            "model": "gemini-pro",
            "contents": [{
                "role": "user",
                "parts": [{
                    "functionCall": {
                        "name": "get_weather",
                        "args": {"location": "London"}
                    },
                    "thought_signature": "client-sent-signature-value-12345"
                }]
            }]
        });

        let result = wrap_request(&body, "proj", "gemini-pro", None, None, None);
        let part = &result["request"]["contents"][0]["parts"][0];
        assert_eq!(
            part["thoughtSignature"].as_str(),
            Some("client-sent-signature-value-12345")
        );
        assert!(
            part.get("thought_signature").is_none(),
            "Snake case 'thought_signature' must be stripped when sending to Google"
        );
    }
}
