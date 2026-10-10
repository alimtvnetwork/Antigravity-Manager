// Claude request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

#[test]
fn test_gemini_pro_thinking_support() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Setup request for Gemini Pro (no -thinking suffix)
    let req = ClaudeRequest {
        model: "gemini-3-pro-preview".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        }],
        thinking: Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(16000),
            effort: None,
        }),
        max_tokens: None,
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

    // Transform
    let result =
        transform_claude_request_in(&req, "proj", false, None, "test_session", None).unwrap();
    let gen_config = &result["request"]["generationConfig"];

    // thinkingConfig should be present (not forced disabled)
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig should be preserved for gemini-3-pro"
    );

    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();
    // In Auto mode, client's budget is ignored and bare gemini-3-pro defaults to 10001
    assert_eq!(budget, 10001);
}

#[test]
fn test_gemini_pro_default_thinking() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Setup request for Gemini Pro WITHOUT thinking config
    let req = ClaudeRequest {
        model: "gemini-3-pro-preview".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        }],
        thinking: None, // No thinking config provided by client
        max_tokens: None,
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

    // Transform
    let result =
        transform_claude_request_in(&req, "proj", false, None, "test_session", None).unwrap();
    let gen_config = &result["request"]["generationConfig"];

    // thinkingConfig SHOULD be injected because of default-on logic
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig should be auto-enabled for gemini-3-pro"
    );
}

#[test]
fn test_claude_image_thinking_mode_disabled() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // 1. Force image thinking mode to "disabled"
    crate::proxy::config::update_image_thinking_mode(Some("disabled".to_string()));
    struct ImageResetGuard;
    impl Drop for ImageResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_image_thinking_mode(Some("enabled".to_string()));
        }
    }
    let _guard = ImageResetGuard;

    // 2. Setup Claude request for an image model (mapped to gemini-3-pro-image)
    let req = ClaudeRequest {
        model: "gemini-3-pro-image".to_string(), // Explicitly use recognized image model
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Draw a cat".to_string()),
        }],
        thinking: None,
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        stream: false,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: Some("1024x1024".to_string()),
        quality: Some("hd".to_string()),
    };

    // 3. Transform request
    let result =
        transform_claude_request_in(&req, "test-proj", false, None, "test_session", None).unwrap();

    // 4. Verify thinkingConfig has includeThoughts: false
    let gen_config = result["request"]["generationConfig"]
        .as_object()
        .expect("Should have generationConfig");
    let thinking_config = gen_config
        .get("thinkingConfig")
        .and_then(|t| t.as_object())
        .expect("Should have thinkingConfig (explicitly disabled)");

    assert_eq!(thinking_config["includeThoughts"], false);
}

#[test]
fn test_claude_adaptive_global_config() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Set global config to Adaptive + High effort
    let config = ThinkingBudgetConfig {
        mode: crate::proxy::config::ThinkingBudgetMode::Adaptive,
        custom_value: 0,
        effort: Some("high".to_string()),
        ..Default::default()
    };
    crate::proxy::config::update_thinking_budget_config(config);
    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = ResetGuard;

    let req = ClaudeRequest {
        model: "claude-3-7-sonnet-thinking".to_string(), // thinking capable
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("test".to_string()),
        }],
        thinking: None, // No client thinking config
        stream: false,
        // ... minimal fields
        max_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        system: None,
        tools: None,
        metadata: None,
        output_config: None,
        size: None,
        quality: None,
    };

    // Transform
    let result =
        transform_claude_request_in(&req, "test-proj", false, None, "test_session", None).unwrap();

    let gen_config = result["request"]["generationConfig"].as_object().unwrap();
    let thinking_config = gen_config["thinkingConfig"].as_object().unwrap();

    // Check injection: Claude models use thinkingLevel in adaptive mode
    assert_eq!(thinking_config["includeThoughts"], true);
    assert_eq!(thinking_config["thinkingLevel"], "HIGH");
    assert!(thinking_config.get("thinkingBudget").is_none());
    assert!(thinking_config.get("thinkingType").is_none());
    assert!(thinking_config.get("effort").is_none());

    // Check maxOutputTokens default for adaptive
    let max_output_tokens = gen_config["maxOutputTokens"].as_i64().unwrap();
    assert_eq!(max_output_tokens, 64000);
}

#[test]
fn test_mixed_tools_injection_for_gemini_2_0() {
    // [场景] 使用 Gemini 2.0 模型，同时提供自定义工具和启用全网搜索
    // 期望: 转换后的请求应同时包含 googleSearch 和 functionDeclarations
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(), // 映射到 gemini-2.0-flash-exp
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Help me search and use tools".to_string()),
        }],
        system: None,
        tools: Some(vec![Tool {
            type_: None,
            name: Some("get_weather".to_string()),
            description: Some("Get weather".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            })),
        }]),
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

    // 模拟映射到 Gemini 2.0
    let mapped_model = "gemini-2.0-flash-exp";

    // 这里我们直接测试 build_tools 函数 (它是 pub(crate) 且在同模块下)
    let result = build_tools(&req.tools, true, mapped_model);
    assert!(result.is_ok());

    let tools_val = result.unwrap().expect("Should have tools");
    let tools_arr = tools_val.as_array().expect("Tools should be an array");

    let has_google_search = tools_arr.iter().any(|t| t.get("googleSearch").is_some());
    let has_functions = tools_arr
        .iter()
        .any(|t| t.get("functionDeclarations").is_some());

    // 在 v1internal 接口受限环境下，强制禁用混合工具调用以避免 400 报错
    // 存在自定义工具时，优先使用 functionDeclarations，不注入 googleSearch
    assert!(
        !has_google_search,
        "v1internal should avoid mixed Google Search when functionDeclarations present"
    );
    assert!(has_functions, "Should have function declarations");
}

#[test]
fn test_no_mixed_tools_for_older_gemini() {
    // [场景] 使用 Gemini 1.5 模型，同时提供自定义工具和启用全网搜索
    // 期望: 转换后的请求应只包含 functionDeclarations，googleSearch 被跳过以避免 400
    let req = ClaudeRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::String("Help me search and use tools".to_string()),
        }],
        system: None,
        tools: Some(vec![Tool {
            type_: None,
            name: Some("get_weather".to_string()),
            description: Some("Get weather".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            })),
        }]),
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

    // 模拟映射到 Gemini 1.5
    let mapped_model = "gemini-1.5-flash-002";

    // 测试 build_tools 函数
    let result = build_tools(&req.tools, true, mapped_model);
    assert!(result.is_ok());

    let tools_val = result.unwrap().expect("Should have tools");
    let tools_arr = tools_val.as_array().expect("Tools should be an array");

    let has_google_search = tools_arr.iter().any(|t| t.get("googleSearch").is_some());
    let has_functions = tools_arr
        .iter()
        .any(|t| t.get("functionDeclarations").is_some());

    assert!(
        !has_google_search,
        "Older Gemini models should NOT have mixed tools"
    );
    assert!(has_functions);
}

#[test]
fn test_model_supports_thinking() {
    // gemini-pro-agent is the mapped target of gemini-3.1-pro-high /
    // gemini-3-pro-high and is a forced-thinking Pro agent model.
    assert!(model_supports_thinking("gemini-pro-agent"));
    assert!(model_supports_thinking("gemini-3-pro"));
    assert!(model_supports_thinking("gemini-3-pro-preview"));
    assert!(model_supports_thinking("gemini-3.1-pro"));
    assert!(model_supports_thinking("gemini-3.1-pro-preview"));
    assert!(model_supports_thinking("gemini-2.0-pro"));
    assert!(model_supports_thinking("gemini-3-flash"));
    assert!(model_supports_thinking("gemini-3.1-flash"));
    assert!(model_supports_thinking("claude-opus-4-6-thinking"));

    // Keyword models (gemini/flash/pro/agent) now force server-side thinking.
    assert!(model_supports_thinking("gemini-2.5-pro"));
    assert!(model_supports_thinking("gemini-1.5-pro"));
}

#[test]
fn test_thinking_block_preserved_with_empty_signature() {
    use crate::proxy::mappers::claude::thinking_utils::filter_invalid_thinking_blocks_with_family;

    let mut messages = vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::String("Hello".to_string()),
        },
        Message {
            role: "assistant".to_string(),
            content: MessageContent::Array(vec![
                ContentBlock::Thinking {
                    thinking: "Considering the question deeply...".to_string(),
                    signature: Some("".to_string()), // Client sends empty signature
                    cache_control: None,
                },
                ContentBlock::Text {
                    text: "Here is my answer".to_string(),
                },
            ]),
        },
        Message {
            role: "user".to_string(),
            content: MessageContent::String("Next question".to_string()),
        },
    ];

    // 1. filter_invalid_thinking_blocks_with_family must NOT drop the thinking block
    filter_invalid_thinking_blocks_with_family(&mut messages, Some("gemini"));
    if let MessageContent::Array(blocks) = &messages[1].content {
        assert_eq!(blocks.len(), 2, "Thinking block must NOT be dropped!");
        assert!(matches!(blocks[0], ContentBlock::Thinking { .. }));
    } else {
        panic!("Expected array content");
    }

    // 2. transform_claude_request_in should produce a thinking block with sentinel signature for gemini-3.8-flash-high
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
    assert_eq!(
        assistant_parts[0]["thoughtSignature"],
        "skip_thought_signature_validator"
    );
    assert_eq!(
        assistant_parts[0]["text"],
        "Considering the question deeply..."
    );
    assert_eq!(assistant_parts[1]["text"], "Here is my answer");
}
