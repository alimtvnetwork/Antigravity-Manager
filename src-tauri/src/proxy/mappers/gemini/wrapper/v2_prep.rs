// V2 thinking-prep phase (split from wrapper.rs).

pub(crate) struct ThinkingPrep {
    pub lower_model: String,
    pub tb_config: crate::proxy::config::ThinkingBudgetConfig,
    pub is_client_control: bool,
    pub client_budget: Option<i64>,
    pub client_level: Option<String>,
    pub client_switch: crate::proxy::pipeline::inbound::types::ClientThinkingSwitch,
    pub is_client_disabled: bool,
    pub is_under_v3: bool,
    pub force_server_thinking: bool,
    pub is_preview: bool,
    pub should_inject: bool,
    pub has_explicit_thinking: bool,
    pub is_thinking_active: bool,
}

pub(crate) fn phase_thinking_prep(
    final_model_name: &str,
    original_model: &str,
    inner_request: &serde_json::Value,
) -> ThinkingPrep {
    let lower_model = final_model_name.to_lowercase();
    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;

    let client_budget = inner_request
        .get("generationConfig")
        .and_then(|gc| gc.get("thinkingConfig"))
        .and_then(|tc| {
            tc.get("thinkingBudget")
                .or_else(|| tc.get("thinking_budget"))
                .or_else(|| tc.get("budget_tokens"))
                .or_else(|| tc.get("budgetTokens"))
                .or_else(|| tc.get("max_tokens"))
                .or_else(|| tc.get("maxTokens"))
        })
        .and_then(|b| b.as_i64());
    let client_level = inner_request
        .get("generationConfig")
        .and_then(|gc| gc.get("thinkingConfig"))
        .and_then(|tc| {
            tc.get("thinkingLevel")
                .or_else(|| tc.get("thinking_level"))
                .or_else(|| tc.get("reasoning_effort"))
                .or_else(|| tc.get("reasoningEffort"))
                .or_else(|| tc.get("effort"))
        })
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        None,
        client_budget.map(|b| if b <= 0 { 0 } else { b as u64 }),
        client_level,
    );
    let is_client_disabled = is_client_control && client_switch.is_disabled();

    let is_under_v3 = crate::proxy::model_specs::is_gemini_under_v3(final_model_name)
        || crate::proxy::model_specs::is_gemini_under_v3(original_model);
    let force_server_thinking = !is_under_v3
        && crate::proxy::thinking_store::any_model_forces_server_thinking(&[
            final_model_name,
            original_model,
        ]);
    let is_preview = lower_model.contains("preview");
    let should_inject = !is_client_disabled
        && !is_under_v3
        && (force_server_thinking
            || lower_model.contains("thinking")
            || (crate::proxy::model_specs::is_gemini_v3_or_above(final_model_name) && !is_preview));

    let has_explicit_thinking = inner_request
        .get("generationConfig")
        .and_then(|gc| gc.get("thinkingConfig"))
        .and_then(|tc| tc.get("thinkingBudget"))
        .and_then(|b| b.as_i64())
        .map(|b| b > 0)
        .unwrap_or(false);
    let is_thinking_active = should_inject || has_explicit_thinking;

    ThinkingPrep {
        lower_model,
        tb_config,
        is_client_control,
        client_budget,
        client_level,
        client_switch,
        is_client_disabled,
        is_under_v3,
        force_server_thinking,
        is_preview,
        should_inject,
        has_explicit_thinking,
        is_thinking_active,
    }
}
