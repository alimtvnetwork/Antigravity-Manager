use super::*;

fn apply_variant(
    request: &mut ClaudeRequest,
    effort_tier: Option<crate::proxy::common::variant_mapping::VariantTier>,
    client_budget: Option<u32>,
) -> Option<crate::proxy::common::variant_mapping::RealModelSpec> {
    let spec = crate::proxy::common::variant_mapping::resolve_with_tier(
        &request.model,
        effort_tier,
        client_budget,
    )?;

    request.model = spec.id.to_string();
    if spec.thinking_budget == 0 {
        request.thinking = None;
        request.tools = None;
    } else {
        request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(spec.effective_thinking_budget(client_budget)),
            effort: None,
        });
    }
    request.output_config = None;
    request.max_tokens = Some(spec.max_output_tokens);

    Some(spec)
}
