// V2 wrapper tests (split from wrapper.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

#[test]
fn test_openai_image_params_support() {
    // Test Case 1: Standard Size + Quality (HD/4K)
    let body_1 = json!({
        "model": "gemini-3-pro-image",
        "size": "1920x1080",
        "quality": "hd",
        "prompt": "Test"
    });

    let result_1 = wrap_request(&body_1, "test-proj", "gemini-3-pro-image", None, None, None);
    let req_1 = result_1.get("request").unwrap();
    let gen_config_1 = req_1.get("generationConfig").unwrap();
    let image_config_1 = gen_config_1.get("imageConfig").unwrap();

    assert_eq!(image_config_1["aspectRatio"], "16:9");
    assert_eq!(image_config_1["imageSize"], "4K");

    // Test Case 2: Aspect Ratio String + Standard Quality
    let body_2 = json!({
        "model": "gemini-3-pro-image",
        "size": "1:1",
        "quality": "standard",
         "prompt": "Test"
    });

    let result_2 = wrap_request(&body_2, "test-proj", "gemini-3-pro-image", None, None, None);
    let req_2 = result_2.get("request").unwrap();
    let image_config_2 = req_2["generationConfig"]["imageConfig"]
        .as_object()
        .unwrap();

    assert_eq!(image_config_2["aspectRatio"], "1:1");
    assert_eq!(image_config_2["imageSize"], "1K");
}

#[test]
fn test_mixed_tools_injection_gemini_native() {
    // Verify Gemini Native protocol supports hybrid tools under Gemini 2.0+
    let body = json!({
        "contents": [{"parts": [{"text": "Hello"}]}],
        "tools": [{"functionDeclarations": [{"name": "get_weather", "parameters": {"type": "OBJECT", "properties": {"location": {"type": "STRING"}}}}]}],
        "generationConfig": {}
    });

    // Simulate a RequestConfig triggered by -online
    use crate::proxy::mappers::common_utils::resolve_request_config;
    let _config =
        resolve_request_config("-online", "gemini-2.0-flash", &None, None, None, None, None);

    // In fact, wrap_request internally calls inject_google_search_tool based on config.inject_google_search
    // But the signature of wrap_request does not directly accept RequestConfig. Its internal logic is as follows:
    // if config.inject_google_search { ... }

    // We instead directly test the piece of wrap_request logic involved.
    // Since testing wrap_request is more complex (involving external config),
    // We can directly verify the performance of inject_google_search_tool in native format.

    let mut inner_request = body.clone();
    crate::proxy::mappers::common_utils::inject_google_search_tool(
        &mut inner_request,
        Some("gemini-2.0-flash"),
    );

    let tools = inner_request["tools"]
        .as_array()
        .expect("Should have tools");
    let has_functions = tools
        .iter()
        .any(|t| t.get("functionDeclarations").is_some());
    let has_google_search = tools.iter().any(|t| t.get("googleSearch").is_some());

    assert!(has_functions, "Should contain functionDeclarations");
    assert!(
        !has_google_search,
        "Should NOT contain googleSearch due to functionDeclarations presence (preventing client tool dispatch conflicts)"
    );
}

#[test]
fn test_gemini_wrapper_context_compression() {
    crate::proxy::config::update_global_compression_level("high".to_string(), true);
    let body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [{"text": "Hello there! Could you please tell me how to fix this?"}]
            },
            {
                "role": "model",
                "parts": [{"text": "Basically, it appears to be a bug."}]
            },
            {
                "role": "user",
                "parts": [{"text": "Old message 3. I was wondering if you could help."}]
            },
            {
                "role": "user",
                "parts": [
                    {
                        "functionResponse": {
                            "name": "run_test",
                            "response": {
                                "output": "Progress: 10%\nProgress: 20%\nProgress: 30%\nProgress: 40%\nProgress: 50%\nError: compilation failed"
                            }
                        }
                    }
                ]
            },
            {
                "role": "user",
                "parts": [{"text": "Latest message 1. Please keep this."}]
            },
            {
                "role": "model",
                "parts": [{"text": "Latest message 2. Of course!"}]
            }
        ],
        "model": "gemini-2.5-pro"
    });

    let wrapped = wrap_request(&body, "test-proj", "gemini-2.5-pro", None, None, None);
    println!(
        "DEBUG: wrapped = {}",
        serde_json::to_string_pretty(&wrapped).unwrap()
    );
    let contents = wrapped["request"]["contents"].as_array().unwrap();

    let text_1 = contents[0]["parts"][0]["text"].as_str().unwrap();
    assert!(!text_1.contains("please"));
    assert!(!text_1.contains("Could you please"));

    let text_2 = contents[1]["parts"][0]["text"].as_str().unwrap();
    assert!(!text_2.contains("Basically"));

    let tool_resp = contents[3]["parts"][0]["functionResponse"]["response"]["output"]
        .as_str()
        .unwrap();
    assert!(tool_resp.contains("Collapsed"));
    assert!(tool_resp.contains("Error: compilation failed"));

    let text_5 = contents[4]["parts"][0]["text"].as_str().unwrap();
    assert!(text_5.contains("Please"));
}

#[test]
fn test_gemini_anthropic_alignment_thinking_and_signatures() {
    let valid_sig = "A".repeat(60); // 60 chars valid signature
    let body = json!({
        "contents": [
            {
                "role": "model",
                "parts": [
                    {
                        "thought": true,
                        "text": "·", // placeholder thought
                        "thoughtSignature": valid_sig
                    },
                    {
                        "thought": true,
                        "text": "second thought block that should be downgraded",
                        "thoughtSignature": "short"
                    },
                    {
                        "text": "Regular model response"
                    }
                ]
            }
        ],
        "generationConfig": {
            "thinkingConfig": {
                "thinkingBudget": 2048
            }
        }
    });

    let wrapped = wrap_request(&body, "test-proj", "gemini-3-pro", None, None, None);
    let contents = wrapped["request"]["contents"].as_array().unwrap();
    let model_msg = &contents[0];
    let parts = model_msg["parts"].as_array().unwrap();

    // 【2026-09-27】The placeholder thinking block ("·") is discarded directly and is no longer normalized to "..."
    // Part 0 should be the second (meaningful) thinking block, now promoted to the head
    assert_eq!(parts[0]["thought"], true);
    assert_eq!(
        parts[0]["text"],
        "second thought block that should be downgraded"
    );

    // Part 1 remains regular text
    assert_eq!(parts[1]["text"], "Regular model response");
}

#[test]
fn test_gemini_thinking_level_authority_resolution() {
    let _lock = crate::proxy::config::TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::proxy::config::update_thinking_budget_config(
        crate::proxy::config::ThinkingBudgetConfig::default(),
    );
    // 1. Heuristic model ignores client thinkingLevel
    let req_high = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingLevel": "LOW" }
        }
    });
    let wrapped = wrap_request(
        &req_high,
        "test-p",
        "gemini-3.7-flash-high",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 16384);
    assert!(tc.get("thinkingLevel").is_none());

    // 2. Naked model Flash takes over client thinkingLevel
    let req_flash_high = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingLevel": "HIGH" }
        }
    });
    let wrapped = wrap_request(
        &req_flash_high,
        "test-p",
        "gemini-3-flash",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 16384);
    assert!(tc.get("thinkingLevel").is_none());

    let req_flash_low = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingLevel": "LOW" }
        }
    });
    let wrapped = wrap_request(&req_flash_low, "test-p", "gemini-3-flash", None, None, None);
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 1024);
    assert!(tc.get("thinkingLevel").is_none());

    // 3. Naked model Flash client passes NONE or not: never close thinking and force backfill -medium (4096)
    let req_flash_none = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingLevel": "NONE" }
        }
    });
    let wrapped = wrap_request(
        &req_flash_none,
        "test-p",
        "gemini-3-flash",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 4096);
    assert!(tc.get("thinkingLevel").is_none());

    let req_flash_empty = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}]
    });
    let wrapped = wrap_request(
        &req_flash_empty,
        "test-p",
        "gemini-3-flash",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 4096);

    // 4. The naked model Flash client passes in a custom thinkingBudget: it is completely ignored and is backfilled by the server's authority level.
    let req_flash_custom_budget = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingBudget": 12345 }
        }
    });
    let wrapped = wrap_request(
        &req_flash_custom_budget,
        "test-p",
        "gemini-3-flash",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 4096);

    let req_flash_high_custom_budget = json!({
        "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
        "generationConfig": {
            "thinkingConfig": { "thinkingLevel": "HIGH", "thinkingBudget": 1234 }
        }
    });
    let wrapped = wrap_request(
        &req_flash_high_custom_budget,
        "test-p",
        "gemini-3-flash",
        None,
        None,
        None,
    );
    let tc = &wrapped["request"]["generationConfig"]["thinkingConfig"];
    assert_eq!(tc["thinkingBudget"], 16384);
}
