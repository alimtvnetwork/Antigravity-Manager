//! Strict FIFO Prompt Restore Query Validator (`Imp-644`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Mandatory FIFO ordering clause enforced across all prompt restore and queue dispatch queries.
pub const MANDATORY_FIFO_ORDER_CLAUSE: &str = "ORDER BY created_at ASC, id ASC";

// ============================================================================
// Imp-644 API (Primary)
// ============================================================================

/// Violations identified when inspecting prompt restore and dispatch SQL queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoQueryViolation {
    /// An inverted LIFO ordering clause was detected (e.g. `ORDER BY updated_at DESC` or `created_at DESC`).
    InvertedLIFOOrderDetected { clause: String },
    /// The secondary deterministic tie-breaker `id ASC` is missing from the ordering clause.
    MissingTieBreakerIdAsc { clause: String },
    /// The query is missing an `ORDER BY` clause altogether.
    MissingOrderByClause,
    /// The primary sort column is not `created_at ASC` (e.g. sorting by priority or title).
    DisallowedSortColumn { column: String },
}

impl fmt::Display for FifoQueryViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedLIFOOrderDetected { clause } => {
                write!(
                    f,
                    "Inverted LIFO ordering detected in clause '{clause}': queries must use 'ORDER BY created_at ASC, id ASC'"
                )
            }
            Self::MissingTieBreakerIdAsc { clause } => {
                write!(
                    f,
                    "Missing mandatory secondary tie-breaker 'id ASC' in clause '{clause}'"
                )
            }
            Self::MissingOrderByClause => {
                write!(
                    f,
                    "Missing ORDER BY clause: prompt restore queries must enforce strict FIFO ordering"
                )
            }
            Self::DisallowedSortColumn { column } => {
                write!(
                    f,
                    "Disallowed sort column '{column}': primary sort column must be 'created_at ASC'"
                )
            }
        }
    }
}

impl std::error::Error for FifoQueryViolation {}

/// Validation report summarizing the FIFO compliance of an SQL restore/dispatch query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoQueryValidationReport {
    /// True if the query strictly adheres to FIFO ordering invariants.
    pub is_valid_fifo: bool,
    /// The extracted ORDER BY clause string, if present.
    pub order_by_clause: Option<String>,
    /// The detected table name being queried, if identifiable.
    pub table_name: Option<String>,
    /// List of violations detected during inspection.
    pub violations: Vec<FifoQueryViolation>,
}

impl FifoQueryValidationReport {
    /// Creates a new validation report instance.
    pub fn new(
        is_valid_fifo: bool,
        order_by_clause: Option<String>,
        table_name: Option<String>,
        violations: Vec<FifoQueryViolation>,
    ) -> Self {
        Self {
            is_valid_fifo,
            order_by_clause,
            table_name,
            violations,
        }
    }

    /// Returns true if any FIFO ordering violations were detected.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns the number of violations detected.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Extracts the target table name from the query, prioritizing known prompt tables.
pub fn extract_table_name(sql: &str) -> Option<String> {
    let lower = sql.to_lowercase();
    for known in &["active_prompts", "prompt_backups", "repo_prompts"] {
        if lower.contains(known) {
            return Some((*known).to_string());
        }
    }

    let words: Vec<&str> = sql.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if word.eq_ignore_ascii_case("FROM") && i + 1 < words.len() {
            let table = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '.');
            let table_clean = table.trim_end_matches(".db");
            if !table_clean.is_empty() {
                return Some(table_clean.to_string());
            }
        }
    }

    None
}

/// Finds the character offset immediately following `ORDER BY` with word boundary checks.
fn find_order_by_boundary(upper: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find("ORDER") {
        let abs_order = search_from + pos;
        let before_ok = abs_order == 0 || upper.as_bytes().get(abs_order - 1).map_or(true, |b| b.is_ascii_whitespace());
        let after_order = abs_order + 5;
        let after_ok = after_order < upper.len() && upper.as_bytes()[after_order].is_ascii_whitespace();

        if before_ok && after_ok {
            let rest = &upper[after_order..];
            let trimmed = rest.trim_start();
            let whitespace_len = rest.len() - trimmed.len();
            if trimmed.starts_with("BY") {
                let after_by = 2;
                let by_end_ok = after_by >= trimmed.len() || trimmed.as_bytes()[after_by].is_ascii_whitespace();
                if by_end_ok {
                    return Some(after_order + whitespace_len + 2);
                }
            }
        }
        search_from = abs_order + 5;
    }
    None
}

/// Determines the end boundary of the ORDER BY clause (before LIMIT, OFFSET, or ';').
fn find_clause_end_pos(upper_rem: &str) -> usize {
    let mut end_idx = upper_rem.len();
    for marker in &["LIMIT", "OFFSET", ";"] {
        let mut search_from = 0;
        while let Some(pos) = upper_rem[search_from..].find(marker) {
            let abs_pos = search_from + pos;
            if *marker == ";" {
                if abs_pos < end_idx {
                    end_idx = abs_pos;
                }
                break;
            }
            let before_ok = abs_pos == 0 || upper_rem.as_bytes().get(abs_pos - 1).map_or(false, |b| b.is_ascii_whitespace());
            let after_marker = abs_pos + marker.len();
            let after_ok = after_marker >= upper_rem.len() || upper_rem.as_bytes().get(after_marker).map_or(false, |b| b.is_ascii_whitespace());
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

/// Extracts the raw ORDER BY clause text up to trailing LIMIT/OFFSET/semicolon clauses.
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

/// Returns true if the ORDER BY clause contains the required secondary tie-breaker `id ASC`.
fn has_id_asc_tiebreaker(clause: &str) -> bool {
    let parts: Vec<&str> = clause.split(',').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return false;
    }
    parts[1..].iter().any(|part| {
        let upper = part.to_uppercase();
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

/// Inspects the extracted ORDER BY clause and detects any FIFO rule violations.
fn validate_order_clause(clause: &str) -> Vec<FifoQueryViolation> {
    let mut violations = Vec::new();
    let upper_clause = clause.to_uppercase();

    if upper_clause.contains("DESC") {
        violations.push(FifoQueryViolation::InvertedLIFOOrderDetected {
            clause: clause.to_string(),
        });
        return violations;
    }

    let primary_col = extract_primary_sort_column(clause);
    if !primary_col.eq_ignore_ascii_case("created_at") {
        violations.push(FifoQueryViolation::DisallowedSortColumn {
            column: primary_col,
        });
    }

    if !has_id_asc_tiebreaker(clause) {
        violations.push(FifoQueryViolation::MissingTieBreakerIdAsc {
            clause: clause.to_string(),
        });
    }

    violations
}

/// Validates an SQL query string to ensure it adheres to the strict FIFO prompt restore ordering invariant.
pub fn validate_fifo_query(sql: &str) -> FifoQueryValidationReport {
    let table_name = extract_table_name(sql);
    let order_by_clause = extract_order_by_clause(sql);

    let (is_valid_fifo, violations) = match &order_by_clause {
        None => (false, vec![FifoQueryViolation::MissingOrderByClause]),
        Some(clause) => {
            let v = validate_order_clause(clause);
            (v.is_empty(), v)
        }
    };

    FifoQueryValidationReport {
        is_valid_fifo,
        order_by_clause,
        table_name,
        violations,
    }
}

/// Builds a canonical, FIFO-compliant SQL query string for restoring prompts.
pub fn build_canonical_fifo_query(table: &str, limit: Option<usize>) -> String {
    match limit {
        Some(lim) => format!(
            "SELECT * FROM {} ORDER BY created_at ASC, id ASC LIMIT {}",
            table, lim
        ),
        None => format!("SELECT * FROM {} ORDER BY created_at ASC, id ASC", table),
    }
}

// ============================================================================
// Legacy Helpers & Aliases (Imp-623 Backwards Compatibility)
// ============================================================================

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

/// Enumerates violations against the strict FIFO ordering invariant (legacy representation).
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

/// Report resulting from FIFO validation of a prompt restore or dispatch query (legacy).
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
fn has_legacy_id_asc_tiebreaker(order_clause: &str) -> bool {
    let Ok(re) = Regex::new(
        r"(?i),\s*(?:[a-zA-Z0-9_]+\.)?id\s+asc(?:\s*,|\s*$)",
    ) else {
        return false;
    };
    let single_line = order_clause.split_whitespace().collect::<Vec<_>>().join(" ");
    re.is_match(&single_line)
}

/// Identifies the first FIFO ordering violation present in the given SQL string (legacy).
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
    if !has_legacy_id_asc_tiebreaker(order_clause) {
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

/// Validates an SQL query against strict FIFO restoration invariants (legacy API).
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
