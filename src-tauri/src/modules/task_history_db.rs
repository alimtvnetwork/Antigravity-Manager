use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const SPLIT_ROW_CAP: i64 = 500;

#[derive(Debug, Clone, Serialize)]
pub struct SplitInfo {
    pub id: String,
    pub file_path: String,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub row_count: i64,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskRecord {
    pub id: String,
    pub action: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    pub split_path: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskHistoryPage {
    pub total: i64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<TaskRecord>,
    pub splits: Vec<SplitInfo>,
}

pub struct AuditTask {
    id: String,
    open: bool,
}

impl AuditTask {
    pub fn start(action: &str, subject: &str, instance_id: Option<&str>) -> Self {
        match enqueue(action, subject, "queued", instance_id) {
            Ok(id) => Self { id, open: true },
            Err(err) => {
                crate::modules::logger::log_warn(&format!("[History] enqueue failed: {}", err));
                Self {
                    id: String::new(),
                    open: false,
                }
            }
        }
    }

    pub fn succeed(&mut self, detail: &str) {
        self.finish("ok", detail);
    }

    pub fn fail(&mut self, detail: &str) {
        self.finish("fail", detail);
    }

    fn finish(&mut self, status: &str, detail: &str) {
        if !self.open {
            return;
        }
        if let Err(err) = complete(&self.id, status, detail) {
            crate::modules::logger::log_warn(&format!("[History] complete failed: {}", err));
        }
        self.open = false;
    }
}

impl Drop for AuditTask {
    fn drop(&mut self) {
        if self.open {
            let _ = complete(&self.id, "fail", "stopped before the task finished");
        }
    }
}

pub fn record(action: &str, subject: &str, status: &str, detail: &str, instance_id: Option<&str>) {
    match enqueue(action, subject, status, instance_id) {
        Ok(id) => {
            if status != "queued" && status != "running" {
                let _ = complete(&id, status, detail);
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
                "SELECT id, action, status, subject, detail, instance_id, created_at, finished_at
                 FROM tasks ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map(params![take, skip], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action: row.get(1)?,
                    status: row.get(2)?,
                    subject: row.get(3)?,
                    detail: row.get(4)?,
                    instance_id: row.get(5)?,
                    split_path: split.file_path.clone(),
                    created_at: row.get(6)?,
                    finished_at: row.get(7)?,
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

fn enqueue(
    action: &str,
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
            "INSERT INTO tasks (id, action, status, subject, detail, instance_id, created_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, '', ?5, ?6, NULL)",
            params![id, action, status, subject, instance_id.unwrap_or(""), now],
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
    Ok(id)
}

fn complete(id: &str, status: &str, detail: &str) -> Result<(), String> {
    if id.is_empty() {
        return Ok(());
    }
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let now = Utc::now().timestamp();
    for split in splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let changed = conn
            .execute(
                "UPDATE tasks SET status = ?1, detail = ?2, finished_at = ?3 WHERE id = ?4",
                params![status, detail, now, id],
            )
            .map_err(|err| err.to_string())?;
        if changed > 0 {
            return Ok(());
        }
    }
    Err(format!("history task '{}' was not found", id))
}

fn history_index_path(dir: &Path) -> PathBuf {
    dir.join("task_index.db")
}

fn open_index(dir: &Path) -> Result<Connection, String> {
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
    Ok(conn)
}

fn open_split(path: &Path) -> Result<Connection, String> {
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
    Ok(conn)
}

fn current_split(index: &Connection, dir: &Path) -> Result<(String, PathBuf), String> {
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

fn load_splits(index: &Connection, _dir: &Path) -> Result<Vec<SplitInfo>, String> {
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
