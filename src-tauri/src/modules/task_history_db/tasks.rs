use crate::modules::audit_action::{resolve_action, AuditAction};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

pub fn record(
    action: AuditAction,
    subject: &str,
    status: &str,
    detail: &str,
    instance_id: Option<&str>,
) {
    match enqueue(action, subject, status, instance_id) {
        Ok(id) => {
            if status != "queued" && status != "running" {
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(complete(&id, status, detail, None), "complete");
            }
        }
        Err(err) => {
            crate::modules::logger::log_warn(&format!("[History] record failed: {}", err));
        }
    }
}

pub fn history_dir() -> Result<PathBuf, String> {
    let local = PathBuf::from("data").join("task-history");
    if local.exists() {
        return Ok(local);
    }
    let data_dir = crate::modules::account::get_data_dir()?;
    let dir = data_dir.join("task-history");
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create task-history folder: {}", err))?;
    Ok(dir)
}

pub fn list_page(offset: u32, limit: u32) -> Result<TaskHistoryPage, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let total: i64 = splits.iter().map(|split| split.row_count).sum();

    // Fast path: if offset == 0 && limit <= 200, query hot_tasks_cache directly for sub-millisecond response
    if offset == 0 && limit <= 200 {
        let mut stmt = index
            .prepare(
                "SELECT id, action, action_code, status, subject, detail, instance_id, split_path, created_at, finished_at,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM hot_tasks_cache ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: row.get(7)?,
                    created_at: row.get(8)?,
                    finished_at: row.get(9)?,
                    from_email: row.get(10)?,
                    to_email: row.get(11)?,
                })
            })
            .map_err(|err| err.to_string())?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| err.to_string())?);
        }
        if !items.is_empty() || total == 0 {
            return Ok(TaskHistoryPage {
                total,
                offset,
                limit,
                items,
                splits,
            });
        }
    }

    let mut items = Vec::new();
    let mut skip = offset as i64;
    let mut need = limit.max(1) as i64;
    for split in &splits {
        if need <= 0 {
            break;
        }
        if skip >= split.row_count {
            skip -= split.row_count;
            continue;
        }
        let take = need.min(split.row_count - skip);
        let conn = open_split(Path::new(&split.file_path))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM tasks ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map(params![take, skip], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: split.file_path.clone(),
                    created_at: row.get(7)?,
                    finished_at: row.get(8)?,
                    from_email: row.get(9)?,
                    to_email: row.get(10)?,
                })
            })
            .map_err(|err| err.to_string())?;
        for row in rows {
            items.push(row.map_err(|err| err.to_string())?);
        }
        skip = 0;
        need -= take;
    }
    Ok(TaskHistoryPage {
        total,
        offset,
        limit,
        items,
        splits,
    })
}

pub(crate) fn enqueue(
    action: AuditAction,
    subject: &str,
    status: &str,
    instance_id: Option<&str>,
) -> Result<String, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let (split_id, split_path) = current_split(&index, &dir)?;
    let split = open_split(&split_path)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    split
        .execute(
            "INSERT INTO tasks (id, action, action_code, status, subject, detail, instance_id, payload_json, created_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, '', ?7, NULL)",
            params![
                id,
                action.pascal(),
                action.code(),
                status,
                subject,
                instance_id.unwrap_or(""),
                now
            ],
        )
        .map_err(|err| err.to_string())?;
    index
        .execute(
            "UPDATE split_catalog SET row_count = row_count + 1 WHERE id = ?1",
            params![split_id],
        )
        .map_err(|err| err.to_string())?;
    let count: i64 = index
        .query_row(
            "SELECT row_count FROM split_catalog WHERE id = ?1",
            params![split_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if count >= SPLIT_ROW_CAP {
        index
            .execute(
                "UPDATE split_catalog SET is_current = 0, closed_at = ?1 WHERE id = ?2",
                params![now, split_id],
            )
            .map_err(|err| err.to_string())?;
    }

    // Synchronously update hot_tasks_cache and cap at 200 entries
    let split_path_str = split_path.to_string_lossy().to_string();
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "INSERT OR REPLACE INTO hot_tasks_cache (id, action, action_code, status, subject, detail, instance_id, split_path, created_at, finished_at, from_email, to_email)
         VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, ?8, NULL, '', '')",
        params![
            id,
            action.pascal(),
            action.code(),
            status,
            subject,
            instance_id.unwrap_or(""),
            split_path_str,
            now
        ],
    ), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "DELETE FROM hot_tasks_cache WHERE id NOT IN (SELECT id FROM hot_tasks_cache ORDER BY created_at DESC LIMIT 200)",
        [],
    ), "db execute");

    Ok(id)
}

pub fn get_detail(id: &str) -> Result<TaskDetail, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    for split in splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let row = conn
            .query_row(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at, payload_json
                 FROM tasks WHERE id = ?1",
                params![id],
                |row| {
                    let legacy: String = row.get(1)?;
                    let code: Option<i32> = row.get(2)?;
                    let (action_code, action, action_label) = resolve_action(code, &legacy);
                    Ok(TaskDetail {
                        id: row.get(0)?,
                        action_code,
                        action,
                        action_label,
                        status: row.get(3)?,
                        subject: row.get(4)?,
                        detail: row.get(5)?,
                        instance_id: row.get(6)?,
                        split_path: split.file_path.clone(),
                        created_at: row.get(7)?,
                        finished_at: row.get(8)?,
                        payload_json: row.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(|err| err.to_string())?;
        if let Some(detail) = row {
            return Ok(detail);
        }
    }
    Err(format!("history task '{}' was not found", id))
}
