// Claude request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

#[test]
fn test_tool_call_inherits_real_signature_without_sentinel() {
    use crate::proxy::mappers::claude::thinking_utils::filter_invalid_thinking_blocks_with_family;
    let real_sig = "Ep4MCpsMARFNMg9NDlK9RXXz5Mzq9mniX9KSQBBzbUx3k85w/qDgtcE+28NH+1EvPeULAprqUquvYXGMzUXGy1xJoMnqdkC4vqebuhyd2Xhs0oz+OhqcOTwLhGYOG0KBKQ87Hfw4q/sMCSgf2gz4vFMa6V6kKMJepYlPXKFJJF4ok+W6lUt3PfYln8K9Dh7wB/40iHiZ2BnJd++6hfUwu9Bz1n795S50l0yCj84EaSCDDF334Erxq7Fo";

    let mut messages = vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::String("List directory".to_string()),
        },
        Message {
            role: "assistant".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::Thinking {
                    thinking: "Confirming conversational readiness".to_string(),
                    signature: Some(real_sig.to_string()),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "call_548709".to_string(),
                    name: "list_directory".to_string(),
                    input: serde_json::json!({}),
                    signature: None,
                    cache_control: None,
                },
            ]),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::ToolResult {
                tool_use_id: "call_548709".to_string(),
                content: serde_json::json!("file1.txt\nfile2.txt"),
                is_error: None,
            }]),
        },
    ];

    // 1. filter_invalid_thinking_blocks_with_family must NOT strip real signature even if family cache is empty
    filter_invalid_thinking_blocks_with_family(&mut messages, Some("gemini"));
    if let MessageContent::Array(blocks) = &messages[1].content {
        assert_eq!(blocks.len(), 2);
        if let ContentBlock::Thinking { signature, .. } = &blocks[0] {
            assert_eq!(
                signature.as_deref(),
                Some(real_sig),
                "Real signature must NOT be stripped!"
            );
        } else {
            panic!("Expected thinking block");
        }
    }

    // 2. transform_claude_request_in should map both thinking and functionCall with the real signature
    let req = ClaudeRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages,
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(12288),
            effort: None,
        }),
        system: None,
        tools: None,
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result = transform_claude_request_in(&req, "test-proj", false, None, "test-session", None)
        .expect("Transform should succeed");

    let contents = result["request"]["contents"]
        .as_array()
        .expect("Contents array");
    let assistant_parts = contents[1]["parts"].as_array().expect("Assistant parts");
    assert_eq!(assistant_parts.len(), 2);
    assert_eq!(assistant_parts[0]["thought"], true);
    assert_eq!(assistant_parts[0]["thoughtSignature"], real_sig);
    assert_eq!(assistant_parts[1]["functionCall"]["name"], "list_directory");
    assert!(
        assistant_parts[1].get("thoughtSignature").is_none(),
        "Claude model functionCall must NOT carry thoughtSignature!"
    );
}

#[test]
fn test_foreign_claude_signature_dropped_for_gemini() {
    use crate::proxy::mappers::claude::thinking_utils::filter_invalid_thinking_blocks_with_family;
    let foreign_claude_sig = "3mgp11XmVXq9InniGA4VAKd7c97NqFw+dWZt79Uz/w9znho88gSM76jv2bZmir7wI86Ixpha7eWdGuznAot4PNbe3+V9bgMTIEyUarn4MLAiiFVb830ZlM+H5ukQwXdD2Zv8nUSmmZTYinpLPGha8TORZAfpU1FJEvwyECel5+W7kc9kpTWrd8DqRNBTOz5EDtvoatiZgKv5SqInhGXK74SJ+PRIC6fNXvYG082HR6TsVxvVYaerz8A40rloIVTxRNK43h3Ecs1boxY4PZqBT8Yhl2qn/iZ+4Xt7FNkI0DAuS9iK0HYKMC4yw0OqKx/LeU+WFZlyc6hGm1BkzLY6yG97MH7kmJ0OPlBWgWFaTeL/uXuGJX6QkKObXN+phoq+kkF2vdFt/mdJMbdgfmSCVQ9037hGBhOHm0zN50KLkp1SxuAY1oWc+lDcI4ufWoyn";

    let mut messages = vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::String("Run tool".to_string()),
        },
        Message {
            role: "assistant".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::Thinking {
                    thinking: "Thinking from foreign Claude model".to_string(),
                    signature: Some(foreign_claude_sig.to_string()),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "call_999999".to_string(),
                    name: "web_fetch".to_string(),
                    input: serde_json::json!({"url": "https://example.com"}),
                    signature: None,
                    cache_control: None,
                },
            ]),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![ContentBlock::ToolResult {
                tool_use_id: "call_999999".to_string(),
                content: serde_json::json!("ok"),
                is_error: None,
            }]),
        },
    ];

    // 1. 验证 filter_invalid_thinking_blocks_with_family 会将非 Gemini 格式的异构签名从思考块中剔除
    filter_invalid_thinking_blocks_with_family(&mut messages, Some("gemini"));
    if let MessageContent::Array(blocks) = &messages[1].content {
        if let ContentBlock::Thinking { signature, .. } = &blocks[0] {
            assert!(
                signature.is_none(),
                "Foreign Claude signature must be stripped for Gemini target!"
            );
        }
    }

    // 2. 验证 transform_claude_request_in 在目标为 Gemini 时，思考块与工具调用的签名均安全降级为哨兵
    let req = ClaudeRequest {
        model: "gemini-3.7-flash-high".to_string(),
        messages,
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(8192),
            effort: None,
        }),
        system: None,
        tools: None,
        stream: false,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-proj", false, None, "test-session-foreign", None)
            .expect("Transform should succeed");

    let contents = result["request"]["contents"]
        .as_array()
        .expect("Contents array");
    let assistant_parts = contents[1]["parts"].as_array().expect("Assistant parts");
    assert_eq!(assistant_parts[0]["thought"], true);
    assert!(
        assistant_parts[0].get("thoughtSignature").is_none(),
        "Thinking block must be clean without signature"
    );
    // 铁律：不兼容的外来 Claude 签名被剥离后**留空**，绝不回退成哨兵。
    // 官方报文里哨兵出现 0/23 次，它不属于 Antigravity 协议。
    assert!(
        assistant_parts[1].get("thoughtSignature").is_none(),
        "Gemini functionCall must drop the foreign signature instead of falling back to sentinel"
    );
}

#[test]
fn test_claude_request_with_corrupt_and_empty_images_defense() {
    let valid_png_b64 = "iVBORw0KGgo=";
    let req = ClaudeRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::Text {
                    text: "Look at these images".to_string(),
                },
                // Corrupt image block 1 (empty data)
                ContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: None,
                        data: None,
                    },
                    cache_control: None,
                },
                // Corrupt image block 2 (invalid short base64 +A==)
                ContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: Some("image/png".to_string()),
                        data: Some("+A==".to_string()),
                    },
                    cache_control: None,
                },
                // Valid image block
                ContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: Some("image/png".to_string()),
                        data: Some(valid_png_b64.to_string()),
                    },
                    cache_control: None,
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

    let result = transform_claude_request_in(&req, "test-proj", false, None, "test-session", None)
        .expect("Transform must not fail on corrupt images");

    let contents = result["request"]["contents"].as_array().unwrap();
    let user_parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(user_parts.len(), 4);
    assert_eq!(user_parts[0]["text"], "Look at these images");
    assert_eq!(
        user_parts[1]["text"],
        "[Image: invalid or corrupted data omitted]"
    );
    assert_eq!(
        user_parts[2]["text"],
        "[Image: invalid or corrupted data omitted]"
    );
    assert!(user_parts[3].get("inlineData").is_some());
    assert_eq!(user_parts[3]["inlineData"]["mimeType"], "image/png");
    assert_eq!(user_parts[3]["inlineData"]["data"], valid_png_b64);
}

#[test]
fn test_gemini_under_v3_thinking_disabled_and_no_thinking_config() {
    // 验证 gemini-2.5-flash 请求，即使客户端带或不带 thinking，也绝对不会注入 thinkingConfig
    let req = ClaudeRequest {
        model: "gemini-2.5-flash".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello 123".to_string()),
        }],
        thinking: None,
        max_tokens: Some(1024),
        temperature: None,
        top_p: None,
        top_k: None,
        stream: false,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-proj", false, None, "test-session", None).unwrap();

    let gen_config = &result["request"]["generationConfig"];
    assert!(
        gen_config.get("thinkingConfig").is_none(),
        "gemini-2.5-flash must NOT have thinkingConfig"
    );
}

#[test]
fn test_gemini_v3_flash_high_thinking_enabled_and_client_budget_ignored() {
    // 验证 gemini-3.7-flash-high 开启思考，并且客户端传的 99999 预算被忽略，强制使用档位字典的 10000
    let req = ClaudeRequest {
        model: "gemini-3.7-flash-high".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        }],
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(99999), // 客户端传入过大/错误预算
            effort: None,
        }),
        max_tokens: Some(1024),
        temperature: None,
        top_p: None,
        top_k: None,
        stream: false,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    let result =
        transform_claude_request_in(&req, "test-proj", false, None, "test-session", None).unwrap();

    let gen_config = &result["request"]["generationConfig"];
    let thinking_config = gen_config
        .get("thinkingConfig")
        .expect("gemini-3.7-flash-high must have thinkingConfig");

    assert_eq!(thinking_config["includeThoughts"], true);
    assert_eq!(
        thinking_config["thinkingBudget"], 16384,
        "Client budget (99999) must be ignored in favor of tier dictionary budget (16384)"
    );
}

#[test]
fn test_gemini_client_billing_metadata_filtering() {
    // [Issue #3452] Standalone billing header line should be filtered for Gemini targets
    assert!(is_gemini_client_billing_metadata(
        "gemini-3.8-flash-high",
        "x-anthropic-billing-header: cc_version=2.1.270.ffc; cc_entrypoint=claude-desktop-3p;"
    ));
    assert!(is_gemini_client_billing_metadata(
        "gemini-2.5-flash",
        "  x-anthropic-billing-header: cc_entrypoint=desktop; \n"
    ));

    // 风险清洗不分 model：非 Gemini 模型同样必须过滤
    // （任何客户端注入的计费元数据都是风险，与当前模型无关）
    assert!(is_gemini_client_billing_metadata(
        "claude-sonnet-4-6",
        "x-anthropic-billing-header: cc_version=2.1.270.ffc;"
    ));

    // Multiline text should NOT be filtered
    assert!(!is_gemini_client_billing_metadata(
        "gemini-3.8-flash-high",
        "x-anthropic-billing-header: test;\nSecond line prompt instruction"
    ));

    // Unrelated system prompt should NOT be filtered
    assert!(!is_gemini_client_billing_metadata(
        "gemini-3.8-flash-high",
        "You are an expert coder."
    ));
}

#[test]
fn test_claude_desktop_billing_metadata_filtered_in_transform_for_gemini() {
    let req: ClaudeRequest = serde_json::from_value(json!({
        "model": "gemini-3.8-flash-high",
        "messages": [{"role": "user", "content": "Hello"}],
        "system": [
            {
                "type": "text",
                "text": "x-anthropic-billing-header: cc_version=2.1.270.ffc; cc_entrypoint=claude-desktop-3p;"
            },
            {
                "type": "text",
                "text": "You are a helpful assistant."
            }
        ]
    }))
    .expect("ClaudeRequest should deserialize");

    let body = transform_claude_request_in(&req, "test-project", false, None, "test-session", None)
        .expect("Request should transform");
    let system_parts = body["request"]["systemInstruction"]["parts"]
        .as_array()
        .expect("system instruction should contain parts");
    let system_texts = system_parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<Vec<_>>();

    // Billing header must be filtered out to avoid Gemini 429 RESOURCE_EXHAUSTED (#3452)
    assert!(!system_texts
        .iter()
        .any(|t| t.contains("x-anthropic-billing-header:")));
    // Normal prompt must be preserved
    assert!(system_texts.contains(&"You are a helpful assistant."));
}

#[test]
fn test_claude_tool_result_multimodal_string_extraction() {
    let fake_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let tool_content = format!(
        "Here is the screenshot: ![screen](data:image/png;base64,{}) and log text",
        fake_b64
    );

    let req: ClaudeRequest = serde_json::from_value(json!({
        "model": "gemini-2.5-flash",
        "messages": [
            {
                "role": "user",
                "content": "Take screenshot"
            },
            {
                "role": "assistant",
                "content": [
                    {
                        "type": "tool_use",
                        "id": "toolu_shot_1",
                        "name": "screenshot",
                        "input": {}
                    }
                ]
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_shot_1",
                        "content": tool_content
                    }
                ]
            }
        ]
    }))
    .expect("ClaudeRequest should deserialize");

    let body = transform_claude_request_in(&req, "test-project", false, None, "test-session", None)
        .expect("Request should transform");
    let contents = body["request"]["contents"]
        .as_array()
        .expect("contents array");
    let tool_parts = contents[2]["parts"].as_array().expect("tool turn parts");

    // 验证同时存在 functionResponse 和 inlineData 两个 parts
    assert_eq!(tool_parts.len(), 2);
    assert!(tool_parts[0].get("functionResponse").is_some());
    assert!(tool_parts[1].get("inlineData").is_some());

    let inline_data = &tool_parts[1]["inlineData"];
    assert_eq!(inline_data["mimeType"], "image/png");
    assert_eq!(inline_data["data"], fake_b64);

    let res_str = tool_parts[0]["functionResponse"]["response"]["result"]
        .as_str()
        .unwrap();
    assert!(!res_str.contains(fake_b64));
    assert!(res_str.contains("[Image: forwarded to visual input (image/png)]"));
}
