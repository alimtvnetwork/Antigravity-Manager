//! Side Effect Order Enforcer & Risky Path Replacement Guard (`Imp-655`).
//!
//! Enforces AGENTS.md Risky Path Replacement & Side Effects Ordering:
//! - "Order side effects to fail before point of no return: write credentials before
//!   killing a process, validate before deleting. Keep an explicit fallback to the
//!   verified path whenever you replace it. Bound every wait with a timeout."
//! - Detect broadly, act narrowly: verify irreversible actions (process kill, storage deletion)
//!   strictly follow successful credentials persistence and pre-validation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum default timeout bound for a side-effect stage (5 minutes).
pub const DEFAULT_MAX_STAGE_TIMEOUT_MS: u64 = 300_000;

/// Minimum non-zero timeout bound for wait stages (1 millisecond).
pub const MIN_STAGE_TIMEOUT_MS: u64 = 1;

/// Classification of side-effect operations in an execution plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideEffectKind {
    /// Pre-validation operations (e.g., path checks, precondition verification, permission checks).
    Validation,
    /// Persistence of credentials, auth tokens, session state, or encryption keys.
    CredentialPersist,
    /// Termination, signaling, or killing of running operating system processes.
    ProcessTermination,
    /// Deletion, wiping, or scrubbing of persistent storage, directories, or cache databases.
    StorageDeletion,
}

impl SideEffectKind {
    /// Returns true if this side effect represents an irreversible action / point of no return.
    pub fn is_irreversible(&self) -> bool {
        matches!(self, Self::ProcessTermination | Self::StorageDeletion)
    }

    /// Returns true if this side effect is a recoverable pre-execution precursor.
    pub fn is_safe_precursor(&self) -> bool {
        matches!(self, Self::Validation | Self::CredentialPersist)
    }

    /// Returns true if this is a validation stage.
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation)
    }

    /// Returns true if this is a credential persistence stage.
    pub fn is_credential_persist(&self) -> bool {
        matches!(self, Self::CredentialPersist)
    }

    /// Returns true if this is a process termination stage.
    pub fn is_process_termination(&self) -> bool {
        matches!(self, Self::ProcessTermination)
    }

    /// Returns true if this is a storage deletion stage.
    pub fn is_storage_deletion(&self) -> bool {
        matches!(self, Self::StorageDeletion)
    }

    /// Returns a static string representation of the side effect kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::CredentialPersist => "credential_persist",
            Self::ProcessTermination => "process_termination",
            Self::StorageDeletion => "storage_deletion",
        }
    }
}

impl fmt::Display for SideEffectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A discrete stage in a side-effect execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideEffectStage {
    pub id: String,
    pub kind: SideEffectKind,
    pub description: String,
    pub timeout_ms: Option<u64>,
    pub fallback: Option<String>,
    pub is_replacement: bool,
}

impl SideEffectStage {
    /// Constructs a basic side-effect stage.
    pub fn new(id: impl Into<String>, kind: SideEffectKind) -> Self {
        let id_str = id.into();
        let description = format!("Stage: {}", id_str);
        Self {
            id: id_str,
            kind,
            description,
            timeout_ms: None,
            fallback: None,
            is_replacement: false,
        }
    }

    /// Constructs a pre-validation stage.
    pub fn validation(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: SideEffectKind::Validation,
            description: description.into(),
            timeout_ms: None,
            fallback: None,
            is_replacement: false,
        }
    }

    /// Constructs a credentials persistence stage.
    pub fn credential_persist(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: SideEffectKind::CredentialPersist,
            description: description.into(),
            timeout_ms: None,
            fallback: None,
            is_replacement: false,
        }
    }

    /// Constructs a process termination stage with a mandatory timeout bound.
    pub fn process_termination(
        id: impl Into<String>,
        description: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            kind: SideEffectKind::ProcessTermination,
            description: description.into(),
            timeout_ms: Some(timeout_ms),
            fallback: None,
            is_replacement: false,
        }
    }

    /// Constructs a storage deletion stage.
    pub fn storage_deletion(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: SideEffectKind::StorageDeletion,
            description: description.into(),
            timeout_ms: None,
            fallback: None,
            is_replacement: false,
        }
    }

    /// Sets the stage description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Bounds this stage with an explicit timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Attaches an explicit verified fallback path to this stage.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    /// Marks this stage as a replacement path requiring a verified fallback.
    pub fn with_replacement(mut self, is_replacement: bool) -> Self {
        self.is_replacement = is_replacement;
        self
    }

    /// Returns true if this stage represents an irreversible operation.
    pub fn is_irreversible(&self) -> bool {
        self.kind.is_irreversible()
    }

    /// Returns true if this stage has a timeout bound.
    pub fn has_timeout(&self) -> bool {
        self.timeout_ms.is_some()
    }

    /// Returns true if this stage has a non-empty fallback specified.
    pub fn has_fallback(&self) -> bool {
        self.fallback
            .as_deref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }
}

/// Policy governing validation constraints on side-effect execution plans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideEffectOrderPolicy {
    /// Enforce that credential persist stages strictly precede process termination.
    pub enforce_credential_before_kill: bool,
    /// Enforce that storage deletion is preceded by at least one validation stage.
    pub require_validation_before_deletion: bool,
    /// Enforce that process termination wait has a non-zero, bounded timeout.
    pub require_bounded_timeout: bool,
    /// Maximum allowed timeout in milliseconds.
    pub max_timeout_ms: u64,
    /// Enforce that any stage flagged as replacement specifies an explicit verified fallback path.
    pub require_fallback_on_replacement: bool,
}

impl Default for SideEffectOrderPolicy {
    fn default() -> Self {
        Self {
            enforce_credential_before_kill: true,
            require_validation_before_deletion: true,
            require_bounded_timeout: true,
            max_timeout_ms: DEFAULT_MAX_STAGE_TIMEOUT_MS,
            require_fallback_on_replacement: true,
        }
    }
}

impl SideEffectOrderPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_timeout_ms(mut self, max_ms: u64) -> Self {
        self.max_timeout_ms = max_ms;
        self
    }

    pub fn with_require_bounded_timeout(mut self, require: bool) -> Self {
        self.require_bounded_timeout = require;
        self
    }

    pub fn with_require_fallback_on_replacement(mut self, require: bool) -> Self {
        self.require_fallback_on_replacement = require;
        self
    }
}

/// An ordered plan of side-effect operations to be validated before execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideEffectPlan {
    pub plan_id: String,
    pub stages: Vec<SideEffectStage>,
    pub policy: SideEffectOrderPolicy,
}

impl SideEffectPlan {
    /// Constructs a new empty plan with default policy.
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            stages: Vec::new(),
            policy: SideEffectOrderPolicy::default(),
        }
    }

    /// Sets the policy for this plan.
    pub fn with_policy(mut self, policy: SideEffectOrderPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Appends a stage and returns a mutable reference.
    pub fn add_stage(&mut self, stage: SideEffectStage) -> &mut Self {
        self.stages.push(stage);
        self
    }

    /// Appends a stage and returns self for fluent builder patterns.
    pub fn with_stage(mut self, stage: SideEffectStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Returns the number of stages in the plan.
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Returns true if the plan has no stages.
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Validates the ordering and invariants of this plan using its embedded policy.
    pub fn validate(&self) -> Result<(), SideEffectOrderViolation> {
        validate_side_effect_order_with_policy(self, &self.policy)
    }
}

/// Violations identified when verifying side-effect execution order and bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectOrderViolation {
    /// Plan contains no stages.
    EmptyPlan,

    /// Process termination occurred before credentials persistence (`kill before write credentials`).
    InvertedCredentialKillOrder {
        kill_stage_id: String,
        credential_stage_id: String,
        kill_index: usize,
        credential_index: usize,
    },

    /// Storage deletion was scheduled without any preceding validation stage.
    MissingValidationBeforeDeletion {
        deletion_stage_id: String,
        deletion_index: usize,
    },

    /// Validation stage was scheduled after storage deletion.
    InvertedValidationDeletionOrder {
        deletion_stage_id: String,
        validation_stage_id: String,
        deletion_index: usize,
        validation_index: usize,
    },

    /// An irreversible wait or termination stage has no bounded timeout.
    UnboundedWaitTimeout {
        stage_id: String,
        kind: SideEffectKind,
    },

    /// Stage timeout exceeds the maximum allowed threshold.
    TimeoutExceedsMaximum {
        stage_id: String,
        timeout_ms: u64,
        max_timeout_ms: u64,
    },

    /// Replacement stage lacks an explicit fallback to the verified path.
    MissingVerifiedPathFallback { stage_id: String, operation: String },
}

impl fmt::Display for SideEffectOrderViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPlan => write!(f, "Side effect plan is empty"),
            Self::InvertedCredentialKillOrder {
                kill_stage_id,
                credential_stage_id,
                kill_index,
                credential_index,
            } => write!(
                f,
                "Inverted order violation: process termination '{}' (index {}) scheduled before credential persistence '{}' (index {})",
                kill_stage_id, kill_index, credential_stage_id, credential_index
            ),
            Self::MissingValidationBeforeDeletion {
                deletion_stage_id,
                deletion_index,
            } => write!(
                f,
                "Missing validation violation: storage deletion '{}' (index {}) has no preceding validation stage",
                deletion_stage_id, deletion_index
            ),
            Self::InvertedValidationDeletionOrder {
                deletion_stage_id,
                validation_stage_id,
                deletion_index,
                validation_index,
            } => write!(
                f,
                "Inverted order violation: storage deletion '{}' (index {}) scheduled before validation '{}' (index {})",
                deletion_stage_id, deletion_index, validation_stage_id, validation_index
            ),
            Self::UnboundedWaitTimeout { stage_id, kind } => write!(
                f,
                "Unbounded wait timeout violation: stage '{}' ({}) must be bounded by a positive timeout",
                stage_id, kind
            ),
            Self::TimeoutExceedsMaximum {
                stage_id,
                timeout_ms,
                max_timeout_ms,
            } => write!(
                f,
                "Timeout exceeds maximum: stage '{}' timeout {}ms exceeds maximum allowed {}ms",
                stage_id, timeout_ms, max_timeout_ms
            ),
            Self::MissingVerifiedPathFallback {
                stage_id,
                operation,
            } => write!(
                f,
                "Missing fallback violation: replacement stage '{}' ({}) must provide an explicit fallback to a verified path",
                stage_id, operation
            ),
        }
    }
}

impl std::error::Error for SideEffectOrderViolation {}

/// Validates that irreversible actions (kill, delete) strictly follow successful
/// credentials persistence and pre-validation, that timeouts are bounded, and that
/// replacement paths have verified fallbacks.
pub fn validate_side_effect_order(plan: &SideEffectPlan) -> Result<(), SideEffectOrderViolation> {
    validate_side_effect_order_with_policy(plan, &plan.policy)
}

/// Validates a plan against an explicit policy.
pub fn validate_side_effect_order_with_policy(
    plan: &SideEffectPlan,
    policy: &SideEffectOrderPolicy,
) -> Result<(), SideEffectOrderViolation> {
    if plan.stages.is_empty() {
        return Err(SideEffectOrderViolation::EmptyPlan);
    }

    // 1. Timeout bounds and fallback verification across all stages
    for stage in &plan.stages {
        // Enforce verified path fallback for replacements
        if policy.require_fallback_on_replacement && stage.is_replacement && !stage.has_fallback() {
            return Err(SideEffectOrderViolation::MissingVerifiedPathFallback {
                stage_id: stage.id.clone(),
                operation: stage.description.clone(),
            });
        }

        // Enforce timeout bounds on process termination and wait-bound stages
        let requires_timeout = policy.require_bounded_timeout
            && (stage.kind == SideEffectKind::ProcessTermination || stage.timeout_ms.is_some());

        if requires_timeout {
            match stage.timeout_ms {
                None | Some(0) => {
                    return Err(SideEffectOrderViolation::UnboundedWaitTimeout {
                        stage_id: stage.id.clone(),
                        kind: stage.kind,
                    });
                }
                Some(timeout) if timeout > policy.max_timeout_ms => {
                    return Err(SideEffectOrderViolation::TimeoutExceedsMaximum {
                        stage_id: stage.id.clone(),
                        timeout_ms: timeout,
                        max_timeout_ms: policy.max_timeout_ms,
                    });
                }
                _ => {}
            }
        }
    }

    // 2. Enforce: Write credentials before killing a process
    if policy.enforce_credential_before_kill {
        for (kill_idx, kill_stage) in plan.stages.iter().enumerate() {
            if kill_stage.kind == SideEffectKind::ProcessTermination {
                let late_credential = plan.stages.iter().enumerate().find(|(idx, s)| {
                    *idx > kill_idx && s.kind == SideEffectKind::CredentialPersist
                });

                if let Some((cred_idx, cred_stage)) = late_credential {
                    return Err(SideEffectOrderViolation::InvertedCredentialKillOrder {
                        kill_stage_id: kill_stage.id.clone(),
                        credential_stage_id: cred_stage.id.clone(),
                        kill_index: kill_idx,
                        credential_index: cred_idx,
                    });
                }
            }
        }
    }

    // 3. Enforce: Validate before deleting
    if policy.require_validation_before_deletion {
        for (del_idx, del_stage) in plan.stages.iter().enumerate() {
            if del_stage.kind == SideEffectKind::StorageDeletion {
                let has_prior_validation = plan.stages[..del_idx]
                    .iter()
                    .any(|s| s.kind == SideEffectKind::Validation);

                if !has_prior_validation {
                    let late_validation = plan.stages[del_idx + 1..]
                        .iter()
                        .enumerate()
                        .find(|(_, s)| s.kind == SideEffectKind::Validation);

                    if let Some((offset, val_stage)) = late_validation {
                        return Err(SideEffectOrderViolation::InvertedValidationDeletionOrder {
                            deletion_stage_id: del_stage.id.clone(),
                            validation_stage_id: val_stage.id.clone(),
                            deletion_index: del_idx,
                            validation_index: del_idx + 1 + offset,
                        });
                    } else {
                        return Err(SideEffectOrderViolation::MissingValidationBeforeDeletion {
                            deletion_stage_id: del_stage.id.clone(),
                            deletion_index: del_idx,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

/// Runtime tracker recording executed stages to prevent out-of-order execution dynamically.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SideEffectExecutionTracker {
    executed_stages: Vec<SideEffectStage>,
}

impl SideEffectExecutionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that a stage was successfully executed after validating invariants.
    pub fn record_executed(
        &mut self,
        stage: SideEffectStage,
    ) -> Result<(), SideEffectOrderViolation> {
        let mut candidate = SideEffectPlan::new("runtime_tracking");
        for prev in &self.executed_stages {
            candidate.add_stage(prev.clone());
        }
        candidate.add_stage(stage.clone());

        validate_side_effect_order(&candidate)?;

        self.executed_stages.push(stage);
        Ok(())
    }

    /// Returns the number of stages recorded so far.
    pub fn count(&self) -> usize {
        self.executed_stages.len()
    }

    /// Returns a slice of the stages executed so far.
    pub fn executed_stages(&self) -> &[SideEffectStage] {
        &self.executed_stages
    }
}
