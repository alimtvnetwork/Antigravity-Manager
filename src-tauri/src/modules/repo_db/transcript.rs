//! Repo DB: transcript

use super::models::TranscriptInspection;
use super::text_utils::extract_clean_user_prompt;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Helper to read a specific un-truncated line content from transcript_full.jsonl
fn get_line_from_full_transcript(full_path: &Path, target_line_idx: usize) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open(full_path).ok()?;
    let reader = BufReader::new(file);
    for (idx, line_res) in reader.lines().enumerate() {
        if idx == target_line_idx {
            if let Ok(l) = line_res {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&l) {
                    if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                        return Some(c.to_string());
                    }
                }
            }
            break;
        }
    }
    None
}

/// Resolve the best transcript file for a conversation ID:
/// Prefers `transcript_full.jsonl` if non-empty; falls back to `transcript.jsonl`.
pub fn resolve_transcript_path(base_dir: &Path, cid: &str) -> Option<PathBuf> {
    let logs_dir = base_dir
        .join("brain")
        .join(cid)
        .join(".system_generated")
        .join("logs");

    let full_path = logs_dir.join("transcript_full.jsonl");
    if full_path.exists() {
        if let Ok(meta) = std::fs::metadata(&full_path) {
            if meta.len() > 0 {
                return Some(full_path);
            }
        }
    }

    let regular_path = logs_dir.join("transcript.jsonl");
    if regular_path.exists() {
        return Some(regular_path);
    }

    None
}

/// Inspect transcript.jsonl and transcript_full.jsonl for a conversation to obtain:
/// step count, un-truncated user prompt, AI response, tool execution output, and subagent classification.
pub(crate) fn inspect_conversation_transcript(
    base_dir: &Path,
    conversation_id: &str,
) -> TranscriptInspection {
    let logs_dir = base_dir
        .join("brain")
        .join(conversation_id)
        .join(".system_generated")
        .join("logs");
    let transcript_full_path = logs_dir.join("transcript_full.jsonl");

    let chosen_path = match resolve_transcript_path(base_dir, conversation_id) {
        Some(p) => p,
        None => return TranscriptInspection::default(),
    };

    let now_epoch = Utc::now().timestamp();
    let mtime_epoch = std::fs::metadata(&chosen_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let content = match std::fs::read_to_string(&chosen_path) {
        Ok(c) => c,
        Err(_) => {
            return TranscriptInspection {
                step_count: 0,
                latest_prompt: None,
                latest_response: None,
                execution_results: None,
                tool_calls_summary: None,
                latest_step_summary: None,
                is_subagent: false,
                is_recent_active: false,
                is_non_prompt: true,
                is_terminal_done: false,
            };
        }
    };

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let step_count = lines.len();

    // Check if the conversation turn has completed or is waiting for user
    let mut is_completed_or_waiting = false;
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if s_type == "TOKEN_USAGE"
                || s_type == "TELEMETRY"
                || s_type == "HEARTBEAT"
                || s_type == "PROGRESS"
                || s_type == "METRICS"
                || s_type == "SYSTEM_LOG"
            {
                continue;
            }
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if s_status.eq_ignore_ascii_case("done")
                || s_status.eq_ignore_ascii_case("completed")
                || s_status.eq_ignore_ascii_case("error")
                || s_status.eq_ignore_ascii_case("idle")
            {
                is_completed_or_waiting = true;
                break;
            }
            if s_type == "PLANNER_RESPONSE"
                || val.get("source").and_then(|s| s.as_str()) == Some("MODEL")
            {
                let has_tool_calls = val
                    .get("tool_calls")
                    .and_then(|t| t.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if !has_tool_calls
                    && (s_status.is_empty()
                        || s_status.eq_ignore_ascii_case("done")
                        || s_status.eq_ignore_ascii_case("completed")
                        || s_status.eq_ignore_ascii_case("idle"))
                {
                    is_completed_or_waiting = true;
                    break;
                }
            }
        }
    }

    let is_recent_active =
        !is_completed_or_waiting && mtime_epoch > 0 && (now_epoch - mtime_epoch <= 60);

    let mut latest_prompt: Option<String> = None;
    let mut latest_response: Option<String> = None;
    let mut execution_results: Option<String> = None;
    let mut latest_step_summary: Option<String> = None;
    let mut tool_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut is_subagent = false;
    let mut has_real_user_prompt = false;

    // Check first step to see if this conversation was initiated as an autonomous subagent
    if let Some(first_line) = lines.first() {
        if let Ok(first_val) = serde_json::from_str::<serde_json::Value>(first_line) {
            let src = first_val
                .get("source")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let tp = first_val.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if src == "SYSTEM" || tp == "SYSTEM_MESSAGE" {
                let txt = first_val
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                let txt_lower = txt.to_lowercase();
                if txt.contains("You are ")
                    || txt_lower.contains("subagent")
                    || txt_lower.contains("worker")
                    || txt_lower.contains("author")
                    || txt_lower.contains("research")
                    || txt_lower.contains("debugger")
                    || txt_lower.contains("analysis")
                    || txt_lower.contains("architect")
                    || txt_lower.contains("invoked by a caller agent")
                    || txt_lower.contains("execute enhanced read memory")
                {
                    is_subagent = true;
                }
            }
        }
    }

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");

            // 1. User inputs
            if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" {
                if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                    let c_trim = c.trim();
                    let is_system_msg = c_trim.starts_with("The following is a <SYSTEM_MESSAGE>")
                        || c_trim.starts_with("[Message] timestamp=")
                        || c_trim.starts_with("Task id ")
                        || c_trim.starts_with("<SYSTEM_MESSAGE>");
                    if !is_system_msg {
                        let full_c = if val
                            .get("truncated_fields")
                            .and_then(|tf| tf.as_array())
                            .map(|arr| arr.iter().any(|v| v.as_str() == Some("content")))
                            .unwrap_or(false)
                            && transcript_full_path.exists()
                        {
                            get_line_from_full_transcript(&transcript_full_path, line_idx)
                                .unwrap_or_else(|| c.to_string())
                        } else {
                            c.to_string()
                        };

                        let clean = extract_clean_user_prompt(&full_c);
                        if !clean.is_empty() {
                            latest_prompt = Some(clean);
                            has_real_user_prompt = true;
                        }
                    } else if c_trim.contains("subagent") || c_trim.contains("Worker") {
                        is_subagent = true;
                    }
                }
            }

            // 2. Model planner response (AI answer / thoughts)
            if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                if let Some(tool_calls) = val.get("tool_calls").and_then(|tc| tc.as_array()) {
                    for tc in tool_calls {
                        let name = tc
                            .get("tool_name")
                            .or_else(|| tc.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("tool");
                        *tool_counts.entry(name.to_string()).or_insert(0) += 1;
                    }
                }

                if let Some(resp_txt) = val.get("content").and_then(|v| v.as_str()) {
                    let resp_trim = resp_txt.trim();
                    if !resp_trim.is_empty() {
                        let full_resp = if val
                            .get("truncated_fields")
                            .and_then(|tf| tf.as_array())
                            .map(|arr| arr.iter().any(|v| v.as_str() == Some("content")))
                            .unwrap_or(false)
                            && transcript_full_path.exists()
                        {
                            get_line_from_full_transcript(&transcript_full_path, line_idx)
                                .unwrap_or_else(|| resp_txt.to_string())
                        } else {
                            resp_txt.to_string()
                        };
                        latest_response = Some(full_resp);
                    }
                }
            }

            // 3. Tool results / Generic output / Tool call steps
            if s_type == "GENERIC"
                || s_type == "TOOL_OUTPUT"
                || s_type == "TOOL_RESPONSE"
                || s_source == "TOOL"
                || val.get("tool_call_id").is_some()
            {
                if let Some(tool_out) = val.get("content").and_then(|v| v.as_str()) {
                    let out_trim = tool_out.trim();
                    if !out_trim.is_empty() && out_trim.len() > 5 {
                        let preview: String = out_trim.chars().take(4000).collect();
                        execution_results = Some(preview);
                    }
                }
            }
        }
    }

    let mut is_terminal_done = false;

    // DEEP SCAN: Inspect up to 25 lines in reverse order to bypass telemetry noise
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");

            // Bypass pure telemetry / token usage metadata
            if s_type == "TOKEN_USAGE"
                || s_type == "TELEMETRY"
                || s_type == "HEARTBEAT"
                || s_type == "PROGRESS"
                || s_type == "METRICS"
                || s_type == "SYSTEM_LOG"
            {
                continue;
            }

            let step_idx = val
                .get("step_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(step_count as u64);
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            // Inspect for terminal completion
            if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                let tool_calls = val.get("tool_calls").and_then(|tc| tc.as_array());
                let has_open_tools = tool_calls.map(|tc| !tc.is_empty()).unwrap_or(false);
                let is_explicit_done = s_status.eq_ignore_ascii_case("DONE")
                    || s_status.eq_ignore_ascii_case("COMPLETED")
                    || s_status.eq_ignore_ascii_case("IDLE")
                    || s_status.eq_ignore_ascii_case("ERROR");
                if !has_open_tools && (is_explicit_done || s_status.is_empty()) {
                    is_terminal_done = true;
                }
            }

            // 1. Check for genuine tool_calls
            if let Some(tool_calls) = val.get("tool_calls").and_then(|tc| tc.as_array()) {
                if let Some(first_tc) = tool_calls.first() {
                    let tool_name = first_tc
                        .get("tool_name")
                        .or_else(|| first_tc.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("action");
                    if latest_step_summary.is_none() {
                        latest_step_summary = Some(format!(
                            "Step {} · Tool: {} ({})",
                            step_idx, tool_name, s_status
                        ));
                    }
                    if execution_results.is_none() {
                        let call_descs: Vec<String> = tool_calls
                            .iter()
                            .map(|tc| {
                                let name = tc
                                    .get("tool_name")
                                    .or_else(|| tc.get("name"))
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("tool");
                                let args = tc
                                    .get("arguments")
                                    .or_else(|| tc.get("args"))
                                    .map(|a| a.to_string())
                                    .unwrap_or_default();
                                format!("{}({})", name, args)
                            })
                            .collect();
                        execution_results = Some(format!(
                            "In-Flight Tool Execution:\n{}",
                            call_descs.join("\n")
                        ));
                    }
                    break;
                }
            }

            // 2. Check for assistant thoughts / reasoning
            if let Some(thinking) = val
                .get("thinking")
                .or_else(|| val.get("thought"))
                .and_then(|t| t.as_str())
            {
                let clean_thought = thinking.trim();
                let snippet: String = clean_thought.replace('\n', " ").chars().take(80).collect();
                if !snippet.is_empty() && latest_step_summary.is_none() {
                    latest_step_summary =
                        Some(format!("Step {} · Thinking · \"{}\"", step_idx, snippet));
                }
                if latest_response.is_none() && !clean_thought.is_empty() {
                    latest_response = Some(format!(
                        "💭 *AI Thinking / Reasoning:*\n\n{}",
                        clean_thought
                    ));
                }
                if latest_step_summary.is_some() {
                    break;
                }
            }

            // 3. Fallback to standard step content
            if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
                let clean_txt = txt.trim().replace('\n', " ");
                let short_txt: String = clean_txt.chars().take(80).collect();
                if !short_txt.is_empty() {
                    latest_step_summary =
                        Some(format!("Step {} · {} · {}", step_idx, s_type, short_txt));
                    break;
                }
            } else if latest_step_summary.is_none() {
                latest_step_summary =
                    Some(format!("Step {} · {} ({})", step_idx, s_type, s_status));
                break;
            }
        }
    }

    if !has_real_user_prompt && is_subagent {
        is_subagent = true;
    }

    let tool_calls_summary = if !tool_counts.is_empty() {
        let total: usize = tool_counts.values().sum();
        let mut parts: Vec<String> = tool_counts
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();
        parts.sort();
        Some(format!("{} tool calls ({})", total, parts.join(", ")))
    } else {
        None
    };

    let is_non_prompt = !has_real_user_prompt && !is_subagent;
    let terminal_done = is_terminal_done || is_completed_or_waiting;
    let effective_recent_active = if terminal_done {
        false
    } else {
        is_recent_active
    };

    TranscriptInspection {
        step_count,
        latest_prompt,
        latest_response,
        execution_results,
        tool_calls_summary,
        latest_step_summary,
        is_subagent,
        is_recent_active: effective_recent_active,
        is_non_prompt,
        is_terminal_done: terminal_done,
    }
}
