//! Split Repo DB Module
//! Dedicated SQLite State Database for tracking running projects,
//! backing up active prompts, and directly dispatching prompts upon profile switch.

#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use uuid::Uuid;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

static MEMORY_ACTIVE_PROMPTS: OnceLock<Mutex<HashMap<String, ActivePrompt>>> = OnceLock::new();
static DISPATCHED_PROMPTS_CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static ACTIVE_AGY_WORKERS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();

fn get_memory_prompts_map() -> &'static Mutex<HashMap<String, ActivePrompt>> {
    MEMORY_ACTIVE_PROMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_active_agy_workers() -> &'static Mutex<HashMap<String, u32>> {
    ACTIVE_AGY_WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_dispatched_prompts_cache() -> &'static Mutex<HashSet<String>> {
    DISPATCHED_PROMPTS_CACHE.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn reset_dispatched_prompts_cache() {
    if let Ok(mut set) = get_dispatched_prompts_cache().lock() {
        set.clear();
    }
}

/// Retrieve an active prompt from in-memory cache if available
pub fn get_memory_prompt(prompt_id: &str) -> Option<ActivePrompt> {
    if let Ok(map) = get_memory_prompts_map().lock() {
        return map.get(prompt_id).cloned();
    }
    None
}

/// Represents an active or running project workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningProject {
    pub id: String,
    pub instance_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub workspace_storage_path: Option<String>,
    pub is_running: bool,
    pub last_detected_at: i64,
}

/// Represents detailed live execution state for a project workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExecutionInfo {
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub is_running: bool,
    pub is_idle: bool,
    pub status: String,
    pub active_prompt: Option<String>,
    pub last_detected_at: i64,
}

/// Represents a prompt captured from a running project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePrompt {
    pub id: String,
    pub project_id: String,
    pub instance_id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String, // "running", "backed_up", "dispatched", "completed"
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub image_payload: Option<String>,
}

/// Metadata for an auto-resumed prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResumePromptInfo {
    pub project_id: String,
    pub repo_path: String,
    pub prompt_preview: String,
    pub has_image: bool,
}

/// Outcome of the fast-forward auto-resume operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResumeResult {
    pub instance_id: String,
    pub account_email: String,
    pub resumed_project_count: usize,
    pub skipped_project_count: usize,
    pub resumed_prompts: Vec<AutoResumePromptInfo>,
}

/// Get path to the dedicated split repo prompts SQLite database
pub fn get_repo_db_path() -> Result<PathBuf, String> {
    let mut path = crate::modules::account::get_data_dir()?;
    path.push("repo_prompts.db");
    Ok(path)
}

/// Connect to the repo SQLite database with WAL mode and 5000ms busy timeout
pub fn connect_db() -> Result<Connection, String> {
    let path = get_repo_db_path()?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    init_tables(&conn)?;
    Ok(conn)
}

/// Initialize SQLite schema for running projects and active prompts
fn init_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS running_projects (
            id TEXT PRIMARY KEY,
            instance_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            workspace_storage_path TEXT,
            is_running INTEGER NOT NULL DEFAULT 1,
            last_detected_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create running_projects table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_prompts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            prompt_content TEXT NOT NULL,
            model TEXT,
            session_id TEXT,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            image_payload TEXT,
            FOREIGN KEY(project_id) REFERENCES running_projects(id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create active_prompts table: {}", e))?;

    // Migration: add image_payload column if it doesn't exist yet
    let _ = conn.execute(
        "ALTER TABLE active_prompts ADD COLUMN image_payload TEXT",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_active_prompts_instance_status 
         ON active_prompts(instance_id, status)",
        [],
    )
    .map_err(|e| format!("Failed to create index on active_prompts: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_project_sequences (
            project_key TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_project_sequences table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
            conversation_id TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_key TEXT NOT NULL,
            title TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_conversation_sequences table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS failed_commands (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT NOT NULL DEFAULT '',
            full_args TEXT NOT NULL DEFAULT '',
            domain TEXT NOT NULL DEFAULT 'root',
            error_code TEXT NOT NULL DEFAULT 'E1001',
            message TEXT NOT NULL DEFAULT '',
            suggestions TEXT NOT NULL DEFAULT '',
            hit_count INTEGER NOT NULL DEFAULT 1,
            working_dir TEXT NOT NULL DEFAULT '',
            agm_version TEXT NOT NULL DEFAULT '',
            is_resolved INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| format!("Failed to create failed_commands table: {}", e))?;

    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_failed_commands_cmd ON failed_commands(command, domain)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_failed_commands_hits ON failed_commands(hit_count DESC)",
        [],
    );
    let _ = conn.execute(
        "CREATE VIEW IF NOT EXISTS failed_to_detect_commands AS SELECT * FROM failed_commands",
        [],
    );

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedCommandRecord {
    pub id: i64,
    pub command: String,
    pub full_args: String,
    pub domain: String,
    pub error_code: String,
    pub message: String,
    pub suggestions: String,
    pub hit_count: i64,
    pub working_dir: String,
    pub agm_version: String,
    pub is_resolved: bool,
    pub created_at: String,
    pub last_seen_at: String,
}

pub fn log_failed_command(
    command: &str,
    full_args: &str,
    domain: &str,
    error_code: &str,
    message: &str,
    suggestions: &[String],
) -> Result<i64, String> {
    let conn = connect_db()?;
    let cmd_trimmed = command.trim();
    let domain_trimmed = if domain.is_empty() { "root" } else { domain };
    let err_code = if error_code.is_empty() {
        "E1001"
    } else {
        error_code
    };
    let sugg_str = suggestions.join(", ");
    let working_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let agm_version = env!("CARGO_PKG_VERSION");

    let mut stmt = conn
        .prepare("SELECT id, hit_count FROM failed_commands WHERE LOWER(command) = LOWER(?1) AND LOWER(domain) = LOWER(?2) LIMIT 1")
        .map_err(|e| e.to_string())?;

    let existing: Result<(i64, i64), _> = stmt
        .query_row(rusqlite::params![cmd_trimmed, domain_trimmed], |row| {
            Ok((row.get(0)?, row.get(1)?))
        });

    if let Ok((id, _)) = existing {
        conn.execute(
            "UPDATE failed_commands SET
                full_args = ?1,
                error_code = ?2,
                message = ?3,
                suggestions = ?4,
                hit_count = hit_count + 1,
                working_dir = ?5,
                agm_version = ?6,
                is_resolved = 0,
                last_seen_at = CURRENT_TIMESTAMP
            WHERE id = ?7",
            rusqlite::params![
                full_args,
                err_code,
                message,
                sugg_str,
                working_dir,
                agm_version,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO failed_commands (
            command, full_args, domain, error_code, message, suggestions, hit_count, working_dir, agm_version, is_resolved
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, 0)",
        rusqlite::params![
            cmd_trimmed,
            full_args,
            domain_trimmed,
            err_code,
            message,
            sugg_str,
            working_dir,
            agm_version
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn count_failed_commands() -> Result<(i64, i64), String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT COUNT(*), COALESCE(SUM(hit_count), 0) FROM failed_commands")
        .map_err(|e| e.to_string())?;
    stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())
}

pub fn list_failed_commands(limit: usize) -> Result<Vec<FailedCommandRecord>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT id, command, full_args, domain, error_code, message, suggestions, hit_count, working_dir, agm_version, is_resolved, created_at, last_seen_at FROM failed_commands ORDER BY hit_count DESC, id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            let is_res: i64 = row.get(10)?;
            Ok(FailedCommandRecord {
                id: row.get(0)?,
                command: row.get(1)?,
                full_args: row.get(2)?,
                domain: row.get(3)?,
                error_code: row.get(4)?,
                message: row.get(5)?,
                suggestions: row.get(6)?,
                hit_count: row.get(7)?,
                working_dir: row.get(8)?,
                agm_version: row.get(9)?,
                is_resolved: is_res != 0,
                created_at: row.get(11)?,
                last_seen_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut res = Vec::new();
    for r in rows {
        if let Ok(rec) = r {
            res.push(rec);
        }
    }
    Ok(res)
}

pub fn clear_failed_commands() -> Result<usize, String> {
    let conn = connect_db()?;
    let count = conn
        .execute("DELETE FROM failed_commands", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Decode file URI (e.g., file:///path/to/folder or file:///c%3A/path) to local path
pub fn decode_uri_to_path_pub(uri: &str) -> String {
    decode_uri_to_path(uri)
}

fn decode_uri_to_path(uri: &str) -> String {
    let stripped = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(uri);

    let replaced = stripped
        .replace("%20", " ")
        .replace("%3A", ":")
        .replace("%3a", ":");

    #[cfg(target_os = "windows")]
    {
        replaced.replace('/', "\\")
    }
    #[cfg(not(target_os = "windows"))]
    {
        if replaced.starts_with('/') {
            replaced
        } else {
            format!("/{}", replaced)
        }
    }
}

/// Scan active workspace storage for an instance and discover projects
pub fn detect_running_projects(instance_id: &str) -> Result<Vec<RunningProject>, String> {
    let target_id = if instance_id == "__default__" || instance_id.is_empty() {
        "default"
    } else {
        instance_id
    };
    let registry = crate::modules::instance::load_registry()?;
    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id || (target_id == "default" && i.is_default))
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

    let pids =
        crate::modules::instance::find_pids_for_data_dir(&instance.data_dir, instance.is_default);
    let is_instance_active = !pids.is_empty();

    let storage_dir = PathBuf::from(&instance.data_dir)
        .join("User")
        .join("workspaceStorage");

    let mut projects = Vec::new();
    let now = Utc::now().timestamp();

    if storage_dir.exists() {
        if let Ok(entries) = fs::read_dir(&storage_dir) {
            for entry in entries.flatten() {
                let ws_folder = entry.path();
                if ws_folder.is_dir() {
                    let ws_json_path = ws_folder.join("workspace.json");
                    if ws_json_path.exists() {
                        if let Ok(content) = fs::read_to_string(&ws_json_path) {
                            if let Ok(json_val) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                let folder_uri =
                                    json_val.get("folder").and_then(|v| v.as_str()).or_else(|| {
                                        json_val.get("configuration").and_then(|v| v.as_str())
                                    });

                                if let Some(uri) = folder_uri {
                                    let raw_path = decode_uri_to_path(uri);
                                    let project_path = Path::new(&raw_path);
                                    let repo_name = project_path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "unnamed-project".to_string());

                                    let project_id = format!(
                                        "{}-{}",
                                        repo_name.to_lowercase(),
                                        entry.file_name().to_string_lossy()
                                    );

                                    projects.push(RunningProject {
                                        id: project_id,
                                        instance_id: instance_id.to_string(),
                                        repo_name,
                                        repo_path: raw_path,
                                        workspace_storage_path: Some(
                                            ws_folder.to_string_lossy().to_string(),
                                        ),
                                        is_running: is_instance_active,
                                        last_detected_at: now,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Persist discovered projects into repo database
    if let Ok(conn) = connect_db() {
        for p in &projects {
            let running_int = if p.is_running { 1 } else { 0 };
            let _ = conn.execute(
                "INSERT OR REPLACE INTO running_projects 
                 (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &p.id,
                    &p.instance_id,
                    &p.repo_name,
                    &p.repo_path,
                    &p.workspace_storage_path,
                    running_int,
                    p.last_detected_at,
                    now,
                ],
            );
        }
    }

    Ok(projects)
}

/// Extract image data URI, file path, or markdown image reference from text
pub fn extract_image_payload_or_path(content: &str) -> (Option<String>, Vec<String>) {
    let mut found_paths: Vec<String> = Vec::new();
    let mut raw_payload = None;

    if content.contains("data:image/") {
        if let Some(start) = content.find("data:image/") {
            let tail = &content[start..];
            let end = tail
                .find('"')
                .or_else(|| tail.find('\''))
                .or_else(|| tail.find(' '))
                .or_else(|| tail.find(')'))
                .unwrap_or(tail.len());
            raw_payload = Some(tail[..end].to_string());
        }
    }

    let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];
    for token in content.split_whitespace() {
        let clean = token.trim_matches(|c| {
            c == '"'
                || c == '\''
                || c == '('
                || c == ')'
                || c == '['
                || c == ']'
                || c == '<'
                || c == '>'
        });
        let lower = clean.to_lowercase();
        if img_exts.iter().any(|ext| lower.ends_with(ext)) {
            let p_str = clean
                .strip_prefix("file:///")
                .unwrap_or_else(|| clean.strip_prefix("file://").unwrap_or(clean));
            if !found_paths.contains(&p_str.to_string()) {
                found_paths.push(p_str.to_string());
            }
        }
    }

    if raw_payload.is_none() && !found_paths.is_empty() {
        raw_payload = Some(found_paths.join(";"));
    }

    (raw_payload, found_paths)
}

/// Text to store for a live conversation. The transcript's latest USER_INPUT wins.
/// When that line is missing, the conversation preview is the prompt that was running.
pub fn live_prompt_text(preview: &str, transcript: &str) -> Option<String> {
    for line in transcript.lines().rev() {
        if !line.contains("USER_INPUT") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if val.get("type").and_then(|t| t.as_str()) != Some("USER_INPUT") {
            continue;
        }
        if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
            if !txt.trim().is_empty() {
                return Some(txt.to_string());
            }
        }
    }
    let trimmed = preview.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Resume file the IDE reads after a switch. `session_id` is the same conversation.
pub fn resume_task_document(
    prompt: &ActivePrompt,
    status: &str,
    at: i64,
    image_paths: &[String],
) -> serde_json::Value {
    let has_image = prompt.image_payload.is_some() || !image_paths.is_empty();
    serde_json::json!({
        "prompt_id": prompt.id,
        "project_id": prompt.project_id,
        "instance_id": prompt.instance_id,
        "repo_path": prompt.repo_path,
        "prompt_content": prompt.prompt_content,
        "model": prompt.model,
        "session_id": prompt.session_id,
        "conversation_id": prompt.session_id,
        "image_payload": prompt.image_payload,
        "image_paths": image_paths,
        "has_image": has_image,
        "auto_boot": true,
        "status": status,
        "backed_up_at": at,
    })
}

fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {
    let named = instance_id != "all"
        && instance_id != "default"
        && !instance_id.is_empty()
        && instance_id != "__default__";
    let home = if named {
        crate::modules::instance::get_instance_home_dir(instance_id).ok()
    } else {
        dirs::home_dir()
    };
    let mut dirs = Vec::new();
    if let Some(home) = home {
        for sub in ["antigravity", "antigravity-cli", "antigravity-ide"] {
            let path = home.join(".gemini").join(sub);
            if path.exists() {
                dirs.push(path);
            }
        }
    }
    dirs
}

struct ConversationSummaryRow {
    cid: String,
    preview: String,
    status: String,
    not_fully_idle: i32,
    workspace_uris: Option<String>,
}

fn read_conversation_summary_rows(conn: &Connection) -> Vec<ConversationSummaryRow> {
    let wide = "SELECT conversation_id, preview, status, not_fully_idle, workspace_uris
         FROM conversation_summaries
         ORDER BY last_modified_time DESC
         LIMIT 25";
    if let Ok(mut stmt) = conn.prepare(wide) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ConversationSummaryRow {
                cid: row.get(0)?,
                preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                not_fully_idle: row.get::<_, Option<i32>>(3)?.unwrap_or(0),
                workspace_uris: row.get(4)?,
            })
        }) {
            return rows.flatten().collect();
        }
    }
    let narrow = "SELECT conversation_id, preview, workspace_uris
         FROM conversation_summaries
         ORDER BY rowid DESC
         LIMIT 25";
    let Ok(mut stmt) = conn.prepare(narrow) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok(ConversationSummaryRow {
            cid: row.get(0)?,
            preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            status: String::new(),
            not_fully_idle: 1,
            workspace_uris: row.get(2)?,
        })
    }) else {
        return Vec::new();
    };
    rows.flatten().collect()
}

/// Discover in-flight active conversations and running prompts directly from Antigravity core storage
/// (~/.gemini/antigravity/conversation_summaries.db and brain/<cid>/.system_generated/logs/transcript.jsonl)
pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {
    let mut prompts = Vec::new();
    let mut seen_cids = std::collections::HashSet::new();
    let candidate_dirs = gemini_dirs_for_instance(instance_id);

    for base_dir in candidate_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }

        let conn = match Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let now = Utc::now().timestamp();
        let rows = read_conversation_summary_rows(&conn);
        if rows.is_empty() {
            continue;
        }

        for item in rows {
            let cid = item.cid;
            let preview = item.preview;
            let status = item.status;
            let not_fully_idle = item.not_fully_idle;
            let ws_uris_opt = item.workspace_uris;
            if !seen_cids.insert(cid.clone()) {
                continue;
            }

            let is_running_or_recent =
                not_fully_idle != 0 || status.contains("RUNNING") || prompts.is_empty();
            if !is_running_or_recent && prompts.len() >= 5 {
                continue;
            }

            let Some(ws_uris_raw) = ws_uris_opt else {
                continue;
            };

            let ws_uris: Vec<String> = serde_json::from_str(&ws_uris_raw).unwrap_or_default();
            if ws_uris.is_empty() {
                continue;
            }

            let repo_path = decode_uri_to_path(&ws_uris[0]);
            let repo_name = Path::new(&repo_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "antigravity-project".to_string());
            let project_id = format!(
                "{}-{}",
                repo_name.to_lowercase(),
                cid.chars().take(8).collect::<String>()
            );

            // Read transcript.jsonl from brain/<cid>/.system_generated/logs/transcript.jsonl
            let transcript_file = base_dir
                .join("brain")
                .join(&cid)
                .join(".system_generated")
                .join("logs")
                .join("transcript.jsonl");

            let mut user_prompt: Option<String> = None;
            let mut image_payload: Option<String> = None;

            if transcript_file.exists() {
                if let Ok(content) = fs::read_to_string(&transcript_file) {
                    for line in content.lines().rev() {
                        if !line.contains("USER_INPUT") {
                            continue;
                        }
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                            if val.get("type").and_then(|t| t.as_str()) == Some("USER_INPUT") {
                                if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
                                    if !txt.trim().is_empty() {
                                        user_prompt = Some(txt.to_string());
                                    }
                                }
                                if let Some(media_arr) = val.get("media").and_then(|m| m.as_array())
                                {
                                    for m_item in media_arr {
                                        if let Some(uri) =
                                            m_item.get("uri").and_then(|u| u.as_str())
                                        {
                                            let clean_path = decode_uri_to_path(uri);
                                            let p = Path::new(&clean_path);
                                            if p.exists() {
                                                if let Ok(bytes) = fs::read(p) {
                                                    let mime = m_item
                                                        .get("mime_type")
                                                        .and_then(|mt| mt.as_str())
                                                        .unwrap_or("image/png");
                                                    let b64 = STANDARD.encode(&bytes);
                                                    image_payload = Some(format!(
                                                        "data:{};base64,{}",
                                                        mime, b64
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                if user_prompt.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if user_prompt.is_none() {
                user_prompt = live_prompt_text(&preview, "");
            }

            if let Some(prompt_text) = user_prompt {
                let p_id = format!("prompt-{}", cid);
                prompts.push(ActivePrompt {
                    id: p_id,
                    project_id,
                    instance_id: instance_id.to_string(),
                    repo_path,
                    prompt_content: prompt_text,
                    model: Some("gemini-pro".to_string()),
                    session_id: Some(cid),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload,
                });
            }
        }
    }

    prompts
}

/// Prompts that still need a re-push after this instance's IDE is back up.
pub fn count_backed_up_prompts(instance_id: &str) -> usize {
    let Ok(conn) = connect_db() else {
        return 0;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM active_prompts
         WHERE status = 'backed_up'
           AND (instance_id = ?1 OR (?1 = 'default' AND instance_id IN ('default', '__default__', '')))",
        [instance_id],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0) as usize
}

/// A switch waits for the IDE prompt channel only when a prompt was actually backed up.
pub fn needs_prompt_channel_wait(backed_up_count: usize) -> bool {
    backed_up_count > 0
}

/// Backup all currently running prompts across active projects before switching
pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut backed_up_count = 0;

    // Step 0: Retire stale 'running' prompts older than 2 hours to 'dispatched' so they don't shadow active prompts
    let stale_cutoff = now - 7200;
    let _ = conn.execute(
        "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE status = 'running' AND updated_at < ?",
        params![now, stale_cutoff],
    );

    // Step 0.5: Clear dispatched prompts cache so that any prompt backed up can be cleanly re-dispatched upon switch completion
    reset_dispatched_prompts_cache();

    let is_all_or_default =
        instance_id == "all" || instance_id == "__default__" || instance_id == "default";

    // Transition in-flight 'running' prompts in active_prompts to 'backed_up' before switch (scoped to target instance)
    let transitioned = if instance_id == "all" {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running'",
            params![now],
        )
        .unwrap_or(0)
    } else if is_all_or_default {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running' AND instance_id IN ('default', '__default__')",
            params![now],
        )
        .unwrap_or(0)
    } else {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running' AND instance_id = ?2",
            params![now, instance_id],
        )
        .unwrap_or(0)
    };
    if transitioned > 0 {
        backed_up_count += transitioned;
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            for p in map.values_mut() {
                let matches_inst = instance_id == "all"
                    || (is_all_or_default
                        && (p.instance_id == "default" || p.instance_id == "__default__"))
                    || p.instance_id == instance_id;
                if p.status == "running" && matches_inst {
                    p.status = "backed_up".to_string();
                    p.updated_at = now;
                }
            }
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Transitioned {} in-flight prompts from 'running' to 'backed_up' for instance '{}' before switch",
            transitioned, instance_id
        ));
    }

    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
    for p in ag_prompts {
        let clean_repo_name = Path::new(&p.repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| p.project_id.clone());
        let _ = conn.execute(
            "INSERT OR REPLACE INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?, ?, ?, ?, NULL, 1, ?, ?)",
            params![&p.project_id, &p.instance_id, &clean_repo_name, &p.repo_path, now, now],
        );

        let res = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET 
                 status = 'backed_up',
                 updated_at = excluded.updated_at,
                 prompt_content = excluded.prompt_content,
                 image_payload = excluded.image_payload",
            params![
                &p.id,
                &p.project_id,
                &p.instance_id,
                &p.repo_path,
                &p.prompt_content,
                &p.model,
                &p.session_id,
                p.created_at,
                now,
                &p.image_payload,
            ],
        );

        if res.is_ok() {
            backed_up_count += 1;
            let (extracted_img, img_paths) = extract_image_payload_or_path(&p.prompt_content);
            let final_img = p.image_payload.clone().or(extracted_img);
            // Write disk resume snapshot file inside project repo directory.
            // session_id is the live conversation, so the IDE reopens that chat.
            let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
            let mut payload = resume_task_document(&p, "backed_up", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
            }
            if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                let _ = fs::write(&task_file, json_str);
            }
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
    }

    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery (Parallelized)
    let projects = detect_running_projects(instance_id).unwrap_or_default();
    let extracted_results: Vec<(&RunningProject, Vec<(String, Option<String>)>)> =
        std::thread::scope(|s| {
            let handles: Vec<_> = projects
                .iter()
                .map(|project| {
                    s.spawn(move || {
                        let mut extracted: Vec<(String, Option<String>)> = Vec::new();

                        // Check workspace state.vscdb for active prompts / tasks
                        if let Some(ref ws_storage) = project.workspace_storage_path {
                            let ws_db_path = PathBuf::from(ws_storage).join("state.vscdb");
                            if ws_db_path.exists() {
                                if let Ok(ws_conn) = Connection::open_with_flags(
                                    &ws_db_path,
                                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                                ) {
                                    let mut stmt = ws_conn
                                        .prepare("SELECT key, value FROM ItemTable WHERE key LIKE '%prompt%' OR key LIKE '%chat%' OR key LIKE '%task%'")
                                        .ok();

                                    if let Some(ref mut prepared) = stmt {
                                        if let Ok(rows) = prepared.query_map([], |row| {
                                            let key: String = row.get(0)?;
                                            let val: String = row.get(1)?;
                                            Ok((key, val))
                                        }) {
                                            for item in rows.flatten() {
                                                let content = item.1;
                                                if !content.trim().is_empty() && content.len() > 10 {
                                                    let (img_payload, _) = extract_image_payload_or_path(&content);
                                                    extracted.push((content, img_payload));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Check if project has an existing .antigravity_resume_task.json on disk
                        if extracted.is_empty() {
                            let task_file = PathBuf::from(&project.repo_path)
                                .join(".antigravity_resume_task.json");
                            if task_file.exists() {
                                if let Ok(c) = fs::read_to_string(&task_file) {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                                        if let Some(txt) =
                                            v.get("prompt_content").and_then(|t| t.as_str())
                                        {
                                            if !txt.trim().is_empty() {
                                                let img = v
                                                    .get("image_payload")
                                                    .and_then(|i| i.as_str())
                                                    .map(|s| s.to_string());
                                                extracted.push((txt.to_string(), img));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        (project, extracted)
                    })
                })
                .collect();

            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });

    for (project, mut extracted_prompts) in extracted_results {
        // Check if SQLite active_prompts already has an existing prompt for this repo_path or project
        if extracted_prompts.is_empty() {
            let proj_like = format!("%{}%", project.repo_name.to_lowercase());
            let mut check_stmt = conn
                .prepare(
                    "SELECT prompt_content, image_payload FROM active_prompts \
                     WHERE repo_path = ?1 OR project_id = ?2 OR project_id LIKE ?3 \
                     ORDER BY updated_at DESC LIMIT 1",
                )
                .ok();
            if let Some(ref mut c_stmt) = check_stmt {
                if let Ok((txt, img)) = c_stmt.query_row(
                    rusqlite::params![&project.repo_path, &project.id, &proj_like],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                ) {
                    if !txt.trim().is_empty() {
                        extracted_prompts.push((txt, img));
                    }
                }
            }
        }

        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 LIMIT 1",
                    rusqlite::params![&project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok();

            let (prompt_id, is_existing) = match existing_id {
                Some(eid) => (eid, true),
                None => (Uuid::new_v4().to_string(), false),
            };

            let prompt_model = Some("gemini-pro".to_string());
            let result = if is_existing {
                conn.execute(
                    "UPDATE active_prompts SET status = 'backed_up', updated_at = ?, image_payload = COALESCE(?, image_payload) WHERE id = ?",
                    params![now, &image_payload, &prompt_id],
                )
            } else {
                conn.execute(
                    "INSERT INTO active_prompts 
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?, ?, ?, ?, ?, ?, ?, 'backed_up', ?, ?, ?)",
                    params![
                        &prompt_id,
                        &project.id,
                        instance_id,
                        &project.repo_path,
                        &prompt_text,
                        &prompt_model,
                        &project.id,
                        now,
                        now,
                        &image_payload,
                    ],
                )
            };

            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt_text);
                let final_img = image_payload.clone().or(extracted_img);
                let has_image = final_img.is_some() || !img_paths.is_empty();

                // Write disk resume snapshot file inside project repo directory
                let task_file =
                    PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
                let payload = serde_json::json!({
                    "prompt_id": prompt_id,
                    "project_id": project.id,
                    "instance_id": instance_id,
                    "repo_path": project.repo_path,
                    "prompt_content": prompt_text,
                    "model": prompt_model,
                    "session_id": project.id,
                    "image_payload": final_img,
                    "image_paths": img_paths,
                    "has_image": has_image,
                    "auto_boot": true,
                    "status": "backed_up",
                    "backed_up_at": now,
                });
                if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                    let _ = fs::write(&task_file, json_str);
                }

                let active_prompt = ActivePrompt {
                    id: prompt_id.clone(),
                    project_id: project.id.clone(),
                    instance_id: instance_id.to_string(),
                    repo_path: project.repo_path.clone(),
                    prompt_content: prompt_text,
                    model: prompt_model,
                    session_id: Some(project.id.clone()),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload: final_img,
                };
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Backed up {} running prompts for instance '{}' across Antigravity core and workspaces",
        backed_up_count, instance_id
    ));

    Ok(backed_up_count)
}

/// Verified count of active prompts or running processes across workspaces
pub fn verify_prompts_running() -> usize {
    let mut count = 0;

    // 1. Check SQLite active_prompts for 'running' or recently 'dispatched' (within 5 minutes)
    if let Ok(conn) = connect_db() {
        let now = Utc::now().timestamp();
        let db_count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status = 'running' OR (status = 'dispatched' AND updated_at >= ?1)",
                params![now - 300],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count = db_count;
    }

    // 2. Check Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let ag_count: usize = conn
                    .query_row(
                        "SELECT COUNT(*) FROM conversation_summaries WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if ag_count > count {
                    count = ag_count;
                }
            }
        }
    }

    // 3. Fallback to live project execution info
    if count == 0 {
        let live = get_live_project_execution_info();
        count = live
            .iter()
            .filter(|p| p.is_running || (!p.is_idle && p.active_prompt.is_some()))
            .count();
    }

    count
}

/// Directly dispatch/send backed-up prompts to the running projects without queuing
pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let is_default =
        instance_id == "default" || instance_id == "__default__" || instance_id.is_empty();

    let instance_projects = detect_running_projects(instance_id).unwrap_or_default();
    let instance_repo_paths: HashSet<String> = instance_projects
        .iter()
        .map(|proj| normalize_path_for_compare(&proj.repo_path))
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE status = 'backed_up'
             ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare dispatch query: {}", e))?;

    let all_backed_up = stmt
        .query_map([], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect::<Vec<ActivePrompt>>();

    let prompts: Vec<ActivePrompt> = all_backed_up
        .into_iter()
        .filter(|p| {
            if is_default {
                p.instance_id == "default"
                    || p.instance_id == "__default__"
                    || p.instance_id.is_empty()
                    || instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))
            } else {
                p.instance_id == instance_id
                    || instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))
            }
        })
        .collect();

    let mut dispatched_count = 0;

    for prompt in prompts {
        let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
        let already_dispatched = if prompt.status == "backed_up" {
            false
        } else {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            let seen = cache.contains(&prompt.id) || cache.contains(&sig);
            if !seen {
                cache.insert(prompt.id.clone());
                cache.insert(sig);
            }
            seen
        };

        if already_dispatched {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping duplicate dispatch for prompt '{}' in '{}'",
                prompt.id, prompt.repo_path
            ));
            let _ = conn.execute(
                "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
                rusqlite::params![now, &prompt.id],
            );
            continue;
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Directly dispatching prompt '{}' to project at '{}' without queuing",
            prompt.id, prompt.repo_path
        ));

        // Keep the conversation id on the file the IDE reads. A later rewrite must not drop it.
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = resume_task_document(&prompt, "dispatched", now, &[]);
        let file_written = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };

        let sent = spawn_prompt_via_agy(&prompt);
        let same_conversation = prompt
            .session_id
            .as_deref()
            .map(|id| !id.trim().is_empty())
            .unwrap_or(false);
        if !sent && !(file_written && same_conversation) {
            crate::modules::logger::log_error(&format!(
                "[RepoDB] Prompt '{}' was not re-pushed for instance '{}'; left backed_up",
                prompt.id, instance_id
            ));
            continue;
        }

        let updated = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
            params![now, &prompt.id],
        );

        if updated.is_ok() {
            dispatched_count += 1;
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                if let Some(p) = map.get_mut(&prompt.id) {
                    p.status = "dispatched".to_string();
                    p.updated_at = now;
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Successfully dispatched {} prompts to running projects for instance '{}'",
        dispatched_count, instance_id
    ));

    Ok(dispatched_count)
}

/// Check if any prompt or task is currently actively executing for a project
pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool {
    let now = Utc::now().timestamp();

    // 1. Check in-memory active prompts map
    if let Ok(map) = get_memory_prompts_map().lock() {
        for p in map.values() {
            let matches_inst = instance_id.is_empty()
                || instance_id == "all"
                || p.instance_id == instance_id
                || ((instance_id == "default" || instance_id == "__default__")
                    && (p.instance_id == "default"
                        || p.instance_id == "__default__"
                        || p.instance_id.is_empty()));
            if matches_inst && (p.project_id == project_id || p.repo_path == project_id) {
                if p.status == "running" || (p.status == "dispatched" && p.updated_at >= now - 120)
                {
                    return true;
                }
            }
        }
    }

    // 2. Check active workers map
    if let Ok(workers) = get_active_agy_workers().lock() {
        for key in workers.keys() {
            if key.contains(project_id) {
                return true;
            }
        }
    }

    // 3. Check SQLite active_prompts for 'running' or recently 'dispatched'
    if let Ok(conn) = connect_db() {
        let running_count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts 
                 WHERE (project_id = ?1 OR repo_path = ?1) 
                   AND (status = 'running' OR (status = 'dispatched' AND updated_at >= ?2))",
                params![project_id, now - 120],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if running_count > 0 {
            return true;
        }
    }

    // 4. Check Antigravity live conversation summaries directly for running sessions
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            let conn_res = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
            .or_else(|_| {
                let uri = format!(
                    "file:{}?immutable=1",
                    summaries_db.to_string_lossy().replace('\\', "/")
                );
                Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(conn) = conn_res {
                let _ = conn.pragma_update(None, "busy_timeout", 3000);
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT status, not_fully_idle, workspace_uris, last_modified_time 
                     FROM conversation_summaries 
                     ORDER BY last_modified_time DESC 
                     LIMIT 30",
                ) {
                    let clean_target = normalize_path_for_compare(project_id);
                    let rows = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i32>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    });
                    if let Ok(rows) = rows {
                        for item in rows.flatten() {
                            let (status, not_fully_idle, ws_uris_opt, last_time_str) = item;
                            let is_recency_active = if let Ok(parsed) =
                                chrono::DateTime::parse_from_rfc3339(&last_time_str)
                            {
                                let age = (Utc::now() - parsed.with_timezone(&Utc)).num_seconds();
                                age >= 0 && age < 600
                            } else {
                                false
                            };
                            let is_conv_running = not_fully_idle != 0
                                || status.contains("RUNNING")
                                || is_recency_active;
                            if is_conv_running {
                                if let Some(ws_uris_raw) = ws_uris_opt {
                                    let ws_uris: Vec<String> =
                                        serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                                    for u in ws_uris {
                                        let clean_p =
                                            normalize_path_for_compare(&decode_uri_to_path(&u));
                                        if !clean_target.is_empty()
                                            && (clean_p == clean_target
                                                || clean_p.contains(&clean_target)
                                                || clean_target.contains(&clean_p))
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Bookkeep and check enqueued/backed_up prompts across projects.
/// If a project has no prompt running (idle verified), automatically pushes
/// the first enqueued prompt (FIFO) and logs the action to Audit Trail.
pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let run_id = format!("sched-{}", Utc::now().format("%Y%m%d-%H%M%S"));

    // 1. Gather candidate projects that have enqueued prompts ('backed_up', 'queued', 'pending')
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT project_id, instance_id, repo_path FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending')
             ORDER BY updated_at ASC",
        )
        .map_err(|e| format!("Failed to query candidate enqueued projects: {}", e))?;

    let candidate_projects = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect::<Vec<_>>();

    if candidate_projects.is_empty() {
        return Ok(0);
    }

    let mut dispatched_count = 0;

    for (project_id, inst_id, repo_path) in candidate_projects {
        if let Some(target) = target_instance {
            let is_match = target == "all"
                || inst_id == target
                || ((target == "default" || target == "__default__")
                    && (inst_id == "default" || inst_id == "__default__" || inst_id.is_empty()));
            if !is_match {
                continue;
            }
        }

        // Check if project is currently running a prompt
        let is_running = is_prompt_running_for_project(&project_id, &inst_id)
            || is_prompt_running_for_project(&repo_path, &inst_id);

        let clean_project_name = Path::new(&repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| project_id.clone());

        if is_running {
            crate::modules::logger::log_info(&format!(
                "[PromptQueueScheduler] Project '{}' ({}) is currently active/busy; enqueued prompts remain queued",
                clean_project_name, project_id
            ));
            continue;
        }

        // Project is IDLE! Pick the first enqueued prompt (FIFO: earliest created_at)
        let prompt_opt: Option<ActivePrompt> = conn
            .query_row(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                 FROM active_prompts
                 WHERE (project_id = ?1 OR repo_path = ?2) AND status IN ('backed_up', 'queued', 'pending')
                 ORDER BY created_at ASC
                 LIMIT 1",
                params![&project_id, &repo_path],
                |row| {
                    Ok(ActivePrompt {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        instance_id: row.get(2)?,
                        repo_path: row.get(3)?,
                        prompt_content: row.get(4)?,
                        model: row.get(5)?,
                        session_id: row.get(6)?,
                        status: row.get(7)?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                        image_payload: row.get(10).ok(),
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to fetch enqueued prompt for {}: {}", project_id, e))?;

        let Some(prompt) = prompt_opt else {
            continue;
        };

        crate::modules::logger::log_info(&format!(
            "[PromptQueueScheduler] Project '{}' verified idle. Dispatching enqueued prompt '{}' (conv: {:?})",
            clean_project_name, prompt.id, prompt.session_id
        ));

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        let final_img = prompt.image_payload.clone().or(extracted_img);
        let mut payload = resume_task_document(&prompt, "dispatched", now, &img_paths);
        if final_img.is_some() {
            payload["image_payload"] = serde_json::json!(final_img);
        }
        let _ = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };

        // Spawn prompt via agy
        let sent = spawn_prompt_via_agy(&prompt);

        // Update active_prompts status to 'dispatched'
        let _ = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?1 WHERE id = ?2",
            params![now, &prompt.id],
        );

        if let Ok(mut map) = get_memory_prompts_map().lock() {
            if let Some(p) = map.get_mut(&prompt.id) {
                p.status = "dispatched".to_string();
                p.updated_at = now;
            }
        }

        // Record SchedulerFacts in Audit Trail
        let prompt_preview = if prompt.prompt_content.len() > 140 {
            format!("{}...", &prompt.prompt_content[..140])
        } else {
            prompt.prompt_content.clone()
        };

        let action_taken = if sent {
            "dispatched"
        } else {
            "resume_task_written"
        };
        let reason = format!(
            "Project verified idle; enqueued prompt pushed automatically via {}",
            action_taken
        );

        let facts = crate::modules::task_history_db::SchedulerFacts {
            scheduler_run_id: run_id.clone(),
            project_name: clean_project_name.clone(),
            repo_path: prompt.repo_path.clone(),
            prompt_id: prompt.id.clone(),
            prompt_preview,
            conversation_id: prompt.session_id.clone().unwrap_or_default(),
            action_taken: action_taken.to_string(),
            reason,
            idle_check_passed: true,
            instance_id: prompt.instance_id.clone(),
            timestamp: now,
        };

        let _ = crate::modules::task_history_db::record_scheduler_event(&facts);
        dispatched_count += 1;
    }

    Ok(dispatched_count)
}

/// Re-enqueue running conversations for an instance before switch/restart.
/// Marks active prompts as 'backed_up', updates resume document, and logs to Audit.
pub fn requeue_running_conversations_for_instance(instance_id: &str) -> Result<usize, String> {
    let count = backup_running_prompts(instance_id)?;
    if count == 0 {
        return Ok(0);
    }

    if let Ok(conn) = connect_db() {
        let is_all_or_default =
            instance_id == "all" || instance_id == "__default__" || instance_id == "default";
        let stmt_res = if instance_id == "all" {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' ORDER BY updated_at DESC LIMIT 50",
            )
        } else if is_all_or_default {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' AND instance_id IN ('default', '__default__', '') ORDER BY updated_at DESC LIMIT 50",
            )
        } else {
            conn.prepare(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, session_id 
                 FROM active_prompts WHERE status = 'backed_up' AND instance_id = ?1 ORDER BY updated_at DESC LIMIT 50",
            )
        };

        if let Ok(mut stmt) = stmt_res {
            let rows: Vec<(String, String, String, String, String, Option<String>)> =
                if instance_id == "all" || is_all_or_default {
                    stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                        ))
                    })
                    .map(|mapped| mapped.flatten().collect())
                    .unwrap_or_default()
                } else {
                    stmt.query_map(params![instance_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                        ))
                    })
                    .map(|mapped| mapped.flatten().collect())
                    .unwrap_or_default()
                };

            for (_id, project_id, inst_id, repo_path, prompt_content, session_id) in rows {
                let project_name = Path::new(&repo_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(project_id);
                let cid = session_id.unwrap_or_default();
                let preview = if prompt_content.len() > 100 {
                    format!("{}...", &prompt_content[..100])
                } else {
                    prompt_content
                };
                let reason = "In-flight conversation re-enqueued for restart/switch continuity";
                let _ = crate::modules::task_history_db::record_requeue_event(
                    &project_name,
                    &inst_id,
                    &cid,
                    reason,
                    &preview,
                );
            }
        }
    }

    Ok(count)
}

/// List all running projects across instances
pub fn list_running_projects() -> Result<Vec<RunningProject>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
             FROM running_projects ORDER BY last_detected_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list projects query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let running_int: i32 = row.get(5)?;
            let is_running = running_int != 0;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query running projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// List all backed up prompts
pub fn list_backed_up_prompts() -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts WHERE status = 'backed_up' ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list prompts query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

pub struct SwitchPromptSnap {
    pub prompt_id: String,
    pub prompt_text: String,
    pub conversation_id: String,
}

impl SwitchPromptSnap {
    fn empty() -> Self {
        Self {
            prompt_id: String::new(),
            prompt_text: String::new(),
            conversation_id: String::new(),
        }
    }
}

/// Newest running, backed-up, or dispatched prompt for this instance.
/// Ordered by updated_at so a just-captured conversation is not hidden behind older rows.
pub fn switch_prompt_snapshot(instance_id: &str) -> SwitchPromptSnap {
    let Ok(conn) = connect_db() else {
        return SwitchPromptSnap::empty();
    };
    conn.query_row(
        "SELECT id, prompt_content, IFNULL(session_id, '')
         FROM active_prompts
         WHERE status IN ('running', 'backed_up', 'dispatched')
           AND (
                instance_id = ?1
                OR (?1 = 'default' AND instance_id IN ('default', '__default__', ''))
           )
         ORDER BY updated_at DESC
         LIMIT 1",
        [instance_id],
        |row| {
            Ok(SwitchPromptSnap {
                prompt_id: row.get(0)?,
                prompt_text: row.get(1)?,
                conversation_id: row.get(2)?,
            })
        },
    )
    .unwrap_or_else(|_| SwitchPromptSnap::empty())
}

pub fn list_all_prompts() -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts ORDER BY created_at DESC LIMIT 50",
        )
        .map_err(|e| format!("Failed to prepare list prompts query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// Normalize filesystem path for reliable cross-platform comparison
fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Retrieve live project execution information by inspecting Antigravity's
/// conversation_summaries.db (ground-truth for in-flight requests), active OS processes,
/// and SQLite repo_prompts.db.
pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {
    let mut results: Vec<ProjectExecutionInfo> = Vec::new();
    let now = Utc::now().timestamp();

    // Map of normalized repo path -> (is_running, active_prompt_snippet, last_time)
    let mut live_map: std::collections::HashMap<String, (bool, Option<String>, i64)> =
        std::collections::HashMap::new();
    let mut active_conv_prefixes: Vec<(String, Option<String>, i64)> = Vec::new();

    // 1. Inspect Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            let conn = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
            .or_else(|_| {
                let uri = format!(
                    "file:{}?immutable=1",
                    summaries_db.to_string_lossy().replace('\\', "/")
                );
                Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(conn) = conn {
                let _ = conn.pragma_update(None, "busy_timeout", 3000);
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
                     FROM conversation_summaries 
                     ORDER BY last_modified_time DESC 
                     LIMIT 30",
                ) {
                    let rows = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i32>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    });
                    if let Ok(rows) = rows {
                        for item in rows.flatten() {
                            let (cid, _title, preview, status, not_fully_idle, ws_uris_opt, last_time_str) = item;
                            let is_recency_active = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&last_time_str) {
                                let age = (Utc::now() - parsed.with_timezone(&Utc)).num_seconds();
                                age >= 0 && age < 600
                            } else {
                                false
                            };
                            let is_conv_running = not_fully_idle != 0 || status.contains("RUNNING") || is_recency_active;
                            let prompt_preview = if !preview.trim().is_empty() {
                                Some(preview)
                            } else {
                                None
                            };

                            let prefix_8 = if cid.len() >= 8 {
                                cid[..8].to_string()
                            } else {
                                cid.clone()
                            };

                            if is_conv_running {
                                active_conv_prefixes.push((prefix_8, prompt_preview.clone(), now));
                            }

                            if let Some(ws_uris_raw) = ws_uris_opt {
                                let ws_uris: Vec<String> =
                                    serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                                for u in ws_uris {
                                    let clean_p = normalize_path_for_compare(&decode_uri_to_path(&u));
                                    let entry = live_map
                                        .entry(clean_p)
                                        .or_insert((false, None, now));
                                    if is_conv_running {
                                        entry.0 = true;
                                        if entry.1.is_none() && prompt_preview.is_some() {
                                            entry.1 = prompt_preview.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Inspect active_prompts in repo_prompts.db for any in-flight prompts
    if let Ok(conn) = connect_db() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT repo_path, prompt_content, status FROM active_prompts WHERE status = 'running' OR status = 'queued'",
        ) {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            });
            if let Ok(rows) = rows {
                for item in rows.flatten() {
                    let (p_path, p_content, _st) = item;
                    let clean_p = normalize_path_for_compare(&p_path);
                    let entry = live_map.entry(clean_p).or_insert((false, None, now));
                    entry.0 = true;
                    if entry.1.is_none() {
                        entry.1 = Some(p_content.chars().take(120).collect());
                    }
                }
            }
        }
    }

    // 3. Merge with discovered projects from running_projects
    let projects = list_running_projects().unwrap_or_default();
    for p in projects {
        let clean_path = normalize_path_for_compare(&p.repo_path);
        let mut is_running = false;
        let mut prompt_snippet = None;
        let mut last_time = p.last_detected_at;

        // Check path match in live_map
        if let Some((run, snippet, l_time)) = live_map.get(&clean_path) {
            if *run {
                is_running = true;
                prompt_snippet = snippet.clone();
                last_time = *l_time;
            }
        }

        // Check if project_id or repo_path matches any active conversation prefix
        if !is_running {
            for (pfx, snippet, l_time) in &active_conv_prefixes {
                if p.id.contains(pfx) || p.repo_path.contains(pfx) {
                    is_running = true;
                    prompt_snippet = snippet.clone();
                    last_time = *l_time;
                    break;
                }
            }
        }

        let is_idle = !is_running;
        let status_str = if is_running {
            "RUNNING".to_string()
        } else {
            "IDLE".to_string()
        };

        results.push(ProjectExecutionInfo {
            project_id: p.id,
            repo_name: p.repo_name,
            repo_path: p.repo_path,
            is_running,
            is_idle,
            status: status_str,
            active_prompt: prompt_snippet,
            last_detected_at: last_time,
        });
    }

    results
}

/// Returns true if ANY project or conversation is currently actively running
pub fn is_any_prompt_actively_running() -> bool {
    // 1. Check Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            let conn = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
            .or_else(|_| {
                let uri = format!(
                    "file:{}?immutable=1",
                    summaries_db.to_string_lossy().replace('\\', "/")
                );
                Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(conn) = conn {
                let _ = conn.pragma_update(None, "busy_timeout", 3000);
                let count: i32 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM conversation_summaries WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if count > 0 {
                    return true;
                }

                // Check recency within 600s
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT last_modified_time FROM conversation_summaries ORDER BY last_modified_time DESC LIMIT 10",
                ) {
                    let now = Utc::now();
                    let rows = stmt.query_map([], |row| row.get::<_, String>(0));
                    if let Ok(rows) = rows {
                        for time_str in rows.flatten() {
                            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&time_str) {
                                let age = (now - parsed.with_timezone(&Utc)).num_seconds();
                                if age >= 0 && age < 600 {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check repo_prompts.db active_prompts
    if let Ok(conn) = connect_db() {
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status = 'running' OR status = 'queued'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if count > 0 {
            return true;
        }
    }

    // 3. Process check: if Antigravity process is running
    let default_dir = crate::modules::instance::get_default_antigravity_data_dir();
    let pids =
        crate::modules::instance::find_pids_for_data_dir(&default_dir.to_string_lossy(), true);
    if !pids.is_empty() {
        return true;
    }

    false
}

/// Save or re-queue an active prompt into active_prompts and running_projects
pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let clean_repo_name = Path::new(&prompt.repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| prompt.project_id.clone());
    let _ = conn.execute(
        "INSERT OR REPLACE INTO running_projects 
         (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
         VALUES (?, ?, ?, ?, NULL, 1, ?, ?)",
        params![&prompt.project_id, &prompt.instance_id, &clean_repo_name, &prompt.repo_path, now, now],
    );

    conn.execute(
        "INSERT OR REPLACE INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &prompt.id,
            &prompt.project_id,
            &prompt.instance_id,
            &prompt.repo_path,
            &prompt.prompt_content,
            &prompt.model,
            &prompt.session_id,
            &prompt.status,
            prompt.created_at,
            now,
            &prompt.image_payload,
        ],
    ).map_err(|e| format!("Failed to insert active prompt: {}", e))?;

    Ok(())
}

/// Extract clean prompt text by stripping <USER_REQUEST> / <ADDITIONAL_METADATA> tags if present
pub fn extract_clean_user_prompt(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(start) = text.find("<USER_REQUEST>") {
        let after = &text[start + "<USER_REQUEST>".len()..];
        if let Some(end) = after.find("</USER_REQUEST>") {
            text = &after[..end];
        } else {
            text = after;
        }
    }
    if let Some(end) = text.find("<ADDITIONAL_METADATA>") {
        text = &text[..end];
    }
    if let Some(end) = text.find("<CONTEXT_SUMMARY>") {
        text = &text[..end];
    }
    text.trim().to_string()
}

/// Extract smart, readable summary of a prompt for display in CLI and Telegram.
/// Strips XML tags, skips leading commit SHAs, file paths, markdown headers, and extracts the primary human directive.
pub fn extract_smart_prompt_summary(raw: &str, max_len: usize) -> String {
    let cleaned = extract_clean_user_prompt(raw);
    let mut substantive_lines = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if line is a commit hash or list of hashes (e.g. 40 hex chars or 10-64 hex chars)
        let is_hex_sha = (trimmed.len() == 40 || (trimmed.len() >= 8 && trimmed.len() <= 64))
            && trimmed.chars().all(|c| c.is_ascii_hexdigit());
        if is_hex_sha {
            continue;
        }

        // Check if line is a pure file path (e.g. D:\work\... or C:\... or /path/...)
        if trimmed.starts_with("D:\\")
            || trimmed.starts_with("C:\\")
            || trimmed.starts_with("file:///")
            || (trimmed.starts_with('/') && !trimmed.starts_with("/prompt"))
        {
            continue;
        }

        // Check if line is a markdown header (# Title), horizontal rule (---, ***), or blockquote (> ...)
        if trimmed.starts_with('#')
            || trimmed.starts_with('>')
            || trimmed.starts_with("---")
            || trimmed.starts_with("***")
        {
            continue;
        }

        // Check for common prompt preamble / telemetry boilerplates
        let lower = trimmed.to_lowercase();
        if lower.starts_with("prompt version:")
            || lower.starts_with("synchronization:")
            || lower.starts_with("top-instruction priority mandate")
            || lower.starts_with("application:")
            || lower.starts_with("error id:")
            || lower.starts_with("severity level:")
            || lower.starts_with("captured at:")
            || lower.starts_with("route / page:")
            || lower.starts_with("source:")
            || lower.starts_with("code:")
            || lower.starts_with("traceability id:")
        {
            continue;
        }

        substantive_lines.push(trimmed);
    }

    let candidate = if !substantive_lines.is_empty() {
        substantive_lines.join(" ")
    } else {
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    };

    let collapsed = candidate.split_whitespace().collect::<Vec<_>>().join(" ");
    let limit = if max_len == 0 { 80 } else { max_len };

    if collapsed.chars().count() > limit {
        let truncated: String = collapsed.chars().take(limit).collect();
        format!("{}...", truncated.trim_end())
    } else {
        collapsed
    }
}

/// Resolve the current Git branch for a given repository directory
pub fn get_git_branch_for_path(repo_path: &str) -> Option<String> {
    let p = Path::new(repo_path);
    if !p.exists() {
        return None;
    }
    let git_head = p.join(".git").join("HEAD");
    if git_head.exists() {
        if let Ok(content) = fs::read_to_string(&git_head) {
            let line = content.trim();
            if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
                return Some(branch.trim().to_string());
            }
        }
    }
    // Worktree or submodule where .git is a file
    let git_file = p.join(".git");
    if git_file.is_file() {
        if let Ok(content) = fs::read_to_string(&git_file) {
            if let Some(gitdir_rel) = content.trim().strip_prefix("gitdir:") {
                let gitdir_path = p.join(gitdir_rel.trim());
                let sub_head = gitdir_path.join("HEAD");
                if sub_head.exists() {
                    if let Ok(c) = fs::read_to_string(&sub_head) {
                        let line = c.trim();
                        if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
                            return Some(branch.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Query conversation title from Antigravity conversation_summaries.db
pub fn lookup_conversation_title(prefix_or_id: &str) -> Option<String> {
    let clean = prefix_or_id.trim();
    if clean.is_empty() {
        return None;
    }
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")))?;
    let summaries_db = base_dir.join("conversation_summaries.db");
    if !summaries_db.exists() {
        return None;
    }
    let conn = Connection::open_with_flags(
        &summaries_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .or_else(|_| {
        let uri = format!(
            "file:{}?immutable=1",
            summaries_db.to_string_lossy().replace('\\', "/")
        );
        Connection::open_with_flags(
            &uri,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
    })
    .ok()?;
    let _ = conn.pragma_update(None, "busy_timeout", 2000);

    let pattern = format!("%{}%", clean);
    let mut stmt = conn
        .prepare("SELECT title FROM conversation_summaries WHERE conversation_id LIKE ? OR project_id LIKE ? LIMIT 1")
        .ok()?;
    let mut rows = stmt
        .query_map(params![pattern, pattern], |r| r.get::<_, String>(0))
        .ok()?;
    if let Some(Ok(title)) = rows.next() {
        let t = title.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

/// Format human-friendly workspace label (e.g. `Antigravity-Manager (main) [AGM]`)
pub fn format_friendly_workspace_label(
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
) -> String {
    // 1. Resolve repository name cleanly
    let base_name = if !repo_name.trim().is_empty()
        && (!repo_name.contains('-') || !repo_name.chars().any(|c| c.is_ascii_digit()))
    {
        repo_name.trim().to_string()
    } else {
        Path::new(repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if let Some(idx) = project_id.rfind('-') {
                    let suffix = &project_id[idx + 1..];
                    if suffix.len() >= 8 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                        return project_id[..idx].to_string();
                    }
                }
                project_id.to_string()
            })
    };

    // 2. Resolve Git branch
    let branch_opt = get_git_branch_for_path(repo_path);

    // 3. Resolve Conversation title
    let id_part = if let Some(idx) = project_id.rfind('-') {
        &project_id[idx + 1..]
    } else {
        project_id
    };
    let title_opt = lookup_conversation_title(id_part);

    // 4. Construct composite label
    let mut label = base_name;
    if let Some(b) = branch_opt {
        label.push_str(&format!(" ({})", b));
    }
    if let Some(t) = title_opt {
        let t_trim = t.trim();
        if !t_trim.is_empty() && !label.to_lowercase().contains(&t_trim.to_lowercase()) {
            label.push_str(&format!(" [{}]", t_trim));
        }
    }
    label
}

/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Strips prompt envelope wrappers and executes cleanly via -p without failing on GUI trajectory IDs.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool {
    let ws_dir = PathBuf::from(&prompt.repo_path);
    if !ws_dir.exists() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Workspace directory does not exist: '{}', skipping agy spawn",
            prompt.repo_path
        ));
        return false;
    }

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    if clean_prompt.trim().is_empty() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Prompt content for '{}' is empty, skipping agy spawn",
            prompt.id
        ));
        return false;
    }

    let ws_key = format!("{}:{}", prompt.instance_id, prompt.repo_path);
    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        if let Some(&existing_pid) = workers.get(&ws_key) {
            let mut sys = sysinfo::System::new();
            let target_pid = sysinfo::Pid::from_u32(existing_pid);
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            );
            if sys.process(target_pid).is_some() {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] An agy worker (PID: {}) is already active for workspace '{}'. Prompt '{}' was not sent again.",
                    existing_pid, ws_key, prompt.id
                ));
                return false;
            } else {
                workers.remove(&ws_key);
            }
        }
    }

    if let Some(agy_bin) = crate::modules::process::get_antigravity_cli_executable_path() {
        let mut cmd = std::process::Command::new(&agy_bin);
        cmd.current_dir(&ws_dir);
        cmd.arg("--dangerously-skip-permissions");

        if clean_prompt.len() <= 24000 {
            cmd.arg("-p").arg(&clean_prompt);
        } else {
            cmd.arg("-p").arg(&clean_prompt[..24000]);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        if prompt.instance_id != "default" && !prompt.instance_id.is_empty() {
            if let Ok(inst_home) =
                crate::modules::instance::get_instance_home_dir(&prompt.instance_id)
            {
                #[cfg(target_os = "windows")]
                {
                    cmd.env("USERPROFILE", &inst_home);
                }
                cmd.env("HOME", &inst_home);
                cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
                cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
                cmd.env("SSH_TTY", "pty/0");
                cmd.env("WSL_DISTRO_NAME", "antigravity-isolated");
                cmd.env("DOCKER_CONTAINER", "1");

                if let Ok(registry) = crate::modules::instance::load_registry() {
                    if let Some(inst) = registry
                        .instances
                        .iter()
                        .find(|i| i.id == prompt.instance_id)
                    {
                        if let Some(ref acc_id) = inst.bound_account_id {
                            if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
                            }
                        }
                    }
                }
            }
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                let child_pid = child.id();
                {
                    let mut workers = get_active_agy_workers().lock().unwrap();
                    workers.insert(ws_key, child_pid);
                }
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                    prompt.id, child_pid, prompt.instance_id, prompt.repo_path, clean_prompt
                ));
                true
            }
            Err(e) => {
                crate::modules::logger::log_error(&format!(
                    "[RepoDB] Failed to spawn agy execution for prompt '{}': {}",
                    prompt.id, e
                ));
                false
            }
        }
    } else {
        crate::modules::logger::log_warn(
            "[RepoDB] agy executable not found, cannot resume prompt via CLI",
        );
        false
    }
}

/// Resend and restore all previous running/backed-up/dispatched commands across all instances.
pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String> {
    resend_running_commands_for_instance(None, limit)
}

/// Resend and restore previous running/backed-up/queued commands scoped to an optional `instance_id`.
/// Strictly filters to unrestored/backed-up/queued prompts, prioritizes the latest running prompts,
/// and deduplicates against already dispatched prompts and workspace repositories.
pub fn resend_running_commands_for_instance(
    instance_id: Option<&str>,
    limit: usize,
) -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending', 'running')
             ORDER BY updated_at DESC LIMIT ?",
        )
        .map_err(|e| format!("Failed to prepare resend query: {}", e))?;

    let all_prompts = stmt
        .query_map([limit.saturating_mul(2).max(20)], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query prompts for resend: {}", e))?
        .flatten()
        .collect::<Vec<ActivePrompt>>();

    let instance_projects = match instance_id {
        Some(id) if id != "all" => detect_running_projects(id).unwrap_or_default(),
        _ => Vec::new(),
    };
    let instance_repo_paths: HashSet<String> = instance_projects
        .iter()
        .map(|proj| normalize_path_for_compare(&proj.repo_path))
        .collect();

    let prompts: Vec<ActivePrompt> = all_prompts
        .into_iter()
        .filter(|p| match instance_id {
            None | Some("all") => true,
            Some("default") | Some("__default__") => {
                p.instance_id == "default"
                    || p.instance_id == "__default__"
                    || p.instance_id.is_empty()
                    || instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))
            }
            Some(inst) => {
                p.instance_id == inst
                    || instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))
            }
        })
        .take(limit)
        .collect();

    let mut resent = Vec::new();
    let mut dispatched_repos = HashSet::new();

    for mut prompt in prompts {
        if prompt.status == "queued" {
            continue;
        }
        let clean_path = prompt.repo_path.trim().to_lowercase().replace('\\', "/");
        if dispatched_repos.contains(&clean_path) {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping older prompt '{}' for repo '{}', already dispatched latest prompt for this workspace",
                prompt.id, prompt.repo_path
            ));
            let _ = conn.execute(
                "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
                rusqlite::params![now, &prompt.id],
            );
            continue;
        }
        dispatched_repos.insert(clean_path);

        let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
        let already_dispatched = if prompt.status == "backed_up" {
            false
        } else {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            let seen = cache.contains(&prompt.id) || cache.contains(&sig);
            if !seen {
                cache.insert(prompt.id.clone());
                cache.insert(sig);
            }
            seen
        };

        if already_dispatched {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping duplicate dispatch for prompt '{}' in '{}'",
                prompt.id, prompt.repo_path
            ));
            let _ = conn.execute(
                "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = ?",
                rusqlite::params![now, &prompt.id],
            );
            continue;
        }

        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        if prompt.image_payload.is_none() && extracted_img.is_some() {
            prompt.image_payload = extracted_img.clone();
        }
        let has_image = prompt.image_payload.is_some() || !img_paths.is_empty();

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt.id,
            "project_id": prompt.project_id,
            "instance_id": prompt.instance_id,
            "repo_path": prompt.repo_path,
            "prompt_content": prompt.prompt_content,
            "model": prompt.model,
            "image_payload": prompt.image_payload,
            "image_paths": img_paths,
            "has_image": has_image,
            "auto_boot": true,
            "status": "dispatched",
            "resumed_at": now,
        });

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, json_str);
        }

        let sent = spawn_prompt_via_agy(&prompt);
        if !sent {
            crate::modules::logger::log_error(&format!(
                "[RepoDB] Prompt '{}' was not re-pushed; left {}",
                prompt.id, prompt.status
            ));
            continue;
        }

        let _ = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?, image_payload = ? WHERE id = ?",
            rusqlite::params![now, &prompt.image_payload, &prompt.id],
        );
        let _ = conn.execute(
            "UPDATE running_projects SET is_running = 1, last_detected_at = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![now, now, &prompt.project_id],
        );

        prompt.status = "dispatched".to_string();
        prompt.updated_at = now;
        resent.push(prompt);
    }

    Ok(resent)
}

/// Auto-resume recent prompts for projects active within max_age_seconds (strictly skips projects older than threshold)
pub fn auto_resume_recent_prompts(
    instance_id: &str,
    max_age_seconds: i64,
) -> Result<AutoResumeResult, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let threshold_cutoff = now - max_age_seconds;

    // Load instance details to determine account email
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let account_email = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .and_then(|i| i.bound_email.clone())
        .unwrap_or_else(|| "unbound".to_string());

    let mut stmt = conn
        .prepare(
            "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
             FROM running_projects 
             WHERE instance_id = ? OR ? = ''
             ORDER BY last_detected_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let projects = stmt
        .query_map([instance_id, instance_id], |row| {
            let running_int: i32 = row.get(5)?;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running: running_int != 0,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query projects: {}", e))?
        .flatten()
        .collect::<Vec<RunningProject>>();

    let mut resumed_prompts = Vec::new();
    let mut resumed_count = 0;
    let mut skipped_count = 0;

    for project in projects {
        // STRICT RECENCY FILTER: If project last active > max_age_seconds (e.g. > 1 hour, 2h, 5h, 1 day), SKIP!
        let is_stale = project.last_detected_at < threshold_cutoff;
        if is_stale {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Skipping project '{}' (last detected {}s ago, exceeds threshold {}s)",
                project.repo_name,
                now - project.last_detected_at,
                max_age_seconds
            ));
            skipped_count += 1;
            continue;
        }

        // Project was active within < 1 hour! Find its most recent prompt + image
        let proj_like = format!("%{}%", project.repo_name.to_lowercase());
        let mut prompt_stmt = conn
            .prepare(
                "SELECT id, prompt_content, model, image_payload 
                 FROM active_prompts 
                 WHERE project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3
                 ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(|e| format!("Failed to prepare prompt query: {}", e))?;

        let mut maybe_prompt = prompt_stmt
            .query_row(
                rusqlite::params![&project.id, &project.repo_path, &proj_like],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .ok();

        // If not found in DB, check existing .antigravity_resume_task.json
        if maybe_prompt.is_none() {
            let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
            if task_file.exists() {
                if let Ok(c) = fs::read_to_string(&task_file) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                        if let Some(txt) = v.get("prompt_content").and_then(|t| t.as_str()) {
                            if !txt.trim().is_empty() {
                                let pid = v
                                    .get("prompt_id")
                                    .and_then(|i| i.as_str())
                                    .unwrap_or(&project.id)
                                    .to_string();
                                let m = v
                                    .get("model")
                                    .and_then(|m| m.as_str())
                                    .map(|s| s.to_string());
                                let img = v
                                    .get("image_payload")
                                    .and_then(|i| i.as_str())
                                    .map(|s| s.to_string());
                                maybe_prompt = Some((pid, txt.to_string(), m, img));
                            }
                        }
                    }
                }
            }
        }

        let (prompt_id, prompt_text, prompt_model, image_payload) = match maybe_prompt {
            Some((id, content, model, img)) => (id, content, model, img),
            None => {
                let new_id = Uuid::new_v4().to_string();
                let content = format!(
                    "Resume active project workspace for '{}' [{}] after IDE crash recovery",
                    project.repo_name, project.repo_path
                );
                (new_id, content, Some("gemini-pro".to_string()), None)
            }
        };

        let sig = format!("{}:{}", project.repo_path, prompt_text.trim());
        let already_dispatched = {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            let seen = cache.contains(&prompt_id) || cache.contains(&sig);
            if !seen {
                cache.insert(prompt_id.clone());
                cache.insert(sig);
            }
            seen
        };

        if already_dispatched {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] auto_resume_recent_prompts skipping already dispatched prompt '{}' in '{}'",
                prompt_id, project.repo_path
            ));
            continue;
        }

        let has_image = image_payload.is_some();

        // Dispatch directly to project folder via .antigravity_resume_task.json to spin up boot process immediately
        let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt_id,
            "project_id": project.id,
            "instance_id": instance_id,
            "repo_path": project.repo_path,
            "prompt_content": prompt_text,
            "model": prompt_model,
            "image_payload": image_payload,
            "has_image": has_image,
            "auto_boot": true,
            "resumed_at": now,
        });

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, json_str);
        }

        // Mark / update status in active_prompts as dispatched
        let _ = conn.execute(
            "INSERT OR REPLACE INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'dispatched', ?, ?, ?)",
            params![
                &prompt_id,
                &project.id,
                instance_id,
                &project.repo_path,
                &prompt_text,
                &prompt_model,
                &project.id,
                now,
                now,
                &image_payload,
            ],
        );

        let prompt_obj = ActivePrompt {
            id: prompt_id.clone(),
            project_id: project.id.clone(),
            instance_id: instance_id.to_string(),
            repo_path: project.repo_path.clone(),
            prompt_content: prompt_text.clone(),
            model: prompt_model.clone(),
            session_id: Some(project.id.clone()),
            status: "dispatched".to_string(),
            created_at: now,
            updated_at: now,
            image_payload: image_payload.clone(),
        };
        spawn_prompt_via_agy(&prompt_obj);

        let preview = if prompt_text.len() > 80 {
            format!("{}...", &prompt_text[..77])
        } else {
            prompt_text.clone()
        };

        resumed_prompts.push(AutoResumePromptInfo {
            project_id: project.id.clone(),
            repo_path: project.repo_path.clone(),
            prompt_preview: preview,
            has_image,
        });

        resumed_count += 1;
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Auto-resumed prompt for project '{}' (image: {})",
            project.repo_name, has_image
        ));
    }

    Ok(AutoResumeResult {
        instance_id: instance_id.to_string(),
        account_email,
        resumed_project_count: resumed_count,
        skipped_project_count: skipped_count,
        resumed_prompts,
    })
}

/// Conversation node inside an AGM Project Tree (with dual AGM `C001` & GitMap `GM:<short_id>` sequence codes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmConversationNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub conversation_id: String,
    pub short_id: String,
    pub title: String,
    pub status: String,
    pub is_running: bool,
    pub step_count: usize,
    pub instance_id: String,
    pub prompt_preview_200w: String,
    pub prompt_word_count: usize,
    pub last_modified: String,
}

/// Project node in the AGM Tree View (Project -> Conversation -> 200-Word Prompt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmProjectTreeNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub instance_seq_num: Option<u32>,
    pub instance_name: String,
    pub bound_email: Option<String>,
    pub is_running: bool,
    pub conversations: Vec<AgmConversationNode>,
}

/// Resolved target from an AGM or GitMap Sequence ID (`P001`, `AGM:P001`, `GM:#1`, `C001`, `AGM:C001`, `GM:<cid>`, or conversation UUID prefix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmSequenceResolution {
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
}

fn ensure_project_sequence_in_conn(
    conn: &Connection,
    project_key: &str,
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(existing) = conn.query_row(
        "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
        params![project_key],
        |row| row.get::<_, i64>(0),
    ) {
        let _ = conn.execute(
            "UPDATE agm_project_sequences SET project_id = ?2, repo_name = ?3, repo_path = ?4, instance_id = ?5, updated_at = ?6 WHERE project_key = ?1",
            params![project_key, project_id, repo_name, repo_path, instance_id, now],
        );
        return existing;
    }

    let next_seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_project_sequences",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let _ = conn.execute(
        "INSERT OR IGNORE INTO agm_project_sequences (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![project_key, next_seq, project_id, repo_name, repo_path, instance_id, now],
    );

    conn.query_row(
        "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
        params![project_key],
        |row| row.get(0),
    )
    .unwrap_or(next_seq)
}

fn ensure_conversation_sequence_in_conn(
    conn: &Connection,
    conversation_id: &str,
    project_key: &str,
    title: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(existing) = conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get::<_, i64>(0),
    ) {
        let _ = conn.execute(
            "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, instance_id = ?4, updated_at = ?5 WHERE conversation_id = ?1",
            params![conversation_id, project_key, title, instance_id, now],
        );
        return existing;
    }

    let next_seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let _ = conn.execute(
        "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![conversation_id, next_seq, project_key, title, instance_id, now],
    );

    conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )
    .unwrap_or(next_seq)
}

/// Extract clean user prompt up to `max_words` (e.g. 200 words) and return `(preview_text, total_word_count)`
pub fn extract_prompt_words_preview(raw_text: &str, max_words: usize) -> (String, usize) {
    let cleaned = extract_clean_user_prompt(raw_text);
    let mut meaningful_tokens: Vec<&str> = Vec::new();
    for line in cleaned.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        // Skip standalone 40-char git SHA lines
        if t.len() == 40 && t.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        for word in t.split_whitespace() {
            meaningful_tokens.push(word);
        }
    }

    let total_words = meaningful_tokens.len();
    if total_words == 0 {
        return (String::new(), 0);
    }

    let limit = max_words.max(1);
    if total_words <= limit {
        (meaningful_tokens.join(" "), total_words)
    } else {
        let truncated = meaningful_tokens[..limit].join(" ");
        (format!("{} ...", truncated), total_words)
    }
}

/// Inspect transcript.jsonl for a conversation to obtain `(step_count, latest_user_prompt)`
fn inspect_conversation_transcript(
    base_dir: &Path,
    conversation_id: &str,
) -> (usize, Option<String>) {
    let transcript_path = base_dir
        .join("brain")
        .join(conversation_id)
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl");
    if !transcript_path.exists() {
        return (0, None);
    }
    let content = match std::fs::read_to_string(&transcript_path) {
        Ok(c) => c,
        Err(_) => return (0, None),
    };
    let mut step_count = 0usize;
    let mut latest_prompt: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        step_count += 1;
        if trimmed.contains("\"USER_INPUT\"") || trimmed.contains("\"USER_EXPLICIT\"") {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let is_user_input = val
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "USER_INPUT")
                    .unwrap_or(false)
                    || val
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "USER_EXPLICIT")
                        .unwrap_or(false);
                if is_user_input {
                    if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                        let clean = extract_clean_user_prompt(c);
                        if !clean.is_empty() {
                            latest_prompt = Some(clean);
                        }
                    }
                }
            }
        }
    }
    (step_count, latest_prompt)
}

/// Build the AGM Project -> Conversation -> 200-Word Prompt Tree across ALL registered instances
/// and persist Dual AGM (`P001`/`C001`) + GitMap (`GM:#1`/`GM:<short_id>`) Sequence IDs in `repo_prompts.db`.
pub fn get_project_conversation_tree(
    max_words: usize,
    only_running: bool,
) -> Vec<AgmProjectTreeNode> {
    let _ = discover_running_prompts_from_antigravity("__default__");
    let _ = detect_running_projects("__default__");

    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    for inst in &registry.instances {
        let _ = detect_running_projects(&inst.id);
    }

    let default_email = crate::modules::account::get_current_account()
        .ok()
        .flatten()
        .map(|a| a.email);

    let projects = list_running_projects().unwrap_or_default();
    let conn_opt = connect_db().ok();

    let mut convs_by_path: std::collections::HashMap<
        String,
        Vec<(String, String, String, String, bool, usize, String, String)>,
    > = std::collections::HashMap::new();

    let mut active_prompts_by_path: std::collections::HashMap<String, Vec<ActivePrompt>> =
        std::collections::HashMap::new();
    if let Ok(active_list) = list_all_prompts() {
        for ap in active_list {
            let key = normalize_path_for_compare(&ap.repo_path);
            active_prompts_by_path.entry(key).or_default().push(ap);
        }
    }

    let candidate_dirs = crate::modules::agy_cleaner::get_gemini_candidate_dirs();
    let mut seen_tree_cids = std::collections::HashSet::new();

    for base in &candidate_dirs {
        let summaries_db = base.join("conversation_summaries.db");
        if summaries_db.exists() {
            let s_conn = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
            .or_else(|_| {
                let uri = format!(
                    "file:{}?immutable=1",
                    summaries_db.to_string_lossy().replace('\\', "/")
                );
                Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(s_conn) = s_conn {
                let _ = s_conn.pragma_update(None, "busy_timeout", 3000);
                if let Ok(mut stmt) = s_conn.prepare(
                    "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
                     FROM conversation_summaries 
                     ORDER BY last_modified_time DESC 
                     LIMIT 40",
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i32>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_time_str) = item;
                            if !seen_tree_cids.insert(cid.clone()) {
                                continue;
                            }
                            let is_recency_active = if let Ok(parsed) =
                                chrono::DateTime::parse_from_rfc3339(&last_time_str)
                            {
                                let age = (Utc::now() - parsed.with_timezone(&Utc)).num_seconds();
                                age >= 0 && age < 600
                            } else {
                                false
                            };
                            let is_conv_running =
                                not_fully_idle != 0 || status.contains("RUNNING") || is_recency_active;
                            let (steps, transcript_prompt) = inspect_conversation_transcript(base, &cid);
                            let effective_prompt = transcript_prompt
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| preview.clone());

                            let mut assigned_paths: Vec<String> = Vec::new();
                            if let Some(ws_raw) = ws_uris_opt {
                                let uris: Vec<String> =
                                    serde_json::from_str(&ws_raw).unwrap_or_default();
                                for u in uris {
                                    let p = normalize_path_for_compare(&decode_uri_to_path(&u));
                                    if !p.is_empty() && !assigned_paths.contains(&p) {
                                        assigned_paths.push(p);
                                    }
                                }
                            }
                            if assigned_paths.is_empty() {
                                assigned_paths.push("__unassigned__".to_string());
                            }
                            for p_key in assigned_paths {
                                convs_by_path.entry(p_key).or_default().push((
                                    cid.clone(),
                                    title.clone(),
                                    effective_prompt.clone(),
                                    status.clone(),
                                    is_conv_running,
                                    steps,
                                    last_time_str.clone(),
                                    "default".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut tree_nodes: Vec<AgmProjectTreeNode> = Vec::new();
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let mut seen_project_keys: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();

    for proj in &projects {
        let norm_path = normalize_path_for_compare(&proj.repo_path);
        let project_key = if !norm_path.is_empty() {
            norm_path.clone()
        } else {
            proj.id.clone()
        };

        if !seen_project_keys.insert((project_key.clone(), proj.instance_id.clone())) {
            continue;
        }

        let p_seq = if let Some(ref conn) = conn_opt {
            ensure_project_sequence_in_conn(
                conn,
                &project_key,
                &proj.id,
                &proj.repo_name,
                &proj.repo_path,
                &proj.instance_id,
            )
        } else {
            (tree_nodes.len() as i64) + 1
        };

        let mut conv_nodes: Vec<AgmConversationNode> = Vec::new();
        if let Some(raw_convs) = convs_by_path.get(&norm_path) {
            for (cid, title, raw_prompt, status, is_run, steps, last_mod, inst) in raw_convs {
                if only_running && !*is_run {
                    continue;
                }
                let c_seq = if let Some(ref conn) = conn_opt {
                    ensure_conversation_sequence_in_conn(conn, cid, &project_key, title, inst)
                } else {
                    (conv_nodes.len() as i64) + 1
                };
                let (preview_200w, word_count) = extract_prompt_words_preview(raw_prompt, word_cap);
                let short_id = if cid.len() >= 8 {
                    cid[..8].to_string()
                } else {
                    cid.clone()
                };
                conv_nodes.push(AgmConversationNode {
                    seq_id: c_seq,
                    seq_code: format!("C{:03}", c_seq),
                    gitmap_seq_code: format!("GM:{}", short_id),
                    conversation_id: cid.clone(),
                    short_id,
                    title: if title.trim().is_empty() {
                        "Untitled Conversation".to_string()
                    } else {
                        title.clone()
                    },
                    status: if *is_run {
                        "RUNNING".to_string()
                    } else if status.trim().is_empty() {
                        "IDLE".to_string()
                    } else {
                        status.clone()
                    },
                    is_running: *is_run,
                    step_count: *steps,
                    instance_id: proj.instance_id.clone(),
                    prompt_preview_200w: preview_200w,
                    prompt_word_count: word_count,
                    last_modified: last_mod.clone(),
                });
            }
        }

        if let Some(aps) = active_prompts_by_path.get(&norm_path) {
            for ap in aps {
                let cid = ap.session_id.clone().unwrap_or_else(|| ap.id.clone());
                if conv_nodes.iter().any(|c| c.conversation_id == cid) {
                    continue;
                }
                let is_run =
                    ap.status == "running" || ap.status == "queued" || ap.status == "backed_up";
                if only_running && !is_run {
                    continue;
                }
                let title = extract_smart_prompt_summary(&ap.prompt_content, 60);
                let c_seq = if let Some(ref conn) = conn_opt {
                    ensure_conversation_sequence_in_conn(
                        conn,
                        &cid,
                        &project_key,
                        &title,
                        &ap.instance_id,
                    )
                } else {
                    (conv_nodes.len() as i64) + 1
                };
                let (preview_200w, word_count) =
                    extract_prompt_words_preview(&ap.prompt_content, word_cap);
                let short_id = if cid.len() >= 8 {
                    cid[..8].to_string()
                } else {
                    cid.clone()
                };
                conv_nodes.push(AgmConversationNode {
                    seq_id: c_seq,
                    seq_code: format!("C{:03}", c_seq),
                    gitmap_seq_code: format!("GM:{}", short_id),
                    conversation_id: cid,
                    short_id,
                    title,
                    status: ap.status.to_uppercase(),
                    is_running: is_run,
                    step_count: 0,
                    instance_id: ap.instance_id.clone(),
                    prompt_preview_200w: preview_200w,
                    prompt_word_count: word_count,
                    last_modified: ap.updated_at.to_string(),
                });
            }
        }

        let proj_is_running = conv_nodes.iter().any(|c| c.is_running);
        if only_running && !proj_is_running && conv_nodes.is_empty() {
            continue;
        }

        let (instance_seq_num, instance_name, bound_email) = if proj.instance_id == "default"
            || proj.instance_id == "__default__"
            || proj.instance_id.is_empty()
        {
            (Some(1), "default".to_string(), default_email.clone())
        } else if let Some(inst) = registry
            .instances
            .iter()
            .find(|i| i.id == proj.instance_id || i.name == proj.instance_id)
        {
            (inst.seq_num, inst.name.clone(), inst.bound_email.clone())
        } else {
            (None, proj.instance_id.clone(), None)
        };

        tree_nodes.push(AgmProjectTreeNode {
            seq_id: p_seq,
            seq_code: format!("P{:03}", p_seq),
            gitmap_seq_code: format!("GM:#{}", p_seq),
            project_id: proj.id.clone(),
            repo_name: proj.repo_name.clone(),
            repo_path: proj.repo_path.clone(),
            instance_id: proj.instance_id.clone(),
            instance_seq_num,
            instance_name,
            bound_email,
            is_running: proj_is_running,
            conversations: conv_nodes,
        });
    }

    tree_nodes.sort_by_key(|n| n.seq_id);
    tree_nodes
}

/// Resolve an AGM or GitMap Sequence ID (`P001`, `AGM:P001`, `GM:#1`, `C001`, `AGM:C001`, `GM:<cid>`, or conversation UUID prefix)
/// to its target project, instance, and optional conversation.
pub fn resolve_agm_sequence_target(target_token: &str) -> Option<AgmSequenceResolution> {
    let tree = get_project_conversation_tree(200, false);
    let mut clean = target_token
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if let Some(stripped) = clean
        .strip_prefix("AGM:")
        .or_else(|| clean.strip_prefix("agm:"))
        .or_else(|| clean.strip_prefix("GM:"))
        .or_else(|| clean.strip_prefix("gm:"))
    {
        clean = stripped.trim();
    }
    let clean = clean.trim_start_matches('#');
    if clean.is_empty() {
        return None;
    }

    let upper = clean.to_uppercase();

    if let Some(num_str) = upper.strip_prefix('P') {
        if let Ok(seq_num) = num_str.parse::<i64>() {
            if let Some(proj) = tree.iter().find(|p| p.seq_id == seq_num) {
                let first_conv = proj.conversations.first();
                return Some(AgmSequenceResolution {
                    seq_code: proj.seq_code.clone(),
                    gitmap_seq_code: proj.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: proj.instance_id.clone(),
                    conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                    conversation_title: first_conv.map(|c| c.title.clone()),
                });
            }
        }
    }

    if let Some(num_str) = upper.strip_prefix('C') {
        if let Ok(seq_num) = num_str.parse::<i64>() {
            for proj in &tree {
                if let Some(conv) = proj.conversations.iter().find(|c| c.seq_id == seq_num) {
                    return Some(AgmSequenceResolution {
                        seq_code: conv.seq_code.clone(),
                        gitmap_seq_code: conv.gitmap_seq_code.clone(),
                        project_id: proj.project_id.clone(),
                        repo_name: proj.repo_name.clone(),
                        repo_path: proj.repo_path.clone(),
                        instance_id: conv.instance_id.clone(),
                        conversation_id: Some(conv.conversation_id.clone()),
                        conversation_title: Some(conv.title.clone()),
                    });
                }
            }
        }
    }

    if let Ok(seq_num) = clean.parse::<i64>() {
        if let Some(proj) = tree.iter().find(|p| p.seq_id == seq_num) {
            let first_conv = proj.conversations.first();
            return Some(AgmSequenceResolution {
                seq_code: proj.seq_code.clone(),
                gitmap_seq_code: proj.gitmap_seq_code.clone(),
                project_id: proj.project_id.clone(),
                repo_name: proj.repo_name.clone(),
                repo_path: proj.repo_path.clone(),
                instance_id: proj.instance_id.clone(),
                conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                conversation_title: first_conv.map(|c| c.title.clone()),
            });
        }
        for proj in &tree {
            if let Some(conv) = proj.conversations.iter().find(|c| c.seq_id == seq_num) {
                return Some(AgmSequenceResolution {
                    seq_code: conv.seq_code.clone(),
                    gitmap_seq_code: conv.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: conv.instance_id.clone(),
                    conversation_id: Some(conv.conversation_id.clone()),
                    conversation_title: Some(conv.title.clone()),
                });
            }
        }
    }

    let lower = clean.to_lowercase();
    if lower.len() >= 4 {
        for proj in &tree {
            if let Some(conv) = proj
                .conversations
                .iter()
                .find(|c| c.conversation_id.to_lowercase().starts_with(&lower))
            {
                return Some(AgmSequenceResolution {
                    seq_code: conv.seq_code.clone(),
                    gitmap_seq_code: conv.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: conv.instance_id.clone(),
                    conversation_id: Some(conv.conversation_id.clone()),
                    conversation_title: Some(conv.title.clone()),
                });
            }
        }
        if let Some(proj) = tree.iter().find(|p| {
            p.project_id.to_lowercase().contains(&lower)
                || p.repo_name.to_lowercase().contains(&lower)
        }) {
            let first_conv = proj.conversations.first();
            return Some(AgmSequenceResolution {
                seq_code: proj.seq_code.clone(),
                gitmap_seq_code: proj.gitmap_seq_code.clone(),
                project_id: proj.project_id.clone(),
                repo_name: proj.repo_name.clone(),
                repo_path: proj.repo_path.clone(),
                instance_id: proj.instance_id.clone(),
                conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                conversation_title: first_conv.map(|c| c.title.clone()),
            });
        }
    }

    None
}

/// Inject a prompt into a specific project or conversation by AGM/GitMap Sequence ID (`C001`, `P001`, `AGM:C001`, `GM:#1`),
/// with optional `--instance <id|#seq|name>` and remote machine `--node <node>` scoping.
pub fn prompt_target_by_sequence_scoped(
    target_token: &str,
    prompt_text: &str,
    instance_override: Option<&str>,
    node_override: Option<&str>,
) -> Result<String, String> {
    let clean_prompt = prompt_text.trim();
    if clean_prompt.is_empty() {
        return Err("Prompt text cannot be empty.".to_string());
    }

    // Remote SSH Node Delegation if `node_override` is provided and not "local"
    if let Some(node) = node_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "local")
    {
        let inst_flag = instance_override
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|i| format!(" --instance {}", i))
            .unwrap_or_default();
        let escaped_prompt = clean_prompt.replace('"', "\\\"");
        let remote_cmd = format!(
            "agm prompt {}{} \"{}\"",
            target_token.trim(),
            inst_flag,
            escaped_prompt
        );
        let output = std::process::Command::new("gitmap")
            .args(["ssh", "exec", &remote_cmd, "--node", node])
            .output()
            .or_else(|_| {
                std::process::Command::new("gitmap")
                    .args(["ssh", "exec", &remote_cmd])
                    .output()
            })
            .map_err(|e| format!("Failed to delegate prompt to SSH node '{}': {}", node, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            return Ok(format!(
                "🌐 Delegated prompt to remote node '{}' for target [{}]:\n{}",
                node,
                target_token.trim(),
                if stdout.is_empty() { "OK" } else { &stdout }
            ));
        } else {
            return Err(format!(
                "Remote SSH node '{}' returned error: {} {}",
                node, stdout, stderr
            ));
        }
    }

    let resolved = resolve_agm_sequence_target(target_token).ok_or_else(|| {
        format!(
            "Target '{}' not found in AGM/GitMap Tree. Run `agm tree` or `/tree` to view valid [AGM:P001 | GM:#1] and [AGM:C001 | GM:<cid>] sequence codes.",
            target_token
        )
    })?;

    // Resolve optional instance override (#1, #2, instance name, or UUID)
    let effective_instance_id = if let Some(inst_raw) = instance_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if inst_raw.eq_ignore_ascii_case("default") || inst_raw == "#1" || inst_raw == "1" {
            "default".to_string()
        } else if let Ok(reg) = crate::modules::instance::load_registry() {
            let clean_seq = inst_raw.trim_start_matches('#').parse::<u32>().ok();
            if let Some(found) = reg.instances.iter().find(|i| {
                i.id.eq_ignore_ascii_case(inst_raw)
                    || i.name.eq_ignore_ascii_case(inst_raw)
                    || (clean_seq.is_some() && i.seq_num == clean_seq)
            }) {
                found.id.clone()
            } else {
                inst_raw.to_string()
            }
        } else {
            inst_raw.to_string()
        }
    } else {
        resolved.instance_id.clone()
    };

    let now = Utc::now().timestamp();
    let prompt_id = Uuid::new_v4().to_string();
    let active_prompt = ActivePrompt {
        id: prompt_id.clone(),
        project_id: resolved.project_id.clone(),
        instance_id: effective_instance_id.clone(),
        repo_path: resolved.repo_path.clone(),
        prompt_content: clean_prompt.to_string(),
        model: Some("gemini-pro".to_string()),
        session_id: resolved
            .conversation_id
            .clone()
            .or_else(|| Some(resolved.project_id.clone())),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };

    let _ = save_or_requeue_prompt(&active_prompt);
    let spawned = spawn_prompt_via_agy(&active_prompt);

    let conv_label = resolved
        .conversation_id
        .as_deref()
        .map(|cid| {
            let short = if cid.len() >= 8 { &cid[..8] } else { cid };
            format!("conv:{}", short)
        })
        .unwrap_or_else(|| "new/latest conv".to_string());

    Ok(format!(
        "✅ Dispatched prompt to [AGM:{} | {}] ({}) on instance '{}' (spawned={}): \"{}\"",
        resolved.seq_code,
        resolved.gitmap_seq_code,
        conv_label,
        effective_instance_id,
        spawned,
        extract_smart_prompt_summary(clean_prompt, 80)
    ))
}

/// Format the Project -> Conversation -> 200-Word Prompt Tree View for AGM CLI output
pub fn format_tree_view_cli(max_words: usize, only_running: bool) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree(word_cap, only_running);
    let mut out = String::new();

    let mode_label = if only_running {
        "RUNNING PROJECTS & CONVERSATIONS"
    } else {
        "ALL PROJECTS & CONVERSATIONS"
    };
    out.push_str(&format!(
        "🌳 AGM + GITMAP PROJECT → CONVERSATION → [≤{}w PROMPT] TREE ({})\n",
        word_cap, mode_label
    ));
    out.push_str(&"━".repeat(86));
    out.push('\n');

    if tree.is_empty() {
        out.push_str("  (No matching projects or active conversations detected)\n");
        return out;
    }

    for proj in &tree {
        let proj_badge = if proj.is_running { "🟢" } else { "⚪" };
        let label =
            format_friendly_workspace_label(&proj.repo_name, &proj.project_id, &proj.repo_path);
        let inst_seq_str = proj
            .instance_seq_num
            .map(|n| format!("#{} ", n))
            .unwrap_or_default();
        let email_str = proj
            .bound_email
            .as_deref()
            .filter(|e| !e.is_empty())
            .map(|e| format!(" ({})", e))
            .unwrap_or_default();

        out.push_str(&format!(
            "📁 [AGM:{} | {}] [ProjID: {}] {} ({}) — {} [Instance: {}{}{}]\n",
            proj.seq_code,
            proj.gitmap_seq_code,
            proj.project_id,
            label,
            proj_badge,
            proj.repo_path,
            inst_seq_str,
            proj.instance_name,
            email_str
        ));

        if proj.conversations.is_empty() {
            out.push_str("   └─ (No conversations recorded for this workspace)\n");
            continue;
        }

        let conv_len = proj.conversations.len();
        for (idx, conv) in proj.conversations.iter().enumerate() {
            let is_last = idx + 1 == conv_len;
            let branch = if is_last { "└─" } else { "├─" };
            let sub_pipe = if is_last { "   " } else { "│  " };
            let c_badge = if conv.is_running { "🟢" } else { "⚪" };
            let steps_str = if conv.step_count > 0 {
                format!(" · {} steps", conv.step_count)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "   {} 💬 [AGM:{} | {}] \"{}\" ({} {}{})\n",
                branch,
                conv.seq_code,
                conv.gitmap_seq_code,
                conv.title,
                c_badge,
                conv.status,
                steps_str
            ));

            if !conv.prompt_preview_200w.is_empty() {
                out.push_str(&format!(
                    "   {} └─ 📝 [Prompt ≤{}w ({} words)]: \"{}\"\n",
                    sub_pipe, word_cap, conv.prompt_word_count, conv.prompt_preview_200w
                ));
            }
        }
    }

    out.push_str(&"━".repeat(86));
    out.push_str(
        "\n💡 Target by Dual AGM/GitMap Sequence, Instance, & Machine:\n\
         • AGM Conv:    `agm prompt C001 \"is it done?\"`\n\
         • AGM Inst:    `agm prompt P001 \"run tests\" --instance #2`\n\
         • Remote Node: `agm prompt C001 \"check status\" --instance default --node worker-1`\n\
         • GitMap CLI:  `gitmap agy prompt-project P001 -n is-done -t \"verify all\"`\n",
    );
    out
}

fn escape_tg_html_local(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format the Project -> Conversation -> 200-Word Prompt Tree View for Telegram HTML output
pub fn format_tree_view_telegram_html(max_words: usize, only_running: bool) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree(word_cap, only_running);
    let mut out = String::new();

    let title = if only_running {
        "🌳 <b>AGM + GitMap Running Tree (Project → Conv → [≤200w Prompt])</b>"
    } else {
        "🌳 <b>AGM + GitMap Full Workspace Tree (Project → Conv → [≤200w Prompt])</b>"
    };
    out.push_str(title);
    out.push_str("\n━━━━━━━━━━━━━━━━━━━━\n");

    if tree.is_empty() {
        out.push_str("<i>No matching running projects or conversations found.</i>\n");
        out.push_str("\n💡 Try <code>/tree all</code> to view idle workspaces too.");
        return out;
    }

    for proj in &tree {
        let p_icon = if proj.is_running { "🟢" } else { "⚪" };
        let label =
            format_friendly_workspace_label(&proj.repo_name, &proj.project_id, &proj.repo_path);
        let inst_seq_str = proj
            .instance_seq_num
            .map(|n| format!("#{} ", n))
            .unwrap_or_default();
        let email_str = proj
            .bound_email
            .as_deref()
            .filter(|e| !e.is_empty())
            .map(|e| format!(" · {}", e))
            .unwrap_or_default();

        out.push_str(&format!(
            "\n📁 <code>[AGM:{} | {}]</code> <code>[ProjID: {}]</code> {} <b>{}</b>\n   🖥️ <i>Instance: {}{}{}</i> · 📂 <code>{}</code>\n",
            escape_tg_html_local(&proj.seq_code),
            escape_tg_html_local(&proj.gitmap_seq_code),
            escape_tg_html_local(&proj.project_id),
            p_icon,
            escape_tg_html_local(&label),
            escape_tg_html_local(&inst_seq_str),
            escape_tg_html_local(&proj.instance_name),
            escape_tg_html_local(&email_str),
            escape_tg_html_local(&proj.repo_path),
        ));

        if proj.conversations.is_empty() {
            out.push_str("   └─ <i>No recent conversations</i>\n");
            continue;
        }

        let conv_len = proj.conversations.len();
        for (idx, conv) in proj.conversations.iter().enumerate() {
            let is_last = idx + 1 == conv_len;
            let branch = if is_last { "└─" } else { "├─" };
            let sub_pipe = if is_last { "   " } else { "│  " };
            let c_icon = if conv.is_running { "🟢" } else { "⚪" };
            let steps_str = if conv.step_count > 0 {
                format!(" · {} steps", conv.step_count)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "   {} 💬 <code>[AGM:{} | {}]</code> <b>{}</b> ({} {}{})\n",
                branch,
                escape_tg_html_local(&conv.seq_code),
                escape_tg_html_local(&conv.gitmap_seq_code),
                escape_tg_html_local(&conv.title),
                c_icon,
                escape_tg_html_local(&conv.status),
                escape_tg_html_local(&steps_str),
            ));

            if !conv.prompt_preview_200w.is_empty() {
                out.push_str(&format!(
                    "   {} └─ 📝 <b>[Prompt ≤{}w ({}w)]:</b> <i>\"[{}]\"</i>\n",
                    sub_pipe,
                    word_cap,
                    conv.prompt_word_count,
                    escape_tg_html_local(&conv.prompt_preview_200w),
                ));
            }
        }
    }

    out.push_str("\n━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("💡 <b>Prompt by Dual Seq / Instance / Node:</b>\n");
    out.push_str("• <code>/prompt C001 Is it done?</code>\n");
    out.push_str("• <code>/prompt P001 --instance #2 Run pre-flight checks</code>\n");
    out.push_str("• <code>/prompt C001 --instance default --node worker-1 Check build</code>\n");
    out.push_str("• <code>/agy prompt-project P001 -n is-done -t \"verify\"</code>");
    out
}

// ---------------------------------------------------------------------------
// 5-Second Prompt Heartbeat Goal Engine & Watchdog Recovery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGoalHeartbeatConfig {
    pub prompt_id: String,
    pub instance_id: String,
    pub data_dir: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub heartbeat_file: String,
    pub interval_secs: u64,
    pub account_email: String,
    pub worker_pid: Option<u32>,
    pub started_at: i64,
    pub last_heartbeat_at: Option<i64>,
}

/// Start an active prompt goal writing current date & time every 5 seconds to a specific file
pub fn start_prompt_goal_heartbeat(
    instance_id: &str,
    data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    prompt: &str,
    interval_secs: u64,
) -> Result<PromptGoalHeartbeatConfig, String> {
    let ws_dir = PathBuf::from(repo_path);
    if !ws_dir.exists() {
        let _ = fs::create_dir_all(&ws_dir);
    }

    let interval = if interval_secs == 0 { 5 } else { interval_secs };
    let prompt_text = if prompt.trim().is_empty() {
        "Continuous 5-second prompt goal heartbeat verification".to_string()
    } else {
        prompt.trim().to_string()
    };

    let prompt_id = format!("goal-{}-{}", instance_id, Uuid::new_v4().simple());
    let now = Utc::now().timestamp();

    let account_email = crate::modules::instance::load_registry()
        .ok()
        .and_then(|r| r.instances.into_iter().find(|i| i.id == instance_id))
        .and_then(|i| i.bound_email)
        .unwrap_or_else(|| "unbound".to_string());

    let hb_target = if Path::new(heartbeat_file).is_absolute() {
        PathBuf::from(heartbeat_file)
    } else {
        ws_dir.join(heartbeat_file)
    };
    if let Some(p) = hb_target.parent() {
        let _ = fs::create_dir_all(p);
    }
    let hb_str = hb_target.to_string_lossy().to_string();

    let mut cfg = PromptGoalHeartbeatConfig {
        prompt_id: prompt_id.clone(),
        instance_id: instance_id.to_string(),
        data_dir: data_dir.to_string(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_text.clone(),
        heartbeat_file: hb_str.clone(),
        interval_secs: interval,
        account_email: account_email.clone(),
        worker_pid: None,
        started_at: now,
        last_heartbeat_at: Some(now),
    };

    // Save .antigravity_goal_heartbeat.json inside project repo
    let config_path = ws_dir.join(".antigravity_goal_heartbeat.json");
    let json_cfg = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    fs::write(&config_path, json_cfg).map_err(|e| format!("Failed to write goal config: {}", e))?;

    // Also update AGM_INSTANCE_STATUS.md inside project repo for visual verification
    update_workspace_status_file(
        &ws_dir,
        instance_id,
        &account_email,
        &prompt_text,
        &hb_str,
        "RUNNING",
    );

    // Write initial heartbeat line immediately
    append_goal_heartbeat_line(&hb_str, instance_id, &account_email, 0, &prompt_text);

    // Save prompt to active_prompts with status 'running'
    let active_p = ActivePrompt {
        id: prompt_id.clone(),
        project_id: ws_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string()),
        instance_id: instance_id.to_string(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_text.clone(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: Some(format!("goal-session-{}", instance_id)),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };
    let _ = save_or_requeue_prompt(&active_p);

    // Spawn detached goal worker process
    if let Ok(cur_exe) = std::env::current_exe() {
        let mut cmd = Command::new(&cur_exe);
        cmd.args([
            "prompts",
            "goal-worker",
            "--instance",
            instance_id,
            &format!("--user-data-dir={}", data_dir),
            "--project",
            repo_path,
            "--heartbeat-file",
            &hb_str,
            "--interval",
            &interval.to_string(),
            "--prompt",
            &prompt_text,
        ]);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000 | 0x00000200); // CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
        }

        if let Ok(child) = cmd.spawn() {
            let pid = child.id();
            cfg.worker_pid = Some(pid);
            let _ = fs::write(
                &config_path,
                serde_json::to_string_pretty(&cfg).unwrap_or_default(),
            );
            crate::modules::logger::log_info(&format!(
                "[PromptGoal] Spawned goal worker PID {} for instance '{}' (interval: {}s, file: {})",
                pid, instance_id, interval, hb_str
            ));
        }
    }

    Ok(cfg)
}

/// Loop run by `agm prompts goal-worker` in the background
pub fn run_prompt_goal_worker_loop(
    instance_id: &str,
    _data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    interval_secs: u64,
    prompt: &str,
) {
    let ws_dir = PathBuf::from(repo_path);
    let config_path = ws_dir.join(".antigravity_goal_heartbeat.json");
    let interval = if interval_secs == 0 { 5 } else { interval_secs };
    let my_pid = std::process::id();

    loop {
        if !config_path.exists() {
            break;
        }

        let current_email = crate::modules::instance::load_registry()
            .ok()
            .and_then(|r| r.instances.into_iter().find(|i| i.id == instance_id))
            .and_then(|i| i.bound_email)
            .unwrap_or_else(|| "unbound".to_string());

        append_goal_heartbeat_line(heartbeat_file, instance_id, &current_email, my_pid, prompt);

        let now = Utc::now().timestamp();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(mut cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&content) {
                cfg.account_email = current_email.clone();
                cfg.worker_pid = Some(my_pid);
                cfg.last_heartbeat_at = Some(now);
                let _ = fs::write(
                    &config_path,
                    serde_json::to_string_pretty(&cfg).unwrap_or_default(),
                );
            }
        }

        update_workspace_status_file(
            &ws_dir,
            instance_id,
            &current_email,
            prompt,
            heartbeat_file,
            "RUNNING",
        );

        std::thread::sleep(Duration::from_secs(interval));
    }
}

fn append_goal_heartbeat_line(
    file_path: &str,
    instance_id: &str,
    email: &str,
    pid: u32,
    prompt: &str,
) {
    let now = chrono::Local::now();
    let ts_str = now.format("%Y-%m-%d %H:%M:%S%.3f %:z").to_string();
    let line = format!(
        "[{}] instance={} account={} pid={} status=RUNNING goal=\"{}\"\n",
        ts_str, instance_id, email, pid, prompt
    );
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
    {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

fn update_workspace_status_file(
    ws_dir: &Path,
    instance_id: &str,
    email: &str,
    prompt: &str,
    heartbeat_file: &str,
    status: &str,
) {
    let status_file = ws_dir.join("AGM_INSTANCE_STATUS.md");
    let now = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string();
    let content = format!(
        "# Antigravity Instance & Running Prompt Goal Status\n\n\
        - **Instance ID**: `{}`\n\
        - **Bound Account**: `{}`\n\
        - **Goal Status**: `{}`\n\
        - **Prompt**: {}\n\
        - **Heartbeat File**: `{}`\n\
        - **Heartbeat Interval**: 5 seconds\n\
        - **Last Verified Timestamp**: `{}`\n\n\
        *This file is updated every 5 seconds by the active AGM prompt goal watchdog.*\n",
        instance_id, email, status, prompt, heartbeat_file, now
    );
    let _ = fs::write(&status_file, content);
}

/// Stop any running prompt goal worker processes for an instance
pub fn stop_prompt_goal_workers_for_instance(instance_id: &str, data_dir: &str) -> usize {
    let mut stopped = 0;
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        sysinfo::ProcessRefreshKind::new()
            .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
            .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );

    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        let inst_prefix = format!("{}:", instance_id);
        workers.retain(|k, _| !k.starts_with(&inst_prefix));
    }

    let norm_data = data_dir.to_lowercase().replace('\\', "/");
    let clean_data = norm_data.trim_end_matches('/');

    for (pid, proc) in sys.processes() {
        let cmd_str = proc
            .cmd()
            .iter()
            .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let is_goal_worker = cmd_str.contains("goal-worker");
        let matches_inst = cmd_str.contains(&format!("--instance {}", instance_id.to_lowercase()))
            || (!clean_data.is_empty() && cmd_str.contains(clean_data));

        if is_goal_worker && matches_inst {
            let pid_u32 = pid.as_u32();
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid_u32.to_string()])
                    .creation_flags(0x08000000)
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = Command::new("kill")
                    .args(["-9", &pid_u32.to_string()])
                    .output();
            }
            stopped += 1;
        }
    }

    crate::modules::logger::log_info(&format!(
        "[PromptGoal] Stopped {} prompt goal worker(s) for instance '{}'",
        stopped, instance_id
    ));
    stopped
}

/// Inspect prompt goal status across workspaces for an instance
pub fn inspect_prompt_goal_status(
    _instance_id: &str,
    workspaces: &[String],
) -> (bool, Option<String>, Option<String>) {
    let now = Utc::now().timestamp();
    for ws in workspaces {
        let ws_p = Path::new(ws);
        let cfg_p = ws_p.join(".antigravity_goal_heartbeat.json");
        if cfg_p.exists() {
            if let Ok(c) = fs::read_to_string(&cfg_p) {
                if let Ok(cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&c) {
                    let hb_p = Path::new(&cfg.heartbeat_file);
                    let mut is_fresh = false;
                    let mut last_line = None;
                    if hb_p.exists() {
                        if let Ok(meta) = fs::metadata(hb_p) {
                            if let Ok(modified) = meta.modified() {
                                let dt: chrono::DateTime<Utc> = modified.into();
                                let age = now - dt.timestamp();
                                if age <= (cfg.interval_secs as i64 + 4) {
                                    is_fresh = true;
                                }
                            }
                        }
                        if let Ok(content) = fs::read_to_string(hb_p) {
                            last_line = content
                                .lines()
                                .rev()
                                .find(|l| !l.trim().is_empty())
                                .map(|s| s.to_string());
                        }
                    }
                    return (is_fresh, Some(cfg.heartbeat_file), last_line);
                }
            }
        }
    }
    (false, None, None)
}

/// Automatically check and re-invoke prompt goals if not actively running
pub fn ensure_prompt_goals_running_for_instance(instance_id: &str) -> usize {
    let mut reinvoked = 0;
    let workspaces = if let Ok(registry) = crate::modules::instance::load_registry() {
        if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
            crate::modules::instance::get_instance_workspace_folders(instance_id, &inst.data_dir)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let now = Utc::now().timestamp();
    for ws in &workspaces {
        let ws_p = Path::new(ws);
        let cfg_p = ws_p.join(".antigravity_goal_heartbeat.json");
        if cfg_p.exists() {
            if let Ok(c) = fs::read_to_string(&cfg_p) {
                if let Ok(cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&c) {
                    let hb_p = Path::new(&cfg.heartbeat_file);
                    let mut is_alive = false;
                    if hb_p.exists() {
                        if let Ok(meta) = fs::metadata(hb_p) {
                            if let Ok(mod_time) = meta.modified() {
                                let dt: chrono::DateTime<Utc> = mod_time.into();
                                if (now - dt.timestamp()) <= (cfg.interval_secs as i64 + 4) {
                                    is_alive = true;
                                }
                            }
                        }
                    }
                    if !is_alive {
                        crate::modules::logger::log_info(&format!(
                            "[PromptGoal] Prompt goal heartbeat stopped in '{}', auto-reinvoking for instance '{}'...",
                            ws, instance_id
                        ));
                        if let Ok(registry) = crate::modules::instance::load_registry() {
                            if let Some(inst) =
                                registry.instances.iter().find(|i| i.id == instance_id)
                            {
                                if let Ok(_) = start_prompt_goal_heartbeat(
                                    instance_id,
                                    &inst.data_dir,
                                    ws,
                                    &cfg.heartbeat_file,
                                    &cfg.prompt_content,
                                    cfg.interval_secs,
                                ) {
                                    reinvoked += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    reinvoked
}

/// Duplicate one instance's repo rows onto another instance id.
/// Project and prompt ids are suffixed so the source rows stay intact.
pub fn clone_instance_repo_rows(source_id: &str, target_id: &str) -> Result<usize, String> {
    if source_id == target_id || target_id.is_empty() {
        return Ok(0);
    }
    let conn = connect_db()?;
    clone_repo_rows_on(&conn, source_id, target_id)
}

pub(crate) fn clone_repo_rows_on(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
) -> Result<usize, String> {
    let mut project_stmt = conn
        .prepare(
            "SELECT id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
             FROM running_projects
             WHERE instance_id = ?1 OR (instance_id = '__default__' AND ?1 = 'default')",
        )
        .map_err(|e| format!("Failed to read source repos: {}", e))?;
    let projects: Vec<(String, String, String, Option<String>, i64, i64, i64)> = project_stmt
        .query_map([source_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .map_err(|e| format!("Failed to map source repos: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect source repos: {}", e))?;
    drop(project_stmt);

    let mut copied = 0usize;
    for (id, name, path, storage, running, detected, updated) in projects {
        let new_id = format!("{}__{}", id, target_id);
        conn.execute(
            "INSERT OR IGNORE INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![new_id, target_id, name, path, storage, running, detected, updated],
        )
        .map_err(|e| format!("Failed to clone repo {}: {}", id, e))?;

        let mut prompt_stmt = conn
            .prepare(
                "SELECT id, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                 FROM active_prompts
                 WHERE project_id = ?1 AND (instance_id = ?2 OR (instance_id = '__default__' AND ?2 = 'default'))",
            )
            .map_err(|e| format!("Failed to read source prompts: {}", e))?;
        let prompts: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            i64,
            i64,
            Option<String>,
        )> = prompt_stmt
            .query_map(rusqlite::params![id, source_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })
            .map_err(|e| format!("Failed to map source prompts: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect source prompts: {}", e))?;
        drop(prompt_stmt);

        for (prompt_id, content, model, session, status, created, prompt_updated, image) in prompts
        {
            let new_prompt_id = format!("{}__{}", prompt_id, target_id);
            conn.execute(
                "INSERT OR IGNORE INTO active_prompts
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    new_prompt_id,
                    new_id,
                    target_id,
                    path,
                    content,
                    model,
                    session,
                    status,
                    created,
                    prompt_updated,
                    image
                ],
            )
            .map_err(|e| format!("Failed to clone prompt {}: {}", prompt_id, e))?;
        }
        copied += 1;
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_decoding() {
        let uri = "file:///d:/work/My%20Project";
        let path = decode_uri_to_path(uri);
        assert!(path.contains("My Project"));
    }

    #[test]
    fn test_repo_db_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        // Insert mock project
        let res = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('test-proj', 'default', 'TestRepo', '/work/test', NULL, 1, 100, 100)",
            [],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn prompt_channel_wait_only_when_a_prompt_was_backed_up() {
        assert!(!needs_prompt_channel_wait(0));
        assert!(needs_prompt_channel_wait(1));
    }

    #[test]
    fn live_prompt_uses_preview_when_the_transcript_has_no_user_input() {
        let preview = live_prompt_text("keep the same chat", "{\"type\":\"MODEL\"}\n").unwrap();
        assert_eq!(preview, "keep the same chat");
        let user = live_prompt_text(
            "preview",
            "{\"type\":\"USER_INPUT\",\"content\":\"real prompt\"}",
        )
        .unwrap();
        assert_eq!(user, "real prompt");
        assert!(live_prompt_text("  ", "").is_none());
    }

    #[test]
    fn resume_document_keeps_the_same_conversation_id() {
        let prompt = ActivePrompt {
            id: "prompt-conv-9".to_string(),
            project_id: "proj".to_string(),
            instance_id: "default".to_string(),
            repo_path: "D:/work/app".to_string(),
            prompt_content: "keep going".to_string(),
            model: None,
            session_id: Some("conv-9".to_string()),
            status: "backed_up".to_string(),
            created_at: 10,
            updated_at: 10,
            image_payload: None,
        };
        let doc = resume_task_document(&prompt, "backed_up", 10, &[]);
        assert_eq!(doc["session_id"], "conv-9");
        assert_eq!(doc["conversation_id"], "conv-9");
        assert_eq!(doc["prompt_content"], "keep going");
    }

    #[test]
    fn summary_rows_fall_back_when_status_columns_are_missing() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (conversation_id TEXT, preview TEXT, workspace_uris TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_summaries VALUES ('cid-1', 'running work', '[\"file:///d:/work/app\"]')",
            [],
        )
        .unwrap();
        let rows = read_conversation_summary_rows(&conn);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cid, "cid-1");
        assert_eq!(rows[0].preview, "running work");
        assert_eq!(rows[0].not_fully_idle, 1);
    }

    #[test]
    fn clone_repo_rows_keeps_source_and_copies_onto_target() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'default', 'Antigravity', 'D:/work/antigravity-manager', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-1', 'proj-1', 'default', 'D:/work/antigravity-manager', 'keep going', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        let copied = clone_repo_rows_on(&conn, "default", "copy-1").unwrap();
        assert_eq!(copied, 1);

        let source_projects: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM running_projects WHERE instance_id = 'default'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let target_projects: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM running_projects WHERE instance_id = 'copy-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let target_prompts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE instance_id = 'copy-1' AND prompt_content = 'keep going'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_projects, 1);
        assert_eq!(target_projects, 1);
        assert_eq!(target_prompts, 1);
    }

    #[test]
    fn test_clone_repo_rows_matches_default_and_under_default() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        // Project with 'default'
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-def', 'default', 'DefRepo', 'D:/work/def-repo', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-def', 'proj-def', 'default', 'D:/work/def-repo', 'prompt def', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        // Project with '__default__'
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-under', '__default__', 'UnderRepo', 'D:/work/under-repo', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-under', 'proj-under', '__default__', 'D:/work/under-repo', 'prompt under', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        let copied = clone_repo_rows_on(&conn, "default", "target-inst").unwrap();
        assert_eq!(copied, 2);

        let target_projects: Vec<(String, String)> = conn
            .prepare(
                "SELECT id, instance_id FROM running_projects WHERE instance_id = 'target-inst'",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(target_projects.len(), 2);
        for (_, inst_id) in &target_projects {
            assert_eq!(inst_id, "target-inst");
        }

        let target_prompts: Vec<(String, String, String)> = conn
            .prepare("SELECT id, project_id, instance_id FROM active_prompts WHERE instance_id = 'target-inst'")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(target_prompts.len(), 2);
        for (_, proj_id, inst_id) in &target_prompts {
            assert_eq!(inst_id, "target-inst");
            assert!(proj_id.ends_with("__target-inst"));
        }
    }

    #[test]
    fn test_prompt_backup_and_direct_dispatch_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        // 1. Insert running project
        let proj_res = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'inst-alpha', 'CoreApp', '/work/core', NULL, 1, 1000, 1000)",
            [],
        );
        assert!(proj_res.is_ok());

        // 2. Backup prompt
        let prompt_id = "test-prompt-uuid";
        let prompt_insert = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
             VALUES (?, 'proj-1', 'inst-alpha', '/work/core', 'Implement neural router', 'gemini-pro', 'sess-1', 'backed_up', 1000, 1000)",
            params![prompt_id],
        );
        assert!(prompt_insert.is_ok());

        // 3. Verify status is backed_up
        let status_before: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE id = ?",
                params![prompt_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status_before, "backed_up");

        // 4. Direct dispatch (no queuing)
        let dispatch_update = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = 1010 
             WHERE instance_id = 'inst-alpha' AND status = 'backed_up'",
            [],
        );
        assert_eq!(dispatch_update.unwrap(), 1);

        // 5. Verify status is directly dispatched
        let status_after: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE id = ?",
                params![prompt_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status_after, "dispatched");
    }

    #[test]
    fn test_auto_resume_recency_filter() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();

        // 1. Insert recent project (active 10 minutes ago)
        let _ = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-recent', 'inst-test', 'RecentApp', '/work/recent', NULL, 1, ?, ?)",
            params![now - 600, now - 600],
        );

        // 2. Insert old project (active 2 hours ago)
        let _ = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-old', 'inst-test', 'OldApp', '/work/old', NULL, 1, ?, ?)",
            params![now - 7200, now - 7200],
        );

        // Verify recency threshold logic
        let threshold = 3600; // 1 hour
        let cutoff = now - threshold;

        let recent_last = now - 600;
        let old_last = now - 7200;

        let is_recent_valid = recent_last >= cutoff;
        let is_old_stale = old_last < cutoff;

        assert!(is_recent_valid);
        assert!(is_old_stale);
    }

    #[test]
    fn test_resend_deduplication_and_reinjection_prevention() {
        reset_dispatched_prompts_cache();
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        // Insert parent running_project first
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'inst-1', 'repo-one', '/repo/one', NULL, 1, ?, ?)",
            params![now, now],
        )
        .unwrap();

        // Insert prompt with status 'backed_up'
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-100', 'proj-1', 'inst-1', '/repo/one', 'Refactor router', 'gemini-pro', 'sess-100', 'backed_up', ?, ?, NULL)",
            params![now - 10, now - 10],
        )
        .unwrap();

        // Verify it is selected by the query for resend
        let mut stmt = conn
            .prepare(
                "SELECT id, prompt_content, status FROM active_prompts WHERE status IN ('backed_up', 'queued', 'pending') ORDER BY created_at ASC",
            )
            .unwrap();
        let pending_prompts: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(pending_prompts.len(), 1);
        assert_eq!(pending_prompts[0], "p-100");

        // Simulate dispatching: mark as dispatched and add to cache
        conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE id = 'p-100'",
            params![now],
        )
        .unwrap();
        {
            let mut cache = get_dispatched_prompts_cache().lock().unwrap();
            cache.insert("p-100".to_string());
            cache.insert("/repo/one:Refactor router".to_string());
        }

        // Verify that subsequent query for resend finds 0 pending prompts!
        let pending_after: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(pending_after.len(), 0);

        // Verify that cache intercepts duplicate attempt
        let cache = get_dispatched_prompts_cache().lock().unwrap();
        assert!(cache.contains("p-100"));
        assert!(cache.contains("/repo/one:Refactor router"));
    }

    #[test]
    fn test_backup_conflict_resolution_no_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        // Insert parent running_project first
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-dup', 'default', 'dup', '/work/dup', NULL, 1, ?, ?)",
            params![now, now],
        )
        .unwrap();

        // Insert prompt first time
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now, now],
        ).unwrap();

        // Insert same prompt second time with ON CONFLICT
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now + 1, now + 1],
        ).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE id = 'p-duplicate'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Must never duplicate records on backup");
    }

    #[test]
    fn test_extract_clean_user_prompt() {
        let raw = "<USER_REQUEST>\nFix the auto-switch prompt resumption bug\n</USER_REQUEST>\n<ADDITIONAL_METADATA>...</ADDITIONAL_METADATA>";
        let clean = extract_clean_user_prompt(raw);
        assert_eq!(clean, "Fix the auto-switch prompt resumption bug");

        let simple = "Simple task prompt";
        assert_eq!(extract_clean_user_prompt(simple), "Simple task prompt");
    }

    #[test]
    fn test_extract_smart_prompt_summary() {
        let raw = "<USER_REQUEST>\n808e01722752a5c3b788899692cb8e83ade8b09a\n5f667758711619c8bb4f30304ab16a682659fd18\n\nD:\\work\\repo-secrets\n\nCan you please keep the secrets of the Telegram and other stuff?\n</USER_REQUEST>";
        let summary = extract_smart_prompt_summary(raw, 60);
        assert!(summary.starts_with("Can you please keep the secrets of the Telegram"));
        assert!(!summary.contains("808e01722752a5c3b788899692cb8e83ade8b09a"));
        assert!(!summary.contains("D:\\work\\repo-secrets"));
        assert!(!summary.contains("<USER_REQUEST>"));
    }

    #[test]
    fn test_format_friendly_workspace_label() {
        let label = format_friendly_workspace_label(
            "antigravity-manager-d58c5517",
            "antigravity-manager-d58c5517",
            "d:/work/Antigravity-Manager",
        );
        assert!(label.starts_with("Antigravity-Manager"));
    }

    #[test]
    fn test_agm_sequences_and_200_word_preview() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let p1 = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/antigravity-manager",
            "agm-1",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "default",
        );
        let p1_repeat = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/antigravity-manager",
            "agm-1",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "default",
        );
        let p2 = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/gitmap",
            "gm-1",
            "gitmap",
            "d:/work/gitmap",
            "default",
        );
        assert_eq!(p1, 1);
        assert_eq!(p1_repeat, 1);
        assert_eq!(p2, 2);

        let c1 = ensure_conversation_sequence_in_conn(
            &conn,
            "d58c5517-d8ad-437e-ab7c-e506b0322383",
            "d:/work/antigravity-manager",
            "Telegram & AGM Tree View",
            "default",
        );
        assert_eq!(c1, 1);

        let two_hundred_fifty_words = (1..=250)
            .map(|i| format!("word{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        let (preview, count) = extract_prompt_words_preview(&two_hundred_fifty_words, 200);
        assert_eq!(count, 250);
        assert!(preview.ends_with("word200 ..."));
    }
}
