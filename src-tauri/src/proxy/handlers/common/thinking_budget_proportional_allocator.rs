//! Proportional Thinking Budget Allocator & Context Tier Profiler (`Imp-658`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariants:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including
//! thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability..."
//! "Prioritize wildcard patterns (e.g., `gemini-*-flash-*`) to anticipate future model releases rather than exact string matches."
//!
//! Allocates proportional thinking budget ratios (default 75% thinking budget, 25% output tokens headroom)
//! bounded by model context tier ceilings, safely clamping requested budgets to prevent
//! context overflow or token starvation.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard default ratio of max output tokens allocated to thinking / reasoning (75%).
pub const DEFAULT_PROPORTIONAL_THINKING_RATIO: f64 = 0.75;

/// Standard default ratio of max output tokens reserved for final visible output headroom (25%).
pub const DEFAULT_OUTPUT_HEADROOM_RATIO: f64 = 0.25;

/// Minimum headroom tokens required above thinking budget to avoid upstream 400 Bad Request.
pub const MIN_HEADROOM_TOKENS_FLOOR: u32 = 1_024;

/// Standard 8k context window tier limit.
pub const STANDARD_8K_CONTEXT_WINDOW: u32 = 8_192;
/// Standard 8k tier max output ceiling.
pub const STANDARD_8K_MAX_OUTPUT_CEILING: u32 = 4_096;
/// Standard 8k tier minimum thinking floor.
pub const STANDARD_8K_MIN_THINKING_FLOOR: u32 = 512;

/// Medium 32k context window tier limit.
pub const MEDIUM_32K_CONTEXT_WINDOW: u32 = 32_768;
/// Medium 32k tier max output ceiling.
pub const MEDIUM_32K_MAX_OUTPUT_CEILING: u32 = 8_192;
/// Medium 32k tier minimum thinking floor.
pub const MEDIUM_32K_MIN_THINKING_FLOOR: u32 = 1_024;

/// Medium 64k context window tier limit.
pub const MEDIUM_64K_CONTEXT_WINDOW: u32 = 65_536;
/// Medium 64k tier max output ceiling.
pub const MEDIUM_64K_MAX_OUTPUT_CEILING: u32 = 16_384;
/// Medium 64k tier minimum thinking floor.
pub const MEDIUM_64K_MIN_THINKING_FLOOR: u32 = 1_024;

/// Large 128k context window tier limit.
pub const LARGE_128K_CONTEXT_WINDOW: u32 = 131_072;
/// Large 128k tier max output ceiling.
pub const LARGE_128K_MAX_OUTPUT_CEILING: u32 = 65_536;
/// Large 128k tier minimum thinking floor.
pub const LARGE_128K_MIN_THINKING_FLOOR: u32 = 2_048;

/// Large 1M context window tier limit.
pub const LARGE_1M_CONTEXT_WINDOW: u32 = 1_048_576;
/// Large 1M tier max output ceiling.
pub const LARGE_1M_MAX_OUTPUT_CEILING: u32 = 65_536;
/// Large 1M tier minimum thinking floor.
pub const LARGE_1M_MIN_THINKING_FLOOR: u32 = 2_048;

/// Model context tier classification profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelContextTier {
    /// Standard tier: 8k context window, 4k output ceiling.
    Standard8k,
    /// Medium tier: 32k context window, 8k output ceiling.
    Medium32k,
    /// Medium tier: 64k context window, 16k output ceiling.
    Medium64k,
    /// Large tier: 128k context window, 64k output ceiling.
    Large128k,
    /// Massive tier: 1M+ context window, 64k output ceiling.
    Large1M,
    /// Custom tier with user-defined boundaries.
    Custom,
}

impl fmt::Display for ModelContextTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard8k => write!(f, "Standard-8k"),
            Self::Medium32k => write!(f, "Medium-32k"),
            Self::Medium64k => write!(f, "Medium-64k"),
            Self::Large128k => write!(f, "Large-128k"),
            Self::Large1M => write!(f, "Large-1M"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Profile defining context and token boundaries for a model context tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextTierProfile {
    pub tier: ModelContextTier,
    pub context_window: u32,
    pub max_output_ceiling: u32,
    pub min_thinking_floor: u32,
    pub min_headroom_tokens: u32,
    pub default_thinking_ratio: f64,
}

impl ContextTierProfile {
    /// Returns the standard profile for a given context tier.
    pub fn for_tier(tier: ModelContextTier) -> Self {
        match tier {
            ModelContextTier::Standard8k => Self {
                tier,
                context_window: STANDARD_8K_CONTEXT_WINDOW,
                max_output_ceiling: STANDARD_8K_MAX_OUTPUT_CEILING,
                min_thinking_floor: STANDARD_8K_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
            ModelContextTier::Medium32k => Self {
                tier,
                context_window: MEDIUM_32K_CONTEXT_WINDOW,
                max_output_ceiling: MEDIUM_32K_MAX_OUTPUT_CEILING,
                min_thinking_floor: MEDIUM_32K_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
            ModelContextTier::Medium64k => Self {
                tier,
                context_window: MEDIUM_64K_CONTEXT_WINDOW,
                max_output_ceiling: MEDIUM_64K_MAX_OUTPUT_CEILING,
                min_thinking_floor: MEDIUM_64K_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
            ModelContextTier::Large128k => Self {
                tier,
                context_window: LARGE_128K_CONTEXT_WINDOW,
                max_output_ceiling: LARGE_128K_MAX_OUTPUT_CEILING,
                min_thinking_floor: LARGE_128K_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
            ModelContextTier::Large1M => Self {
                tier,
                context_window: LARGE_1M_CONTEXT_WINDOW,
                max_output_ceiling: LARGE_1M_MAX_OUTPUT_CEILING,
                min_thinking_floor: LARGE_1M_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR * 2,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
            ModelContextTier::Custom => Self {
                tier,
                context_window: LARGE_128K_CONTEXT_WINDOW,
                max_output_ceiling: LARGE_128K_MAX_OUTPUT_CEILING,
                min_thinking_floor: LARGE_128K_MIN_THINKING_FLOOR,
                min_headroom_tokens: MIN_HEADROOM_TOKENS_FLOOR,
                default_thinking_ratio: DEFAULT_PROPORTIONAL_THINKING_RATIO,
            },
        }
    }

    /// Constructs a custom tier profile with explicit parameters.
    pub fn custom(
        context_window: u32,
        max_output_ceiling: u32,
        min_thinking_floor: u32,
        min_headroom_tokens: u32,
        default_thinking_ratio: f64,
    ) -> Self {
        Self {
            tier: ModelContextTier::Custom,
            context_window,
            max_output_ceiling,
            min_thinking_floor,
            min_headroom_tokens,
            default_thinking_ratio,
        }
    }

    /// Resolves the context tier profile from a model name using AGENTS.md wildcard patterns.
    pub fn from_model_name(model_name: &str) -> Self {
        let lower = model_name.trim().to_ascii_lowercase();

        // 1. Massive 1M+ context window models (Gemini series)
        if wildcard_match("gemini-*-flash*", &lower)
            || wildcard_match("gemini-*-pro*", &lower)
            || wildcard_match("gemini-1.5-*", &lower)
            || wildcard_match("gemini-2.0-*", &lower)
            || wildcard_match("gemini-2.5-*", &lower)
            || wildcard_match("gemini-3*", &lower)
            || lower.contains("1m")
        {
            return Self::for_tier(ModelContextTier::Large1M);
        }

        // 2. Large 128k - 200k context window models (Claude 3.x, GPT-4o, o1, o3)
        if wildcard_match("claude-3-7-*", &lower)
            || wildcard_match("claude-3-5-*", &lower)
            || wildcard_match("claude-3-*", &lower)
            || wildcard_match("o1-*", &lower)
            || wildcard_match("o3-*", &lower)
            || wildcard_match("gpt-4o*", &lower)
            || wildcard_match("gpt-4-turbo*", &lower)
            || lower.contains("128k")
            || lower.contains("200k")
        {
            return Self::for_tier(ModelContextTier::Large128k);
        }

        // 3. Medium 64k context models
        if wildcard_match("*-64k*", &lower) || lower.contains("claude-2.1") {
            return Self::for_tier(ModelContextTier::Medium64k);
        }

        // 4. Medium 32k context models
        if wildcard_match("*-32k*", &lower) || wildcard_match("gpt-4-32k*", &lower) {
            return Self::for_tier(ModelContextTier::Medium32k);
        }

        // 5. Standard 8k context models
        if wildcard_match("gpt-3.5*", &lower)
            || wildcard_match("claude-instant*", &lower)
            || wildcard_match("*-8k*", &lower)
            || lower == "gpt-4"
            || lower == "gpt-4-0613"
        {
            return Self::for_tier(ModelContextTier::Standard8k);
        }

        // Safe fallback: default to Large 128k profile
        Self::for_tier(ModelContextTier::Large128k)
    }
}

/// Matches a string against a wildcard pattern containing `*` or `?`.
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p_bytes = pattern.as_bytes();
    let t_bytes = text.as_bytes();
    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx: Option<usize> = None;
    let mut match_idx = 0;

    while t_idx < t_bytes.len() {
        if p_idx < p_bytes.len() && (p_bytes[p_idx] == b'?' || p_bytes[p_idx] == t_bytes[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p_bytes.len() && p_bytes[p_idx] == b'*' {
            star_idx = Some(p_idx);
            p_idx += 1;
            match_idx = t_idx;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_bytes.len() && p_bytes[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == p_bytes.len()
}

/// Request parameters for proportional thinking budget allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProportionalBudgetRequest {
    pub model_name: String,
    pub prompt_tokens: u32,
    pub requested_max_output_tokens: Option<u32>,
    pub requested_thinking_budget: Option<u32>,
    pub custom_ratio: Option<f64>,
    pub tier_override: Option<ModelContextTier>,
    pub enforce_proportional: bool,
}

impl ProportionalBudgetRequest {
    /// Constructs a basic proportional budget request with model name and prompt token count.
    pub fn new(model_name: impl Into<String>, prompt_tokens: u32) -> Self {
        Self {
            model_name: model_name.into(),
            prompt_tokens,
            requested_max_output_tokens: None,
            requested_thinking_budget: None,
            custom_ratio: None,
            tier_override: None,
            enforce_proportional: false,
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

    /// Sets a custom thinking budget allocation ratio (e.g. 0.80 for 80%).
    pub fn with_custom_ratio(mut self, ratio: f64) -> Self {
        self.custom_ratio = Some(ratio);
        self
    }

    /// Overrides the auto-detected model context tier.
    pub fn with_tier_override(mut self, tier: ModelContextTier) -> Self {
        self.tier_override = Some(tier);
        self
    }

    /// Enforces strict proportional capping even when a client requests an explicit budget.
    pub fn with_enforce_proportional(mut self, enforce: bool) -> Self {
        self.enforce_proportional = enforce;
        self
    }
}

/// Action taken during proportional token budget allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProportionalAllocationAction {
    /// No adjustment needed; parameters satisfy proportional and headroom invariants.
    NoAdjustmentNeeded,
    /// Default proportional thinking budget was calculated and allocated.
    DefaultProportionalAllocated {
        ratio: f64,
        thinking_budget: u32,
        headroom: u32,
    },
    /// Client-requested thinking budget was within safe bounds and preserved.
    RequestedBudgetPreserved {
        thinking_budget: u32,
        headroom: u32,
    },
    /// Requested budget was below minimum thinking floor and clamped upwards.
    ClampedUpToFloor {
        requested: u32,
        clamped: u32,
        floor: u32,
    },
    /// Requested budget was clamped down to guarantee required output headroom.
    ClampedDownForHeadroom {
        requested: u32,
        clamped: u32,
        min_headroom: u32,
    },
    /// Max output tokens were clamped down because total tokens would exceed the context window.
    ClampedDueToContextOverflow {
        requested_max_output: u32,
        clamped_max_output: u32,
        prompt_tokens: u32,
        context_window: u32,
    },
    /// Thinking budget was explicitly set to 0 or disabled.
    ThinkingDisabled,
}

impl fmt::Display for ProportionalAllocationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdjustmentNeeded => write!(f, "No adjustment needed"),
            Self::DefaultProportionalAllocated {
                ratio,
                thinking_budget,
                headroom,
            } => {
                write!(
                    f,
                    "Allocated default proportional thinking: {:.1}% (thinking: {}, headroom: {})",
                    ratio * 100.0,
                    thinking_budget,
                    headroom
                )
            }
            Self::RequestedBudgetPreserved {
                thinking_budget,
                headroom,
            } => {
                write!(
                    f,
                    "Preserved requested thinking budget: {} (headroom: {})",
                    thinking_budget, headroom
                )
            }
            Self::ClampedUpToFloor {
                requested,
                clamped,
                floor,
            } => {
                write!(
                    f,
                    "Clamped thinking budget up to floor: {} -> {} (floor: {})",
                    requested, clamped, floor
                )
            }
            Self::ClampedDownForHeadroom {
                requested,
                clamped,
                min_headroom,
            } => {
                write!(
                    f,
                    "Clamped thinking budget down for headroom: {} -> {} (min headroom: {})",
                    requested, clamped, min_headroom
                )
            }
            Self::ClampedDueToContextOverflow {
                requested_max_output,
                clamped_max_output,
                prompt_tokens,
                context_window,
            } => {
                write!(
                    f,
                    "Clamped max output tokens due to context overflow: {} -> {} (prompt: {}, context: {})",
                    requested_max_output, clamped_max_output, prompt_tokens, context_window
                )
            }
            Self::ThinkingDisabled => write!(f, "Thinking disabled (budget 0)"),
        }
    }
}

/// Outcome of the proportional thinking budget allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProportionalAllocationOutcome {
    pub effective_max_output_tokens: u32,
    pub effective_thinking_budget: Option<u32>,
    pub headroom_tokens: u32,
    pub remaining_context_tokens: u32,
    pub applied_ratio: f64,
    pub tier_profile: ContextTierProfile,
    pub action: ProportionalAllocationAction,
    pub is_valid: bool,
}

impl ProportionalAllocationOutcome {
    /// Returns true if thinking is active and non-zero.
    pub fn has_thinking(&self) -> bool {
        self.effective_thinking_budget.map_or(false, |b| b > 0)
    }

    /// Returns true if any budget clamping was performed.
    pub fn is_clamped(&self) -> bool {
        matches!(
            self.action,
            ProportionalAllocationAction::ClampedUpToFloor { .. }
                | ProportionalAllocationAction::ClampedDownForHeadroom { .. }
                | ProportionalAllocationAction::ClampedDueToContextOverflow { .. }
        )
    }

    /// Returns the effective output tokens headroom.
    pub fn headroom_tokens(&self) -> u32 {
        self.headroom_tokens
    }

    /// Returns the effective max output tokens limit.
    pub fn effective_max_output(&self) -> u32 {
        self.effective_max_output_tokens
    }

    /// Returns the allocated thinking budget, if any.
    pub fn effective_thinking_budget(&self) -> Option<u32> {
        self.effective_thinking_budget
    }
}

/// Errors occurring during proportional thinking budget evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProportionalBudgetError {
    /// Prompt tokens alone exceed or exhaust the model context window.
    ContextWindowExhausted {
        prompt_tokens: u32,
        context_window: u32,
    },
    /// Remaining context tokens are insufficient to support minimum generation headroom.
    InsufficientContextForGeneration {
        remaining_tokens: u32,
        required_minimum: u32,
    },
    /// Invalid thinking budget ratio (must be strictly between 0.0 and 1.0 exclusive).
    InvalidRatio {
        ratio: String,
        reason: String,
    },
}

impl fmt::Display for ProportionalBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextWindowExhausted {
                prompt_tokens,
                context_window,
            } => {
                write!(
                    f,
                    "Prompt tokens ({prompt_tokens}) exhaust model context window ({context_window})"
                )
            }
            Self::InsufficientContextForGeneration {
                remaining_tokens,
                required_minimum,
            } => {
                write!(
                    f,
                    "Remaining context tokens ({remaining_tokens}) insufficient for minimum generation ({required_minimum})"
                )
            }
            Self::InvalidRatio { ratio, reason } => {
                write!(f, "Invalid proportional ratio '{ratio}': {reason}")
            }
        }
    }
}

impl std::error::Error for ProportionalBudgetError {}

impl From<ProportionalBudgetError> for AppError {
    fn from(err: ProportionalBudgetError) -> Self {
        AppError::Config(format!("Proportional Budget Allocator Error: {err}"))
    }
}

/// Calculates and enforces proportional thinking budget allocation across context tiers.
pub fn allocate_proportional_thinking_budget(
    request: &ProportionalBudgetRequest,
) -> Result<ProportionalAllocationOutcome, ProportionalBudgetError> {
    // 1. Resolve context tier profile
    let profile = match request.tier_override {
        Some(tier) => ContextTierProfile::for_tier(tier),
        None => ContextTierProfile::from_model_name(&request.model_name),
    };

    // 2. Validate prompt token budget against context window
    if request.prompt_tokens >= profile.context_window {
        return Err(ProportionalBudgetError::ContextWindowExhausted {
            prompt_tokens: request.prompt_tokens,
            context_window: profile.context_window,
        });
    }

    let remaining_context = profile.context_window.saturating_sub(request.prompt_tokens);
    if remaining_context < profile.min_headroom_tokens {
        return Err(
            ProportionalBudgetError::InsufficientContextForGeneration {
                remaining_tokens: remaining_context,
                required_minimum: profile.min_headroom_tokens,
            },
        );
    }

    // 3. Resolve allocation ratio
    let ratio = request
        .custom_ratio
        .unwrap_or(profile.default_thinking_ratio);
    if ratio <= 0.0 || ratio >= 1.0 {
        return Err(ProportionalBudgetError::InvalidRatio {
            ratio: ratio.to_string(),
            reason: "Ratio must be strictly greater than 0.0 and less than 1.0".to_string(),
        });
    }

    // 4. Determine effective max output tokens bounded by model ceiling and remaining context
    let raw_max_output = request
        .requested_max_output_tokens
        .unwrap_or(profile.max_output_ceiling)
        .min(profile.max_output_ceiling);

    let (effective_max_output, context_clamped_action) = if raw_max_output > remaining_context {
        (
            remaining_context,
            Some(
                ProportionalAllocationAction::ClampedDueToContextOverflow {
                    requested_max_output: raw_max_output,
                    clamped_max_output: remaining_context,
                    prompt_tokens: request.prompt_tokens,
                    context_window: profile.context_window,
                },
            ),
        )
    } else {
        (raw_max_output, None)
    };

    // 5. Evaluate thinking budget allocation
    let (effective_thinking, action) = match request.requested_thinking_budget {
        // Explicitly disabled thinking
        Some(0) => (None, ProportionalAllocationAction::ThinkingDisabled),

        // Explicit thinking budget requested by client
        Some(requested) => {
            let max_allowable_budget =
                effective_max_output.saturating_sub(profile.min_headroom_tokens);

            if request.enforce_proportional {
                let proportional_target =
                    ((effective_max_output as f64) * ratio).round() as u32;
                let upper_limit = proportional_target.min(max_allowable_budget);
                let lower_limit = profile.min_thinking_floor.min(upper_limit);

                if requested < lower_limit {
                    (
                        Some(lower_limit),
                        ProportionalAllocationAction::ClampedUpToFloor {
                            requested,
                            clamped: lower_limit,
                            floor: profile.min_thinking_floor,
                        },
                    )
                } else if requested > upper_limit {
                    (
                        Some(upper_limit),
                        ProportionalAllocationAction::ClampedDownForHeadroom {
                            requested,
                            clamped: upper_limit,
                            min_headroom: profile.min_headroom_tokens,
                        },
                    )
                } else {
                    let headroom = effective_max_output.saturating_sub(requested);
                    (
                        Some(requested),
                        ProportionalAllocationAction::RequestedBudgetPreserved {
                            thinking_budget: requested,
                            headroom,
                        },
                    )
                }
            } else if requested > max_allowable_budget {
                (
                    Some(max_allowable_budget),
                    ProportionalAllocationAction::ClampedDownForHeadroom {
                        requested,
                        clamped: max_allowable_budget,
                        min_headroom: profile.min_headroom_tokens,
                    },
                )
            } else if requested < profile.min_thinking_floor
                && effective_max_output
                    >= profile.min_thinking_floor + profile.min_headroom_tokens
            {
                (
                    Some(profile.min_thinking_floor),
                    ProportionalAllocationAction::ClampedUpToFloor {
                        requested,
                        clamped: profile.min_thinking_floor,
                        floor: profile.min_thinking_floor,
                    },
                )
            } else {
                let headroom = effective_max_output.saturating_sub(requested);
                (
                    Some(requested),
                    ProportionalAllocationAction::RequestedBudgetPreserved {
                        thinking_budget: requested,
                        headroom,
                    },
                )
            }
        }

        // Thinking budget omitted: calculate proportional budget
        None => {
            if effective_max_output
                <= profile.min_headroom_tokens + profile.min_thinking_floor / 2
            {
                (None, ProportionalAllocationAction::ThinkingDisabled)
            } else {
                let max_allowable =
                    effective_max_output.saturating_sub(profile.min_headroom_tokens);
                let calculated = ((effective_max_output as f64) * ratio).round() as u32;
                let bounded = calculated
                    .min(max_allowable)
                    .max(profile.min_thinking_floor.min(max_allowable));

                let headroom = effective_max_output.saturating_sub(bounded);
                (
                    Some(bounded),
                    ProportionalAllocationAction::DefaultProportionalAllocated {
                        ratio,
                        thinking_budget: bounded,
                        headroom,
                    },
                )
            }
        }
    };

    // If context clamping happened and thinking action was NoAdjustmentNeeded, prioritize context action
    let final_action = context_clamped_action.unwrap_or(action);
    let final_headroom =
        effective_max_output.saturating_sub(effective_thinking.unwrap_or(0));

    Ok(ProportionalAllocationOutcome {
        effective_max_output_tokens: effective_max_output,
        effective_thinking_budget: effective_thinking,
        headroom_tokens: final_headroom,
        remaining_context_tokens: remaining_context,
        applied_ratio: ratio,
        tier_profile: profile,
        action: final_action,
        is_valid: true,
    })
}

/// Evaluates proportional token allocation using default standard parameters.
pub fn evaluate_proportional_budget_default(
    model_name: &str,
    prompt_tokens: u32,
    requested_max_output: Option<u32>,
    requested_thinking_budget: Option<u32>,
) -> Result<ProportionalAllocationOutcome, ProportionalBudgetError> {
    let req = ProportionalBudgetRequest::new(model_name, prompt_tokens)
        .with_max_output_tokens(requested_max_output.unwrap_or(DEFAULT_PROPORTIONAL_THINKING_RATIO as u32))
        .with_enforce_proportional(true);

    let req = match requested_max_output {
        Some(m) => req.with_max_output_tokens(m),
        None => req,
    };

    let req = match requested_thinking_budget {
        Some(b) => req.with_thinking_budget(b),
        None => req,
    };

    allocate_proportional_thinking_budget(&req)
}
