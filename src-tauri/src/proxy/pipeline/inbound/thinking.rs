use super::*;

impl InboundThinkingPipeline {
    /// 统一进站思考配置与参数治理（流水线节点）：
    /// 保证四大协议（OpenAI, Claude, Gemini, Codex）的协议无关性。
    /// 1. 自动识别目标模型（包括 Tiered 自适应模型与具名模型）
    /// 2. 忽略客户端数字 budget 防污染，精准捕获客户端无后缀思考参数 (low/medium/high)
    /// 3. 在网关控制模式下，思考参数仅对 Tiered 模型开放操控权，分别映射至网关思考板块的 flash_low, flash_medium, flash_high
    /// 4. 组装并规范化 generationConfig 中的 thinkingConfig 与 maxOutputTokens
    pub fn configure_inbound_thinking(
        target_model: &str,
        generation_config: &mut Value,
        client_switch: ClientThinkingSwitch,
        client_effort: Option<&str>,
        client_budget: Option<u64>,
        token: Option<&crate::proxy::token_manager::ProxyToken>,
    ) -> Option<i64> {
        let is_under_v3 = crate::proxy::model_specs::is_gemini_under_v3(target_model);
        if is_under_v3 {
            // Gemini < 3 非思考模型严禁注入 thinkingConfig
            if let Some(obj) = generation_config.as_object_mut() {
                obj.remove("thinkingConfig");
                obj.remove("thinking_config");
            }
            return None;
        }

        let tb_config = crate::proxy::config::get_thinking_budget_config();

        // ════════════════════════════════════════════════════════════════════
        // 模式分流 1: 客户端自填控制模式（Client Direct Control）
        // 遵循最高指令：思考开关 > 思考等级 > 思考预算，缺省默认开，上游自适应
        // ════════════════════════════════════════════════════════════════════
        if tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client {
            match client_switch {
                ClientThinkingSwitch::Disabled => {
                    // 1. 思考开关显式关闭（一票否决）：彻底不带 thinkingConfig，不填预算
                    if let Some(obj) = generation_config.as_object_mut() {
                        obj.remove("thinkingConfig");
                        obj.remove("thinking_config");
                    }
                    return None;
                }
                ClientThinkingSwitch::Enabled | ClientThinkingSwitch::Default => {
                    // 2. 允许思考（显式开 OR 缺省默认开）
                    // 2.1 预算显式 (> 0)：忠实透传预算数字，绝不脑补等级（防止 Google 400 双字段冲突），必须带上 includeThoughts: true
                    if let Some(budget) = client_budget.filter(|&b| b > 0) {
                        generation_config["thinkingConfig"] = json!({
                            "includeThoughts": true,
                            "thinkingBudget": budget
                        });
                        // 确保 maxOutputTokens 大于 thinkingBudget 避免 400
                        let min_overhead = 8192;
                        let current_max = generation_config
                            .get("maxOutputTokens")
                            .and_then(Value::as_i64)
                            .unwrap_or(65536);
                        if current_max <= budget as i64 {
                            generation_config["maxOutputTokens"] =
                                json!(budget as i64 + min_overhead);
                        }
                        return Some(budget as i64);
                    }

                    // 2.2 预算缺省，但客户端携带了思考等级（包括 low / medium / high 以及任何客户自定义的思考等级）：
                    // 核心铁律：坚决不填预算！忠实透传等级，并且必须带上 includeThoughts: true 核心开关！
                    if let Some(raw_effort) = client_effort.map(str::trim).filter(|s| !s.is_empty())
                    {
                        let lower_effort = raw_effort.to_lowercase();
                        if lower_effort != "default"
                            && lower_effort != "none"
                            && lower_effort != "off"
                            && lower_effort != "disabled"
                        {
                            let final_level = match lower_effort.as_str() {
                                "low" | "extra-low" | "min" | "minimal" => "LOW".to_string(),
                                "medium" | "normal" | "standard" => {
                                    if target_model.to_lowercase().contains("pro") {
                                        "HIGH".to_string()
                                    } else {
                                        "MEDIUM".to_string()
                                    }
                                }
                                "high" | "xhigh" | "max" | "extreme" => "HIGH".to_string(),
                                // 客户带了任何自定义等级，直接忠实透传，绝不硬编码限制！
                                _ => raw_effort.to_uppercase(),
                            };
                            generation_config["thinkingConfig"] = json!({
                                "includeThoughts": true,
                                "thinkingLevel": final_level
                            });
                            return None;
                        }
                    }

                    // 2.3 等级与预算均缺省（或 default）：全部预算不传递，默认上游处理（上游自适应）
                    // ★ 绝对不塞 4000/Medium 预算，仅带 includeThoughts: true
                    generation_config["thinkingConfig"] = json!({
                        "includeThoughts": true
                    });
                    return None;
                }
            }
        }

        // ════════════════════════════════════════════════════════════════════
        // 模式分流 2: 网关权威控制模式（Gateway Authority，99% 用户）
        // 100% 保持原有权威逻辑不变：档位锁死、flash_low/med/high 映射、防 429 注入
        // ════════════════════════════════════════════════════════════════════
        let resolved_budget = crate::proxy::model_specs::resolve_custom_budget(
            target_model,
            client_effort,
            client_budget,
            &tb_config,
            token,
        );

        let is_tiered = crate::proxy::model_specs::is_tiered_flash_model(target_model)
            || target_model.to_lowercase().contains("tiered");

        let mut tc = json!({
            "includeThoughts": true
        });

        if let Some(budget) = resolved_budget {
            if budget == 0 {
                tc = json!({
                    "thinkingBudget": 0
                });
            } else {
                tc["thinkingBudget"] = json!(budget);

                // 确保 maxOutputTokens 大于 thinkingBudget 避免 400
                let min_overhead = 8192;
                let current_max = generation_config
                    .get("maxOutputTokens")
                    .and_then(Value::as_i64)
                    .unwrap_or(65536);
                if current_max <= budget {
                    generation_config["maxOutputTokens"] = json!(budget + min_overhead);
                }
            }
        } else if is_tiered {
            // Tiered 模型未指定具体数字 budget 时，纯自适应模式：不注入 thinkingBudget
            tc = json!({
                "includeThoughts": true
            });
        }

        generation_config["thinkingConfig"] = tc;

        // 终审上限保护
        let target_lower = target_model.to_lowercase();
        let safe_limit = if target_lower.contains("claude") {
            64000
        } else if target_lower.contains("pro") {
            65535
        } else {
            65536
        };
        if let Some(val) = generation_config["maxOutputTokens"].as_i64() {
            if val > safe_limit {
                generation_config["maxOutputTokens"] = json!(safe_limit);
            }
        }

        resolved_budget
    }
}
