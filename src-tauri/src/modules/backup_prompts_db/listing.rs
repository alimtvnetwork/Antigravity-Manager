use rusqlite::{params, Connection};

use super::*;

/// List all backup batches and total counts
pub fn list_backup_batches(custom_file: Option<&str>) -> Result<Vec<BackupBatchInfo>, String> {
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        auto_cleanup_expired(custom_file, 86400),
        "auto_cleanup_expired",
    );
    let conn = connect_backup_db(custom_file)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, prompts_count, file_path, retention_days, is_fully_restored
             FROM backup_batches ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list batches query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BackupBatchInfo {
                id: row.get(0)?,
                created_at: row.get(1)?,
                prompts_count: row.get(2)?,
                file_path: row.get(3)?,
                retention_days: row.get(4)?,
                is_fully_restored: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query batches: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// List prompt backups across batches or for a specific batch
pub fn list_prompt_backups(
    batch_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        auto_cleanup_expired(custom_file, 86400),
        "auto_cleanup_expired",
    );
    let conn = connect_backup_db(custom_file)?;

    let (query, has_param) = if batch_id.is_some() {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups WHERE backup_batch_id = ? ORDER BY sequence_id ASC",
            true,
        )
    } else {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups ORDER BY created_at DESC, sequence_id ASC",
            false,
        )
    };

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare list prompt backups query: {}", e))?;

    let rows: Vec<PromptBackupRecord> = if has_param {
        stmt.query_map(params![batch_id.unwrap()], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query prompt backups: {}", e))?
            .flatten()
            .collect()
    } else {
        stmt.query_map([], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query prompt backups: {}", e))?
            .flatten()
            .collect()
    };

    Ok(rows)
}

pub(crate) fn parse_prompt_record(row: &rusqlite::Row) -> rusqlite::Result<PromptBackupRecord> {
    Ok(PromptBackupRecord {
        id: row.get(0)?,
        backup_batch_id: row.get(1)?,
        prompt_id: row.get(2)?,
        project_name: row.get(3)?,
        project_path: row.get(4)?,
        project_id: row.get(5)?,
        conversation_id: row.get(6)?,
        conversation_name: row.get(7)?,
        sequence_id: row.get(8)?,
        prompt_text: row.get(9)?,
        has_images: row.get(10)?,
        images_payload: row.get(11)?,
        status: row.get(12)?,
        created_at: row.get(13)?,
        is_restored: row.get(14)?,
        restored_at: row.get(15)?,
        instance_id: row.get(16).ok(),
    })
}
