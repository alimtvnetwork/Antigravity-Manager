use super::*;

pub(crate) struct ThinkingHint {
    pub(crate) budget_tokens: Option<u32>,
    pub(crate) level: Option<String>,
}

/// Extract thinking hints from raw request JSON (OpenCode variants compatibility)
/// Checks multiple possible paths for budget and level configuration
pub(crate) fn extract_thinking_hint(body: &Value) -> ThinkingHint {
    let mut hint = ThinkingHint {
        budget_tokens: None,
        level: None,
    };

    // Try to extract budget_tokens from various paths
    // Priority: thinking.budget_tokens > thinking.budgetTokens > thinking.max_tokens > thinking.budget > thinkingConfig.thinkingBudget > reasoning.max_tokens
    if let Some(budget) = body
        .get("thinking")
        .and_then(|t| {
            t.get("budget_tokens")
                .or_else(|| t.get("budgetTokens"))
                .or_else(|| t.get("max_tokens"))
                .or_else(|| t.get("maxTokens"))
                .or_else(|| t.get("budget"))
        })
        .and_then(|b| b.as_u64())
    {
        hint.budget_tokens = Some(budget as u32);
    } else if let Some(budget) = body
        .get("thinkingConfig")
        .and_then(|t| {
            t.get("thinkingBudget")
                .or_else(|| t.get("thinking_budget"))
                .or_else(|| t.get("budget_tokens"))
                .or_else(|| t.get("budgetTokens"))
        })
        .and_then(|b| b.as_u64())
    {
        hint.budget_tokens = Some(budget as u32);
    } else if let Some(budget) = body
        .get("reasoning")
        .and_then(|r| {
            r.get("max_tokens")
                .or_else(|| r.get("maxTokens"))
                .or_else(|| r.get("budget_tokens"))
                .or_else(|| r.get("budgetTokens"))
        })
        .and_then(|b| b.as_u64())
    {
        hint.budget_tokens = Some(budget as u32);
    }

    // Try to extract level from thinkingLevel / reasoning_effort / output_config.effort
    if let Some(level) = body
        .get("thinkingLevel")
        .or_else(|| body.get("thinking_level"))
        .or_else(|| body.get("reasoning_effort"))
        .or_else(|| body.get("reasoningEffort"))
        .or_else(|| body.get("output_config").and_then(|o| o.get("effort")))
        .and_then(|l| l.as_str())
    {
        hint.level = Some(level.to_lowercase());
    }

    hint
}

/// Map thinking level to suggested budget tokens
fn level_to_budget(level: &str, cap: u64) -> u32 {
    let base = match level {
        "minimal" => 1024,
        "low" => 8192,
        "medium" => 16384,
        "high" => 24576,
        _ => 8192, // default to low
    };
    base.min(cap as u32)
}

/// Map thinking level to effort level for output_config
fn level_to_effort(level: &str) -> String {
    match level {
        "minimal" | "low" => "low".to_string(),
        "medium" => "medium".to_string(),
        "high" => "high".to_string(),
        _ => "low".to_string(),
    }
}

/// Apply thinking hints to ClaudeRequest
pub(crate) fn apply_thinking_hints(
    request: &mut crate::proxy::mappers::claude::models::ClaudeRequest,
    hint: &ThinkingHint,
    trace_id: &str,
    budget_cap: u64, // [NEW]
) {
    let mut applied = false;

    // If budget is provided, set/override thinking config
    if let Some(budget) = hint.budget_tokens {
        request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
            type_: "enabled".to_string(),
            pub(crate) budget_tokens: Some(budget),
            effort: None,
        });
        tracing::debug!(
            "[{}] Applied thinking hint: budget_tokens={}",
            trace_id,
            budget
        );
        applied = true;
    }

    // If level is provided
    if let Some(ref level) = hint.level {
        // Map to output_config.effort if not already set
        if request.output_config.is_none() {
            request.output_config = Some(crate::proxy::mappers::claude::models::OutputConfig {
                effort: Some(level_to_effort(level)),
            });
            tracing::debug!("[{}] Applied thinking hint: effort={}", trace_id, level);
            applied = true;
        }

        // If no budget provided but level is, map level to budget
        if hint.budget_tokens.is_none() {
            let budget = level_to_budget(level, budget_cap);
            request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
                type_: "enabled".to_string(),
                pub(crate) budget_tokens: Some(budget),
                effort: None,
            });
            tracing::debug!(
                "[{}] Applied thinking hint: level={} -> budget_tokens={}",
                trace_id,
                level,
                budget
            );
            applied = true;
        }
    }

    if applied {
        tracing::info!("[{}] Applied OpenCode thinking hints to request", trace_id);
    }
}
