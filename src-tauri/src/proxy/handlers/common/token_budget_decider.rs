//! Token Budget Decider & Thinking Allocation Controller (`Imp-617`)
//!
//! Enforces AGENTS.md invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment..."
//!
//! Allocates token budgets across `max_output_tokens`, thinking/reasoning budgets,
//! and remaining prompt context headroom before forwarding upstream.

use serde::{Deserialize, Serialize};

/// Standard default max output tokens when not specified by client.
pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 8_192;

/// Standard minimum output headroom allocated above thinking budget to avoid upstream 400 Bad Request.
/// Upstream providers (Gemini/Claude) require `maxOutputTokens > thinkingBudget`.
pub const MIN_OUTPUT_OVERHEAD_ABOVE_THINKING: u32 = 8_192;

/// Hard ceiling for max output tokens to prevent provider rejection (e.g. Gemini 65,536 limit).
pub const MAX_OUTPUT_TOKENS_CEILING: u32 = 65_536;

/// Model thinking / reasoning token capability classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelThinkingCapability {
    /// Model strictly requires a thinking budget (e.g., Gemini 2.0/2.5 Flash thinking, Claude 3.7 with reasoning).
    RequiresThinkingBudget {
        min_budget: u32,
        max_budget: u32,
        default_budget: u32,
    },
    /// Model does not support thinking/reasoning tokens (e.g., Gemini 1.5, GPT-4o, Claude 3.5 Sonnet).
    /// Any incoming thinking budget must be filtered out to prevent upstream 400 Bad Request errors.
    ThinkingNotSupported,
    /// Model supports optional thinking (e.g., flexible reasoning models).
    /// If requested, it is preserved/clamped; if omitted, no thinking budget is forced.
    OptionalThinking,
}

/// Request parameters for evaluating token budget allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBudgetRequest {
    pub model_name: String,
    pub prompt_tokens: u32,
    pub max_context_window: u32,
    pub requested_max_output_tokens: Option<u32>,
    pub requested_thinking_budget: Option<u32>,
}

impl TokenBudgetRequest {
    /// Constructs a basic token budget request with prompt tokens and context window.
    pub fn new(model_name: impl Into<String>, prompt_tokens: u32, max_context_window: u32) -> Self {
        Self {
            model_name: model_name.into(),
            prompt_tokens,
            max_context_window,
            requested_max_output_tokens: None,
            requested_thinking_budget: None,
        }
    }

    /// Sets the requested max output tokens.
    pub fn with_max_output_tokens(mut self, tokens: u32) -> Self {
        self.requested_max_output_tokens = Some(tokens);
        self
    }

    /// Sets the requested thinking budget.
    pub fn with_thinking_budget(mut self, budget: u32) -> Self {
        self.requested_thinking_budget = Some(budget);
        self
    }
}

/// Outcome of the token budget decision process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAllocation {
    pub effective_max_output_tokens: u32,
    pub effective_thinking_budget: Option<u32>,
    pub remaining_headroom_tokens: u32,
    pub was_thinking_backfilled: bool,
    pub was_thinking_filtered: bool,
}

/// Summary classification of the thinking budget decision for audit and metrics logging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingBudgetDecision {
    Backfilled(u32),
    Filtered,
    Passthrough(u32),
    Clamped { requested: u32, effective: u32 },
    Disabled,
    NotApplicable,
}

impl TokenAllocation {
    /// Categorizes the outcome of the thinking budget decision for telemetry and diagnostics.
    pub fn decision_summary(&self) -> ThinkingBudgetDecision {
        if self.was_thinking_backfilled {
            if let Some(b) = self.effective_thinking_budget {
                ThinkingBudgetDecision::Backfilled(b)
            } else {
                ThinkingBudgetDecision::NotApplicable
            }
        } else if self.was_thinking_filtered {
            ThinkingBudgetDecision::Filtered
        } else if let Some(b) = self.effective_thinking_budget {
            ThinkingBudgetDecision::Passthrough(b)
        } else {
            ThinkingBudgetDecision::Disabled
        }
    }
}

/// Violations preventing safe upstream token allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetViolation {
    /// Prompt tokens alone exceed or exhaust the model's supported context window.
    ContextWindowExceeded { prompt: u32, context_window: u32 },
    /// Output tokens budget resolved to zero (explicit zero requested or no headroom).
    ZeroOutputTokensBudget,
    /// Model thinking capability configuration is invalid (e.g. min_budget > max_budget or default out of bounds).
    InvalidThinkingBudgetRange,
}

impl std::fmt::Display for BudgetViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextWindowExceeded {
                prompt,
                context_window,
            } => {
                write!(
                    f,
                    "Prompt token count ({prompt}) exceeds or exhausts context window ({context_window})"
                )
            }
            Self::ZeroOutputTokensBudget => {
                write!(f, "Calculated effective output tokens budget is zero")
            }
            Self::InvalidThinkingBudgetRange => {
                write!(
                    f,
                    "Thinking budget configuration is invalid: min_budget exceeds max_budget or default_budget out of bounds"
                )
            }
        }
    }
}

impl std::error::Error for BudgetViolation {}

/// Resolves model thinking capability based on model name naming patterns.
pub fn resolve_model_thinking_capability(model_name: &str) -> ModelThinkingCapability {
    let lower = model_name.to_lowercase();
    if lower.contains("claude-3-7")
        || lower.contains("gemini-2.0-flash-thinking")
        || lower.contains("thinking")
    {
        ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_024,
            max_budget: 32_768,
            default_budget: 8_192,
        }
    } else if lower.contains("gemini-2.5")
        || lower.contains("gemini-3")
        || lower.contains("o1")
        || lower.contains("o3")
    {
        ModelThinkingCapability::OptionalThinking
    } else {
        ModelThinkingCapability::ThinkingNotSupported
    }
}

/// Evaluates and calculates effective token allocation across output, thinking, and headroom.
pub fn decide_token_budget(
    request: &TokenBudgetRequest,
    capability: &ModelThinkingCapability,
) -> Result<TokenAllocation, BudgetViolation> {
    // 1. Validate capability range integrity
    if let ModelThinkingCapability::RequiresThinkingBudget {
        min_budget,
        max_budget,
        default_budget,
    } = capability
    {
        if min_budget > max_budget {
            return Err(BudgetViolation::InvalidThinkingBudgetRange);
        }
        if default_budget < min_budget || default_budget > max_budget {
            return Err(BudgetViolation::InvalidThinkingBudgetRange);
        }
    }

    // 2. Validate context window vs prompt tokens
    if request.prompt_tokens >= request.max_context_window {
        return Err(BudgetViolation::ContextWindowExceeded {
            prompt: request.prompt_tokens,
            context_window: request.max_context_window,
        });
    }

    // 3. Reject explicit zero requested max output tokens
    if request.requested_max_output_tokens == Some(0) {
        return Err(BudgetViolation::ZeroOutputTokensBudget);
    }

    let available_headroom = request.max_context_window - request.prompt_tokens;

    // 4. Resolve thinking budget based on capability and request
    let mut effective_thinking_budget: Option<u32> = None;
    let mut was_thinking_backfilled = false;
    let mut was_thinking_filtered = false;

    match capability {
        ModelThinkingCapability::ThinkingNotSupported => {
            if request.requested_thinking_budget.is_some() {
                was_thinking_filtered = true;
            }
            effective_thinking_budget = None;
        }
        ModelThinkingCapability::RequiresThinkingBudget {
            min_budget,
            max_budget,
            default_budget,
        } => {
            if available_headroom < *min_budget {
                return Err(BudgetViolation::ContextWindowExceeded {
                    prompt: request.prompt_tokens,
                    context_window: request.max_context_window,
                });
            }

            match request.requested_thinking_budget {
                None => {
                    effective_thinking_budget = Some(*default_budget);
                    was_thinking_backfilled = true;
                }
                Some(0) if *min_budget > 0 => {
                    effective_thinking_budget = Some(*default_budget);
                    was_thinking_backfilled = true;
                }
                Some(budget) => {
                    let clamped = budget.clamp(*min_budget, *max_budget);
                    effective_thinking_budget = Some(clamped);
                }
            }
        }
        ModelThinkingCapability::OptionalThinking => match request.requested_thinking_budget {
            Some(0) => {
                effective_thinking_budget = None;
                was_thinking_filtered = true;
            }
            Some(budget) => {
                effective_thinking_budget = Some(budget);
            }
            None => {
                effective_thinking_budget = None;
            }
        },
    }

    // 5. Determine baseline max output tokens
    let mut base_output = request
        .requested_max_output_tokens
        .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);

    // 6. Enforce API constraint: maxOutputTokens > thinkingBudget
    if let Some(tb) = effective_thinking_budget {
        if base_output <= tb {
            let target_with_overhead = tb
                .saturating_add(MIN_OUTPUT_OVERHEAD_ABOVE_THINKING)
                .min(MAX_OUTPUT_TOKENS_CEILING);
            if target_with_overhead > tb {
                base_output = target_with_overhead;
            } else {
                base_output = tb.saturating_add(1);
            }
        }
    }

    // 7. Clamp effective output tokens to physical context window headroom
    let effective_max_output_tokens = base_output.min(available_headroom);

    if effective_max_output_tokens == 0 {
        return Err(BudgetViolation::ZeroOutputTokensBudget);
    }

    // 8. Reconcile thinking budget with clamped output tokens (output must remain > thinking)
    if let Some(ref mut tb) = effective_thinking_budget {
        if *tb >= effective_max_output_tokens {
            match capability {
                ModelThinkingCapability::RequiresThinkingBudget { min_budget, .. } => {
                    if effective_max_output_tokens <= *min_budget {
                        return Err(BudgetViolation::ContextWindowExceeded {
                            prompt: request.prompt_tokens,
                            context_window: request.max_context_window,
                        });
                    }
                    *tb = effective_max_output_tokens.saturating_sub(1);
                }
                ModelThinkingCapability::OptionalThinking => {
                    if effective_max_output_tokens > 1 {
                        *tb = effective_max_output_tokens.saturating_sub(1);
                    } else {
                        effective_thinking_budget = None;
                        was_thinking_filtered = true;
                    }
                }
                _ => {}
            }
        }
    }

    // 9. Compute remaining context headroom
    let remaining_headroom_tokens = available_headroom.saturating_sub(effective_max_output_tokens);

    Ok(TokenAllocation {
        effective_max_output_tokens,
        effective_thinking_budget,
        remaining_headroom_tokens,
        was_thinking_backfilled,
        was_thinking_filtered,
    })
}

/// Automatically resolves capability from request model name and decides budget.
pub fn decide_token_budget_auto(
    request: &TokenBudgetRequest,
) -> Result<TokenAllocation, BudgetViolation> {
    let capability = resolve_model_thinking_capability(&request.model_name);
    decide_token_budget(request, &capability)
}
