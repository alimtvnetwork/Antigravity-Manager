use super::*;
use crate::proxy::mappers::claude::models::{ContentBlock, Message, MessageContent};
use crate::proxy::mappers::claude::ClaudeRequest;

#[test]
fn test_is_warmup_request_strictly_exact() {
    // 1. Requests that are strictly congruent to Warmup
    let exact_req = ClaudeRequest {
        model: "claude-3-7-sonnet".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Warmup".to_string()),
        }],
        system: None,
        max_tokens: Some(100),
        stream: false,
        temperature: None,
        top_p: None,
        top_k: None,
        tools: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };
    assert!(is_warmup_request(&exact_req));

    // 2. Real user questions with follow-up sentences, never accidentally kill!
    let real_question_req = ClaudeRequest {
        model: "claude-3-7-sonnet".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("How to write Warmup function in PyTorch?".to_string()),
        }],
        system: None,
        max_tokens: Some(100),
        stream: false,
        temperature: None,
        top_p: None,
        top_k: None,
        tools: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };
    assert!(!is_warmup_request(&real_question_req));

    // 3. Messages containing ToolResult will never be accidentally killed!
    let tool_error_req = ClaudeRequest {
        model: "claude-3-7-sonnet".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::ToolResult {
                tool_use_id: "tool_1".to_string(),
                content: serde_json::json!("Warmup failed: connection refused"),
                is_error: Some(true),
            }]),
        }],
        system: None,
        max_tokens: Some(100),
        stream: false,
        temperature: None,
        top_p: None,
        top_k: None,
        tools: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };
    assert!(!is_warmup_request(&tool_error_req));
}
