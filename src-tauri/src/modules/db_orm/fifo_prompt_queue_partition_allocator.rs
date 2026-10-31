//! FIFO Prompt Queue Partition & Priority Allocator (`Imp-712`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`,
//! `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Specification Reference: `02-spec/21-app/217-implementing-next-five-tasks/spec.md`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical FIFO ORDER BY clause required across all prompt partition and dispatch queries.
pub const CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC";

/// Known prompt queue tables subject to strict FIFO ordering enforcement.
pub const PROMPT_QUEUE_TABLES: &[&str] = &[
    "active_prompts",
    "prompt_backups",
    "repo_prompts",
    "prompts_queue",
    "prompt_queue",
];

/// Priority classification band for queue partitions and prompt items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum QueuePriorityBand {
    /// Urgent priority prompts (processed before normal queue items).
    Urgent,
    /// Standard baseline priority prompts.
    Normal,
    /// Background and deferred asynchronous batch tasks.
    Background,
}

impl QueuePriorityBand {
    /// Returns the canonical lowercase string identifier for this priority band.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Urgent => "urgent",
            Self::Normal => "normal",
            Self::Background => "background",
        }
    }

    /// Numerical priority rank where lowest value represents highest priority.
    pub fn rank(&self) -> u8 {
        match self {
            Self::Urgent => 0,
            Self::Normal => 1,
            Self::Background => 2,
        }
    }

    /// Parses a string into a `QueuePriorityBand` loosely (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("urgent") {
            Some(Self::Urgent)
        } else if trimmed.eq_ignore_ascii_case("normal") {
            Some(Self::Normal)
        } else if trimmed.eq_ignore_ascii_case("background") {
            Some(Self::Background)
        } else {
            None
        }
    }
}

impl fmt::Display for QueuePriorityBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Target bucket configuration for partition allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuePartitionBucket {
    /// Unique identifier of the partition bucket.
    pub partition_id: String,
    /// Target instance ID bound to this partition (empty for shared/unbound).
    pub instance_id: String,
    /// Priority level serviced by this partition bucket.
    pub priority: QueuePriorityBand,
    /// Maximum number of items allocated to this bucket.
    pub limit: usize,
}

impl QueuePartitionBucket {
    /// Constructs a new partition bucket definition.
    pub fn new(
        partition_id: impl Into<String>,
        instance_id: impl Into<String>,
        priority: QueuePriorityBand,
        limit: usize,
    ) -> Self {
        Self {
            partition_id: partition_id.into(),
            instance_id: instance_id.into(),
            priority,
            limit,
        }
    }

    /// Partition bucket identifier.
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    /// Bound instance identifier.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Assigned queue priority band.
    pub fn priority(&self) -> QueuePriorityBand {
        self.priority
    }

    /// Allocation limit cap.
    pub fn limit(&self) -> usize {
        self.limit
    }
}

/// An allocated prompt item assigned to a partition bucket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocatedPromptItem {
    /// Unique primary database ID of the prompt.
    pub id: i64,
    /// ISO-8601 or epoch timestamp string when the prompt was originally created.
    pub created_at: String,
    /// Prompt content text.
    pub prompt_text: String,
    /// Priority level of the prompt.
    pub priority: QueuePriorityBand,
    /// Allocated partition bucket identifier.
    pub partition_id: String,
}

impl AllocatedPromptItem {
    /// Constructs a new allocated prompt item record.
    pub fn new(
        id: i64,
        created_at: impl Into<String>,
        prompt_text: impl Into<String>,
        priority: QueuePriorityBand,
        partition_id: impl Into<String>,
    ) -> Self {
        Self {
            id,
            created_at: created_at.into(),
            prompt_text: prompt_text.into(),
            priority,
            partition_id: partition_id.into(),
        }
    }

    /// Primary database identifier.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Timestamp string of creation.
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Prompt content string.
    pub fn prompt_text(&self) -> &str {
        &self.prompt_text
    }

    /// Priority band.
    pub fn priority(&self) -> QueuePriorityBand {
        self.priority
    }

    /// Partition bucket identifier.
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }
}

/// Comprehensive report resulting from partition allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionAllocationReport {
    /// Total count of successfully allocated prompt items across all buckets.
    pub total_allocated: usize,
    /// List of allocated items.
    pub items: Vec<AllocatedPromptItem>,
    /// The SQLite query SQL string used or generated for partition querying.
    pub query_sql: String,
    /// Whether all allocated items strictly adhere to FIFO ordering invariants.
    pub is_fifo_compliant: bool,
}

impl PartitionAllocationReport {
    /// Constructs a new allocation report.
    pub fn new(
        total_allocated: usize,
        items: Vec<AllocatedPromptItem>,
        query_sql: impl Into<String>,
        is_fifo_compliant: bool,
    ) -> Self {
        Self {
            total_allocated,
            items,
            query_sql: query_sql.into(),
            is_fifo_compliant,
        }
    }

    /// Total count of allocated prompts.
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Reference to allocated items.
    pub fn items(&self) -> &[AllocatedPromptItem] {
        &self.items
    }

    /// Generated or validated query SQL.
    pub fn query_sql(&self) -> &str {
        &self.query_sql
    }

    /// Returns true if all allocated items satisfy FIFO monotonicity.
    pub fn is_fifo_compliant(&self) -> bool {
        self.is_fifo_compliant
    }

    /// Filters allocated items for a specific partition ID.
    pub fn items_for_partition(&self, partition_id: &str) -> Vec<&AllocatedPromptItem> {
        self.items
            .iter()
            .filter(|item| item.partition_id == partition_id)
            .collect()
    }
}

/// Errors detected during partition allocation and query validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum PartitionAllocationError {
    /// Forbidden inverted LIFO ordering detected (e.g. `ORDER BY ... DESC` or inverted primary sort).
    #[error("Inverted LIFO ordering detected. Strict FIFO queries must use 'ORDER BY created_at ASC, id ASC'")]
    InvertedLifoOrderingDetected,

    /// Missing mandatory deterministic secondary tie-breaker `id ASC`.
    #[error("Missing mandatory tie-breaker 'id ASC' in ordering clause")]
    MissingTieBreakerId,

    /// Partition limit is invalid (e.g. limit is zero).
    #[error("Invalid partition limit (limit must be greater than 0)")]
    InvalidPartitionLimit,
}

impl From<PartitionAllocationError> for crate::error::AppError {
    fn from(err: PartitionAllocationError) -> Self {
        crate::error::AppError::Config(err.to_string())
    }
}

/// Compares two timestamp representations either numerically or lexicographically.
pub fn compare_timestamps(a: &str, b: &str) -> std::cmp::Ordering {
    let a_trim = a.trim();
    let b_trim = b.trim();
    if let (Ok(num_a), Ok(num_b)) = (a_trim.parse::<i64>(), b_trim.parse::<i64>()) {
        num_a.cmp(&num_b)
    } else {
        a_trim.cmp(b_trim)
    }
}

/// Normalizes a column name by removing table qualifiers, backticks, quotes, and punctuation.
pub fn clean_column_name(col: &str) -> &str {
    let trimmed = col
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(',');
    if let Some((_, after_dot)) = trimmed.rsplit_once('.') {
        after_dot
            .trim_matches('`')
            .trim_matches('"')
            .trim_matches('\'')
    } else {
        trimmed
    }
}

/// Checks if a table name is one of the recognized prompt queue tables.
pub fn is_prompt_queue_table(table_name: &str) -> bool {
    let clean = clean_column_name(table_name);
    PROMPT_QUEUE_TABLES
        .iter()
        .any(|known| clean.eq_ignore_ascii_case(known))
}

/// Validates that an SQL query string strictly conforms to FIFO dispatch invariants.
pub fn validate_partition_query_sql(sql: &str) -> Result<(), PartitionAllocationError> {
    let normalized = sql.to_uppercase();

    // 1. Check for explicit forbidden LIFO clauses anywhere in the statement
    if normalized.contains("ORDER BY UPDATED_AT DESC")
        || normalized.contains("ORDER BY CREATED_AT DESC")
        || normalized.contains("ORDER BY ID DESC")
        || normalized.contains("ORDER BY ROWID DESC")
    {
        return Err(PartitionAllocationError::InvertedLifoOrderingDetected);
    }

    // 2. Query must contain an ORDER BY clause
    let order_by_idx = match normalized.find("ORDER BY") {
        Some(idx) => idx + "ORDER BY".len(),
        None => return Err(PartitionAllocationError::MissingTieBreakerId),
    };

    let tail = &sql[order_by_idx..];
    // Strip trailing LIMIT, OFFSET, semicolon, etc.
    let end_idx = tail
        .to_uppercase()
        .find("LIMIT")
        .or_else(|| tail.to_uppercase().find("OFFSET"))
        .or_else(|| tail.find(';'))
        .unwrap_or(tail.len());

    let order_clause = tail[..end_idx].trim();
    if order_clause.is_empty() {
        return Err(PartitionAllocationError::MissingTieBreakerId);
    }

    // Split order clause by comma into sort parts
    let parts: Vec<&str> = order_clause.split(',').collect();

    // 3. Any DESC keyword in the ORDER BY clause is forbidden
    for part in &parts {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        for tok in tokens {
            let clean_tok = tok
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_uppercase();
            if clean_tok == "DESC" {
                return Err(PartitionAllocationError::InvertedLifoOrderingDetected);
            }
        }
    }

    // 4. Must contain id ASC tie-breaker and verify primary ordering
    let mut id_asc_index: Option<usize> = None;
    let mut created_at_index: Option<usize> = None;

    for (idx, part) in parts.iter().enumerate() {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        let col = clean_column_name(tokens[0]);
        if col.eq_ignore_ascii_case("id") {
            // Must have explicit ASC direction
            let has_asc =
                tokens.len() >= 2 && tokens[1].trim_matches(';').eq_ignore_ascii_case("ASC");
            if has_asc {
                id_asc_index = Some(idx);
            }
        } else if col.eq_ignore_ascii_case("created_at") {
            created_at_index = Some(idx);
        }
    }

    let id_asc_idx = match id_asc_index {
        Some(idx) => idx,
        None => return Err(PartitionAllocationError::MissingTieBreakerId),
    };

    // If created_at is present, it MUST precede id ASC
    if let Some(cat_idx) = created_at_index {
        if cat_idx > id_asc_idx {
            // id ASC was placed before created_at -> Inverted primary ordering!
            return Err(PartitionAllocationError::InvertedLifoOrderingDetected);
        }
    }

    Ok(())
}

/// Constructs a partition query SQL statement for a specific table and bucket.
pub fn build_partition_query_sql_for_table(
    table: &str,
    bucket: &QueuePartitionBucket,
) -> Result<String, PartitionAllocationError> {
    if bucket.limit == 0 {
        return Err(PartitionAllocationError::InvalidPartitionLimit);
    }

    let priority_val = bucket.priority.as_str();
    let instance_filter = if bucket.instance_id.trim().is_empty() {
        String::new()
    } else {
        format!(
            " AND instance_id = '{}'",
            bucket.instance_id.replace('\'', "''")
        )
    };

    let sql = format!(
        "SELECT id, created_at, prompt_text, priority, partition_id FROM {table} WHERE priority = '{priority_val}'{instance_filter} ORDER BY created_at ASC, id ASC LIMIT {};",
        bucket.limit
    );

    validate_partition_query_sql(&sql)?;
    Ok(sql)
}

/// Constructs a partition query SQL statement enforcing strict FIFO ordering.
pub fn build_partition_query_sql(
    bucket: &QueuePartitionBucket,
) -> Result<String, PartitionAllocationError> {
    build_partition_query_sql_for_table("active_prompts", bucket)
}

/// Verifies that an allocated partition item sequence strictly satisfies chronological FIFO monotonicity.
pub fn verify_allocated_partition_monotonicity(items: &[AllocatedPromptItem]) -> bool {
    if items.is_empty() {
        return true;
    }
    if items.len() == 1 {
        return !items[0].created_at.trim().is_empty();
    }
    for window in items.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        if prev.created_at.trim().is_empty() || curr.created_at.trim().is_empty() {
            return false;
        }
        let ordering = compare_timestamps(&prev.created_at, &curr.created_at);
        match ordering {
            std::cmp::Ordering::Greater => {
                // Inverted timestamp: prev was created after curr
                return false;
            }
            std::cmp::Ordering::Equal => {
                // Identical timestamps: verify deterministic secondary tie-breaker id ASC
                if prev.id > curr.id {
                    return false;
                }
            }
            std::cmp::Ordering::Less => {
                // Strictly monotonic forward progress
            }
        }
    }
    true
}

/// Allocates raw prompt queue items to partition buckets while enforcing priority and FIFO invariants.
pub fn allocate_items_to_partitions(
    raw_items: &[AllocatedPromptItem],
    buckets: &[QueuePartitionBucket],
) -> PartitionAllocationReport {
    let mut allocated_items = Vec::new();
    let mut allocated_ids = std::collections::HashSet::new();

    // 1. Generate partition query SQL strings
    let mut query_sqls = Vec::new();
    for bucket in buckets {
        if let Ok(sql) = build_partition_query_sql(bucket) {
            query_sqls.push(sql);
        }
    }
    let query_sql = if query_sqls.is_empty() {
        "SELECT id, created_at, prompt_text, priority, partition_id FROM active_prompts ORDER BY created_at ASC, id ASC;".to_string()
    } else {
        query_sqls.join("\n")
    };

    // 2. Allocate items per bucket according to priority band and partition limit
    for bucket in buckets {
        if bucket.limit == 0 {
            continue;
        }

        let mut bucket_count = 0;
        for raw in raw_items {
            if bucket_count >= bucket.limit {
                break;
            }
            if !allocated_ids.contains(&raw.id) && raw.priority == bucket.priority {
                allocated_ids.insert(raw.id);
                allocated_items.push(AllocatedPromptItem {
                    id: raw.id,
                    created_at: raw.created_at.clone(),
                    prompt_text: raw.prompt_text.clone(),
                    priority: raw.priority,
                    partition_id: bucket.partition_id.clone(),
                });
                bucket_count += 1;
            }
        }
    }

    // 3. Verify FIFO monotonicity across partitions
    let is_fifo_compliant = if allocated_items.is_empty() {
        true
    } else {
        let mut by_partition: std::collections::HashMap<&str, Vec<AllocatedPromptItem>> =
            std::collections::HashMap::new();
        for item in &allocated_items {
            by_partition
                .entry(&item.partition_id)
                .or_default()
                .push(item.clone());
        }

        by_partition
            .values()
            .all(|items| verify_allocated_partition_monotonicity(items))
    };

    PartitionAllocationReport {
        total_allocated: allocated_items.len(),
        items: allocated_items,
        query_sql,
        is_fifo_compliant,
    }
}
