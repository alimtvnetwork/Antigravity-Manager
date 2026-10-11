use super::*;
use crate::proxy::mappers::claude::models::{ContentBlock, Message, MessageContent};
use crate::proxy::mappers::claude::ClaudeRequest;

#[test]
fn test_is_warmup_request_strictly_exact() {
    // 1. 严格全等为 Warmup 的请求
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

    // 2. 带后续句子的真实用户问题，绝不误杀！
    let real_question_req = ClaudeRequest {
        model: "claude-3-7-sonnet".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Warmup function in PyTorch 怎么写？".to_string()),
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

    // 3. 包含 ToolResult 的消息，绝不误杀！
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
