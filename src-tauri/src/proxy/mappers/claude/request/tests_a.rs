// Claude request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use crate::proxy::common::json_schema::clean_json_schema;
use crate::proxy::mappers::claude::request::messages::normalize_claude_client_identity;
use serde_json::json;

#[test]
fn test_agent_sdk_identity_is_normalized_for_antigravity() {
    let req: ClaudeRequest = serde_json::from_value(json!({
        "model": "claude-sonnet-4-6",
        "messages": [{"role": "user", "content": "Reply with ok"}],
        "system": [
            {
                "type": "text",
                "text": "x-anthropic-billing-header: cc_entrypoint=sdk-cli;"
            },
            {
                "type": "text",
                "text": CLAUDE_AGENT_SDK_IDENTITY,
                "cache_control": {"type": "ephemeral"}
            }
        ]
    }))
    .expect("Agent SDK request should deserialize");

    let body = transform_claude_request_in(&req, "test-project", false, None, "test-session", None)
        .expect("Agent SDK request should transform");
    let system_parts = body["request"]["systemInstruction"]["parts"]
        .as_array()
        .expect("system instruction should contain parts");
    let system_texts = system_parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<Vec<_>>();

    assert!(system_texts.contains(&CLAUDE_CODE_CLI_IDENTITY));
    assert!(!system_texts.contains(&CLAUDE_AGENT_SDK_IDENTITY));
    assert!(!system_texts.contains(&"x-anthropic-billing-header: cc_entrypoint=sdk-cli;"));
}

#[test]
fn claude_transform_does_not_inject_legacy_stop_sequences() {
    let req: ClaudeRequest = serde_json::from_value(json!({
        "model": "claude-sonnet-4-6",
        "messages": [{"role": "user", "content": "Reply with ok"}]
    }))
    .expect("request should deserialize");

    let body = transform_claude_request_in(&req, "test-project", false, None, "test-session", None)
        .expect("request should transform");
    let gen_config = &body["request"]["generationConfig"];
    assert!(
        gen_config.get("stopSequences").is_none(),
        "legacy stopSequences must not be injected: {gen_config}"
    );
}

#[test]
fn test_agent_sdk_identity_mention_is_not_rewritten() {
    let quoted_identity = format!("Compatibility note: {CLAUDE_AGENT_SDK_IDENTITY}");

    assert_eq!(
        normalize_claude_client_identity(&quoted_identity),
        quoted_identity
    );
}

#[test]
fn test_ephemeral_injection_debug() {
    // This test simulates the issue where cache_control might be injected
    let json_with_null = json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {
                        "type": "thinking",
                        "thinking": "test",
                        "signature": "sig_1234567890",
                        "cache_control": null
                    }
                ]
            }
        ]
    });

    let req: ClaudeRequest = serde_json::from_value(json_with_null).unwrap();
    if let MessageContent::Array(blocks) = &req.messages[0].content {
        if let ContentBlock::Thinking { cache_control, .. } = &blocks[0] {
            assert!(
                cache_control.is_none(),
                "Deserialization should result in None for null cache_control"
            );
        }
    }

    // Now test serialization
    let serialized = serde_json::to_value(&req).unwrap();
    println!("Serialized: {}", serialized);
    assert!(serialized["messages"][0]["content"][0]
        .get("cache_control")
        .is_none());
}

#[test]
fn test_simple_request() {
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        }],
        system: None,
        tools: None,
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-project", false, None, "test_session", None);
    assert!(result.is_ok());

    let body = result.unwrap();
    assert_eq!(body["project"], "test-project");
    assert!(body["requestId"].as_str().unwrap().starts_with("agent/"));
}

#[test]
fn test_clean_json_schema() {
    let mut schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA",
                "minLength": 1,
                "exclusiveMinimum": 0
            },
            "unit": {
                "type": ["string", "null"],
                "enum": ["celsius", "fahrenheit"],
                "default": "celsius"
            },
            "date": {
                "type": "string",
                "format": "date"
            }
        },
        "required": ["location"]
    });

    clean_json_schema(&mut schema);

    // Check removed fields
    assert!(schema.get("$schema").is_none());
    assert!(schema.get("additionalProperties").is_none());
    assert!(schema["properties"]["location"].get("minLength").is_none());
    assert!(schema["properties"]["unit"].get("default").is_none());
    assert!(schema["properties"]["date"].get("format").is_none());

    // Check union type handling ["string", "null"] -> "string"
    assert_eq!(schema["properties"]["unit"]["type"], "string");

    // Check types are lowercased
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["location"]["type"], "string");
    assert_eq!(schema["properties"]["date"]["type"], "string");
}

#[test]
fn test_complex_tool_result() {
    let req = ClaudeRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::String("Run command".to_string()),
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Array(vec![ContentBlock::ToolUse {
                    id: "call_1".to_string(),
                    name: "run_command".to_string(),
                    input: json!({"command": "ls"}),
                    signature: None,
                    cache_control: None,
                }]),
            },
            Message {
                role: "user".to_string(),
                content: MessageContent::Array(vec![ContentBlock::ToolResult {
                    tool_use_id: "call_1".to_string(),
                    content: json!([
                        {"type": "text", "text": "file1.txt\n"},
                        {"type": "text", "text": "file2.txt"}
                    ]),
                    is_error: Some(false),
                }]),
            },
        ],
        system: None,
        tools: None,
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-project", false, None, "test_session", None);
    assert!(result.is_ok());

    let body = result.unwrap();
    let contents = body["request"]["contents"].as_array().unwrap();

    // Check the tool result message (last message)
    let tool_resp_msg = &contents[2];
    let parts = tool_resp_msg["parts"].as_array().unwrap();
    let func_resp = &parts[0]["functionResponse"];

    assert_eq!(func_resp["name"], "run_command");
    assert_eq!(func_resp["id"], "call_1");

    // Verify merged content
    let resp_text = func_resp["response"]["result"].as_str().unwrap();
    assert!(resp_text.contains("file1.txt"));
    assert!(resp_text.contains("file2.txt"));
    assert!(resp_text.contains("\n"));
}

#[test]
fn test_cache_control_cleanup() {
    // Simulate historical messages containing cache_control sent by the VS Code plug-in
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::String("Hello".to_string()),
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Array(vec![
                    ContentBlock::Thinking {
                        thinking: "Let me think...".to_string(),
                        signature: Some("sig123".to_string()),
                        cache_control: Some(json!({"type": "ephemeral"})), // This should be cleaned up
                    },
                    ContentBlock::Text {
                        text: "Here is my response".to_string(),
                    },
                ]),
            },
            Message {
                role: "user".to_string(),
                content: MessageContent::Array(vec![ContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: Some("image/png".to_string()),
                        data: Some("iVBORw0KGgo=".to_string()),
                    },
                    cache_control: Some(json!({"type": "ephemeral"})), // This should be cleaned up too
                }]),
            },
        ],
        system: None,
        tools: None,
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        thinking: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-project", false, None, "test_session", None);
    assert!(result.is_ok());

    // Verify that the request was successfully converted
    let body = result.unwrap();
    assert_eq!(body["project"], "test-project");

    // NOTE: The cleanup of cache_control happens internally and we cannot verify it directly from the JSON output
    // But if it is not cleaned, an error will be reported when sending it to the Anthropic API later.
    // This test mainly ensures that the cleanup logic does not cause the conversion to fail
}

#[test]
fn test_thinking_mode_auto_disable_on_tool_use_history() {
    // [Scenario] There is a tool call chain in the historical message, and the Assistant message does not have a Thinking block
    // Expectation: The system no longer blindly downgrades and disables thinking, but retains thinkingConfig and completes the guaranteed thinking block.
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::String("Check files".to_string()),
            },
            // Assistant uses tools, but in non-Thinking mode
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Array(vec![
                    ContentBlock::Text {
                        text: "Checking...".to_string(),
                    },
                    ContentBlock::ToolUse {
                        id: "tool_1".to_string(),
                        name: "list_files".to_string(),
                        input: json!({}),
                        cache_control: None,
                        signature: None,
                    },
                ]),
            },
            // User returns tool results
            Message {
                role: "user".to_string(),
                content: MessageContent::Array(vec![ContentBlock::ToolResult {
                    tool_use_id: "tool_1".to_string(),
                    content: serde_json::Value::String("file1.txt\nfile2.txt".to_string()),
                    is_error: Some(false),
                }]),
            },
        ],
        system: None,
        tools: Some(vec![Tool {
            name: Some("list_files".to_string()),
            description: Some("List files".to_string()),
            input_schema: Some(json!({"type": "object"})),
            type_: None,
        }]),
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(1024),
            effort: None,
        }),
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-project", false, None, "test_session", None);
    assert!(result.is_ok());

    let body = result.unwrap();
    let request = &body["request"];

    // Verification: thinkingConfig must be retained in generationConfig (no longer demoted and removed due to unsigned historical messages or no thinking block)
    let gen_config = request
        .get("generationConfig")
        .expect("Should have generationConfig");
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig must be preserved per server-side thinking persistence policy"
    );

    // Verification: The thinking block is completed in the historical Assistant message to avoid upstream 400
    let contents = request["contents"].as_array().expect("Contents array");
    let assistant_msg = &contents[1];
    let parts = assistant_msg["parts"].as_array().expect("Parts array");
    assert!(
        parts.iter().any(|p| p.get("thought") == Some(&json!(true))),
        "Assistant message must contain a thinking block"
    );
}
