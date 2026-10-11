use super::types::*;
use crate::error::AppError;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// Initialize the standalone compact SQLite database schema with indexes and constraints.
pub fn init_compact_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS meta (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             schema_version TEXT NOT NULL DEFAULT '2.0',
             created_at TEXT NOT NULL,
             source_machine TEXT NOT NULL,
             source_work_dir TEXT NOT NULL,
             agm_version TEXT NOT NULL,
             notes TEXT,
             is_portable INTEGER NOT NULL DEFAULT 1
         );

         CREATE TABLE IF NOT EXISTS pinned_projects (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             project_key TEXT NOT NULL UNIQUE,
             repo_name TEXT NOT NULL,
             relative_path TEXT NOT NULL,
             pinned_at TEXT NOT NULL,
             is_active INTEGER NOT NULL DEFAULT 1,
             display_order INTEGER NOT NULL DEFAULT 0,
             notes TEXT
         );

         CREATE TABLE IF NOT EXISTS conversations (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             conversation_id TEXT NOT NULL UNIQUE,
             project_key TEXT NOT NULL,
             seq_id INTEGER NOT NULL,
             title TEXT NOT NULL,
             relative_path TEXT NOT NULL,
             last_active_at TEXT NOT NULL,
             message_count INTEGER NOT NULL DEFAULT 0,
             is_archived INTEGER NOT NULL DEFAULT 0,
             has_active_prompt INTEGER NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS running_prompts (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             prompt_id TEXT NOT NULL UNIQUE,
             project_key TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             prompt_content TEXT NOT NULL,
             model TEXT,
             status TEXT NOT NULL,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL,
             image_payload TEXT,
             is_restored INTEGER NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS prompt_history (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             history_id TEXT NOT NULL UNIQUE,
             project_key TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             prompt_content TEXT NOT NULL,
             model TEXT,
             role TEXT NOT NULL DEFAULT 'user',
             executed_at TEXT NOT NULL,
             exit_code INTEGER,
             has_completed INTEGER NOT NULL DEFAULT 1,
             duration_ms INTEGER NOT NULL DEFAULT 0
         );

         CREATE INDEX IF NOT EXISTS idx_pinned_projects_key ON pinned_projects(project_key);
         CREATE INDEX IF NOT EXISTS idx_conversations_proj ON conversations(project_key);
         CREATE INDEX IF NOT EXISTS idx_running_prompts_status ON running_prompts(status);
         CREATE INDEX IF NOT EXISTS idx_prompt_history_executed ON prompt_history(executed_at DESC);",
    )
    .map_err(AppError::from)?;

    Ok(())
}

/// Connects to a standalone compact SQLite database file.
pub fn connect_compact_db(path: &Path, is_create: bool) -> Result<Connection, AppError> {
    if is_create {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(format!("Failed to create directory {parent:?}: {e}"))
            })?;
        }
    }

    let conn = Connection::open(path)
        .map_err(|e| AppError::Config(format!("Failed to open compact DB {path:?}: {e}")))?;

    init_compact_schema(&conn)?;
    Ok(conn)
}

/// Serializes and writes a full `CompactExportData` snapshot into a portable SQLite file.
pub fn write_compact_db(path: &Path, data: &CompactExportData) -> Result<(), AppError> {
    // If destination already exists, remove it first to ensure clean snapshot
    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            AppError::Config(format!(
                "Failed to remove existing compact DB {path:?}: {e}"
            ))
        })?;
    }

    let mut conn = connect_compact_db(path, true)?;
    let tx = conn.transaction().map_err(AppError::from)?;

    // 1. Meta table
    tx.execute(
        "INSERT INTO meta (schema_version, created_at, source_machine, source_work_dir, agm_version, notes, is_portable)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            data.meta.schema_version,
            data.meta.created_at,
            data.meta.source_machine,
            data.meta.source_work_dir,
            env!("CARGO_PKG_VERSION"),
            data.meta.notes,
            if data.meta.is_portable { 1 } else { 0 },
        ],
    )
    .map_err(AppError::from)?;

    // 2. Pinned projects
    for p in &data.pinned_projects {
        tx.execute(
            "INSERT OR REPLACE INTO pinned_projects (project_key, repo_name, relative_path, pinned_at, is_active, display_order, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                p.project_key,
                p.repo_name,
                p.relative_path,
                p.pinned_at,
                if p.is_active { 1 } else { 0 },
                p.display_order,
                p.notes,
            ],
        )
        .map_err(AppError::from)?;
    }

    // 3. Conversations
    for c in &data.conversations {
        tx.execute(
            "INSERT OR REPLACE INTO conversations (conversation_id, project_key, seq_id, title, relative_path, last_active_at, message_count, is_archived, has_active_prompt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                c.conversation_id,
                c.project_key,
                c.seq_id,
                c.title,
                c.relative_path,
                c.last_active_at,
                c.message_count as i64,
                if c.is_archived { 1 } else { 0 },
                if c.has_active_prompt { 1 } else { 0 },
            ],
        )
        .map_err(AppError::from)?;
    }

    // 4. Running prompts
    for r in &data.running_prompts {
        tx.execute(
            "INSERT OR REPLACE INTO running_prompts (prompt_id, project_key, conversation_id, prompt_content, model, status, created_at, updated_at, image_payload, is_restored)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                r.prompt_id,
                r.project_key,
                r.conversation_id,
                r.prompt_content,
                r.model,
                r.status,
                r.created_at,
                r.updated_at,
                r.image_payload,
                if r.is_restored { 1 } else { 0 },
            ],
        )
        .map_err(AppError::from)?;
    }

    // 5. Prompt history
    for h in &data.prompt_history {
        tx.execute(
            "INSERT OR REPLACE INTO prompt_history (history_id, project_key, conversation_id, prompt_content, model, role, executed_at, exit_code, has_completed, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                h.history_id,
                h.project_key,
                h.conversation_id,
                h.prompt_content,
                h.model,
                h.role,
                h.executed_at,
                h.exit_code,
                if h.has_completed { 1 } else { 0 },
                h.duration_ms as i64,
            ],
        )
        .map_err(AppError::from)?;
    }

    tx.commit().map_err(AppError::from)?;

    // Vacuum database to reclaim unused disk pages
    // Justification: VACUUM is best-effort post-commit file compaction; failure is benign
    crate::error::record_ignored(conn.execute("VACUUM;", []), "vacuum compact db");

    Ok(())
}

/// Reads a standalone compact SQLite database file into `CompactExportData`.
pub fn read_compact_db(path: &Path) -> Result<CompactExportData, AppError> {
    if !path.exists() {
        return Err(AppError::Config(format!(
            "Compact database file not found: {path:?}"
        )));
    }

    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| AppError::Config(format!("Failed to open compact DB {path:?}: {e}")))?;

    // 1. Meta
    let meta = conn
        .query_row(
            "SELECT schema_version, created_at, source_machine, source_work_dir, notes, is_portable FROM meta ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok(CompactMetadata {
                    schema_version: row.get(0)?,
                    created_at: row.get(1)?,
                    source_machine: row.get(2)?,
                    source_work_dir: row.get(3)?,
                    notes: row.get(4)?,
                    is_portable: row.get::<_, i32>(5)? != 0,
                })
            },
        )
        .unwrap_or_else(|_| CompactMetadata {
            schema_version: "2.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            source_machine: "unknown".to_string(),
            source_work_dir: ".".to_string(),
            is_portable: true,
            notes: None,
        });

    // 2. Pinned projects
    let mut pinned_projects = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT project_key, repo_name, relative_path, pinned_at, is_active, display_order, notes FROM pinned_projects ORDER BY display_order ASC, id ASC") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(CompactPinnedProject {
                project_key: row.get(0)?,
                repo_name: row.get(1)?,
                relative_path: row.get(2)?,
                pinned_at: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                display_order: row.get(5)?,
                notes: row.get(6)?,
            })
        }) {
            for p in rows.flatten() {
                pinned_projects.push(p);
            }
        }
    }

    // 3. Conversations
    let mut conversations = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT conversation_id, project_key, seq_id, title, relative_path, last_active_at, message_count, is_archived, has_active_prompt FROM conversations ORDER BY last_active_at DESC") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(CompactConversation {
                conversation_id: row.get(0)?,
                project_key: row.get(1)?,
                seq_id: row.get(2)?,
                title: row.get(3)?,
                relative_path: row.get(4)?,
                last_active_at: row.get(5)?,
                message_count: row.get::<_, i64>(6)? as usize,
                is_archived: row.get::<_, i32>(7)? != 0,
                has_active_prompt: row.get::<_, i32>(8)? != 0,
            })
        }) {
            for c in rows.flatten() {
                conversations.push(c);
            }
        }
    }

    // 4. Running prompts
    let mut running_prompts = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT prompt_id, project_key, conversation_id, prompt_content, model, status, created_at, updated_at, image_payload, is_restored FROM running_prompts ORDER BY created_at ASC, id ASC") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(CompactRunningPrompt {
                prompt_id: row.get(0)?,
                project_key: row.get(1)?,
                conversation_id: row.get(2)?,
                prompt_content: row.get(3)?,
                model: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                image_payload: row.get(8)?,
                is_restored: row.get::<_, i32>(9)? != 0,
            })
        }) {
            for r in rows.flatten() {
                running_prompts.push(r);
            }
        }
    }

    // 5. Prompt history
    let mut prompt_history = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT history_id, project_key, conversation_id, prompt_content, model, role, executed_at, exit_code, has_completed, duration_ms FROM prompt_history ORDER BY executed_at DESC LIMIT 500") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(CompactPromptHistory {
                history_id: row.get(0)?,
                project_key: row.get(1)?,
                conversation_id: row.get(2)?,
                prompt_content: row.get(3)?,
                model: row.get(4)?,
                role: row.get(5)?,
                executed_at: row.get(6)?,
                exit_code: row.get(7)?,
                has_completed: row.get::<_, i32>(8)? != 0,
                duration_ms: row.get::<_, i64>(9)? as u64,
            })
        }) {
            for h in rows.flatten() {
                prompt_history.push(h);
            }
        }
    }

    Ok(CompactExportData {
        meta,
        pinned_projects,
        conversations,
        running_prompts,
        prompt_history,
    })
}
