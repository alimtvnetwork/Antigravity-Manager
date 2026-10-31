//! Rollback Reference & History Drift Verifier.
//!
//! Enforces AGENTS.md branch and history hygiene:
//! Inspect in-flight PRs (`gh pr list --base main`) before rewriting published tips,
//! avoiding force-pushes across shared branches. Retain local rollback refs (`backup/*`)
//! before history rewrites, confirming zero content drift via `git diff --stat <backup> HEAD`.

use serde::{Deserialize, Serialize};

/// Violations identified when verifying history rewrite safety.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackRefViolation {
    MissingBackupReference {
        expected_prefix: String,
        target_branch: String,
    },
    ContentDriftDetected {
        diff_stat: String,
        files_changed: usize,
        insertions: usize,
        deletions: usize,
    },
    InFlightPrsExist {
        target_branch: String,
        open_prs: Vec<String>,
    },
}

impl std::fmt::Display for RollbackRefViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBackupReference {
                expected_prefix,
                target_branch,
            } => {
                write!(
                    f,
                    "Missing backup reference with prefix '{}' for branch '{}'",
                    expected_prefix, target_branch
                )
            }
            Self::ContentDriftDetected {
                diff_stat,
                files_changed,
                insertions,
                deletions,
            } => {
                write!(
                    f,
                    "Content drift detected: {} files changed, +{} -{} ({})",
                    files_changed, insertions, deletions, diff_stat
                )
            }
            Self::InFlightPrsExist {
                target_branch,
                open_prs,
            } => {
                write!(
                    f,
                    "In-flight PRs exist for branch '{}': {:?}",
                    target_branch, open_prs
                )
            }
        }
    }
}

impl std::error::Error for RollbackRefViolation {}

/// Summary of parsed `git diff --stat` output.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DiffStatSummary {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub is_zero_drift: bool,
}

/// Audit report evaluating the safety of a Git history rewrite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRewriteAuditReport {
    pub is_safe: bool,
    pub target_branch: String,
    pub backup_ref: Option<String>,
    pub diff_stat: DiffStatSummary,
    pub open_pr_count: usize,
    pub violations: Vec<RollbackRefViolation>,
}

/// Extracts the first integer token from a comma-separated diff stat segment.
fn parse_stat_token(token: &str) -> usize {
    token
        .split_whitespace()
        .next()
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Parses a summary line from `git diff --stat`.
fn parse_summary_line(line: &str) -> (usize, usize, usize) {
    let mut files = 0;
    let mut ins = 0;
    let mut dels = 0;

    for part in line.split(',') {
        let p = part.trim();
        if p.contains("file") {
            files = parse_stat_token(p);
        } else if p.contains("insertion") {
            ins = parse_stat_token(p);
        } else if p.contains("deletion") {
            dels = parse_stat_token(p);
        }
    }

    (files, ins, dels)
}

/// Parses output from `git diff --stat` to detect content changes.
pub fn parse_git_diff_stat(stat_output: &str) -> DiffStatSummary {
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in stat_output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("file changed") || trimmed.contains("files changed") {
            let (f, i, d) = parse_summary_line(trimmed);
            files_changed = f;
            insertions = i;
            deletions = d;
        }
    }

    let is_zero_drift = files_changed == 0 && insertions == 0 && deletions == 0;

    DiffStatSummary {
        files_changed,
        insertions,
        deletions,
        is_zero_drift,
    }
}

/// Locates a matching local backup reference starting with `backup/` or `refs/heads/backup/`.
fn find_backup_reference(existing_refs: &[String]) -> Option<String> {
    existing_refs
        .iter()
        .find(|r| {
            let trimmed = r.trim();
            trimmed.starts_with("backup/") || trimmed.starts_with("refs/heads/backup/")
        })
        .cloned()
}

/// Evaluates whether a history rewrite operation on a branch is safe.
pub fn verify_history_rewrite_safety(
    target_branch: &str,
    existing_refs: &[String],
    diff_stat_raw: &str,
    open_prs: &[String],
) -> HistoryRewriteAuditReport {
    let mut violations = Vec::new();

    let backup_ref = find_backup_reference(existing_refs);
    if backup_ref.is_none() {
        violations.push(RollbackRefViolation::MissingBackupReference {
            expected_prefix: "backup/".to_string(),
            target_branch: target_branch.to_string(),
        });
    }

    let diff_stat = parse_git_diff_stat(diff_stat_raw);
    if !diff_stat.is_zero_drift {
        violations.push(RollbackRefViolation::ContentDriftDetected {
            diff_stat: diff_stat_raw.trim().to_string(),
            files_changed: diff_stat.files_changed,
            insertions: diff_stat.insertions,
            deletions: diff_stat.deletions,
        });
    }

    if !open_prs.is_empty() {
        violations.push(RollbackRefViolation::InFlightPrsExist {
            target_branch: target_branch.to_string(),
            open_prs: open_prs.to_vec(),
        });
    }

    HistoryRewriteAuditReport {
        is_safe: violations.is_empty(),
        target_branch: target_branch.to_string(),
        backup_ref,
        diff_stat,
        open_pr_count: open_prs.len(),
        violations,
    }
}
