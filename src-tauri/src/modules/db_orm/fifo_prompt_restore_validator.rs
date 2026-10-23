//! FIFO Prompt Restore & Queue Dispatch Validator (`Imp-623`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict
//! First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`.
//! Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`)
//! are strictly forbidden in dispatch logic."

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Mandatory FIFO ordering clause enforced across all prompt restore and queue dispatch queries.
pub const MANDATORY_FIFO_ORDER_CLAUSE: &str = "ORDER BY created_at ASC, id ASC";

/// Categorizes the context of prompt queries for FIFO restoration and queue operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptRestoreQueryKind {
    ActivePromptsRestore,
    PromptBackupsRestore,
    RepoPromptsDispatch,
    GeneralPromptQuery,
}

impl Default for PromptRestoreQueryKind {
    fn default() -> Self {
        Self::GeneralPromptQuery
    }
}

/// Enumerates violations against the strict FIFO ordering invariant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoOrderingViolation {
    ForbiddenLifoDetected {
        query_snippet: String,
        forbidden_clause: String,
    },
    MissingIdTieBreaker {
        query_snippet: String,
    },
    MissingCreatedAtSort {
        query_snippet: String,
    },
}

impl fmt::Display for FifoOrderingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenLifoDetected {
                query_snippet,
                forbidden_clause,
            } => {
                write!(
                    f,
                    "Forbidden LIFO ordering detected: '{forbidden_clause}' in query snippet '{query_snippet}'"
                )
            }
            Self::MissingIdTieBreaker { query_snippet } => {
                write!(
                    f,
                    "Missing mandatory 'id ASC' tie-breaker in query snippet '{query_snippet}'"
                )
            }
            Self::MissingCreatedAtSort { query_snippet } => {
                write!(
                    f,
                    "Missing mandatory 'created_at ASC' primary sort in query snippet '{query_snippet}'"
                )
            }
        }
    }
}

impl std::error::Error for FifoOrderingViolation {}

/// Report resulting from FIFO validation of a prompt restore or dispatch query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoValidationReport {
    pub is_valid: bool,
    pub query_kind: PromptRestoreQueryKind,
    pub normalized_order_clause: String,
    pub violation: Option<FifoOrderingViolation>,
}

impl FifoValidationReport {
    pub fn valid(kind: PromptRestoreQueryKind, normalized_order_clause: String) -> Self {
        Self {
            is_valid: true,
            query_kind: kind,
            normalized_order_clause,
            violation: None,
        }
    }

    pub fn invalid(
        kind: PromptRestoreQueryKind,
        normalized_order_clause: String,
        violation: FifoOrderingViolation,
    ) -> Self {
        Self {
            is_valid: false,
            query_kind: kind,
            normalized_order_clause,
            violation: Some(violation),
        }
    }
}

/// Generates a compact, single-line representation of an SQL query for diagnostic reporting.
fn make_query_snippet(sql: &str) -> String {
    let single_line = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.len() <= 80 {
        single_line
    } else {
        format!("{}...", &single_line[..77])
    }
}

/// Locates the byte range `(start, end)` of the `ORDER BY` clause inside an SQL query.
fn find_order_by_span(sql: &str) -> Option<(usize, usize)> {
    let order_by_re = Regex::new(r"(?i)\border\s+by\b").ok()?;
    let mat = order_by_re.find(sql)?;
    let start_idx = mat.start();
    let term_re = Regex::new(r"(?i)\b(?:limit|offset)\b|;").ok()?;
    let end_idx = match term_re.find(&sql[start_idx..]) {
        Some(t_mat) => start_idx + t_mat.start(),
        None => sql.len(),
    };
    Some((start_idx, end_idx))
}

/// Extracts and collapses whitespace of an existing `ORDER BY` clause from an SQL query.
fn extract_and_normalize_order_by(sql: &str) -> Option<String> {
    let (start, end) = find_order_by_span(sql)?;
    let raw_clause = &sql[start..end];
    Some(raw_clause.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Detects forbidden LIFO sorting patterns (`DESC` on `created_at`, `updated_at`, or `rowid`).
fn detect_forbidden_lifo(sql: &str) -> Option<String> {
    let lifo_re = Regex::new(
        r"(?i)\b(?:[a-zA-Z0-9_]+\.)?(?:created_at|updated_at|rowid)\s+desc\b",
    ).ok()?;

    if let Some(mat) = lifo_re.find(sql) {
        if let Some((start, end)) = find_order_by_span(sql) {
            let clause = sql[start..end].trim();
            if lifo_re.is_match(clause) {
                return Some(clause.split_whitespace().collect::<Vec<_>>().join(" "));
            }
        }
        return Some(mat.as_str().to_string());
    }
    None
}

/// Verifies whether the `ORDER BY` clause designates `created_at ASC` as its primary sort expression.
fn has_created_at_asc(order_clause: &str) -> bool {
    let Ok(re) = Regex::new(
        r"(?i)^order\s+by\s+(?:[a-zA-Z0-9_]+\.)?created_at\s+asc(?:\s*,|\s*$)",
    ) else {
        return false;
    };
    let single_line = order_clause.split_whitespace().collect::<Vec<_>>().join(" ");
    re.is_match(&single_line)
}

/// Verifies whether the `ORDER BY` clause contains the mandatory `id ASC` secondary tie-breaker.
fn has_id_asc_tiebreaker(order_clause: &str) -> bool {
    let Ok(re) = Regex::new(
        r"(?i),\s*(?:[a-zA-Z0-9_]+\.)?id\s+asc(?:\s*,|\s*$)",
    ) else {
        return false;
    };
    let single_line = order_clause.split_whitespace().collect::<Vec<_>>().join(" ");
    re.is_match(&single_line)
}

/// Identifies the first FIFO ordering violation present in the given SQL string.
fn check_fifo_violation(sql: &str) -> Option<FifoOrderingViolation> {
    let snippet = make_query_snippet(sql);
    if let Some(forbidden_clause) = detect_forbidden_lifo(sql) {
        return Some(FifoOrderingViolation::ForbiddenLifoDetected {
            query_snippet: snippet,
            forbidden_clause,
        });
    }
    let Some((start, end)) = find_order_by_span(sql) else {
        return Some(FifoOrderingViolation::MissingCreatedAtSort {
            query_snippet: snippet,
        });
    };
    let order_clause = &sql[start..end];
    if !has_created_at_asc(order_clause) {
        return Some(FifoOrderingViolation::MissingCreatedAtSort {
            query_snippet: snippet,
        });
    }
    if !has_id_asc_tiebreaker(order_clause) {
        return Some(FifoOrderingViolation::MissingIdTieBreaker {
            query_snippet: snippet,
        });
    }
    None
}

/// Formats prefix and suffix SQL segments around the mandatory FIFO order clause.
fn format_sql_with_clause(prefix: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        format!("{prefix} {MANDATORY_FIFO_ORDER_CLAUSE}")
    } else if suffix.starts_with(';') {
        format!("{prefix} {MANDATORY_FIFO_ORDER_CLAUSE}{suffix}")
    } else {
        format!("{prefix} {MANDATORY_FIFO_ORDER_CLAUSE} {suffix}")
    }
}

/// Replaces an existing `ORDER BY` clause span with the canonical FIFO order clause.
fn replace_order_by_span(query: &str, start: usize, end: usize) -> String {
    let prefix = query[..start].trim_end();
    let suffix = query[end..].trim_start();
    format_sql_with_clause(prefix, suffix)
}

/// Inserts the mandatory FIFO order clause into a query lacking an `ORDER BY` clause.
fn insert_fifo_order_by(query: &str) -> String {
    let term_re = Regex::new(r"(?i)\b(?:limit|offset)\b|;").ok();
    if let Some(re) = term_re {
        if let Some(mat) = re.find(query) {
            let prefix = query[..mat.start()].trim_end();
            let suffix = query[mat.start()..].trim_start();
            return format_sql_with_clause(prefix, suffix);
        }
    }
    format!("{query} {MANDATORY_FIFO_ORDER_CLAUSE}")
}

/// Validates an SQL query against strict FIFO restoration invariants.
pub fn validate_prompt_restore_sql(
    sql: &str,
    kind: PromptRestoreQueryKind,
) -> FifoValidationReport {
    let violation = check_fifo_violation(sql);
    let is_valid = violation.is_none();
    let normalized_order_clause = if is_valid {
        MANDATORY_FIFO_ORDER_CLAUSE.to_string()
    } else {
        extract_and_normalize_order_by(sql).unwrap_or_default()
    };

    FifoValidationReport {
        is_valid,
        query_kind: kind,
        normalized_order_clause,
        violation,
    }
}

/// Sanitizes an SQL query by enforcing or replacing its `ORDER BY` clause with the canonical FIFO clause.
pub fn sanitize_or_enforce_fifo_clause(raw_query: &str) -> String {
    let trimmed = raw_query.trim();
    if trimmed.is_empty() {
        return MANDATORY_FIFO_ORDER_CLAUSE.to_string();
    }
    if let Some((start, end)) = find_order_by_span(trimmed) {
        replace_order_by_span(trimmed, start, end)
    } else {
        insert_fifo_order_by(trimmed)
    }
}
