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
    /// The now-running prompt (status = `running`, FIFO earliest).
    pub now_running: Option<PromptQueueRow>,
    /// Up to 4 queued prompts (status in `queued`/`pending`/`dispatched`, FIFO).
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
/// Returns `(where_sql, param)`; `param` is `None` for the default scope.
fn instance_predicate(target: &str) -> (&'static str, Option<String>) {
    if target == "default" || target == "__default__" {
        (
            "(instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL)",
            None,
        )
    } else {
        ("instance_id = ?1", Some(target.to_string()))
    }
}

fn query_now_running(conn: &Connection, target: &str) -> Result<Option<PromptQueueRow>, String> {
    let (pred, param) = instance_predicate(target);
    let sql = format!(
        "SELECT id, instance_id, status, created_at, image_payload IS NOT NULL \
         FROM active_prompts \
         WHERE status = 'running' AND {pred} \
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
         WHERE status IN ('queued', 'pending', 'dispatched') AND {pred} \
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
         WHERE status IN ('queued', 'pending', 'dispatched') AND {pred}"
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
    let target = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");

    let conn = crate::modules::repo_db::schema::connect_db()
        .map_err(|e| format!("Failed to open repo DB: {e}"))?;

    let now_running = query_now_running(&conn, target)?;
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
