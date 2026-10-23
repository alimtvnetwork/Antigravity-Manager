//! Strict FIFO Queue Compaction Guard & Tombstone Purge Verifier (`Imp-692`).
//!
//! Enforces the AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`,
//! `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! This module coordinates SQLite database queue compaction, historical archiving, and tombstone cleanup,
//! ensuring that compaction passes strictly preserve original chronological enqueue order, never invert
//! row sequences, and safely purge acknowledged/tombstoned rows without creating sequence gaps or deadlocks.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Canonical FIFO ORDER BY clause required across all prompt queue compaction queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Known SQLite tables subject to strict FIFO compaction and tombstone cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompactionTable {
    /// Active prompt processing queue (`active_prompts`).
    ActivePrompts,
    /// Backup and historical prompt restore store (`prompt_backups`).
    PromptBackups,
    /// Repository-level prompt storage database (`repo_prompts`).
    RepoPrompts,
}

impl CompactionTable {
    /// Returns the canonical SQLite table name.
    pub fn table_name(&self) -> &'static str {
        match self {
            Self::ActivePrompts => "active_prompts",
            Self::PromptBackups => "prompt_backups",
            Self::RepoPrompts => "repo_prompts",
        }
    }

    /// Resolves a table name string into a recognized `CompactionTable` enum variant.
    pub fn from_table_name(table_name: &str) -> Option<Self> {
        let normalized = table_name
            .trim()
            .trim_matches('`')
            .trim_matches('"')
            .trim_matches('[');
        let normalized = normalized.trim_end_matches(']');

        if normalized.eq_ignore_ascii_case("active_prompts") {
            Some(Self::ActivePrompts)
        } else if normalized.eq_ignore_ascii_case("prompt_backups") {
            Some(Self::PromptBackups)
        } else if normalized.eq_ignore_ascii_case("repo_prompts")
            || normalized.eq_ignore_ascii_case("repo_prompts.db")
        {
            Some(Self::RepoPrompts)
        } else {
            None
        }
    }

    /// Returns an array of all recognized compaction tables.
    pub fn all() -> &'static [CompactionTable] {
        &[Self::ActivePrompts, Self::PromptBackups, Self::RepoPrompts]
    }
}

/// An individual queue record evaluated during a compaction pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueRow {
    /// Primary key identifier of the prompt queue row.
    pub id: i64,
    /// Unix epoch timestamp in milliseconds when the prompt was enqueued.
    pub created_at: i64,
    /// Lifecycle status of the prompt row (e.g., "pending", "processing", "tombstone", "completed").
    pub status: String,
    /// Flag indicating whether this row is marked as a tombstone eligible for purge.
    pub is_tombstone: bool,
}

impl QueueRow {
    /// Constructs a new queue row.
    pub fn new(id: i64, created_at: i64, status: impl Into<String>, is_tombstone: bool) -> Self {
        Self {
            id,
            created_at,
            status: status.into(),
            is_tombstone,
        }
    }

    /// Constructs an active queue row.
    pub fn active(id: i64, created_at: i64, status: impl Into<String>) -> Self {
        Self::new(id, created_at, status, false)
    }

    /// Constructs a tombstone queue row ready for compaction purge.
    pub fn tombstone(id: i64, created_at: i64) -> Self {
        Self::new(id, created_at, "tombstone", true)
    }

    /// Returns true if this row represents a purgeable tombstone.
    pub fn is_purgeable(&self) -> bool {
        self.is_tombstone
            || self.status.eq_ignore_ascii_case("tombstone")
            || self.status.eq_ignore_ascii_case("purged")
            || self.status.eq_ignore_ascii_case("deleted")
    }
}

/// Violations detected during compaction query SQL validation or batch compaction execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionViolation {
    /// The query string was empty or contained only whitespace.
    EmptyQuery,
    /// The SQL query is missing a mandatory ORDER BY clause.
    MissingOrderByClause { sql: String },
    /// An inverted LIFO ordering clause was found (e.g., `DESC`).
    InvertedLifoOrder { found: String },
    /// The ORDER BY clause is missing the secondary tie-breaker `id ASC`.
    MissingIdTieBreaker { found: String },
    /// The ORDER BY clause references unexpected columns or invalid sort clauses.
    InvalidOrderByClause { found: String, expected: String },
    /// A row in the compaction stream regressed to an earlier creation timestamp.
    TimestampRegression {
        prev_id: i64,
        prev_created_at: i64,
        curr_id: i64,
        curr_created_at: i64,
    },
    /// Identical timestamps must be tie-broken by strictly increasing ID.
    IdTieBreakerInversion {
        created_at: i64,
        prev_id: i64,
        curr_id: i64,
    },
    /// The same primary key ID was encountered more than once during compaction.
    DuplicateId { id: i64 },
    /// The targeted table is not a recognized prompt compaction table.
    TargetTableMismatch { table_name: String },
}

impl fmt::Display for CompactionViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuery => write!(f, "Compaction query SQL cannot be empty"),
            Self::MissingOrderByClause { sql } => write!(
                f,
                "Missing mandatory canonical FIFO '{CANONICAL_FIFO_ORDER_BY}' clause in compaction query: {sql}"
            ),
            Self::InvertedLifoOrder { found } => write!(
                f,
                "Inverted LIFO ordering forbidden: found '{found}', expected '{CANONICAL_FIFO_ORDER_BY}'"
            ),
            Self::MissingIdTieBreaker { found } => write!(
                f,
                "Missing mandatory secondary tie-breaker 'id ASC' in ORDER BY clause: found '{found}'"
            ),
            Self::InvalidOrderByClause { found, expected } => write!(
                f,
                "Invalid ORDER BY clause: found '{found}', expected '{expected}'"
            ),
            Self::TimestampRegression {
                prev_id,
                prev_created_at,
                curr_id,
                curr_created_at,
            } => write!(
                f,
                "Timestamp regression detected: row #{curr_id} (created_at={curr_created_at}) is older than row #{prev_id} (created_at={prev_created_at})"
            ),
            Self::IdTieBreakerInversion {
                created_at,
                prev_id,
                curr_id,
            } => write!(
                f,
                "ID tie-breaker inversion at created_at={created_at}: row #{curr_id} <= previous row #{prev_id}"
            ),
            Self::DuplicateId { id } => {
                write!(f, "Duplicate row ID #{id} detected during compaction pass")
            }
            Self::TargetTableMismatch { table_name } => write!(
                f,
                "Table '{table_name}' is not a recognized FIFO compaction queue table"
            ),
        }
    }
}

impl std::error::Error for CompactionViolation {}

/// Comprehensive audit report generated following a FIFO compaction and tombstone purge pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionAuditReport {
    /// True if original chronological ordering was preserved without inversions.
    pub is_order_preserved: bool,
    /// Total number of queue rows inspected prior to compaction.
    pub total_rows_before: usize,
    /// Number of tombstoned or acknowledged rows purged during compaction.
    pub tombstones_purged: usize,
    /// Number of active rows retained after compaction.
    pub retained_rows_count: usize,
    /// List of violations detected during compaction.
    pub violations: Vec<String>,
    /// Retained active rows in strict FIFO order.
    pub retained_rows: Vec<QueueRow>,
}

impl CompactionAuditReport {
    /// Creates a new compaction audit report.
    pub fn new(
        is_order_preserved: bool,
        total_rows_before: usize,
        tombstones_purged: usize,
        retained_rows_count: usize,
        violations: Vec<String>,
        retained_rows: Vec<QueueRow>,
    ) -> Self {
        Self {
            is_order_preserved,
            total_rows_before,
            tombstones_purged,
            retained_rows_count,
            violations,
            retained_rows,
        }
    }

    /// Returns true if the compaction pass was completely clean and order-preserved.
    pub fn is_clean(&self) -> bool {
        self.is_order_preserved && self.violations.is_empty()
    }

    /// Returns whether chronological order was strictly maintained.
    pub fn is_order_preserved(&self) -> bool {
        self.is_order_preserved
    }

    /// Total rows prior to compaction.
    pub fn total_rows_before(&self) -> usize {
        self.total_rows_before
    }

    /// Count of purged tombstones.
    pub fn tombstones_purged(&self) -> usize {
        self.tombstones_purged
    }

    /// Count of retained active rows.
    pub fn retained_rows_count(&self) -> usize {
        self.retained_rows_count
    }

    /// List of stringified violations.
    pub fn violations(&self) -> &[String] {
        &self.violations
    }

    /// Retained rows slice.
    pub fn retained_rows(&self) -> &[QueueRow] {
        &self.retained_rows
    }
}

/// Locates the byte offset immediately following `ORDER BY` with word boundary checks.
fn find_order_by_boundary(upper: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find("ORDER") {
        let abs_order = search_from + pos;
        let before_ok = abs_order == 0
            || upper
                .as_bytes()
                .get(abs_order - 1)
                .map_or(true, |b| b.is_ascii_whitespace() || *b == b')');
        let after_order = abs_order + 5;
        let after_ok =
            after_order < upper.len() && upper.as_bytes()[after_order].is_ascii_whitespace();

        if before_ok && after_ok {
            let rest = &upper[after_order..];
            let trimmed = rest.trim_start();
            let whitespace_len = rest.len() - trimmed.len();
            if trimmed.starts_with("BY") {
                let after_by = 2;
                let by_end_ok = after_by >= trimmed.len()
                    || trimmed
                        .as_bytes()
                        .get(after_by)
                        .map_or(true, |b| b.is_ascii_whitespace());
                if by_end_ok {
                    return Some(after_order + whitespace_len + 2);
                }
            }
        }
        search_from = abs_order + 5;
    }
    None
}

/// Finds the starting index of `ORDER BY` with word boundaries.
fn find_order_by_index(upper: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find("ORDER") {
        let abs_order = search_from + pos;
        let before_ok = abs_order == 0
            || upper
                .as_bytes()
                .get(abs_order - 1)
                .map_or(true, |b| b.is_ascii_whitespace() || *b == b')');
        let after_order = abs_order + 5;
        let after_ok =
            after_order < upper.len() && upper.as_bytes()[after_order].is_ascii_whitespace();

        if before_ok && after_ok {
            let rest = &upper[after_order..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with("BY") {
                let after_by = 2;
                let by_end_ok = after_by >= trimmed.len()
                    || trimmed
                        .as_bytes()
                        .get(after_by)
                        .map_or(true, |b| b.is_ascii_whitespace());
                if by_end_ok {
                    return Some(abs_order);
                }
            }
        }
        search_from = abs_order + 5;
    }
    None
}

/// Determines the end boundary of an ORDER BY clause (before LIMIT, OFFSET, FOR, or ';').
fn find_clause_end_pos(upper_rem: &str) -> usize {
    let mut end_idx = upper_rem.len();
    for marker in &["LIMIT", "OFFSET", "FOR", ";"] {
        let mut search_from = 0;
        while let Some(pos) = upper_rem[search_from..].find(marker) {
            let abs_pos = search_from + pos;
            if *marker == ";" {
                if abs_pos < end_idx {
                    end_idx = abs_pos;
                }
                break;
            }
            let before_ok = abs_pos == 0
                || upper_rem
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map_or(false, |b| b.is_ascii_whitespace());
            let after_marker = abs_pos + marker.len();
            let after_ok = after_marker >= upper_rem.len()
                || upper_rem
                    .as_bytes()
                    .get(after_marker)
                    .map_or(false, |b| b.is_ascii_whitespace());
            if before_ok && after_ok {
                if abs_pos < end_idx {
                    end_idx = abs_pos;
                }
                break;
            }
            search_from = abs_pos + marker.len();
        }
    }
    end_idx
}

/// Strips surrounding quote characters or table alias prefixes from a SQL column name.
fn clean_column_name(token: &str) -> String {
    let unquoted =
        token.trim_matches(|c: char| c == '`' || c == '"' || c == '[' || c == ']' || c == '\'');
    if let Some((_, col)) = unquoted.rsplit_once('.') {
        col.to_string()
    } else {
        unquoted.to_string()
    }
}

/// Extracts the raw ORDER BY clause from an SQL statement.
pub fn extract_order_by_clause(sql: &str) -> Option<String> {
    let upper = sql.to_uppercase();
    let by_end = find_order_by_boundary(&upper)?;
    let remaining = sql[by_end..].trim();
    if remaining.is_empty() {
        return None;
    }

    let upper_rem = remaining.to_uppercase();
    let end_idx = find_clause_end_pos(&upper_rem);
    let clause = remaining[..end_idx].trim();
    if clause.is_empty() {
        None
    } else {
        Some(clause.to_string())
    }
}

/// Validates that an SQL query targeting queue compaction enforces strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).
///
/// Returns `Ok(())` if fully compliant, or `Err(CompactionViolation)` identifying the defect.
pub fn validate_compaction_query_sql(sql: &str) -> Result<(), CompactionViolation> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(CompactionViolation::EmptyQuery);
    }

    let clause = match extract_order_by_clause(trimmed) {
        Some(c) => c,
        None => {
            return Err(CompactionViolation::MissingOrderByClause {
                sql: trimmed.to_string(),
            })
        }
    };

    let upper_clause = clause.to_uppercase();

    // Check for explicit inverted DESC ordering
    for token in upper_clause.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean == "DESC" {
            return Err(CompactionViolation::InvertedLifoOrder { found: clause });
        }
    }

    let parts: Vec<&str> = clause
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(CompactionViolation::MissingOrderByClause {
            sql: trimmed.to_string(),
        });
    }

    // Check primary sort column: must be created_at (optionally qualified)
    let first_tokens: Vec<&str> = parts[0].split_whitespace().collect();
    if first_tokens.is_empty() {
        return Err(CompactionViolation::MissingOrderByClause {
            sql: trimmed.to_string(),
        });
    }

    let primary_col = clean_column_name(first_tokens[0]);
    if !primary_col.eq_ignore_ascii_case("created_at") {
        return Err(CompactionViolation::InvalidOrderByClause {
            found: parts[0].to_string(),
            expected: "created_at ASC".to_string(),
        });
    }

    // Check secondary sort column: must exist and be id ASC
    if parts.len() < 2 {
        return Err(CompactionViolation::MissingIdTieBreaker { found: clause });
    }

    let second_tokens: Vec<&str> = parts[1].split_whitespace().collect();
    if second_tokens.is_empty() {
        return Err(CompactionViolation::MissingIdTieBreaker { found: clause });
    }

    let secondary_col = clean_column_name(second_tokens[0]);
    if !secondary_col.eq_ignore_ascii_case("id") {
        return Err(CompactionViolation::InvalidOrderByClause {
            found: parts[1].to_string(),
            expected: "id ASC".to_string(),
        });
    }

    Ok(())
}

/// Executes a FIFO queue compaction pass over a slice of queue rows.
///
/// Ensures:
/// 1. Original chronological enqueue order (`created_at ASC, id ASC`) is strictly maintained without regressions.
/// 2. Purgeable tombstones are safely eliminated.
/// 3. Retained rows maintain deterministic FIFO continuity.
pub fn execute_fifo_compaction(rows: &[QueueRow]) -> CompactionAuditReport {
    let total_rows_before = rows.len();
    let mut tombstones_purged = 0;
    let mut retained_rows = Vec::with_capacity(total_rows_before);
    let mut violations = Vec::new();
    let mut seen_ids = HashSet::new();

    let mut prev_active: Option<&QueueRow> = None;

    for row in rows {
        // Check for duplicate primary keys across the compaction batch
        if !seen_ids.insert(row.id) {
            violations.push(CompactionViolation::DuplicateId { id: row.id }.to_string());
        }

        if row.is_purgeable() {
            tombstones_purged += 1;
            continue;
        }

        // Verify monotonicity with respect to preceding active row
        if let Some(prev) = prev_active {
            if row.created_at < prev.created_at {
                violations.push(
                    CompactionViolation::TimestampRegression {
                        prev_id: prev.id,
                        prev_created_at: prev.created_at,
                        curr_id: row.id,
                        curr_created_at: row.created_at,
                    }
                    .to_string(),
                );
            } else if row.created_at == prev.created_at && row.id <= prev.id {
                violations.push(
                    CompactionViolation::IdTieBreakerInversion {
                        created_at: row.created_at,
                        prev_id: prev.id,
                        curr_id: row.id,
                    }
                    .to_string(),
                );
            }
        }

        retained_rows.push(row.clone());
        prev_active = Some(row);
    }

    let is_order_preserved = violations.is_empty();
    let retained_rows_count = retained_rows.len();

    CompactionAuditReport::new(
        is_order_preserved,
        total_rows_before,
        tombstones_purged,
        retained_rows_count,
        violations,
        retained_rows,
    )
}

/// Builds a canonical SQLite compaction selection query enforcing strict FIFO order.
pub fn build_canonical_compaction_query(table: CompactionTable) -> String {
    format!(
        "SELECT id, created_at, status, is_tombstone FROM {} {}",
        table.table_name(),
        CANONICAL_FIFO_ORDER_BY
    )
}

/// Builds a canonical SQLite tombstone cleanup DELETE query for a given queue table.
pub fn build_canonical_tombstone_purge_query(table: CompactionTable) -> String {
    format!(
        "DELETE FROM {} WHERE is_tombstone = 1 OR status = 'tombstone'",
        table.table_name()
    )
}

/// Enforces canonical FIFO ordering on any SQL query, replacing or appending `ORDER BY created_at ASC, id ASC`.
pub fn enforce_compaction_order_by(sql: &str) -> String {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    let upper = trimmed.to_uppercase();

    if let Some(order_start) = find_order_by_index(&upper) {
        let before_order = trimmed[..order_start].trim_end();
        let after_order = &upper[order_start..];
        let by_end = find_order_by_boundary(after_order).unwrap_or(8);
        let upper_rem = &after_order[by_end..];
        let end_idx = find_clause_end_pos(upper_rem);
        let trailing_start = order_start + by_end + end_idx;
        let trailing = trimmed[trailing_start..].trim();

        if trailing.is_empty() {
            format!("{before_order} {CANONICAL_FIFO_ORDER_BY}")
        } else {
            format!("{before_order} {CANONICAL_FIFO_ORDER_BY} {trailing}")
        }
    } else {
        // No ORDER BY present; check if there is LIMIT / OFFSET
        let end_idx = find_clause_end_pos(&upper);
        if end_idx < trimmed.len() {
            let before = trimmed[..end_idx].trim_end();
            let trailing = trimmed[end_idx..].trim();
            format!("{before} {CANONICAL_FIFO_ORDER_BY} {trailing}")
        } else {
            format!("{trimmed} {CANONICAL_FIFO_ORDER_BY}")
        }
    }
}
