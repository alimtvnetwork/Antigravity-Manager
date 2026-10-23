//! Prefix Stability Pipeline Guard (`Imp-200`)
//!
//! Enforces AGENTS.md invariant:
//! "The pipeline stage uniformly handles the converted Gemini payloads,
//! including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability, and the sanitization
//! of risky prompts and request headers."
//!
//! Validates conversational prefix continuity and cache friendliness across dialog turns,
//! detecting system prompt mutation, turn re-ordering, retrospective content edits,
//! and unexpected turn history truncation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Maximum character length for dialog turn snippet previews.
pub const MAX_SNIPPET_PREVIEW_CHARS: usize = 60;

/// Snapshot of a single conversational dialog turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogTurn {
    pub turn_index: usize,
    pub role: String,
    pub content_hash: String,
    pub content_snippet: String,
}

impl DialogTurn {
    /// Constructs a dialog turn with explicit field values.
    pub fn new(
        turn_index: usize,
        role: impl Into<String>,
        content_hash: impl Into<String>,
        content_snippet: impl Into<String>,
    ) -> Self {
        Self {
            turn_index,
            role: role.into(),
            content_hash: content_hash.into(),
            content_snippet: content_snippet.into(),
        }
    }

    /// Constructs a dialog turn by hashing content and generating a snippet.
    pub fn from_role_and_content(turn_index: usize, role: &str, content: &str) -> Self {
        create_dialog_turn(turn_index, role, content)
    }
}

/// Violations representing drift or disruption in prefix stability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrefixStabilityViolation {
    SystemPromptMutated {
        initial_hash: String,
        mutated_hash: String,
    },
    TurnOrderAltered {
        turn_index: usize,
        expected_role: String,
        actual_role: String,
    },
    EarlierTurnContentMutated {
        turn_index: usize,
        snippet: String,
    },
    TurnHistoryTruncated {
        baseline_turns: usize,
        new_turns: usize,
    },
}

impl fmt::Display for PrefixStabilityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemPromptMutated {
                initial_hash,
                mutated_hash,
            } => {
                write!(
                    f,
                    "System prompt mutated: initial hash {} != mutated hash {}",
                    initial_hash, mutated_hash
                )
            }
            Self::TurnOrderAltered {
                turn_index,
                expected_role,
                actual_role,
            } => {
                write!(
                    f,
                    "Turn order altered at turn {}: expected role '{}', actual role '{}'",
                    turn_index, expected_role, actual_role
                )
            }
            Self::EarlierTurnContentMutated {
                turn_index,
                snippet,
            } => {
                write!(
                    f,
                    "Earlier turn content mutated at turn {}: '{}'",
                    turn_index, snippet
                )
            }
            Self::TurnHistoryTruncated {
                baseline_turns,
                new_turns,
            } => {
                write!(
                    f,
                    "Turn history truncated: baseline has {} turns, new history has {} turns",
                    baseline_turns, new_turns
                )
            }
        }
    }
}

impl std::error::Error for PrefixStabilityViolation {}

/// Comprehensive outcome report of prefix stability verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixStabilityReport {
    pub is_stable: bool,
    pub prefix_turns_preserved: usize,
    pub total_new_turns: usize,
    pub violations: Vec<PrefixStabilityViolation>,
}

impl PrefixStabilityReport {
    /// Returns true if no violations were encountered and the prefix is stable.
    pub fn is_stable(&self) -> bool {
        self.is_stable
    }

    /// Returns true if one or more prefix stability violations were detected.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Formats all detected violations into a concatenated summary string.
    pub fn violation_summary(&self) -> String {
        self.violations
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Computes a deterministic SHA-256 hex digest for a role and content pair.
pub fn compute_turn_hash(role: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(role.as_bytes());
    hasher.update(b":");
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Truncates string content safely to UTF-8 character boundaries.
fn truncate_preview_snippet(content: &str, max_chars: usize) -> String {
    content.chars().take(max_chars).collect()
}

/// Creates a `DialogTurn` with computed SHA-256 hash and truncated snippet preview.
pub fn create_dialog_turn(turn_index: usize, role: &str, content: &str) -> DialogTurn {
    let content_hash = compute_turn_hash(role, content);
    let content_snippet = truncate_preview_snippet(content, MAX_SNIPPET_PREVIEW_CHARS);
    DialogTurn {
        turn_index,
        role: role.to_string(),
        content_hash,
        content_snippet,
    }
}

/// Audits role alignment between baseline and incoming turns.
fn check_role_alignment(
    turn_index: usize,
    expected_role: &str,
    actual_role: &str,
) -> Option<PrefixStabilityViolation> {
    if expected_role != actual_role {
        Some(PrefixStabilityViolation::TurnOrderAltered {
            turn_index,
            expected_role: expected_role.to_string(),
            actual_role: actual_role.to_string(),
        })
    } else {
        None
    }
}

/// Audits content hash equality between baseline and incoming turns.
fn check_content_alignment(
    turn_index: usize,
    baseline: &DialogTurn,
    new_turn: &DialogTurn,
) -> Option<PrefixStabilityViolation> {
    if baseline.content_hash == new_turn.content_hash {
        return None;
    }
    if baseline.role == "system" {
        Some(PrefixStabilityViolation::SystemPromptMutated {
            initial_hash: baseline.content_hash.clone(),
            mutated_hash: new_turn.content_hash.clone(),
        })
    } else {
        Some(PrefixStabilityViolation::EarlierTurnContentMutated {
            turn_index,
            snippet: new_turn.content_snippet.clone(),
        })
    }
}

/// Inspects a single turn slot for both role order alterations and content mutations.
fn audit_turn_at_slot(
    slot: usize,
    baseline: &DialogTurn,
    new_turn: &DialogTurn,
    violations: &mut Vec<PrefixStabilityViolation>,
) {
    if let Some(violation) = check_role_alignment(slot, &baseline.role, &new_turn.role) {
        violations.push(violation);
    }
    if let Some(violation) = check_content_alignment(slot, baseline, new_turn) {
        violations.push(violation);
    }
}

/// Computes count of consecutive initial turns preserved identically without mutation.
fn count_preserved_prefix_turns(
    baseline_history: &[DialogTurn],
    new_history: &[DialogTurn],
) -> usize {
    baseline_history
        .iter()
        .zip(new_history.iter())
        .take_while(|(b, n)| b.role == n.role && b.content_hash == n.content_hash)
        .count()
}

/// Evaluates prefix stability between an existing baseline turn history and incoming new turn history.
pub fn verify_prefix_stability(
    baseline_history: &[DialogTurn],
    new_history: &[DialogTurn],
) -> PrefixStabilityReport {
    let mut violations = Vec::new();
    if new_history.len() < baseline_history.len() {
        violations.push(PrefixStabilityViolation::TurnHistoryTruncated {
            baseline_turns: baseline_history.len(),
            new_turns: new_history.len(),
        });
    }
    let cmp_len = baseline_history.len().min(new_history.len());
    for i in 0..cmp_len {
        audit_turn_at_slot(i, &baseline_history[i], &new_history[i], &mut violations);
    }
    let prefix_turns_preserved = count_preserved_prefix_turns(baseline_history, new_history);
    let is_stable = violations.is_empty();
    PrefixStabilityReport {
        is_stable,
        prefix_turns_preserved,
        total_new_turns: new_history.len(),
        violations,
    }
}

/// Helper to build a sequence of dialog turns from slice of (role, content) pairs.
pub fn build_dialog_turns<'a, I>(turns: I) -> Vec<DialogTurn>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    turns
        .into_iter()
        .enumerate()
        .map(|(idx, (role, content))| create_dialog_turn(idx, role, content))
        .collect()
}
