//! Type definitions for pre-migration snapshots and database export payloads.

pub use crate::modules::db_migration::account_guard::AccountsVerification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Verification snapshot representing accounts integrity state.
pub type AccountVerificationSnapshot = AccountsVerification;

/// Export payload holding table records and metadata for a specific SQLite database file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DatabaseExportPayload {
    pub db_name: String,
    pub tables: HashMap<String, Vec<serde_json::Value>>,
    pub total_rows: usize,
    pub exported_at: i64,
}

impl DatabaseExportPayload {
    pub fn new(db_name: impl Into<String>) -> Self {
        Self {
            db_name: db_name.into(),
            tables: HashMap::new(),
            total_rows: 0,
            exported_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn add_table(&mut self, table_name: impl Into<String>, rows: Vec<serde_json::Value>) {
        let rows_len = rows.len();
        self.total_rows += rows_len;
        self.tables.insert(table_name.into(), rows);
    }
}

/// Unified single-file pre-migration snapshot payload.
///
/// Contains account verification metadata to prevent data loss, alongside complete table dumps
/// for all historical Split-DB databases (`errors.db`, `repo_prompts.db`, `prompt-backups.db`, `task-history.db`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MigrationSnapshot {
    pub format_version: String,
    pub exported_at: i64,
    pub agm_version: String,
    pub accounts_verification: AccountVerificationSnapshot,
    pub databases: HashMap<String, HashMap<String, Vec<serde_json::Value>>>,
}

/// Backwards-compatible alias for `MigrationSnapshot`.
pub type PreMigrationSnapshot = MigrationSnapshot;
