//! "Now Running + Up Next" IPC command (Task-01, spec §6).
//!
//! Returns the prompt currently running plus the next up-to-4 queued prompts
//! (FIFO) for an instance, so the prompt tree view can render the live
//! "now running" row and the "up next" queue without polling the full tree.
//!
//! Registration follow-up (not owned by Task-01):
//! - add `mod cmd_running_up_next;` + `pub use cmd_running_up_next::*;`
//!   to `src-tauri/src/commands/mod.rs`
//! - add `commands::get_running_and_up_next,` to the `invoke_handler!` list
//!   in `src-tauri/src/lib/run.rs`

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// One row of the Now Running + Up Next panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptQueueRow {
    /// The prompt `Id` (shown on every test row per the task contract).
    pub id: String,
    pub instance_id: String,
    pub status: String,
    /// 4-tier origin classification (`USER_PROMPT` / `SUBAGENT_INSTRUCTION` /
    /// `SYSTEM_MESSAGE` / `TOOL_OUTPUT`). The backend does not track origin;
    /// defaults to `USER_PROMPT`. Subtask 02 owns frontend refinement.
    pub origin_tier: String,
    pub created_at: i64,
    pub has_images: bool,
}

/// Payload for `get_running_and_up_next` (spec §6.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningAndUpNext {
    /// The now-running prompt (status in 'running'/'dispatched', FIFO earliest).
    pub now_running: Option<PromptQueueRow>,
    /// Up to 4 queued prompts (status in 'queued'/'pending', FIFO).
    pub up_next: Vec<PromptQueueRow>,
    /// Full queued count ("four or whatever the queue holds").
    pub up_next_total: i64,
    pub instance_id: String,
    /// Backend timestamp (ms); UI uses it for staleness display.
    pub as_of_ms: i64,
}

fn map_queue_row(row: &rusqlite::Row) -> rusqlite::Result<PromptQueueRow> {
    Ok(PromptQueueRow {
        id: row.get(0)?,
        instance_id: row.get(1)?,
        status: row.get(2)?,
        origin_tier: "USER_PROMPT".to_string(),
        created_at: row.get(3)?,
        has_images: row.get(4)?,
    })
}

/// Instance predicate shared by the three queries.
/// Returns `(where_sql, param)`; `param` is `None` for the default scope when no resolved ID is needed.
fn instance_predicate(target: &str) -> (&'static str, Option<String>) {
    let resolved = crate::modules::instance::resolve_instance_id(target).unwrap_or_else(|e| {
        // Justification: fallback to literal target when instance registry cannot resolve ID
        tracing::debug!(target: "prompt_queue", target_id = target, error = %e, "resolve_instance_id fallback to target literal");
        target.to_string()
    });

    let is_default = crate::modules::repo_db::gemini_dirs::is_default_instance_match(target)
        || crate::modules::repo_db::gemini_dirs::is_default_instance_match(&resolved);

    if is_default {
        if resolved != "default" && resolved != "__default__" && !resolved.trim().is_empty() {
            (
                "(instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL OR instance_id = ?1)",
                Some(resolved),
            )
        } else {
            (
                "(instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL)",
                None,
            )
        }
    } else {
        ("instance_id = ?1", Some(resolved))
    }
}

fn query_now_running(conn: &Connection, target: &str) -> Result<Option<PromptQueueRow>, String> {
    let (pred, param) = instance_predicate(target);
    let sql = format!(
        "SELECT id, instance_id, status, created_at, image_payload IS NOT NULL \
         FROM active_prompts \
         WHERE status IN ('running', 'dispatched') AND {pred} \
         ORDER BY created_at ASC, id ASC LIMIT 1"
    );
    let result = match param {
        Some(p) => conn.query_row(&sql, params![p], map_queue_row),
        None => conn.query_row(&sql, [], map_queue_row),
    };
    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to query running prompt: {e}")),
    }
}

fn query_up_next(conn: &Connection, target: &str) -> Result<Vec<PromptQueueRow>, String> {
    let (pred, param) = instance_predicate(target);
    let sql = format!(
        "SELECT id, instance_id, status, created_at, image_payload IS NOT NULL \
         FROM active_prompts \
         WHERE status IN ('queued', 'pending') AND {pred} \
         ORDER BY created_at ASC, id ASC LIMIT 4"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare up-next query: {e}"))?;
    let rows = match param {
        Some(p) => stmt.query_map(params![p], map_queue_row),
        None => stmt.query_map([], map_queue_row),
    }
    .map_err(|e| format!("Failed to query up-next prompts: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read up-next rows: {e}"))
}

fn query_up_next_total(conn: &Connection, target: &str) -> Result<i64, String> {
    let (pred, param) = instance_predicate(target);
    let sql = format!(
        "SELECT COUNT(*) FROM active_prompts \
         WHERE status IN ('queued', 'pending') AND {pred}"
    );
    let total: i64 = match param {
        Some(p) => conn.query_row(&sql, params![p], |r| r.get(0)),
        None => conn.query_row(&sql, [], |r| r.get(0)),
    }
    .map_err(|e| format!("Failed to count queued prompts: {e}"))?;
    Ok(total)
}

/// Return the currently-running prompt and the next up-to-4 queued prompts
/// (FIFO) for an instance, plus the full queued count and a backend
/// timestamp. Powers the prompt tree "Now Running + Up Next" panel.
#[tauri::command]
pub fn get_running_and_up_next(instance_id: Option<String>) -> Result<RunningAndUpNext, String> {
    let raw_target = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    let target = crate::modules::instance::observe::normalize_instance_specifier(raw_target);

    let conn = crate::modules::repo_db::schema::connect_db()
        .map_err(|e| format!("Failed to open repo DB: {e}"))?;

    let mut now_running = query_now_running(&conn, target)?;

    // Heartbeat scan fallback for active IDE sessions if SQLite active_prompts has no running row
    if now_running.is_none() {
        let resolved = crate::modules::instance::resolve_instance_id(target)
            .unwrap_or_else(|_| target.to_string());
        let data_dir = if crate::modules::repo_db::gemini_dirs::is_default_instance_match(&resolved)
        {
            crate::modules::instance::get_default_antigravity_data_dir()
        } else if let Ok(reg) = crate::modules::instance::load_registry() {
            reg.instances
                .iter()
                .find(|i| i.id == resolved)
                .map(|i| std::path::PathBuf::from(&i.data_dir))
                .unwrap_or_else(crate::modules::instance::get_default_antigravity_data_dir)
        } else {
            crate::modules::instance::get_default_antigravity_data_dir()
        };
        let workspaces = crate::modules::instance::get_instance_workspace_folders(
            &resolved,
            &data_dir.to_string_lossy(),
        );
        let (is_running, _hb_file, last_line) =
            crate::modules::repo_db::inspect_prompt_goal_status(&resolved, &workspaces);

        if is_running {
            let prompt_id = last_line
                .as_deref()
                .and_then(|line| {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        val.get("prompt_id")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "live-ide-prompt".to_string());

            now_running = Some(PromptQueueRow {
                id: prompt_id,
                instance_id: target.to_string(),
                status: "running".to_string(),
                origin_tier: "USER_PROMPT".to_string(),
                created_at: Utc::now().timestamp(),
                has_images: false,
            });
        }
    }

    let up_next = query_up_next(&conn, target)?;
    let up_next_total = query_up_next_total(&conn, target)?;

    Ok(RunningAndUpNext {
        now_running,
        up_next,
        up_next_total,
        instance_id: target.to_string(),
        as_of_ms: Utc::now().timestamp_millis(),
    })
}
