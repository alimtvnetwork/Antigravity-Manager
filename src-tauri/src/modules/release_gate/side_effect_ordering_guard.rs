//! Side Effect Ordering & Point of No Return Guard.
//!
//! Enforces side effect execution order per AGENTS.md:
//! "Order side effects to fail before the point of no return:
//! write credentials before killing a process, validate before deleting."
//!
//! Architecture:
//! - Validation Phase: Pre-flight checks, schema validation, existence checks.
//! - Staging Phase: Credential persistence, state snapshotting, rollback ref preparation.
//! - Irreversible Execution Phase: Process termination, file deletion, irreversible state mutations (Point of No Return).
//! - Cleanup Phase: Post-execution transient file removal, notification, audit logging.
//!
//! Any reversal or inversion (such as killing a process before writing credentials,
//! or deleting a resource before validating) is intercepted and rejected as a critical
//! architectural violation.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Phase & Pipeline Models (Task 2 Invariant Requirements)
// ============================================================================

/// Distinct lifecycle phases of an operational workflow.
/// Ordered monotonically from pre-flight validation to post-execution cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OperationPhase {
    /// Step 1: Input validation, syntax checks, permissions, schema inspection.
    Validation = 1,
    /// Step 2: Persistence of credentials, configs, backups, and staging state.
    Staging = 2,
    /// Step 3: Irreversible actions (Point of No Return): killing processes, deleting records.
    IrreversibleExecution = 3,
    /// Step 4: Transient buffer cleanup, temporary lock release, and telemetry.
    Cleanup = 4,
}

impl OperationPhase {
    /// Returns the monotonic ranking level of the operation phase (1..=4).
    pub fn rank(&self) -> u8 {
        *self as u8
    }

    /// Whether this phase precedes any irreversible state mutations.
    pub fn is_pre_execution(&self) -> bool {
        matches!(self, Self::Validation | Self::Staging)
    }

    /// Whether this phase represents the point of no return.
    pub fn is_point_of_no_return(&self) -> bool {
        matches!(self, Self::IrreversibleExecution)
    }

    /// Whether this phase is post-execution cleanup.
    pub fn is_cleanup(&self) -> bool {
        matches!(self, Self::Cleanup)
    }
}

impl fmt::Display for OperationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "Validation"),
            Self::Staging => write!(f, "Staging"),
            Self::IrreversibleExecution => write!(f, "IrreversibleExecution"),
            Self::Cleanup => write!(f, "Cleanup"),
        }
    }
}

/// A discrete step within an operation pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationStep {
    /// Distinct identifier or step name.
    pub name: String,
    /// Lifecycle phase to which this step belongs.
    pub phase: OperationPhase,
    /// Human-readable description of the operation.
    pub description: String,
    /// Whether this step explicitly persists credentials, state, or backups.
    pub is_persistence: bool,
    /// Whether this step represents a point of no return (e.g. kill process, delete data).
    pub is_point_of_no_return: bool,
    /// Target resource affected by this step (e.g. process name, credential file, table).
    pub resource: Option<String>,
}

impl OperationStep {
    /// Creates a generic operation step.
    pub fn new(
        name: impl Into<String>,
        phase: OperationPhase,
        description: impl Into<String>,
    ) -> Self {
        let is_point_of_no_return = phase.is_point_of_no_return();
        Self {
            name: name.into(),
            phase,
            description: description.into(),
            is_persistence: false,
            is_point_of_no_return,
            resource: None,
        }
    }

    /// Creates a pre-flight validation step.
    pub fn validation(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(name, OperationPhase::Validation, description)
    }

    /// Creates a staging step.
    pub fn staging(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(name, OperationPhase::Staging, description)
    }

    /// Creates a credential or state persistence step (Staging phase with is_persistence = true).
    pub fn persistence(name: impl Into<String>, description: impl Into<String>) -> Self {
        let mut step = Self::new(name, OperationPhase::Staging, description);
        step.is_persistence = true;
        step
    }

    /// Creates an irreversible execution step (Point of No Return).
    pub fn irreversible(name: impl Into<String>, description: impl Into<String>) -> Self {
        let mut step = Self::new(name, OperationPhase::IrreversibleExecution, description);
        step.is_point_of_no_return = true;
        step
    }

    /// Creates a post-execution cleanup step.
    pub fn cleanup(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(name, OperationPhase::Cleanup, description)
    }

    /// Sets whether this step persists credentials or state.
    pub fn with_persistence(mut self, persists: bool) -> Self {
        self.is_persistence = persists;
        self
    }

    /// Sets whether this step represents a point of no return.
    pub fn with_point_of_no_return(mut self, irreversible: bool) -> Self {
        self.is_point_of_no_return = irreversible;
        self
    }

    /// Sets the target resource for this step.
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Evaluates whether this step performs persistence (explicit or heuristic).
    pub fn is_persistence_operation(&self) -> bool {
        self.is_persistence
            || (self.phase == OperationPhase::Staging
                && is_persistence_keyword(&self.name, &self.description))
    }

    /// Evaluates whether this step represents an irreversible action (explicit or heuristic).
    pub fn is_irreversible_operation(&self) -> bool {
        self.is_point_of_no_return
            || self.phase == OperationPhase::IrreversibleExecution
            || is_irreversible_keyword(&self.name, &self.description)
    }
}

/// Specific violations detected when evaluating an operation pipeline against AGENTS.md rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectOrderingViolation {
    /// Invariant violation: "write credentials before killing a process".
    /// An irreversible step occurred before state or credential persistence.
    IrreversibleBeforePersistence {
        irreversible_step: String,
        persistence_step: String,
        details: String,
    },
    /// Invariant violation: "validate before deleting".
    /// An irreversible step occurred before validation.
    IrreversibleBeforeValidation {
        irreversible_step: String,
        validation_step: String,
        details: String,
    },
    /// An earlier step has a higher lifecycle phase rank than a subsequent step.
    PhaseOrderRegression {
        prior_step: String,
        prior_phase: OperationPhase,
        subsequent_step: String,
        subsequent_phase: OperationPhase,
        details: String,
    },
    /// An irreversible step is scheduled without any preceding validation step.
    MissingValidationBeforeDestruction {
        irreversible_step: String,
        details: String,
    },
    /// An irreversible step is scheduled without any preceding persistence step.
    MissingPersistenceBeforeIrreversible {
        irreversible_step: String,
        details: String,
    },
    /// The provided operation pipeline is empty.
    EmptyPipeline,
}

impl fmt::Display for SideEffectOrderingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IrreversibleBeforePersistence {
                irreversible_step,
                persistence_step,
                details,
            } => {
                write!(
                    f,
                    "Point of No Return violation: irreversible step '{irreversible_step}' precedes persistence step '{persistence_step}'. {details}"
                )
            }
            Self::IrreversibleBeforeValidation {
                irreversible_step,
                validation_step,
                details,
            } => {
                write!(
                    f,
                    "Point of No Return violation: irreversible step '{irreversible_step}' precedes validation step '{validation_step}'. {details}"
                )
            }
            Self::PhaseOrderRegression {
                prior_step,
                prior_phase,
                subsequent_step,
                subsequent_phase,
                details,
            } => {
                write!(
                    f,
                    "Phase order regression: step '{prior_step}' ({prior_phase}) precedes step '{subsequent_step}' ({subsequent_phase}). {details}"
                )
            }
            Self::MissingValidationBeforeDestruction {
                irreversible_step,
                details,
            } => {
                write!(
                    f,
                    "Missing validation before destruction: irreversible step '{irreversible_step}' has no prior validation. {details}"
                )
            }
            Self::MissingPersistenceBeforeIrreversible {
                irreversible_step,
                details,
            } => {
                write!(
                    f,
                    "Missing persistence before irreversible execution: step '{irreversible_step}' requires prior persistence. {details}"
                )
            }
            Self::EmptyPipeline => {
                write!(f, "Operation pipeline is empty")
            }
        }
    }
}

impl std::error::Error for SideEffectOrderingViolation {}

/// Comprehensive audit report of an operation pipeline sequence.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OperationPipelineReport {
    /// Whether the pipeline strictly complies with side effect ordering invariants.
    pub is_valid: bool,
    /// Total count of steps in the pipeline.
    pub total_steps: usize,
    /// Whether the pipeline contains any irreversible execution (point of no return).
    pub has_irreversible_execution: bool,
    /// The index of the first point of no return step, if present.
    pub point_of_no_return_index: Option<usize>,
    /// All detected ordering violations.
    pub violations: Vec<SideEffectOrderingViolation>,
    /// Human-readable summary of the evaluation.
    pub summary: String,
}

impl OperationPipelineReport {
    /// Converts this report into a standard `Result<(), SideEffectOrderingViolation>`.
    pub fn into_result(self) -> Result<(), SideEffectOrderingViolation> {
        if let Some(first_violation) = self.violations.into_iter().next() {
            Err(first_violation)
        } else {
            Ok(())
        }
    }
}

/// Evaluates whether a name or description indicates credential or state persistence.
pub fn is_persistence_keyword(name: &str, description: &str) -> bool {
    let lower = format!("{} {}", name, description).to_ascii_lowercase();
    lower.contains("credential")
        || lower.contains("auth")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("persist")
        || lower.contains("save")
        || lower.contains("write_cred")
        || lower.contains("write credentials")
        || lower.contains("backup")
        || lower.contains("store")
}

/// Evaluates whether a name or description indicates an irreversible destructive action.
pub fn is_irreversible_keyword(name: &str, description: &str) -> bool {
    let lower = format!("{} {}", name, description).to_ascii_lowercase();
    lower.contains("kill")
        || lower.contains("terminate")
        || lower.contains("delete")
        || lower.contains("destroy")
        || lower.contains("drop")
        || lower.contains("purge")
        || lower.contains("truncate")
        || lower.contains("overwrite")
}

/// Audits an entire pipeline of operational steps for compliance with AGENTS.md ordering invariants.
pub fn audit_operation_pipeline(steps: &[OperationStep]) -> OperationPipelineReport {
    if steps.is_empty() {
        return OperationPipelineReport {
            is_valid: false,
            total_steps: 0,
            has_irreversible_execution: false,
            point_of_no_return_index: None,
            violations: vec![SideEffectOrderingViolation::EmptyPipeline],
            summary: "Pipeline is rejected: no operational steps provided.".to_string(),
        };
    }

    let mut violations = Vec::new();
    let total_steps = steps.len();

    // Identify first point of no return (irreversible step)
    let point_of_no_return_index = steps
        .iter()
        .position(|step| step.is_irreversible_operation());

    let has_irreversible_execution = point_of_no_return_index.is_some();

    // Invariant Check 1: If irreversible execution occurs, verify prior validation.
    if let Some(ponr_idx) = point_of_no_return_index {
        let has_prior_validation = steps[..ponr_idx]
            .iter()
            .any(|s| s.phase == OperationPhase::Validation);

        if !has_prior_validation {
            violations.push(
                SideEffectOrderingViolation::MissingValidationBeforeDestruction {
                    irreversible_step: steps[ponr_idx].name.clone(),
                    details: format!(
                        "Irreversible step '{}' at index {} has no preceding Validation step.",
                        steps[ponr_idx].name, ponr_idx
                    ),
                },
            );
        }

        // Invariant Check 2: Check for persistence or validation scheduled AFTER point of no return.
        for (idx, subsequent_step) in steps.iter().enumerate().skip(ponr_idx + 1) {
            if subsequent_step.is_persistence_operation() {
                violations.push(
                    SideEffectOrderingViolation::IrreversibleBeforePersistence {
                        irreversible_step: steps[ponr_idx].name.clone(),
                        persistence_step: subsequent_step.name.clone(),
                        details: format!(
                            "Irreversible step '{}' (index {}) occurs before persistence step '{}' (index {}). Rule: write credentials before killing a process.",
                            steps[ponr_idx].name, ponr_idx, subsequent_step.name, idx
                        ),
                    },
                );
            }

            if subsequent_step.phase == OperationPhase::Validation {
                violations.push(
                    SideEffectOrderingViolation::IrreversibleBeforeValidation {
                        irreversible_step: steps[ponr_idx].name.clone(),
                        validation_step: subsequent_step.name.clone(),
                        details: format!(
                            "Irreversible step '{}' (index {}) occurs before validation step '{}' (index {}). Rule: validate before deleting.",
                            steps[ponr_idx].name, ponr_idx, subsequent_step.name, idx
                        ),
                    },
                );
            }
        }
    }

    // Invariant Check 3: Check general monotonic phase progression
    for i in 0..steps.len() {
        for j in (i + 1)..steps.len() {
            let prior = &steps[i];
            let subsequent = &steps[j];

            if subsequent.phase.rank() < prior.phase.rank() {
                let violation = SideEffectOrderingViolation::PhaseOrderRegression {
                    prior_step: prior.name.clone(),
                    prior_phase: prior.phase,
                    subsequent_step: subsequent.name.clone(),
                    subsequent_phase: subsequent.phase,
                    details: format!(
                        "Phase {} (step '{}' at index {}) cannot precede Phase {} (step '{}' at index {}).",
                        prior.phase, prior.name, i, subsequent.phase, subsequent.name, j
                    ),
                };

                // Avoid duplicating if already recorded by specific checks
                if !violations.contains(&violation) {
                    violations.push(violation);
                }
            }
        }
    }

    let is_valid = violations.is_empty();
    let summary = if is_valid {
        format!(
            "Pipeline is compliant: {} steps correctly sequenced through point of no return.",
            total_steps
        )
    } else {
        format!(
            "Pipeline rejected: {} side effect ordering violation(s) detected.",
            violations.len()
        )
    };

    OperationPipelineReport {
        is_valid,
        total_steps,
        has_irreversible_execution,
        point_of_no_return_index,
        violations,
        summary,
    }
}

/// Validates an operation pipeline, returning `Ok(())` if fully compliant or `Err(violation)`.
pub fn validate_operation_pipeline(
    steps: &[OperationStep],
) -> Result<(), SideEffectOrderingViolation> {
    audit_operation_pipeline(steps).into_result()
}

/// Alias for `audit_operation_pipeline` providing comprehensive inspection reports.
pub fn inspect_operation_pipeline(steps: &[OperationStep]) -> OperationPipelineReport {
    audit_operation_pipeline(steps)
}

/// Ergonomic builder for constructing strictly validated operation pipelines.
#[derive(Debug, Default, Clone)]
pub struct OperationPipelineBuilder {
    steps: Vec<OperationStep>,
}

impl OperationPipelineBuilder {
    /// Creates a new empty pipeline builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validation step.
    pub fn validate(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.steps.push(OperationStep::validation(name, desc));
        self
    }

    /// Adds a general staging step.
    pub fn stage(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.steps.push(OperationStep::staging(name, desc));
        self
    }

    /// Adds a credential or state persistence step.
    pub fn persist(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.steps.push(OperationStep::persistence(name, desc));
        self
    }

    /// Adds an irreversible execution step (Point of No Return).
    pub fn execute_irreversible(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        self.steps.push(OperationStep::irreversible(name, desc));
        self
    }

    /// Adds a post-execution cleanup step.
    pub fn cleanup(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.steps.push(OperationStep::cleanup(name, desc));
        self
    }

    /// Returns the accumulated slice of steps.
    pub fn build(self) -> Vec<OperationStep> {
        self.steps
    }

    /// Validates the built pipeline against ordering invariants.
    pub fn validate_pipeline(&self) -> Result<(), SideEffectOrderingViolation> {
        validate_operation_pipeline(&self.steps)
    }

    /// Audits the built pipeline and generates a comprehensive report.
    pub fn audit_pipeline(&self) -> OperationPipelineReport {
        audit_operation_pipeline(&self.steps)
    }
}

// ============================================================================
// Planned Operation & Sequence Models (Alternative Schema Support)
// ============================================================================

/// Category of an operation in a multi-stage lifecycle or mutation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationCategory {
    /// Safe preliminary validation, schema verification, or precondition assertion.
    PreValidation,
    /// Persisting credentials, tokens, or configuration to durable storage.
    CredentialPersistence,
    /// Creating backup snapshots or rollback references.
    StateBackup,
    /// Explicit point of no return boundary marker in the pipeline.
    PointOfNoReturnBoundary,
    /// Terminating an active process or service daemon (destructive/irreversible).
    ProcessKill,
    /// Deleting database records or application entities.
    EntityDeletion,
    /// Removing directories, cache volumes, or persistent filesystem files.
    FilesystemPurge,
}

impl OperationCategory {
    /// Returns true if this operation is destructive / irreversible.
    pub fn is_irreversible(&self) -> bool {
        matches!(
            self,
            Self::ProcessKill | Self::EntityDeletion | Self::FilesystemPurge
        )
    }

    /// Returns true if this operation is a safety prerequisite (validation, persistence, backup).
    pub fn is_safety_prerequisite(&self) -> bool {
        matches!(
            self,
            Self::PreValidation | Self::CredentialPersistence | Self::StateBackup
        )
    }
}

/// A planned operation within a sequential execution pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedOperation {
    pub id: String,
    pub name: String,
    pub category: OperationCategory,
    pub is_destructive: bool,
}

impl PlannedOperation {
    /// Creates a new planned operation.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: OperationCategory,
    ) -> Self {
        let is_destructive = category.is_irreversible();
        Self {
            id: id.into(),
            name: name.into(),
            category,
            is_destructive,
        }
    }
}

/// Violations identified when evaluating operation sequencing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderingViolation {
    /// A destructive action occurs before required pre-validation.
    DestructiveActionBeforePreValidation {
        action_name: String,
        action_index: usize,
    },
    /// A process is killed before credentials or config are written to disk.
    ProcessKillBeforeCredentialWrite {
        kill_index: usize,
        write_index: usize,
    },
    /// Deletion occurs before prerequisite validation.
    DeletionBeforeValidation {
        delete_index: usize,
        validate_index: usize,
    },
    /// Destructive operation occurs with no preceding validation anywhere in sequence.
    MissingValidationBeforeIrreversibleOp { op_name: String },
    /// Empty sequence provided for validation.
    EmptyOperationSequence,
}

impl fmt::Display for OrderingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DestructiveActionBeforePreValidation {
                action_name,
                action_index,
            } => {
                write!(
                    f,
                    "Destructive action '{action_name}' at step {action_index} occurs before pre-validation"
                )
            }
            Self::ProcessKillBeforeCredentialWrite {
                kill_index,
                write_index,
            } => {
                write!(
                    f,
                    "Process kill at step {kill_index} occurs before credential persistence at step {write_index}"
                )
            }
            Self::DeletionBeforeValidation {
                delete_index,
                validate_index,
            } => {
                write!(
                    f,
                    "Deletion at step {delete_index} occurs before validation at step {validate_index}"
                )
            }
            Self::MissingValidationBeforeIrreversibleOp { op_name } => {
                write!(
                    f,
                    "Irreversible operation '{op_name}' planned without prior validation"
                )
            }
            Self::EmptyOperationSequence => {
                write!(f, "Operation sequence is empty")
            }
        }
    }
}

impl std::error::Error for OrderingViolation {}

/// Report summarizing compliance of the operation sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderingValidationReport {
    pub is_order_compliant: bool,
    pub point_of_no_return_index: Option<usize>,
    pub pre_boundary_ops_count: usize,
    pub post_boundary_ops_count: usize,
    pub violations: Vec<OrderingViolation>,
    pub summary: String,
}

/// Validates that an operation sequence orders side effects to fail before the point of no return.
pub fn validate_operation_sequence(
    operations: &[PlannedOperation],
) -> Result<OrderingValidationReport, OrderingViolation> {
    if operations.is_empty() {
        return Err(OrderingViolation::EmptyOperationSequence);
    }

    let mut violations = Vec::new();
    let mut has_validation = false;
    let mut first_validation_idx: Option<usize> = None;
    let mut credential_write_indices = Vec::new();
    let mut process_kill_indices = Vec::new();
    let mut deletion_indices = Vec::new();
    let mut point_of_no_return_index: Option<usize> = None;

    for (idx, op) in operations.iter().enumerate() {
        match op.category {
            OperationCategory::PreValidation => {
                if !has_validation {
                    has_validation = true;
                    first_validation_idx = Some(idx);
                }
            }
            OperationCategory::CredentialPersistence => {
                credential_write_indices.push(idx);
            }
            OperationCategory::PointOfNoReturnBoundary => {
                if point_of_no_return_index.is_none() {
                    point_of_no_return_index = Some(idx);
                }
            }
            OperationCategory::ProcessKill => {
                process_kill_indices.push(idx);
            }
            OperationCategory::EntityDeletion | OperationCategory::FilesystemPurge => {
                deletion_indices.push(idx);
            }
            _ => {}
        }
    }

    // Check 1: Any irreversible operation must have prior validation
    for (idx, op) in operations.iter().enumerate() {
        if op.category.is_irreversible() {
            match first_validation_idx {
                None => {
                    violations.push(OrderingViolation::MissingValidationBeforeIrreversibleOp {
                        op_name: op.name.clone(),
                    });
                }
                Some(v_idx) if v_idx > idx => {
                    violations.push(OrderingViolation::DestructiveActionBeforePreValidation {
                        action_name: op.name.clone(),
                        action_index: idx,
                    });
                }
                _ => {}
            }
        }
    }

    // Check 2: Process kills must occur AFTER credential writes
    for &k_idx in &process_kill_indices {
        for &w_idx in &credential_write_indices {
            if k_idx < w_idx {
                violations.push(OrderingViolation::ProcessKillBeforeCredentialWrite {
                    kill_index: k_idx,
                    write_index: w_idx,
                });
            }
        }
    }

    // Check 3: Deletions must occur AFTER validations
    if let Some(v_idx) = first_validation_idx {
        for &d_idx in &deletion_indices {
            if d_idx < v_idx {
                violations.push(OrderingViolation::DeletionBeforeValidation {
                    delete_index: d_idx,
                    validate_index: v_idx,
                });
            }
        }
    }

    let is_order_compliant = violations.is_empty();

    let pre_count = match point_of_no_return_index {
        Some(idx) => idx,
        None => operations.len(),
    };
    let post_count = operations.len().saturating_sub(pre_count);

    let summary = if is_order_compliant {
        format!(
            "Side effect ordering compliant: {} operations safely ordered around point of no return",
            operations.len()
        )
    } else {
        format!(
            "Side effect ordering failed with {} violation(s)",
            violations.len()
        )
    };

    let report = OrderingValidationReport {
        is_order_compliant,
        point_of_no_return_index,
        pre_boundary_ops_count: pre_count,
        post_boundary_ops_count: post_count,
        violations,
        summary,
    };

    if report.is_order_compliant {
        Ok(report)
    } else {
        Err(report.violations[0].clone())
    }
}
