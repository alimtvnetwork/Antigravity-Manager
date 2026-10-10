// V2 wrapper tests (split from wrapper.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

#[test]
fn test_wrap_request() {
    let body = json!({
        "model": "gemini-2.5-flash",
        "contents": [{"role": "user", "parts": [{"text": "Hi"}]}]
    });

    let result = wrap_request(&body, "test-project", "gemini-2.5-flash", None, None, None);
    assert_eq!(result["project"], "test-project");
    assert_eq!(result["model"], "gemini-2.5-flash");
    assert!(result["requestId"].as_str().unwrap().starts_with("agent/"));
}

#[test]
fn test_unwrap_response() {
    let wrapped = json!({
        "response": {
            "candidates": [{"content": {"parts": [{"text": "Hello"}]}}]
        }
    });

    let result = unwrap_response(&wrapped);
    assert!(result.get("candidates").is_some());
    assert!(result.get("response").is_none());
}

#[test]
fn test_antigravity_identity_injection_with_role() {
    let body = json!({
        "model": "gemini-pro",
        "messages": []
    });

    let result = wrap_request(&body, "test-proj", "gemini-pro", None, None, None);

    // 验证没有多余注入的 systemInstruction
    assert!(result
        .get("request")
        .unwrap()
        .get("systemInstruction")
        .is_none());
}

#[test]
fn test_gemini_flash_thinking_budget_capping() {
    // Ensure default config (Auto mode)
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );

    let body = json!({
        "model": "gemini-2.0-flash-thinking-exp",
        "generationConfig": {
            "thinkingConfig": {
                "includeThoughts": true,
                "thinkingBudget": 32000
            }
        }
    });

    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );

    // Test with Flash model
    let result = wrap_request(
        &body,
        "test-proj",
        "gemini-2.0-flash-thinking-exp",
        None,
        None,
        None,
    );
    let req = result.get("request").unwrap();
    let gen_config = req.get("generationConfig").unwrap();
    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();

    // Should be capped at 24576
    assert_eq!(budget, 24576);

    // Test with Pro model (should NOT cap)
    let body_pro = json!({
        "model": "gemini-2.0-pro-exp",
        "generationConfig": {
            "thinkingConfig": {
                "includeThoughts": true,
                "thinkingBudget": 32000
            }
        }
    });
    let result_pro = wrap_request(
        &body_pro,
        "test-proj",
        "gemini-2.0-pro-exp",
        None,
        None,
        None,
    );
    let budget_pro = result_pro["request"]["generationConfig"]["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();
    // Pro models without suffix now default to 10001 in wrap_request logic
    assert_eq!(budget_pro, 10001);
}

#[test]
fn test_image_thinking_mode_disabled() {
    let _test_lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _config_lock = crate::proxy::config::TEST_CONFIG_LOCK
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

    // 2. Create a request for an image model (which triggers the image logic)
    // Note: resolve_request_config needs to return image_config for the logic to trigger
    // So we use a model name that resolves to image_gen
    let body = json!({
        "model": "gemini-3-pro-image-2k",
        "contents": [{"role": "user", "parts": [{"text": "Draw a cat"}]}]
    });

    let result = wrap_request(
        &body,
        "test-proj",
        "gemini-3-pro-image-2k",
        None,
        None,
        None,
    );
    let req = result.get("request").unwrap();
    let gen_config = req.get("generationConfig").unwrap();

    // 3. Verify thinkingConfig has includeThoughts: false
    let thinking_config = gen_config.get("thinkingConfig").unwrap();
    assert_eq!(thinking_config["includeThoughts"], false);
}

#[test]
fn test_user_instruction_preservation() {
    let body = json!({
        "model": "gemini-pro",
        "systemInstruction": {
            "role": "user",
            "parts": [{"text": "User custom prompt"}]
        }
    });

    let result = wrap_request(&body, "test-proj", "gemini-pro", None, None, None);
    let sys = result
        .get("request")
        .unwrap()
        .get("systemInstruction")
        .unwrap();
    let parts = sys.get("parts").unwrap().as_array().unwrap();

    // User custom prompt is preserved without injecting unwanted Antigravity identity
    assert_eq!(parts.len(), 1);
    assert_eq!(
        parts[0].get("text").unwrap().as_str().unwrap(),
        "User custom prompt"
    );
}

#[test]
fn test_duplicate_prevention() {
    let body = json!({
        "model": "gemini-pro",
        "systemInstruction": {
            "parts": [{"text": "You are Antigravity..."}]
        }
    });

    let result = wrap_request(&body, "test-proj", "gemini-pro", None, None, None);
    let sys = result
        .get("request")
        .unwrap()
        .get("systemInstruction")
        .unwrap();
    let parts = sys.get("parts").unwrap().as_array().unwrap();

    // Should NOT inject duplicate, so only 1 part remains
    assert_eq!(parts.len(), 1);
}

#[test]
fn test_image_generation_with_reference_images() {
    // Create 14 reference images + 1 text prompt
    let mut parts = Vec::new();
    parts.push(json!({"text": "Generate a variation"}));

    for _ in 0..14 {
        parts.push(json!({
            "inlineData": {
                "mimeType": "image/jpeg",
                "data": "base64data..."
            }
        }));
    }

    let body = json!({
        "model": "gemini-3-pro-image",
        "contents": [{"parts": parts}]
    });

    let result = wrap_request(&body, "test-proj", "gemini-3-pro-image", None, None, None);

    let request = result.get("request").unwrap();
    let contents = request.get("contents").unwrap().as_array().unwrap();
    let result_parts = contents[0].get("parts").unwrap().as_array().unwrap();

    // Verify all 15 parts (1 text + 14 images) are preserved
    assert_eq!(result_parts.len(), 15);
}

#[test]
fn test_gemini_pro_thinking_budget_processing() {
    let _test_lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _config_lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Update global config to Custom mode to verify logic execution
    use crate::proxy::config::{
        update_thinking_budget_config, ThinkingBudgetConfig, ThinkingBudgetMode,
    };

    update_thinking_budget_config(ThinkingBudgetConfig {
        mode: ThinkingBudgetMode::Custom,
        custom_value: 1024, // Distinct value
        effort: None,
        ..Default::default()
    });
    struct GeminiCustomResetGuard;
    impl Drop for GeminiCustomResetGuard {
        fn drop(&mut self) {
            update_thinking_budget_config(ThinkingBudgetConfig::default());
        }
    }
    let _guard = GeminiCustomResetGuard;

    let body = json!({
        "model": "gemini-3-pro-preview",
        "generationConfig": {
            "thinkingConfig": {
                "includeThoughts": true,
                "thinkingBudget": 32000
            }
        }
    });

    // Test with Pro model
    let result = wrap_request(&body, "test-proj", "gemini-3-pro-preview", None, None, None);
    let req = result.get("request").unwrap();
    let gen_config = req.get("generationConfig").unwrap();

    let budget = gen_config["thinkingConfig"]["thinkingBudget"]
        .as_u64()
        .unwrap();

    // If logic executes, it sees Custom mode and sets 1024
    // If logic skipped, it keeps 32000
    assert_eq!(
        budget, 1024,
        "Budget should be overridden to 1024 by custom config, proving logic execution"
    );
}

#[cfg(test)]
mod test_v4_fixes {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_claude_no_root_thinking_injection() {
        let _test_lock = super::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _config_lock = crate::proxy::config::TEST_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // 验证 Claude 模型不会在根目录注入 thinking，而是注入到 generationConfig.thinkingConfig
        // 并且 budget 默认为 16000

        // 使用 Auto 模式避免干扰
        crate::proxy::config::update_thinking_budget_config(
            crate::proxy::config::ThinkingBudgetConfig {
                mode: crate::proxy::config::ThinkingBudgetMode::Auto,
                custom_value: 0,
                effort: None,
                ..Default::default()
            },
        );

        let body = json!({
            "model": "claude-3-7-sonnet-thinking",
            "messages": [{"role": "user", "content": "hi"}]
        });

        let result = wrap_request(
            &body,
            "proj",
            "claude-3-7-sonnet-thinking",
            None,
            None,
            None,
        );
        let req = result.get("request").unwrap();

        // 1. 确保根目录没有 thinking
        assert!(
            req.get("thinking").is_none(),
            "Root level 'thinking' should NOT be present"
        );

        // 2. 确保 generationConfig.thinkingConfig 存在
        let gen_config = req
            .get("generationConfig")
            .expect("generationConfig should be present");
        let thinking_config = gen_config
            .get("thinkingConfig")
            .expect("thinkingConfig should be injected");

        // 3. 验证 Claude 默认预算为 16384
        let budget = thinking_config["thinkingBudget"]
            .as_u64()
            .expect("thinkingBudget should be a number");
        assert_eq!(
            budget, 16384,
            "Claude default thinking budget should be 16384"
        );
    }

    #[test]
    fn test_gemini_thinking_injection_default() {
        let _test_lock = super::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _config_lock = crate::proxy::config::TEST_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::proxy::config::update_thinking_budget_config(
            crate::proxy::config::ThinkingBudgetConfig::default(),
        );
        // 验证 Gemini 模型注入默认预算 24576
        let body = json!({
            "model": "gemini-2.0-flash-thinking-exp",
            "contents": [{"role": "user", "parts": [{"text": "hi"}]}]
        });

        let result = wrap_request(
            &body,
            "proj",
            "gemini-2.0-flash-thinking-exp",
            None,
            None,
            None,
        );
        let req = result.get("request").unwrap();
        let gen_config = req.get("generationConfig").unwrap();
        let thinking_config = gen_config.get("thinkingConfig").unwrap();

        let budget = thinking_config["thinkingBudget"].as_u64().unwrap();
        assert_eq!(
            budget, 24576,
            "Gemini default thinking budget should be 24576"
        );
    }
}

#[test]
fn test_gemini_pro_auto_inject_thinking() {
    let _test_lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _config_lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Reset thinking budget to auto mode at the start to avoid interference from parallel tests
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig {
            mode: crate::proxy::config::ThinkingBudgetMode::Auto,
            custom_value: 24576,
            effort: None,
            ..Default::default()
        },
    );

    // Request WITHOUT thinkingConfig
    let body = json!({
        "model": "gemini-3-pro-preview",
        // No generationConfig or empty one
        "generationConfig": {}
    });

    // Test with Pro-preview model (should NOT auto-inject to avoid 400)
    let result = wrap_request(&body, "test-proj", "gemini-3-pro-preview", None, None, None);
    let req = result.get("request").unwrap();
    let gen_config = req.get("generationConfig").unwrap();

    // Should NOT have auto-injected thinkingConfig
    assert!(
        gen_config.get("thinkingConfig").is_none(),
        "Should NOT auto-inject thinkingConfig for gemini-3-pro-preview to avoid 400 error"
    );

    // Test with standard gemini-3-pro (non-preview)
    let body_std = json!({
        "model": "gemini-3-pro",
        "generationConfig": {}
    });
    let result_std = wrap_request(&body_std, "test-proj", "gemini-3-pro", None, None, None);
    let gen_config_std = result_std
        .get("request")
        .unwrap()
        .get("generationConfig")
        .unwrap();

    assert!(
        gen_config_std.get("thinkingConfig").is_some(),
        "Should still auto-inject thinkingConfig for standard gemini-3-pro"
    );
}
