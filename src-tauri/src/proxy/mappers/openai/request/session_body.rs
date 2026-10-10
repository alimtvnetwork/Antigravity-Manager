// Request body phase (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

use super::session_setup::SetupState;

pub(crate) fn phase_body(
    request: &super::models::OpenAIRequest,
    mapped_model: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    setup: &super::session_setup::SetupState,
    contents: &[serde_json::Value],
) -> (serde_json::Value, serde_json::Value) {
    // 3. 构建请求体

    let mut gen_config = json!({
        // [CHANGED v4.1.24] Default topP from 0.95 → 1.0 to match native behavior
        "topP": request.top_p.unwrap_or(1.0),
        // [ADDED v4.1.24] topK=40 aligns with official client generationConfig
        "topK": 40,
    });
    if let Some(temp) = request.temperature {
        gen_config["temperature"] = json!(temp);
    }

    // [FIX] 移除旧的硬编码限额，改为动态查询 (v4.1.29)
    if let Some(max_tokens) = request.max_tokens {
        gen_config["maxOutputTokens"] = json!(max_tokens);
    } else {
        // 使用动态优先的规格限额
        let limit = model_specs::get_max_output_tokens(mapped_model, token);
        gen_config["maxOutputTokens"] = json!(limit);
    }

    // [NEW] 支持多候选结果数量 (n -> candidateCount)
    if let Some(n) = request.n {
        gen_config["candidateCount"] = json!(n);
    }

    if let Some(presence_penalty) = request.presence_penalty {
        gen_config["presencePenalty"] = json!(presence_penalty);
    }
    if let Some(frequency_penalty) = request.frequency_penalty {
        gen_config["frequencyPenalty"] = json!(frequency_penalty);
    }
    if let Some(seed) = request.seed {
        gen_config["seed"] = json!(seed);
    }

    // 为 thinking 模型注入 thinkingConfig (使用流水线统一配置治理)
    if setup.is_client_disabled {
        tracing::debug!(
            "[OpenAI-Request] Client direct control disabled thinking: removing thinkingConfig for {}",
            mapped_model
        );
        crate::proxy::pipeline::InboundThinkingPipeline::configure_inbound_thinking(
            mapped_model,
            &mut gen_config,
            setup.client_switch,
            None,
            None,
            token,
        );
    } else if setup.actual_include_thinking {
        // [RESOLVE #1694] Check image thinking mode
        let image_thinking_mode = crate::proxy::config::get_image_thinking_mode();
        // Only disable if mode is explicitly "disabled" AND it's an image generation request
        let is_image_gen_disabled =
            setup.config.request_type == "image_gen" && image_thinking_mode == "disabled";

        if is_image_gen_disabled {
            tracing::debug!("[OpenAI-Request] Image thinking mode disabled: enforcing includeThoughts=false for {}", mapped_model);
            gen_config["thinkingConfig"] = json!({
                "includeThoughts": false
            });
        } else {
            // [CONFIGURABLE] 思考预算与思考配置：全协议统一由 InboundThinkingPipeline 流水线节点权威解析与治理
            let client_effort = request
                .reasoning_effort
                .as_deref()
                .or_else(|| request.reasoning.as_ref().and_then(|r| r.effort.as_deref()))
                .or_else(|| request.thinking.as_ref().and_then(|t| t.effort.as_deref()));

            let client_budget = request
                .thinking
                .as_ref()
                .and_then(|t| t.budget_tokens.map(|b| b as u64))
                .or_else(|| {
                    request
                        .reasoning
                        .as_ref()
                        .and_then(|r| r.max_tokens.map(|b| b as u64))
                });

            let resolved_budget =
                crate::proxy::pipeline::InboundThinkingPipeline::configure_inbound_thinking(
                    mapped_model,
                    &mut gen_config,
                    setup.client_switch,
                    client_effort,
                    client_budget,
                    token,
                );

            if let Some(final_budget) = resolved_budget {
                // [CRITICAL] 思维模型的 maxOutputTokens 必须大于 thinkingBudget
                // [FIX #1675] 针对图像模型使用更保守的 max_tokens 增量，避免触发 128k 限制
                let overhead = if setup.config.request_type == "image_gen" {
                    2048
                } else {
                    32768
                };
                let min_overhead = if setup.config.request_type == "image_gen" {
                    1024
                } else {
                    8192
                };

                if setup
                    .mapped_model_lower
                    .contains("claude-opus-4-6-thinking")
                {
                    gen_config["maxOutputTokens"] = json!(57344);
                    tracing::debug!(
                        "[Opus-Alignment] Enforcing maxOutputTokens 57344 for Opus 4.6 (OpenAI)"
                    );
                } else if let Some(max_tokens) = request.max_tokens {
                    if (max_tokens as i64) <= final_budget {
                        gen_config["maxOutputTokens"] = json!(final_budget + min_overhead);
                    }
                } else {
                    // [FIX #1592] Use a more conservative default to avoid 400 error on 128k context models
                    gen_config["maxOutputTokens"] = json!(final_budget + overhead);
                }

                let new_max = gen_config["maxOutputTokens"].as_i64().unwrap_or(0);
                tracing::debug!(
                    "[OpenAI-Request] Adjusted maxOutputTokens to {} for thinking model (budget={})",
                    new_max,
                    final_budget
                );
            }

            tracing::debug!(
                "[OpenAI-Request] Configured thinkingConfig for model {}: {:?} (source={:?})",
                mapped_model,
                gen_config["thinkingConfig"],
                setup.tb_config.control_source
            );
        }
    }

    // Tiered Flash models: if resolved_budget was None (default) in gateway authority, ensure only includeThoughts is set
    if !setup.is_client_control
        && is_tiered_flash_model(mapped_model)
        && gen_config["thinkingConfig"].get("thinkingBudget").is_none()
    {
        gen_config["thinkingConfig"] = json!({ "includeThoughts": true });
    }

    // [FIX] Cap maxOutputTokens to prevent 400 Invalid Argument
    if let Some(val) = gen_config["maxOutputTokens"].as_i64() {
        let safe_limit = if setup.mapped_model_lower.contains("claude") {
            64000
        } else if setup.mapped_model_lower.contains("pro") {
            65535
        } else {
            65536
        };
        if val > safe_limit {
            tracing::warn!(
                "[Generation-Config] Capping maxOutputTokens from {} to {} to prevent 400 Invalid Argument",
                val, safe_limit
            );
            gen_config["maxOutputTokens"] = json!(safe_limit);
        }
    }

    if let Some(stop) = &request.stop {
        if !setup
            .mapped_model_lower
            .contains("claude-opus-4-6-thinking")
        {
            if stop.is_string() {
                gen_config["stopSequences"] = json!([stop]);
            } else if stop.is_array() {
                gen_config["stopSequences"] = stop.clone();
            }
        } else {
            tracing::debug!(
                "[Opus-Alignment] Skipping stopSequences for Opus 4.6 to match OpenAI protocol"
            );
        }
    }

    if let Some(fmt) = &request.response_format {
        if fmt.r#type == "json_object" {
            gen_config["responseMimeType"] = json!("application/json");
        } else if fmt.r#type == "json_schema" {
            gen_config["responseMimeType"] = json!("application/json");
            if let Some(js) = &fmt.json_schema {
                if let Some(mut schema) = js.schema.clone() {
                    crate::proxy::common::json_schema::clean_response_schema(&mut schema);
                    gen_config["responseSchema"] = schema;
                }
            }
        }
    }

    // [CACHE] inner_request 先创建为空的 Map，后续按稳定顺序填充
    let mut inner_request = json!({});
    // 先放 contents（后续会被 reordered_request 覆盖到后面）
    inner_request["contents"] = json!(contents);
    inner_request["generationConfig"] = gen_config;
    inner_request["safetySettings"] = json!([
        { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
    ]);

    // [PIPELINE] 统一清洗提示词与风控伪 Header（含 [undefined] 深度清理，见 PromptSanitizer）
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
        &mut inner_request,
    );

    (inner_request, gen_config)
}
