//! Strict FIFO Prompt Restore Cursor Stepper (`Imp-697`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`)
//! are strictly forbidden in dispatch logic."
//!
//! Implements a deterministic cursor-driven FIFO prompt restore query stepper
//! (`WHERE (created_at > '<last_created_at>') OR (created_at = '<last_created_at>' AND id > <last_id>) ORDER BY created_at ASC, id ASC LIMIT <batch_size>`)
//! that enables deterministic paginated drains of prompt queues without unbounded memory usage or sequence inversions.

use serde::{Deserialize, Serialize};

/// Canonical FIFO ORDER BY clause required across all prompt restore queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Represents a deterministic cursor pointing to the last retrieved prompt record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptCursor {
    /// The timestamp string (`created_at`) of the last retrieved record.
    pub last_created_at: String,
    /// The tie-breaker primary key (`id`) of the last retrieved record.
    pub last_id: i64,
}

impl PromptCursor {
    /// Creates a new prompt cursor.
    pub fn new(last_created_at: impl Into<String>, last_id: i64) -> Self {
        Self {
            last_created_at: last_created_at.into(),
            last_id,
        }
    }
}

/// Configuration parameters for cursor-based FIFO stepping across prompt queue tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorStepConfig {
    /// The maximum number of records to retrieve per query step (default 50).
    pub batch_size: usize,
    /// The target SQLite queue table name (default "prompt_backups").
    pub target_table: String,
    /// Optional upper bound on the total items to drain across multiple steps.
    pub max_drain_limit: Option<usize>,
    /// Optional custom SQL template override, strictly validated against FIFO invariants.
    pub custom_sql_template: Option<String>,
}

impl Default for CursorStepConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            target_table: "prompt_backups".to_string(),
            max_drain_limit: None,
            custom_sql_template: None,
        }
    }
}

impl CursorStepConfig {
    /// Creates a new cursor step configuration with specified batch size and target table.
    pub fn new(batch_size: usize, target_table: impl Into<String>) -> Self {
        Self {
            batch_size,
            target_table: target_table.into(),
            max_drain_limit: None,
            custom_sql_template: None,
        }
    }

    /// Sets the maximum drain limit.
    pub fn with_max_drain_limit(mut self, limit: usize) -> Self {
        self.max_drain_limit = Some(limit);
        self
    }

    /// Sets a custom SQL query template (validated for strict FIFO compliance).
    pub fn with_custom_sql(mut self, sql: impl Into<String>) -> Self {
        self.custom_sql_template = Some(sql.into());
        self
    }
}

/// Represents an item retrieved from a prompt queue table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptQueueItem {
    /// Unique sequence primary key identifier.
    pub id: i64,
    /// Creation timestamp string (e.g. ISO 8601 or epoch string).
    pub created_at: String,
    /// Raw prompt prompt_text payload.
    pub prompt_text: String,
    /// Queue processing status (e.g. "pending", "restored", "processing").
    pub status: String,
}

impl PromptQueueItem {
    /// Creates a new prompt queue item.
    pub fn new(
        id: i64,
        created_at: impl Into<String>,
        prompt_text: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            id,
            created_at: created_at.into(),
            prompt_text: prompt_text.into(),
            status: status.into(),
        }
    }
}

/// Result produced by stepping a cursor through a batch of queue items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorStepResult {
    /// Retrieved queue items in strictly increasing FIFO order.
    pub items: Vec<PromptQueueItem>,
    /// Cursor identifying the boundary of this batch, or `None` if the queue is empty.
    pub next_cursor: Option<PromptCursor>,
    /// True if the current batch is the end of the queue (fewer items than batch_size).
    pub is_end_of_queue: bool,
    /// Total count of valid items scanned in this step.
    pub total_scanned: usize,
}

impl CursorStepResult {
    /// Creates a new cursor step result.
    pub fn new(
        items: Vec<PromptQueueItem>,
        next_cursor: Option<PromptCursor>,
        is_end_of_queue: bool,
        total_scanned: usize,
    ) -> Self {
        Self {
            items,
            next_cursor,
            is_end_of_queue,
            total_scanned,
        }
    }

    /// Returns true if no items were returned.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in this step.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Errors detected when validating cursor-based FIFO queries and item sequences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum CursorValidationError {
    /// Inverted LIFO ordering detected (e.g. `ORDER BY ... DESC`).
    #[error("LIFO ordering detected in query: '{0}'. FIFO queries must strictly use ORDER BY created_at ASC, id ASC")]
    LifoOrderingDetected(String),
    /// The secondary deterministic tie-breaker `id ASC` is missing from the query.
    #[error("Missing tie-breaker 'id ASC' in query ordering: '{0}'")]
    MissingTieBreakerId(String),
    /// Invalid cursor timestamp or injection hazard detected.
    #[error("Invalid cursor timestamp: '{0}'")]
    InvalidCursorTimestamp(String),
    /// Sequence regression or duplicate detected across items in a batch.
    #[error("Out-of-order item sequence detected: {0}")]
    OutOfOrderSequence(String),
}

/// Validates that a timestamp string is non-empty, contains safe characters, and lacks SQL injection vectors.
pub fn validate_timestamp(ts: &str) -> Result<(), CursorValidationError> {
    let trimmed = ts.trim();
    if trimmed.is_empty() {
        return Err(CursorValidationError::InvalidCursorTimestamp(
            "Timestamp cannot be empty".to_string(),
        ));
    }

    if trimmed.contains('\'')
        || trimmed.contains('"')
        || trimmed.contains(';')
        || trimmed.contains("--")
        || trimmed.contains("/*")
        || trimmed.contains('\\')
    {
        return Err(CursorValidationError::InvalidCursorTimestamp(format!(
            "Timestamp contains forbidden characters or SQL injection hazard: '{trimmed}'"
        )));
    }

    let is_valid = trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == ':' || c == '.' || c == ' ' || c == '+'
    });
    if !is_valid {
        return Err(CursorValidationError::InvalidCursorTimestamp(format!(
            "Timestamp contains invalid characters: '{trimmed}'"
        )));
    }

    Ok(())
}

/// Validates that a SQL query adheres strictly to canonical FIFO ordering rules.
pub fn validate_query_ordering(sql: &str) -> Result<(), CursorValidationError> {
    let upper = sql.to_uppercase();

    // LIFO ordering detection
    if upper.contains("DESC") {
        return Err(CursorValidationError::LifoOrderingDetected(sql.to_string()));
    }

    // Mandatory ORDER BY check
    if !upper.contains("ORDER BY") {
        return Err(CursorValidationError::MissingTieBreakerId(format!(
            "Missing ORDER BY clause in query: '{sql}'"
        )));
    }

    // Mandatory secondary tie-breaker id ASC check
    if !upper.contains("ID ASC") {
        return Err(CursorValidationError::MissingTieBreakerId(sql.to_string()));
    }

    // Mandatory primary sort column created_at ASC check
    if !upper.contains("CREATED_AT ASC") {
        return Err(CursorValidationError::LifoOrderingDetected(format!(
            "Primary sort column must be 'created_at ASC': '{sql}'"
        )));
    }

    Ok(())
}

/// Alias for `validate_query_ordering`.
pub fn validate_cursor_query(sql: &str) -> Result<(), CursorValidationError> {
    validate_query_ordering(sql)
}

/// Builds the deterministic FIFO SQL query string for a cursor step.
///
/// - Initial pass (`cursor` is `None`):
///   `SELECT id, created_at, prompt_text, status FROM <table_name> ORDER BY created_at ASC, id ASC LIMIT <batch_size>`
/// - Subsequent steps (`cursor` is `Some(cursor)`):
///   `SELECT id, created_at, prompt_text, status FROM <table_name> WHERE (created_at > '<last_created_at>') OR (created_at = '<last_created_at>' AND id > <last_id>) ORDER BY created_at ASC, id ASC LIMIT <batch_size>`
///
/// Returns an error if any LIFO ordering (`DESC`) is detected or if `id ASC` is missing.
pub fn build_cursor_query_sql(
    cursor: Option<&PromptCursor>,
    config: &CursorStepConfig,
) -> Result<String, CursorValidationError> {
    if let Some(ref custom_sql) = config.custom_sql_template {
        validate_query_ordering(custom_sql)?;
        return Ok(custom_sql.clone());
    }

    let table = config.target_table.trim();
    if table.is_empty() {
        return Err(CursorValidationError::InvalidCursorTimestamp(
            "Target table cannot be empty".to_string(),
        ));
    }
    if table.to_uppercase().contains("DESC") {
        return Err(CursorValidationError::LifoOrderingDetected(table.to_string()));
    }
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        return Err(CursorValidationError::InvalidCursorTimestamp(format!(
            "Invalid characters in table name: '{table}'"
        )));
    }

    let effective_batch = config
        .max_drain_limit
        .map(|max| max.min(config.batch_size))
        .unwrap_or(config.batch_size);

    let sql = match cursor {
        None => {
            format!(
                "SELECT id, created_at, prompt_text, status FROM {} ORDER BY created_at ASC, id ASC LIMIT {}",
                table, effective_batch
            )
        }
        Some(c) => {
            validate_timestamp(&c.last_created_at)?;
            format!(
                "SELECT id, created_at, prompt_text, status FROM {} WHERE (created_at > '{}') OR (created_at = '{}' AND id > {}) ORDER BY created_at ASC, id ASC LIMIT {}",
                table, c.last_created_at, c.last_created_at, c.last_id, effective_batch
            )
        }
    };

    validate_query_ordering(&sql)?;

    Ok(sql)
}

/// Verifies whether `current_cursor` strictly advances past `previous_cursor` monotonically.
pub fn verify_cursor_monotonicity(
    previous_cursor: &PromptCursor,
    current_cursor: &PromptCursor,
) -> bool {
    if current_cursor.last_created_at > previous_cursor.last_created_at {
        true
    } else if current_cursor.last_created_at == previous_cursor.last_created_at {
        current_cursor.last_id > previous_cursor.last_id
    } else {
        false
    }
}

/// Checks whether an item sequence strictly adheres to increasing `(created_at, id)` monotonicity.
pub fn verify_items_monotonicity(items: &[PromptQueueItem]) -> bool {
    if items.len() <= 1 {
        return true;
    }
    for window in items.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        if curr.created_at < prev.created_at {
            return false;
        }
        if curr.created_at == prev.created_at && curr.id <= prev.id {
            return false;
        }
    }
    true
}

/// Validates item sequence monotonicity, returning a `CursorValidationError` if an inversion is found.
pub fn validate_items_monotonicity(items: &[PromptQueueItem]) -> Result<(), CursorValidationError> {
    if items.len() <= 1 {
        return Ok(());
    }
    for window in items.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        if curr.created_at < prev.created_at {
            return Err(CursorValidationError::OutOfOrderSequence(format!(
                "Timestamp regression: item #{} ({}) preceded by item #{} ({})",
                curr.id, curr.created_at, prev.id, prev.created_at
            )));
        }
        if curr.created_at == prev.created_at && curr.id <= prev.id {
            return Err(CursorValidationError::OutOfOrderSequence(format!(
                "ID tie-breaker inversion at timestamp {}: item #{} <= preceding item #{}",
                curr.created_at, curr.id, prev.id
            )));
        }
    }
    Ok(())
}

/// Steps through a batch of queue items, validates monotonicity, extracts the next cursor, and returns the result.
pub fn step_cursor_batch(items: &[PromptQueueItem], config: &CursorStepConfig) -> CursorStepResult {
    let mut valid_items = Vec::new();
    let mut prev_created_at: Option<&str> = None;
    let mut prev_id: Option<i64> = None;

    for item in items {
        if let (Some(p_created), Some(p_id)) = (prev_created_at, prev_id) {
            let is_increasing = if item.created_at.as_str() > p_created {
                true
            } else if item.created_at.as_str() == p_created {
                item.id > p_id
            } else {
                false
            };

            if !is_increasing {
                tracing::warn!(
                    "Out-of-order sequence detected in step_cursor_batch: item id={} at {} <= previous id={} at {}",
                    item.id,
                    item.created_at,
                    p_id,
                    p_created
                );
                break;
            }
        }
        valid_items.push(item.clone());
        prev_created_at = Some(&item.created_at);
        prev_id = Some(item.id);
    }

    let next_cursor = valid_items.last().map(|last| PromptCursor {
        last_created_at: last.created_at.clone(),
        last_id: last.id,
    });
    let total_scanned = valid_items.len();
    let is_end_of_queue = items.len() < config.batch_size;

    CursorStepResult {
        items: valid_items,
        next_cursor,
        is_end_of_queue,
        total_scanned,
    }
}

/// Fallible variation of `step_cursor_batch` returning an error if any out-of-order item is detected.
pub fn try_step_cursor_batch(
    items: &[PromptQueueItem],
    config: &CursorStepConfig,
) -> Result<CursorStepResult, CursorValidationError> {
    validate_items_monotonicity(items)?;
    Ok(step_cursor_batch(items, config))
}
