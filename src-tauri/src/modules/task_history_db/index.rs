use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

pub(crate) fn payload_emails(payload_json: Option<&str>) -> (String, String) {
    let Some(raw) = payload_json else {
        return (String::new(), String::new());
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return (String::new(), String::new());
    };
    let text = |key: &str| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string()
    };
    (text("from_email"), text("to_email"))
}

pub(crate) fn complete(
    id: &str,
    status: &str,
    detail: &str,
    payload_json: Option<&str>,
) -> Result<(), String> {
    if id.is_empty() {
        return Ok(());
    }
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let now = Utc::now().timestamp();
    let (from_email, to_email) = payload_emails(payload_json);
    let mut updated_in_split = false;
    for split in splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let changed = if let Some(payload) = payload_json {
            conn.execute(
                "UPDATE tasks SET status = ?1, detail = ?2, finished_at = ?3, payload_json = ?4, from_email = ?5, to_email = ?6 WHERE id = ?7",
                params![status, detail, now, payload, from_email, to_email, id],
            )
            .map_err(|err| err.to_string())?
        } else {
            conn.execute(
                "UPDATE tasks SET status = ?1, detail = ?2, finished_at = ?3 WHERE id = ?4",
                params![status, detail, now, id],
            )
            .map_err(|err| err.to_string())?
        };
        if changed > 0 {
            updated_in_split = true;
            break;
        }
    }

    // Synchronously update hot_tasks_cache and cap at 200 entries
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "UPDATE hot_tasks_cache SET status = ?1, detail = ?2, finished_at = ?3, from_email = ?4, to_email = ?5 WHERE id = ?6",
        params![status, detail, now, from_email, to_email, id],
    ), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "DELETE FROM hot_tasks_cache WHERE id NOT IN (SELECT id FROM hot_tasks_cache ORDER BY created_at DESC LIMIT 200)",
        [],
    ), "db execute");

    if updated_in_split {
        Ok(())
    } else {
        Err(format!("history task '{}' was not found", id))
    }
}

pub(crate) fn history_index_path(dir: &Path) -> PathBuf {
    dir.join("task_index.db")
}

pub(crate) fn open_index(dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let conn = Connection::open(history_index_path(dir)).map_err(|err| err.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS split_catalog (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            file_path TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            closed_at INTEGER,
            row_count INTEGER NOT NULL DEFAULT 0,
            is_current INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS hot_tasks_cache (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            action_code INTEGER,
            status TEXT NOT NULL,
            subject TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            instance_id TEXT NOT NULL DEFAULT '',
            split_path TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            finished_at INTEGER,
            from_email TEXT NOT NULL DEFAULT '',
            to_email TEXT NOT NULL DEFAULT ''
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    Ok(conn)
}

pub(crate) fn open_split(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            subject TEXT NOT NULL,
            detail TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            finished_at INTEGER
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE tasks ADD COLUMN action_code INTEGER", []),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN payload_json TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN from_email TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN to_email TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE tasks
         SET from_email = COALESCE(json_extract(payload_json, '$.from_email'), from_email),
             to_email = COALESCE(json_extract(payload_json, '$.to_email'), to_email)
         WHERE payload_json <> '' AND (from_email = '' OR to_email = '')",
            [],
        ),
        "db execute",
    );
    Ok(conn)
}

pub(crate) fn current_split(index: &Connection, dir: &Path) -> Result<(String, PathBuf), String> {
    let existing: Option<(String, String)> = index
        .query_row(
            "SELECT id, file_path FROM split_catalog WHERE is_current = 1 ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    if let Some((id, path)) = existing {
        return Ok((id, PathBuf::from(path)));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let path = dir.join(format!("history-{}.db", &id[..8]));
    let now = Utc::now().timestamp();
    let _ = open_split(&path)?;
    index
        .execute(
            "INSERT INTO split_catalog (id, kind, file_path, created_at, closed_at, row_count, is_current)
             VALUES (?1, 'history', ?2, ?3, NULL, 0, 1)",
            params![id, path.to_string_lossy().to_string(), now],
        )
        .map_err(|err| err.to_string())?;
    Ok((id, path))
}

pub(crate) fn load_splits(index: &Connection, _dir: &Path) -> Result<Vec<SplitInfo>, String> {
    let mut stmt = index
        .prepare(
            "SELECT id, file_path, created_at, closed_at, row_count, is_current
             FROM split_catalog ORDER BY created_at DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SplitInfo {
                id: row.get(0)?,
                file_path: row.get(1)?,
                created_at: row.get(2)?,
                closed_at: row.get(3)?,
                row_count: row.get(4)?,
                is_current: row.get::<_, i64>(5)? == 1,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}
