use super::*;

impl InboundThinkingPipeline {
    /// Unified inbound thinking configuration and parameter management (pipeline node):
    /// Ensure the protocol independence of the four major protocols (OpenAI, Claude, Gemini, Codex).
    /// 1. Automatically identify target models (including Tiered adaptive models and named models)
    /// 2. Ignore the client number budget to prevent pollution and accurately capture the client’s suffix-free thinking parameters (low/medium/high)
    /// 3. In the gateway control mode, the thinking parameters only open control rights to the Tiered model, and are mapped to the flash_low, flash_medium, and flash_high of the gateway thinking section respectively.
    /// 4. Assemble and normalize thinkingConfig and maxOutputTokens in generationConfig
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
            // Gemini < 3 Non-thinking models are strictly prohibited from injecting thinkingConfig
            if let Some(obj) = generation_config.as_object_mut() {
                obj.remove("thinkingConfig");
                obj.remove("thinking_config");
            }
            return None;
        }

        let tb_config = crate::proxy::config::get_thinking_budget_config();

        // ════════════════════════════════════════════════════════════════════
        // Mode diversion 1: Client Direct Control mode (Client Direct Control)
        // Follow the highest directive: Thinking switch > Thinking level > Thinking budget, enabled by default, upstream adaptive
        // ════════════════════════════════════════════════════════════════════
        if tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client {
            match client_switch {
                ClientThinkingSwitch::Disabled => {
                    // 1. Turn off the thinking switch explicitly (one vote to veto): no thinkingConfig at all, no budget required
                    if let Some(obj) = generation_config.as_object_mut() {
                        obj.remove("thinkingConfig");
                        obj.remove("thinking_config");
                    }
                    return None;
                }
                ClientThinkingSwitch::Enabled | ClientThinkingSwitch::Default => {
                    // 2. Allow thinking (explicitly enabled or enabled by default)
                    // 2.1 Explicit budget (> 0): Transmit budget figures faithfully and never make up for the level (to prevent Google 400 double-field conflicts). IncludeThoughts: true must be included
                    if let Some(budget) = client_budget.filter(|&b| b > 0) {
                        generation_config["thinkingConfig"] = json!({
                            "includeThoughts": true,
                            "thinkingBudget": budget
                        });
                        // Make sure maxOutputTokens is larger than thinkingBudget to avoid 400
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

                    // 2.2 The budget is default, but the client carries the consideration level (including low / medium / high and any customer-defined consideration level):
                    // The core iron rule: never fill the budget! Faithful transparent transmission level, and must bring includeThoughts: true core switch!
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
                                // Any custom level brought by the customer will be directly and faithfully transmitted without any hard-coded restrictions!
                                _ => raw_effort.to_uppercase(),
                            };
                            generation_config["thinkingConfig"] = json!({
                                "includeThoughts": true,
                                "thinkingLevel": final_level
                            });
                            return None;
                        }
                    }

                    // 2.3 The level and budget are both default (or default): all budgets are not passed, and the default is upstream processing (upstream adaptive)
                    // ★ Absolutely no budget of 4000/Medium, only with includeThoughts: true
                    generation_config["thinkingConfig"] = json!({
                        "includeThoughts": true
                    });
                    return None;
                }
            }
        }

        // ════════════════════════════════════════════════════════════════════
        // Mode diversion 2: Gateway Authority control mode (Gateway Authority, 99% users)
        // 100% keep the original authoritative logic unchanged: gear lock, flash_low/med/high mapping, anti-429 injection
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

                // Make sure maxOutputTokens is larger than thinkingBudget to avoid 400
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
            // When the Tiered model does not specify a specific numerical budget, pure adaptive mode: no thinkingBudget is injected
            tc = json!({
                "includeThoughts": true
            });
        } else {
            // Imp-22: Gemini Thinking Budget Backfilling
            // If thinking budget was omitted by client on a thinking model, backfill default 2048
            let default_budget = 2048i64;
            tc["thinkingBudget"] = json!(default_budget);
            let min_overhead = 8192;
            let current_max = generation_config
                .get("maxOutputTokens")
                .and_then(Value::as_i64)
                .unwrap_or(65536);
            if current_max <= default_budget {
                generation_config["maxOutputTokens"] = json!(default_budget + min_overhead);
            }
        }

        generation_config["thinkingConfig"] = tc;

        // Final review cap protection
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
