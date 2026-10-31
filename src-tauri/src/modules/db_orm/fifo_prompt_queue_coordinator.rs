//! FIFO Prompt Queue Coordinator (`Imp-622`).
//! Enforces strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`
//! across prompt restore, dispatch, and queue processing queries in SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`).
//! Enforces AGENTS.md invariant: "Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Lifecycle state of an item inside the FIFO prompt queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueueItemStatus {
    /// Waiting in queue to be picked up.
    Enqueued,
    /// Leased to an active instance worker.
    Leased,
    /// Dispatched upstream to LLM.
    Dispatched,
    /// Processed successfully.
    Completed,
    /// Failed execution or permanently aborted.
    Failed,
}

impl Default for QueueItemStatus {
    fn default() -> Self {
        Self::Enqueued
    }
}

/// Represents a prompt unit inside the FIFO queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoPromptItem {
    pub id: String,
    pub conversation_id: String,
    pub prompt_text: String,
    pub created_at_epoch_ms: i64,
    pub status: QueueItemStatus,
    pub leased_by_instance: Option<String>,
    pub lease_expires_at_epoch_ms: Option<i64>,
    pub attempt_count: u32,
}

impl FifoPromptItem {
    /// Creates a newly enqueued prompt item.
    pub fn new(
        id: impl Into<String>,
        conversation_id: impl Into<String>,
        prompt_text: impl Into<String>,
        created_at_epoch_ms: i64,
    ) -> Self {
        Self {
            id: id.into(),
            conversation_id: conversation_id.into(),
            prompt_text: prompt_text.into(),
            created_at_epoch_ms,
            status: QueueItemStatus::Enqueued,
            leased_by_instance: None,
            lease_expires_at_epoch_ms: None,
            attempt_count: 0,
        }
    }

    /// Returns true if the lease has expired relative to current epoch ms.
    pub fn is_lease_expired(&self, current_time_ms: i64) -> bool {
        match self.lease_expires_at_epoch_ms {
            Some(exp) => current_time_ms >= exp,
            None => true,
        }
    }
}

/// SQL validator enforcing the FIFO queue ordering invariant.
pub struct FifoQueryValidator;

impl FifoQueryValidator {
    /// Validates an SQL query string to ensure it strictly respects FIFO dispatch invariants.
    pub fn validate_dispatch_query_sql(sql: &str) -> AppResult<()> {
        let upper = sql.to_uppercase();

        // 1. Strictly forbid inverted LIFO queries
        if upper.contains("ORDER BY UPDATED_AT DESC")
            || upper.contains("ORDER BY CREATED_AT DESC")
            || upper.contains("ORDER BY ROWID DESC")
        {
            return Err(AppError::Config(format!(
                "Forbidden LIFO ordering detected in dispatch query: '{}'. All dispatch queries must use FIFO 'ORDER BY created_at ASC, id ASC'.",
                sql
            )));
        }

        // 2. Must explicitly contain ORDER BY created_at
        if !upper.contains("ORDER BY CREATED_AT") && !upper.contains("ORDER BY C.CREATED_AT") {
            return Err(AppError::Config(format!(
                "Missing mandatory FIFO ordering in dispatch query: '{}'. Expected 'ORDER BY created_at ASC, id ASC'.",
                sql
            )));
        }

        Ok(())
    }
}

/// Coordinator managing in-memory FIFO prompt queues and generating compliant SQL queries.
#[derive(Debug, Clone, Default)]
pub struct FifoPromptQueueCoordinator {
    items: VecDeque<FifoPromptItem>,
}

impl FifoPromptQueueCoordinator {
    /// Creates an empty FIFO prompt queue coordinator.
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    /// Generates a standardized, FIFO-compliant SQL query string for queue dispatch.
    pub fn build_dispatch_query(table_name: &str) -> String {
        format!(
            "SELECT id, conversation_id, prompt_text, created_at, status, leased_by_instance, lease_expires_at \
             FROM {} \
             WHERE status = 'Enqueued' OR (status = 'Leased' AND lease_expires_at < ?1) \
             ORDER BY created_at ASC, id ASC \
             LIMIT ?2",
            table_name
        )
    }

    /// Generates a standardized, FIFO-compliant SQL query string for backup restoration.
    pub fn build_restore_query(table_name: &str) -> String {
        format!(
            "SELECT id, conversation_id, prompt_text, created_at \
             FROM {} \
             ORDER BY created_at ASC, id ASC \
             LIMIT ?1",
            table_name
        )
    }

    /// Enqueues a new prompt item, maintaining strict FIFO order by created_at then id.
    pub fn enqueue(&mut self, item: FifoPromptItem) -> AppResult<()> {
        if item.id.trim().is_empty() {
            return Err(AppError::Config(
                "Cannot enqueue prompt with empty id".to_string(),
            ));
        }

        // Binary search insertion index to enforce FIFO order
        let insert_idx = self
            .items
            .binary_search_by(|probe| {
                probe
                    .created_at_epoch_ms
                    .cmp(&item.created_at_epoch_ms)
                    .then_with(|| probe.id.cmp(&item.id))
            })
            .unwrap_or_else(|idx| idx);

        self.items.insert(insert_idx, item);
        Ok(())
    }

    /// Leases the next batch of available FIFO items to the given instance.
    pub fn lease_batch(
        &mut self,
        instance_id: &str,
        batch_size: usize,
        lease_duration_ms: i64,
        current_time_ms: i64,
    ) -> Vec<FifoPromptItem> {
        let mut leased = Vec::new();

        for item in self.items.iter_mut() {
            if leased.len() >= batch_size {
                break;
            }

            let is_available = match item.status {
                QueueItemStatus::Enqueued => true,
                QueueItemStatus::Leased => item.is_lease_expired(current_time_ms),
                _ => false,
            };

            if is_available {
                item.status = QueueItemStatus::Leased;
                item.leased_by_instance = Some(instance_id.to_string());
                item.lease_expires_at_epoch_ms = Some(current_time_ms + lease_duration_ms);
                item.attempt_count += 1;
                leased.push(item.clone());
            }
        }

        leased
    }

    /// Marks an item as successfully completed, removing it from active queue.
    pub fn complete(&mut self, item_id: &str) -> AppResult<FifoPromptItem> {
        if let Some(pos) = self.items.iter().position(|i| i.id == item_id) {
            let mut item = self.items.remove(pos).unwrap();
            item.status = QueueItemStatus::Completed;
            Ok(item)
        } else {
            Err(AppError::Config(format!(
                "Prompt item '{}' not found in queue",
                item_id
            )))
        }
    }

    /// Marks an item as failed.
    pub fn fail(&mut self, item_id: &str) -> AppResult<()> {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == item_id) {
            item.status = QueueItemStatus::Failed;
            item.leased_by_instance = None;
            item.lease_expires_at_epoch_ms = None;
            Ok(())
        } else {
            Err(AppError::Config(format!(
                "Prompt item '{}' not found in queue",
                item_id
            )))
        }
    }

    /// Reclaims all expired leases back to Enqueued state.
    pub fn reclaim_expired_leases(&mut self, current_time_ms: i64) -> usize {
        let mut reclaimed = 0;
        for item in self.items.iter_mut() {
            if item.status == QueueItemStatus::Leased && item.is_lease_expired(current_time_ms) {
                item.status = QueueItemStatus::Enqueued;
                item.leased_by_instance = None;
                item.lease_expires_at_epoch_ms = None;
                reclaimed += 1;
            }
        }
        reclaimed
    }

    /// Number of items currently tracked in the queue.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
