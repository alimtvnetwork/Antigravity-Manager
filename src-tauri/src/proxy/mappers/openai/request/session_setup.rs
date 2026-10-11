// Session setup phase (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

pub(crate) struct SetupState {
    pub session_id: String,
    pub message_count: usize,
    pub tools_val: Option<Vec<serde_json::Value>>,
    pub mapped_model_lower: String,
    pub config: crate::proxy::mappers::common_utils::RequestConfig,
    pub is_under_v3: bool,
    pub is_gemini_3_thinking: bool,
    pub is_gemini_flash_thinking: bool,
    pub is_claude_model: bool,
    pub is_claude_thinking: bool,
    pub force_server_thinking: bool,
    pub is_thinking_model: bool,
    pub client_switch: crate::proxy::pipeline::inbound::types::ClientThinkingSwitch,
    pub tb_config: crate::proxy::config::ThinkingBudgetConfig,
    pub is_client_control: bool,
    pub is_client_disabled: bool,
    pub actual_include_thinking: bool,
}

pub(crate) fn phase_setup(
    request: &super::super::models::OpenAIRequest,
    project_id: &str,
    mapped_model: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    routing_session_id: &str,
    is_responses_api: bool,
) -> SetupState {
    let remember_cwd =
        |text: &str| crate::proxy::adapters::apply_patch_preflight::remember_cwd_from_text(text);
    let found_cwd = request.instructions.as_deref().is_some_and(remember_cwd);
    if !found_cwd {
        'messages: for message in &request.messages {
            let Some(content) = &message.content else {
                continue;
            };
            match content {
                OpenAIContent::String(text) => {
                    if remember_cwd(text) {
                        break 'messages;
                    }
                }
                OpenAIContent::Array(blocks) => {
                    for block in blocks {
                        if let OpenAIContentBlock::Text { text } = block {
                            if remember_cwd(text) {
                                break 'messages;
                            }
                        }
                    }
                }
            }
        }
    }

    let session_id = routing_session_id.to_string();
    // ThinkingStore must use the stable tenant-scoped store_key (request.session_id),
    // not the Responses routing / previous_response_id chain. Capture already writes
    // to store_key; hydrating with a different key leaves history as placeholder+sentinel.
    let thinking_store_key = request
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(routing_session_id)
        .to_string();
    let message_count = request.messages.len();
    // 将 OpenAI 工具转为 Value 数组以便探测
    let tools_val = request
        .tools
        .as_ref()
        .map(|list| list.iter().map(|v| v.clone()).collect::<Vec<_>>());

    let mapped_model_lower = mapped_model.to_lowercase();

    // Resolve grounding config
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &request.model,
        &mapped_model_lower,
        &tools_val,
        request.size.as_deref(),       // [NEW] Pass size parameter
        request.quality.as_deref(),    // [NEW] Pass quality parameter
        request.image_size.as_deref(), // [FIX] Pass imageSize parameter
        None,                          // body
    );

    let is_under_v3 = crate::proxy::model_specs::is_gemini_under_v3(mapped_model)
        || crate::proxy::model_specs::is_gemini_under_v3(&request.model);

    // [FIX] 仅当模型名称显式包含 "-thinking" 或 Gemini 3+ 思维模型时才视为 Gemini 思维模型
    let is_gemini_3_thinking = !is_under_v3
        && mapped_model_lower.contains("gemini")
        && (mapped_model_lower.contains("-thinking")
            || crate::proxy::model_specs::is_gemini_v3_or_above(mapped_model)
            || mapped_model_lower.contains("gemini-pro")
            || mapped_model_lower.contains("-pro-agent"))
        && !mapped_model_lower.contains("claude");
    // [FIX #2167] gemini-*-flash 支持 thinking (需为 Gemini 3 及以上版本)
    let is_gemini_flash_thinking = !is_under_v3
        && crate::proxy::model_specs::is_gemini_v3_or_above(mapped_model)
        && mapped_model_lower.contains("gemini")
        && (mapped_model_lower.contains("flash")
            || mapped_model_lower.contains("-flash-")
            || mapped_model_lower.contains("-flash-agent"))
        && !mapped_model_lower.contains("claude");
    // Client thinking flags/budgets are ignored for enablement and fill.
    // Server authority: model-id heuristics + ThinkingStore hydrate/finalize only.
    let _user_enabled_thinking = request
        .thinking
        .as_ref()
        .map(|t| t.thinking_type.as_deref() == Some("enabled"))
        .unwrap_or(false);
    let _user_thinking_budget = request
        .thinking
        .as_ref()
        .and_then(|t| t.budget_tokens)
        .or_else(|| request.reasoning.as_ref().and_then(|r| r.max_tokens));

    let is_claude_model = mapped_model_lower.contains("claude");
    let is_claude_thinking = mapped_model_lower.ends_with("-thinking")
        || (is_claude_model && mapped_model_lower.contains("thinking"));
    let force_server_thinking = !is_under_v3
        && crate::proxy::thinking_store::any_model_forces_server_thinking(&[
            request.model.as_str(),
            mapped_model,
        ]);
    let is_thinking_model = is_gemini_3_thinking
        || is_claude_thinking
        || is_gemini_flash_thinking
        || force_server_thinking;

    // [NEW] 决定是否开启 Thinking 功能（纯服务端权威 vs 客户端直接控制）:
    // 网关控制模式下：仅按映射后的模型 ID / 强制思考启发式开启，忽略客户端 thinking.type / budget / effort。
    // 客户端直接控制模式下：若客户端显式关闭思考（type: disabled 或 budget: 0 或 effort: none），尊重客户端设置。
    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        request
            .thinking
            .as_ref()
            .and_then(|t| t.thinking_type.as_deref()),
        request
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens.map(|b| b as u64))
            .or_else(|| {
                request
                    .reasoning
                    .as_ref()
                    .and_then(|r| r.max_tokens.map(|b| b as u64))
            }),
        request
            .reasoning_effort
            .as_deref()
            .or_else(|| request.reasoning.as_ref().and_then(|r| r.effort.as_deref()))
            .or_else(|| request.thinking.as_ref().and_then(|t| t.effort.as_deref())),
    );

    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;

    let is_client_disabled = is_client_control && client_switch.is_disabled();

    let actual_include_thinking = if is_client_disabled {
        false
    } else {
        !is_under_v3 && (is_thinking_model || force_server_thinking || is_client_control)
    };

    if _user_enabled_thinking || _user_thinking_budget.is_some() {
        tracing::debug!(
            "[OpenAI-Thinking] Ignoring client thinking enable/budget (enabled={}, budget={:?}); server model heuristics decide fill",
            _user_enabled_thinking,
            _user_thinking_budget
        );
    }

    tracing::debug!(
        "[Debug] OpenAI Request: original='{}', mapped='{}', type='{}', has_image_config={}",
        request.model,
        mapped_model,
        config.request_type,
        config.image_config.is_some()
    );

    SetupState {
        session_id,
        message_count,
        tools_val,
        mapped_model_lower,
        config,
        is_under_v3,
        is_gemini_3_thinking,
        is_gemini_flash_thinking,
        is_claude_model,
        is_claude_thinking,
        force_server_thinking,
        is_thinking_model,
        client_switch,
        tb_config,
        is_client_control,
        is_client_disabled,
        actual_include_thinking,
    }
}
