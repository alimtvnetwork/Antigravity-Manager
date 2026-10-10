// V2 thinking phase (split from wrapper.rs).
use serde_json::json;

pub(crate) fn phase_thinking(
    inner_request: &mut serde_json::Value,
    final_model_name: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    lower_model: &str,
    is_under_v3: bool,
    force_server_thinking: bool,
    should_inject: bool,
) {
    // [FIX Issue #1355] Gemini Flash thinking budget capping
    // [CONFIGURABLE] 现在改为遵循全局 Thinking Budget 配置
    // [FIX #1557] Also apply to Pro/Thinking models to ensure budget processing
    // [FIX #1557] Auto-inject thinkingConfig if missing for these models
    if force_server_thinking
        || lower_model.contains("flash")
        || lower_model.contains("pro")
        || lower_model.contains("thinking")
        || lower_model.contains("agent")
        || lower_model.contains("gemini")
    {
        // [NEW] Extract OpenAI/Claude-style max_tokens before mutably borrowing gen_config
        let req_max_tokens = inner_request
            .get("max_tokens")
            .or_else(|| inner_request.get("max_completion_tokens"))
            .or_else(|| inner_request.get("maxCompletionTokens"))
            .or_else(|| inner_request.get("maxTokens"))
            .and_then(|v| v.as_u64());

        // Determine model family and capability beforehand to avoid borrow checker conflicts
        let is_claude = lower_model.contains("claude");

        if should_inject {
            // Scope for borrowing inner_request/gen_config
            let has_thinking = if is_claude {
                inner_request.get("thinking").is_some()
            } else {
                inner_request
                    .get("generationConfig")
                    .and_then(|v| v.as_object())
                    .map_or(false, |gc| gc.get("thinkingConfig").is_some())
            };

            let tb_config = crate::proxy::config::get_thinking_budget_config();
            let is_client_control =
                tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;

            let default_budget =
                crate::proxy::model_specs::get_thinking_budget(final_model_name, token);

            let is_explicit_tier =
                crate::proxy::model_specs::is_explicit_heuristic_tier_model(final_model_name);

            // [ANTI-POLLUTION] 对齐 Anthropic 与 OpenAI：对于未设置思考配置或显式档位模型，设定权威 default_budget；对于裸模型保留客户端配置供后续 resolve_authoritative_thinking_budget 仲裁
            // 客户端直接控制模式下，严禁篡改覆盖客户端的思考意图
            let should_override_budget = !is_client_control && (!has_thinking || is_explicit_tier);

            if should_override_budget {
                tracing::debug!(
                    "[Gemini-Wrap] Enforcing authoritative thinking budget {} for {}",
                    default_budget,
                    final_model_name
                );

                let gen_config = inner_request
                    .as_object_mut()
                    .unwrap()
                    .entry("generationConfig")
                    .or_insert(json!({}))
                    .as_object_mut()
                    .unwrap();

                gen_config.insert(
                    "thinkingConfig".to_string(),
                    json!({
                        "includeThoughts": true,
                        "thinkingBudget": default_budget
                    }),
                );
            }
        }

        // Re-acquire gen_config to satisfy borrow checker and scope requirements for later logic
        let gen_config = inner_request
            .as_object_mut()
            .unwrap()
            .entry("generationConfig")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        if is_under_v3 {
            gen_config.remove("thinkingConfig");
        }

        // [ADDED v4.1.24] Inject topK=40 and topP=1.0 if not present to match official client
        if !gen_config.contains_key("topK") {
            gen_config.insert("topK".to_string(), json!(40));
        }
        if !gen_config.contains_key("topP") {
            gen_config.insert("topP".to_string(), json!(1.0));
        }

        if force_server_thinking {
            let default_budget =
                crate::proxy::model_specs::get_thinking_budget(final_model_name, token);
            let thinking_config = gen_config
                .entry("thinkingConfig".to_string())
                .or_insert(json!({}))
                .as_object_mut()
                .unwrap();
            thinking_config.insert("includeThoughts".to_string(), json!(true));
            if !thinking_config.contains_key("thinkingBudget")
                && !thinking_config.contains_key("thinkingLevel")
            {
                thinking_config.insert("thinkingBudget".to_string(), json!(default_budget));
            }
            tracing::debug!(
                "[Gemini-Wrap] Forced includeThoughts=true for keyword model {}",
                final_model_name
            );
        }

        // [AUTHORITATIVE RESOLUTION] 全协议统一由进站流水线节点解析思考预算
        let has_thinking_config = gen_config.contains_key("thinkingConfig");
        let client_level = gen_config
            .get("thinkingConfig")
            .and_then(|t| {
                t.get("thinkingLevel")
                    .or_else(|| t.get("thinking_level"))
                    .or_else(|| t.get("reasoning_effort"))
                    .or_else(|| t.get("reasoningEffort"))
                    .or_else(|| t.get("effort"))
            })
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let client_budget = gen_config
            .get("thinkingConfig")
            .and_then(|t| {
                t.get("thinkingBudget")
                    .or_else(|| t.get("thinking_budget"))
                    .or_else(|| t.get("budget_tokens"))
                    .or_else(|| t.get("budgetTokens"))
                    .or_else(|| t.get("max_tokens"))
                    .or_else(|| t.get("maxTokens"))
            })
            .and_then(|v| v.as_i64());

        let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
            None,
            client_budget.map(|b| if b <= 0 { 0 } else { b as u64 }),
            client_level.as_deref(),
        );

        let tb_config = crate::proxy::config::get_thinking_budget_config();
        let is_client_control =
            tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;
        let budget_opt = if has_thinking_config || force_server_thinking {
            let mut gc_val = serde_json::Value::Object(std::mem::take(gen_config));
            let client_budget_for_pipeline = if is_client_control {
                client_budget.filter(|b| *b >= 0).map(|b| b as u64)
            } else {
                client_budget.filter(|b| *b > 0).map(|b| b as u64)
            };
            let resolved =
                crate::proxy::pipeline::InboundThinkingPipeline::configure_inbound_thinking(
                    final_model_name,
                    &mut gc_val,
                    client_switch,
                    client_level.as_deref(),
                    client_budget_for_pipeline,
                    token,
                );
            if let serde_json::Value::Object(map) = gc_val {
                *gen_config = map;
            }
            resolved
        } else {
            None
        };

        if !is_client_control {
            if let Some(tc) = gen_config
                .get_mut("thinkingConfig")
                .and_then(|v| v.as_object_mut())
            {
                tracing::info!(
                    "[Gemini-Wrap] Pipeline thinking budget {:?} for {} (client_level={:?})",
                    budget_opt,
                    final_model_name,
                    client_level
                );
                tc.remove("thinkingLevel");
            }
        }

        // [FIX #1747] Ensure max_tokens (maxOutputTokens) is greater than thinking_budget
        // Google v1internal requires maxOutputTokens > thinkingBudget.
        // [FIX #1825] Handle adaptive fallback (incl. -1 and thinkingLevel)
        let thinking_config_opt = gen_config.get("thinkingConfig");
        let is_adaptive = thinking_config_opt.map_or(false, |t| {
            t.get("thinkingLevel").is_some()
                || t.get("thinkingBudget").is_none()
                || t.get("thinkingBudget").and_then(|v| v.as_i64()) == Some(-1)
        }) || (thinking_config_opt
            .and_then(|t| t.get("thinkingBudget").and_then(|v| v.as_u64()))
            == Some(32768)
            && is_claude);

        if let Some(thinking_config) = gen_config.get("thinkingConfig") {
            let budget_opt = thinking_config
                .get("thinkingBudget")
                .and_then(|v| v.as_i64());

            // For adaptive or dynamic mode, we only need to ensure max tokens is large.
            // For fixed budget, we must satisfy maxOutputTokens > thinkingBudget.
            let current_max = gen_config
                .get("maxOutputTokens")
                .and_then(|v| v.as_u64())
                .or(req_max_tokens);

            if is_adaptive {
                if current_max.map_or(true, |m| m < 131072) {
                    gen_config.insert("maxOutputTokens".to_string(), json!(131072));
                }
            } else if let Some(budget_i64) = budget_opt {
                if budget_i64 > 0 {
                    let budget = budget_i64 as u64;
                    let min_required_max = budget + 8192;
                    if current_max.map_or(true, |m| m <= budget) {
                        tracing::info!(
                            "[Gemini-Wrap] Bumping maxOutputTokens from {:?} to {} to satisfy thinkingBudget ({})",
                            current_max, min_required_max, budget
                        );
                        gen_config.insert("maxOutputTokens".to_string(), json!(min_required_max));
                    }
                }
            }
        }
    }
}
