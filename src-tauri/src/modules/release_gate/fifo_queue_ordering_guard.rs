//! FIFO Queue Ordering Guard & Dispatch Query Normalizer.
//!
//! Enforces prompt queue discipline per AGENTS.md:
//! All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict
//! First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`.
//! Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`)
//! are strictly forbidden in dispatch logic.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical FIFO ordering clause required for all prompt queue and dispatch queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Canonical FIFO ordering clause terms without the ORDER BY keyword.
pub const CANONICAL_FIFO_ORDER_TERMS: &str = "created_at ASC, id ASC";

/// Known SQLite prompt queue tables and database targets monitored for FIFO compliance.
pub const MONITORED_TARGETS: &[&str] = &[
    "active_prompts",
    "prompt_backups",
    "repo_prompts.db",
    "repo_prompts",
];

/// Specific violations when inspecting SQL queries against strict FIFO ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoOrderingViolation {
    /// Inverted LIFO ordering (e.g. `updated_at DESC`, `created_at DESC`) is strictly forbidden.
    LIFOOrderingForbidden { query: String, details: String },
    /// Missing ORDER BY clause on a monitored queue/prompt table.
    MissingOrderByClause { query: String, target_table: String },
    /// Primary sort is valid, but missing deterministic ascending tie-breaker (`id ASC`).
    MissingAscendingTieBreaker {
        query: String,
        found_order: String,
        details: String,
    },
    /// A DESC sort direction was encountered where ASC is mandatory.
    DescendingDirectionForbidden { query: String, column: String },
    /// An unexpected primary ordering column was encountered (expected `created_at`).
    InvalidPrimaryOrderColumn { query: String, found_column: String },
    /// Empty or whitespace-only query string.
    EmptyQuery,
}

impl fmt::Display for FifoOrderingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LIFOOrderingForbidden { query, details } => {
                write!(
                    f,
                    "LIFO ordering is strictly forbidden in dispatch queries: {details} (query: {query})"
                )
            }
            Self::MissingOrderByClause {
                query,
                target_table,
            } => {
                write!(
                    f,
                    "Missing ORDER BY clause for monitored table '{target_table}' (query: {query})"
                )
            }
            Self::MissingAscendingTieBreaker {
                query,
                found_order,
                details,
            } => {
                write!(
                    f,
                    "Missing ascending tie-breaker in ORDER BY '{found_order}': {details} (query: {query})"
                )
            }
            Self::DescendingDirectionForbidden { query, column } => {
                write!(
                    f,
                    "Descending ordering direction forbidden on column '{column}' (query: {query})"
                )
            }
            Self::InvalidPrimaryOrderColumn {
                query,
                found_column,
            } => {
                write!(
                    f,
                    "Invalid primary order column '{found_column}', expected 'created_at' (query: {query})"
                )
            }
            Self::EmptyQuery => write!(f, "Query cannot be empty"),
        }
    }
}

impl std::error::Error for FifoOrderingViolation {}

/// Comprehensive inspection report for a SQL query's FIFO compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoInspectionReport {
    pub is_fifo_compliant: bool,
    pub target_detected: Option<String>,
    pub extracted_order_by: Option<String>,
    pub violations: Vec<FifoOrderingViolation>,
    pub normalized_query: Option<String>,
}

/// Checks if a SQL query targets one of the monitored prompt queue tables or databases.
pub fn is_fifo_monitored_query(sql: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    MONITORED_TARGETS
        .iter()
        .any(|&target| lower.contains(target))
}

/// Identifies which monitored target table or database is referenced in the query.
pub fn detect_target_table(sql: &str) -> Option<String> {
    let lower = sql.to_ascii_lowercase();
    for &target in MONITORED_TARGETS {
        if lower.contains(target) {
            return Some(target.to_string());
        }
    }
    None
}

/// Checks for prohibited LIFO patterns (e.g. `updated_at DESC`, `created_at DESC`).
pub fn detect_lifo_patterns(sql: &str) -> Option<String> {
    let lower = sql.to_ascii_lowercase();
    if lower.contains("updated_at desc") {
        Some("Inverted LIFO ordering via 'updated_at DESC' is strictly forbidden".to_string())
    } else if lower.contains("created_at desc") {
        Some("Inverted LIFO ordering via 'created_at DESC' is strictly forbidden".to_string())
    } else if lower.contains("id desc") {
        Some("Descending 'id DESC' ordering is forbidden in FIFO dispatch queries".to_string())
    } else {
        None
    }
}

/// Extracts the substring following `ORDER BY` up to trailing SQL clauses (`LIMIT`, `OFFSET`, `;`).
pub fn extract_order_by_clause(sql: &str) -> Option<String> {
    let lower = sql.to_ascii_lowercase();
    let order_by_kw = "order by";
    let order_idx = lower.find(order_by_kw)?;
    let start = order_idx + order_by_kw.len();
    let remainder = &sql[start..];
    let lower_remainder = &lower[start..];

    let mut end_offset = remainder.len();

    for keyword in &[
        " limit", "\nlimit", "\tlimit", " offset", "\noffset", " window", "\nwindow", ";",
    ] {
        if let Some(pos) = lower_remainder.find(keyword) {
            if pos < end_offset {
                end_offset = pos;
            }
        }
    }

    let extracted = remainder[..end_offset].trim();
    if extracted.is_empty() {
        None
    } else {
        Some(extracted.to_string())
    }
}

/// Validates that a SQL query strictly complies with FIFO queue ordering invariants.
pub fn validate_dispatch_query(sql: &str) -> Result<(), FifoOrderingViolation> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(FifoOrderingViolation::EmptyQuery);
    }

    if let Some(lifo_details) = detect_lifo_patterns(trimmed) {
        return Err(FifoOrderingViolation::LIFOOrderingForbidden {
            query: trimmed.to_string(),
            details: lifo_details,
        });
    }

    let target = detect_target_table(trimmed).unwrap_or_else(|| "prompt_queue".to_string());
    let order_clause = match extract_order_by_clause(trimmed) {
        Some(c) => c,
        None => {
            return Err(FifoOrderingViolation::MissingOrderByClause {
                query: trimmed.to_string(),
                target_table: target,
            });
        }
    };

    let terms: Vec<&str> = order_clause.split(',').map(|s| s.trim()).collect();
    if terms.is_empty() {
        return Err(FifoOrderingViolation::MissingOrderByClause {
            query: trimmed.to_string(),
            target_table: target,
        });
    }

    // Inspect primary sort term (must be created_at ASC)
    let parts0: Vec<&str> = terms[0].split_whitespace().collect();
    if parts0.is_empty() {
        return Err(FifoOrderingViolation::MissingOrderByClause {
            query: trimmed.to_string(),
            target_table: target,
        });
    }

    let raw_col0 = parts0[0];
    let col0 = raw_col0
        .rsplit('.')
        .next()
        .unwrap_or(raw_col0)
        .to_ascii_lowercase();

    if col0 != "created_at" {
        return Err(FifoOrderingViolation::InvalidPrimaryOrderColumn {
            query: trimmed.to_string(),
            found_column: col0,
        });
    }

    if parts0.len() > 1 && parts0[1].eq_ignore_ascii_case("desc") {
        return Err(FifoOrderingViolation::DescendingDirectionForbidden {
            query: trimmed.to_string(),
            column: col0,
        });
    }

    // Inspect secondary tie-breaker term (must be id ASC)
    if terms.len() < 2 {
        return Err(FifoOrderingViolation::MissingAscendingTieBreaker {
            query: trimmed.to_string(),
            found_order: order_clause.clone(),
            details: "Missing mandatory secondary tie-breaker 'id ASC' for deterministic FIFO dispatch ordering"
                .to_string(),
        });
    }

    let parts1: Vec<&str> = terms[1].split_whitespace().collect();
    if parts1.is_empty() {
        return Err(FifoOrderingViolation::MissingAscendingTieBreaker {
            query: trimmed.to_string(),
            found_order: order_clause.clone(),
            details: "Empty secondary tie-breaker term".to_string(),
        });
    }

    let raw_col1 = parts1[0];
    let col1 = raw_col1
        .rsplit('.')
        .next()
        .unwrap_or(raw_col1)
        .to_ascii_lowercase();

    if col1 != "id" {
        return Err(FifoOrderingViolation::MissingAscendingTieBreaker {
            query: trimmed.to_string(),
            found_order: order_clause.clone(),
            details: format!("Expected tie-breaker column 'id', but found '{col1}'"),
        });
    }

    if parts1.len() > 1 && parts1[1].eq_ignore_ascii_case("desc") {
        return Err(FifoOrderingViolation::DescendingDirectionForbidden {
            query: trimmed.to_string(),
            column: col1,
        });
    }

    Ok(())
}

/// Normalizes any dispatch SQL query to strictly enforce `ORDER BY created_at ASC, id ASC`.
pub fn normalize_to_fifo_dispatch_query(sql: &str) -> String {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return CANONICAL_FIFO_ORDER_BY.to_string();
    }

    let lower = trimmed.to_ascii_lowercase();
    let order_by_kw = "order by";

    if let Some(order_idx) = lower.find(order_by_kw) {
        let before_order = trimmed[..order_idx].trim_end();
        let after_start = order_idx + order_by_kw.len();
        let remainder = &trimmed[after_start..];
        let lower_remainder = &lower[after_start..];

        let mut boundary_offset = remainder.len();
        for keyword in &[
            " limit", "\nlimit", "\tlimit", " offset", "\noffset", " window", "\nwindow", ";",
        ] {
            if let Some(pos) = lower_remainder.find(keyword) {
                if pos < boundary_offset {
                    boundary_offset = pos;
                }
            }
        }

        let trailing_part = remainder[boundary_offset..].trim_start();
        if trailing_part.is_empty() {
            format!("{before_order} {CANONICAL_FIFO_ORDER_BY}")
        } else if trailing_part.starts_with(';') {
            format!("{before_order} {CANONICAL_FIFO_ORDER_BY}{trailing_part}")
        } else {
            format!("{before_order} {CANONICAL_FIFO_ORDER_BY} {trailing_part}")
        }
    } else {
        // Query does not contain an ORDER BY clause; insert before LIMIT/OFFSET/; if present
        let mut insert_pos = trimmed.len();
        for keyword in &[
            " limit", "\nlimit", "\tlimit", " offset", "\noffset", " window", "\nwindow", ";",
        ] {
            if let Some(pos) = lower.find(keyword) {
                if pos < insert_pos {
                    insert_pos = pos;
                }
            }
        }

        let before = trimmed[..insert_pos].trim_end();
        let trailing = trimmed[insert_pos..].trim_start();

        if trailing.is_empty() {
            format!("{before} {CANONICAL_FIFO_ORDER_BY}")
        } else if trailing.starts_with(';') {
            format!("{before} {CANONICAL_FIFO_ORDER_BY}{trailing}")
        } else {
            format!("{before} {CANONICAL_FIFO_ORDER_BY} {trailing}")
        }
    }
}

/// Performs complete inspection of a SQL query and returns a structured FIFO report.
pub fn inspect_and_validate_query(sql: &str) -> FifoInspectionReport {
    let target_detected = detect_target_table(sql);
    let extracted_order_by = extract_order_by_clause(sql);

    match validate_dispatch_query(sql) {
        Ok(()) => FifoInspectionReport {
            is_fifo_compliant: true,
            target_detected,
            extracted_order_by,
            violations: Vec::new(),
            normalized_query: None,
        },
        Err(violation) => {
            let normalized = normalize_to_fifo_dispatch_query(sql);
            FifoInspectionReport {
                is_fifo_compliant: false,
                target_detected,
                extracted_order_by,
                violations: vec![violation],
                normalized_query: Some(normalized),
            }
        }
    }
}

/// Validates or automatically normalizes a SQL query to satisfy strict FIFO invariants.
pub fn enforce_fifo_or_normalize(sql: &str) -> String {
    if validate_dispatch_query(sql).is_ok() {
        sql.to_string()
    } else {
        normalize_to_fifo_dispatch_query(sql)
    }
}
