//! Published-Tip Rewrite Guard.
//!
//! Enforces Branch & History Hygiene per AGENTS.md:
//! "Inspect in-flight PRs (`gh pr list --base main`) before rewriting published tips,
//! avoiding force-pushes across shared branches. Retain local rollback refs (`backup/*`)
//! before history rewrites, confirming zero content drift via `git diff --stat <backup> HEAD`."

use serde::{Deserialize, Serialize};

/// Status of content drift verification between backup ref and HEAD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DriftCheckStatus {
    /// Zero drift confirmed: diff stat between backup ref and HEAD is empty.
    ZeroDriftConfirmed,
    /// Content drift detected with list of differing files.
    DriftDetected { differing_files: Vec<String> },
    /// Drift verification was not executed prior to the rewrite operation.
    NotChecked,
}

impl Default for DriftCheckStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// Precondition violations blocking a published tip history rewrite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "violation_type", rename_all = "snake_case")]
pub enum RewritePreconditionViolation {
    /// Target branch is published and has active in-flight pull requests.
    PublishedTipWithOpenPrs {
        branch: String,
        open_pr_count: usize,
        pr_numbers: Vec<u64>,
    },
    /// Local rollback reference matching the required pattern is missing.
    MissingRollbackBackupRef { expected_ref_pattern: String },
    /// Drift check was not performed before attempting rewrite.
    DriftCheckNotPerformed,
    /// Non-zero content drift detected between backup ref and HEAD.
    ContentDriftDetected { differing_files_count: usize },
}

impl RewritePreconditionViolation {
    /// Human-readable explanation of the precondition violation.
    pub fn description(&self) -> String {
        match self {
            Self::PublishedTipWithOpenPrs {
                branch,
                open_pr_count,
                pr_numbers,
            } => {
                format!(
                    "Branch '{}' is a published tip with {} in-flight open PR(s) ({:?}). Force-pushing will disrupt active collaborator branches.",
                    branch, open_pr_count, pr_numbers
                )
            }
            Self::MissingRollbackBackupRef {
                expected_ref_pattern,
            } => {
                format!(
                    "Missing local rollback ref matching '{}'. A pre-rewrite backup ref is required for safety.",
                    expected_ref_pattern
                )
            }
            Self::DriftCheckNotPerformed => {
                "Zero content drift check was not performed against backup ref via `git diff --stat <backup> HEAD`.".to_string()
            }
            Self::ContentDriftDetected {
                differing_files_count,
            } => {
                format!(
                    "Content drift detected: {} differing file(s) between backup ref and HEAD.",
                    differing_files_count
                )
            }
        }
    }
}

/// Parameters defining a proposed history rewrite operation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RewriteOperation {
    pub target_branch: String,
    pub is_published_tip: bool,
    pub open_prs_count: usize,
    pub open_pr_numbers: Vec<u64>,
    pub has_backup_rollback_ref: bool,
    pub backup_ref_name: Option<String>,
    pub drift_status: DriftCheckStatus,
}

impl RewriteOperation {
    /// Creates a new rewrite operation.
    pub fn new(target_branch: impl Into<String>, is_published_tip: bool) -> Self {
        Self {
            target_branch: target_branch.into(),
            is_published_tip,
            ..Default::default()
        }
    }

    /// Convenience constructor for unpublished local branch.
    pub fn local(target_branch: impl Into<String>) -> Self {
        Self::new(target_branch, false)
    }

    /// Convenience constructor for published tip branch.
    pub fn published(target_branch: impl Into<String>) -> Self {
        Self::new(target_branch, true)
    }

    /// Configures detected open pull requests.
    pub fn with_open_prs(mut self, pr_numbers: Vec<u64>) -> Self {
        self.open_prs_count = pr_numbers.len();
        self.open_pr_numbers = pr_numbers;
        self
    }

    /// Configures presence of local rollback backup reference.
    pub fn with_backup_ref(mut self, backup_ref: impl Into<String>) -> Self {
        self.has_backup_rollback_ref = true;
        self.backup_ref_name = Some(backup_ref.into());
        self
    }

    /// Sets the drift check status.
    pub fn with_drift_status(mut self, status: DriftCheckStatus) -> Self {
        self.drift_status = status;
        self
    }
}

/// Verdict on whether a proposed rewrite is safe to proceed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteVerdict {
    pub is_allowed: bool,
    pub violations: Vec<RewritePreconditionViolation>,
    pub guidance: String,
}

impl RewriteVerdict {
    /// Whether any safety preconditions were violated.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// Evaluates whether a Git history rewrite operation is safe per AGENTS.md branch hygiene rules.
pub fn evaluate_rewrite_safety(operation: &RewriteOperation) -> RewriteVerdict {
    if !operation.is_published_tip {
        return build_allowed_unpublished_verdict(&operation.target_branch);
    }

    let violations = collect_rewrite_violations(operation);
    let is_allowed = violations.is_empty();
    let guidance = generate_rewrite_guidance(&operation.target_branch, is_allowed, &violations);

    RewriteVerdict {
        is_allowed,
        violations,
        guidance,
    }
}

fn build_allowed_unpublished_verdict(branch: &str) -> RewriteVerdict {
    RewriteVerdict {
        is_allowed: true,
        violations: Vec::new(),
        guidance: format!(
            "Branch '{}' is an unpublished local branch. History rewrite is permitted without remote conflict risk.",
            branch
        ),
    }
}

fn collect_rewrite_violations(op: &RewriteOperation) -> Vec<RewritePreconditionViolation> {
    let mut violations = Vec::new();

    if op.open_prs_count > 0 {
        violations.push(RewritePreconditionViolation::PublishedTipWithOpenPrs {
            branch: op.target_branch.clone(),
            open_pr_count: op.open_prs_count,
            pr_numbers: op.open_pr_numbers.clone(),
        });
    }

    if !op.has_backup_rollback_ref {
        violations.push(RewritePreconditionViolation::MissingRollbackBackupRef {
            expected_ref_pattern: "backup/*".to_string(),
        });
    }

    match &op.drift_status {
        DriftCheckStatus::NotChecked => {
            violations.push(RewritePreconditionViolation::DriftCheckNotPerformed);
        }
        DriftCheckStatus::DriftDetected { differing_files } => {
            violations.push(RewritePreconditionViolation::ContentDriftDetected {
                differing_files_count: differing_files.len(),
            });
        }
        DriftCheckStatus::ZeroDriftConfirmed => {}
    }

    violations
}

fn generate_rewrite_guidance(
    branch: &str,
    is_allowed: bool,
    violations: &[RewritePreconditionViolation],
) -> String {
    if is_allowed {
        return format!(
            "Published tip rewrite for '{}' is approved: no in-flight PRs detected, backup ref retained, and zero content drift confirmed.",
            branch
        );
    }

    let reasons = violations
        .iter()
        .map(|v| format!("- {}", v.description()))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Published tip rewrite for '{}' is blocked by safety preconditions:\n{}\nRemedy: Check in-flight PRs via `gh pr list --base main`, create a rollback ref via `git branch backup/<name>`, and verify zero drift via `git diff --stat <backup> HEAD`.",
        branch, reasons
    )
}
