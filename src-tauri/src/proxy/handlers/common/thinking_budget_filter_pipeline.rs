//! Thinking Budget Filter Pipeline Guard (`Imp-203`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability, and the sanitization
//! of risky prompts and request headers."
//!
//! Evaluates whether the requested model is thinking-capable and enforces
//! budget retention, backfilling with system defaults, or stripping incompatible budgets.

use serde::{Deserialize, Serialize};

/// Recognized wildcard patterns identifying AI models capable of thinking/reasoning.
pub const THINKING_MODEL_PATTERNS: &[&str] = &[
    "*thinking*",
    "*reasoning*",
    "*o1*",
    "*o3*",
    "claude-3-7-sonnet*",
    "*claude-3-7-sonnet*",
];

/// Filtering decision made for the requested model's thinking budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingFilterDecision {
    ThinkingStripped {
        model_pattern: String,
        original_budget: u32,
        reason: String,
    },
    ThinkingBackfilled {
        model_pattern: String,
        backfilled_budget: u32,
    },
    ThinkingRetained {
        model_pattern: String,
        budget: u32,
    },
    NoThinkingConfigured,
}

impl ThinkingFilterDecision {
    /// Returns true if thinking budget was stripped due to model incapability.
    pub fn is_stripped(&self) -> bool {
        matches!(self, Self::ThinkingStripped { .. })
    }

    /// Returns true if default thinking budget was backfilled.
    pub fn is_backfilled(&self) -> bool {
        matches!(self, Self::ThinkingBackfilled { .. })
    }

    /// Returns true if requested thinking budget was retained.
    pub fn is_retained(&self) -> bool {
        matches!(self, Self::ThinkingRetained { .. })
    }

    /// Returns true if no thinking budget is configured or thinking was disabled.
    pub fn is_no_thinking(&self) -> bool {
        matches!(self, Self::NoThinkingConfigured)
    }
}

impl std::fmt::Display for ThinkingFilterDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThinkingStripped {
                model_pattern,
                original_budget,
                reason,
            } => write!(
                f,
                "Thinking stripped for {}: budget {} removed ({})",
                model_pattern, original_budget, reason
            ),
            Self::ThinkingBackfilled {
                model_pattern,
                backfilled_budget,
            } => write!(
                f,
                "Thinking backfilled for {}: budget set to {}",
                model_pattern, backfilled_budget
            ),
            Self::ThinkingRetained {
                model_pattern,
                budget,
            } => write!(
                f,
                "Thinking retained for {}: budget {}",
                model_pattern, budget
            ),
            Self::NoThinkingConfigured => write!(f, "No thinking configured"),
        }
    }
}

/// Comprehensive report produced by the pipeline thinking budget filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineThinkingReport {
    pub decision: ThinkingFilterDecision,
    pub effective_budget: Option<u32>,
    pub is_thinking_active: bool,
}

impl PipelineThinkingReport {
    /// Returns true if thinking is active for this request.
    pub fn is_active(&self) -> bool {
        self.is_thinking_active
    }

    /// Returns the effective thinking budget, if active.
    pub fn effective_budget(&self) -> Option<u32> {
        self.effective_budget
    }

    /// Returns a reference to the filtering decision.
    pub fn decision(&self) -> &ThinkingFilterDecision {
        &self.decision
    }
}

impl std::fmt::Display for PipelineThinkingReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PipelineThinkingReport: decision='{}', active={}, budget={:?}",
            self.decision, self.is_thinking_active, self.effective_budget
        )
    }
}

/// Returns true if the model matches recognized wildcard patterns for thinking/reasoning.
pub fn is_model_thinking_capable(model_name: &str) -> bool {
    let lower = model_name.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    THINKING_MODEL_PATTERNS
        .iter()
        .any(|pat| match_wildcard(pat, &lower))
}

/// Filters or backfills the thinking budget for a given model per pipeline invariants.
pub fn filter_and_backfill_thinking_budget(
    model_name: &str,
    requested_budget: Option<u32>,
    default_budget: u32,
) -> PipelineThinkingReport {
    if !is_model_thinking_capable(model_name) {
        filter_incapable_model_budget(model_name, requested_budget)
    } else {
        process_capable_model_budget(model_name, requested_budget, default_budget)
    }
}

/// Handles budget filtering for models that do not support thinking/reasoning.
fn filter_incapable_model_budget(
    model_name: &str,
    requested_budget: Option<u32>,
) -> PipelineThinkingReport {
    match requested_budget {
        Some(budget) if budget > 0 => PipelineThinkingReport {
            decision: ThinkingFilterDecision::ThinkingStripped {
                model_pattern: model_name.to_string(),
                original_budget: budget,
                reason: "Model does not support thinking budget".to_string(),
            },
            effective_budget: None,
            is_thinking_active: false,
        },
        _ => PipelineThinkingReport {
            decision: ThinkingFilterDecision::NoThinkingConfigured,
            effective_budget: None,
            is_thinking_active: false,
        },
    }
}

/// Handles budget retention or backfilling for thinking-capable models.
fn process_capable_model_budget(
    model_name: &str,
    requested_budget: Option<u32>,
    default_budget: u32,
) -> PipelineThinkingReport {
    match requested_budget {
        Some(budget) => resolve_requested_budget(model_name, budget),
        None => resolve_backfilled_budget(model_name, default_budget),
    }
}

/// Resolves an explicitly requested thinking budget for a capable model.
fn resolve_requested_budget(model_name: &str, budget: u32) -> PipelineThinkingReport {
    if budget > 0 {
        PipelineThinkingReport {
            decision: ThinkingFilterDecision::ThinkingRetained {
                model_pattern: model_name.to_string(),
                budget,
            },
            effective_budget: Some(budget),
            is_thinking_active: true,
        }
    } else {
        PipelineThinkingReport {
            decision: ThinkingFilterDecision::NoThinkingConfigured,
            effective_budget: None,
            is_thinking_active: false,
        }
    }
}

/// Resolves a backfilled default thinking budget for a capable model.
fn resolve_backfilled_budget(model_name: &str, default_budget: u32) -> PipelineThinkingReport {
    PipelineThinkingReport {
        decision: ThinkingFilterDecision::ThinkingBackfilled {
            model_pattern: model_name.to_string(),
            backfilled_budget: default_budget,
        },
        effective_budget: Some(default_budget),
        is_thinking_active: true,
    }
}

/// Matches a candidate string against a wildcard glob pattern supporting `*` and `?`.
pub fn match_wildcard(pattern: &str, candidate: &str) -> bool {
    let p_chars: Vec<char> = pattern.chars().collect();
    let c_chars: Vec<char> = candidate.chars().collect();

    let mut p_idx = 0;
    let mut c_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while c_idx < c_chars.len() {
        if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == c_chars[c_idx]) {
            p_idx += 1;
            c_idx += 1;
        } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = c_idx;
            p_idx += 1;
        } else if let Some(s_idx) = star_idx {
            p_idx = s_idx + 1;
            match_idx += 1;
            c_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == p_chars.len()
}
