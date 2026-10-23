//! Strict FIFO Queue Batch Drain Guard (`Imp-664`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Provides deterministic batch query construction, query validation against LIFO ordering,
//! and safe chunk sequence monotonicity validation for prompt queue drain workers.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Mandatory canonical FIFO ordering clause required across prompt queue batch queries.
pub const MANDATORY_FIFO_ORDER_CLAUSE: &str = "ORDER BY created_at ASC, id ASC";

/// Specification parameters for draining records from a prompt queue table in discrete chunks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrainBatchSpec {
    pub table_name: String,
    pub chunk_size: usize,
    pub max_total_records: usize,
}

impl DrainBatchSpec {
    /// Creates a new batch drain specification.
    pub fn new(table_name: impl Into<String>, chunk_size: usize, max_total_records: usize) -> Self {
        Self {
            table_name: table_name.into(),
            chunk_size,
            max_total_records,
        }
    }

    /// Returns the target table name.
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Returns the size of each drain chunk.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Returns the maximum total records to drain across all chunks.
    pub fn max_total_records(&self) -> usize {
        self.max_total_records
    }

    /// Builds a chunk query corresponding to this specification at the specified offset.
    pub fn build_chunk_query(&self, offset: usize) -> String {
        build_batch_chunk_query(&self.table_name, self.chunk_size, offset)
    }
}

/// A single queue record item drained during batch execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrainedItem {
    pub id: i64,
    pub created_at: i64,
    pub payload: String,
}

impl DrainedItem {
    /// Constructs a new drained queue item.
    pub fn new(id: i64, created_at: i64, payload: impl Into<String>) -> Self {
        Self {
            id,
            created_at,
            payload: payload.into(),
        }
    }

    /// Returns the record ID.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Returns the creation timestamp epoch in milliseconds or seconds.
    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Returns the item payload.
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// Violations identified when validating queue queries or chunk monotonicity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchDrainViolation {
    /// LIFO ordering was detected in the query (e.g. `DESC` keyword present).
    LifoOrderingForbidden { query: String },
    /// Deterministic secondary tie-breaker `id ASC` is missing from the ordering clause.
    MissingOrderTieBreaker { query: String },
    /// Sequence monotonicity was violated across chunk boundaries or within a chunk.
    ChunkSequenceInversion {
        previous_created_at: i64,
        next_created_at: i64,
    },
}

impl fmt::Display for BatchDrainViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LifoOrderingForbidden { query } => {
                write!(
                    f,
                    "LIFO ordering is forbidden in FIFO queue query: {}",
                    query
                )
            }
            Self::MissingOrderTieBreaker { query } => {
                write!(
                    f,
                    "Missing mandatory secondary tie-breaker 'id ASC' in query: {}",
                    query
                )
            }
            Self::ChunkSequenceInversion {
                previous_created_at,
                next_created_at,
            } => {
                write!(
                    f,
                    "FIFO chunk sequence inversion detected: previous created_at ({}) > next created_at ({})",
                    previous_created_at, next_created_at
                )
            }
        }
    }
}

impl std::error::Error for BatchDrainViolation {}

/// Summary report returned after safely draining and validating queue chunks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchDrainReport {
    pub is_fifo_maintained: bool,
    pub total_chunks_processed: usize,
    pub total_records_drained: usize,
    pub drained_items: Vec<DrainedItem>,
    pub violation: Option<BatchDrainViolation>,
}

impl BatchDrainReport {
    /// Returns true if FIFO ordering was strictly maintained without sequence inversions.
    pub fn is_fifo_maintained(&self) -> bool {
        self.is_fifo_maintained
    }

    /// Returns the count of chunks processed.
    pub fn total_chunks_processed(&self) -> usize {
        self.total_chunks_processed
    }

    /// Returns the total count of drained records.
    pub fn total_records_drained(&self) -> usize {
        self.total_records_drained
    }

    /// Returns a slice of drained items.
    pub fn drained_items(&self) -> &[DrainedItem] {
        &self.drained_items
    }

    /// Returns the detected violation if any occurred.
    pub fn violation(&self) -> Option<&BatchDrainViolation> {
        self.violation.as_ref()
    }

    /// Returns true if a violation occurred during batch draining.
    pub fn has_violation(&self) -> bool {
        self.violation.is_some()
    }
}

/// Builds a deterministic batch chunk query enforcing the canonical FIFO ordering clause.
pub fn build_batch_chunk_query(table: &str, limit: usize, offset: usize) -> String {
    format!(
        "SELECT id, created_at, payload FROM {} {} LIMIT {} OFFSET {};",
        table, MANDATORY_FIFO_ORDER_CLAUSE, limit, offset
    )
}

/// Validates that an SQL batch query strictly maintains FIFO ordering without LIFO directions.
pub fn validate_chunk_query_ordering(sql: &str) -> Result<(), BatchDrainViolation> {
    if contains_desc_ordering(sql) {
        return Err(BatchDrainViolation::LifoOrderingForbidden {
            query: sql.to_string(),
        });
    }

    if !has_mandatory_fifo_clause(sql) {
        return Err(BatchDrainViolation::MissingOrderTieBreaker {
            query: sql.to_string(),
        });
    }

    Ok(())
}

/// Detects if `DESC` is used as a standalone keyword in the SQL query.
fn contains_desc_ordering(sql: &str) -> bool {
    sql.to_uppercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|token| token == "DESC")
}

/// Checks whether the query contains the mandatory FIFO clause and secondary tie-breaker.
fn has_mandatory_fifo_clause(sql: &str) -> bool {
    let upper = sql.to_uppercase();
    let words: Vec<&str> = upper.split_whitespace().collect();
    let unified = words.join(" ").replace(" ,", ",").replace(", ", ",");

    if unified.contains("ORDER BY CREATED_AT ASC,ID ASC") {
        return true;
    }

    has_qualified_fifo_clause(&unified)
}

/// Checks for table-qualified canonical ordering (e.g. `ORDER BY t.created_at ASC, t.id ASC`).
fn has_qualified_fifo_clause(unified_sql: &str) -> bool {
    if let Some(order_pos) = unified_sql.find("ORDER BY ") {
        let after_order = &unified_sql[order_pos + 9..];
        let parts: Vec<&str> = after_order.split(',').collect();
        if parts.len() >= 2 {
            let first_col_ok = parts[0].trim().ends_with("CREATED_AT ASC");
            let second_col_ok = parts[1].trim().starts_with("ID ASC")
                || parts[1].trim().contains(".ID ASC");
            return first_col_ok && second_col_ok;
        }
    }
    false
}

/// Verifies that two consecutive queue items maintain FIFO monotonicity.
fn verify_monotonic_order(
    prev: Option<(i64, i64)>,
    current: &DrainedItem,
) -> Result<(), BatchDrainViolation> {
    if let Some((prev_created_at, prev_id)) = prev {
        let is_inversion = current.created_at < prev_created_at
            || (current.created_at == prev_created_at && current.id < prev_id);
        if is_inversion {
            return Err(BatchDrainViolation::ChunkSequenceInversion {
                previous_created_at: prev_created_at,
                next_created_at: current.created_at,
            });
        }
    }
    Ok(())
}

/// Safely drains and validates chunks of queue items, guaranteeing FIFO monotonicity across batches.
pub fn drain_chunks_safely(chunks: &[Vec<DrainedItem>]) -> BatchDrainReport {
    let mut drained_items = Vec::new();
    let mut prev_cursor: Option<(i64, i64)> = None;
    let mut chunks_processed = 0;

    for chunk in chunks {
        chunks_processed += 1;
        for item in chunk {
            if let Err(violation) = verify_monotonic_order(prev_cursor, item) {
                return BatchDrainReport {
                    is_fifo_maintained: false,
                    total_chunks_processed: chunks_processed,
                    total_records_drained: drained_items.len(),
                    drained_items,
                    violation: Some(violation),
                };
            }
            prev_cursor = Some((item.created_at, item.id));
            drained_items.push(item.clone());
        }
    }

    BatchDrainReport {
        is_fifo_maintained: true,
        total_chunks_processed: chunks_processed,
        total_records_drained: drained_items.len(),
        drained_items,
        violation: None,
    }
}
