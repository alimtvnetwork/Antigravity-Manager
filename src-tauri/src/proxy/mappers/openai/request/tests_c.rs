// OpenAI request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn test_flash_thinking_budget_capping() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );

    let req = OpenAIRequest {
        model: "gpt-4".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        stream: false,
        n: None,
        // User specifies a large budget (e.g. xhigh = 32768)
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(32768),
            effort: None,
        }),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stop: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        ..Default::default()
    };

    // Test with Flash model
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-p", "gemini-2.0-flash-thinking-exp", None);
    let gen_config = &result["request"]["generationConfig"];

    // Should be capped at 24576
    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_i64()
        .unwrap();
    assert_eq!(budget, 24576);

    // Max output tokens should be adjusted based on capped budget (24576 + 8192)
    // budget(24576) + overhead(32768) = 57344
    let max_output_tokens = gen_config["maxOutputTokens"].as_i64().unwrap();
    assert_eq!(max_output_tokens, 57344);
}

#[test]
fn test_vertex_ai_drops_sentinel_injection() {
    // [FIX #1650] Verify sentinel signature injection for Vertex AI models
    let req = OpenAIRequest {
        model: "claude-3-7-sonnet-thinking".to_string(), // Triggers is_thinking_model
        messages: vec![OpenAIMessage {
            role: "assistant".to_string(),
            reasoning_content: Some("Thinking...".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_123".to_string(),
                r#type: "function".to_string(),
                function: Some(ToolFunction {
                    name: "test_tool".to_string(),
                    arguments: "{}".to_string(),
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }],
        person_generation: None,
        ..Default::default()
    };

    // Simulate Vertex AI path
    let mapped_model = "projects/my-project/locations/us-central1/publishers/google/models/gemini-2.0-flash-thinking-exp";

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-v", mapped_model, None);

    // Extract the tool call part from contents (under request.contents)
    let contents = result["request"]["contents"].as_array().unwrap();
    // Identify the part with functionCall
    let model_msg = contents
        .iter()
        .find(|c| c["role"] == "model")
        .expect("Should find model role message");
    let parts = model_msg["parts"].as_array().unwrap();
    let tool_part = parts
        .iter()
        .find(|p: &&serde_json::Value| p.get("functionCall").is_some())
        .expect("Should find functionCall part");

    // 铁律：functionCall **绝不**携带哨兵 —— 官方报文 0/23 处出现哨兵，
    // 它不属于 Antigravity 协议；签名归位统一交给流水线终审 `place_turn_signature`。
    assert!(
        tool_part.get("thoughtSignature").is_none(),
        "functionCall must not carry a sentinel signature"
    );
}

#[test]
fn test_issue_2167_gemini_flash_thinking_signature() {
    // [FIX #2167] gemini-3-flash / gemini-3.1-flash 在无缓存签名时，functionCall 必须携带 thoughtSignature
    for model in &["gemini-3-flash", "gemini-3.1-flash"] {
        let req = OpenAIRequest {
            model: model.to_string(),
            messages: vec![OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_flash_test".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "get_weather".to_string(),
                        arguments: "{\"location\":\"Beijing\"}".to_string(),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }],
            ..Default::default()
        };

        let (result, _sid, _msg_count, _) =
            transform_openai_request(&req, "test-proj", model, None);

        let contents = result["request"]["contents"]
            .as_array()
            .expect("Should have request.contents");
        // flash 模型的 assistant role → Gemini "model" role
        let model_msg = contents
            .iter()
            .find(|c| c["role"] == "model")
            .expect("Should find model role message");
        let parts = model_msg["parts"].as_array().expect("Should have parts");
        let tool_part = parts
            .iter()
            .find(|p: &&serde_json::Value| p.get("functionCall").is_some())
            .expect(&format!("[{model}] Should find functionCall part"));

        // 铁律：无缓存签名时**留空**（字段缺席），绝不发明哨兵。
        // 官方报文里哨兵出现 0 次；签名缺失是被上游容忍的（在飞轮即缺席），
        // 且该轮签名由流水线终审 `place_turn_signature` 按锚点归位。
        assert!(
            tool_part.get("thoughtSignature").is_none(),
            "[{model}] functionCall must not carry a sentinel signature when unsigned"
        );
    }
}

#[test]
fn test_openai_image_thinking_mode_disabled() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    // 1. Set global mode to disabled
    crate::proxy::config::update_image_thinking_mode(Some("disabled".to_string()));
    struct ImageResetGuard;
    impl Drop for ImageResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_image_thinking_mode(Some("enabled".to_string()));
        }
    }
    let _guard = ImageResetGuard;

    let req = OpenAIRequest {
        model: "gemini-3-pro-image".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Draw a cat".to_string())),
            ..Default::default()
        }],
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        person_generation: None,
        ..Default::default()
    };

    // 2. Transform request
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-proj", "gemini-3-pro-image", None);

    // 3. Verify thinkingConfig has includeThoughts: false
    let gen_config = result["request"]["generationConfig"]
        .as_object()
        .expect("Should have generationConfig in request payload");
    let thinking_config = gen_config["thinkingConfig"].as_object().unwrap();

    assert_eq!(thinking_config["includeThoughts"], false);
}

#[test]
fn test_mixed_tools_injection_openai() {
    // 验证 OpenAI 协议在 Gemini 2.0+ 下支持混合工具
    let req = OpenAIRequest {
        model: "gpt-4o-online".to_string(), // -online 触发联网
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        tools: Some(vec![json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }
            }
        })]),
        ..Default::default()
    };

    // 使用 gemini-2.0-flash 模型执行转换
    let (result, _, _, _) = transform_openai_request(&req, "proj", "gemini-2.0-flash", None);

    let tools = result["request"]["tools"]
        .as_array()
        .expect("Should have tools");

    let has_functions = tools
        .iter()
        .any(|t: &serde_json::Value| t.get("functionDeclarations").is_some());
    let has_google_search = tools
        .iter()
        .any(|t: &serde_json::Value| t.get("googleSearch").is_some());

    assert!(has_functions, "Should contain functionDeclarations");
    // 在 v1internal 架构下，不开启混合调用以避免 400 报错
    assert!(
        !has_google_search,
        "v1internal should avoid mixed Google Search when functionDeclarations present"
    );
}

#[test]
fn test_response_format_json_schema_mapping() {
    let raw_json = json!({
        "model": "gemini-2.5-flash",
        "messages": [
            {"role": "user", "content": "test"}
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "test_schema",
                "schema": {
                    "type": "object",
                    "properties": {
                        "summary": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" },
                                "sourceId": { "type": "string" },
                                "quote": { "type": "string" }
                            },
                            "required": ["text", "sourceId", "quote"],
                            "additionalProperties": false
                        },
                        "topics": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["summary", "topics"],
                    "additionalProperties": false
                },
                "strict": true
            }
        }
    });

    let request: OpenAIRequest = serde_json::from_value(raw_json).unwrap();
    let (res_val, _sid, _msg_count, _) =
        transform_openai_request(&request, "test-v", "gemini-2.5-flash", None);
    let gen_config = &res_val["request"]["generationConfig"];
    assert_eq!(gen_config["responseMimeType"], "application/json");
    assert!(gen_config.get("responseSchema").is_some());
    let resp_schema = &gen_config["responseSchema"];
    assert_eq!(resp_schema["type"], "object");
    assert_eq!(resp_schema["properties"]["summary"]["type"], "object");
}

#[test]
fn test_issue_3391_claude_without_thinking_suffix_incompatible_history() {
    // claude-sonnet-4-6 forces server thinking by model heuristic (not client enable).
    // Missing client reasoning_content still gets "..." + sentinel placeholder.
    let req = OpenAIRequest {
        model: "claude-sonnet-4-6".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("Hello".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                content: Some(OpenAIContent::String("Hi there!".to_string())),
                reasoning_content: None,
                ..Default::default()
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("How are you?".to_string())),
                ..Default::default()
            },
        ],
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(1024),
            effort: None,
        }),
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-proj", "claude-sonnet-4-6", None);

    let gen_config = &result["request"]["generationConfig"];
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig must be present via server model heuristics"
    );

    let contents = result["request"]["contents"].as_array().unwrap();
    let assistant_msg = contents
        .iter()
        .find(|m| m["role"] == "model")
        .expect("Should have model message");
    let parts = assistant_msg["parts"].as_array().unwrap();
    // 【2026-09-27】客户端无 reasoning 时不再填充 "..." 占位思考块；
    // 无思考块是官方标准形态（锚点签名由流水线终审回填）。
    assert!(
        !parts
            .iter()
            .any(|p| p.get("thought") == Some(&serde_json::json!(true))),
        "No placeholder thinking block should be injected when client reasoning is absent"
    );
}

#[test]
fn server_authoritative_ignores_client_reasoning_content_and_budget() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig {
            mode: crate::proxy::config::ThinkingBudgetMode::Passthrough,
            custom_value: 99999,
            effort: None,
            ..Default::default()
        },
    );
    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(
                crate::proxy::config::ThinkingBudgetConfig::default(),
            );
        }
    }
    let _guard = ResetGuard;

    let client_thought = "Detailed client reasoning thought process";
    let req = OpenAIRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("q1".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                content: Some(OpenAIContent::String("a1".to_string())),
                reasoning_content: Some(client_thought.to_string()),
                signature: Some("fake_client_sig_that_must_be_ignored_in_chat_api".to_string()),
                ..Default::default()
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("q2".to_string())),
                ..Default::default()
            },
        ],
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(16000),
            effort: Some("high".to_string()),
        }),
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-high", None);

    let budget = result["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .expect("thinkingBudget from model_specs");
    assert_eq!(budget, 16384, "client budget + Passthrough must be ignored");

    let contents = result["request"]["contents"].as_array().unwrap();
    let model_msg = contents
        .iter()
        .find(|c| c["role"] == "model")
        .expect("model turn");
    let thought = model_msg["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p.get("thought") == Some(&serde_json::json!(true)))
        .expect("thought part");
    // Reasoning content is preserved (Anthropic alignment)
    assert_eq!(thought["text"], client_thought);
    // Signature is backfilled by server (sentinel or cache), ignoring client signature
    assert_eq!(
        thought["thoughtSignature"].as_str(),
        Some(crate::proxy::thinking_store::SENTINEL_SIGNATURE)
    );
    let dumped = serde_json::to_string(&result).unwrap();
    assert!(
        !dumped.contains("fake_client_sig_that_must_be_ignored"),
        "client signature in chat API must be ignored and backfilled by server"
    );
}
