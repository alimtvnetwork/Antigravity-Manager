//! Strict FIFO Prompt Restore Concurrency Limiter (`Task-210`).
//!
//! Enforces the AGENTS.md invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Regulates concurrent restore transactions across prompt tables (`active_prompts`,
//! `prompt_backups`, `repo_prompts`), ensuring deterministic single-writer or bounded-worker
//! leases without race conditions, out-of-order queue ingestion, or deadlocks.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Distinct SQLite tables subject to FIFO prompt restore concurrency control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestoreQueueTable {
    /// Active prompt processing queue (`active_prompts`).
    ActivePrompts,
    /// Archived prompt backup table (`prompt_backups`).
    PromptBackups,
    /// Repository-level prompt table (`repo_prompts`).
    RepoPrompts,
}

impl RestoreQueueTable {
    /// Returns the canonical SQLite table name for this table kind.
    pub fn as_table_name(&self) -> &'static str {
        match self {
            Self::ActivePrompts => "active_prompts",
            Self::PromptBackups => "prompt_backups",
            Self::RepoPrompts => "repo_prompts",
        }
    }

    /// Returns the table name as a string slice.
    pub fn as_str(&self) -> &'static str {
        self.as_table_name()
    }

    /// Returns a slice containing all table variants.
    pub fn all() -> &'static [Self] {
        &[Self::ActivePrompts, Self::PromptBackups, Self::RepoPrompts]
    }
}

impl fmt::Display for RestoreQueueTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_table_name())
    }
}

/// Active restore lease acquired by an executing task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreLeaseRecord {
    pub lease_id: String,
    pub table: RestoreQueueTable,
    pub holder_task_id: String,
    pub acquired_epoch_ms: u64,
    pub expires_epoch_ms: u64,
}

impl RestoreLeaseRecord {
    /// Constructs a new `RestoreLeaseRecord`.
    pub fn new(
        lease_id: impl Into<String>,
        table: RestoreQueueTable,
        holder_task_id: impl Into<String>,
        acquired_epoch_ms: u64,
        expires_epoch_ms: u64,
    ) -> Self {
        Self {
            lease_id: lease_id.into(),
            table,
            holder_task_id: holder_task_id.into(),
            acquired_epoch_ms,
            expires_epoch_ms,
        }
    }

    /// Returns true if the lease has not yet reached its expiration timestamp.
    pub fn is_active(&self, now_ms: u64) -> bool {
        self.expires_epoch_ms > now_ms
    }

    /// Returns true if the lease has expired at or before the given timestamp.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        self.expires_epoch_ms <= now_ms
    }

    /// Returns remaining validity in milliseconds saturating at zero.
    pub fn remaining_ms(&self, now_ms: u64) -> u64 {
        self.expires_epoch_ms.saturating_sub(now_ms)
    }
}

/// Violations emitted when concurrent restore lease acquisition fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcurrencyLimiterViolation {
    /// Single-worker restore conflict against an existing active lease.
    ConcurrentRestoreConflict {
        table: RestoreQueueTable,
        current_holder: String,
        expires_in_ms: u64,
    },
    /// The maximum configured permits for the target table have been exhausted.
    MaxPermitsExceeded {
        table: RestoreQueueTable,
        max_permits: usize,
    },
}

impl fmt::Display for ConcurrencyLimiterViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConcurrentRestoreConflict { table, current_holder, expires_in_ms } => {
                write!(f, "Concurrent restore conflict on table '{table}': held by '{current_holder}' (expires in {expires_in_ms} ms)")
            }
            Self::MaxPermitsExceeded { table, max_permits } => {
                write!(f, "Max concurrent restore permits ({max_permits}) exceeded for table '{table}'")
            }
        }
    }
}

impl Error for ConcurrencyLimiterViolation {}

/// In-memory state tracking active prompt restore concurrency leases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimiterState {
    pub active_leases: Vec<RestoreLeaseRecord>,
    pub max_concurrent_per_table: usize,
    pub default_lease_duration_ms: u64,
}

impl LimiterState {
    /// Creates a new `LimiterState` with custom limits.
    pub fn new(max_concurrent_per_table: usize, default_lease_duration_ms: u64) -> Self {
        Self {
            active_leases: Vec::new(),
            max_concurrent_per_table,
            default_lease_duration_ms,
        }
    }
}

impl Default for LimiterState {
    fn default() -> Self {
        Self::new(1, 30_000)
    }
}

/// Purges all leases that have expired at or prior to `now_ms`.
pub fn purge_expired_leases(state: &mut LimiterState, now_ms: u64) -> usize {
    let initial_len = state.active_leases.len();
    state.active_leases.retain(|lease| lease.is_active(now_ms));
    initial_len.saturating_sub(state.active_leases.len())
}

/// Returns the number of currently active leases for the specified table.
pub fn count_active_leases(state: &LimiterState, table: RestoreQueueTable, now_ms: u64) -> usize {
    state
        .active_leases
        .iter()
        .filter(|lease| lease.table == table && lease.is_active(now_ms))
        .count()
}

/// Locates the first active lease currently held for the specified table.
pub fn find_active_lease<'a>(
    state: &'a LimiterState,
    table: RestoreQueueTable,
    now_ms: u64,
) -> Option<&'a RestoreLeaseRecord> {
    state
        .active_leases
        .iter()
        .find(|lease| lease.table == table && lease.is_active(now_ms))
}

/// Returns true if the given table currently has at least one active restore lease.
pub fn has_active_lease(state: &LimiterState, table: RestoreQueueTable, now_ms: u64) -> bool {
    find_active_lease(state, table, now_ms).is_some()
}

/// Releases the restore lease identified by `lease_id`.
pub fn release_restore_lease(state: &mut LimiterState, lease_id: &str) -> bool {
    let initial_len = state.active_leases.len();
    state.active_leases.retain(|lease| lease.lease_id != lease_id);
    state.active_leases.len() < initial_len
}

fn build_concurrency_violation(
    state: &LimiterState,
    table: RestoreQueueTable,
    now_ms: u64,
) -> ConcurrencyLimiterViolation {
    if state.max_concurrent_per_table <= 1 {
        if let Some(holder) = find_active_lease(state, table, now_ms) {
            return ConcurrencyLimiterViolation::ConcurrentRestoreConflict {
                table,
                current_holder: holder.holder_task_id.clone(),
                expires_in_ms: holder.remaining_ms(now_ms),
            };
        }
    }
    ConcurrencyLimiterViolation::MaxPermitsExceeded {
        table,
        max_permits: state.max_concurrent_per_table,
    }
}

fn create_new_lease(
    state: &LimiterState,
    table: RestoreQueueTable,
    task_id: &str,
    now_ms: u64,
) -> RestoreLeaseRecord {
    let lease_id = format!(
        "lease-{}-{}-{}-{}",
        table.as_str(),
        task_id,
        now_ms,
        state.active_leases.len() + 1
    );
    let expires_epoch_ms = now_ms.saturating_add(state.default_lease_duration_ms);
    RestoreLeaseRecord::new(lease_id, table, task_id, now_ms, expires_epoch_ms)
}

/// Attempts to acquire a restore lease on the specified table for task `task_id`.
pub fn acquire_restore_lease(
    state: &mut LimiterState,
    table: RestoreQueueTable,
    task_id: &str,
    now_ms: u64,
) -> Result<RestoreLeaseRecord, ConcurrencyLimiterViolation> {
    purge_expired_leases(state, now_ms);
    let active_count = count_active_leases(state, table, now_ms);
    if active_count >= state.max_concurrent_per_table {
        return Err(build_concurrency_violation(state, table, now_ms));
    }
    let lease = create_new_lease(state, table, task_id, now_ms);
    state.active_leases.push(lease.clone());
    Ok(lease)
}
