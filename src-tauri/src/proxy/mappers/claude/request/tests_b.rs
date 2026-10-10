// Claude request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

#[test]
fn test_thinking_block_not_prepend_when_disabled() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // 验证当 thinking 未启用且模型非思考模型时,不会补全 thinking 块
    let req = ClaudeRequest {
        model: "non-reasoning-model".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::String("Hello".to_string()),
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Array(vec![ContentBlock::Text {
                    text: "Response".to_string(),
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
        thinking: None, // 未启用 thinking
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result = transform_claude_request_in(
        &req,
        "test-project",
        false,
        None,
        "test_session_non_thinking",
        None,
    );
    assert!(result.is_ok());

    let body = result.unwrap();
    let contents = body["request"]["contents"].as_array().unwrap();

    let last_model_msg = contents
        .iter()
        .rev()
        .find(|c| c["role"] == "model")
        .unwrap();

    let parts = last_model_msg["parts"].as_array().unwrap();

    // 验证没有补全 thinking 块
    assert_eq!(parts.len(), 1, "Should only have the original text block");
    assert_eq!(parts[0]["text"], "Response");
}

#[test]
fn test_thinking_block_empty_content_fix() {
    // [场景] 客户端发送了一个内容为空的 thinking 块
    // 期望: 自动填充 "..."
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![Message {
            role: "assistant".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::Thinking {
                    thinking: "".to_string(), // 空内容
                    signature: Some("sig".to_string()),
                    cache_control: None,
                },
                ContentBlock::Text {
                    text: "Hi".to_string(),
                },
            ]),
        }],
        system: None,
        tools: None,
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
    assert!(result.is_ok(), "Transformation failed");
    let body = result.unwrap();
    let contents = body["request"]["contents"].as_array().unwrap();
    let parts = contents[0]["parts"].as_array().unwrap();

    // 验证空 thinking 块被降级为包含 "..." 的非 thought 文本部分（并与后续文本紧凑合并）
    let downgraded_part = parts.iter().find(|p| {
        p.get("text")
            .and_then(|t| t.as_str())
            .map(|s| s.contains("..."))
            .unwrap_or(false)
            && p.get("thought").is_none()
    });
    assert!(
        downgraded_part.is_some(),
        "Empty thinking should be downgraded to text without thought: true"
    );
}

#[test]
fn test_redacted_thinking_degradation() {
    // [场景] 客户端包含 RedactedThinking
    // 期望: 降级为普通文本，不带 thought: true
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![Message {
            role: "assistant".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::RedactedThinking {
                    data: "some data".to_string(),
                },
                ContentBlock::Text {
                    text: "Hi".to_string(),
                },
            ]),
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
    let parts = body["request"]["contents"][0]["parts"].as_array().unwrap();

    // 验证 RedactedThinking -> Text 存在且不带 thought: true
    let redacted_part = parts
        .iter()
        .find(|p| {
            p.get("text")
                .and_then(|t| t.as_str())
                .map_or(false, |t| t.contains("[Redacted Thinking: some data]"))
        })
        .expect("Should find redacted thinking degraded text");
    assert!(
        redacted_part.get("thought").is_none(),
        "Redacted thinking should NOT have thought: true"
    );
}

#[test]
fn test_thinking_blocks_sorted_first_after_compression() {
    // Simulate kilo context compression reordering: text BEFORE thinking
    let mut messages = vec![Message {
        role: "assistant".to_string(),
        content: MessageContent::Array(vec![
            // Wrong order: Text before Thinking (simulates kilo compression)
            ContentBlock::Text {
                text: "Some regular text".to_string(),
            },
            ContentBlock::Thinking {
                thinking: "My thinking process".to_string(),
                signature: Some(
                    "valid_signature_1234567890_abcdefghij_klmnopqrstuvwxyz_test".to_string(),
                ),
                cache_control: None,
            },
            ContentBlock::Text {
                text: "More text".to_string(),
            },
        ]),
    }];

    // Apply the fix
    sort_thinking_blocks_first(&mut messages);

    // Verify thinking is now first
    if let MessageContent::Array(blocks) = &messages[0].content {
        assert_eq!(blocks.len(), 3, "Should still have 3 blocks");
        assert!(
            matches!(blocks[0], ContentBlock::Thinking { .. }),
            "Thinking should be first"
        );
        assert!(
            matches!(blocks[1], ContentBlock::Text { .. }),
            "Text should be second"
        );
        assert!(
            matches!(blocks[2], ContentBlock::Text { .. }),
            "Text should be third"
        );

        // Verify content preserved
        if let ContentBlock::Thinking { thinking, .. } = &blocks[0] {
            assert_eq!(thinking, "My thinking process");
        }
    } else {
        panic!("Expected Array content");
    }
}

#[test]
fn test_thinking_blocks_no_reorder_when_already_first() {
    // Correct order: Thinking already first - should not trigger reorder
    let mut messages = vec![Message {
        role: "assistant".to_string(),
        content: MessageContent::Array(vec![
            ContentBlock::Thinking {
                thinking: "My thinking".to_string(),
                signature: Some("sig123".to_string()),
                cache_control: None,
            },
            ContentBlock::Text {
                text: "Some text".to_string(),
            },
        ]),
    }];

    // Apply the fix (should be no-op)
    sort_thinking_blocks_first(&mut messages);

    // Verify order unchanged
    if let MessageContent::Array(blocks) = &messages[0].content {
        assert!(
            matches!(blocks[0], ContentBlock::Thinking { .. }),
            "Thinking should still be first"
        );
        assert!(
            matches!(blocks[1], ContentBlock::Text { .. }),
            "Text should still be second"
        );
    }
}

#[test]
fn test_merge_consecutive_messages() {
    let mut messages = vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::Text {
                text: "World".to_string(),
            }]),
        },
        Message {
            role: "assistant".to_string(),
            content: MessageContent::String("Hi".to_string()),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::ToolResult {
                tool_use_id: "test_id".to_string(),
                content: serde_json::json!("result"),
                is_error: None,
            }]),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::Text {
                text: "System Reminder".to_string(),
            }]),
        },
    ];

    merge_consecutive_messages(&mut messages);

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    if let MessageContent::Array(blocks) = &messages[0].content {
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected text block"),
        }
        match &blocks[1] {
            ContentBlock::Text { text } => assert_eq!(text, "World"),
            _ => panic!("Expected text block"),
        }
    } else {
        panic!("Expected array content at index 0");
    }

    assert_eq!(messages[1].role, "assistant");

    assert_eq!(messages[2].role, "user");
    if let MessageContent::Array(blocks) = &messages[2].content {
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            ContentBlock::ToolResult { tool_use_id, .. } => assert_eq!(tool_use_id, "test_id"),
            _ => panic!("Expected tool_result block"),
        }
        match &blocks[1] {
            ContentBlock::Text { text } => assert_eq!(text, "System Reminder"),
            _ => panic!("Expected text block"),
        }
    } else {
        panic!("Expected array content at index 2");
    }
}

#[test]
fn test_default_max_tokens() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let req = ClaudeRequest {
        model: "non-reasoning-model".to_string(),
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
        transform_claude_request_in(&req, "test-v", false, None, "test_session", None).unwrap();
    // [FIX] Since we removed the default 81920, maxOutputTokens should NOT be present
    // when max_tokens is None and thinking is disabled
    let gen_config = &result["request"]["generationConfig"];
    assert!(
        gen_config.get("maxOutputTokens").is_none(),
        "maxOutputTokens should not be set when max_tokens is None"
    );
}

#[test]
fn test_claude_flash_thinking_budget_capping() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Use full path or ensure import of ThinkingConfig
    // transform_claude_request and models are needed.
    // Assuming models are available via super imports, but let's be explicit if needed.

    // Setup request with high budget
    let req = ClaudeRequest {
        model: "gemini-2.0-flash-thinking-exp".to_string(), // Contains "flash"
        messages: vec![],
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(32000),
            effort: None,
        }),
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None, // Added missing field
        stream: false,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "proj", false, None, "test_session", None).unwrap();
    let budget = result["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();
    assert_eq!(budget, 24576); // capped by model_specs.get_thinking_budget("gemini-2.0-flash-thinking-exp")

    // Setup request for Pro thinking model (mock name for testing)
    let req_pro = ClaudeRequest {
        model: "gemini-2.0-pro-thinking-exp".to_string(), // Contains "thinking" but not "flash"
        messages: vec![],
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(32000),
            effort: None,
        }),
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None, // Added missing field
        stream: false,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    // Should cap
    let result_pro =
        transform_claude_request_in(&req_pro, "proj", false, None, "test_session", None).unwrap();
    assert_eq!(
        result_pro["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        49152
    );
}
