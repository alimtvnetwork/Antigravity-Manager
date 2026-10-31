#[cfg(test)]
mod tests {
    use crate::proxy::mappers::claude::models::{
        ClaudeRequest, ContentBlock, Message, MessageContent, ThinkingConfig,
    };
    use crate::proxy::mappers::claude::request::transform_claude_request_in;
    use crate::proxy::mappers::claude::thinking_utils::{
        analyze_conversation_state, close_tool_loop_for_thinking,
    };
    use serde_json::json;

    // ==================================================================================
    // Scenario 1: First Thinking request (P0-2 Fix)
    // Verify whether the first Thinking request without a historical signature is allowed (Perimssive Mode)
    // ==================================================================================
    #[test]
    fn test_first_thinking_request_permissive_mode() {
        // 1. Construct a brand new request (no historical messages)
        let req = ClaudeRequest {
            model: "claude-3-7-sonnet-20250219".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("Hello, please think.".to_string()),
            }],
            system: None,
            tools: None, // No tool call
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

        // 2. Perform conversion
        // If the fix took effect, this should return successfully and thinkingConfig is retained
        let result =
            transform_claude_request_in(&req, "test-project", false, None, "test_session", None);
        assert!(result.is_ok(), "First thinking request should be allowed");

        let body = result.unwrap();
        let request = &body["request"];

        // Verify thinkingConfig exists (i.e. thinking mode is not disabled)
        let has_thinking_config = request
            .get("generationConfig")
            .and_then(|g| g.get("thinkingConfig"))
            .is_some();

        assert!(
            has_thinking_config,
            "Thinking config should be preserved for first request without tool calls"
        );
    }

    // ==================================================================================
    // Scenario 2: Tool cycle recovery (P1-4 Fix)
    // Verify whether synthetic messages will be automatically injected to close the loop when the Thinking block is lost in the historical message, resulting in an infinite loop.
    // ==================================================================================
    #[test]
    fn test_tool_loop_recovery() {
        // 1. Construct a "Broken Tool Loop" scene
        // Assistant (ToolUse) -> User (ToolResult)
        // But the Thinking block is missing in the Assistant message (simulating being stripped)
        let mut messages = vec![
            Message {
                role: "user".to_string(),
                content: MessageContent::String("Check weather".to_string()),
            },
            Message {
                role: "assistant".to_string(),
                content: MessageContent::Array(vec![
                    // Only ToolUse, no Thinking (Broken State)
                    ContentBlock::ToolUse {
                        id: "call_1".to_string(),
                        name: "get_weather".to_string(),
                        input: json!({"location": "Beijing"}),
                        signature: None,
                        cache_control: None,
                    },
                ]),
            },
            Message {
                role: "user".to_string(),
                content: MessageContent::Array(vec![ContentBlock::ToolResult {
                    tool_use_id: "call_1".to_string(),
                    content: json!("Sunny"),
                    is_error: None,
                }]),
            },
        ];

        // 2. Analyze the current status
        let state = analyze_conversation_state(&messages);
        assert!(state.in_tool_loop, "Should detect tool loop");

        // 3. Execute recovery logic
        close_tool_loop_for_thinking(&mut messages);

        // 4. Verify whether synthetic messages are injected
        assert_eq!(
            messages.len(),
            5,
            "Should have injected 2 synthetic messages"
        );

        // Verify that the second to last message is Assistant's "Completed" message
        let injected_assistant = &messages[3];
        assert_eq!(injected_assistant.role, "assistant");

        // Verify that the last message is User's "Proceed" message
        let injected_user = &messages[4];
        assert_eq!(injected_user.role, "user");

        // In this way, the current state is no longer "in_tool_loop" (the last one is User Text), and the model can start a new Thinking
        let new_state = analyze_conversation_state(&messages);
        assert!(!new_state.in_tool_loop, "Tool loop should be broken/closed");
    }

    // ==================================================================================
    // Scenario Three: Cross-Model Compatibility (P1-5 Fix) - Simulation
    // Since is_model_compatible in request.rs is private, we verify the effect through integration testing
    // ==================================================================================
    /*
        // Note: Since is_model_compatible and cache logic are deeply integrated in transform_claude_request_in,
       且依赖全局单例 SignatureCache，单元测试较难模拟 "Old signature cached but model switched" 的状态。
       这里主要通过验证 "Incompatible signatures are discarded" 的副作用（即 thoughtSignature 字段消息）来测试。
       但由于 SignatureCache 是全局的，我们无法在测试中轻易预置状态。
       因此，此场景主要依赖 Verification Guide 中的手动测试。
       或者，我们可以测试 request.rs 中公开的某些 helper (如果有的话)，但目前没有。
    */
}
