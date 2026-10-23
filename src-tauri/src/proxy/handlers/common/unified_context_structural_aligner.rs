//! Unified Context Structural Aligner (`Imp-626`)
//!
//! Normalizes and enforces Gemini context turn structures:
//! - Deduplicates and coalesces consecutive duplicate roles.
//! - Prunes empty or whitespace-only text parts.
//! - Ensures every turn contains at least one valid part with fallback insertion.
//! - Enforces strict alternating role sequence (user -> model -> user -> model).

use serde::{Deserialize, Serialize};

/// Fallback text token used when turns are empty or when prepending synthetic initial turns.
pub const DEFAULT_FALLBACK_TEXT: &str = "...";

/// Canonical role turn representation for protocol alignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleTurn {
    pub role: String,
    pub parts: Vec<serde_json::Value>,
}

impl RoleTurn {
    /// Creates a new RoleTurn with raw parts.
    pub fn new(role: impl Into<String>, parts: Vec<serde_json::Value>) -> Self {
        Self {
            role: role.into(),
            parts,
        }
    }

    /// Convenience constructor creating a turn with a single text part.
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            parts: vec![serde_json::json!({ "text": text.into() })],
        }
    }
}

/// Structural violations identified during context inspection and alignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentViolation {
    /// Two adjacent turns had the same role and were merged.
    ConsecutiveDuplicateRole { role: String, index: usize },
    /// A turn had no parts or only empty whitespace parts.
    EmptyPartsInTurn { index: usize },
    /// Role was non-canonical or violated expected alternating order.
    InvalidRoleSequence {
        expected: String,
        found: String,
        index: usize,
    },
}

/// Report summarizing structural alignment modifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextAlignmentReport {
    /// True if the input was already strictly aligned without modifications.
    pub is_aligned: bool,
    /// Number of duplicate role turns merged into predecessor turns.
    pub turns_merged_count: usize,
    /// Number of empty or whitespace text parts removed.
    pub empty_parts_pruned_count: usize,
    /// Collection of all detected structural violations.
    pub violations_detected: Vec<AlignmentViolation>,
    /// Final turn count after merging and alignment.
    pub final_turn_count: usize,
}

impl ContextAlignmentReport {
    /// Creates a new alignment report.
    pub fn new(
        turns_merged_count: usize,
        empty_parts_pruned_count: usize,
        violations_detected: Vec<AlignmentViolation>,
        final_turn_count: usize,
    ) -> Self {
        let is_aligned = violations_detected.is_empty()
            && turns_merged_count == 0
            && empty_parts_pruned_count == 0;
        Self {
            is_aligned,
            turns_merged_count,
            empty_parts_pruned_count,
            violations_detected,
            final_turn_count,
        }
    }

    /// Positive helper indicating if any violations were detected.
    pub fn has_violations(&self) -> bool {
        !self.violations_detected.is_empty()
    }
}

/// Determines if a part value represents empty or whitespace-only text.
pub fn is_empty_text_part(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::String(s) => s.trim().is_empty(),
        serde_json::Value::Object(map) => {
            if let Some(text_val) = map.get("text") {
                if let Some(text_str) = text_val.as_str() {
                    return text_str.trim().is_empty() && map.len() == 1;
                }
            }
            map.is_empty()
        }
        serde_json::Value::Null => true,
        serde_json::Value::Array(arr) => arr.is_empty(),
        _ => false,
    }
}

/// Filters out empty text parts and tallies pruned count.
fn filter_valid_parts(
    parts: &[serde_json::Value],
    pruned_count: &mut usize,
) -> Vec<serde_json::Value> {
    parts
        .iter()
        .filter(|part| {
            let empty = is_empty_text_part(part);
            if empty {
                *pruned_count += 1;
            }
            !empty
        })
        .cloned()
        .collect()
}

/// Cleans parts for a single turn, backfilling default text if all parts were empty.
fn clean_turn_parts(
    parts: &[serde_json::Value],
    index: usize,
    pruned_count: &mut usize,
    violations: &mut Vec<AlignmentViolation>,
) -> Vec<serde_json::Value> {
    let mut valid = filter_valid_parts(parts, pruned_count);
    if valid.is_empty() {
        violations.push(AlignmentViolation::EmptyPartsInTurn { index });
        valid.push(serde_json::json!({ "text": DEFAULT_FALLBACK_TEXT }));
    }
    valid
}

/// Canonicalizes raw role strings into standard Gemini roles ("user" or "model").
fn canonicalize_role(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "model" | "assistant" => "model",
        _ => "user",
    }
}

/// Validates role canonicality against standard Gemini roles.
fn validate_role_canonicality(
    raw_role: &str,
    canonical_role: &str,
    index: usize,
    violations: &mut Vec<AlignmentViolation>,
) {
    if raw_role.trim().to_ascii_lowercase() != canonical_role {
        violations.push(AlignmentViolation::InvalidRoleSequence {
            expected: canonical_role.to_string(),
            found: raw_role.to_string(),
            index,
        });
    }
}

/// Attempts to merge adjacent turns having identical roles.
fn try_merge_adjacent_role(
    last: &mut RoleTurn,
    current: &mut RoleTurn,
    index: usize,
    merged_count: &mut usize,
    violations: &mut Vec<AlignmentViolation>,
) -> bool {
    if last.role != current.role {
        return false;
    }
    violations.push(AlignmentViolation::ConsecutiveDuplicateRole {
        role: current.role.clone(),
        index,
    });
    last.parts.append(&mut current.parts);
    *merged_count += 1;
    true
}

/// Pushes a turn to the list or merges into previous if roles match.
fn push_or_merge_turn(
    mut turn: RoleTurn,
    index: usize,
    merged: &mut Vec<RoleTurn>,
    merged_count: &mut usize,
    violations: &mut Vec<AlignmentViolation>,
) {
    if let Some(last) = merged.last_mut() {
        if try_merge_adjacent_role(last, &mut turn, index, merged_count, violations) {
            return;
        }
    }
    merged.push(turn);
}

/// Enforces that the conversation context begins with a user turn.
fn enforce_initial_user_turn(turns: &mut Vec<RoleTurn>, violations: &mut Vec<AlignmentViolation>) {
    if let Some(first) = turns.first() {
        if first.role != "user" {
            violations.push(AlignmentViolation::InvalidRoleSequence {
                expected: "user".to_string(),
                found: first.role.clone(),
                index: 0,
            });
            turns.insert(0, RoleTurn::text("user", DEFAULT_FALLBACK_TEXT));
        }
    }
}

/// Aligns a slice of RoleTurns into conforming alternating Gemini context.
pub fn align_context_turns(turns: &[RoleTurn]) -> (Vec<RoleTurn>, ContextAlignmentReport) {
    if turns.is_empty() {
        return (Vec::new(), ContextAlignmentReport::new(0, 0, Vec::new(), 0));
    }

    let mut violations = Vec::new();
    let mut turns_merged_count = 0;
    let mut empty_parts_pruned_count = 0;
    let mut merged_turns = Vec::new();

    for (idx, turn) in turns.iter().enumerate() {
        let canonical = canonicalize_role(&turn.role);
        validate_role_canonicality(&turn.role, canonical, idx, &mut violations);
        let clean_parts = clean_turn_parts(
            &turn.parts,
            idx,
            &mut empty_parts_pruned_count,
            &mut violations,
        );
        let cleaned_turn = RoleTurn::new(canonical, clean_parts);
        push_or_merge_turn(
            cleaned_turn,
            idx,
            &mut merged_turns,
            &mut turns_merged_count,
            &mut violations,
        );
    }

    enforce_initial_user_turn(&mut merged_turns, &mut violations);

    let final_turn_count = merged_turns.len();
    let report = ContextAlignmentReport::new(
        turns_merged_count,
        empty_parts_pruned_count,
        violations,
        final_turn_count,
    );
    (merged_turns, report)
}
