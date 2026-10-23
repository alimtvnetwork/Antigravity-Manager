//! Individually Revertable Commit Guard (`Imp-684`).
//!
//! Enforces AGENTS.md PR Scope & Grouping invariant:
//! "Self-Contained & Individually Revertable: A PR may contain multiple commits, but each
//! commit must represent an independent, self-contained functional unit that is individually
//! revertable, avoiding messy or tangled changesets."
//!
//! Audits commit series in pull requests and feature branches to ensure each commit is
//! cohesive, describes a distinct verifiable unit of work, and eliminates intermediate
//! trial-and-error noise that would obstruct bisecting or clean git reverts.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard commit prefixes indicating well-structured functional changes.
pub const VALID_COMMIT_PREFIXES: &[&str] = &[
    "Feature:",
    "Fix:",
    "Refactor:",
    "Chore:",
    "Docs:",
    "Test:",
    "Perf:",
    "CI:",
];

/// Keywords indicating intermediate trial-and-error noise that should be squashed.
pub const NOISE_KEYWORDS: &[&str] = &[
    "wip",
    "fix fix",
    "temp",
    "try fix",
    "undo",
    "oops",
    "revert revert",
    "test again",
    "another fix",
    "fixed typo",
    "checkpoint",
];

/// Metadata describing a single git commit within a pull request or branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMetadata {
    /// Commit hash or identifier.
    pub commit_id: String,
    /// Full commit message subject line.
    pub message: String,
    /// List of filepaths modified by this commit.
    pub touched_files: Vec<String>,
}

/// Violations indicating an un-revertable or noisy commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitQualityViolation {
    /// Commit message is empty, single-word, or too brief (< 10 chars).
    VagueOrEmptyMessage {
        commit_id: String,
        message: String,
    },
    /// Commit lacks standard prefix (e.g. `Feature:`, `Fix:`).
    MissingStandardPrefix {
        commit_id: String,
        message: String,
    },
    /// Commit represents iterative debugging noise rather than a consolidated final state.
    TrialAndErrorNoise {
        commit_id: String,
        keyword: String,
    },
    /// Commit touches excessive unrelated files without cohesive focus.
    ExcessiveFileScope {
        commit_id: String,
        file_count: usize,
    },
}

impl fmt::Display for CommitQualityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VagueOrEmptyMessage { commit_id, message } => {
                write!(f, "Commit {commit_id} has vague message: '{message}'")
            }
            Self::MissingStandardPrefix { commit_id, message } => {
                write!(f, "Commit {commit_id} lacks standard prefix: '{message}'")
            }
            Self::TrialAndErrorNoise { commit_id, keyword } => {
                write!(f, "Commit {commit_id} contains trial-and-error noise keyword '{keyword}'")
            }
            Self::ExcessiveFileScope { commit_id, file_count } => {
                write!(f, "Commit {commit_id} touches an excessive {file_count} files")
            }
        }
    }
}

/// Individual audit result for a single commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAuditRecord {
    /// Evaluated commit ID.
    pub commit_id: String,
    /// True if the commit is self-contained and individually revertable.
    pub is_revertable: bool,
    /// Violations found for this commit.
    pub violations: Vec<CommitQualityViolation>,
}

/// Overall report produced after auditing a commit series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevertableCommitReport {
    /// True if all commits in the series are individually revertable and clean.
    pub is_compliant: bool,
    /// Total count of commits evaluated.
    pub total_commits_count: usize,
    /// Count of commits passing all revertability checks.
    pub compliant_commits_count: usize,
    /// Granular audit records per commit.
    pub records: Vec<CommitAuditRecord>,
}

/// Evaluates a single commit for self-contained revertability.
pub fn evaluate_commit(commit: &CommitMetadata) -> CommitAuditRecord {
    let mut violations = Vec::new();
    let trimmed_msg = commit.message.trim();

    // 1. Length check
    if trimmed_msg.len() < 10 {
        violations.push(CommitQualityViolation::VagueOrEmptyMessage {
            commit_id: commit.commit_id.clone(),
            message: commit.message.clone(),
        });
    }

    // 2. Prefix check
    let has_valid_prefix = VALID_COMMIT_PREFIXES
        .iter()
        .any(|p| trimmed_msg.starts_with(p));
    if !has_valid_prefix {
        violations.push(CommitQualityViolation::MissingStandardPrefix {
            commit_id: commit.commit_id.clone(),
            message: commit.message.clone(),
        });
    }

    // 3. Noise keyword check
    let lower_msg = trimmed_msg.to_ascii_lowercase();
    for kw in NOISE_KEYWORDS {
        if lower_msg.contains(kw) {
            violations.push(CommitQualityViolation::TrialAndErrorNoise {
                commit_id: commit.commit_id.clone(),
                keyword: kw.to_string(),
            });
            break;
        }
    }

    // 4. File scope cap (unfocused commit touching > 50 files)
    if commit.touched_files.len() > 50 {
        violations.push(CommitQualityViolation::ExcessiveFileScope {
            commit_id: commit.commit_id.clone(),
            file_count: commit.touched_files.len(),
        });
    }

    let is_revertable = violations.is_empty();
    CommitAuditRecord {
        commit_id: commit.commit_id.clone(),
        is_revertable,
        violations,
    }
}

/// Audits a sequence of commits forming a pull request or branch.
pub fn audit_commit_series(commits: &[CommitMetadata]) -> RevertableCommitReport {
    let mut records = Vec::with_capacity(commits.len());
    let mut compliant_count = 0;

    for commit in commits {
        let record = evaluate_commit(commit);
        if record.is_revertable {
            compliant_count += 1;
        }
        records.push(record);
    }

    let is_compliant = compliant_count == commits.len();
    RevertableCommitReport {
        is_compliant,
        total_commits_count: commits.len(),
        compliant_commits_count: compliant_count,
        records,
    }
}
