//! Risky Path Replacement & Explicit Fallback Guard.
//!
//! Enforces risky path safety per AGENTS.md:
//! "Keep an explicit fallback to the verified path whenever you replace it
//! (e.g. '0 matches found -> full restart'). A silent no-op is worse than the
//! failure it was meant to fix."
//!
//! Additional invariants enforced:
//! - "Order side effects to fail before the point of no return: write credentials
//!    before killing a process, validate before deleting."
//! - "Detect broadly, act narrowly: a matcher may recognize a whole class of
//!    problems, while its effect stays inside the intended data — not across line
//!    breaks, tags, or other clauses. Bound every wait with a timeout."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Defines the fallback strategy required whenever replacing an existing verified path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Full restart using the verified canonical execution path (e.g. "0 matches found -> full restart").
    VerifiedPathRestart,
    /// Explicitly registered fallback handler with name and target path.
    ExplicitHandler {
        handler_name: String,
        target_path: String,
    },
    /// Explicit abort with a structured diagnostic error reason rather than continuing silently.
    FailExplicitly {
        reason: String,
    },
    /// Strictly forbidden: silent no-op (swallows failure or 0 matches silently).
    SilentNoOp,
    /// Strictly forbidden: no fallback strategy specified at all.
    Unspecified,
}

impl FallbackStrategy {
    /// Returns true if this strategy represents an acceptable, explicit fallback.
    pub fn is_explicit(&self) -> bool {
        matches!(
            self,
            Self::VerifiedPathRestart | Self::ExplicitHandler { .. } | Self::FailExplicitly { .. }
        )
    }

    /// Alias for `is_explicit` conforming to runtime validator checks.
    pub fn is_explicit_fallback(&self) -> bool {
        self.is_explicit()
    }

    /// Returns true if this strategy is an unhandled silent no-op.
    pub fn is_silent_no_op(&self) -> bool {
        matches!(self, Self::SilentNoOp)
    }

    /// Formats a human-readable summary of the fallback strategy.
    pub fn description(&self) -> String {
        match self {
            Self::VerifiedPathRestart => "Verified path full restart".to_string(),
            Self::ExplicitHandler {
                handler_name,
                target_path,
            } => {
                format!("Explicit handler '{handler_name}' targeting '{target_path}'")
            }
            Self::FailExplicitly { reason } => {
                format!("Explicit failure: {reason}")
            }
            Self::SilentNoOp => "Forbidden silent no-op".to_string(),
            Self::Unspecified => "Unspecified fallback strategy".to_string(),
        }
    }
}

/// A speculative or optimized replacement path configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskyPathReplacement {
    pub replacement_name: String,
    pub speculative_path: String,
    pub verified_fallback_path: String,
    pub strategy: FallbackStrategy,
    pub handles_zero_matches: bool,
    pub handles_error_failover: bool,
}

impl RiskyPathReplacement {
    /// Creates a new risky path replacement specification.
    pub fn new(
        name: impl Into<String>,
        speculative: impl Into<String>,
        verified: impl Into<String>,
        strategy: FallbackStrategy,
    ) -> Self {
        Self {
            replacement_name: name.into(),
            speculative_path: speculative.into(),
            verified_fallback_path: verified.into(),
            strategy,
            handles_zero_matches: true,
            handles_error_failover: true,
        }
    }
}

/// Represents the execution parameters and safety constraints of a path replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathReplacementExecution {
    /// Unique identifier or name of the operation being replaced.
    pub operation_id: String,
    /// Description of the target path or pattern being replaced.
    pub target_description: String,
    /// Number of matches or transformed items produced by candidate replacement.
    pub matches_found: usize,
    /// The fallback strategy configured for this replacement.
    pub fallback_strategy: FallbackStrategy,
    /// Whether side effects were pre-validated before reaching point of no return.
    pub side_effects_prevalidated: bool,
    /// Timeout bound in milliseconds (bounded wait requirement).
    pub timeout_ms: Option<u64>,
    /// Whether the action was verified to act narrowly within the intended data scope.
    pub narrow_scope_verified: bool,
}

impl PathReplacementExecution {
    /// Creates a new `PathReplacementExecution` with default safety invariants.
    pub fn new(
        operation_id: impl Into<String>,
        target_description: impl Into<String>,
        fallback_strategy: FallbackStrategy,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            target_description: target_description.into(),
            matches_found: 0,
            fallback_strategy,
            side_effects_prevalidated: true,
            timeout_ms: Some(5000), // Default 5s timeout to avoid unbounded waits
            narrow_scope_verified: true,
        }
    }

    /// Sets the number of matches found during replacement.
    pub fn with_matches(mut self, count: usize) -> Self {
        self.matches_found = count;
        self
    }

    /// Sets whether side effects were pre-validated before point of no return.
    pub fn with_prevalidation(mut self, prevalidated: bool) -> Self {
        self.side_effects_prevalidated = prevalidated;
        self
    }

    /// Configures the timeout bound in milliseconds.
    pub fn with_timeout_ms(mut self, timeout: Option<u64>) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Sets whether narrow scope was confirmed.
    pub fn with_narrow_scope(mut self, narrow: bool) -> Self {
        self.narrow_scope_verified = narrow;
        self
    }

    /// Converts this execution configuration into a `RiskyPathReplacement` specification.
    pub fn to_risky_path_replacement(&self) -> RiskyPathReplacement {
        RiskyPathReplacement {
            replacement_name: self.operation_id.clone(),
            speculative_path: self.target_description.clone(),
            verified_fallback_path: format!("verified_{}", self.operation_id),
            strategy: self.fallback_strategy.clone(),
            handles_zero_matches: self.fallback_strategy.is_explicit(),
            handles_error_failover: self.fallback_strategy.is_explicit(),
        }
    }
}

/// Violations identified when inspecting a risky path replacement against AGENTS.md rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementViolation {
    /// Silent no-op strategy is strictly forbidden.
    SilentNoOpForbidden { context: String },
    /// Missing or unspecified fallback strategy.
    MissingFallbackStrategy { replacement_name: String },
    /// The speculative path fails to explicitly handle 0 matches found by falling back.
    MissingZeroMatchHandling { replacement_name: String },
    /// The speculative path fails to explicitly handle error failovers.
    MissingErrorFailover { replacement_name: String },
    /// The verified fallback path target cannot be empty.
    EmptyVerifiedPath { replacement_name: String },
    /// The speculative path cannot be empty.
    EmptySpeculativePath { replacement_name: String },
    /// The speculative path and verified fallback path are identical (redundant replacement).
    RedundantFallbackSamePath { path: String },
    /// Missing explicit fallback for execution.
    MissingExplicitFallback {
        operation_id: String,
        target_description: String,
    },
    /// Replacement found 0 matches without an explicit fallback to a verified path.
    UnhandledZeroMatches {
        operation_id: String,
        target_description: String,
        matches_found: usize,
    },
    /// Side-effects were scheduled or executed without pre-validation before point of no return.
    SideEffectsNotPrevalidated {
        operation_id: String,
        details: String,
    },
    /// Operation performs an unbounded wait without a timeout.
    UnboundedWaitMissingTimeout {
        operation_id: String,
        details: String,
    },
    /// Broad matcher acted beyond intended data boundaries (e.g. across lines/clauses).
    OverbroadScopeDetected {
        operation_id: String,
        details: String,
    },
}

impl fmt::Display for ReplacementViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SilentNoOpForbidden { context } => {
                write!(
                    f,
                    "Silent no-op is forbidden for risky path replacement: {context}. A silent no-op is worse than the failure it was meant to fix."
                )
            }
            Self::MissingFallbackStrategy { replacement_name } => {
                write!(
                    f,
                    "Missing explicit fallback strategy for risky replacement '{replacement_name}'."
                )
            }
            Self::MissingZeroMatchHandling { replacement_name } => {
                write!(
                    f,
                    "Speculative replacement '{replacement_name}' does not handle 0 matches with an explicit fallback."
                )
            }
            Self::MissingErrorFailover { replacement_name } => {
                write!(
                    f,
                    "Speculative replacement '{replacement_name}' does not handle error failover."
                )
            }
            Self::EmptyVerifiedPath { replacement_name } => {
                write!(
                    f,
                    "Verified fallback path for '{replacement_name}' cannot be empty."
                )
            }
            Self::EmptySpeculativePath { replacement_name } => {
                write!(
                    f,
                    "Speculative path for '{replacement_name}' cannot be empty."
                )
            }
            Self::RedundantFallbackSamePath { path } => {
                write!(
                    f,
                    "Speculative and fallback path are identical: '{path}'."
                )
            }
            Self::MissingExplicitFallback {
                operation_id,
                target_description,
            } => {
                write!(
                    f,
                    "Missing explicit fallback for operation '{operation_id}' targeting '{target_description}'."
                )
            }
            Self::UnhandledZeroMatches {
                operation_id,
                target_description,
                matches_found,
            } => {
                write!(
                    f,
                    "Unhandled 0 matches for operation '{operation_id}' ('{target_description}', found: {matches_found}). Must fallback explicitly."
                )
            }
            Self::SideEffectsNotPrevalidated {
                operation_id,
                details,
            } => {
                write!(
                    f,
                    "Side effects not pre-validated before point of no return for '{operation_id}': {details}."
                )
            }
            Self::UnboundedWaitMissingTimeout {
                operation_id,
                details,
            } => {
                write!(
                    f,
                    "Unbounded wait detected without timeout bound for '{operation_id}': {details}."
                )
            }
            Self::OverbroadScopeDetected {
                operation_id,
                details,
            } => {
                write!(
                    f,
                    "Matcher acted beyond narrow intended data scope for '{operation_id}': {details}."
                )
            }
        }
    }
}

impl std::error::Error for ReplacementViolation {}

/// Comprehensive report returned by replacement strategy validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementValidationReport {
    pub is_valid: bool,
    pub has_explicit_fallback: bool,
    pub handles_zero_matches: bool,
    pub violations: Vec<ReplacementViolation>,
    pub summary: String,
}

/// Validates a risky path replacement configuration against AGENTS.md rules.
pub fn validate_replacement_strategy(
    replacement: &RiskyPathReplacement,
) -> Result<ReplacementValidationReport, ReplacementViolation> {
    let mut violations = Vec::new();

    let clean_spec = replacement.speculative_path.trim();
    let clean_verified = replacement.verified_fallback_path.trim();

    if clean_spec.is_empty() {
        violations.push(ReplacementViolation::EmptySpeculativePath {
            replacement_name: replacement.replacement_name.clone(),
        });
    }

    if clean_verified.is_empty() {
        violations.push(ReplacementViolation::EmptyVerifiedPath {
            replacement_name: replacement.replacement_name.clone(),
        });
    }

    if !clean_spec.is_empty() && clean_spec == clean_verified {
        violations.push(ReplacementViolation::RedundantFallbackSamePath {
            path: clean_spec.to_string(),
        });
    }

    match &replacement.strategy {
        FallbackStrategy::SilentNoOp => {
            violations.push(ReplacementViolation::SilentNoOpForbidden {
                context: format!(
                    "Replacement '{}' configured as SilentNoOp",
                    replacement.replacement_name
                ),
            });
        }
        FallbackStrategy::Unspecified => {
            violations.push(ReplacementViolation::MissingFallbackStrategy {
                replacement_name: replacement.replacement_name.clone(),
            });
        }
        FallbackStrategy::VerifiedPathRestart
        | FallbackStrategy::ExplicitHandler { .. }
        | FallbackStrategy::FailExplicitly { .. } => {
            // Explicit strategies are accepted.
        }
    }

    if !replacement.handles_zero_matches {
        violations.push(ReplacementViolation::MissingZeroMatchHandling {
            replacement_name: replacement.replacement_name.clone(),
        });
    }

    if !replacement.handles_error_failover {
        violations.push(ReplacementViolation::MissingErrorFailover {
            replacement_name: replacement.replacement_name.clone(),
        });
    }

    let is_valid = violations.is_empty();
    let has_explicit_fallback = replacement.strategy.is_explicit();
    let handles_zero_matches = replacement.handles_zero_matches;

    let summary = if is_valid {
        format!(
            "Risky path '{}' valid: explicit fallback configured with 0-match failover",
            replacement.replacement_name
        )
    } else {
        format!(
            "Risky path '{}' invalid with {} violation(s)",
            replacement.replacement_name,
            violations.len()
        )
    };

    let report = ReplacementValidationReport {
        is_valid,
        has_explicit_fallback,
        handles_zero_matches,
        violations,
        summary,
    };

    if report.is_valid {
        Ok(report)
    } else {
        Err(report.violations[0].clone())
    }
}

/// Validates that an execution parameters model complies with AGENTS.md rules.
pub fn validate_replacement(execution: &PathReplacementExecution) -> Result<(), ReplacementViolation> {
    // 1. Silent no-op is strictly forbidden
    if execution.fallback_strategy.is_silent_no_op() {
        return Err(ReplacementViolation::SilentNoOpForbidden {
            context: format!(
                "Operation '{}' configured with SilentNoOp. An explicit verified fallback path is mandatory.",
                execution.operation_id
            ),
        });
    }

    // 2. Fallback strategy must be explicitly defined
    if execution.fallback_strategy == FallbackStrategy::Unspecified {
        return Err(ReplacementViolation::MissingExplicitFallback {
            operation_id: execution.operation_id.clone(),
            target_description: execution.target_description.clone(),
        });
    }

    // 3. Zero matches must trigger an explicit fallback strategy
    if execution.matches_found == 0 && !execution.fallback_strategy.is_explicit_fallback() {
        return Err(ReplacementViolation::UnhandledZeroMatches {
            operation_id: execution.operation_id.clone(),
            target_description: execution.target_description.clone(),
            matches_found: execution.matches_found,
        });
    }

    // 4. Side effects must fail before point of no return
    if !execution.side_effects_prevalidated {
        return Err(ReplacementViolation::SideEffectsNotPrevalidated {
            operation_id: execution.operation_id.clone(),
            details: "Pre-validation failed or was omitted prior to irreversible destructive operations.".to_string(),
        });
    }

    // 5. Waits must be bounded by a timeout
    if execution.timeout_ms.is_none() {
        return Err(ReplacementViolation::UnboundedWaitMissingTimeout {
            operation_id: execution.operation_id.clone(),
            details: "Timeout bound is required (e.g. Some(5000) ms) to prevent deadlock hangs.".to_string(),
        });
    }

    // 6. Scope must be narrow and bounded
    if !execution.narrow_scope_verified {
        return Err(ReplacementViolation::OverbroadScopeDetected {
            operation_id: execution.operation_id.clone(),
            details: "Modification scope exceeded target entity boundary or leaked across clauses.".to_string(),
        });
    }

    Ok(())
}

/// Validates a batch of replacement executions and collects all violations.
pub fn validate_replacement_batch(
    executions: &[PathReplacementExecution],
) -> Vec<ReplacementViolation> {
    executions
        .iter()
        .filter_map(|exec| validate_replacement(exec).err())
        .collect()
}

/// Executes a candidate replacement operation with an explicit fallback to the verified path.
pub fn execute_with_fallback<T, F, E>(
    execution: &PathReplacementExecution,
    candidate_op: F,
    verified_fallback_op: E,
) -> Result<T, ReplacementViolation>
where
    F: FnOnce() -> Result<Option<T>, String>,
    E: FnOnce() -> Result<T, String>,
{
    validate_replacement(execution)?;

    match candidate_op() {
        Ok(Some(value)) => {
            if execution.matches_found == 0 {
                match &execution.fallback_strategy {
                    FallbackStrategy::VerifiedPathRestart
                    | FallbackStrategy::ExplicitHandler { .. } => {
                        verified_fallback_op().map_err(|err| ReplacementViolation::UnhandledZeroMatches {
                            operation_id: execution.operation_id.clone(),
                            target_description: format!("Fallback failed: {err}"),
                            matches_found: 0,
                        })
                    }
                    FallbackStrategy::FailExplicitly { reason } => {
                        Err(ReplacementViolation::UnhandledZeroMatches {
                            operation_id: execution.operation_id.clone(),
                            target_description: reason.clone(),
                            matches_found: 0,
                        })
                    }
                    FallbackStrategy::SilentNoOp | FallbackStrategy::Unspecified => {
                        Err(ReplacementViolation::UnhandledZeroMatches {
                            operation_id: execution.operation_id.clone(),
                            target_description: execution.target_description.clone(),
                            matches_found: 0,
                        })
                    }
                }
            } else {
                Ok(value)
            }
        }
        Ok(None) | Err(_) => match &execution.fallback_strategy {
            FallbackStrategy::VerifiedPathRestart
            | FallbackStrategy::ExplicitHandler { .. } => {
                verified_fallback_op().map_err(|err| ReplacementViolation::UnhandledZeroMatches {
                    operation_id: execution.operation_id.clone(),
                    target_description: format!("Fallback execution failed: {err}"),
                    matches_found: 0,
                })
            }
            FallbackStrategy::FailExplicitly { reason } => {
                Err(ReplacementViolation::UnhandledZeroMatches {
                    operation_id: execution.operation_id.clone(),
                    target_description: reason.clone(),
                    matches_found: 0,
                })
            }
            FallbackStrategy::SilentNoOp | FallbackStrategy::Unspecified => {
                Err(ReplacementViolation::UnhandledZeroMatches {
                    operation_id: execution.operation_id.clone(),
                    target_description: execution.target_description.clone(),
                    matches_found: 0,
                })
            }
        },
    }
}

/// Side-effect order state tracker.
///
/// Enforces:
/// "Order side effects to fail before the point of no return:
/// write credentials before killing a process, validate before deleting."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideEffectOrderGuard {
    pub validation_passed: bool,
    pub credentials_written: bool,
    pub point_of_no_return_executed: bool,
}

impl Default for SideEffectOrderGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl SideEffectOrderGuard {
    pub fn new() -> Self {
        Self {
            validation_passed: false,
            credentials_written: false,
            point_of_no_return_executed: false,
        }
    }

    /// Step 1: Pre-validate state before any persistent changes.
    pub fn mark_validated(&mut self) {
        self.validation_passed = true;
    }

    /// Step 2: Persist credentials or backup state prior to destructive actions.
    pub fn mark_credentials_written(&mut self) -> Result<(), ReplacementViolation> {
        if !self.validation_passed {
            return Err(ReplacementViolation::SideEffectsNotPrevalidated {
                operation_id: "side_effect_order_guard".to_string(),
                details: "Cannot write credentials before validation passes".to_string(),
            });
        }
        self.credentials_written = true;
        Ok(())
    }

    /// Step 3: Execute point-of-no-return action (e.g. killing process or deleting resource).
    pub fn execute_destructive_action(&mut self) -> Result<(), ReplacementViolation> {
        if !self.validation_passed {
            return Err(ReplacementViolation::SideEffectsNotPrevalidated {
                operation_id: "side_effect_order_guard".to_string(),
                details: "Cannot execute destructive action: validation was not performed".to_string(),
            });
        }
        if !self.credentials_written {
            return Err(ReplacementViolation::SideEffectsNotPrevalidated {
                operation_id: "side_effect_order_guard".to_string(),
                details: "Cannot execute destructive action: credentials/backup must be written before killing process or deleting".to_string(),
            });
        }
        self.point_of_no_return_executed = true;
        Ok(())
    }
}

/// Static code scanner for pattern detection in source code.
/// Detects unhandled zero-match returns or silent fall-throughs in replacement routines.
pub fn lint_replacement_code_pattern(source_code: &str) -> Vec<ReplacementViolation> {
    let mut violations = Vec::new();

    for line in source_code.lines() {
        let trimmed = line.trim();
        if (trimmed.contains("matches.is_empty()")
            || trimmed.contains("matches.len() == 0")
            || trimmed.contains("found == 0"))
            && (trimmed.ends_with("return Ok(());")
                || trimmed.ends_with("return Ok(vec![]);")
                || trimmed.ends_with("return None;"))
        {
            violations.push(ReplacementViolation::SilentNoOpForbidden {
                context: format!("Suspicious silent no-op on zero matches without fallback: '{trimmed}'"),
            });
        }
    }

    violations
}

/// Static code scanner that inspects source code for dangerous replacement anti-patterns.
pub fn inspect_code_for_silent_replacement(code: &str) -> Vec<ReplacementViolation> {
    let mut violations = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        // Check for silent no-op comments or expressions
        if (trimmed.contains("// silent no-op")
            || trimmed.contains("// do nothing")
            || trimmed.contains("// ignore 0 matches"))
            && (trimmed.contains("replace") || trimmed.contains("match") || trimmed.contains("fallback"))
        {
            violations.push(ReplacementViolation::SilentNoOpForbidden {
                context: format!("Found silent no-op directive in replacement logic on line {}: '{trimmed}'", line_idx + 1),
            });
        }

        // Check for empty zero-matches block: e.g. "if matches == 0 {}" or "if matches == 0 { }"
        if (trimmed.contains("matches == 0") || trimmed.contains("count == 0"))
            && (trimmed.ends_with("{}") || trimmed.ends_with("{ }"))
        {
            violations.push(ReplacementViolation::UnhandledZeroMatches {
                operation_id: format!("line_{}", line_idx + 1),
                target_description: trimmed.to_string(),
                matches_found: 0,
            });
        }

        // Check for kill before credential write comment or marker
        if trimmed.contains("kill_process()")
            && !code[..code.find(line).unwrap_or(0)].contains("write_credentials")
        {
            violations.push(ReplacementViolation::SideEffectsNotPrevalidated {
                operation_id: format!("line_{}", line_idx + 1),
                details: "Process termination invoked before credential persistence".to_string(),
            });
        }
    }

    violations
}
