//! Thinking Budget Headroom & Deficit Evaluator (`Imp-653`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability..."
//!
//! Ensures `effective_max_output_tokens > effective_thinking_budget` across
//! converted Gemini payloads, preventing upstream 400 Bad Request errors.

use serde::{Deserialize, Serialize};

/// Default minimum token headroom required between max output tokens and thinking budget.
pub const MIN_HEADROOM_TOKENS_DEFAULT: u64 = 1024;

/// Default context token ceiling for total output token generation.
pub const CONTEXT_TOKEN_CEILING_DEFAULT: u64 = 65536;

/// Minimum allowable thinking budget tokens before clamping is rejected.
pub const MIN_THINKING_BUDGET_FLOOR: u64 = 512;

/// Default fallback max output tokens when thinking is disabled and unspecified.
pub const DEFAULT_MAX_OUTPUT_TOKENS: u64 = 4096;

/// Status of token headroom evaluated for a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadroomStatus {
    SufficientHeadroom {
        headroom_tokens: u64,
    },
    InsufficientHeadroom {
        max_output: u64,
        thinking_budget: u64,
        deficit: u64,
    },
    ThinkingDisabled,
}

impl std::fmt::Display for HeadroomStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SufficientHeadroom { headroom_tokens } => {
                write!(f, "Sufficient headroom ({} tokens)", headroom_tokens)
            }
            Self::InsufficientHeadroom {
                max_output,
                thinking_budget,
                deficit,
            } => {
                write!(
                    f,
                    "Insufficient headroom (max_output: {}, thinking_budget: {}, deficit: {})",
                    max_output, thinking_budget, deficit
                )
            }
            Self::ThinkingDisabled => write!(f, "Thinking disabled"),
        }
    }
}

/// Adjustment action taken to guarantee pipeline headroom constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadroomAdjustmentAction {
    NoAdjustmentNeeded,
    ExpandedMaxOutputTokens {
        original_max: u64,
        adjusted_max: u64,
    },
    ClampedThinkingBudget {
        original_budget: u64,
        clamped_budget: u64,
    },
    RejectedDueToExceedingContextCeiling {
        requested_total: u64,
        ceiling: u64,
    },
}

impl std::fmt::Display for HeadroomAdjustmentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdjustmentNeeded => write!(f, "No adjustment needed"),
            Self::ExpandedMaxOutputTokens {
                original_max,
                adjusted_max,
            } => {
                write!(
                    f,
                    "Expanded max output tokens: {} -> {}",
                    original_max, adjusted_max
                )
            }
            Self::ClampedThinkingBudget {
                original_budget,
                clamped_budget,
            } => {
                write!(
                    f,
                    "Clamped thinking budget: {} -> {}",
                    original_budget, clamped_budget
                )
            }
            Self::RejectedDueToExceedingContextCeiling {
                requested_total,
                ceiling,
            } => {
                write!(
                    f,
                    "Rejected due to exceeding ceiling: requested {} > ceiling {}",
                    requested_total, ceiling
                )
            }
        }
    }
}

/// Comprehensive evaluation report for thinking budget and max output headroom.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadroomEvaluationReport {
    pub status: HeadroomStatus,
    pub action: HeadroomAdjustmentAction,
    pub effective_max_output_tokens: u64,
    pub effective_thinking_budget: Option<u64>,
    pub is_valid: bool,
}

impl HeadroomEvaluationReport {
    /// Returns true if thinking is active and non-zero for this evaluation.
    pub fn has_thinking(&self) -> bool {
        self.effective_thinking_budget.map_or(false, |b| b > 0)
    }

    /// Returns true if any parameter adjustment was performed.
    pub fn is_adjusted(&self) -> bool {
        !matches!(self.action, HeadroomAdjustmentAction::NoAdjustmentNeeded)
    }

    /// Returns true if the resulting token budget is valid and acceptable.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Returns a reference to the headroom evaluation status.
    pub fn status(&self) -> &HeadroomStatus {
        &self.status
    }

    /// Returns a reference to the adjustment action taken.
    pub fn action(&self) -> &HeadroomAdjustmentAction {
        &self.action
    }
}

impl Default for HeadroomEvaluationReport {
    fn default() -> Self {
        Self {
            status: HeadroomStatus::ThinkingDisabled,
            action: HeadroomAdjustmentAction::NoAdjustmentNeeded,
            effective_max_output_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
            effective_thinking_budget: None,
            is_valid: true,
        }
    }
}

impl std::fmt::Display for HeadroomEvaluationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HeadroomReport: status='{}', action='{}', max_tokens={}, thinking_budget={:?}, valid={}",
            self.status, self.action, self.effective_max_output_tokens, self.effective_thinking_budget, self.is_valid
        )
    }
}

/// Evaluates thinking budget and max output tokens against headroom constraints.
pub fn evaluate_thinking_budget_headroom(
    max_output: Option<u64>,
    thinking_budget: Option<u64>,
    min_headroom_tokens: u64,
    context_token_ceiling: u64,
) -> HeadroomEvaluationReport {
    match thinking_budget {
        None | Some(0) => evaluate_disabled_thinking(max_output),
        Some(budget) => evaluate_active_thinking(
            max_output,
            budget,
            min_headroom_tokens,
            context_token_ceiling,
        ),
    }
}

/// Evaluates headroom using default standard token limits.
pub fn evaluate_thinking_budget_headroom_default(
    max_output: Option<u64>,
    thinking_budget: Option<u64>,
) -> HeadroomEvaluationReport {
    evaluate_thinking_budget_headroom(
        max_output,
        thinking_budget,
        MIN_HEADROOM_TOKENS_DEFAULT,
        CONTEXT_TOKEN_CEILING_DEFAULT,
    )
}

/// Handles evaluation when thinking is disabled or unconfigured.
fn evaluate_disabled_thinking(max_output: Option<u64>) -> HeadroomEvaluationReport {
    HeadroomEvaluationReport {
        status: HeadroomStatus::ThinkingDisabled,
        action: HeadroomAdjustmentAction::NoAdjustmentNeeded,
        effective_max_output_tokens: max_output.unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS),
        effective_thinking_budget: None,
        is_valid: true,
    }
}

/// Handles evaluation when thinking is active with a positive budget.
fn evaluate_active_thinking(
    max_output: Option<u64>,
    budget: u64,
    min_headroom: u64,
    ceiling: u64,
) -> HeadroomEvaluationReport {
    let requested_max = max_output.unwrap_or(budget.saturating_add(min_headroom));
    let needed_output = budget.saturating_add(min_headroom);

    if requested_max >= needed_output && requested_max <= ceiling {
        build_sufficient_headroom(requested_max, budget)
    } else {
        adjust_insufficient_headroom(requested_max, budget, needed_output, min_headroom, ceiling)
    }
}

/// Constructs report when headroom is already sufficient.
fn build_sufficient_headroom(requested_max: u64, budget: u64) -> HeadroomEvaluationReport {
    let headroom = requested_max.saturating_sub(budget);
    HeadroomEvaluationReport {
        status: HeadroomStatus::SufficientHeadroom {
            headroom_tokens: headroom,
        },
        action: HeadroomAdjustmentAction::NoAdjustmentNeeded,
        effective_max_output_tokens: requested_max,
        effective_thinking_budget: Some(budget),
        is_valid: true,
    }
}

/// Performs expansion, clamping, or rejection for insufficient headroom.
fn adjust_insufficient_headroom(
    requested_max: u64,
    budget: u64,
    needed_output: u64,
    min_headroom: u64,
    ceiling: u64,
) -> HeadroomEvaluationReport {
    let deficit = needed_output.saturating_sub(requested_max).max(1);
    let status = HeadroomStatus::InsufficientHeadroom {
        max_output: requested_max,
        thinking_budget: budget,
        deficit,
    };

    if needed_output <= ceiling {
        HeadroomEvaluationReport {
            status,
            action: HeadroomAdjustmentAction::ExpandedMaxOutputTokens {
                original_max: requested_max,
                adjusted_max: needed_output,
            },
            effective_max_output_tokens: needed_output,
            effective_thinking_budget: Some(budget),
            is_valid: true,
        }
    } else {
        clamp_or_reject_budget(status, requested_max, budget, needed_output, min_headroom, ceiling)
    }
}

/// Clamps thinking budget or rejects if minimum budget floor cannot be satisfied.
fn clamp_or_reject_budget(
    status: HeadroomStatus,
    requested_max: u64,
    budget: u64,
    needed_output: u64,
    min_headroom: u64,
    ceiling: u64,
) -> HeadroomEvaluationReport {
    let clamped_budget = ceiling.saturating_sub(min_headroom);

    if clamped_budget >= MIN_THINKING_BUDGET_FLOOR {
        HeadroomEvaluationReport {
            status,
            action: HeadroomAdjustmentAction::ClampedThinkingBudget {
                original_budget: budget,
                clamped_budget,
            },
            effective_max_output_tokens: ceiling,
            effective_thinking_budget: Some(clamped_budget),
            is_valid: true,
        }
    } else {
        HeadroomEvaluationReport {
            status,
            action: HeadroomAdjustmentAction::RejectedDueToExceedingContextCeiling {
                requested_total: needed_output,
                ceiling,
            },
            effective_max_output_tokens: requested_max,
            effective_thinking_budget: Some(budget),
            is_valid: false,
        }
    }
}
