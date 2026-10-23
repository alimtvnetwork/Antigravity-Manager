//! Strict FIFO Dead-Letter Queue Spillover Guard (`Imp-687`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`)
//! are strictly forbidden in dispatch logic."
//!
//! This module coordinates the quarantine of persistently failing prompts into dead-letter storage
//! while strictly preserving original creation timestamps (`original_created_at`) and chronological
//! ordering (`ORDER BY created_at ASC, id ASC`), ensuring full auditability and preventing sequence regressions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical FIFO ORDER BY clause required for all queue drain and spillover queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Default dead-letter storage table name.
pub const DEAD_LETTER_TABLE: &str = "dead_letter_prompts";

/// Categorized failure reasons for routing prompts into the dead-letter queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadLetterReason {
    /// Repeated processing attempts exceeded the permitted threshold.
    RetryLimitExceeded { attempts: u32, max_retries: u32 },
    /// Non-retryable syntax or parsing failure in prompt payload.
    UnrecoverableSyntaxError(String),
    /// Upstream AI provider permanently rejected the request (e.g., policy violation, 400).
    UpstreamPermanentRejection(String),
}

impl fmt::Display for DeadLetterReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetryLimitExceeded { attempts, max_retries } => {
                write!(f, "Retry limit exceeded ({attempts}/{max_retries})")
            }
            Self::UnrecoverableSyntaxError(err) => {
                write!(f, "Unrecoverable syntax error: {err}")
            }
            Self::UpstreamPermanentRejection(msg) => {
                write!(f, "Upstream permanent rejection: {msg}")
            }
        }
    }
}

impl DeadLetterReason {
    /// Returns true if the reason was retry exhaustion.
    pub fn is_retry_limit_exceeded(&self) -> bool {
        matches!(self, Self::RetryLimitExceeded { .. })
    }

    /// Returns true if the failure was an unrecoverable syntax error.
    pub fn is_unrecoverable_syntax_error(&self) -> bool {
        matches!(self, Self::UnrecoverableSyntaxError(_))
    }

    /// Returns true if the upstream provider permanently rejected the prompt.
    pub fn is_upstream_permanent_rejection(&self) -> bool {
        matches!(self, Self::UpstreamPermanentRejection(_))
    }
}

/// A prompt record transitioned into the dead-letter spillover quarantine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadLetterPromptItem {
    /// Unique primary identifier of the original prompt.
    pub id: i64,
    /// Preserved chronological creation timestamp from original enqueueing.
    pub original_created_at: String,
    /// Prompt content text.
    pub prompt_text: String,
    /// Root cause classification for dead-letter routing.
    pub failed_reason: DeadLetterReason,
    /// Unix epoch timestamp in milliseconds when the spillover transition was executed.
    pub spillover_epoch_ms: u64,
}

impl DeadLetterPromptItem {
    /// Constructs a new dead-letter prompt item.
    pub fn new(
        id: i64,
        original_created_at: impl Into<String>,
        prompt_text: impl Into<String>,
        failed_reason: DeadLetterReason,
        spillover_epoch_ms: u64,
    ) -> Self {
        Self {
            id,
            original_created_at: original_created_at.into(),
            prompt_text: prompt_text.into(),
            failed_reason,
            spillover_epoch_ms,
        }
    }

    /// Identifier of the prompt.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Original enqueueing timestamp preserved from lineage.
    pub fn original_created_at(&self) -> &str {
        &self.original_created_at
    }

    /// Prompt text content.
    pub fn prompt_text(&self) -> &str {
        &self.prompt_text
    }

    /// Reason for quarantine.
    pub fn failed_reason(&self) -> &DeadLetterReason {
        &self.failed_reason
    }

    /// Spillover execution timestamp in epoch milliseconds.
    pub fn spillover_epoch_ms(&self) -> u64 {
        self.spillover_epoch_ms
    }
}

/// Violations detected when verifying spillover SQL queries or batch ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpilloverGuardViolation {
    /// Inverted sort ordering detected (e.g., LIFO `DESC` or reversed priority sort).
    InvertedSpilloverOrdering,
    /// Missing or blank original creation timestamp, compromising chronological lineage.
    MissingOriginalTimestamp,
    /// Missing mandatory secondary tie-breaker `id ASC` required for deterministic FIFO dispatch.
    MissingIdTieBreaker,
}

impl fmt::Display for SpilloverGuardViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedSpilloverOrdering => {
                write!(
                    f,
                    "Inverted spillover ordering detected (queries and items must strictly enforce ORDER BY created_at ASC, id ASC)"
                )
            }
            Self::MissingOriginalTimestamp => {
                write!(
                    f,
                    "Missing original timestamp in spillover query or dead-letter record"
                )
            }
            Self::MissingIdTieBreaker => {
                write!(
                    f,
                    "Missing mandatory secondary tie-breaker 'id ASC' in spillover ordering"
                )
            }
        }
    }
}

impl std::error::Error for SpilloverGuardViolation {}

/// Comprehensive audit report summarizing the dead-letter spillover operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadLetterSpilloverReport {
    /// Total count of prompt items transitioned into dead-letter quarantine.
    pub total_spilled_over: usize,
    /// Prompt items recorded during this spillover event.
    pub items: Vec<DeadLetterPromptItem>,
    /// True if all items strictly adhered to chronological FIFO ordering invariants.
    pub is_compliant: bool,
    /// Detailed list of detected ordering or timestamp violations.
    pub violations: Vec<SpilloverGuardViolation>,
}

impl DeadLetterSpilloverReport {
    /// Creates a new spillover report.
    pub fn new(
        total_spilled_over: usize,
        items: Vec<DeadLetterPromptItem>,
        is_compliant: bool,
        violations: Vec<SpilloverGuardViolation>,
    ) -> Self {
        Self {
            total_spilled_over,
            items,
            is_compliant,
            violations,
        }
    }

    /// Returns true if any violations were recorded.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns true if the spillover was completely compliant with FIFO invariants.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Returns the total number of items transitioned.
    pub fn total_spilled_over(&self) -> usize {
        self.total_spilled_over
    }

    /// Returns the list of violations.
    pub fn violations(&self) -> &[SpilloverGuardViolation] {
        &self.violations
    }

    /// Returns the list of dead-letter items.
    pub fn items(&self) -> &[DeadLetterPromptItem] {
        &self.items
    }

    /// Returns all primary key IDs quarantined in this batch.
    pub fn quarantined_ids(&self) -> Vec<i64> {
        self.items.iter().map(|item| item.id).collect()
    }
}

/// Strips surrounding quote characters or table alias prefixes from a SQL column name.
fn clean_column_name(token: &str) -> String {
    let unquoted = token.trim_matches(|c: char| c == '`' || c == '"' || c == '[' || c == ']' || c == '\'');
    if let Some((_, col)) = unquoted.rsplit_once('.') {
        col.to_string()
    } else {
        unquoted.to_string()
    }
}

/// Locates the byte offset immediately following `ORDER BY` with word boundary checks.
fn find_order_by_boundary(upper: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = upper[search_from..].find("ORDER") {
        let abs_order = search_from + pos;
        let before_ok = abs_order == 0
            || upper.as_bytes().get(abs_order - 1).map_or(true, |b| b.is_ascii_whitespace() || *b == b')');
        let after_order = abs_order + 5;
        let after_ok = after_order < upper.len() && upper.as_bytes()[after_order].is_ascii_whitespace();

        if before_ok && after_ok {
            let rest = &upper[after_order..];
            let trimmed = rest.trim_start();
            let whitespace_len = rest.len() - trimmed.len();
            if trimmed.starts_with("BY") {
                let after_by = 2;
                let by_end_ok = after_by >= trimmed.len()
                    || trimmed.as_bytes().get(after_by).map_or(true, |b| b.is_ascii_whitespace());
                if by_end_ok {
                    return Some(after_order + whitespace_len + 2);
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
                || upper_rem.as_bytes().get(abs_pos - 1).map_or(false, |b| b.is_ascii_whitespace());
            let after_marker = abs_pos + marker.len();
            let after_ok = after_marker >= upper_rem.len()
                || upper_rem.as_bytes().get(after_marker).map_or(false, |b| b.is_ascii_whitespace());
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

/// Verifies that a spillover selection SQL query strictly adheres to `ORDER BY created_at ASC, id ASC`.
///
/// Returns `Ok(())` if fully compliant, or `Err(SpilloverGuardViolation)` identifying the defect.
pub fn validate_spillover_query(sql: &str) -> Result<(), SpilloverGuardViolation> {
    let clause = match extract_order_by_clause(sql) {
        Some(c) => c,
        None => return Err(SpilloverGuardViolation::MissingOriginalTimestamp),
    };

    let upper_clause = clause.to_uppercase();

    // Check for explicit inverted DESC ordering
    for token in upper_clause.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean == "DESC" {
            return Err(SpilloverGuardViolation::InvertedSpilloverOrdering);
        }
    }

    let parts: Vec<&str> = clause
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(SpilloverGuardViolation::MissingOriginalTimestamp);
    }

    // Check primary sort column: must be created_at (optionally qualified)
    let first_tokens: Vec<&str> = parts[0].split_whitespace().collect();
    if first_tokens.is_empty() {
        return Err(SpilloverGuardViolation::MissingOriginalTimestamp);
    }

    let primary_col = clean_column_name(first_tokens[0]);

    if !primary_col.eq_ignore_ascii_case("created_at") {
        // If created_at is present later in the clause, order is inverted
        let created_at_present_later = parts[1..].iter().any(|part| {
            let tok = part.split_whitespace().next().unwrap_or("");
            clean_column_name(tok).eq_ignore_ascii_case("created_at")
        });
        if created_at_present_later {
            return Err(SpilloverGuardViolation::InvertedSpilloverOrdering);
        }
        return Err(SpilloverGuardViolation::MissingOriginalTimestamp);
    }

    // Check primary direction: if specified, must be ASC
    if first_tokens.len() >= 2 && !first_tokens[1].eq_ignore_ascii_case("ASC") {
        return Err(SpilloverGuardViolation::InvertedSpilloverOrdering);
    }

    // Check secondary sort column: must include id ASC tie-breaker
    if parts.len() < 2 {
        return Err(SpilloverGuardViolation::MissingIdTieBreaker);
    }

    let has_id_asc = parts[1..].iter().any(|part| {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.is_empty() {
            return false;
        }
        let col = clean_column_name(tokens[0]);
        if col.eq_ignore_ascii_case("id") {
            tokens.len() >= 2 && tokens[1].eq_ignore_ascii_case("ASC")
        } else {
            false
        }
    });

    if !has_id_asc {
        return Err(SpilloverGuardViolation::MissingIdTieBreaker);
    }

    Ok(())
}

/// Compares two timestamp representations either numerically or lexicographically.
fn compare_timestamps(a: &str, b: &str) -> std::cmp::Ordering {
    if let (Ok(num_a), Ok(num_b)) = (a.trim().parse::<i64>(), b.trim().parse::<i64>()) {
        num_a.cmp(&num_b)
    } else {
        a.trim().cmp(b.trim())
    }
}

/// Records and audits a batch of items transitioning into dead-letter spillover quarantine.
///
/// Verifies that all items preserve non-empty `original_created_at` timestamps and strictly
/// advance according to `ORDER BY created_at ASC, id ASC`.
pub fn record_dead_letter_spillover(items: &[DeadLetterPromptItem]) -> DeadLetterSpilloverReport {
    let mut violations = Vec::new();

    // 1. Verify presence of original created_at timestamps
    for item in items {
        if item.original_created_at.trim().is_empty() {
            if !violations.contains(&SpilloverGuardViolation::MissingOriginalTimestamp) {
                violations.push(SpilloverGuardViolation::MissingOriginalTimestamp);
            }
        }
    }

    // 2. Verify sequence monotonicity across successive items
    for idx in 1..items.len() {
        let prev = &items[idx - 1];
        let curr = &items[idx];

        if prev.original_created_at.trim().is_empty() || curr.original_created_at.trim().is_empty() {
            continue;
        }

        let ordering = compare_timestamps(&prev.original_created_at, &curr.original_created_at);
        match ordering {
            std::cmp::Ordering::Greater => {
                // prev > curr: Timestamp regression / inverted ordering!
                if !violations.contains(&SpilloverGuardViolation::InvertedSpilloverOrdering) {
                    violations.push(SpilloverGuardViolation::InvertedSpilloverOrdering);
                }
            }
            std::cmp::Ordering::Equal => {
                // Same timestamp: ID must be strictly increasing (curr.id > prev.id)
                if curr.id <= prev.id {
                    if !violations.contains(&SpilloverGuardViolation::MissingIdTieBreaker) {
                        violations.push(SpilloverGuardViolation::MissingIdTieBreaker);
                    }
                }
            }
            std::cmp::Ordering::Less => {
                // Strictly advancing timestamp: compliant
            }
        }
    }

    let is_compliant = violations.is_empty();

    DeadLetterSpilloverReport {
        total_spilled_over: items.len(),
        items: items.to_vec(),
        is_compliant,
        violations,
    }
}

/// Generates a standardized SQL query to select candidates for dead-letter spillover.
pub fn build_spillover_drain_query(source_table: &str, retry_threshold: u32, limit: Option<usize>) -> String {
    let mut query = format!(
        "SELECT id, original_created_at, prompt_text, retry_count FROM {source_table} WHERE retry_count >= {retry_threshold} ORDER BY created_at ASC, id ASC"
    );
    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {lim}"));
    }
    query
}
