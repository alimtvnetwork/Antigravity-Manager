//! Strict FIFO Prompt Restore Transaction Guard (`Imp-659`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Intercepts prompt queue dispatch and restore operations within SQLite transactions,
//! validates compliance with canonical FIFO ordering invariants, and automatically aborts / rolls back
//! the transaction if LIFO ordering or missing tie-breaker is detected.

use crate::error::AppError;
use rusqlite::{Connection, Row, ToSql, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

/// Mandatory canonical FIFO ordering clause required across prompt restore and queue dispatch operations.
pub const CANONICAL_FIFO_ORDER_CLAUSE: &str = "ORDER BY created_at ASC, id ASC";

/// Known prompt queue tables subject to strict FIFO transaction guarding.
pub const PROMPT_QUEUE_TABLES: &[&str] = &[
    "active_prompts",
    "prompt_backups",
    "repo_prompts",
    "prompts_queue",
    "prompt_queue",
];

/// Violations identified when inspecting prompt restore and dispatch operations within a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoTransactionViolation {
    /// Inverted LIFO ordering clause was detected (e.g. `ORDER BY updated_at DESC` or `created_at DESC`).
    InvertedLIFOOrderDetected {
        sql: String,
        detected_clause: String,
    },
    /// The secondary deterministic tie-breaker `id ASC` is missing from the ordering clause.
    MissingTieBreakerIdAsc {
        sql: String,
        detected_clause: String,
    },
    /// The query is missing an `ORDER BY` clause altogether on a prompt queue table.
    MissingOrderByClause {
        sql: String,
        table: String,
    },
    /// The primary sort column is not `created_at ASC` (e.g. sorting by updated_at or priority).
    DisallowedSortColumn {
        sql: String,
        column: String,
    },
    /// Transaction has already been aborted and poisoned due to an earlier FIFO invariant violation.
    TransactionAlreadyAborted {
        reason: String,
    },
    /// Low-level database error occurred during execution.
    DatabaseError {
        message: String,
    },
}

impl fmt::Display for FifoTransactionViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedLIFOOrderDetected { sql, detected_clause } => {
                write!(
                    f,
                    "Inverted LIFO ordering detected ('{detected_clause}') in query '{sql}': must use '{CANONICAL_FIFO_ORDER_CLAUSE}'"
                )
            }
            Self::MissingTieBreakerIdAsc { sql, detected_clause } => {
                write!(
                    f,
                    "Missing mandatory secondary tie-breaker 'id ASC' ('{detected_clause}') in query '{sql}'"
                )
            }
            Self::MissingOrderByClause { sql, table } => {
                write!(
                    f,
                    "Missing ORDER BY clause for prompt queue table '{table}' in query '{sql}': strict FIFO required"
                )
            }
            Self::DisallowedSortColumn { sql, column } => {
                write!(
                    f,
                    "Disallowed primary sort column '{column}' in query '{sql}': primary sort column must be 'created_at ASC'"
                )
            }
            Self::TransactionAlreadyAborted { reason } => {
                write!(
                    f,
                    "Transaction already aborted and poisoned: {reason}"
                )
            }
            Self::DatabaseError { message } => {
                write!(f, "Database error during FIFO guarded transaction: {message}")
            }
        }
    }
}

impl std::error::Error for FifoTransactionViolation {}

impl From<rusqlite::Error> for FifoTransactionViolation {
    fn from(err: rusqlite::Error) -> Self {
        Self::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<FifoTransactionViolation> for AppError {
    fn from(err: FifoTransactionViolation) -> Self {
        AppError::Config(format!("FIFO Transaction Guard Violation: {err}"))
    }
}

/// Audit telemetry record for queries intercepted by the FIFO transaction guard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoTransactionAuditEntry {
    pub sql: String,
    pub target_table: Option<String>,
    pub is_compliant: bool,
    pub violation_details: Option<String>,
    pub timestamp_epoch_ms: u64,
}

/// Current timestamp in milliseconds since UNIX epoch.
fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Extracts the raw ORDER BY clause text up to trailing LIMIT/OFFSET/semicolon clauses.
pub fn extract_order_by_clause(sql: &str) -> Option<String> {
    let upper = sql.to_ascii_uppercase();
    let needle = "ORDER BY";
    let mut search_from = 0;
    let mut order_start = None;

    while let Some(rel) = upper[search_from..].find(needle) {
        let abs_pos = search_from + rel;
        let before_ok = abs_pos == 0
            || upper
                .as_bytes()
                .get(abs_pos - 1)
                .map_or(true, |b| b.is_ascii_whitespace() || *b == b')');
        let after_pos = abs_pos + needle.len();
        let after_ok = after_pos >= upper.len()
            || upper
                .as_bytes()
                .get(after_pos)
                .map_or(true, |b| b.is_ascii_whitespace());

        if before_ok && after_ok {
            order_start = Some(after_pos);
            break;
        }
        search_from = after_pos;
    }

    let start_idx = order_start?;
    let remaining = sql[start_idx..].trim();
    if remaining.is_empty() {
        return None;
    }

    let upper_rem = remaining.to_ascii_uppercase();
    let mut end_idx = upper_rem.len();

    for marker in &["LIMIT", "OFFSET", ";"] {
        if let Some(pos) = upper_rem.find(marker) {
            if pos < end_idx {
                end_idx = pos;
            }
        }
    }

    let clause = remaining[..end_idx].trim();
    if clause.is_empty() {
        None
    } else {
        Some(clause.to_string())
    }
}

/// Checks if a given table identifier matches one of the prompt queue tables.
pub fn is_prompt_queue_table(table_name: &str) -> bool {
    let normalized = table_name
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase();

    PROMPT_QUEUE_TABLES.iter().any(|&known| {
        normalized == known || normalized.ends_with(&format!(".{known}"))
    })
}

/// Inspects whether an SQL statement targets a prompt queue table.
pub fn detect_prompt_queue_target(sql: &str) -> Option<String> {
    let lower = sql.to_ascii_lowercase();
    for &known in PROMPT_QUEUE_TABLES {
        if lower.contains(known) {
            return Some(known.to_string());
        }
    }
    None
}

/// Extracts the bare primary sort column name from an ORDER BY clause.
fn extract_primary_sort_column(clause: &str) -> String {
    let first_part = clause.split(',').next().unwrap_or("").trim();
    let token = first_part.split_whitespace().next().unwrap_or(first_part);
    let col = token.trim_matches(|c: char| c == '`' || c == '"' || c == '[' || c == ']');
    if let Some((_, c)) = col.rsplit_once('.') {
        c.to_string()
    } else {
        col.to_string()
    }
}

/// Returns true if the ORDER BY clause contains the secondary tie-breaker `id ASC`.
fn has_id_asc_tiebreaker(clause: &str) -> bool {
    let parts: Vec<&str> = clause.split(',').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return false;
    }
    parts[1..].iter().any(|part| {
        let upper = part.to_ascii_uppercase();
        let tokens: Vec<&str> = upper.split_whitespace().collect();
        if tokens.len() == 2 && tokens[1] == "ASC" {
            let col = tokens[0].trim_matches(|c: char| c == '`' || c == '"' || c == '[' || c == ']');
            let bare_col = col.rsplit('.').next().unwrap_or(col);
            bare_col == "ID"
        } else {
            false
        }
    })
}

/// Validates whether an SQL query adheres to strict FIFO ordering.
pub fn verify_fifo_transaction_sql(sql: &str) -> Result<Option<String>, FifoTransactionViolation> {
    let target_table = match detect_prompt_queue_target(sql) {
        Some(t) => t,
        None => return Ok(None),
    };

    let upper = sql.to_ascii_uppercase();
    let is_select = upper.contains("SELECT");
    let has_order_by = upper.contains("ORDER BY");

    // Point mutations (e.g. INSERT INTO ... VALUES or UPDATE ... WHERE id = ...) without SELECT/ORDER BY pass through
    if !is_select && !has_order_by {
        return Ok(Some(target_table));
    }

    let order_by_clause = match extract_order_by_clause(sql) {
        Some(clause) => clause,
        None => {
            return Err(FifoTransactionViolation::MissingOrderByClause {
                sql: sql.to_string(),
                table: target_table,
            });
        }
    };

    let upper_clause = order_by_clause.to_ascii_uppercase();
    if upper_clause.contains("DESC") {
        return Err(FifoTransactionViolation::InvertedLIFOOrderDetected {
            sql: sql.to_string(),
            detected_clause: order_by_clause,
        });
    }

    let primary_col = extract_primary_sort_column(&order_by_clause);
    if !primary_col.eq_ignore_ascii_case("created_at") {
        return Err(FifoTransactionViolation::DisallowedSortColumn {
            sql: sql.to_string(),
            column: primary_col,
        });
    }

    if !has_id_asc_tiebreaker(&order_by_clause) {
        return Err(FifoTransactionViolation::MissingTieBreakerIdAsc {
            sql: sql.to_string(),
            detected_clause: order_by_clause,
        });
    }

    Ok(Some(target_table))
}

/// Transaction guard enforcing strict FIFO ordering for prompt restore and queue dispatch operations.
pub struct FifoPromptRestoreTransactionGuard<'conn> {
    tx: Option<Transaction<'conn>>,
    is_aborted: bool,
    abort_reason: Option<String>,
    audit_log: Vec<FifoTransactionAuditEntry>,
}

impl<'conn> FifoPromptRestoreTransactionGuard<'conn> {
    /// Begins a new transaction wrapped by the FIFO guard.
    pub fn begin(conn: &'conn mut Connection) -> Result<Self, rusqlite::Error> {
        let tx = conn.transaction()?;
        Ok(Self {
            tx: Some(tx),
            is_aborted: false,
            abort_reason: None,
            audit_log: Vec::new(),
        })
    }

    /// Begins an immediate transaction wrapped by the FIFO guard.
    pub fn begin_immediate(conn: &'conn mut Connection) -> Result<Self, rusqlite::Error> {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        Ok(Self {
            tx: Some(tx),
            is_aborted: false,
            abort_reason: None,
            audit_log: Vec::new(),
        })
    }

    /// Wraps an existing SQLite transaction in the FIFO guard.
    pub fn wrap(tx: Transaction<'conn>) -> Self {
        Self {
            tx: Some(tx),
            is_aborted: false,
            abort_reason: None,
            audit_log: Vec::new(),
        }
    }

    /// Returns true if the transaction has been aborted and poisoned.
    pub fn is_aborted(&self) -> bool {
        self.is_aborted
    }

    /// Returns the reason why the transaction was aborted, if applicable.
    pub fn abort_reason(&self) -> Option<&str> {
        self.abort_reason.as_deref()
    }

    /// Returns the audit history of queries inspected by this transaction guard.
    pub fn audit_log(&self) -> &[FifoTransactionAuditEntry] {
        &self.audit_log
    }

    /// Returns a reference to the active transaction, if not aborted.
    pub fn inner(&self) -> Option<&Transaction<'conn>> {
        self.tx.as_ref()
    }

    /// Returns a mutable reference to the active transaction, if not aborted.
    pub fn inner_mut(&mut self) -> Option<&mut Transaction<'conn>> {
        self.tx.as_mut()
    }

    /// Intercepts and validates an SQL query against strict FIFO invariants, aborting if violated.
    pub fn intercept_and_verify(&mut self, sql: &str) -> Result<Option<String>, FifoTransactionViolation> {
        if self.is_aborted {
            return Err(FifoTransactionViolation::TransactionAlreadyAborted {
                reason: self
                    .abort_reason
                    .clone()
                    .unwrap_or_else(|| "Unknown abort reason".to_string()),
            });
        }

        match verify_fifo_transaction_sql(sql) {
            Ok(target_table) => {
                self.audit_log.push(FifoTransactionAuditEntry {
                    sql: sql.to_string(),
                    target_table: target_table.clone(),
                    is_compliant: true,
                    violation_details: None,
                    timestamp_epoch_ms: current_epoch_ms(),
                });
                Ok(target_table)
            }
            Err(violation) => {
                let err_msg = violation.to_string();
                self.audit_log.push(FifoTransactionAuditEntry {
                    sql: sql.to_string(),
                    target_table: detect_prompt_queue_target(sql),
                    is_compliant: false,
                    violation_details: Some(err_msg.clone()),
                    timestamp_epoch_ms: current_epoch_ms(),
                });

                self.abort_transaction(&err_msg);
                Err(violation)
            }
        }
    }

    /// Executes an SQL statement after validating strict FIFO ordering.
    pub fn execute(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<usize, FifoTransactionViolation> {
        self.intercept_and_verify(sql)?;
        let tx = self.tx.as_ref().ok_or_else(|| {
            FifoTransactionViolation::TransactionAlreadyAborted {
                reason: "Transaction was already consumed or rolled back".to_string(),
            }
        })?;

        let affected = tx.execute(sql, params)?;
        Ok(affected)
    }

    /// Executes an SQL batch after validating strict FIFO ordering.
    pub fn execute_batch(&mut self, sql: &str) -> Result<(), FifoTransactionViolation> {
        self.intercept_and_verify(sql)?;
        let tx = self.tx.as_ref().ok_or_else(|| {
            FifoTransactionViolation::TransactionAlreadyAborted {
                reason: "Transaction was already consumed or rolled back".to_string(),
            }
        })?;

        tx.execute_batch(sql)?;
        Ok(())
    }

    /// Intercepts and executes a batch prompt restore operation within the transaction.
    pub fn execute_restore_batch(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<usize, FifoTransactionViolation> {
        self.execute(sql, params)
    }

    /// Intercepts and executes a batch prompt queue dispatch operation within the transaction.
    pub fn execute_dispatch_batch(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<usize, FifoTransactionViolation> {
        self.execute(sql, params)
    }

    /// Queries dispatch prompts, mapping returned rows, after validating strict FIFO ordering.
    pub fn query_dispatch_prompts<T, F>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
        mut row_mapper: F,
    ) -> Result<Vec<T>, FifoTransactionViolation>
    where
        F: FnMut(&Row<'_>) -> rusqlite::Result<T>,
    {
        self.intercept_and_verify(sql)?;
        let tx = self.tx.as_ref().ok_or_else(|| {
            FifoTransactionViolation::TransactionAlreadyAborted {
                reason: "Transaction was already consumed or rolled back".to_string(),
            }
        })?;

        let mut stmt = tx.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row_mapper(row))?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Aborts the transaction immediately, rolling back changes and marking it poisoned.
    pub fn abort_transaction(&mut self, reason: &str) {
        if self.is_aborted {
            return;
        }

        self.is_aborted = true;
        self.abort_reason = Some(reason.to_string());

        warn!(
            target: "db_orm::fifo_guard",
            reason = %reason,
            "FIFO invariant violation detected: aborting and rolling back transaction"
        );

        if let Some(tx) = self.tx.take() {
            // Justification: Rollback transaction upon detected FIFO ordering violation.
            crate::error::record_ignored(tx.rollback(), "fifo_guard_abort_rollback");
        }
    }

    /// Commits the active transaction, failing if it was previously aborted or poisoned.
    pub fn commit(mut self) -> Result<(), FifoTransactionViolation> {
        if self.is_aborted {
            return Err(FifoTransactionViolation::TransactionAlreadyAborted {
                reason: self
                    .abort_reason
                    .take()
                    .unwrap_or_else(|| "Unknown abort reason".to_string()),
            });
        }

        let tx = self.tx.take().ok_or_else(|| {
            FifoTransactionViolation::TransactionAlreadyAborted {
                reason: "Transaction was already consumed or rolled back".to_string(),
            }
        })?;

        tx.commit()?;
        Ok(())
    }

    /// Explicitly rolls back the transaction.
    pub fn rollback(mut self) -> Result<(), rusqlite::Error> {
        if let Some(tx) = self.tx.take() {
            tx.rollback()?;
        }
        self.is_aborted = true;
        Ok(())
    }
}

impl<'conn> Drop for FifoPromptRestoreTransactionGuard<'conn> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            // Justification: Best-effort rollback when guard is dropped without explicit commit.
            crate::error::record_ignored(tx.rollback(), "fifo_guard_drop_rollback");
        }
    }
}
