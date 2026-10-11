#[cfg(test)]
mod tests {
    use super::sanitize::normalize_and_sanitize_tool_args;
    use super::transform::transform_openai_response;
    use super::*;
    use crate::proxy::mappers::openai::OpenAIContent;
    use serde_json::json;

    #[test]
    fn test_transform_openai_response() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello!"}]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert_eq!(result.object, "chat.completion");
        let content = match result.choices[0].message.content.as_ref().unwrap() {
            OpenAIContent::String(s) => s,
            _ => panic!("Expected string content"),
        };
        assert_eq!(content, "Hello!");
        assert_eq!(result.choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_usage_metadata_mapping() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 100,
                "candidatesTokenCount": 50,
                "totalTokenCount": 150,
                "cachedContentTokenCount": 25
            },
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);

        assert!(result.usage.is_some());
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert!(usage.prompt_tokens_details.is_some());
        assert_eq!(usage.prompt_tokens_details.unwrap().cached_tokens, Some(25));
    }

    #[test]
    fn test_interactions_usage_metadata_mapping() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "input_tokens_by_modality": [
                    {
                        "modality": "text",
                        "tokens": 7
                    }
                ],
                "total_cached_tokens": 0,
                "total_input_tokens": 7,
                "total_output_tokens": 20,
                "total_thought_tokens": 22,
                "total_tokens": 49,
                "total_tool_use_tokens": 0
            },
            "modelVersion": "gemini-3-flash-preview",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        let usage = result.usage.unwrap();

        assert_eq!(usage.prompt_tokens, 7);
        assert_eq!(usage.completion_tokens, 42);
        assert_eq!(usage.total_tokens, 49);
        assert_eq!(
            usage
                .completion_tokens_details
                .as_ref()
                .unwrap()
                .reasoning_tokens,
            Some(22)
        );

        let responses_usage = usage.to_responses_usage_value();
        assert_eq!(responses_usage["input_tokens"], 7);
        assert_eq!(responses_usage["input_tokens_details"]["cached_tokens"], 0);
        assert_eq!(responses_usage["output_tokens"], 42);
        assert_eq!(
            responses_usage["output_tokens_details"]["reasoning_tokens"],
            22
        );
        assert_eq!(responses_usage["total_tokens"], 49);
    }

    #[test]
    fn test_response_without_usage_metadata() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert!(result.usage.is_none());
    }

    #[test]
    fn test_normalize_and_sanitize_tool_args_passthrough() {
        let mut args = json!({
            "cmd": "ls -la /tmp",
            "description": "Custom description",
            "arbitrary_field": 123
        });
        normalize_and_sanitize_tool_args("shell", &mut args);
        // 验证纯透传：参数原样保持，没有任何字段被重命名、删除或注入
        assert_eq!(args["cmd"], "ls -la /tmp");
        assert_eq!(args["description"], "Custom description");
        assert_eq!(args["arbitrary_field"], 123);
        assert!(!args.as_object().unwrap().contains_key("command"));
    }

    #[test]
    fn test_malformed_function_call_never_injects_hardcoded_online_prompt() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {
                    "parts": []
                },
                "finishReason": "MALFORMED_FUNCTION_CALL"
            }],
            "modelVersion": "gemini-3.7-flash",
            "responseId": "resp_malformed"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert_eq!(result.choices.len(), 1);
        assert_eq!(result.choices[0].finish_reason, Some("stop".to_string()));
        assert!(result.choices[0].message.content.is_none());
    }

    #[test]
    fn test_tool_calls_response_finish_reason_is_tool_calls() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "functionCall": {
                            "name": "read_file",
                            "args": { "path": "src/main.rs" }
                        }
                    }]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_tool"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert_eq!(result.choices.len(), 1);
        assert_eq!(
            result.choices[0].finish_reason,
            Some("tool_calls".to_string())
        );
        assert!(result.choices[0].message.tool_calls.is_some());
    }
}
