use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::modules::account;
use crate::modules::agy_cleaner;
use crate::modules::repo_db::{self, ActivePrompt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBackupRecord {
    pub id: String,
    pub backup_batch_id: String,
    pub prompt_id: String,
    pub project_name: String,
    pub project_path: String,
    pub project_id: String,
    pub conversation_id: String,
    pub conversation_name: Option<String>,
    pub sequence_id: i64,
    pub prompt_text: String,
    pub has_images: bool,
    pub images_payload: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub is_restored: bool,
    pub restored_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupBatchInfo {
    pub id: String,
    pub created_at: i64,
    pub prompts_count: usize,
    pub file_path: String,
    pub retention_days: i64,
    pub is_fully_restored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenProjectRecord {
    pub id: String,
    pub project_identifier: String,
    pub project_path: String,
    pub added_at: i64,
    pub status: String,
    pub last_checked_at: Option<i64>,
}

/// Helper to resolve a human-readable friendly project name from path or ID without raw UUIDs
pub fn resolve_friendly_project_name(project_path: &str, project_id: &str) -> String {
    let clean_path = project_path.trim().replace('\\', "/");
    let trimmed_path = clean_path.trim_end_matches('/');
    if let Some(pos) = trimmed_path.rfind('/') {
        let name = &trimmed_path[pos + 1..];
        if !name.is_empty() && name != "." {
            return name.to_string();
        }
    } else if !trimmed_path.is_empty() && trimmed_path != "." {
        return trimmed_path.to_string();
    }

    let is_raw_uuid_or_hash = project_id.is_empty()
        || (project_id.len() >= 32
            && project_id
                .chars()
                .all(|c| c.is_ascii_hexdigit() || c == '-'));

    if !is_raw_uuid_or_hash {
        return project_id.to_string();
    }

    "Antigravity-Manager".to_string()
}

/// Determine target database path for running prompt backups
pub fn get_backup_prompts_db_path(custom_file: Option<&str>) -> Result<PathBuf, String> {
    if let Some(file_str) = custom_file {
        let trimmed = file_str.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.extension().is_none() {
                return Ok(path.with_extension("db"));
            }
            return Ok(path);
        }
    }

    // Check if local data/backup-prompts exists close to the CLI
    let local_backup_dir = PathBuf::from("data").join("backup-prompts");
    if local_backup_dir.exists() {
        if local_backup_dir.join("SQL.db").exists() {
            return Ok(local_backup_dir.join("SQL.db"));
        }
        return Ok(local_backup_dir.join("backup-prompts.db"));
    }

    let data_dir = account::get_data_dir()?;
    let backup_dir = data_dir.join("backup-prompts");
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        return Err(format!("Failed to create backup-prompts folder: {}", e));
    }
    if backup_dir.join("SQL.db").exists() {
        return Ok(backup_dir.join("SQL.db"));
    }
    Ok(backup_dir.join("backup-prompts.db"))
}

/// Open and initialize the split SQLite database connection
pub fn connect_backup_db(custom_file: Option<&str>) -> Result<Connection, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let conn = Connection::open(&db_path).map_err(|e| {
        format!(
            "Failed to open backup prompts database '{:?}': {}",
            db_path, e
        )
    })?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS backup_batches (
             id TEXT PRIMARY KEY,
             created_at INTEGER NOT NULL,
             prompts_count INTEGER NOT NULL,
             file_path TEXT NOT NULL,
             retention_days INTEGER NOT NULL DEFAULT 1,
             is_fully_restored BOOLEAN NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS prompt_backups (
             id TEXT PRIMARY KEY,
             backup_batch_id TEXT NOT NULL,
             prompt_id TEXT NOT NULL,
             project_name TEXT NOT NULL,
             project_path TEXT NOT NULL,
             project_id TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             conversation_name TEXT,
             sequence_id INTEGER NOT NULL,
             prompt_text TEXT NOT NULL,
             has_images BOOLEAN NOT NULL DEFAULT 0,
             images_payload TEXT,
             status TEXT NOT NULL DEFAULT 'queued',
             created_at INTEGER NOT NULL,
             is_restored BOOLEAN NOT NULL DEFAULT 0,
             restored_at INTEGER,
             FOREIGN KEY(backup_batch_id) REFERENCES backup_batches(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);

         CREATE TABLE IF NOT EXISTS green_projects (
             id TEXT PRIMARY KEY,
             project_identifier TEXT NOT NULL UNIQUE,
             project_path TEXT NOT NULL,
             added_at INTEGER NOT NULL,
             status TEXT NOT NULL DEFAULT 'pending',
             last_checked_at INTEGER
         );",
    )
    .map_err(|e| format!("Failed to initialize backup prompts tables: {}", e))?;

    Ok(conn)
}

/// Automatically clean up records that were restored more than 1 day ago (default 86400s)
pub fn auto_cleanup_expired(
    custom_file: Option<&str>,
    retention_seconds: i64,
) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let cutoff = Utc::now().timestamp() - retention_seconds;

    let removed = conn
        .execute(
            "DELETE FROM prompt_backups WHERE is_restored = 1 AND restored_at IS NOT NULL AND restored_at <= ?",
            params![cutoff],
        )
        .map_err(|e| format!("Failed to delete expired prompt backups: {}", e))?;

    // Also clean up empty fully-restored batches
    let _ = conn.execute(
        "DELETE FROM backup_batches WHERE is_fully_restored = 1 AND id NOT IN (SELECT DISTINCT backup_batch_id FROM prompt_backups)",
        [],
    );

    Ok(removed)
}

/// Force clean all backup prompts and batches
pub fn force_clean_all(custom_file: Option<&str>) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let count = conn
        .execute("DELETE FROM prompt_backups", [])
        .map_err(|e| format!("Failed to clear prompt backups: {}", e))?;
    let _ = conn.execute("DELETE FROM backup_batches", []);
    Ok(count)
}

/// Backup all currently active and queued running prompts into the split SQLite database
pub fn backup_active_running_prompts(
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let _ = auto_cleanup_expired(custom_file, 86400);

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = repo_db::discover_running_prompts_from_antigravity("default");
    let conversations = agy_cleaner::scan_conversations(100);
    let now = Utc::now().timestamp();
    let freshness_cutoff = now - 7200;

    // Consolidate candidate prompts to back up: prioritize live discovered prompts
    let mut candidate_prompts = Vec::new();
    for p in running_prompts.clone() {
        candidate_prompts.push(p);
    }
    for p in all_prompts {
        if (p.status == "running" || p.status == "queued" || p.status == "backed_up")
            && p.updated_at >= freshness_cutoff
            && !candidate_prompts.iter().any(|c| {
                c.id == p.id
                    || (c.repo_path.to_lowercase().replace('\\', "/")
                        == p.repo_path.to_lowercase().replace('\\', "/")
                        && c.prompt_content.trim() == p.prompt_content.trim())
            })
        {
            candidate_prompts.push(p);
        }
    }

    let conn = connect_backup_db(custom_file)?;
    let batch_id = format!("batch_{}", Uuid::new_v4().simple());

    let db_path_str = get_backup_prompts_db_path(custom_file)?
        .to_string_lossy()
        .to_string();

    let batch_info = BackupBatchInfo {
        id: batch_id.clone(),
        created_at: now,
        prompts_count: candidate_prompts.len(),
        file_path: db_path_str.clone(),
        retention_days: 1,
        is_fully_restored: false,
    };

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            batch_info.id,
            batch_info.created_at,
            batch_info.prompts_count,
            batch_info.file_path,
            batch_info.retention_days,
            batch_info.is_fully_restored,
        ],
    )
    .map_err(|e| format!("Failed to record backup batch: {}", e))?;

    let mut records = Vec::new();
    for (idx, p) in candidate_prompts.iter().enumerate() {
        let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || uris_norm.contains(&p.project_id.to_lowercase())
        });

        let conv_id = matched_conv
            .map(|c| c.conversation_id.clone())
            .or_else(|| p.session_id.clone())
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv.and_then(|c| {
            if c.title.trim().is_empty() {
                None
            } else {
                Some(c.title.clone())
            }
        });

        let proj_name = resolve_friendly_project_name(&p.repo_path, &p.project_id);

        let record = PromptBackupRecord {
            id: format!("rec_{}", Uuid::new_v4().simple()),
            backup_batch_id: batch_id.clone(),
            prompt_id: p.id.clone(),
            project_name: proj_name,
            project_path: p.repo_path.clone(),
            project_id: p.project_id.clone(),
            conversation_id: conv_id,
            conversation_name: conv_name,
            sequence_id: (idx + 1) as i64,
            prompt_text: p.prompt_content.clone(),
            has_images: p.image_payload.is_some(),
            images_payload: p.image_payload.clone(),
            status: p.status.clone(),
            created_at: now,
            is_restored: false,
            restored_at: None,
        };

        // Deduplicate records to prevent duplicate reinjections into prompt_backups
        let is_live_prompt = running_prompts.iter().any(|rp| rp.id == record.prompt_id);
        let existing_unrestored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0 LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_unrestored {
            let _ = conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2 WHERE id = ?3",
                params![record.backup_batch_id, now, existing_rec_id],
            );
            records.push(record);
            continue;
        }

        // If already restored recently and not currently actively running in Antigravity, skip re-queuing
        let already_restored_recently: bool = conn
            .query_row(
                "SELECT 1 FROM prompt_backups WHERE project_path = ?1 AND prompt_text = ?2 AND is_restored = 1 AND restored_at >= ?3 LIMIT 1",
                params![record.project_path, record.prompt_text, freshness_cutoff],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if already_restored_recently && !is_live_prompt {
            continue;
        }

        conn.execute(
            "INSERT INTO prompt_backups (
                id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.id,
                record.backup_batch_id,
                record.prompt_id,
                record.project_name,
                record.project_path,
                record.project_id,
                record.conversation_id,
                record.conversation_name,
                record.sequence_id,
                record.prompt_text,
                record.has_images,
                record.images_payload,
                record.status,
                record.created_at,
                record.is_restored,
                record.restored_at,
            ],
        )
        .map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;

        records.push(record);
    }

    Ok((batch_info, records))
}

/// List all backup batches and total counts
pub fn list_backup_batches(custom_file: Option<&str>) -> Result<Vec<BackupBatchInfo>, String> {
    let _ = auto_cleanup_expired(custom_file, 86400);
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
    let _ = auto_cleanup_expired(custom_file, 86400);
    let conn = connect_backup_db(custom_file)?;

    let (query, has_param) = if batch_id.is_some() {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at
             FROM prompt_backups WHERE backup_batch_id = ? ORDER BY sequence_id ASC",
            true,
        )
    } else {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at
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

fn parse_prompt_record(row: &rusqlite::Row) -> rusqlite::Result<PromptBackupRecord> {
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
    })
}

/// Restore running prompts: re-inserts them into repo_db and updates restoration flags
pub fn restore_running_prompts(
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();

    // Query unrestored prompts
    let mut stmt = conn
        .prepare(
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at
             FROM prompt_backups 
             WHERE is_restored = 0
             ORDER BY created_at ASC, sequence_id ASC",
        )
        .map_err(|e| format!("Failed to prepare restore query: {}", e))?;

    let records: Vec<PromptBackupRecord> = stmt
        .query_map([], |row| parse_prompt_record(row))
        .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
        .flatten()
        .collect();

    // Reset dispatched prompts cache to allow restored prompts to execute post-switch
    repo_db::reset_dispatched_prompts_cache();

    // Re-inject into repo_db active_prompts table if records exist in backup DB
    for rec in &records {
        let active_p = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: rec.project_id.clone(),
            instance_id: "default".to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id: Some(rec.conversation_id.clone()),
            status: "backed_up".to_string(),
            created_at: now,
            updated_at: now,
            image_payload: rec.images_payload.clone(),
        };
        let _ = repo_db::save_or_requeue_prompt(&active_p);
    }

    if !keep_backup && !records.is_empty() {
        let _ = conn.execute(
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ? WHERE is_restored = 0",
            params![now],
        );
        let _ = conn.execute(
            "UPDATE backup_batches SET is_fully_restored = 1 WHERE id IN (
                SELECT backup_batch_id FROM prompt_backups GROUP BY backup_batch_id HAVING min(is_restored) = 1
             )",
            [],
        );
    }

    // Automatically trigger resend and execute restored prompts via CLI
    let _ = repo_db::resend_all_running_commands(20);

    Ok(records)
}

/// Query storage information of the split database
pub fn get_storage_info(custom_file: Option<&str>) -> Result<serde_json::Value, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    let size_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let size_kb = (size_bytes as f64) / 1024.0;

    let conn = connect_backup_db(custom_file)?;
    let total_records: usize = conn
        .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
        .unwrap_or(0);
    let restored_records: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let batches_count: usize = conn
        .query_row("SELECT COUNT(*) FROM backup_batches", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(serde_json::json!({
        "database_path": db_path.to_string_lossy(),
        "size_bytes": size_bytes,
        "size_kb": format!("{:.2} KB", size_kb),
        "total_prompt_records": total_records,
        "restored_records": restored_records,
        "unrestored_records": total_records.saturating_sub(restored_records),
        "total_batches": batches_count,
        "retention_days": 1,
    }))
}

// ---------------------------------------------------------------------------
// Green Projects Store for FPUG & SUG Watchers
// ---------------------------------------------------------------------------

pub fn add_green_project(
    identifier: &str,
    project_path: &str,
    custom_file: Option<&str>,
) -> Result<(), String> {
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO green_projects (id, project_identifier, project_path, added_at, status, last_checked_at)
         VALUES (?, ?, ?, ?, 'pending', ?)
         ON CONFLICT(project_identifier) DO UPDATE SET project_path = excluded.project_path, last_checked_at = excluded.last_checked_at",
        params![
            format!("green_{}", Uuid::new_v4().simple()),
            identifier,
            project_path,
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to add green project: {}", e))?;
    Ok(())
}

pub fn remove_green_project(identifier: &str, custom_file: Option<&str>) -> Result<bool, String> {
    let conn = connect_backup_db(custom_file)?;
    let deleted = conn
        .execute(
            "DELETE FROM green_projects WHERE project_identifier = ? OR project_path = ?",
            params![identifier, identifier],
        )
        .map_err(|e| format!("Failed to remove green project: {}", e))?;
    Ok(deleted > 0)
}

pub fn list_green_projects(custom_file: Option<&str>) -> Result<Vec<GreenProjectRecord>, String> {
    let conn = connect_backup_db(custom_file)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_identifier, project_path, added_at, status, last_checked_at
             FROM green_projects ORDER BY added_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list green projects query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(GreenProjectRecord {
                id: row.get(0)?,
                project_identifier: row.get(1)?,
                project_path: row.get(2)?,
                added_at: row.get(3)?,
                status: row.get(4)?,
                last_checked_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query green projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_db_lifecycle() {
        let temp_dir =
            std::env::temp_dir().join(format!("agm_test_backup_{}", Uuid::new_v4().simple()));
        let db_path = temp_dir.join("test-backup.db");
        let custom_file = db_path.to_str().unwrap();

        let conn = connect_backup_db(Some(custom_file)).unwrap();
        assert!(db_path.exists());

        // Test insert and list batch
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
             VALUES ('b1', ?, 1, ?, 1, 0)",
            params![now, custom_file],
        ).unwrap();

        let batches = list_backup_batches(Some(custom_file)).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].id, "b1");

        // Test insert prompt backup
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored)
             VALUES ('p1', 'b1', 'prompt_1', 'my-project', '/path/to/repo', 'proj-1', 'conv-1', 1, 'Fix tests', ?, 0)",
            params![now],
        ).unwrap();

        let prompts = list_prompt_backups(None, Some(custom_file)).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].prompt_text, "Fix tests");

        // Test restore
        let restored = restore_running_prompts(false, Some(custom_file)).unwrap();
        assert_eq!(restored.len(), 1);

        // Verify second restore does not re-restore already restored prompts
        let restored_second = restore_running_prompts(false, Some(custom_file)).unwrap();
        assert_eq!(
            restored_second.len(),
            0,
            "Already restored prompts must not be restored again"
        );

        // Test auto cleanup (with future cutoff)
        let removed = auto_cleanup_expired(Some(custom_file), -10).unwrap();
        assert_eq!(removed, 1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
