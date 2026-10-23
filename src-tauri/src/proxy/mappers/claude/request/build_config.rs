// Tools and generation config (split from request.rs).
// Claude requested conversion (Claude → Gemini v1internal)
// Corresponds to transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Build Tools
pub(crate) fn build_tools(
    tools: &Option<Vec<Tool>>,
    has_web_search: bool,
    mapped_model: &str,
) -> Result<Option<Value>, String> {
    if let Some(tools_list) = tools {
        let mut function_declarations: Vec<Value> = Vec::new();
        let has_google_search = has_web_search;

        for tool in tools_list {
            let name = tool
                .name
                .as_deref()
                .or(tool.type_.as_deref())
                .unwrap_or("tool");

            let mut input_schema = tool.input_schema.clone().unwrap_or(json!({
                "type": "object",
                "properties": {}
            }));
            crate::proxy::common::json_schema::clean_json_schema(&mut input_schema);
            crate::proxy::mappers::openai::request::enforce_uppercase_types(&mut input_schema);

            function_declarations.push(json!({
                "name": name,
                "description": tool.description,
                "parameters": input_schema
            }));
        }

        let mut tool_list = Vec::new();

        // [Optimization] Gemini 2.0+ and 3.0 series models usually support hybrid tool calling (Function Calling + Google Search)
        // However, since the Google v1internal interface used by the reverse generation is a restricted environment and does not support include_server_side_tool_invocations, a 400 error will be reported for mixed calls.
        // Therefore, under the v1internal architecture, we force not to enable mixed tool calls to avoid 400 errors.
        let supports_mixed_tools = false;

        if !function_declarations.is_empty() {
            // [CACHE] Stable lexicographic sorting by function name to ensure complete protocol tool schema byte consistency
            function_declarations.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });
            let mut func_obj = serde_json::Map::new();
            func_obj.insert(
                "functionDeclarations".to_string(),
                json!(function_declarations),
            );
            tool_list.push(json!(func_obj));

            if has_google_search {
                if supports_mixed_tools {
                    tracing::info!(
                        "[Claude-Request] Enabling MIXED tool calling for {}: Function Calling + Google Search.",
                        mapped_model
                    );
                    let mut search_obj = serde_json::Map::new();
                    search_obj.insert("googleSearch".to_string(), json!({}));
                    tool_list.push(json!(search_obj));
                } else {
                    tracing::info!(
                        "[Claude-Request] Skipping googleSearch injection for {} due to existing function declarations. \
                         Older Gemini models may not support mixed tool types.",
                        mapped_model
                    );
                }
            }
        } else if has_google_search {
            let mut search_obj = serde_json::Map::new();
            search_obj.insert("googleSearch".to_string(), json!({}));
            tool_list.push(json!(search_obj));
        }

        if !tool_list.is_empty() {
            return Ok(Some(json!(tool_list)));
        }
    }

    Ok(None)
}

/// Build Generation Config

/// Build Generation Config
pub(crate) fn build_generation_config(
    claude_req: &ClaudeRequest,
    mapped_model: &str,
    _has_web_search: bool,
    is_thinking_enabled: bool,
    token: Option<&crate::proxy::token_manager::ProxyToken>, // [NEW]
) -> Value {
    let mut config = json!({});

    // Thinking configuration
    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        claude_req.thinking.as_ref().map(|t| t.type_.as_str()),
        claude_req
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens.map(|b| b as u64)),
        claude_req
            .output_config
            .as_ref()
            .and_then(|c| c.effort.as_deref())
            .or_else(|| {
                claude_req
                    .thinking
                    .as_ref()
                    .and_then(|t| t.effort.as_deref())
            }),
    );

    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;
    let is_client_disabled = is_client_control && client_switch.is_disabled();

    let effort = claude_req
        .output_config
        .as_ref()
        .and_then(|c| c.effort.as_ref())
        .or_else(|| claude_req.thinking.as_ref().and_then(|t| t.effort.as_ref()))
        .or_else(|| tb_config.effort.as_ref());

    let client_effort = effort.map(|s| s.as_str());
    let client_budget = claude_req
        .thinking
        .as_ref()
        .and_then(|t| t.budget_tokens.map(|b| b as u64));

    if is_client_disabled {
        crate::proxy::pipeline::InboundThinkingPipeline::configure_inbound_thinking(
            mapped_model,
            &mut config,
            client_switch,
            None,
            None,
            token,
        );
    } else if is_thinking_enabled && !crate::proxy::model_specs::is_gemini_under_v3(mapped_model) {
        let mut thinking_config = json!({"includeThoughts": true});

        let global_mode_is_adaptive = matches!(
            tb_config.mode,
            crate::proxy::config::ThinkingBudgetMode::Adaptive
        );
        let user_is_adaptive = claude_req
            .thinking
            .as_ref()
            .map(|t| t.type_ == "adaptive")
            .unwrap_or(false);
        let should_use_adaptive = (user_is_adaptive || global_mode_is_adaptive)
            && mapped_model.to_lowercase().contains("claude");

        if should_use_adaptive {
            let mapped_level = match effort.map(|e| e.to_lowercase()).as_deref() {
                Some("low") => "LOW",
                Some("medium") => "MEDIUM",
                Some("high") | Some("max") | Some("xhigh") => "HIGH",
                _ => "HIGH",
            };
            tracing::debug!(
                "[Claude-Request] Mapping adaptive mode to thinkingLevel: {} for Claude model",
                mapped_level
            );
            thinking_config["thinkingLevel"] = json!(mapped_level);
            config["thinkingConfig"] = thinking_config;
        } else {
            // Protocol-independent: Thinking budget and thinkingConfig are unified and managed by the inbound pipeline node
            crate::proxy::pipeline::InboundThinkingPipeline::configure_inbound_thinking(
                mapped_model,
                &mut config,
                client_switch,
                client_effort,
                client_budget,
                token,
            );
        }
    }

    // Other parameters
    if let Some(temp) = claude_req.temperature {
        config["temperature"] = json!(temp);
    }
    if let Some(top_p) = claude_req.top_p {
        config["topP"] = json!(top_p);
    } else {
        config["topP"] = json!(1.0); // [CHANGED v4.1.24] Default topP=1.0 to match official client
    }
    if let Some(top_k) = claude_req.top_k {
        config["topK"] = json!(top_k);
    } else {
        config["topK"] = json!(40); // [ADDED v4.1.24] Default topK=40 to match official client
    }

    // web_search forces candidateCount=1
    /*if has_web_search {
        config["candidateCount"] = json!(1);
    }*/

    // max_tokens maps to maxOutputTokens
    // [FIX] No longer defaults to 81920 to prevent non-mental models (such as claude-sonnet-4-6) from reporting 400 Invalid Argument
    let mut final_max_tokens: Option<i64> = claude_req.max_tokens.map(|t| t as i64);

    // [NEW] Make sure maxOutputTokens is greater than thinkingBudget (API strong constraint)
    // [NEW] Make sure maxOutputTokens is greater than thinkingBudget (API strong constraint)
    let model_lower = mapped_model.to_lowercase();
    // Recalculate should_use_adaptive (since the scope defined above is only valid within its if block, or we can assume that the same logic is needed here as well)
    // But for the sake of simplicity and decoupling, we re-read from config here
    let tb_config_chk = crate::proxy::config::get_thinking_budget_config();
    let global_adaptive = matches!(
        tb_config_chk.mode,
        crate::proxy::config::ThinkingBudgetMode::Adaptive
    );
    let req_adaptive = claude_req
        .thinking
        .as_ref()
        .map(|t| t.type_ == "adaptive")
        .unwrap_or(false);

    let is_adaptive_effective = (req_adaptive || global_adaptive) && model_lower.contains("claude");
    // [FIX] Lower default overhead to keep total under 65536
    let final_overhead = if is_adaptive_effective { 64000 } else { 32768 };

    // [FIX #2007] Opus 4.6 Thinking Alignment
    // OpenAI logs show maxOutputTokens = 57344 (24576 + 32768)
    if model_lower.contains("claude-opus-4-6-thinking") && is_thinking_enabled {
        final_max_tokens = Some(57344);
        tracing::debug!("[Opus-Alignment] Enforcing maxOutputTokens 57344 for Opus 4.6");
    }

    if let Some(thinking_config) = config.get("thinkingConfig") {
        if let Some(budget) = thinking_config
            .get("thinkingBudget")
            .and_then(|t| t.as_u64())
        {
            let current = final_max_tokens.unwrap_or(0);
            if current <= budget as i64 {
                // [FIX #1675] Use smaller increments for image models (2048)
                let overhead = if mapped_model.contains("-image") {
                    2048
                } else {
                    8192
                };
                let boosted = (budget + overhead).min(65536); // [FIX] Never exceed hard limit
                final_max_tokens = Some(boosted as i64);
                tracing::info!(
                    "[Generation-Config] Bumping maxOutputTokens to {} due to thinking budget of {}", 
                    boosted, budget
                );
            }
        } else if is_adaptive_effective {
            // [FIX] Adaptive mode (no budget set in thinkingConfig), apply default maxOutputTokens
            if final_max_tokens.is_none() {
                final_max_tokens = Some(final_overhead as i64);
            }
        }
    } else {
        // No thinkingConfig
        if final_max_tokens.is_none() && is_adaptive_effective {
            final_max_tokens = Some(final_overhead as i64);
        }
    }

    if let Some(val) = final_max_tokens {
        // [FIX] Cap maxOutputTokens to safe upper limit (65535 for Pro, 65536 for Flash) to avoid INVALID_ARGUMENT (Cherry Studio sends 128000)
        let safe_limit = if mapped_model.to_lowercase().contains("pro") {
            65535
        } else {
            65536
        };
        if val > safe_limit {
            tracing::warn!(
                "[Generation-Config] Capping maxOutputTokens from {} to {} to prevent 400 Invalid Argument",
                val, safe_limit
            );
            config["maxOutputTokens"] = json!(safe_limit);
        } else {
            config["maxOutputTokens"] = json!(val);
        }
    }

    config
}

/// Recursively remove 'thought' and 'thoughtSignature' fields
/// Used when downgrading thinking (e.g. during 400 retry)
pub fn clean_thinking_fields_recursive(val: &mut Value) {
    match val {
        Value::Object(map) => {
            map.remove("thought");
            map.remove("thoughtSignature");
            map.remove("thought_signature");
            for (_, v) in map.iter_mut() {
                clean_thinking_fields_recursive(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                clean_thinking_fields_recursive(v);
            }
        }
        _ => {}
    }
}

use crate::proxy::mappers::common_utils::is_model_compatible;
