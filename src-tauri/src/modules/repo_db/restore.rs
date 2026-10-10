//! Repo DB: restore

use super::dispatch::parse_flexible_timestamp;
use super::models::{ActivePrompt, SwitchPromptSnap};
use super::schema::connect_db;
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;

/// List all backed up prompts
pub fn list_backed_up_prompts() -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts WHERE status = 'backed_up' ORDER BY created_at ASC, id ASC",
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

/// Restore prompts from a JSON backup file in strict FIFO order, persisting into active_prompts and refreshing the tree cache
pub fn restore_prompts_from_backup_json(backup_json: &str) -> Result<usize, String> {
    let val: serde_json::Value = serde_json::from_str(backup_json)
        .map_err(|e| format!("Invalid JSON backup format: {}", e))?;

    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut restored_count = 0;

    // 1. If activePrompts array is present:
    if let Some(prompts_arr) = val.get("activePrompts").and_then(|v| v.as_array()) {
        let mut sorted_prompts = prompts_arr.clone();
        sorted_prompts.sort_by_key(|p| p.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0));

        for p in sorted_prompts {
            let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let content = p
                .get("prompt_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if id.is_empty() || content.trim().is_empty() {
                continue;
            }
            let project_id = p
                .get("project_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let instance_id = p
                .get("instance_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let repo_path = p.get("repo_path").and_then(|v| v.as_str()).unwrap_or("");
            let model = p.get("model").and_then(|v| v.as_str()).unwrap_or("gemini");
            let session_id = p.get("session_id").and_then(|v| v.as_str());
            let created_at = p.get("created_at").and_then(|v| v.as_i64()).unwrap_or(now);
            let img = p.get("image_payload").and_then(|v| v.as_str());

            // Justification: restore-from-backup insert is best-effort; the loop continues with the remaining entries
            crate::error::record_ignored(
                conn.execute(
                "INSERT OR IGNORE INTO running_projects (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, 0, ?5)",
                rusqlite::params![project_id, instance_id, project_id, repo_path, now],
            ),
                "restore running project from backup",
            );

            // Justification: restore-from-backup insert is best-effort; the loop continues with the remaining entries
            crate::error::record_ignored(
                conn.execute(
                "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                    prompt_content = excluded.prompt_content,
                    status = 'backed_up',
                    updated_at = excluded.updated_at",
                rusqlite::params![id, project_id, instance_id, repo_path, content, model, session_id, created_at, now, img],
            ),
                "restore backed-up prompt from backup",
            );
            restored_count += 1;
        }
    }

    // 2. If projects array is present:
    if let Some(projects_arr) = val.get("projects").and_then(|v| v.as_array()) {
        for proj in projects_arr {
            let proj_id = proj
                .get("project_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let repo_name = proj.get("repo_name").and_then(|v| v.as_str()).unwrap_or("");
            let repo_path = proj.get("repo_path").and_then(|v| v.as_str()).unwrap_or("");
            let inst_id = proj
                .get("instance_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            // Justification: restore-from-backup insert is best-effort; the loop continues with the remaining entries
            crate::error::record_ignored(
                conn.execute(
                "INSERT OR IGNORE INTO running_projects (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, 0, ?5)",
                rusqlite::params![proj_id, inst_id, repo_name, repo_path, now],
            ),
                "restore running project from backup",
            );

            if let Some(convs) = proj.get("conversations").and_then(|v| v.as_array()) {
                let mut sorted_convs = convs.clone();
                sorted_convs.sort_by_key(|c| {
                    c.get("last_modified")
                        .and_then(|v| v.as_str())
                        .map(|s| parse_flexible_timestamp(s))
                        .unwrap_or(0)
                });

                for conv in sorted_convs {
                    let cid = conv
                        .get("conversation_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let prompt_text = conv
                        .get("full_prompt_text")
                        .or_else(|| conv.get("prompt_preview_200w"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if cid.is_empty() || prompt_text.trim().is_empty() {
                        continue;
                    }

                    let prompt_id = format!("restored_{}_{}", proj_id, cid);
                    let conv_time = conv
                        .get("last_modified")
                        .and_then(|v| v.as_str())
                        .map(|s| parse_flexible_timestamp(s))
                        .unwrap_or(now);

                    // Justification: restore-from-backup insert is best-effort; the loop continues with the remaining entries
                    crate::error::record_ignored(
                        conn.execute(
                        "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 'gemini', ?6, 'backed_up', ?7, ?8)
                         ON CONFLICT(id) DO UPDATE SET
                            prompt_content = excluded.prompt_content,
                            status = 'backed_up',
                            updated_at = excluded.updated_at",
                        rusqlite::params![prompt_id, proj_id, inst_id, repo_path, prompt_text, cid, conv_time, now],
                    ),
                        "restore backed-up prompt from backup",
                    );
                    restored_count += 1;
                }
            }
        }
    }

    // Invalidate prompt tree cache to show freshly restored entries
    invalidate_prompt_tree_cache(None);

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Restored and synchronized {} prompts from backup file in FIFO order",
        restored_count
    ));

    Ok(restored_count)
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
