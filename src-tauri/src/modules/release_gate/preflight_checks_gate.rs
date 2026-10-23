//! Preflight Checks Gate (`Imp-640`).
//!
//! Enforces pre-flight quality gates per AGENTS.md:
//! "Pre-flight Checks: Run the essentials before submitting PRs or release tags:
//! - `cd src-tauri && cargo fmt -- --check` (for Rust edits)
//! - `cd src-tauri && cargo clippy --all-targets --all-features` (comprehensive Rust gate, already includes compilation — no separate `cargo check` needed)
//! - `npm run build` (when `src/` or frontend configs changed)"

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of preflight check steps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreflightCheckKind {
    CargoFmtCheck,
    CargoClippyAllFeatures,
    NpmRunBuild,
    CustomStep(String),
}

impl PreflightCheckKind {
    /// Returns the canonical CLI command represented by this check.
    pub fn command_str(&self) -> &str {
        match self {
            Self::CargoFmtCheck => "cd src-tauri && cargo fmt -- --check",
            Self::CargoClippyAllFeatures => {
                "cd src-tauri && cargo clippy --all-targets --all-features"
            }
            Self::NpmRunBuild => "npm run build",
            Self::CustomStep(cmd) => cmd.as_str(),
        }
    }
}

impl fmt::Display for PreflightCheckKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CargoFmtCheck => write!(f, "cargo fmt -- --check"),
            Self::CargoClippyAllFeatures => write!(f, "cargo clippy --all-targets --all-features"),
            Self::NpmRunBuild => write!(f, "npm run build"),
            Self::CustomStep(step) => write!(f, "custom: {}", step),
        }
    }
}

/// Execution status of a preflight check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightCheckStatus {
    Passed,
    Failed { error: String },
    Skipped { reason: String },
}

impl PreflightCheckStatus {
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped { .. })
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Failed { error } => Some(error.as_str()),
            _ => None,
        }
    }

    pub fn skip_reason(&self) -> Option<&str> {
        match self {
            Self::Skipped { reason } => Some(reason.as_str()),
            _ => None,
        }
    }
}

/// Individual preflight check result entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightCheckResult {
    pub check_kind: PreflightCheckKind,
    pub status: PreflightCheckStatus,
    pub duration_ms: u64,
    pub is_blocking: bool,
}

impl PreflightCheckResult {
    pub fn new(
        check_kind: PreflightCheckKind,
        status: PreflightCheckStatus,
        duration_ms: u64,
        is_blocking: bool,
    ) -> Self {
        Self {
            check_kind,
            status,
            duration_ms,
            is_blocking,
        }
    }

    pub fn passed(check_kind: PreflightCheckKind, duration_ms: u64, is_blocking: bool) -> Self {
        Self::new(
            check_kind,
            PreflightCheckStatus::Passed,
            duration_ms,
            is_blocking,
        )
    }

    pub fn failed(
        check_kind: PreflightCheckKind,
        error: impl Into<String>,
        duration_ms: u64,
        is_blocking: bool,
    ) -> Self {
        Self::new(
            check_kind,
            PreflightCheckStatus::Failed {
                error: error.into(),
            },
            duration_ms,
            is_blocking,
        )
    }

    pub fn skipped(
        check_kind: PreflightCheckKind,
        reason: impl Into<String>,
        is_blocking: bool,
    ) -> Self {
        Self::new(
            check_kind,
            PreflightCheckStatus::Skipped {
                reason: reason.into(),
            },
            0,
            is_blocking,
        )
    }

    pub fn is_passed(&self) -> bool {
        self.status.is_passed()
    }

    pub fn is_failed(&self) -> bool {
        self.status.is_failed()
    }

    pub fn is_skipped(&self) -> bool {
        self.status.is_skipped()
    }
}

/// Comprehensive preflight evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub is_ready_for_release: bool,
    pub total_checks: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<PreflightCheckResult>,
}

impl PreflightReport {
    pub fn has_failures(&self) -> bool {
        self.failed_count > 0
    }

    pub fn has_blocking_failures(&self) -> bool {
        self.results.iter().any(|r| r.is_blocking && r.is_failed())
    }

    pub fn skipped_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_skipped()).count()
    }
}

/// Evaluates a list of preflight check results into a consolidated release report.
///
/// Fails (`is_ready_for_release = false`) if any blocking check has status `Failed`.
/// Passes if all blocking checks have status `Passed` or `Skipped`.
pub fn evaluate_preflight_results(results: &[PreflightCheckResult]) -> PreflightReport {
    let total_checks = results.len();
    let passed_count = results.iter().filter(|r| r.is_passed()).count();
    let failed_count = results.iter().filter(|r| r.is_failed()).count();

    let has_blocking_failure = results.iter().any(|r| r.is_blocking && r.is_failed());

    PreflightReport {
        is_ready_for_release: !has_blocking_failure,
        total_checks,
        passed_count,
        failed_count,
        results: results.to_vec(),
    }
}

/// Formats a human-readable summary of preflight check results.
pub fn format_preflight_summary(report: &PreflightReport) -> String {
    let readiness_tag = if report.is_ready_for_release {
        "READY FOR RELEASE"
    } else {
        "BLOCKED FROM RELEASE"
    };

    let mut lines = Vec::new();
    lines.push(format!(
        "=== Preflight Checks Report [{}] ===",
        readiness_tag
    ));
    lines.push(format!(
        "Summary: Total: {}, Passed: {}, Failed: {}, Skipped: {}",
        report.total_checks,
        report.passed_count,
        report.failed_count,
        report.skipped_count()
    ));

    for result in &report.results {
        let block_label = if result.is_blocking {
            "BLOCKING"
        } else {
            "OPTIONAL"
        };
        let status_desc = match &result.status {
            PreflightCheckStatus::Passed => format!("PASSED ({} ms)", result.duration_ms),
            PreflightCheckStatus::Failed { error } => {
                format!("FAILED: {} ({} ms)", error, result.duration_ms)
            }
            PreflightCheckStatus::Skipped { reason } => format!("SKIPPED: {}", reason),
        };
        lines.push(format!(
            "  - [{}] {} -> {}",
            block_label, result.check_kind, status_desc
        ));
    }

    if !report.is_ready_for_release {
        lines.push(
            "Verdict: Pre-flight checks failed on one or more blocking checks. Fix violations before tagging or submitting PR.".to_string(),
        );
    } else if report.has_failures() {
        lines.push(
            "Verdict: Passed with warnings. All blocking checks satisfied, but non-blocking check failures detected.".to_string(),
        );
    } else {
        lines.push("Verdict: All pre-flight checks passed successfully.".to_string());
    }

    lines.join("\n")
}
