//! Strict FIFO Prompt Order Enforcer & SQL Transformer (`Imp-654`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."

use serde::{Deserialize, Serialize};

/// Canonical FIFO ORDER BY clause required across all prompt restore and queue dispatch queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Known prompt queue tables subject to strict FIFO ordering enforcement.
pub const PROMPT_QUEUE_TABLES: &[&str] = &[
    "active_prompts",
    "prompt_backups",
    "repo_prompts",
    "prompts_queue",
    "prompt_queue",
];

/// Actions taken by the FIFO query enforcer when evaluating or transforming SQL statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoEnforcementAction {
    /// The query already strictly matches canonical FIFO ordering.
    AlreadyCompliant,
    /// An existing ORDER BY clause (e.g. inverted LIFO or missing tie-breaker) was replaced with canonical FIFO.
    ReplacedLIFOOrder {
        original_clause: String,
        canonical_clause: String,
    },
    /// The query was missing an ORDER BY clause and canonical FIFO ordering was appended.
    AppendedCanonicalOrder,
    /// The query does not target a prompt queue table and passed through unmodified.
    NonTargetQueryPassThrough,
    /// The query was malformed or empty and could not be safely sanitized.
    RejectedUnsanitizableQuery { reason: String },
}

/// Comprehensive report resulting from FIFO SQL ordering enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoEnforcementReport {
    /// The action taken on the query.
    pub action: FifoEnforcementAction,
    /// The original input SQL query.
    pub original_sql: String,
    /// The resulting enforced SQL query.
    pub enforced_sql: String,
    /// Whether the SQL query was altered during enforcement.
    pub is_modified: bool,
    /// Whether the query is guaranteed to execute with FIFO semantics.
    pub is_fifo_guaranteed: bool,
}

impl FifoEnforcementReport {
    pub fn new(
        action: FifoEnforcementAction,
        original_sql: String,
        enforced_sql: String,
        is_modified: bool,
        is_fifo_guaranteed: bool,
    ) -> Self {
        Self {
            action,
            original_sql,
            enforced_sql,
            is_modified,
            is_fifo_guaranteed,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn is_fifo_guaranteed(&self) -> bool {
        self.is_fifo_guaranteed
    }

    pub fn is_compliant(&self) -> bool {
        matches!(self.action, FifoEnforcementAction::AlreadyCompliant)
    }

    pub fn action(&self) -> &FifoEnforcementAction {
        &self.action
    }

    pub fn enforced_sql(&self) -> &str {
        &self.enforced_sql
    }
}

/// Determines if a given table identifier matches one of the prompt queue tables.
pub fn is_prompt_queue_table(table_name: &str) -> bool {
    let normalized = table_name
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase();
    PROMPT_QUEUE_TABLES
        .iter()
        .any(|&known| normalized == known || normalized.ends_with(&format!(".{known}")))
}

/// Detects the target table name from an SQL statement.
pub fn detect_target_table(sql: &str) -> Option<String> {
    let lower = sql.to_ascii_lowercase();
    for &known in PROMPT_QUEUE_TABLES {
        if lower.contains(known) {
            return Some(known.to_string());
        }
    }

    let tokens: Vec<&str> = sql.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if token.eq_ignore_ascii_case("FROM") && i + 1 < tokens.len() {
            let table =
                tokens[i + 1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '.');
            if !table.is_empty() {
                return Some(table.to_string());
            }
        }
    }
    None
}

/// Finds the starting index of `ORDER BY` with word boundaries.
fn find_order_by_index(upper: &str) -> Option<usize> {
    let needle = "ORDER BY";
    let mut search_from = 0;
    while let Some(rel) = upper[search_from..].find(needle) {
        let start = search_from + rel;
        let end = start + needle.len();

        let prefix_ok = start == 0
            || upper[..start]
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ')')
                .unwrap_or(true);
        let suffix_ok = end >= upper.len()
            || upper[end..]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(true);

        if prefix_ok && suffix_ok {
            return Some(start);
        }
        search_from = end;
    }
    None
}

/// Finds the next terminal keyword index after ORDER BY (e.g. LIMIT, OFFSET, FOR, or trailing semicolon).
fn find_clause_end(upper: &str, start_pos: usize) -> usize {
    let tail = &upper[start_pos..];
    let mut min_end = tail.len();

    for keyword in &["LIMIT", "OFFSET", "FOR UPDATE", ";"] {
        if let Some(pos) = tail.find(keyword) {
            if pos < min_end {
                min_end = pos;
            }
        }
    }

    start_pos + min_end
}

/// Normalizes an ORDER BY clause string for exact comparison (ignoring case and whitespace differences).
fn normalize_clause(clause: &str) -> String {
    clause
        .to_ascii_lowercase()
        .replace("order by", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Analyzes and enforces strict FIFO ordering on the provided SQL query.
///
/// If the query targets a prompt queue table:
/// - Replaces non-FIFO / LIFO `ORDER BY` clauses with `ORDER BY created_at ASC, id ASC`.
/// - Inserts `ORDER BY created_at ASC, id ASC` before `LIMIT`/`OFFSET` or semicolons if missing.
/// - Leaves queries that are already strictly FIFO compliant untouched.
/// If the query targets other tables, passes through unchanged.
pub fn enforce_fifo_ordering(sql: &str) -> FifoEnforcementReport {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return FifoEnforcementReport::new(
            FifoEnforcementAction::RejectedUnsanitizableQuery {
                reason: "Query is empty or whitespace-only".to_string(),
            },
            sql.to_string(),
            sql.to_string(),
            false,
            false,
        );
    }

    let target_table = detect_target_table(trimmed);
    let is_prompt_target = target_table
        .as_deref()
        .map(is_prompt_queue_table)
        .unwrap_or(false);

    if !is_prompt_target {
        return FifoEnforcementReport::new(
            FifoEnforcementAction::NonTargetQueryPassThrough,
            sql.to_string(),
            sql.to_string(),
            false,
            true,
        );
    }

    let upper = trimmed.to_ascii_uppercase();

    if let Some(order_by_start) = find_order_by_index(&upper) {
        let clause_end = find_clause_end(&upper, order_by_start + "ORDER BY".len());
        let original_clause = trimmed[order_by_start..clause_end].trim().to_string();

        let norm_existing = normalize_clause(&original_clause);
        let norm_canonical = normalize_clause(CANONICAL_FIFO_ORDER_BY);

        if norm_existing == norm_canonical {
            return FifoEnforcementReport::new(
                FifoEnforcementAction::AlreadyCompliant,
                sql.to_string(),
                sql.to_string(),
                false,
                true,
            );
        }

        let prefix = &trimmed[..order_by_start].trim_end();
        let suffix = &trimmed[clause_end..].trim_start();

        let enforced_sql = if suffix.is_empty() {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY}")
        } else if suffix.starts_with(';') {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY}{suffix}")
        } else {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY} {suffix}")
        };

        FifoEnforcementReport::new(
            FifoEnforcementAction::ReplacedLIFOOrder {
                original_clause,
                canonical_clause: CANONICAL_FIFO_ORDER_BY.to_string(),
            },
            sql.to_string(),
            enforced_sql,
            true,
            true,
        )
    } else {
        // Query has no ORDER BY clause; determine insertion position before LIMIT/OFFSET/semicolon
        let mut insert_pos = trimmed.len();
        for keyword in &["LIMIT", "OFFSET", ";"] {
            if let Some(pos) = upper.find(keyword) {
                if pos < insert_pos {
                    insert_pos = pos;
                }
            }
        }

        let prefix = trimmed[..insert_pos].trim_end();
        let suffix = trimmed[insert_pos..].trim_start();

        let enforced_sql = if suffix.is_empty() {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY}")
        } else if suffix.starts_with(';') {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY}{suffix}")
        } else {
            format!("{prefix} {CANONICAL_FIFO_ORDER_BY} {suffix}")
        };

        FifoEnforcementReport::new(
            FifoEnforcementAction::AppendedCanonicalOrder,
            sql.to_string(),
            enforced_sql,
            true,
            true,
        )
    }
}
