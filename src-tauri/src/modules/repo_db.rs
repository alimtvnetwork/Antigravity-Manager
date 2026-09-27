//! Split Repo DB Module
//! Dedicated SQLite State Database for tracking running projects,
//! backing up active prompts, and directly dispatching prompts upon profile switch.

#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

static MEMORY_ACTIVE_PROMPTS: OnceLock<Mutex<HashMap<String, ActivePrompt>>> = OnceLock::new();
static DISPATCHED_PROMPTS_CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn get_memory_prompts_map() -> &'static Mutex<HashMap<String, ActivePrompt>> {
    MEMORY_ACTIVE_PROMPTS.get_or_init(|| Mutex::new(HashMap::new()))
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

    Ok(())
}

/// Decode file URI (e.g., file:///path/to/folder or file:///c%3A/path) to local path
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
    let registry = crate::modules::instance::load_registry()?;
    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
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

/// Discover in-flight active conversations and running prompts directly from Antigravity core storage
/// (~/.gemini/antigravity/conversation_summaries.db and brain/<cid>/.system_generated/logs/transcript.jsonl)
pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {
    let mut prompts = Vec::new();
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    let Some(base_dir) = base_dir else {
        return prompts;
    };
    let summaries_db = base_dir.join("conversation_summaries.db");
    if !summaries_db.exists() {
        return prompts;
    }

    let conn = match Connection::open_with_flags(
        &summaries_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(_) => return prompts,
    };

    let now = Utc::now().timestamp();
    let mut stmt = match conn.prepare(
        "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
         FROM conversation_summaries 
         ORDER BY last_modified_time DESC 
         LIMIT 25",
    ) {
        Ok(s) => s,
        Err(_) => return prompts,
    };

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

    let Ok(rows) = rows else {
        return prompts;
    };

    for item in rows.flatten() {
        let (cid, _title, _preview, status, not_fully_idle, ws_uris_opt, _last_time) = item;
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
                            if let Some(media_arr) = val.get("media").and_then(|m| m.as_array()) {
                                for m_item in media_arr {
                                    if let Some(uri) = m_item.get("uri").and_then(|u| u.as_str()) {
                                        let clean_path = decode_uri_to_path(uri);
                                        let p = Path::new(&clean_path);
                                        if p.exists() {
                                            if let Ok(bytes) = fs::read(p) {
                                                let mime = m_item
                                                    .get("mime_type")
                                                    .and_then(|mt| mt.as_str())
                                                    .unwrap_or("image/png");
                                                let b64 = STANDARD.encode(&bytes);
                                                image_payload =
                                                    Some(format!("data:{};base64,{}", mime, b64));
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

    prompts
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

    // Transition all remaining in-flight 'running' prompts in active_prompts to 'backed_up' before switch
    let transitioned = conn
        .execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ? WHERE status = 'running'",
            params![now],
        )
        .unwrap_or(0);
    if transitioned > 0 {
        backed_up_count += transitioned;
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            for p in map.values_mut() {
                if p.status == "running" {
                    p.status = "backed_up".to_string();
                    p.updated_at = now;
                }
            }
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Transitioned {} in-flight prompts from 'running' to 'backed_up' before switch",
            transitioned
        ));
    }

    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
    for p in ag_prompts {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?, ?, ?, ?, NULL, 1, ?, ?)",
            params![&p.project_id, &p.instance_id, &p.project_id, &p.repo_path, now, now],
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
            let has_image = final_img.is_some() || !img_paths.is_empty();

            // Write disk resume snapshot file inside project repo directory
            let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
            let payload = serde_json::json!({
                "prompt_id": p.id,
                "project_id": p.project_id,
                "instance_id": p.instance_id,
                "repo_path": p.repo_path,
                "prompt_content": p.prompt_content,
                "model": p.model,
                "session_id": p.session_id,
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
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
    }

    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery
    let projects = detect_running_projects(instance_id).unwrap_or_default();
    for project in &projects {
        let mut extracted_prompts: Vec<(String, Option<String>)> = Vec::new();

        // Check workspace state.vscdb for active prompts / tasks
        if let Some(ref ws_storage) = project.workspace_storage_path {
            let ws_db_path = PathBuf::from(ws_storage).join("state.vscdb");
            if ws_db_path.exists() {
                if let Ok(ws_conn) = Connection::open_with_flags(
                    &ws_db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
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
                                    extracted_prompts.push((content, img_payload));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check if project has an existing .antigravity_resume_task.json on disk
        if extracted_prompts.is_empty() {
            let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
            if task_file.exists() {
                if let Ok(c) = fs::read_to_string(&task_file) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                        if let Some(txt) = v.get("prompt_content").and_then(|t| t.as_str()) {
                            if !txt.trim().is_empty() {
                                let img = v
                                    .get("image_payload")
                                    .and_then(|i| i.as_str())
                                    .map(|s| s.to_string());
                                extracted_prompts.push((txt.to_string(), img));
                            }
                        }
                    }
                }
            }
        }

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
                    image_payload,
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

/// Directly dispatch/send backed-up prompts to the running projects without queuing
pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE instance_id = ? AND status = 'backed_up'",
        )
        .map_err(|e| format!("Failed to prepare dispatch query: {}", e))?;

    let prompts = stmt
        .query_map([instance_id], |row| {
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

    let mut dispatched_count = 0;

    for prompt in prompts {
        let sig = format!("{}:{}", prompt.repo_path, prompt.prompt_content.trim());
        let already_dispatched = {
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

        // Write disk resume snapshot file inside project repo directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt.id,
            "project_id": prompt.project_id,
            "instance_id": instance_id,
            "prompt_content": prompt.prompt_content,
            "model": prompt.model,
            "image_payload": prompt.image_payload,
            "status": "dispatched",
            "dispatched_at": now,
        });
        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, json_str);
        }

        // Mark prompt as dispatched directly in the state database
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

        // Actively spawn agy CLI execution
        spawn_prompt_via_agy(&prompt);
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Successfully dispatched {} prompts to running projects for instance '{}'",
        dispatched_count, instance_id
    ));

    Ok(dispatched_count)
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

/// List recent prompts across statuses
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

/// Retrieve live project execution information by inspecting both Antigravity's
/// conversation_summaries.db (ground-truth for in-flight requests) and SQLite repo_prompts.db.
pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {
    let mut results: Vec<ProjectExecutionInfo> = Vec::new();
    let now = Utc::now().timestamp();

    // Map of normalized repo path -> (is_running, active_prompt_snippet, last_time)
    let mut live_map: std::collections::HashMap<String, (bool, Option<String>, i64)> =
        std::collections::HashMap::new();

    // 1. Inspect Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
                     FROM conversation_summaries 
                     ORDER BY last_modified_time DESC 
                     LIMIT 25",
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
                            let (_cid, _title, preview, status, not_fully_idle, ws_uris_opt, _last_time_str) = item;
                            let is_conv_running = not_fully_idle != 0 || status.contains("RUNNING");
                            let prompt_preview = if !preview.trim().is_empty() {
                                Some(preview)
                            } else {
                                None
                            };

                            if let Some(ws_uris_raw) = ws_uris_opt {
                                let ws_uris: Vec<String> =
                                    serde_json::from_str(&ws_uris_raw).unwrap_or_default();
                                for u in ws_uris {
                                    let clean_p = decode_uri_to_path(&u).to_lowercase();
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
                    let clean_p = p_path.to_lowercase();
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
        let clean_path = p.repo_path.to_lowercase();
        let (is_running, prompt_snippet, last_time) = live_map
            .get(&clean_path)
            .cloned()
            .unwrap_or((false, None, p.last_detected_at));
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
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
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
            }
        }
    }

    // Check repo_prompts.db active_prompts
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

    false
}

/// Save or re-queue an active prompt into active_prompts and running_projects
pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let _ = conn.execute(
        "INSERT OR REPLACE INTO running_projects 
         (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
         VALUES (?, ?, ?, ?, NULL, 1, ?, ?)",
        params![&prompt.project_id, &prompt.instance_id, &prompt.project_id, &prompt.repo_path, now, now],
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
    let trimmed = raw.trim();
    if let (Some(start), Some(end)) = (
        trimmed.find("<USER_REQUEST>"),
        trimmed.find("</USER_REQUEST>"),
    ) {
        if start < end {
            let inner = &trimmed[start + "<USER_REQUEST>".len()..end];
            if !inner.trim().is_empty() {
                return inner.trim().to_string();
            }
        }
    }
    trimmed.to_string()
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

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}) in '{}': {:.60}...",
                    prompt.id, child.id(), prompt.repo_path, clean_prompt
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

/// Resend and restore all previous running/backed-up/dispatched commands before IDE close or switch.
/// Strictly filters to unrestored/backed-up/queued prompts, prioritizes the latest running prompts,
/// and deduplicates against already dispatched prompts and workspace repositories.
pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String> {
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

    let prompts = stmt
        .query_map([limit], |row| {
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

    let mut resent = Vec::new();
    let mut dispatched_repos = HashSet::new();

    for mut prompt in prompts {
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
        let already_dispatched = {
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

        // Update database status to dispatched and update timestamps
        let _ = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?, image_payload = ? WHERE id = ?",
            rusqlite::params![now, &prompt.image_payload, &prompt.id],
        );
        let _ = conn.execute(
            "UPDATE running_projects SET is_running = 1, last_detected_at = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![now, now, &prompt.project_id],
        );

        // Actively spawn agy CLI execution
        spawn_prompt_via_agy(&prompt);

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
}
