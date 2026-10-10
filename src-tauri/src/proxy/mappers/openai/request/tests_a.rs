// OpenAI request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn test_openai_aliases_max_completion_tokens_and_reasoning_max_tokens() {
    // 1. max_completion_tokens 及 max_output_tokens 别名支持
    let req1: OpenAIRequest = serde_json::from_value(json!({
        "model": "o3-mini",
        "messages": [{"role": "user", "content": "hello"}],
        "max_completion_tokens": 16384
    }))
    .unwrap();
    assert_eq!(req1.max_tokens, Some(16384));

    let req1_resp: OpenAIRequest = serde_json::from_value(json!({
        "model": "gpt-5",
        "messages": [{"role": "user", "content": "hello"}],
        "max_output_tokens": 128000
    }))
    .unwrap();
    assert_eq!(req1_resp.max_tokens, Some(128000));

    // 2. reasoning.max_tokens 与 reasoning.effort 支持
    let req2: OpenAIRequest = serde_json::from_value(json!({
        "model": "o3-mini",
        "messages": [{"role": "user", "content": "hello"}],
        "reasoning": {
            "effort": "high",
            "max_tokens": 8000
        }
    }))
    .unwrap();
    assert_eq!(
        req2.reasoning.as_ref().and_then(|r| r.effort.as_deref()),
        Some("high")
    );
    assert_eq!(
        req2.reasoning.as_ref().and_then(|r| r.max_tokens),
        Some(8000)
    );

    // 3. thinking.max_tokens 别名支持
    let req3: OpenAIRequest = serde_json::from_value(json!({
        "model": "o3-mini",
        "messages": [{"role": "user", "content": "hello"}],
        "thinking": {
            "type": "enabled",
            "max_tokens": 10240
        }
    }))
    .unwrap();
    assert_eq!(
        req3.thinking.as_ref().and_then(|t| t.budget_tokens),
        Some(10240)
    );

    // 4. 客户端控制模式下从 reasoning.max_tokens 提取 client_budget
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig {
            control_source: crate::proxy::config::ThinkingControlSource::Client,
            ..Default::default()
        },
    );
    let (body, _, _, _) = transform_openai_request(&req2, "test-p", "gemini-3.7-flash", None);
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        8000
    );
}

#[test]
fn prompt_log_identity_cleanup_only_changes_system_instructions() {
    // 身份声明归一化由协议无关的流水线节点承担（`PromptSanitizer::sanitize_gemini_payload`），
    // 适配层只产出原样系统提示词。本用例校验「适配层产出 → 流水线清洗」的真实链路：
    // 系统提示词块内的身份声明被统一归一化为中性身份，user / tool 文本逐字保留。
    for old in [
        "You are Codex, an agent based on GPT-5.",
        "You are Codex, a coding agent based on GPT-5.",
        "You are Codex, an advanced coding agent based on GPT-6.",
    ] {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model": "gemini-3.7-flash-high",
            "instructions": format!("Top-level: {old}"),
            "messages": [
                {"role": "system", "content": format!("System: {old}")},
                {"role": "developer", "content": format!("<model_switch>{old}</model_switch>")},
                {"role": "user", "content": old},
                {"role": "assistant", "tool_calls": [{"id": "call_identity", "type": "function", "function": {"name": "identity", "arguments": "{}"}}]},
                {"role": "tool", "tool_call_id": "call_identity", "content": old}
            ]
        }))
        .unwrap();
        let (mut body, _, _, _) = transform_openai_request(&req, "test-project", &req.model, None);

        // 适配层保持纯透传：身份声明在流水线节点介入前原样保留
        assert!(body["request"]["systemInstruction"]
            .to_string()
            .contains(old));

        crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
            &mut body,
        );

        let system = body["request"]["systemInstruction"].to_string();
        assert!(!system.contains(old));
        assert!(system.contains("Top-level: You are an AI Agent."));
        assert!(system.contains("System: You are an AI Agent."));
        assert!(system.contains("<model_switch>You are an AI Agent.</model_switch>"));
        // 用户提问与工具消息（含管道自身提示词）零改动
        let contents = body["request"]["contents"].to_string();
        assert_eq!(contents.matches(old).count(), 2);
    }
}
fn tiered_request_body(model: &str, effort: Option<&str>) -> Value {
    let mut raw = json!({
        "model": model,
        "messages": [{"role": "user", "content": "test"}]
    });
    if let Some(effort) = effort {
        raw["reasoning"] = json!({ "effort": effort });
    }
    let request: OpenAIRequest = serde_json::from_value(raw).unwrap();
    transform_openai_request(&request, "test-project", model, None).0
}

#[test]
fn tiered_flash_ignores_client_effort_and_keeps_include_thoughts_only() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Server-authoritative: client reasoning.effort must not set thinkingLevel.
    for model in ["gemini-3.8-flash-tiered", "gemini-9.9-flash-tiered"] {
        assert!(is_tiered_flash_model(model));
        for effort in [
            None,
            Some("low"),
            Some("medium"),
            Some("high"),
            Some("xhigh"),
        ] {
            let body = tiered_request_body(model, effort);
            let thinking = &body["request"]["generationConfig"]["thinkingConfig"];

            assert_eq!(body["model"], model);
            assert_eq!(thinking["includeThoughts"], true);
            assert!(thinking.get("thinkingLevel").is_none());
            assert!(thinking.get("thinkingBudget").is_none());
        }
    }
}

#[test]
fn reasoning_effort_does_not_select_levels_for_pro_or_ordinary_flash() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for model in ["gemini-3.1-pro-high", "gemini-3.8-flash"] {
        assert!(!is_tiered_flash_model(model));
        let body = tiered_request_body(model, Some("low"));
        let thinking = &body["request"]["generationConfig"]["thinkingConfig"];

        assert_eq!(body["model"], model);
        assert!(thinking.get("thinkingLevel").is_none());
        assert!(thinking.get("thinkingBudget").is_some());
    }
    assert!(!is_tiered_flash_model("gemini-3.8-flash-tiered-image"));
}

#[test]
fn test_openai_reasoning_effort_authority_resolution() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // 1. 启发式模型忽略客户端 reasoning_effort
    let req_high: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3.7-flash-high",
        "messages": [{"role": "user", "content": "hi"}],
        "reasoning_effort": "low"
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_high, "test-p", "gemini-3.7-flash-high", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        16384
    );

    // 2. 裸模型 Flash 接管客户端 reasoning_effort
    let req_flash_high: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}],
        "reasoning_effort": "high"
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_flash_high, "test-p", "gemini-3-flash", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        16384
    );

    let req_flash_low: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}],
        "reasoning_effort": "low"
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_flash_low, "test-p", "gemini-3-flash", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        1024
    );

    // 3. 裸模型 Flash 客户端未填或试图关闭：绝不关闭思考，强制回填 -medium (4096)
    let req_flash_none: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}]
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_flash_none, "test-p", "gemini-3-flash", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        4096
    );

    let req_flash_disabled: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}],
        "reasoning_effort": "none"
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_flash_disabled, "test-p", "gemini-3-flash", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        4096
    );

    // 4. 裸模型 Flash 客户端传入自定义 budget_tokens：彻底被忽略，由服务端权威等级回填
    let req_flash_custom_budget: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}],
        "thinking": {"budget_tokens": 12345}
    }))
    .unwrap();
    let (body, _, _, _) =
        transform_openai_request(&req_flash_custom_budget, "test-p", "gemini-3-flash", None);
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        4096
    );

    let req_flash_high_custom_budget: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3-flash",
        "messages": [{"role": "user", "content": "hi"}],
        "reasoning_effort": "high",
        "thinking": {"budget_tokens": 1234}
    }))
    .unwrap();
    let (body, _, _, _) = transform_openai_request(
        &req_flash_high_custom_budget,
        "test-p",
        "gemini-3-flash",
        None,
    );
    assert_eq!(
        body["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        16384
    );
}

#[test]
fn test_openai_request_id_is_unique_per_upstream_attempt() {
    let req: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3.7-flash-high",
        "messages": [{"role": "user", "content": "test"}]
    }))
    .unwrap();

    let (first, _, _, _) =
        transform_openai_request(&req, "test-project", "gemini-3.7-flash-high", None);
    let (second, _, _, _) =
        transform_openai_request(&req, "test-project", "gemini-3.7-flash-high", None);
    let first_id = first["requestId"].as_str().unwrap();
    let second_id = second["requestId"].as_str().unwrap();

    assert_ne!(first_id, second_id);

    let parts = first_id.split('/').collect::<Vec<_>>();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "agent");
    assert!(parts[1].parse::<i64>().is_ok());
    assert_eq!(parts[2].len(), 8);
    assert!(parts[2].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn responses_session_identity_is_not_written_into_system_instruction() {
    let req: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3.7-flash-high",
        "messages": [{"role": "user", "content": "same first message"}]
    }))
    .unwrap();

    let (body, session_id, _, _) = transform_openai_request_with_session(
        &req,
        "test-project",
        "gemini-3.7-flash-high",
        None,
        "resp-routing-root",
        None,
        true,
    );
    let system_instruction = body["request"].get("systemInstruction");

    assert_eq!(session_id, "resp-routing-root");
    if let Some(sys) = system_instruction {
        let sys_text = sys.to_string();
        assert!(!sys_text.contains("Request type:"));
        assert!(!sys_text.contains("Mapped model:"));
        assert!(!sys_text.contains("user_information"));
        assert!(!sys_text.contains("Session ID:"));
        assert!(!sys_text.contains("resp-routing-root"));
    }
    assert!(!body["request"].to_string().contains("resp-routing-root"));
}

#[test]
fn openai_system_messages_are_forwarded_as_separate_parts() {
    let req: OpenAIRequest = serde_json::from_value(json!({
        "model": "gemini-3.7-flash-high",
        "messages": [
            {"role": "system", "content": "<environment>env</environment>"},
            {"role": "system", "content": [
                {"type": "text", "text": "<workflow_and_execution_discipline>wf</workflow_and_execution_discipline>"},
                {"type": "text", "text": "=== AVAILABLE SKILLS ==="}
            ]},
            {"role": "user", "content": "hello"}
        ]
    }))
    .unwrap();

    let (body, _, _, _) =
        transform_openai_request(&req, "test-project", "gemini-3.7-flash-high", None);
    let parts = body["request"]["systemInstruction"]["parts"]
        .as_array()
        .expect("systemInstruction.parts");
    let texts: Vec<&str> = parts.iter().filter_map(|p| p["text"].as_str()).collect();

    assert_eq!(
        texts,
        vec![
            "<environment>env</environment>",
            "<workflow_and_execution_discipline>wf</workflow_and_execution_discipline>",
            "=== AVAILABLE SKILLS ==="
        ]
    );
    let joined = texts.join("\n");
    assert!(!joined.contains("<user_information>"));
    assert!(!joined.contains("Request type:"));
}
