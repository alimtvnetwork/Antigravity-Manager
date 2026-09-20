//! Split Repo DB Module
//! Dedicated SQLite State Database for tracking running projects,
//! backing up active prompts, and directly dispatching prompts upon profile switch.

#![allow(dead_code)]

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

static MEMORY_ACTIVE_PROMPTS: OnceLock<Mutex<HashMap<String, ActivePrompt>>> = OnceLock::new();

fn get_memory_prompts_map() -> &'static Mutex<HashMap<String, ActivePrompt>> {
    MEMORY_ACTIVE_PROMPTS.get_or_init(|| Mutex::new(HashMap::new()))
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
            FOREIGN KEY(project_id) REFERENCES running_projects(id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create active_prompts table: {}", e))?;

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

/// Backup all currently running prompts across active projects before switching
pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {
    let projects = detect_running_projects(instance_id)?;
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut backed_up_count = 0;

    for project in &projects {
        let is_proj_running = project.is_running;
        if !is_proj_running {
            continue;
        }

        // Check workspace state.vscdb for active prompts / tasks
        let mut extracted_prompts = Vec::new();
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
                                    extracted_prompts.push(content);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If no raw prompt string was extracted from workspace DB, create a project task snapshot
        if extracted_prompts.is_empty() {
            let fallback_snapshot = format!(
                "Active project snapshot for '{}' [{}] before instance rotation at {}",
                project.repo_name, project.repo_path, now
            );
            extracted_prompts.push(fallback_snapshot);
        }

        for prompt_text in extracted_prompts {
            let prompt_id = Uuid::new_v4().to_string();
            let prompt_model = Some("gemini-pro".to_string());
            let result = conn.execute(
                "INSERT INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'backed_up', ?, ?)",
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
                ],
            );

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
                };
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Backed up {} running prompts for instance '{}'",
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
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at 
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
            })
        })
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect::<Vec<ActivePrompt>>();

    let mut dispatched_count = 0;

    for prompt in prompts {
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
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at 
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
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at 
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
            })
        })
        .map_err(|e| format!("Failed to query prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
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
}
