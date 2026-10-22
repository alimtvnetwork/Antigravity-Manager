//! Thinking Budget Clamping Backend for Reverse Proxy Gateway.
//! Normalizes and enforces bounds on runtime model reasoning tokens (0..=32,768).

use serde::{Deserialize, Serialize};

pub const MIN_THINKING_BUDGET: u32 = 0;
pub const MAX_THINKING_BUDGET: u32 = 32_768;
pub const DEFAULT_THINKING_BUDGET: u32 = 8_192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudgetConfig {
    pub min_tokens: u32,
    pub max_tokens: u32,
    pub default_budget: u32,
    pub is_adaptive_clamp_enabled: bool,
}

/// Returns standard configuration metadata for the thinking budget control widget.
pub fn get_thinking_budget_config() -> ThinkingBudgetConfig {
    ThinkingBudgetConfig {
        min_tokens: MIN_THINKING_BUDGET,
        max_tokens: MAX_THINKING_BUDGET,
        default_budget: DEFAULT_THINKING_BUDGET,
        is_adaptive_clamp_enabled: true,
    }
}

/// Clamps user-requested thinking budget to safe operational limits.
pub fn clamp_thinking_budget(requested: Option<u32>, user_limit: Option<u32>) -> u32 {
    let cap = user_limit
        .unwrap_or(MAX_THINKING_BUDGET)
        .clamp(MIN_THINKING_BUDGET, MAX_THINKING_BUDGET);

    match requested {
        Some(req) => req.clamp(MIN_THINKING_BUDGET, cap),
        None => DEFAULT_THINKING_BUDGET.min(cap),
    }
}
