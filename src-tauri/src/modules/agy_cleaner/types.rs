use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub conversation_id: String,
    pub title: String,
    pub preview: String,
    pub step_count: i64,
    pub workspace_uris: String,
    pub last_modified_time: String,
    pub db_path: String,
    pub file_size: u64,
    pub is_preserved: bool,
}

/// Pre-flight report for dry-run inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub total_conversations: usize,
    pub keep_count: usize,
    pub preserved_count: usize,
    pub pruned_count: usize,
    pub total_conversation_bytes: u64,
    pub projected_reclaimed_bytes: u64,
    pub cache_paths_count: usize,
    pub cache_bytes: u64,
    pub conversations_to_preserve: Vec<ConversationItem>,
    pub conversations_to_prune: Vec<ConversationItem>,
    pub cache_targets: Vec<String>,
    pub staging_dir: String,
}

/// Result of an applied pruning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneResult {
    pub transaction_id: String,
    pub keep_count: usize,
    pub preserved_count: usize,
    pub pruned_count: usize,
    pub pruned_bytes: u64,
    pub cache_cleared_bytes: u64,
    pub total_freed_bytes: u64,
    pub staging_dir: String,
    pub errors: Vec<String>,
}

/// Result of an undo restoration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub transaction_id: String,
    pub restored_conversations: usize,
    pub restored_bytes: u64,
    pub errors: Vec<String>,
}

/// Staged item record in transaction manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedItem {
    pub conversation_id: String,
    pub db_staged_path: String,
    pub db_original_path: String,
    pub brain_staged_path: Option<String>,
    pub brain_original_path: Option<String>,
    pub size: u64,
}

/// Manifest saved in the temp staging directory for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionManifest {
    pub transaction_id: String,
    pub timestamp: String,
    pub keep_count: usize,
    pub items: Vec<StagedItem>,
}
