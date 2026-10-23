//! FIFO Restore Transaction Coordinator (`Task-208`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Coordinates multi-table restore operations across `active_prompts`, `prompt_backups`,
//! and `repo_prompts`, verifying stream monotonicity and guaranteeing deterministic rollback
//! if any out-of-order or LIFO item is encountered.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Canonical FIFO order clause required across prompt restore and queue dispatch queries.
pub const CANONICAL_FIFO_ORDER: &str = "ORDER BY created_at ASC, id ASC";

/// Distinct SQLite tables participating in prompt restore transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestoreTableKind {
    /// Active prompt processing queue table (`active_prompts`).
    ActivePrompts,
    /// Archived prompt backup table (`prompt_backups`).
    PromptBackups,
    /// Repository-level prompt table (`repo_prompts`).
    RepoPrompts,
}

impl RestoreTableKind {
    /// Returns the canonical SQLite table name for this table kind.
    pub fn as_table_name(&self) -> &'static str {
        match self {
            Self::ActivePrompts => "active_prompts",
            Self::PromptBackups => "prompt_backups",
            Self::RepoPrompts => "repo_prompts",
        }
    }

    /// Returns the table name as a string slice.
    pub fn as_str(&self) -> &'static str {
        self.as_table_name()
    }

    /// Returns an array containing all table kind variants.
    pub fn all() -> &'static [Self] {
        &[Self::ActivePrompts, Self::PromptBackups, Self::RepoPrompts]
    }
}

impl fmt::Display for RestoreTableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_table_name())
    }
}

/// Represents an individual prompt record retrieved during a restore operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreItemRecord {
    pub id: i64,
    pub table: RestoreTableKind,
    pub created_at: i64,
    pub prompt_text: String,
}

impl RestoreItemRecord {
    /// Constructs a new `RestoreItemRecord`.
    pub fn new(
        id: i64,
        table: RestoreTableKind,
        created_at: i64,
        prompt_text: impl Into<String>,
    ) -> Self {
        Self {
            id,
            table,
            created_at,
            prompt_text: prompt_text.into(),
        }
    }
}

/// Violations identified when coordinating prompt restore transactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionCoordinatorViolation {
    /// Monotonic ID order was violated when timestamps are equal.
    OutOrOrderSequenceDetected { previous_id: i64, current_id: i64 },
    /// Timestamp ordering was inverted (current item created before previous item).
    InvertedTimestampDetected {
        previous_created_at: i64,
        current_created_at: i64,
    },
    /// A forbidden LIFO ordering clause was found in a restore query.
    LifoClauseForbidden { query: String },
    /// Mandatory secondary tie-breaker `id ASC` is missing from the ordering clause.
    MissingOrderTieBreaker { query: String },
}

impl fmt::Display for TransactionCoordinatorViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOrOrderSequenceDetected {
                previous_id,
                current_id,
            } => {
                write!(
                    f,
                    "Out of order sequence detected: previous_id={previous_id}, current_id={current_id}"
                )
            }
            Self::InvertedTimestampDetected {
                previous_created_at,
                current_created_at,
            } => {
                write!(
                    f,
                    "Inverted timestamp detected: previous_created_at={previous_created_at}, current_created_at={current_created_at}"
                )
            }
            Self::LifoClauseForbidden { query } => {
                write!(f, "LIFO clause is forbidden in query: '{query}'")
            }
            Self::MissingOrderTieBreaker { query } => {
                write!(
                    f,
                    "Missing secondary tie-breaker 'id ASC' in query: '{query}'"
                )
            }
        }
    }
}

impl Error for TransactionCoordinatorViolation {}

/// Comprehensive report produced by the restore transaction coordinator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinatedRestoreReport {
    pub is_committed: bool,
    pub total_restored: usize,
    pub processed_tables: Vec<RestoreTableKind>,
    pub violation: Option<TransactionCoordinatorViolation>,
}

impl CoordinatedRestoreReport {
    /// Returns true if the restore transaction was committed without violations.
    pub fn is_committed(&self) -> bool {
        self.is_committed
    }

    /// Returns the total count of successfully restored records.
    pub fn total_restored(&self) -> usize {
        self.total_restored
    }

    /// Returns a reference to the violation if the transaction failed.
    pub fn violation(&self) -> Option<&TransactionCoordinatorViolation> {
        self.violation.as_ref()
    }
}

/// Constructs a canonical SQL query for restoring prompt records in strict FIFO order.
pub fn build_coordinated_restore_sql(table: RestoreTableKind, limit: usize) -> String {
    format!(
        "SELECT id, created_at, prompt_text FROM {} {} LIMIT {};",
        table.as_table_name(),
        CANONICAL_FIFO_ORDER,
        limit
    )
}

/// Validates that a restore SQL query conforms to strict FIFO dispatch invariants.
pub fn validate_coordinated_query_sql(query: &str) -> Result<(), TransactionCoordinatorViolation> {
    let normalized = query.to_uppercase();
    if normalized.contains("ORDER BY UPDATED_AT DESC")
        || normalized.contains("ORDER BY CREATED_AT DESC")
        || normalized.contains(" DESC")
    {
        return Err(TransactionCoordinatorViolation::LifoClauseForbidden {
            query: query.to_string(),
        });
    }
    if !normalized.contains("ID ASC") {
        return Err(TransactionCoordinatorViolation::MissingOrderTieBreaker {
            query: query.to_string(),
        });
    }
    Ok(())
}

/// Validates monotonicity across a pair of adjacent restore item records.
fn check_item_pair_monotonicity(
    prev: &RestoreItemRecord,
    curr: &RestoreItemRecord,
) -> Result<(), TransactionCoordinatorViolation> {
    if curr.created_at < prev.created_at {
        return Err(TransactionCoordinatorViolation::InvertedTimestampDetected {
            previous_created_at: prev.created_at,
            current_created_at: curr.created_at,
        });
    }
    if curr.created_at == prev.created_at && curr.id < prev.id {
        return Err(TransactionCoordinatorViolation::OutOrOrderSequenceDetected {
            previous_id: prev.id,
            current_id: curr.id,
        });
    }
    Ok(())
}

/// Validates that an ordered sequence of restored prompt records strictly maintains FIFO monotonicity.
pub fn validate_restore_stream_monotonicity(
    items: &[RestoreItemRecord],
) -> Result<(), TransactionCoordinatorViolation> {
    for window in items.windows(2) {
        check_item_pair_monotonicity(&window[0], &window[1])?;
    }
    Ok(())
}

/// Builds an uncommitted failure report upon encountering a violation.
fn make_failure_report(
    processed_tables: Vec<RestoreTableKind>,
    violation: TransactionCoordinatorViolation,
) -> CoordinatedRestoreReport {
    CoordinatedRestoreReport {
        is_committed: false,
        total_restored: 0,
        processed_tables,
        violation: Some(violation),
    }
}

/// Builds a committed success report when all table streams satisfy FIFO invariants.
fn make_success_report(
    processed_tables: Vec<RestoreTableKind>,
    total_restored: usize,
) -> CoordinatedRestoreReport {
    CoordinatedRestoreReport {
        is_committed: true,
        total_restored,
        processed_tables,
        violation: None,
    }
}

/// Coordinates a multi-table restore transaction, validating each stream for strict FIFO compliance.
pub fn coordinate_restore_transaction(
    table_streams: &[(RestoreTableKind, Vec<RestoreItemRecord>)],
) -> CoordinatedRestoreReport {
    let mut total_restored = 0;
    let mut processed_tables = Vec::with_capacity(table_streams.len());

    for (table, stream) in table_streams {
        processed_tables.push(*table);
        if let Err(violation) = validate_restore_stream_monotonicity(stream) {
            return make_failure_report(processed_tables, violation);
        }
        total_restored += stream.len();
    }

    make_success_report(processed_tables, total_restored)
}
