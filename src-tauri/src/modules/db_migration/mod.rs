//! Unified zero-data-loss database migration and backup engine.

pub mod account_guard;
pub mod adapters;
pub mod backup;
pub mod restore;
pub mod types;
pub mod version;

#[cfg(test)]
pub mod tests;

use crate::error::AppError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub use account_guard::{
    backup_accounts_json, backup_accounts_json_in, capture_accounts_snapshot,
    capture_accounts_snapshot_in, get_accounts_backup_path, get_accounts_backup_path_in,
    get_accounts_json_path, get_accounts_json_path_in, rollback_accounts_json,
    rollback_accounts_json_in, verify_accounts_equivalence, verify_zero_account_loss,
    verify_zero_account_loss_in, AccountGuard, AccountsVerification,
};
pub use adapters::{
    adapt_active_prompt, adapt_backup_batch, adapt_conversation_sequence, adapt_error_log,
    adapt_failed_command, adapt_green_project, adapt_project_sequence, adapt_prompt_backup,
    adapt_running_project, adapt_task_history, get_bool, get_i64, get_opt_i64, get_opt_str,
    get_str,
};
pub use backup::{
    dump_table_rows, export_pre_migration_snapshot, export_pre_migration_snapshot_in,
    get_default_snapshot_path, get_default_snapshot_path_in, PreMigrationSnapshot,
};
pub use restore::{restore_from_snapshot, verify_row_counts, MigrationSummary};
pub use types::*;
pub use version::{
    get_schema_version, is_all_schemas_current, is_migrated_to_pascalcase, is_schema_current,
    record_schema_version, seed_initial_schema_version, TARGET_SCHEMA_VERSION,
};

/// Orchestrator entry point: fast boot check (<10ms).
/// If any database is unmigrated, runs pre-flight export, fresh restoration, and integrity verification.
/// Returns `Ok(true)` if migration was performed, or `Ok(false)` if already up to date.
pub fn check_and_run_migrations() -> Result<bool, AppError> {
    let data_dir = crate::modules::account::get_data_dir().map_err(AppError::Config)?;

    // Fast path: bypass if all existing databases are already on target schema version
    if is_all_schemas_current(&data_dir) {
        return Ok(false);
    }

    crate::modules::logger::log_info(
        "[DB Migration] Starting zero-data-loss automatic database migration...",
    );

    // 1. Acquire pre-flight account protection guard
    let guard = AccountGuard::acquire()?;

    // 2. Export pre-migration snapshot
    let snapshot_path = export_snapshot(None)?;

    // 3. Restore snapshot into freshly initialized PascalCase Split-DB schemas
    let summary = restore_snapshot(&snapshot_path, None)?;

    // 4. Verify post-migration integrity and 100% row counts
    verify_integrity(&snapshot_path, None)?;

    // 5. Verify and commit account preservation, releasing backup
    guard.verify_and_commit()?;

    crate::modules::logger::log_info(&format!(
        "[DB Migration] Automatic database migration successfully completed: {} records imported.",
        summary.total_imported
    ));

    Ok(true)
}

/// Exports a pre-migration snapshot containing all legacy databases and accounts verification.
pub fn export_snapshot(custom_path: Option<&Path>) -> Result<PathBuf, AppError> {
    export_snapshot_in(custom_path, None)
}

/// Exports a pre-migration snapshot with optional data directory override.
pub fn export_snapshot_in(
    custom_path: Option<&Path>,
    data_dir_override: Option<&Path>,
) -> Result<PathBuf, AppError> {
    export_pre_migration_snapshot_in(custom_path, data_dir_override)
}

/// Restores databases from a single-file snapshot into PascalCase schemas with field adapters.
pub fn restore_snapshot(
    snapshot_path: &Path,
    data_dir_override: Option<&Path>,
) -> Result<MigrationSummary, AppError> {
    restore_from_snapshot(snapshot_path, data_dir_override)
}

/// Verifies post-migration integrity, asserting 100% row count match and schema version compliance.
pub fn verify_integrity(
    snapshot_path: &Path,
    data_dir_override: Option<&Path>,
) -> Result<(), AppError> {
    let raw = std::fs::read(snapshot_path).map_err(AppError::Io)?;
    let snapshot: PreMigrationSnapshot = serde_json::from_slice(&raw)
        .map_err(|e| AppError::Config(format!("Failed to parse pre-migration snapshot: {}", e)))?;

    let data_dir = match data_dir_override {
        Some(p) => p.to_path_buf(),
        None => crate::modules::account::get_data_dir().map_err(AppError::Config)?,
    };

    // 1. Verify account preservation
    verify_zero_account_loss_in(&snapshot.accounts_verification, data_dir_override)?;

    // 2. Open connections to verify row counts
    let errors_db_path = data_dir.join("errors.db");
    let repo_db_path = data_dir.join("repo_prompts.db");
    let backup_db_path = data_dir.join("backup-prompts").join("backup-prompts.db");
    let task_history_db_path = data_dir.join("task-history.db");

    let errors_conn = Connection::open(&errors_db_path).map_err(AppError::Database)?;
    let repo_conn = Connection::open(&repo_db_path).map_err(AppError::Database)?;
    let backup_conn = Connection::open(&backup_db_path).map_err(AppError::Database)?;
    let task_history_conn = Connection::open(&task_history_db_path).map_err(AppError::Database)?;

    // 3. Assert 100% row count match
    verify_row_counts(
        &snapshot,
        &errors_conn,
        &repo_conn,
        &backup_conn,
        &task_history_conn,
    )?;

    // 4. Assert schema versions
    if !is_schema_current(&errors_conn)
        || !is_schema_current(&repo_conn)
        || !is_schema_current(&backup_conn)
        || !is_schema_current(&task_history_conn)
    {
        let msg = "Integrity check failed: One or more databases have SchemaVersion < 1";
        crate::modules::logger::log_error(msg);
        return Err(AppError::Database(rusqlite::Error::StatementChangedRows(0)));
    }

    crate::modules::logger::log_info("[DB Migration] Post-migration integrity verified 100%.");

    Ok(())
}
