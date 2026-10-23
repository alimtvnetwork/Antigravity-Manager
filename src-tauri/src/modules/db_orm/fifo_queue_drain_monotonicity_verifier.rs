//! FIFO Queue Drain Monotonicity Verifier (`Imp-681`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`)
//! are strictly forbidden in dispatch logic."
//!
//! Verifies that items returned from sequential batch drains maintain strictly increasing
//! `(created_at, id)` progression without sequence inversion, duplicate IDs, or timestamp regressions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// An item drained from a FIFO SQLite queue table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItemRecord {
    /// Deterministic sequence primary key identifier.
    pub id: i64,
    /// Unix epoch timestamp in milliseconds when the prompt was enqueued.
    pub created_at: i64,
    /// Name of the SQLite queue table (e.g. `active_prompts`, `prompt_backups`).
    pub table_name: String,
}

impl QueueItemRecord {
    /// Creates a new queue item record.
    pub fn new(id: i64, created_at: i64, table_name: impl Into<String>) -> Self {
        Self {
            id,
            created_at,
            table_name: table_name.into(),
        }
    }
}

/// Violations detected when verifying FIFO monotonicity across drained items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonotonicityViolation {
    /// An item has a timestamp earlier than its preceding item in the drain.
    TimestampRegression {
        table_name: String,
        prev_id: i64,
        prev_created_at: i64,
        curr_id: i64,
        curr_created_at: i64,
    },
    /// Identical timestamps must be tie-broken by strictly increasing ID.
    IdTieBreakerInversion {
        table_name: String,
        created_at: i64,
        prev_id: i64,
        curr_id: i64,
    },
    /// The same item ID appeared more than once in the drain sequence.
    DuplicateItemId {
        table_name: String,
        id: i64,
    },
}

impl fmt::Display for MonotonicityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimestampRegression {
                table_name,
                prev_id,
                prev_created_at,
                curr_id,
                curr_created_at,
            } => write!(
                f,
                "Timestamp regression in '{table_name}': item #{curr_id} (t={curr_created_at}) preceded by item #{prev_id} (t={prev_created_at})"
            ),
            Self::IdTieBreakerInversion {
                table_name,
                created_at,
                prev_id,
                curr_id,
            } => write!(
                f,
                "ID tie-breaker inversion in '{table_name}' at t={created_at}: item #{curr_id} <= preceding item #{prev_id}"
            ),
            Self::DuplicateItemId { table_name, id } => {
                write!(f, "Duplicate item ID #{id} detected in '{table_name}' drain")
            }
        }
    }
}

/// Audit report produced after verifying FIFO queue monotonicity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonotonicityAuditReport {
    /// True if all items strictly satisfy `ORDER BY created_at ASC, id ASC`.
    pub is_strictly_monotonic: bool,
    /// Total count of queue items inspected across batches.
    pub total_items_verified: usize,
    /// Minimum `created_at` timestamp observed in the drain sequence.
    pub min_created_at: Option<i64>,
    /// Maximum `created_at` timestamp observed in the drain sequence.
    pub max_created_at: Option<i64>,
    /// Granular list of detected sequence ordering violations.
    pub violations: Vec<MonotonicityViolation>,
}

/// Verifies that a slice of queue item records strictly adheres to FIFO ordering.
pub fn verify_queue_items_monotonicity(items: &[QueueItemRecord]) -> Result<MonotonicityAuditReport, MonotonicityViolation> {
    if items.is_empty() {
        return Ok(MonotonicityAuditReport {
            is_strictly_monotonic: true,
            total_items_verified: 0,
            min_created_at: None,
            max_created_at: None,
            violations: Vec::new(),
        });
    }

    let mut violations = Vec::new();
    let min_created_at = Some(items[0].created_at);
    let mut max_created_at = Some(items[0].created_at);

    for idx in 1..items.len() {
        let prev = &items[idx - 1];
        let curr = &items[idx];

        max_created_at = Some(max_created_at.unwrap().max(curr.created_at));

        if curr.id == prev.id {
            let violation = MonotonicityViolation::DuplicateItemId {
                table_name: curr.table_name.clone(),
                id: curr.id,
            };
            violations.push(violation.clone());
            return Err(violation);
        }

        if curr.created_at < prev.created_at {
            let violation = MonotonicityViolation::TimestampRegression {
                table_name: curr.table_name.clone(),
                prev_id: prev.id,
                prev_created_at: prev.created_at,
                curr_id: curr.id,
                curr_created_at: curr.created_at,
            };
            violations.push(violation.clone());
            return Err(violation);
        } else if curr.created_at == prev.created_at && curr.id <= prev.id {
            let violation = MonotonicityViolation::IdTieBreakerInversion {
                table_name: curr.table_name.clone(),
                created_at: curr.created_at,
                prev_id: prev.id,
                curr_id: curr.id,
            };
            violations.push(violation.clone());
            return Err(violation);
        }
    }

    Ok(MonotonicityAuditReport {
        is_strictly_monotonic: violations.is_empty(),
        total_items_verified: items.len(),
        min_created_at,
        max_created_at,
        violations,
    })
}

/// Audits a stream of successive chunked batches for end-to-end FIFO continuity.
pub fn audit_chunked_batch_stream(batches: &[Vec<QueueItemRecord>]) -> MonotonicityAuditReport {
    let mut all_items = Vec::new();
    for batch in batches {
        all_items.extend(batch.iter().cloned());
    }

    match verify_queue_items_monotonicity(&all_items) {
        Ok(report) => report,
        Err(violation) => MonotonicityAuditReport {
            is_strictly_monotonic: false,
            total_items_verified: all_items.len(),
            min_created_at: all_items.first().map(|i| i.created_at),
            max_created_at: all_items.last().map(|i| i.created_at),
            violations: vec![violation],
        },
    }
}
