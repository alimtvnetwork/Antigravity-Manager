use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_configure_inbound_thinking_client_mode_routing_clean_isolation() {
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

        // 1. Explicitly close: completely without thinkingConfig
        let mut gc1 = json!({
            "thinkingConfig": { "includeThoughts": true }
        });
        InboundThinkingPipeline::configure_inbound_thinking(
            "gemini-3.8-flash-tiered",
            &mut gc1,
            ClientThinkingSwitch::Disabled,
            None,
            None,
            None,
        );
        assert!(gc1.get("thinkingConfig").is_none());

        // 2. Default: includeThoughts=true, no thinkingBudget
        let mut gc2 = json!({});
        InboundThinkingPipeline::configure_inbound_thinking(
            "gemini-3.8-flash-tiered",
            &mut gc2,
            ClientThinkingSwitch::Default,
            None,
            None,
            None,
        );
        let tc2 = gc2.get("thinkingConfig").unwrap().as_object().unwrap();
        assert_eq!(tc2.get("includeThoughts"), Some(&json!(true)));
        assert!(tc2.get("thinkingBudget").is_none());
        assert!(tc2.get("thinkingLevel").is_none());

        // 3. Explicit level: thinkingLevel=LOW / HIGH, no thinkingBudget, must include includeThoughts: true
        let mut gc3 = json!({});
        InboundThinkingPipeline::configure_inbound_thinking(
            "gemini-3.8-flash-tiered",
            &mut gc3,
            ClientThinkingSwitch::Enabled,
            Some("low"),
            None,
            None,
        );
        let tc3 = gc3.get("thinkingConfig").unwrap().as_object().unwrap();
        assert_eq!(tc3.get("includeThoughts"), Some(&json!(true)));
        assert_eq!(tc3.get("thinkingLevel"), Some(&json!("LOW")));
        assert!(tc3.get("thinkingBudget").is_none());

        // 3.1 If the client fills in any custom level (not hard-coded) and does not fill in the budget, the level will be faithfully and transparently transmitted, and the budget will not be filled in.
        let mut gc3_custom = json!({});
        let budget_custom_res = InboundThinkingPipeline::configure_inbound_thinking(
            "gemini-3.8-flash-tiered",
            &mut gc3_custom,
            ClientThinkingSwitch::Enabled,
            Some("custom_ultra_level"),
            None,
            None,
        );
        let tc3_custom = gc3_custom
            .get("thinkingConfig")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(tc3_custom.get("includeThoughts"), Some(&json!(true)));
        assert_eq!(
            tc3_custom.get("thinkingLevel"),
            Some(&json!("CUSTOM_ULTRA_LEVEL"))
        );
        assert!(tc3_custom.get("thinkingBudget").is_none(), "When client provides custom effort without budget, thinkingBudget must strictly remain None");
        assert!(budget_custom_res.is_none());

        // 4. Explicit budget: thinkingBudget=8192, no thinkingLevel, must include includeThoughts: true
        let mut gc4 = json!({});
        InboundThinkingPipeline::configure_inbound_thinking(
            "gemini-3.8-flash-tiered",
            &mut gc4,
            ClientThinkingSwitch::Enabled,
            None,
            Some(8192),
            None,
        );
        let tc4 = gc4.get("thinkingConfig").unwrap().as_object().unwrap();
        assert_eq!(tc4.get("includeThoughts"), Some(&json!(true)));
        assert_eq!(tc4.get("thinkingBudget"), Some(&json!(8192)));
        assert!(tc4.get("thinkingLevel").is_none());
    }

    #[test]
    fn test_cross_family_think_tag_extraction_and_elevation_for_gemini() {
        let thought_text = "Analyzing user code structure and determining route.";
        let visible_answer = "The issue has been identified and isolated.";
        let wrapped_text = format!("<think>\n{}\n</think>\n\n{}", thought_text, visible_answer);

        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": wrapped_text }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::AnthropicClaude,
            "gemini-3.8-flash-tiered",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        // 1. The first thought block successfully promoted to thought: true
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(parts[0]["text"], thought_text);
        // Iron Rule I4: A Gemini target's thought block **never** carries a signature.
        // Sentinel (skip_thought_signature_validator) does not belong to the Antigravity protocol -
        // Appears 0 times in 23 signatures in 3 official documents.
        assert!(
            parts[0].get("thoughtSignature").is_none(),
            "Think blocks for Gemini targets must not carry signatures (I4)"
        );

        // 2. The text components have been cleanly removed with <think>...</think> tags and line breaks, leaving only true answers.
        assert_eq!(parts[1]["text"], visible_answer);
        assert!(parts[1].get("thought").is_none());
    }

    // ============ Tool receipt (functionResponse) role normalization ============
    //
    // Goal: One-to-one comparison with the official Antigravity form - `functionResponse` in `role: "model"`
    // content. Actual measurement (gemini-3.8-flash-tiered@daily) shows that upstream has two types of user / model
    // role **fully tolerant**, so this normalization is "Align to official form + stable prefix byte",
    // And it is naturally backward compatible with two inbound forms.

    fn fr_part(id: &str, name: &str) -> Value {
        json!({"functionResponse": {"id": id, "name": name, "response": {"output": "ok"}}})
    }

    fn fc_part(id: &str, name: &str) -> Value {
        json!({"functionCall": {"id": id, "name": name, "args": {}}})
    }

    /// Official form (receipt already in `role:"model"`) → Must be fully idempotent.
    #[test]
    fn test_fr_role_official_shape_is_idempotent() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "view_file")]}),
            json!({"role": "model", "parts": [fr_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [{"text": "next"}]}),
        ];
        let snapshot = contents.clone();
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 0, "There should be zero changes to the official form");
        assert_eq!(contents, snapshot);
    }

    /// Inbound form: The receipt is in `role:"user"` → normalized to the official form (`role:"model"`).
    #[test]
    fn test_fr_role_user_response_is_mapped_to_model() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [fr_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [{"text": "next"}]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 1);
        assert_eq!(
            contents.len(),
            4,
            "The pure receipt wheel only changes the role and does not add or delete content."
        );
        assert_eq!(contents[2]["role"], "model");
        assert_eq!(
            contents[2]["parts"][0]["functionResponse"]["name"],
            "view_file"
        );
        // Adjacent rounds are not affected
        assert_eq!(contents[3]["role"], "user");
    }

    /// Concurrent receipts: Multiple receipts in one round are migrated as a whole, and the internal order remains intact.
    #[test]
    fn test_fr_role_parallel_responses_kept_in_order() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "list_dir"), fc_part("c2", "run_command")]}),
            json!({"role": "user", "parts": [fr_part("c1", "list_dir"), fr_part("c2", "run_command")]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 1);
        assert_eq!(contents[2]["role"], "model");
        let parts = contents[2]["parts"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["functionResponse"]["name"], "list_dir");
        assert_eq!(parts[1]["functionResponse"]["name"], "run_command");
    }

    /// Mixing wheel: `user [text, fr]` → `user[text]` + `model[fr]`, order maintained.
    #[test]
    fn test_fr_role_mixed_turn_is_split_preserving_order() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [{"text": "By the way"}, fr_part("c1", "view_file")]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 1);
        assert_eq!(contents.len(), 4);
        assert_eq!(contents[2]["role"], "user");
        assert_eq!(contents[2]["parts"][0]["text"], "By the way");
        assert_eq!(contents[3]["role"], "model");
        assert!(contents[3]["parts"][0].get("functionResponse").is_some());
    }

    /// The receipt comes with an image: `user [fr, inlineData]` → `model [fr, inlineData]`.
    #[test]
    fn test_fr_role_response_with_inline_data_moves_together() {
        let img = json!({"inlineData": {"mimeType": "image/png", "data": "AAA"}});
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [fr_part("c1", "view_file"), img]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 1);
        assert_eq!(contents[2]["role"], "model");
        assert_eq!(contents[2]["parts"].as_array().unwrap().len(), 2);
    }

    /// A user posting a picture to ask a question `user [inlineData, text]` **must** not be misjudged as a receipt wheel.
    #[test]
    fn test_fr_role_user_image_question_is_untouched() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [{"text": "ok"}]}),
            json!({"role": "user", "parts": [
                {"inlineData": {"mimeType": "image/png", "data": "AAA"}},
                {"text": "what is this picture"}
            ]}),
        ];
        let snapshot = contents.clone();
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(
            n, 0,
            "Rounds without functionResponse are never overwritten"
        );
        assert_eq!(contents, snapshot);
    }

    /// Tool call wheel(`model [fc, fc]`) is not affected.
    #[test]
    fn test_fr_role_function_call_turns_untouched() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "a"), fc_part("c2", "b")]}),
        ];
        let snapshot = contents.clone();
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 0);
        assert_eq!(contents, snapshot);
    }

    /// Defense: The first content must not be changed to `model` (Gemini requires the conversation to start with user).
    #[test]
    fn test_fr_role_leading_response_stays_user() {
        let mut contents = vec![
            json!({"role": "user", "parts": [fr_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [{"text": "hi"}]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 0);
        assert_eq!(contents[0]["role"], "user");
    }

    /// Test tool calls and receipt IDs are unified and normalized into the call_ standard form
    #[test]
    fn test_normalize_tool_call_ids() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "read file"}]}),
            json!({
                "role": "model",
                "parts": [
                    {
                        "functionCall": {
                            "name": "read",
                            "id": "call573077",
                            "args": {"path": "USER.md"}
                        }
                    }
                ]
            }),
            json!({
                "role": "user",
                "parts": [
                    {
                        "functionResponse": {
                            "name": "read",
                            "id": "call573077",
                            "response": {"result": "hello"}
                        }
                    }
                ]
            }),
            json!({
                "role": "model",
                "parts": [
                    {
                        "functionCall": {
                            "name": "exec",
                            "id": "call_1542346", // Underlined, leave as is
                            "args": {"command": "ls"}
                        }
                    }
                ]
            }),
        ];

        let count = InboundThinkingPipeline::normalize_tool_call_ids(&mut contents);
        assert_eq!(
            count, 2,
            "2 tool call/response IDs that are missing underscores should be normalized"
        );

        // functionCall verification
        assert_eq!(contents[1]["parts"][0]["functionCall"]["id"], "call_573077");
        // functionResponse verification
        assert_eq!(
            contents[2]["parts"][0]["functionResponse"]["id"],
            "call_573077"
        );
        // Native underlined values are not affected
        assert_eq!(
            contents[3]["parts"][0]["functionCall"]["id"],
            "call_1542346"
        );
    }
}
