//! Local Commit Convergence & Noise Elimination Auditor.
//!
//! Enforces AGENTS.md PR Scope & History Hygiene invariant:
//! - Commit freely during local debugging on development branches.
//! - Before opening or merging a PR, audit and consolidate scattered iterative attempts into clean, high-quality units.
//! - Each consolidated commit must describe only its successful final state and rationale, eliminating intermediate trial-and-error noise.

use serde::{Deserialize, Serialize};

/// Representation of a Git commit entry in the branch history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitEntry {
    /// Commit SHA hash.
    pub hash: String,
    /// Git commit message subject & body.
    pub message: String,
    /// Commit author name or handle.
    pub author: String,
    /// List of file paths changed in this commit.
    pub files_changed: Vec<String>,
}

/// Noise classification for a commit in the branch history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseClassification {
    /// Consolidated commit describing clean final state and rationale.
    CleanFinalState,
    /// Intermediate trial-and-error noise commit (e.g. "try fix", "undo test").
    TrialAndErrorNoise(String),
    /// Work-in-progress checkpoint commit (e.g. "wip", "checkpoint").
    WipCheckpoint,
    /// Scattered iterative micro-fixes on identical files across the sequence.
    ScatteredFixes(usize),
}

/// Comprehensive report on commit history convergence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitConvergenceReport {
    /// Whether all audited commits represent converged, clean final states.
    pub is_converged: bool,
    /// Total number of commits inspected.
    pub total_commits: usize,
    /// Count of commits identified as intermediate noise or unconsolidated scattered fixes.
    pub noise_commit_count: usize,
    /// Flagged commit tuples: (commit_hash, commit_message, noise_classification).
    pub flagged_commits: Vec<(String, String, NoiseClassification)>,
    /// Actionable guidance for the developer.
    pub suggested_action: String,
}

/// Known WIP and checkpoint keywords that mark intermediate development checkpoints.
const WIP_KEYWORDS: &[&str] = &[
    "wip",
    "wip2",
    "wip3",
    "checkpoint",
    "check point",
    "work in progress",
];

/// Known trial-and-error noise keywords and phrases specified in AGENTS.md.
const NOISE_PHRASES: &[&str] = &[
    "try fix",
    "trying fix",
    "attempt fix",
    "temp",
    "temp fix",
    "temporary",
    "undo test",
    "undo change",
    "fixed typo",
    "fix typo",
    "typo fix",
    "revert previous",
    "revert test",
    "test again",
    "testing again",
    "try again",
    "fix fix",
    "oops",
    "oopsie",
];

/// Checks whether a message contains a specific standalone word or delimiter-separated token.
fn contains_isolated_token(message: &str, token: &str) -> bool {
    let lower_msg = message.to_ascii_lowercase();
    let lower_token = token.to_ascii_lowercase();

    // Check for tokens separated by non-alphanumeric characters
    for part in lower_msg.split(|c: char| !c.is_ascii_alphanumeric()) {
        if part == lower_token {
            return true;
        }
    }
    false
}

/// Classifies a single commit against WIP, trial-and-error noise, and scattered fix patterns.
pub fn classify_commit(commit: &CommitEntry, all_commits: &[CommitEntry]) -> NoiseClassification {
    let msg_lower = commit.message.trim().to_ascii_lowercase();

    // 1. Check for WIP & Checkpoints
    for &wip_kw in WIP_KEYWORDS {
        if msg_lower.starts_with(wip_kw)
            || msg_lower.starts_with(&format!("{}:", wip_kw))
            || msg_lower.starts_with(&format!("{}-", wip_kw))
            || msg_lower.starts_with(&format!("[{}]", wip_kw))
            || contains_isolated_token(&msg_lower, wip_kw)
        {
            return NoiseClassification::WipCheckpoint;
        }
    }

    // 2. Check for explicit Trial-and-Error noise phrases
    for &phrase in NOISE_PHRASES {
        if phrase.contains(' ') {
            if msg_lower.contains(phrase) {
                return NoiseClassification::TrialAndErrorNoise(phrase.to_string());
            }
        } else if contains_isolated_token(&msg_lower, phrase) || msg_lower.starts_with(phrase) {
            return NoiseClassification::TrialAndErrorNoise(phrase.to_string());
        }
    }

    // 3. Check for ScatteredFixes:
    // If commit message is a generic micro-fix (e.g. "fix", "tweak", "update", "patch", "adjust")
    // and shares modified files with other micro-fix commits in the sequence.
    let is_generic_fix = is_generic_micro_fix(&msg_lower);
    if is_generic_fix && !commit.files_changed.is_empty() {
        let mut matching_scattered_commits = 0;
        for other in all_commits {
            let other_lower = other.message.trim().to_ascii_lowercase();
            if is_generic_micro_fix(&other_lower) {
                let shares_file = commit
                    .files_changed
                    .iter()
                    .any(|f| other.files_changed.contains(f));
                if shares_file {
                    matching_scattered_commits += 1;
                }
            }
        }

        if matching_scattered_commits >= 2 {
            return NoiseClassification::ScatteredFixes(matching_scattered_commits);
        }
    }

    NoiseClassification::CleanFinalState
}

/// Determines whether a commit message represents a vague, unconsolidated micro-fix without rationale.
fn is_generic_micro_fix(msg_lower: &str) -> bool {
    let trimmed = msg_lower.trim();
    if trimmed.len() <= 20 {
        let generic_stems = &[
            "fix",
            "tweak",
            "update",
            "patch",
            "adjust",
            "change",
            "styling",
            "small fix",
            "quick fix",
        ];
        for &stem in generic_stems {
            if trimmed == stem || trimmed.starts_with(stem) {
                return true;
            }
        }
    }
    false
}

/// Audits a slice of `CommitEntry` records to verify branch convergence per AGENTS.md.
pub fn audit_commit_history(commits: &[CommitEntry]) -> CommitConvergenceReport {
    if commits.is_empty() {
        return CommitConvergenceReport {
            is_converged: true,
            total_commits: 0,
            noise_commit_count: 0,
            flagged_commits: Vec::new(),
            suggested_action: "No commits to audit; working tree is clean.".to_string(),
        };
    }

    let mut flagged_commits = Vec::new();

    for commit in commits {
        let classification = classify_commit(commit, commits);
        if classification != NoiseClassification::CleanFinalState {
            flagged_commits.push((commit.hash.clone(), commit.message.clone(), classification));
        }
    }

    let noise_commit_count = flagged_commits.len();
    let is_converged = noise_commit_count == 0;

    let suggested_action = if is_converged {
        "All commits are converged into clean, high-quality units with rationale. Ready for PR and merge.".to_string()
    } else {
        format!(
            "Found {} unconsolidated noise/scattered commit(s). Consolidate intermediate commits using 'git rebase -i' or GitMap commit consolidation before opening or merging a PR. Each commit must represent a single, self-contained functional unit describing only its final state and rationale.",
            noise_commit_count
        )
    };

    CommitConvergenceReport {
        is_converged,
        total_commits: commits.len(),
        noise_commit_count,
        flagged_commits,
        suggested_action,
    }
}
