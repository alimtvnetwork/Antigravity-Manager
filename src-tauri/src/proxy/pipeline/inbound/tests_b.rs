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

        // 1. 显式关闭：彻底不带 thinkingConfig
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

        // 2. 缺省：includeThoughts=true，绝无 thinkingBudget
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

        // 3. 显式等级：thinkingLevel=LOW / HIGH，绝无 thinkingBudget，必须带上 includeThoughts: true
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

        // 3.1 客户端填了任何自定义等级（非硬编码）且没填预算，忠实透传等级，坚决不填预算
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

        // 4. 显式预算：thinkingBudget=8192，绝无 thinkingLevel，必须带上 includeThoughts: true
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
        // 1. 首位成功提升为 thought: true 的思考块
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(parts[0]["text"], thought_text);
        // 铁律 I4：Gemini 目标的思考块**绝不**携带签名。
        // 哨兵（skip_thought_signature_validator）不属于 Antigravity 协议 ——
        // 官方 3 份报文 23 处签名里出现 0 次。
        assert!(
            parts[0].get("thoughtSignature").is_none(),
            "Gemini 目标的思考块不得携带签名（I4）"
        );

        // 2. 正文部件已干净剔除 <think>...</think> 标签与换行，仅保留真实回答
        assert_eq!(parts[1]["text"], visible_answer);
        assert!(parts[1].get("thought").is_none());
    }

    // ============ 工具回执（functionResponse）role 归一化 ============
    //
    // 目标：与官方 Antigravity 形态一比一 —— `functionResponse` 位于 `role: "model"` 的
    // content 中。实测（gemini-3.8-flash-tiered @ daily）表明上游对 user / model 两种
    // role **完全宽容**，故本归一化是「对齐官方形态 + 稳定前缀字节」，
    // 并天然**向下兼容**两种入站形态。

    fn fr_part(id: &str, name: &str) -> Value {
        json!({"functionResponse": {"id": id, "name": name, "response": {"output": "ok"}}})
    }

    fn fc_part(id: &str, name: &str) -> Value {
        json!({"functionCall": {"id": id, "name": name, "args": {}}})
    }

    /// 官方形态（回执已在 `role:"model"`）→ 必须完全幂等。
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
        assert_eq!(n, 0, "官方形态应零改动");
        assert_eq!(contents, snapshot);
    }

    /// 入站形态：回执在 `role:"user"` → 归一到官方形态（`role:"model"`）。
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
        assert_eq!(contents.len(), 4, "纯回执轮只改 role，不增删 content");
        assert_eq!(contents[2]["role"], "model");
        assert_eq!(
            contents[2]["parts"][0]["functionResponse"]["name"],
            "view_file"
        );
        // 相邻轮次不受影响
        assert_eq!(contents[3]["role"], "user");
    }

    /// 并发回执：一轮多个回执整体迁移，**内部顺序原样保持**。
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

    /// 混合轮：`user [text, fr]` → `user[text]` + `model[fr]`，顺序保持。
    #[test]
    fn test_fr_role_mixed_turn_is_split_preserving_order() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [fc_part("c1", "view_file")]}),
            json!({"role": "user", "parts": [{"text": "顺便说明"}, fr_part("c1", "view_file")]}),
        ];
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 1);
        assert_eq!(contents.len(), 4);
        assert_eq!(contents[2]["role"], "user");
        assert_eq!(contents[2]["parts"][0]["text"], "顺便说明");
        assert_eq!(contents[3]["role"], "model");
        assert!(contents[3]["parts"][0].get("functionResponse").is_some());
    }

    /// 回执附带图片：`user [fr, inlineData]` → `model [fr, inlineData]`。
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

    /// 用户发图提问 `user [inlineData, text]` **不得**被误判为回执轮。
    #[test]
    fn test_fr_role_user_image_question_is_untouched() {
        let mut contents = vec![
            json!({"role": "user", "parts": [{"text": "go"}]}),
            json!({"role": "model", "parts": [{"text": "ok"}]}),
            json!({"role": "user", "parts": [
                {"inlineData": {"mimeType": "image/png", "data": "AAA"}},
                {"text": "这张图是什么"}
            ]}),
        ];
        let snapshot = contents.clone();
        let n = InboundThinkingPipeline::normalize_function_response_roles(&mut contents);
        assert_eq!(n, 0, "无 functionResponse 的轮次绝不改写");
        assert_eq!(contents, snapshot);
    }

    /// 工具调用轮（`model [fc, fc]`）不受影响。
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

    /// 防御：首条 content 不得被改成 `model`（Gemini 要求对话以 user 开头）。
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

    /// 测试工具调用与回执 ID 统一归一化为 call_ 规范形态
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
                            "id": "call_1542346", // 已带下划线，保持原样
                            "args": {"command": "ls"}
                        }
                    }
                ]
            }),
        ];

        let count = InboundThinkingPipeline::normalize_tool_call_ids(&mut contents);
        assert_eq!(count, 2, "应归一化 2 个丢失下划线的 tool call/response ID");

        // functionCall 验证
        assert_eq!(contents[1]["parts"][0]["functionCall"]["id"], "call_573077");
        // functionResponse 验证
        assert_eq!(
            contents[2]["parts"][0]["functionResponse"]["id"],
            "call_573077"
        );
        // 原生已带下划线不受影响
        assert_eq!(
            contents[3]["parts"][0]["functionCall"]["id"],
            "call_1542346"
        );
    }
}
