//! Zero-data-loss snapshot re-importer and fresh schema initializer.

use crate::error::AppError;
use crate::modules::db_migration::account_guard::{
    verify_zero_account_loss, verify_zero_account_loss_in,
};
use crate::modules::db_migration::adapters;
use crate::modules::db_migration::backup::PreMigrationSnapshot;
use crate::modules::db_migration::version::seed_initial_schema_version;
use crate::modules::db_orm::*;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Result summary of the migration re-import process
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MigrationSummary {
    pub errors_imported: usize,
    pub projects_imported: usize,
    pub prompts_imported: usize,
    pub failed_commands_imported: usize,
    pub project_sequences_imported: usize,
    pub conversation_sequences_imported: usize,
    pub batches_imported: usize,
    pub backups_imported: usize,
    pub green_projects_imported: usize,
    pub task_history_imported: usize,
    pub total_imported: usize,
}

impl MigrationSummary {
    /// Recalculates total imported records count.
    pub fn calculate_total(&mut self) {
        self.total_imported = self.errors_imported
            + self.projects_imported
            + self.prompts_imported
            + self.failed_commands_imported
            + self.project_sequences_imported
            + self.conversation_sequences_imported
            + self.batches_imported
            + self.backups_imported
            + self.green_projects_imported
            + self.task_history_imported;
    }
}

/// Helper to query table row count in SQLite.
fn get_table_count(conn: &Connection, table_name: &str) -> Result<usize, AppError> {
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            [table_name],
            |r| r.get(0),
        )
        .map_err(AppError::Database)?;

    if exists == 0 {
        return Ok(0);
    }

    let count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |r| {
            r.get(0)
        })
        .map_err(AppError::Database)?;

    Ok(count as usize)
}

/// Asserts 100% row count match between snapshot and destination tables.
pub fn verify_row_counts(
    snapshot: &PreMigrationSnapshot,
    errors_conn: &Connection,
    repo_conn: &Connection,
    backup_conn: &Connection,
    task_history_conn: &Connection,
) -> Result<(), AppError> {
    // 1. Verify errors.db ErrorLog count
    if let Some(err_db) = snapshot
        .databases
        .get("errors.db")
        .or_else(|| snapshot.databases.get("errors"))
    {
        let expected = err_db
            .get("errors")
            .or_else(|| err_db.get("ErrorLog"))
            .map(|v| v.len())
            .unwrap_or(0);
        let actual = get_table_count(errors_conn, "ErrorLog")?;
        if actual < expected {
            let msg = format!(
                "100% row count verification failed for ErrorLog: expected {} rows, but found {}",
                expected, actual
            );
            crate::modules::logger::log_error(&msg);
            return Err(AppError::Database(rusqlite::Error::StatementChangedRows(
                expected,
            )));
        }
    }

    // 2. Verify repo_prompts.db tables
    if let Some(repo_db) = snapshot
        .databases
        .get("repo_prompts.db")
        .or_else(|| snapshot.databases.get("repo-prompts.db"))
    {
        let checks = [
            (
                &["running_projects", "RunningProject"][..],
                "RunningProject",
            ),
            (&["active_prompts", "ActivePrompt"][..], "ActivePrompt"),
            (
                &["agm_project_sequences", "ProjectSequence"][..],
                "ProjectSequence",
            ),
            (
                &["agm_conversation_sequences", "ConversationSequence"][..],
                "ConversationSequence",
            ),
            (&["failed_commands", "FailedCommand"][..], "FailedCommand"),
        ];

        for (keys, target_table) in checks {
            let expected = keys
                .iter()
                .find_map(|k| repo_db.get(*k))
                .map(|v| v.len())
                .unwrap_or(0);
            let actual = get_table_count(repo_conn, target_table)?;
            if actual < expected {
                let msg = format!(
                    "100% row count verification failed for {}: expected {} rows, but found {}",
                    target_table, expected, actual
                );
                crate::modules::logger::log_error(&msg);
                return Err(AppError::Database(rusqlite::Error::StatementChangedRows(
                    expected,
                )));
            }
        }
    }

    // 3. Verify prompt-backups.db tables
    if let Some(bk_db) = snapshot
        .databases
        .get("prompt-backups.db")
        .or_else(|| snapshot.databases.get("backup-prompts.db"))
        .or_else(|| snapshot.databases.get("prompt_backups"))
    {
        let checks = [
            (&["backup_batches", "BackupBatch"][..], "BackupBatch"),
            (&["prompt_backups", "PromptBackup"][..], "PromptBackup"),
            (&["green_projects", "GreenProject"][..], "GreenProject"),
        ];

        for (keys, target_table) in checks {
            let expected = keys
                .iter()
                .find_map(|k| bk_db.get(*k))
                .map(|v| v.len())
                .unwrap_or(0);
            let actual = get_table_count(backup_conn, target_table)?;
            if actual < expected {
                let msg = format!(
                    "100% row count verification failed for {}: expected {} rows, but found {}",
                    target_table, expected, actual
                );
                crate::modules::logger::log_error(&msg);
                return Err(AppError::Database(rusqlite::Error::StatementChangedRows(
                    expected,
                )));
            }
        }
    }

    // 4. Verify task-history.db tables
    if let Some(th_db) = snapshot
        .databases
        .get("task-history.db")
        .or_else(|| snapshot.databases.get("task_history"))
    {
        let expected = th_db
            .get("task_history")
            .or_else(|| th_db.get("TaskHistory"))
            .map(|v| v.len())
            .unwrap_or(0);
        let actual = get_table_count(task_history_conn, "TaskHistory")?;
        if actual < expected {
            let msg = format!(
                "100% row count verification failed for TaskHistory: expected {} rows, but found {}",
                expected, actual
            );
            crate::modules::logger::log_error(&msg);
            return Err(AppError::Database(rusqlite::Error::StatementChangedRows(
                expected,
            )));
        }
    }

    crate::modules::logger::log_info(
        "[DB Migration] 100% row count verification successfully confirmed across all split databases.",
    );

    Ok(())
}

/// Imports snapshot data into freshly initialized PascalCase Split-DB databases.
pub fn restore_from_snapshot(
    snapshot_path: &Path,
    data_dir_override: Option<&Path>,
) -> Result<MigrationSummary, AppError> {
    let raw = fs::read(snapshot_path).map_err(AppError::Io)?;
    let snapshot: PreMigrationSnapshot = serde_json::from_slice(&raw)
        .map_err(|e| AppError::Config(format!("Failed to parse pre-migration snapshot: {}", e)))?;

    // 1. Verify zero account loss prior to restore
    verify_zero_account_loss_in(&snapshot.accounts_verification, data_dir_override)?;

    let data_dir = match data_dir_override {
        Some(p) => p.to_path_buf(),
        None => crate::modules::account::get_data_dir().map_err(AppError::Config)?,
    };

    let mut summary = MigrationSummary::default();

    // 2. Fresh initialize and import errors.db
    let errors_db_path = data_dir.join("errors.db");
    let errors_conn = open_split_db(&errors_db_path)?;
    init_split_db_schema(&errors_conn, "errors.db")?;

    if let Some(errors_db) = snapshot
        .databases
        .get("errors.db")
        .or_else(|| snapshot.databases.get("errors"))
    {
        let err_rows = errors_db
            .get("errors")
            .or_else(|| errors_db.get("ErrorLog"))
            .cloned()
            .unwrap_or_default();

        for row in err_rows {
            match adapters::adapt_error_log(&row) {
                Ok(entity) => match QueryBuilder::insert(&errors_conn, &entity) {
                    Ok(_) => summary.errors_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert ErrorLog row during snapshot restore",
                        );
                        // Justification: Non-fatal insert failure recorded; row count verification will assert zero data loss.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt ErrorLog row during snapshot restore",
                    );
                    // Justification: Adapter failures recorded; row count verification ensures integrity.
                }
            }
        }
    }
    seed_initial_schema_version(&errors_conn, None, None)?;

    // 3. Fresh initialize and import repo_prompts.db
    let repo_db_path = data_dir.join("repo_prompts.db");
    let repo_conn = open_split_db(&repo_db_path)?;
    init_split_db_schema(&repo_conn, "repo-prompts.db")?;

    if let Some(repo_db) = snapshot
        .databases
        .get("repo_prompts.db")
        .or_else(|| snapshot.databases.get("repo-prompts.db"))
    {
        // Projects
        let proj_rows = repo_db
            .get("running_projects")
            .or_else(|| repo_db.get("RunningProject"))
            .cloned()
            .unwrap_or_default();

        for row in proj_rows {
            match adapters::adapt_running_project(&row) {
                Ok(entity) => match QueryBuilder::insert(&repo_conn, &entity) {
                    Ok(_) => summary.projects_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert RunningProject during snapshot restore",
                        );
                        // Justification: Individual insert failure recorded; row count verification will detect discrepancy.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt RunningProject during snapshot restore",
                    );
                    // Justification: Adapter error recorded; verification step validates row count.
                }
            }
        }

        // Active Prompts
        let prompt_rows = repo_db
            .get("active_prompts")
            .or_else(|| repo_db.get("ActivePrompt"))
            .cloned()
            .unwrap_or_default();

        for row in prompt_rows {
            match adapters::adapt_active_prompt(&row) {
                Ok(entity) => match QueryBuilder::insert(&repo_conn, &entity) {
                    Ok(_) => summary.prompts_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert ActivePrompt during snapshot restore",
                        );
                        // Justification: Individual insert failure recorded; row count verification will detect discrepancy.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt ActivePrompt during snapshot restore",
                    );
                    // Justification: Adapter error recorded; verification step validates row count.
                }
            }
        }

        // Project Sequences
        let proj_seq_rows = repo_db
            .get("agm_project_sequences")
            .or_else(|| repo_db.get("ProjectSequence"))
            .cloned()
            .unwrap_or_default();

        for row in proj_seq_rows {
            match adapters::adapt_project_sequence(&row) {
                Ok(entity) => match QueryBuilder::insert(&repo_conn, &entity) {
                    Ok(_) => summary.project_sequences_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert ProjectSequence during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt ProjectSequence during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }

        // Conversation Sequences
        let conv_seq_rows = repo_db
            .get("agm_conversation_sequences")
            .or_else(|| repo_db.get("ConversationSequence"))
            .cloned()
            .unwrap_or_default();

        for row in conv_seq_rows {
            match adapters::adapt_conversation_sequence(&row) {
                Ok(entity) => match QueryBuilder::insert(&repo_conn, &entity) {
                    Ok(_) => summary.conversation_sequences_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert ConversationSequence during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt ConversationSequence during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }

        // Failed Commands
        let failed_rows = repo_db
            .get("failed_commands")
            .or_else(|| repo_db.get("FailedCommand"))
            .cloned()
            .unwrap_or_default();

        for row in failed_rows {
            match adapters::adapt_failed_command(&row) {
                Ok(entity) => match QueryBuilder::insert(&repo_conn, &entity) {
                    Ok(_) => summary.failed_commands_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert FailedCommand during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt FailedCommand during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }
    }
    seed_initial_schema_version(&repo_conn, None, None)?;

    // 4. Fresh initialize and import prompt-backups.db
    let backup_dir = data_dir.join("backup-prompts");
    let backup_db_path = backup_dir.join("backup-prompts.db");
    let backup_conn = open_split_db(&backup_db_path)?;
    init_split_db_schema(&backup_conn, "prompt-backups.db")?;

    if let Some(backup_db) = snapshot
        .databases
        .get("prompt-backups.db")
        .or_else(|| snapshot.databases.get("backup-prompts.db"))
        .or_else(|| snapshot.databases.get("prompt_backups"))
    {
        // Batches
        let batch_rows = backup_db
            .get("backup_batches")
            .or_else(|| backup_db.get("BackupBatch"))
            .cloned()
            .unwrap_or_default();

        for row in batch_rows {
            match adapters::adapt_backup_batch(&row) {
                Ok(entity) => match QueryBuilder::insert(&backup_conn, &entity) {
                    Ok(_) => summary.batches_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert BackupBatch during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt BackupBatch during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }

        // Backups
        let backup_rows = backup_db
            .get("prompt_backups")
            .or_else(|| backup_db.get("PromptBackup"))
            .cloned()
            .unwrap_or_default();

        for row in backup_rows {
            match adapters::adapt_prompt_backup(&row) {
                Ok(entity) => match QueryBuilder::insert(&backup_conn, &entity) {
                    Ok(_) => summary.backups_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert PromptBackup during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt PromptBackup during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }

        // Green Projects
        let green_rows = backup_db
            .get("green_projects")
            .or_else(|| backup_db.get("GreenProject"))
            .cloned()
            .unwrap_or_default();

        for row in green_rows {
            match adapters::adapt_green_project(&row) {
                Ok(entity) => match QueryBuilder::insert(&backup_conn, &entity) {
                    Ok(_) => summary.green_projects_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert GreenProject during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt GreenProject during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }
    }
    seed_initial_schema_version(&backup_conn, None, None)?;

    // 5. Fresh initialize and import task-history.db
    let task_history_db_path = data_dir.join("task-history.db");
    let task_history_conn = open_split_db(&task_history_db_path)?;
    init_split_db_schema(&task_history_conn, "task-history.db")?;

    if let Some(th_db) = snapshot
        .databases
        .get("task-history.db")
        .or_else(|| snapshot.databases.get("task_history"))
    {
        let th_rows = th_db
            .get("task_history")
            .or_else(|| th_db.get("TaskHistory"))
            .cloned()
            .unwrap_or_default();

        for row in th_rows {
            match adapters::adapt_task_history(&row) {
                Ok(entity) => match QueryBuilder::insert(&task_history_conn, &entity) {
                    Ok(_) => summary.task_history_imported += 1,
                    Err(e) => {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "insert TaskHistory during snapshot restore",
                        );
                        // Justification: Insert error recorded; row count verification validates integrity.
                    }
                },
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "adapt TaskHistory during snapshot restore",
                    );
                    // Justification: Adapter error recorded; row count verification validates integrity.
                }
            }
        }
    }
    seed_initial_schema_version(&task_history_conn, None, None)?;

    // 6. Verify 100% row count match across all 4 Split-DBs
    verify_row_counts(
        &snapshot,
        &errors_conn,
        &repo_conn,
        &backup_conn,
        &task_history_conn,
    )?;

    // 7. Final zero account loss verification post-migration
    verify_zero_account_loss_in(&snapshot.accounts_verification, data_dir_override)?;

    summary.calculate_total();

    crate::modules::logger::log_info(&format!(
        "[DB Migration] Successfully migrated to PascalCase schemas: {:?}",
        summary
    ));

    Ok(summary)
}
