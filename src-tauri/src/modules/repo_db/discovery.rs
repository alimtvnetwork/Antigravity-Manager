//! Repo DB: discovery

use super::detection::live_prompt_text;
use super::dispatch::parse_flexible_timestamp;
use super::failed_commands::decode_uri_to_path;
use super::gemini_dirs::gemini_dirs_for_instance;
use super::models::ActivePrompt;
use super::transcript::{inspect_conversation_transcript, resolve_transcript_path};
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

struct ConversationSummaryRow {
    cid: String,
    preview: String,
    status: String,
    not_fully_idle: i32,
    workspace_uris: Option<String>,
    last_modified_time: Option<String>,
}

pub(crate) fn read_conversation_summary_rows(conn: &Connection) -> Vec<ConversationSummaryRow> {
    let wide = "SELECT conversation_id, preview, status, not_fully_idle, workspace_uris, last_modified_time
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
                last_modified_time: row.get::<_, Option<String>>(5).ok().flatten(),
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
            not_fully_idle: 0,
            workspace_uris: row.get(2)?,
            last_modified_time: None,
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

            let is_genuinely_running = not_fully_idle != 0 && status.contains("RUNNING");
            if !is_genuinely_running {
                continue;
            }

            let last_time_epoch = item
                .last_modified_time
                .as_deref()
                .map(parse_flexible_timestamp)
                .unwrap_or(0);
            let is_recent = last_time_epoch > 0 && (now - last_time_epoch <= 60);
            if !is_recent {
                continue;
            }

            let inspection = inspect_conversation_transcript(&base_dir, &cid);
            if inspection.is_terminal_done {
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

            // Read transcript file (prefers transcript_full.jsonl)
            let transcript_file = resolve_transcript_path(&base_dir, &cid);

            let mut user_prompt: Option<String> = None;
            let mut image_payload: Option<String> = None;

            if let Some(tf) = transcript_file {
                if let Ok(content) = fs::read_to_string(&tf) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
}
