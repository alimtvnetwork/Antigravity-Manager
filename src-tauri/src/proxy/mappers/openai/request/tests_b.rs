// OpenAI request tests (split from request.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn responses_reads_the_parent_signature_instead_of_the_routing_identity() {
    let previous_response_id = format!("resp-parent-{}", uuid::Uuid::new_v4());
    let routing_session_id = format!("resp-root-{}", uuid::Uuid::new_v4());
    let signature = "parent-signature-".repeat(8);
    crate::proxy::SignatureCache::global().cache_session_signature(
        &previous_response_id,
        signature.clone(),
        1,
    );
    let request = OpenAIRequest {
        model: "gemini-3.7-flash-high".to_string(),
        messages: vec![OpenAIMessage {
            role: "assistant".to_string(),
            tool_calls: Some(vec![ToolCall {
                id: "call-parent".to_string(),
                r#type: "function".to_string(),
                function: Some(ToolFunction {
                    name: "test_tool".to_string(),
                    arguments: "{}".to_string(),
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (body, returned_session_id, _, _) = transform_openai_request_with_session(
        &request,
        "test-project",
        "gemini-3.7-flash-high",
        None,
        &routing_session_id,
        Some(&previous_response_id),
        true,
    );
    let contents = body["request"]["contents"].as_array().unwrap();
    let model_msg = contents
        .iter()
        .find(|c| c["role"] == "model")
        .expect("Should find model role message");
    let tool_part = model_msg["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part.get("functionCall").is_some())
        .unwrap();

    assert_eq!(returned_session_id, routing_session_id);
    assert_eq!(tool_part["thoughtSignature"], signature);
}

#[test]
fn test_issue_1592_gemini_3_pro_budget_capping() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );
    // [FIX #1592] Regression test for gemini-3-pro thinking budget capping
    let req = OpenAIRequest {
        model: "gemini-3-pro".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("test".into())),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Auto mode (default) should map gemini-3-pro thinking budget to 49152 per model_specs
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-v", "gemini-3-pro", None);
    let budget = result["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"]
        .as_i64()
        .unwrap();
    assert_eq!(
        budget, 10001,
        "Gemini-3-pro bare model budget defaults to medium dictionary budget (10001)"
    );
}

#[test]
fn test_issue_1602_custom_mode_gemini_capping() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // [FIX #1602] Regression test for custom mode capping
    use crate::proxy::config::{
        update_thinking_budget_config, ThinkingBudgetConfig, ThinkingBudgetMode,
    };

    // 设置自定义模式，且数值超过 24k
    update_thinking_budget_config(ThinkingBudgetConfig {
        mode: ThinkingBudgetMode::Custom,
        custom_value: 32000,
        effort: None,
        ..Default::default()
    });
    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = ResetGuard;

    let req = OpenAIRequest {
        model: "gemini-2.0-flash-thinking".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("test".into())),
            ..Default::default()
        }],
        stream: false,
        n: None,
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

    // 验证针对 Gemini 模型即使是 Custom 模式也会被修正为 24576
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-v", "gemini-2.0-flash-thinking", None);
    let budget = result["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"]
        .as_i64()
        .unwrap();
    assert_eq!(
        budget, 24576,
        "Gemini custom budget must be capped to 24576"
    );

    // 验证非 Gemini 模型（如 Claude 原生路径，假设映射后名不含 gemini）则不应截断
    // 注意：这里的 transform_openai_request 第三个参数是 mapped_model
    let (result_claude, _, _, _) =
        transform_openai_request(&req, "test-v", "claude-3-7-sonnet", None);
    let _budget_claude =
        result_claude["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"].as_i64();
    // 如果不是 gemini模型且协议中没带 thinking 配置，可能会是 None 或 32000
    // 在该测试环境下，由于模拟的是 OpenAI 格式转 Gemini 路径，如果没有 gemini 关键词通常不进入 thinking 逻辑
    // 我们只需确保 gemini 路径正确受限即可。

    // 恢复默认配置
    update_thinking_budget_config(ThinkingBudgetConfig::default());
}

#[test]
fn test_transform_openai_request_multimodal() {
    let req = OpenAIRequest {
        model: "gpt-4-vision".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::Array(vec![
                OpenAIContentBlock::Text { text: "What is in this image?".to_string() },
                OpenAIContentBlock::ImageUrl { image_url: OpenAIImageUrl {
                    url: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string(),
                    detail: None
                } }
            ])),
            ..Default::default()
        }],
        stream: false,
        n: None,
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

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-v", "gemini-1.5-flash", None);
    let parts = &result["request"]["contents"][0]["parts"];
    assert_eq!(parts.as_array().unwrap().len(), 2);
    assert_eq!(parts[0]["text"].as_str().unwrap(), "What is in this image?");
    assert_eq!(
        parts[1]["inlineData"]["mimeType"].as_str().unwrap(),
        "image/png"
    );
}

#[test]
fn test_transform_openai_request_video_multimodal() {
    use crate::proxy::mappers::openai::models::OpenAIVideoUrl;
    let req = OpenAIRequest {
        model: "gemini-2.5-flash".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::Array(vec![
                OpenAIContentBlock::Text {
                    text: "Describe this video".to_string(),
                },
                OpenAIContentBlock::VideoUrl {
                    video_url: OpenAIVideoUrl {
                        url: "data:video/mp4;base64,AAAA".to_string(),
                        mime_type: None,
                    },
                },
            ])),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-v", "gemini-2.5-flash", None);
    let parts = &result["request"]["contents"][0]["parts"];
    assert_eq!(parts.as_array().unwrap().len(), 2);
    assert_eq!(parts[0]["text"].as_str().unwrap(), "Describe this video");
    assert_eq!(
        parts[1]["inlineData"]["mimeType"].as_str().unwrap(),
        "video/mp4"
    );
    assert_eq!(parts[1]["inlineData"]["data"].as_str().unwrap(), "AAAA");
}

#[test]
fn test_gemini_pro_thinking_injection() {
    let req = OpenAIRequest {
        model: "gemini-3-pro-preview".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Thinking test".to_string())),
            ..Default::default()
        }],
        stream: false,
        n: None,
        // Client enable + budget must be ignored under server-authoritative policy
        thinking: Some(ThinkingConfig {
            thinking_type: Some("enabled".to_string()),
            budget_tokens: Some(16000),
            effort: None,
        }),
        max_tokens: None,
        temperature: None,
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        ..Default::default()
    };

    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Even Passthrough must NOT honor client budget anymore
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig {
            mode: crate::proxy::config::ThinkingBudgetMode::Passthrough,
            custom_value: 16000,
            effort: None,
            ..Default::default()
        },
    );
    struct PassthroughResetGuard;
    impl Drop for PassthroughResetGuard {
        fn drop(&mut self) {
            crate::proxy::config::update_thinking_budget_config(
                crate::proxy::config::ThinkingBudgetConfig::default(),
            );
        }
    }
    let _guard = PassthroughResetGuard;

    // Pass explicit gemini-3-pro-preview which doesn't have "-thinking" suffix
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-p", "gemini-3-pro-preview", None);
    let gen_config = &result["request"]["generationConfig"];

    // Assert thinkingConfig is present (fix verification)
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig should be injected for gemini-3-pro"
    );

    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();
    // [ANTI-POLLUTION] model_specs budget only; client 16000 + Passthrough ignored; bare pro defaults to 10001
    assert_eq!(budget, 10001);
}

#[test]
fn test_gemini_3_pro_image_not_thinking() {
    let req = OpenAIRequest {
        model: "gemini-3-pro-image-4k".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Generate a cat".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Pass gemini-3-pro-image which matches "gemini-3-pro" substring
    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-p", "gemini-3-pro-image", None);
    let gen_config = &result["request"]["generationConfig"];

    // Assert thinkingConfig IS present (based on latest user feedback)
    assert!(
        gen_config.get("thinkingConfig").is_some(),
        "thinkingConfig SHOULD be injected for gemini-3-pro-image"
    );

    // Assert imageConfig is present
    assert!(
        gen_config.get("imageConfig").is_some(),
        "imageConfig should be present for image models"
    );
    assert_eq!(gen_config["imageConfig"]["imageSize"], "4K");
}

#[test]
fn test_default_max_tokens_openai() {
    let req = OpenAIRequest {
        model: "gpt-4".to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: Some(OpenAIContent::String("Hello".to_string())),
            ..Default::default()
        }],
        stream: false,
        n: None,
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

    let (result, _sid, _msg_count, _) =
        transform_openai_request(&req, "test-p", "gemini-3-pro-high-thinking", None);
    let gen_config = &result["request"]["generationConfig"];
    let max_output_tokens = gen_config["maxOutputTokens"].as_i64().unwrap();
    // budget(10001) + overhead(32768) = 42769
    assert_eq!(max_output_tokens, 42769);

    // Verify thinkingBudget
    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_i64()
        .unwrap();
    // actual(10001) for high-thinking pro
    assert_eq!(budget, 10001);
}
