// OpenAI request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn test_multi_turn_responses_preserves_historical_signature_prefix() {
    let sid = format!("test-sess-{}", uuid::Uuid::new_v4());
    let sig_round_1 = "s1_".to_string() + &"a".repeat(60);
    let sig_round_2 = "s2_".to_string() + &"b".repeat(60);

    // 缓存第 1 轮工具的专属签名
    crate::proxy::SignatureCache::global().cache_tool_signature("call_1", sig_round_1.clone());

    // 模拟第 2 轮刚完成，产生了会话级别的最新签名 sig_round_2 (通过 previous_response_id)
    let prev_resp_id = format!("resp-prev-{}", uuid::Uuid::new_v4());
    crate::proxy::SignatureCache::global().cache_session_signature(
        &prev_resp_id,
        sig_round_2.clone(),
        3,
    );

    // 构造第 3 轮请求：包含历史第 1 轮、第 2 轮的完整上下文
    let req = OpenAIRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("第 1 轮指令".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "run_command".to_string(),
                        arguments: "{\"command\":\"ls\"}".to_string(),
                    }),
                    status: None,
                    call_id: None,
                    operation: None,
                    signature: None,
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_1".to_string()),
                content: Some(OpenAIContent::String("file1.txt".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_2".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "run_command".to_string(),
                        arguments: "{\"command\":\"cat file1.txt\"}".to_string(),
                    }),
                    status: None,
                    call_id: None,
                    operation: None,
                    signature: None,
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_2".to_string()),
                content: Some(OpenAIContent::String("hello world".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("第 3 轮指令".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let (result, _, _, _) = transform_openai_request_with_session(
        &req,
        "test-proj",
        "gemini-3.8-flash-high",
        None,
        &sid,
        Some(&prev_resp_id),
        true, // is_responses_api
    );

    let contents = result["request"]["contents"].as_array().unwrap();

    // 验证：第 1 轮 model
    let model_1_parts = contents[1]["parts"].as_array().unwrap();
    assert_eq!(model_1_parts[0]["thought"], true, "第 1 轮首位必须是思考块");
    let sig_1 = model_1_parts[0]["thoughtSignature"].as_str().unwrap();
    // 核心断言：历史第 1 轮绝不能被最新一轮的签名 sig_round_2 覆盖！
    assert_ne!(sig_1, sig_round_2, "历史第 1 轮绝不能被最新签名覆盖");

    // 验证：最新一条 model（第 2 轮）
    let model_2_parts = contents[3]["parts"].as_array().unwrap();
    assert_eq!(model_2_parts[0]["thought"], true, "第 2 轮首位必须是思考块");
    let sig_2 = model_2_parts[0]["thoughtSignature"].as_str().unwrap();
    // 最新一条 model 应当正确采纳 prev_resp_id 的签名
    assert_eq!(sig_2, sig_round_2, "最新一条 model 应当正确继承上一轮签名");
}

#[test]
fn test_transform_openai_request_preserves_command_with_description() {
    let req = OpenAIRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("执行命令".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_test_1".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "run_command".to_string(),
                        arguments: serde_json::to_string(&json!({
                            "description": "Run: git checkout main",
                            "command": "git checkout main",
                            "shell": "default"
                        }))
                        .unwrap(),
                    }),
                    signature: None,
                    status: None,
                    call_id: None,
                    operation: None,
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_test_1".to_string()),
                content: Some(OpenAIContent::String("Switched to branch main".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let (result, _, _, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-high", None);
    let contents = result["request"]["contents"].as_array().unwrap();

    // 验证 assistant 的 functionCall.args 中的 command 绝对未被剔除
    let model_parts = contents[1]["parts"].as_array().unwrap();
    let fc = model_parts
        .iter()
        .find(|p| p.get("functionCall").is_some())
        .unwrap();
    let args = &fc["functionCall"]["args"];
    assert_eq!(
        args["command"], "git checkout main",
        "command 参数必须完好保留！"
    );
    assert_eq!(
        args["description"], "Run: git checkout main",
        "description 参数必须完好保留！"
    );
    assert_eq!(args["shell"], "default", "shell 参数必须完好保留！");
}

#[test]
fn test_transform_openai_request_handles_colliding_tool_call_ids() {
    let req = OpenAIRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("修改文件".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_duplicate_1025976".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "edit_file".to_string(),
                        arguments: "{}".to_string(),
                    }),
                    signature: None,
                    status: None,
                    call_id: None,
                    operation: None,
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_duplicate_1025976".to_string()),
                content: Some(OpenAIContent::String("ok".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_duplicate_1025976".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "run_command".to_string(),
                        arguments: "{\"command\":\"git status\"}".to_string(),
                    }),
                    signature: None,
                    status: None,
                    call_id: None,
                    operation: None,
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_duplicate_1025976".to_string()),
                content: Some(OpenAIContent::String("clean".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let (result, _, _, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-high", None);
    let contents = result["request"]["contents"].as_array().unwrap();

    // 验证第 1 轮工具响应的名字为 edit_file，绝不能被后续同 ID 的 run_command 覆盖
    let tool_resp_1 = contents[2]["parts"][0]["functionResponse"]["name"]
        .as_str()
        .unwrap();
    assert_eq!(
        tool_resp_1, "edit_file",
        "第 1 轮工具响应必须匹配其调用时的 edit_file 工具名"
    );

    // 验证第 2 轮工具响应的名字为 run_command
    let tool_resp_2 = contents[4]["parts"][0]["functionResponse"]["name"]
        .as_str()
        .unwrap();
    assert_eq!(
        tool_resp_2, "run_command",
        "第 2 轮工具响应必须匹配其调用时的 run_command 工具名"
    );
}

#[test]
fn test_tool_result_multimodal_image_extraction_and_passthrough() {
    let fake_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let tool_content = format!(
        "Here is the screenshot: ![screen](data:image/png;base64,{}) and some log text",
        fake_b64
    );

    let req = OpenAIRequest {
        model: "gemini-2.5-flash".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("Take screenshot".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: "call_shot_1".to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "screenshot".to_string(),
                        arguments: "{}".to_string(),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some("call_shot_1".to_string()),
                content: Some(OpenAIContent::String(tool_content)),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let (result, _, _, _) = transform_openai_request(&req, "test-proj", "gemini-2.5-flash", None);
    let contents = result["request"]["contents"].as_array().unwrap();
    let tool_turn_parts = contents[2]["parts"].as_array().unwrap();

    // 验证同时存在 functionResponse 和 inlineData 两个 parts
    assert_eq!(tool_turn_parts.len(), 2);
    assert!(tool_turn_parts[0].get("functionResponse").is_some());
    assert!(tool_turn_parts[1].get("inlineData").is_some());

    let inline_data = &tool_turn_parts[1]["inlineData"];
    assert_eq!(inline_data["mimeType"], "image/png");
    assert_eq!(inline_data["data"], fake_b64);

    // 验证文本中的 base64 已被替换为摘要说明，防止 functionResponse 体积膨胀
    let func_res_str = tool_turn_parts[0]["functionResponse"]["response"]["result"]
        .as_str()
        .unwrap();
    assert!(!func_res_str.contains(fake_b64));
    assert!(func_res_str.contains("[Image: forwarded to Gemini visual input (image/png)]"));
}

#[test]
fn test_function_declarations_schema_sanitization() {
    let req = OpenAIRequest {
        model: "gemini-2.5-flash".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        tools: Some(vec![json!({
            "type": "function",
            "function": {
                "name": "complex_tool",
                "description": "A complex tool\nwith multi-line\r\ndescriptions",
                "parameters": {
                    "type": "object"
                    // 故意省略 properties
                }
            }
        })]),
        ..Default::default()
    };

    let (result, _, _, _) = transform_openai_request(&req, "test-proj", "gemini-2.5-flash", None);
    let tools = result["request"]["tools"].as_array().unwrap();
    let func_decls = tools[0]["functionDeclarations"].as_array().unwrap();
    let decl = &func_decls[0];

    // 验证 description 保持原始格式透传
    assert_eq!(
        decl["description"],
        "A complex tool\nwith multi-line\r\ndescriptions"
    );

    // 验证 parameters 保证包含 OBJECT 和 properties: {}
    assert_eq!(decl["parameters"]["type"], "OBJECT");
    assert_eq!(decl["parameters"]["properties"], json!({}));
}

#[test]
fn test_openai_tool_call_retrieves_signature_from_signature_cache() {
    let tool_id = "call_cached_test_999";
    let valid_gemini_sig = "EmIKYAFpFH0TDqviLY1vZ8EuHqBLLj5xxD+0hchYg2VaoyolUQRP+hSCsKRpSpj+yrQA2H27yVFnF7tlp5OHIUvTdZKKErAqILJzK5FG8RJg42jCaaI2/iwqoBuRd5BDVwBxaQ==";
    crate::proxy::SignatureCache::global()
        .cache_tool_signature(tool_id, valid_gemini_sig.to_string());

    let req = OpenAIRequest {
        model: "gemini-3.8-flash".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(OpenAIContent::String("Run tool".to_string())),
                ..Default::default()
            },
            OpenAIMessage {
                role: "assistant".to_string(),
                tool_calls: Some(vec![ToolCall {
                    id: tool_id.to_string(),
                    r#type: "function".to_string(),
                    function: Some(ToolFunction {
                        name: "bash".to_string(),
                        arguments: "{}".to_string(),
                    }),
                    signature: None, // 客户端未带签名 (标准 OpenAI 协议)
                    ..Default::default()
                }]),
                ..Default::default()
            },
            OpenAIMessage {
                role: "tool".to_string(),
                tool_call_id: Some(tool_id.to_string()),
                content: Some(OpenAIContent::String("done".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let (result, _, _, _) =
        transform_openai_request(&req, "test-proj", "gemini-3.8-flash-tiered", None);
    let contents = result["request"]["contents"].as_array().unwrap();

    // 查找 model 轮次中的 functionCall 部件
    let model_msg = contents
        .iter()
        .find(|c| c["role"] == "model")
        .expect("must find model message");
    let fc_part = model_msg["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p.get("functionCall").is_some())
        .expect("must find functionCall part");

    assert_eq!(
        fc_part["thoughtSignature"], valid_gemini_sig,
        "OpenAI adapter and pipeline must recover real tool signature from SignatureCache"
    );
}
