//! Unified Context Alignment Guard (`Imp-261`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariants:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including
//! thinking block backfilling, thinking budget filtering and backfilling, unified
//! context structural alignment, prefix stability, and the sanitization of risky
//! prompts and request headers."
//!
//! Provides strict alternating role enforcement, contiguous role merging,
//! system turn extraction to preamble, and synthetic turn backfilling.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Fallback synthetic prompt text when backfilling user turns.
pub const DEFAULT_SYNTHETIC_USER_PROMPT: &str = "...";

/// Canonical conversation roles supported by the context alignment guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationRole {
    User,
    Model,
    System,
}

impl ConversationRole {
    /// Returns the canonical protocol role identifier string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model => "model",
            Self::System => "system",
        }
    }

    /// Returns true if this role represents a user prompt.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User)
    }

    /// Returns true if this role represents a model response.
    pub fn is_model(&self) -> bool {
        matches!(self, Self::Model)
    }

    /// Returns true if this role represents a system instruction.
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
    }
}

impl fmt::Display for ConversationRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Static helper to resolve the string representation of a ConversationRole.
pub fn role_as_str(role: ConversationRole) -> &'static str {
    role.as_str()
}

/// An individual conversational turn containing a role and text content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: ConversationRole,
    pub text: String,
}

impl ConversationTurn {
    /// Constructs a new conversation turn with the given role and text.
    pub fn new(role: ConversationRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
        }
    }

    /// Helper constructor for a user turn.
    pub fn user(text: impl Into<String>) -> Self {
        Self::new(ConversationRole::User, text)
    }

    /// Helper constructor for a model turn.
    pub fn model(text: impl Into<String>) -> Self {
        Self::new(ConversationRole::Model, text)
    }

    /// Helper constructor for a system turn.
    pub fn system(text: impl Into<String>) -> Self {
        Self::new(ConversationRole::System, text)
    }

    /// Returns true if this turn is from a user.
    pub fn is_user(&self) -> bool {
        self.role.is_user()
    }

    /// Returns true if this turn is from a model.
    pub fn is_model(&self) -> bool {
        self.role.is_model()
    }

    /// Returns true if this turn is from the system.
    pub fn is_system(&self) -> bool {
        self.role.is_system()
    }
}

/// Discrete alignment modifications performed during conversation normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentAction {
    Unchanged,
    ContiguousRolesMerged {
        role: ConversationRole,
        merged_count: usize,
    },
    ModelTurnBackfilled {
        at_index: usize,
    },
    SystemTurnMovedToPreamble,
}

impl AlignmentAction {
    /// Returns true if the action indicates no modifications were made.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged)
    }
}

impl fmt::Display for AlignmentAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unchanged => write!(f, "Unchanged"),
            Self::ContiguousRolesMerged { role, merged_count } => {
                write!(
                    f,
                    "Contiguous {} turns merged (count: {})",
                    role.as_str(),
                    merged_count
                )
            }
            Self::ModelTurnBackfilled { at_index } => {
                write!(f, "Model turn backfilled at index {}", at_index)
            }
            Self::SystemTurnMovedToPreamble => {
                write!(f, "System turn moved to preamble")
            }
        }
    }
}

/// Outcome report summarizing context structural alignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextAlignmentReport {
    pub is_strictly_alternating: bool,
    pub initial_turn_count: usize,
    pub aligned_turn_count: usize,
    pub aligned_turns: Vec<ConversationTurn>,
    pub actions: Vec<AlignmentAction>,
}

impl ContextAlignmentReport {
    /// Creates a new context alignment report.
    pub fn new(
        initial_turn_count: usize,
        aligned_turns: Vec<ConversationTurn>,
        actions: Vec<AlignmentAction>,
    ) -> Self {
        let is_strictly_alternating = is_strictly_alternating(&aligned_turns);
        let aligned_turn_count = aligned_turns.len();
        Self {
            is_strictly_alternating,
            initial_turn_count,
            aligned_turn_count,
            aligned_turns,
            actions,
        }
    }

    /// Returns true if any structural modifications were applied.
    pub fn has_modifications(&self) -> bool {
        !self
            .actions
            .iter()
            .all(|action| matches!(action, AlignmentAction::Unchanged))
    }

    /// Returns true if the conversation context remained completely unchanged.
    pub fn is_unchanged(&self) -> bool {
        !self.has_modifications()
    }

    /// Backfills a model turn at the specified index and records the action.
    pub fn backfill_model_turn(&mut self, at_index: usize, text: impl Into<String>) {
        let action = backfill_model_turn(&mut self.aligned_turns, at_index, text);
        self.aligned_turn_count = self.aligned_turns.len();
        self.actions.push(action);
        self.is_strictly_alternating = is_strictly_alternating(&self.aligned_turns);
    }
}

/// Verifies whether the slice of conversation turns strictly alternates user -> model -> user -> model.
pub fn is_strictly_alternating(turns: &[ConversationTurn]) -> bool {
    if turns.is_empty() {
        return true;
    }
    if turns[0].role != ConversationRole::User {
        return false;
    }
    for window in turns.windows(2) {
        if window[0].role == window[1].role {
            return false;
        }
    }
    true
}

/// Inserts a synthetic model turn into the turns vector at the requested index.
pub fn backfill_model_turn(
    turns: &mut Vec<ConversationTurn>,
    at_index: usize,
    text: impl Into<String>,
) -> AlignmentAction {
    let insert_idx = at_index.min(turns.len());
    turns.insert(
        insert_idx,
        ConversationTurn::new(ConversationRole::Model, text.into()),
    );
    AlignmentAction::ModelTurnBackfilled {
        at_index: insert_idx,
    }
}

/// Filters system turns out of the input sequence, logging actions.
fn filter_system_turns(
    turns: &[ConversationTurn],
    actions: &mut Vec<AlignmentAction>,
) -> Vec<ConversationTurn> {
    let mut filtered = Vec::with_capacity(turns.len());
    for turn in turns {
        if turn.role.is_system() {
            actions.push(AlignmentAction::SystemTurnMovedToPreamble);
        } else {
            filtered.push(turn.clone());
        }
    }
    filtered
}

/// Emits a single merged turn and logs an action if multiple turns were joined.
fn flush_merged_group(
    role: ConversationRole,
    count: usize,
    texts: &[String],
    merged: &mut Vec<ConversationTurn>,
    actions: &mut Vec<AlignmentAction>,
) {
    if count > 1 {
        actions.push(AlignmentAction::ContiguousRolesMerged {
            role,
            merged_count: count,
        });
    }
    merged.push(ConversationTurn::new(role, texts.join("\n\n")));
}

/// Merges contiguous turns sharing the same role into unified text blocks.
fn merge_contiguous_roles(
    turns: Vec<ConversationTurn>,
    actions: &mut Vec<AlignmentAction>,
) -> Vec<ConversationTurn> {
    if turns.is_empty() {
        return Vec::new();
    }
    let mut merged: Vec<ConversationTurn> = Vec::new();
    let mut current_role = turns[0].role;
    let mut current_texts = vec![turns[0].text.clone()];
    let mut current_count = 1;

    for turn in turns.into_iter().skip(1) {
        if turn.role == current_role {
            current_texts.push(turn.text);
            current_count += 1;
        } else {
            flush_merged_group(
                current_role,
                current_count,
                &current_texts,
                &mut merged,
                actions,
            );
            current_role = turn.role;
            current_texts = vec![turn.text];
            current_count = 1;
        }
    }

    flush_merged_group(
        current_role,
        current_count,
        &current_texts,
        &mut merged,
        actions,
    );
    merged
}

/// Ensures the initial turn is a user turn by prepending synthetic user prompt if needed.
fn ensure_initial_user_turn(turns: &mut Vec<ConversationTurn>) {
    if let Some(first) = turns.first() {
        if !first.role.is_user() {
            turns.insert(0, ConversationTurn::user(DEFAULT_SYNTHETIC_USER_PROMPT));
        }
    }
}

/// Aligns a slice of conversation turns into conforming strictly alternating context.
pub fn align_conversation_turns(turns: &[ConversationTurn]) -> ContextAlignmentReport {
    let initial_turn_count = turns.len();
    if turns.is_empty() {
        return ContextAlignmentReport {
            is_strictly_alternating: true,
            initial_turn_count: 0,
            aligned_turn_count: 0,
            aligned_turns: Vec::new(),
            actions: vec![AlignmentAction::Unchanged],
        };
    }

    let mut actions = Vec::new();
    let non_system_turns = filter_system_turns(turns, &mut actions);
    let mut aligned = merge_contiguous_roles(non_system_turns, &mut actions);
    ensure_initial_user_turn(&mut aligned);

    if actions.is_empty() {
        actions.push(AlignmentAction::Unchanged);
    }

    ContextAlignmentReport::new(initial_turn_count, aligned, actions)
}
