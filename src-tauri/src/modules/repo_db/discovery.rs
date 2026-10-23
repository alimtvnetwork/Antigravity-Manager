//! Repo DB: discovery

use super::dispatch::parse_flexible_timestamp;
use super::failed_commands::decode_uri_to_path;
use super::gemini_dirs::gemini_dirs_for_instance;
use super::models::ActivePrompt;
use super::text_utils::extract_clean_user_prompt;
use super::transcript::{inspect_conversation_transcript, resolve_transcript_path};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
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
            not_fully_idle: 1,
            workspace_uris: row.get(2)?,
            last_modified_time: None,
        })
    }) else {
        return Vec::new();
    };
    rows.flatten().collect()
}

fn is_subagent_or_system_content(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.contains("<SYSTEM_MESSAGE>")
        || trimmed.contains("[Message] timestamp=")
        || trimmed.contains("You are Research")
        || trimmed.contains("You are Worker")
        || trimmed.contains("You are Spec Author")
        || trimmed.starts_with("Task id ")
    {
        return true;
    }
    let lower = trimmed.to_lowercase();
    lower.contains("subagent")
        || lower.contains("invoked by a caller agent")
        || lower.starts_with("you are ")
}

fn read_step_from_full_transcript(path: &Path, target_step_index: usize) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    for line_res in reader.lines() {
        if let Ok(l) = line_res {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&l) {
                if val.get("step_index").and_then(|s| s.as_u64()) == Some(target_step_index as u64)
                {
                    return val
                        .get("content")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string());
                }
            }
        }
    }
    None
}

/// Imp-542: Parses transcript content to extract user prompt and attached images.
/// Uses early-return guard clauses to eliminate nested control flow.
pub(crate) fn extract_user_prompt_and_media(
    content: &str,
    full_transcript_path: &Path,
) -> (Option<String>, Option<String>) {
    for line in content.lines().rev() {
        if !line.contains("USER_INPUT") && !line.contains("USER_EXPLICIT") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let s_type = val.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let s_source = val.get("source").and_then(|s| s.as_str()).unwrap_or("");
        if s_type != "USER_INPUT" && s_source != "USER_EXPLICIT" {
            continue;
        }

        let raw_txt = val.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if is_subagent_or_system_content(raw_txt) {
            continue;
        }

        let is_truncated = raw_txt.contains("<truncated")
            || val
                .get("truncated_fields")
                .and_then(|tf| tf.as_array())
                .map(|arr| arr.iter().any(|v| v.as_str() == Some("content")))
                .unwrap_or(false);

        let resolved_txt = if is_truncated && full_transcript_path.exists() {
            let target_step = val.get("step_index").and_then(|s| s.as_u64());
            if let Some(target_idx) = target_step {
                read_step_from_full_transcript(full_transcript_path, target_idx as usize)
                    .unwrap_or_else(|| raw_txt.to_string())
            } else {
                raw_txt.to_string()
            }
        } else {
            raw_txt.to_string()
        };

        let clean_txt = extract_clean_user_prompt(&resolved_txt);
        if clean_txt.trim().is_empty() {
            continue;
        }

        let mut image_payload: Option<String> = None;
        if let Some(media_arr) = val.get("media").and_then(|m| m.as_array()) {
            for m_item in media_arr {
                let Some(uri) = m_item.get("uri").and_then(|u| u.as_str()) else {
                    continue;
                };
                let clean_path = decode_uri_to_path(uri);
                let p = Path::new(&clean_path);
                if !p.exists() {
                    continue;
                }
                if let Ok(bytes) = fs::read(p) {
                    let mime = m_item
                        .get("mime_type")
                        .and_then(|mt| mt.as_str())
                        .unwrap_or("image/png");
                    let b64 = STANDARD.encode(&bytes);
                    image_payload = Some(format!("data:{};base64,{}", mime, b64));
                    break;
                }
            }
        }

        return (Some(clean_txt), image_payload);
    }
    (None, None)
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

            let last_time_epoch = item
                .last_modified_time
                .as_deref()
                .map(parse_flexible_timestamp)
                .unwrap_or(0);
            // 2-hour freshness window (7200s); if last_time_epoch is missing or 0, fallback to transcript file mtime
            let effective_mtime = if last_time_epoch > 0 {
                last_time_epoch
            } else {
                resolve_transcript_path(&base_dir, &cid)
                    .and_then(|p| fs::metadata(p).ok())
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            };
            let is_recent = effective_mtime == 0 || (now - effective_mtime <= 7200);
            if !is_recent {
                continue;
            }

            let inspection = inspect_conversation_transcript(&base_dir, &cid);
            if inspection.is_terminal_done || inspection.is_subagent {
                continue;
            }

            let is_genuinely_running = not_fully_idle != 0 && status.contains("RUNNING");
            let is_quota_or_error_interrupted = inspection.is_error_or_quota
                || status.contains("ERROR")
                || status.contains("FAILED");

            if !is_genuinely_running && !is_quota_or_error_interrupted {
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
            let logs_dir = base_dir
                .join("brain")
                .join(&cid)
                .join(".system_generated")
                .join("logs");
            let full_transcript_path = logs_dir.join("transcript_full.jsonl");
            let transcript_file = resolve_transcript_path(&base_dir, &cid);

            let (mut user_prompt, image_payload) = match transcript_file {
                Some(tf) => match fs::read_to_string(&tf) {
                    Ok(c) => extract_user_prompt_and_media(&c, &full_transcript_path),
                    Err(_) => (None, None),
                },
                None => (None, None),
            };

            // If reverse scan missed but inspection has recent clean prompt, use it
            if user_prompt.is_none() && inspection.latest_prompt.is_some() {
                if let Some(ref lp) = inspection.latest_prompt {
                    if !is_subagent_or_system_content(lp) {
                        user_prompt = Some(lp.clone());
                    }
                }
            }

            // Fallback to conversation preview if transcript extraction found no user prompt
            if user_prompt.is_none()
                && !preview.trim().is_empty()
                && !is_subagent_or_system_content(&preview)
            {
                user_prompt = Some(preview.clone());
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

    if !prompts.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Discovered {} prompt(s) from Antigravity storage for instance '{}' (freshness window 7200s, including quota/error-interrupted)",
            prompts.len(),
            instance_id
        ));
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
