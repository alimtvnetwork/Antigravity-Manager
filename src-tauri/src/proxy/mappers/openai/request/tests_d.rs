// OpenAI request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn client_thinking_enable_ignored_for_non_thinking_model() {
    let req = OpenAIRequest {
        model: "gpt-4o".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("hi".to_string())),
            ..Default::default()
        }],
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(8000),
            effort: Some("high".to_string()),
        }),
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) = transform_openai_request(&req, "test-proj", "gpt-4o", None);
    let gen_config = &result["request"]["generationConfig"];
    assert!(
        gen_config.get("thinkingConfig").is_none(),
        "non-thinking model must not enable thinking from client flags"
    );
}

#[test]
fn test_issue_3515_client_direct_control_disable_thinking() {
    use crate::proxy::config::{
        update_thinking_budget_config, ThinkingBudgetConfig, ThinkingControlSource,
    };

    let mut config = ThinkingBudgetConfig::default();
    config.control_source = ThinkingControlSource::Client;
    update_thinking_budget_config(config);

    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = ResetGuard;

    let req = OpenAIRequest {
        model: "gemini-3.8-flash-medium".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        thinking: Some(ThinkingConfig {
            thinking_type: None,
            budget_tokens: Some(0),
            effort: None,
        }),
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-medium", None);

    let gen_config = result["request"]["generationConfig"]
        .as_object()
        .expect("Should have generationConfig in request payload");

    // 客户端直接控制模式显式关闭思考：彻底不填任何转出报文的谷歌思考块字段和预算
    assert!(gen_config.get("thinkingConfig").is_none());
}

#[test]
fn test_client_direct_control_all_scenarios_for_gemini_38_flash_tiered() {
    use crate::proxy::config::{
        update_thinking_budget_config, ThinkingBudgetConfig, ThinkingControlSource,
    };

    let mut config = ThinkingBudgetConfig::default();
    config.control_source = ThinkingControlSource::Client;
    update_thinking_budget_config(config);

    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = ResetGuard;

    // 场景 1: 客户端思考块全缺省（开关缺省就是默认开，预算留空交由上游自适应）
    let req_default = OpenAIRequest {
        model: "gemini-3.8-flash-tiered".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };
    let (res1, _, _, _) =
        transform_openai_request(&req_default, "test-proj", "gemini-3.8-flash-tiered", None);
    let tc1 = res1["request"]["generationConfig"]["thinkingConfig"]
        .as_object()
        .expect("Should have thinkingConfig for default thinking");
    assert_eq!(tc1.get("includeThoughts"), Some(&json!(true)));
    assert!(tc1.get("thinkingBudget").is_none());
    assert!(tc1.get("thinkingLevel").is_none());

    // 场景 2: 客户端仅传思考等级 low（等级透传，绝不脑补 budget，必须带上 includeThoughts: true）
    let req_level = OpenAIRequest {
        model: "gemini-3.8-flash-tiered".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        reasoning_effort: Some("low".to_string()),
        ..Default::default()
    };
    let (res2, _, _, _) =
        transform_openai_request(&req_level, "test-proj", "gemini-3.8-flash-tiered", None);
    let tc2 = res2["request"]["generationConfig"]["thinkingConfig"]
        .as_object()
        .expect("Should have thinkingConfig for level");
    assert_eq!(tc2.get("includeThoughts"), Some(&json!(true)));
    assert_eq!(tc2.get("thinkingLevel"), Some(&json!("LOW")));
    assert!(tc2.get("thinkingBudget").is_none());

    // 场景 3: 客户端仅传预算 8192（忠实透传预算，绝不脑补等级，必须带上 includeThoughts: true）
    let req_budget = OpenAIRequest {
        model: "gemini-3.8-flash-tiered".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(8192),
            effort: None,
        }),
        ..Default::default()
    };
    let (res3, _, _, _) =
        transform_openai_request(&req_budget, "test-proj", "gemini-3.8-flash-tiered", None);
    let tc3 = res3["request"]["generationConfig"]["thinkingConfig"]
        .as_object()
        .expect("Should have thinkingConfig for budget");
    assert_eq!(tc3.get("includeThoughts"), Some(&json!(true)));
    assert_eq!(tc3.get("thinkingBudget"), Some(&json!(8192)));
    assert!(tc3.get("thinkingLevel").is_none());
    // 且验证 maxOutputTokens 自动垫高保证 > budget
    let max_out = res3["request"]["generationConfig"]["maxOutputTokens"]
        .as_i64()
        .unwrap();
    assert!(max_out > 8192);
}

#[test]
fn test_issue_3515_gateway_control_preserves_medium_budget_when_client_budget_zero() {
    use crate::proxy::config::{
        update_thinking_budget_config, ThinkingBudgetConfig, ThinkingControlSource,
    };

    let mut config = ThinkingBudgetConfig::default();
    config.control_source = ThinkingControlSource::Gateway;
    update_thinking_budget_config(config);

    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = ResetGuard;

    let req = OpenAIRequest {
        model: "gemini-3.8-flash-medium".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        thinking: Some(ThinkingConfig {
            thinking_type: None,
            budget_tokens: Some(0),
            effort: None,
        }),
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-medium", None);

    let gen_config = result["request"]["generationConfig"]
        .as_object()
        .expect("Should have generationConfig in request payload");
    let thinking_config = gen_config["thinkingConfig"].as_object().unwrap();

    // 网关权威控制模式下，90% 用户行为保持不变，依然权威锁定 4000 预算
    assert_eq!(thinking_config.get("thinkingBudget"), Some(&json!(4000)));
    assert_eq!(thinking_config.get("includeThoughts"), Some(&json!(true)));
}

#[test]
fn test_hermes_autonomous_first_assistant_tool_call_injected_user_primer() {
    // Ensure conversation starting with assistant tool calls (common in Hermes / autonomous agents)
    // has a user primer injected at index 0 so Google Gemini does not reject with:
    // "Please ensure that function call turn comes immediately after a user turn or after a function response turn."
    let raw_json = json!({
        "model": "gemini-3.8-flash-high",
        "messages": [
            {
                "role": "system",
                "content": "You are Don Santo, an autonomous agent."
            },
            {
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    {
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "terminal",
                            "arguments": "{\"command\": \"ls\"}"
                        }
                    }
                ]
            },
            {
                "role": "tool",
                "tool_call_id": "call_123",
                "name": "terminal",
                "content": "output of ls"
            }
        ]
    });

    let request: OpenAIRequest = serde_json::from_value(raw_json).unwrap();
    let (res_val, _sid, _msg_count, _) =
        transform_openai_request(&request, "test-v", "gemini-3.8-flash-high", None);
    let contents = res_val["request"]["contents"]
        .as_array()
        .expect("contents must be an array");

    // First turn MUST be user
    assert_eq!(contents[0]["role"], "user");
    assert!(contents[0]["parts"][0]["text"].as_str().is_some());

    // Second turn MUST be model with functionCall
    assert_eq!(contents[1]["role"], "model");
    let has_func_call = contents[1]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p.get("functionCall").is_some());
    assert!(has_func_call);

    // Third turn MUST be user with functionResponse
    assert_eq!(contents[2]["role"], "user");
    let has_func_resp = contents[2]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p.get("functionResponse").is_some());
    assert!(has_func_resp);

    // Since it has tool_calls / functionCall, it should have requestType: "agent"
    assert_eq!(res_val.get("requestType"), Some(&json!("agent")));
}

#[test]
fn test_plain_chat_omits_agent_request_type() {
    let raw_json = json!({
        "model": "gemini-2.5-flash",
        "messages": [
            {
                "role": "user",
                "content": "Hello world!"
            }
        ]
    });

    let request: OpenAIRequest = serde_json::from_value(raw_json).unwrap();
    let (res_val, _, _, _) = transform_openai_request(&request, "test-v", "gemini-2.5-flash", None);
    assert!(
        res_val.get("requestType").is_none(),
        "Plain text request should not have requestType: 'agent'"
    );
}

#[test]
fn test_openai_responses_api_vs_chat_api_thinking_and_signature() {
    let valid_client_sig = "B".repeat(60);
    let client_thought = "Responses API client thinking block";

    let req = OpenAIRequest {
        model: "gemini-3-pro".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("first question".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                content: Some(OpenAIContent::String("assistant answer".to_string())),
                reasoning_content: Some(client_thought.to_string()),
                signature: Some(valid_client_sig.clone()),
                ..Default::default()
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("follow up".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // 1. Responses API (is_responses_api = true): honors client signature and reasoning content
    let (resp_result, _, _, _) = transform_openai_request_with_session(
        &req,
        "test-proj",
        "gemini-3-pro",
        None,
        "routing-1",
        None,
        true, // is_responses_api
    );
    let resp_contents = resp_result["request"]["contents"].as_array().unwrap();
    let resp_model_msg = resp_contents.iter().find(|m| m["role"] == "model").unwrap();
    let resp_thought = resp_model_msg["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p.get("thought") == Some(&json!(true)))
        .unwrap();
    assert_eq!(resp_thought["text"], client_thought);
    assert_eq!(resp_thought["thoughtSignature"], valid_client_sig);

    // 2. Chat API (is_responses_api = false): honors reasoning content, but ignores client signature
    let (chat_result, _, _, _) = transform_openai_request_with_session(
        &req,
        "test-proj",
        "gemini-3-pro",
        None,
        "routing-chat",
        None,
        false, // is_responses_api
    );
    let chat_contents = chat_result["request"]["contents"].as_array().unwrap();
    let chat_model_msg = chat_contents.iter().find(|m| m["role"] == "model").unwrap();
    let chat_thought = chat_model_msg["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p.get("thought") == Some(&json!(true)))
        .unwrap();
    assert_eq!(chat_thought["text"], client_thought);
    // Chat API signature must be server-filled (sentinel), not client signature
    assert_eq!(
        chat_thought["thoughtSignature"].as_str(),
        Some(crate::proxy::thinking_store::SENTINEL_SIGNATURE)
    );
}

#[test]
fn test_shell_tool_strips_description_parameter_for_gemini() {
    let req = OpenAIRequest {
        model: "gemini-2.5-pro".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("run command".to_string())),
            ..Default::default()
        }],
        tools: Some(vec![json!({
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Run a shell command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "CLI command" },
                        "description": { "type": "string", "description": "Optional human label" }
                    },
                    "required": ["command", "description"]
                }
            }
        })]),
        ..Default::default()
    };

    let (result, _, _, _) = transform_openai_request_with_session(
        &req,
        "test-proj",
        "gemini-2.5-pro",
        None,
        "routing-1",
        None,
        false,
    );

    let tools = result["request"]["tools"].as_array().unwrap();
    let func_decls = tools[0]["functionDeclarations"].as_array().unwrap();
    let run_cmd = func_decls
        .iter()
        .find(|f| f["name"] == "run_command")
        .unwrap();
    let props = run_cmd["parameters"]["properties"].as_object().unwrap();
    assert!(props.contains_key("command"));
    assert!(
        !props.contains_key("description"),
        "description parameter must be stripped for Gemini"
    );
    let req_arr = run_cmd["parameters"]["required"].as_array().unwrap();
    assert!(req_arr.iter().any(|v| v == "command"));
    assert!(
        !req_arr.iter().any(|v| v == "description"),
        "description must not be required"
    );
}
