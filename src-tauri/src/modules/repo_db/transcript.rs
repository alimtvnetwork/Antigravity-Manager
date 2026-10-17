//! Repo DB: transcript

use super::models::{AgmPromptTurnNode, TranscriptInspection};
use super::text_utils::extract_clean_user_prompt;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use std::io::{Read, Seek, SeekFrom};

/// Bounded tail reading avoids loading 100MB+ into RAM
const MAX_TAIL_BYTES: u64 = 256 * 1024; // 256 KB
const MAX_TAIL_LINES: usize = 100;

/// Inspect strictly line 0 of transcript to determine if conversation was initiated
/// as an autonomous subagent without loading the whole file into RAM.
fn inspect_first_line_is_subagent(path: &Path) -> bool {
    use std::io::{BufRead, BufReader};
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() || first_line.trim().is_empty() {
        return false;
    }
    if let Ok(first_val) = serde_json::from_str::<serde_json::Value>(first_line.trim()) {
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
            return txt.contains("You are ")
                || txt_lower.contains("subagent")
                || txt_lower.contains("worker")
                || txt_lower.contains("author")
                || txt_lower.contains("research")
                || txt_lower.contains("debugger")
                || txt_lower.contains("analysis")
                || txt_lower.contains("architect")
                || txt_lower.contains("invoked by a caller agent")
                || txt_lower.contains("execute enhanced read memory");
        }
    }
    false
}

fn count_lines_fast(file: &mut std::fs::File) -> std::io::Result<usize> {
    file.seek(SeekFrom::Start(0))?;
    let mut reader = std::io::BufReader::with_capacity(64 * 1024, file);
    let mut count = 0;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        count += buf[..n].iter().filter(|&&b| b == b'\n').count();
    }
    Ok(count)
}

/// Read up to `max_bytes` from the tail of `path` and return the most recent `max_lines` non-empty lines,
/// along with an accurate or bounded step count estimate.
fn read_bounded_tail_lines(path: &Path, max_bytes: u64, max_lines: usize) -> (Vec<String>, usize) {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (Vec::new(), 0),
    };

    let meta = match file.metadata() {
        Ok(m) => m,
        Err(_) => return (Vec::new(), 0),
    };

    let file_len = meta.len();
    if file_len == 0 {
        return (Vec::new(), 0);
    }

    let (read_start, is_full_file) = if file_len <= max_bytes {
        (0u64, true)
    } else {
        (file_len - max_bytes, false)
    };

    if file.seek(SeekFrom::Start(read_start)).is_err() {
        return (Vec::new(), 0);
    }

    let to_read = (file_len - read_start) as usize;
    let mut buffer = vec![0u8; to_read];
    if file.read_exact(&mut buffer).is_err() {
        return (Vec::new(), 0);
    }

    let content_slice = String::from_utf8_lossy(&buffer);
    let mut all_lines: Vec<&str> = content_slice.lines().collect();

    // Discard initial partial line if we sought into middle of the file
    if !is_full_file && !all_lines.is_empty() {
        all_lines.remove(0);
    }

    let non_empty: Vec<String> = all_lines
        .into_iter()
        .filter(|l| !l.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    let last_step_index = non_empty.last().and_then(|last_line| {
        serde_json::from_str::<serde_json::Value>(last_line)
            .ok()
            .and_then(|v| v.get("step_index").and_then(|s| s.as_u64()))
    });

    let total_steps = if let Some(last_idx) = last_step_index {
        (last_idx + 1) as usize
    } else if is_full_file {
        non_empty.len()
    } else {
        count_lines_fast(&mut file).unwrap_or(non_empty.len())
    };

    let tail = if non_empty.len() > max_lines {
        non_empty[non_empty.len() - max_lines..].to_vec()
    } else {
        non_empty
    };

    (tail, total_steps)
}

/// Pre-load tail content of transcript_full.jsonl into a map keyed by step_index
/// to guarantee O(1) single-pass lookup for truncated lines without rescanning from line 0.
fn load_full_transcript_tail_map(full_path: &Path, max_bytes: u64) -> HashMap<usize, String> {
    let mut map = HashMap::new();
    let (lines, _) = read_bounded_tail_lines(full_path, max_bytes, MAX_TAIL_LINES);
    for line in lines {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                if let Some(step_idx) = val.get("step_index").and_then(|s| s.as_u64()) {
                    map.insert(step_idx as usize, c.to_string());
                }
            }
        }
    }
    map
}

/// Helper to read a specific un-truncated line content from transcript_full.jsonl.
/// Checks the tail map first; only falls back to a forward buffered pass if not found in tail.
fn get_line_from_full_transcript(
    full_path: &Path,
    target_idx: usize,
    tail_map: &HashMap<usize, String>,
) -> Option<String> {
    if let Some(content) = tail_map.get(&target_idx) {
        return Some(content.clone());
    }

    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open(full_path).ok()?;
    let reader = BufReader::new(file);
    for (idx, line_res) in reader.lines().enumerate() {
        if let Ok(l) = line_res {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&l) {
                let step_idx = val
                    .get("step_index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(idx);
                if step_idx == target_idx {
                    if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                        return Some(c.to_string());
                    }
                }
            }
        }
        if idx > target_idx + 100 {
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

    let (lines, step_count) = read_bounded_tail_lines(&chosen_path, MAX_TAIL_BYTES, MAX_TAIL_LINES);
    if lines.is_empty() {
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

    // Check if the conversation turn has completed or is waiting for user.
    // Invariant: Differentiate step-level tool completion from turn-level assistant completion.
    let mut is_completed_or_waiting = false;
    let mut checked_first_meaningful = false;
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
            if s_type == "TOKEN_USAGE"
                || s_type == "TELEMETRY"
                || s_type == "HEARTBEAT"
                || s_type == "PROGRESS"
                || s_type == "METRICS"
                || s_type == "SYSTEM_LOG"
            {
                continue;
            }

            // Step-level outputs (tool execution results) report status: "DONE" on the step itself.
            // These indicate tool completion, NOT turn completion. Skip them.
            let is_tool_step = s_type == "GENERIC"
                || s_type == "TOOL_OUTPUT"
                || s_type == "TOOL_RESPONSE"
                || s_source == "TOOL"
                || val.get("tool_call_id").is_some();
            if is_tool_step {
                continue;
            }

            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            if !checked_first_meaningful {
                checked_first_meaningful = true;
                // Explicit turn-level terminal completion: Planner/Model response with ZERO tool calls
                if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                    let has_tool_calls = val
                        .get("tool_calls")
                        .and_then(|t| t.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);
                    if !has_tool_calls
                        && (s_status.is_empty()
                            || s_status.eq_ignore_ascii_case("done")
                            || s_status.eq_ignore_ascii_case("completed")
                            || s_status.eq_ignore_ascii_case("idle")
                            || s_status.eq_ignore_ascii_case("error"))
                    {
                        is_completed_or_waiting = true;
                        break;
                    }
                } else if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" {
                    // If the most recent meaningful event is user input awaiting AI, turn is active (not completed)
                    break;
                } else if s_status.eq_ignore_ascii_case("completed")
                    || s_status.eq_ignore_ascii_case("error")
                    || s_status.eq_ignore_ascii_case("idle")
                {
                    is_completed_or_waiting = true;
                    break;
                }
            } else {
                break;
            }
        }
    }

    // Adaptive 10-Minute Thinking Window: Expand timestamp window to 600s (10 minutes) when mtime is fresh
    let is_mtime_fresh = mtime_epoch > 0 && (now_epoch - mtime_epoch <= 600);
    let is_recent_active = !is_completed_or_waiting && is_mtime_fresh;

    let mut latest_prompt: Option<String> = None;
    let mut latest_response: Option<String> = None;
    let mut execution_results: Option<String> = None;
    let mut latest_step_summary: Option<String> = None;
    let mut tool_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut is_subagent = inspect_first_line_is_subagent(&chosen_path);
    let mut has_real_user_prompt = false;
    let mut full_tail_map: Option<HashMap<usize, String>> = None;

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
                            let target_idx = val
                                .get("step_index")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as usize)
                                .unwrap_or(line_idx);
                            let map = full_tail_map.get_or_insert_with(|| {
                                load_full_transcript_tail_map(&transcript_full_path, MAX_TAIL_BYTES)
                            });
                            get_line_from_full_transcript(&transcript_full_path, target_idx, map)
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
                            let target_idx = val
                                .get("step_index")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as usize)
                                .unwrap_or(line_idx);
                            let map = full_tail_map.get_or_insert_with(|| {
                                load_full_transcript_tail_map(&transcript_full_path, MAX_TAIL_BYTES)
                            });
                            get_line_from_full_transcript(&transcript_full_path, target_idx, map)
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

            // Step-level outputs (tool execution results) report status: "DONE" on the step itself.
            // These indicate tool completion, NOT turn completion. Skip them.
            let is_tool_step = s_type == "GENERIC"
                || s_type == "TOOL_OUTPUT"
                || s_type == "TOOL_RESPONSE"
                || s_source == "TOOL"
                || val.get("tool_call_id").is_some();
            if is_tool_step {
                continue;
            }

            let step_idx = val
                .get("step_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(step_count as u64);
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            // Inspect for terminal completion:
            // Only if PLANNER_RESPONSE (or non-tool MODEL text output) has ZERO tool calls
            // and explicit terminal status (DONE, COMPLETED, IDLE, ERROR), set is_terminal_done = true.
            // If has_open_tools is true (the model called tools), explicitly ensure is_terminal_done = false and break!
            if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                let tool_calls = val.get("tool_calls").and_then(|tc| tc.as_array());
                let has_open_tools = tool_calls.map(|tc| !tc.is_empty()).unwrap_or(false);
                if has_open_tools {
                    is_terminal_done = false;
                    if let Some(first_tc) = tool_calls.and_then(|tc| tc.first()) {
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
                            if let Some(tc_arr) = tool_calls {
                                let call_descs: Vec<String> = tc_arr
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
                        }
                    }
                    break;
                } else {
                    let is_explicit_done = s_status.eq_ignore_ascii_case("DONE")
                        || s_status.eq_ignore_ascii_case("COMPLETED")
                        || s_status.eq_ignore_ascii_case("IDLE")
                        || s_status.eq_ignore_ascii_case("ERROR");
                    // Guard: Intermediate planner turns without tool calls must NOT set is_terminal_done = true
                    // when subsequent turns or steps exist in the transcript.
                    // Only flag is_terminal_done if this step is at the very tail of the transcript.
                    if is_explicit_done && (step_idx + 1 >= step_count as u64) {
                        is_terminal_done = true;
                    }
                }
            }

            // 1. Check for genuine tool_calls
            if let Some(tool_calls) = val.get("tool_calls").and_then(|tc| tc.as_array()) {
                if let Some(first_tc) = tool_calls.first() {
                    is_terminal_done = false;
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

/// Extract all prompt turns (Tier 3) dispatched to a conversation from transcript_full.jsonl / transcript.jsonl.
/// Accurately parses user prompts, image attachments, 200-word previews, and marks execution states
/// (RUNNING, QUEUED, COMPLETED) based on whether the conversation is currently active.
pub fn extract_conversation_prompt_turns(
    base_dir: &Path,
    cid: &str,
    is_conv_running: bool,
) -> Vec<AgmPromptTurnNode> {
    let chosen_path = match resolve_transcript_path(base_dir, cid) {
        Some(p) => p,
        None => return Vec::new(),
    };

    use std::io::{BufRead, BufReader};
    let file = match std::fs::File::open(&chosen_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::with_capacity(64 * 1024, file);

    #[derive(Debug)]
    struct RawTurnInfo {
        step_index: usize,
        prompt_text: String,
        timestamp: String,
        created_at: i64,
        media_paths: Vec<String>,
        media_thumbnails: Vec<String>,
    }

    let mut raw_turns: Vec<RawTurnInfo> = Vec::new();
    let mut last_model_step_idx: Option<usize> = None;
    let mut last_model_is_done = false;

    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Fast string probe before full JSON deserialization to skip huge tool output lines
        let is_user_candidate = trimmed.contains("\"USER_INPUT\"")
            || trimmed.contains("\"USER_EXPLICIT\"")
            || trimmed.contains("\"source\":\"USER\"")
            || trimmed.contains("\"source\": \"USER\"");
        let is_model_candidate = trimmed.contains("\"PLANNER_RESPONSE\"")
            || trimmed.contains("\"source\":\"MODEL\"")
            || trimmed.contains("\"source\": \"MODEL\"");

        if !is_user_candidate && !is_model_candidate {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let step_idx = val
            .get("step_index")
            .and_then(|v| v.as_u64())
            .map(|u| u as usize)
            .unwrap_or(line_idx);

        if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" || s_source == "USER" {
            if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                let c_trim = c.trim();
                let is_internal = c_trim.starts_with("The following is a <SYSTEM_MESSAGE>")
                    || c_trim.starts_with("[Message] timestamp=")
                    || c_trim.starts_with("Task id ")
                    || c_trim.starts_with("<SYSTEM_MESSAGE>");
                if !is_internal {
                    let clean = extract_clean_user_prompt(c);
                    if !clean.trim().is_empty() {
                        let ts = val
                            .get("created_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let created_at_ms = if !ts.is_empty() {
                            chrono::DateTime::parse_from_rfc3339(&ts)
                                .map(|dt| dt.timestamp_millis())
                                .unwrap_or_else(|_| {
                                    chrono::DateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                                        .map(|dt| dt.timestamp_millis())
                                        .unwrap_or(0)
                                })
                        } else {
                            0
                        };

                        let mut media_paths = Vec::new();
                        let mut media_thumbnails = Vec::new();
                        if let Some(media_arr) = val.get("media").and_then(|m| m.as_array()) {
                            for m in media_arr {
                                if let Some(uri) = m.get("uri").and_then(|u| u.as_str()) {
                                    let clean_uri = uri.trim();
                                    let file_path = if let Some(stripped) = clean_uri.strip_prefix("file:///") {
                                        stripped.to_string()
                                    } else if let Some(stripped) = clean_uri.strip_prefix("file://") {
                                        stripped.to_string()
                                    } else {
                                        clean_uri.to_string()
                                    };
                                    let p = std::path::Path::new(&file_path);
                                    let resolved_path = if p.exists() {
                                        file_path.clone()
                                    } else {
                                        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                        let cand1 = base_dir.join("brain").join(cid).join(".user_uploaded").join(fname);
                                        let cand2 = base_dir.join("brain").join(cid).join(fname);
                                        if cand1.exists() {
                                            cand1.to_string_lossy().to_string()
                                        } else if cand2.exists() {
                                            cand2.to_string_lossy().to_string()
                                        } else {
                                            file_path.clone()
                                        }
                                    };
                                    if !resolved_path.is_empty() && !media_paths.contains(&resolved_path) {
                                        media_paths.push(resolved_path.clone());
                                        media_thumbnails.push(resolved_path);
                                    }
                                }
                            }
                        }

                        raw_turns.push(RawTurnInfo {
                            step_index: step_idx,
                            prompt_text: clean,
                            timestamp: ts,
                            created_at: created_at_ms,
                            media_paths,
                            media_thumbnails,
                        });
                    }
                }
            }
        } else if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
            let tool_calls = val.get("tool_calls").and_then(|tc| tc.as_array());
            let has_open_tools = tool_calls.map(|tc| !tc.is_empty()).unwrap_or(false);
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let is_done = !has_open_tools
                && (s_status.eq_ignore_ascii_case("DONE")
                    || s_status.eq_ignore_ascii_case("COMPLETED")
                    || s_status.eq_ignore_ascii_case("IDLE"));
            last_model_step_idx = Some(step_idx);
            last_model_is_done = is_done;
        }
    }

    if raw_turns.is_empty() {
        return Vec::new();
    }

    let n = raw_turns.len();

    // Determine the active turn index if conversation is running
    let active_turn_idx: Option<usize> = if is_conv_running {
        if let Some(last_model_idx) = last_model_step_idx {
            let mut matched_idx = 0;
            for (idx, turn) in raw_turns.iter().enumerate() {
                if turn.step_index <= last_model_idx {
                    matched_idx = idx;
                } else {
                    break;
                }
            }
            if last_model_is_done && matched_idx + 1 < n {
                Some(matched_idx + 1)
            } else {
                Some(matched_idx)
            }
        } else {
            Some(0)
        }
    } else {
        None
    };

    let mut result = Vec::with_capacity(n);
    for (idx, turn) in raw_turns.into_iter().enumerate() {
        let (status, is_running, is_queued) = match active_turn_idx {
            Some(act_idx) => {
                if idx < act_idx {
                    ("COMPLETED".to_string(), false, false)
                } else if idx == act_idx {
                    ("RUNNING".to_string(), true, false)
                } else {
                    ("QUEUED".to_string(), false, true)
                }
            }
            None => ("COMPLETED".to_string(), false, false),
        };

        let (preview_200w, _) = super::sequences::extract_prompt_words_preview(&turn.prompt_text, 200);

        result.push(AgmPromptTurnNode {
            turn_number: idx + 1,
            step_index: turn.step_index,
            prompt_text: turn.prompt_text,
            preview_200w,
            status,
            is_running,
            is_queued,
            timestamp: turn.timestamp,
            created_at: turn.created_at,
            media_paths: turn.media_paths,
            media_thumbnails: turn.media_thumbnails,
        });
    }

    result
}

