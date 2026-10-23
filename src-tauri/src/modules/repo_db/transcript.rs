//! Repo DB: transcript

use super::models::{AgmPromptTurnNode, TranscriptInspection};
use super::text_utils::{extract_clean_user_prompt, StrExt};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use std::io::{Read, Seek, SeekFrom};

/// Bounded tail reading avoids loading 100MB+ into RAM
const MAX_TAIL_BYTES: u64 = 256 * 1024; // 256 KB
const MAX_TAIL_LINES: usize = 100;

/// Evaluates whether text content contains known subagent role or instruction markers.
pub(crate) fn is_subagent_prompt_text(txt: &str) -> bool {
    let is_caller_invoked = txt.contains("You are ");
    if is_caller_invoked {
        return true;
    }
    const SUBAGENT_KEYWORDS: &[&str] = &[
        "subagent",
        "worker",
        "author",
        "research",
        "debugger",
        "invoked by a caller agent",
        "execute enhanced read memory",
    ];
    txt.has_any_word_ignore_case(SUBAGENT_KEYWORDS)
}

/// Returns true if the transcript step type represents telemetry or heartbeat metadata.
pub(crate) fn is_telemetry_event(s_type: &str) -> bool {
    matches!(
        s_type,
        "TOKEN_USAGE" | "TELEMETRY" | "HEARTBEAT" | "PROGRESS" | "METRICS" | "SYSTEM_LOG"
    )
}

/// Evaluates whether a transcript step represents an intermediate tool execution rather than turn completion.
pub(crate) fn is_tool_execution_step(
    s_type: &str,
    s_source: &str,
    val: &serde_json::Value,
) -> bool {
    let is_tool_type = matches!(s_type, "GENERIC" | "TOOL_OUTPUT" | "TOOL_RESPONSE");
    let is_tool_source = s_source.eq_ignore_ascii_case("TOOL");
    let has_tool_call_id = val.get("tool_call_id").is_some();
    is_tool_type || is_tool_source || has_tool_call_id
}

/// Determines if a status token represents a successful terminal turn state (DONE, COMPLETED, IDLE).
/// Errors and quota exhaustion are explicitly excluded so that interrupted turns are not treated as done.
pub(crate) fn is_terminal_status(status: &str) -> bool {
    const TERMINAL_STATUSES: &[&str] = &["done", "completed", "idle"];
    status.trim().equals_any_ignore_case(TERMINAL_STATUSES)
}

/// Evaluates whether a status token represents an error, failure, or quota exhaustion.
pub(crate) fn is_error_or_quota_status(status: &str) -> bool {
    let s = status.trim().to_ascii_lowercase();
    const ERROR_STATUSES: &[&str] = &[
        "error",
        "failed",
        "failure",
        "resource_exhausted",
        "quota_exhausted",
        "quota_depleted",
        "exhausted",
        "rate_limited",
        "429",
    ];
    s.equals_any_ignore_case(ERROR_STATUSES)
        || s.contains("quota")
        || s.contains("resource_exhausted")
        || s.contains("exhaust")
        || s.contains("rate_limit")
}

/// Checks if text content or error payload indicates quota or resource exhaustion.
pub(crate) fn contains_quota_or_rate_limit_error(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("resource_exhausted")
        || lower.contains("quota")
        || lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("rate_limit")
        || lower.contains("exhausted")
        || lower.contains("credit")
}

/// Evaluates whether a transcript step records an error, failure, or quota exhaustion.
pub(crate) fn is_step_error_or_quota(val: &serde_json::Value) -> bool {
    let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
    if is_genuine_user_input(s_type, s_source) {
        return false;
    }

    if s_type.eq_ignore_ascii_case("ERROR") || s_type.eq_ignore_ascii_case("SYSTEM_ERROR") {
        return true;
    }

    let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if is_error_or_quota_status(s_status) {
        return true;
    }

    if let Some(err_val) = val.get("error") {
        let err_str = err_val.to_string();
        if contains_quota_or_rate_limit_error(&err_str) {
            return true;
        }
    }

    if let Some(content) = val.get("content").and_then(|v| v.as_str()) {
        if contains_quota_or_rate_limit_error(content) {
            return true;
        }
    }

    false
}

/// Evaluates whether a planner response has completed without pending tool invocations.
pub(crate) fn is_planner_turn_completed(has_open_tools: bool, s_status: &str) -> bool {
    if has_open_tools {
        return false;
    }
    // An empty s_status indicates an in-flight, streaming, or non-finalized turn.
    // Only explicit successful terminal statuses (done, completed, idle) indicate turn completion.
    // Error or quota depletion is NOT treated as a completed turn.
    !s_status.trim().is_empty()
        && is_terminal_status(s_status)
        && !is_error_or_quota_status(s_status)
}

/// Determines whether an entry represents genuine user input rather than internal system events.
pub(crate) fn is_genuine_user_input(s_type: &str, s_source: &str) -> bool {
    if s_source.eq_ignore_ascii_case("SYSTEM") {
        return false;
    }
    let is_user_type = s_type.eq_ignore_ascii_case("USER_INPUT");
    const USER_SOURCES: &[&str] = &["USER_EXPLICIT", "USER"];
    let is_user_source = s_source.equals_any_ignore_case(USER_SOURCES);
    is_user_type || is_user_source
}

/// Determines whether message content represents internal system prompts or subagent instructions.
pub(crate) fn is_system_prompt_message(s_source: &str, content: &str) -> bool {
    if s_source.eq_ignore_ascii_case("SYSTEM") {
        return true;
    }
    const SYSTEM_PREFIXES: &[&str] = &[
        "<system_message>",
        "the following is a <system_message>",
        "[message] timestamp=",
        "task id ",
    ];
    content.trim().starts_with_any_ignore_case(SYSTEM_PREFIXES)
}

/// Evaluates whether a transcript step records content truncation requiring full transcript lookup.
pub(crate) fn is_content_truncated(val: &serde_json::Value) -> bool {
    val.get("truncated_fields")
        .and_then(|tf| tf.as_array())
        .map(|arr| arr.iter().any(|v| v.as_str() == Some("content")))
        .unwrap_or(false)
}

/// Pre-filters raw transcript lines for candidate user or planner markers prior to JSON parsing.
pub(crate) fn is_relevant_turn_candidate(trimmed: &str) -> bool {
    const CANDIDATE_MARKERS: &[&str] = &[
        "USER_INPUT",
        "USER_EXPLICIT",
        "\"USER\"",
        "\"user\"",
        "PLANNER_RESPONSE",
        "\"MODEL\"",
        "\"model\"",
    ];
    trimmed.has_any_word(CANDIDATE_MARKERS)
}

/// Inspect early lines of transcript to determine if conversation was initiated
/// as an autonomous subagent without loading the whole file into RAM.
fn inspect_first_line_is_subagent(path: &Path) -> bool {
    use std::io::{BufRead, BufReader};
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let reader = BufReader::new(file);
    for line_res in reader.lines().take(5) {
        if let Ok(line) = line_res {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(first_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let tp = first_val.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if tp == "EPHEMERAL_MESSAGE" {
                    continue;
                }
                let src = first_val
                    .get("source")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                if src == "SYSTEM" || tp == "SYSTEM_MESSAGE" {
                    let txt = first_val
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    return is_subagent_prompt_text(txt);
                } else {
                    break;
                }
            }
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
    let is_partial_read = !is_full_file;
    let has_lines = !all_lines.is_empty();
    if is_partial_read && has_lines {
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
            is_error_or_quota: false,
        };
    }

    // Check if the conversation turn has completed or is waiting for user.
    // Invariant: Differentiate step-level tool completion from turn-level assistant completion.
    let mut is_completed_or_waiting = false;
    let mut is_latest_turn_error_or_quota = false;
    let mut checked_first_meaningful = false;
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
            if is_telemetry_event(s_type) {
                continue;
            }

            // Step-level outputs (tool execution results) report status: "DONE" on the step itself.
            // These indicate tool completion, NOT turn completion. Skip them.
            if is_tool_execution_step(s_type, s_source, &val) {
                continue;
            }

            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            if !checked_first_meaningful {
                checked_first_meaningful = true;

                // Error or quota exhaustion steps must NOT be treated as terminal completion
                if is_step_error_or_quota(&val) {
                    is_latest_turn_error_or_quota = true;
                    is_completed_or_waiting = false;
                    break;
                }

                // Explicit turn-level terminal completion: Planner/Model response with ZERO tool calls
                if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                    let has_tool_calls = val
                        .get("tool_calls")
                        .and_then(|t| t.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);
                    if is_planner_turn_completed(has_tool_calls, s_status) {
                        is_completed_or_waiting = true;
                        break;
                    }
                } else if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" {
                    // If the most recent meaningful event is user input awaiting AI, turn is active (not completed)
                    break;
                } else if is_terminal_status(s_status) {
                    is_completed_or_waiting = true;
                    break;
                }
            } else {
                break;
            }
        }
    }

    // Adaptive 10-Minute Thinking Window: Expand timestamp window to 600s (10 minutes) when mtime is fresh
    let is_within_freshness_window = now_epoch - mtime_epoch <= 600;
    let is_mtime_fresh = mtime_epoch > 0 && is_within_freshness_window;
    let is_turn_in_flight = !is_completed_or_waiting;
    let is_recent_active = is_turn_in_flight && is_mtime_fresh;

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
            let is_user_input = is_genuine_user_input(s_type, s_source);

            if is_user_input {
                if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                    let c_trim = c.trim();
                    let is_system_msg = is_system_prompt_message(s_source, c);
                    if !is_system_msg {
                        let full_c = if is_content_truncated(&val) && transcript_full_path.exists()
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
                    } else if is_subagent_prompt_text(c_trim) {
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
                        let full_resp = if is_content_truncated(&val)
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
            if is_tool_execution_step(s_type, s_source, &val) {
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
    let mut checked_terminal_meaningful = false;

    // DEEP SCAN: Inspect up to 25 lines in reverse order to bypass telemetry noise
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");

            // Bypass pure telemetry / token usage metadata
            if is_telemetry_event(s_type) {
                continue;
            }

            // Step-level outputs (tool execution results) report status: "DONE" on the step itself.
            // These indicate tool completion, NOT turn completion. Skip them.
            if is_tool_execution_step(s_type, s_source, &val) {
                continue;
            }

            let step_idx = val
                .get("step_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(step_count as u64);
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            if !checked_terminal_meaningful {
                checked_terminal_meaningful = true;
                if is_step_error_or_quota(&val) {
                    is_latest_turn_error_or_quota = true;
                    is_terminal_done = false;
                    if latest_step_summary.is_none() {
                        latest_step_summary = Some(format!(
                            "Step {} · Interrupted (Quota/Error: {})",
                            step_idx, s_status
                        ));
                    }
                    break;
                }
            }

            // Inspect for terminal completion:
            // Only if PLANNER_RESPONSE (or non-tool MODEL text output) has ZERO tool calls
            // and explicit successful terminal status (DONE, COMPLETED, IDLE), set is_terminal_done = true.
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
                    let is_explicit_done =
                        is_terminal_status(s_status) && !is_error_or_quota_status(s_status);
                    // Guard: Intermediate planner turns without tool calls must NOT set is_terminal_done = true
                    // when subsequent turns or steps exist in the transcript.
                    // Only flag is_terminal_done if this step is at the very tail of the transcript.
                    let is_tail_step = step_idx + 1 >= step_count as u64;
                    if is_explicit_done && is_tail_step {
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

    if latest_prompt.is_none() {
        use std::io::{BufRead, BufReader};
        if let Ok(f) = std::fs::File::open(&chosen_path) {
            let r = BufReader::new(f);
            for line_res in r.lines().take(50) {
                if let Ok(line) = line_res {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
                        let is_user = is_genuine_user_input(s_type, s_source);
                        if is_user {
                            if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                                let clean = extract_clean_user_prompt(c);
                                if !clean.is_empty() {
                                    latest_prompt = Some(clean);
                                    has_real_user_prompt = true;
                                    break;
                                }
                            }
                        }
                    }
                }
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

    let has_identified_prompt = has_real_user_prompt || is_subagent;
    let is_non_prompt = !has_identified_prompt;
    let terminal_done =
        !is_latest_turn_error_or_quota && (is_terminal_done || is_completed_or_waiting);
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
        is_error_or_quota: is_latest_turn_error_or_quota,
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
    let mut last_completed_model_step_idx: Option<usize> = None;

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
        if !is_relevant_turn_candidate(trimmed) {
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

        let is_user_input = is_genuine_user_input(s_type, s_source);

        if is_user_input {
            if let Some(c) = val.get("content").and_then(|v| v.as_str()) {
                let is_internal = is_system_prompt_message(s_source, c);
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
                                    let file_path =
                                        crate::modules::repo_db::failed_commands::decode_uri_to_path(clean_uri);
                                    let p = std::path::Path::new(&file_path);
                                    let resolved_path = if p.exists() {
                                        file_path.clone()
                                    } else {
                                        let fname =
                                            p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                        let cand1 = base_dir
                                            .join("brain")
                                            .join(cid)
                                            .join(".user_uploaded")
                                            .join(fname);
                                        let cand2 = base_dir.join("brain").join(cid).join(fname);
                                        if cand1.exists() {
                                            cand1.to_string_lossy().to_string()
                                        } else if cand2.exists() {
                                            cand2.to_string_lossy().to_string()
                                        } else {
                                            file_path.clone()
                                        }
                                    };
                                    let has_resolved_path = !resolved_path.is_empty();
                                    let is_new_media = !media_paths.contains(&resolved_path);
                                    if has_resolved_path && is_new_media {
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
            let is_done = is_planner_turn_completed(has_open_tools, s_status);
            last_model_step_idx = Some(step_idx);
            last_model_is_done = is_done;
            if is_done {
                last_completed_model_step_idx = Some(step_idx);
            }
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

    let now = Utc::now().timestamp();
    let is_inst_alive =
        !crate::modules::instance::find_pids_for_data_dir(&base_dir.to_string_lossy(), true)
            .is_empty()
            || crate::modules::process::is_antigravity_running(None);

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
            None => {
                let turn_sec = if turn.created_at > 100_000_000_000 {
                    turn.created_at / 1000
                } else {
                    turn.created_at
                };
                let is_recent = turn_sec > 0 && (now - turn_sec <= 600);
                let is_beyond_last_completed = match last_completed_model_step_idx {
                    Some(last_comp) => turn.step_index > last_comp,
                    None => true,
                };
                if is_beyond_last_completed && (is_conv_running || (is_inst_alive && is_recent)) {
                    ("QUEUED".to_string(), false, true)
                } else {
                    ("COMPLETED".to_string(), false, false)
                }
            }
        };

        let (preview_200w, _) =
            super::sequences::extract_prompt_words_preview(&turn.prompt_text, 200);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subagent_prompt_text() {
        assert!(is_subagent_prompt_text("You are Worker 02"));
        assert!(is_subagent_prompt_text("Task delegated to subagent"));
        assert!(is_subagent_prompt_text(
            "invoked by a caller agent to run tasks"
        ));
        assert!(is_subagent_prompt_text(
            "execute enhanced read memory protocol"
        ));
        assert!(!is_subagent_prompt_text(
            "Please fix the bug in this function"
        ));
    }

    #[test]
    fn test_is_telemetry_event() {
        assert!(is_telemetry_event("TOKEN_USAGE"));
        assert!(is_telemetry_event("TELEMETRY"));
        assert!(is_telemetry_event("HEARTBEAT"));
        assert!(is_telemetry_event("PROGRESS"));
        assert!(is_telemetry_event("METRICS"));
        assert!(is_telemetry_event("SYSTEM_LOG"));
        assert!(!is_telemetry_event("USER_INPUT"));
        assert!(!is_telemetry_event("PLANNER_RESPONSE"));
    }

    #[test]
    fn test_is_tool_execution_step() {
        let empty_val = serde_json::json!({});
        let tool_val = serde_json::json!({ "tool_call_id": "call_123" });

        assert!(is_tool_execution_step("GENERIC", "", &empty_val));
        assert!(is_tool_execution_step("TOOL_OUTPUT", "", &empty_val));
        assert!(is_tool_execution_step("TOOL_RESPONSE", "", &empty_val));
        assert!(is_tool_execution_step("", "TOOL", &empty_val));
        assert!(is_tool_execution_step("", "", &tool_val));
        assert!(!is_tool_execution_step("USER_INPUT", "USER", &empty_val));
    }

    #[test]
    fn test_is_terminal_status() {
        assert!(is_terminal_status("DONE"));
        assert!(is_terminal_status("done"));
        assert!(is_terminal_status("COMPLETED"));
        assert!(is_terminal_status("completed"));
        assert!(is_terminal_status("IDLE"));
        assert!(is_terminal_status("idle"));
        assert!(!is_terminal_status("ERROR"));
        assert!(!is_terminal_status("error"));
        assert!(!is_terminal_status("RUNNING"));
        assert!(!is_terminal_status(""));
    }

    #[test]
    fn test_is_error_or_quota_status() {
        assert!(is_error_or_quota_status("ERROR"));
        assert!(is_error_or_quota_status("error"));
        assert!(is_error_or_quota_status("failed"));
        assert!(is_error_or_quota_status("RESOURCE_EXHAUSTED"));
        assert!(is_error_or_quota_status("resource_exhausted"));
        assert!(is_error_or_quota_status("quota_depleted"));
        assert!(is_error_or_quota_status("429"));
        assert!(!is_error_or_quota_status("done"));
        assert!(!is_error_or_quota_status("completed"));
        assert!(!is_error_or_quota_status("idle"));
        assert!(!is_error_or_quota_status("running"));
    }

    #[test]
    fn test_is_planner_turn_completed() {
        assert!(is_planner_turn_completed(false, "DONE"));
        assert!(!is_planner_turn_completed(false, ""));
        assert!(!is_planner_turn_completed(true, "DONE"));
        assert!(!is_planner_turn_completed(false, "RUNNING"));
        assert!(!is_planner_turn_completed(false, "ERROR"));
    }

    #[test]
    fn test_is_step_error_or_quota() {
        let err_step = serde_json::json!({
            "type": "PLANNER_RESPONSE",
            "status": "error",
            "content": "429 Resource Exhausted: Quota limit exceeded"
        });
        assert!(is_step_error_or_quota(&err_step));

        let ok_step = serde_json::json!({
            "type": "PLANNER_RESPONSE",
            "status": "done",
            "content": "All items completed successfully."
        });
        assert!(!is_step_error_or_quota(&ok_step));

        let user_step = serde_json::json!({
            "type": "USER_INPUT",
            "source": "USER",
            "content": "What is my quota limit?"
        });
        assert!(!is_step_error_or_quota(&user_step));
    }

    #[test]
    fn test_is_genuine_user_input() {
        assert!(is_genuine_user_input("USER_INPUT", "USER"));
        assert!(is_genuine_user_input("SOMETHING", "USER_EXPLICIT"));
        assert!(!is_genuine_user_input("USER_INPUT", "SYSTEM"));
        assert!(!is_genuine_user_input("PLANNER_RESPONSE", "MODEL"));
    }

    #[test]
    fn test_is_system_prompt_message() {
        assert!(is_system_prompt_message("SYSTEM", "Any text"));
        assert!(is_system_prompt_message(
            "USER",
            "<system_message>System prompt</system_message>"
        ));
        assert!(is_system_prompt_message(
            "USER",
            "The following is a <system_message>"
        ));
        assert!(is_system_prompt_message("USER", "[message] timestamp=123"));
        assert!(is_system_prompt_message("USER", "Task id 123"));
        assert!(!is_system_prompt_message(
            "USER",
            "Hello, please write a test"
        ));
    }

    #[test]
    fn test_is_content_truncated() {
        let val_truncated = serde_json::json!({ "truncated_fields": ["content", "thinking"] });
        let val_other = serde_json::json!({ "truncated_fields": ["thinking"] });
        let val_none = serde_json::json!({});

        assert!(is_content_truncated(&val_truncated));
        assert!(!is_content_truncated(&val_other));
        assert!(!is_content_truncated(&val_none));
    }

    #[test]
    fn test_is_relevant_turn_candidate() {
        assert!(is_relevant_turn_candidate(r#"{"type":"USER_INPUT"}"#));
        assert!(is_relevant_turn_candidate(r#"{"source":"USER_EXPLICIT"}"#));
        assert!(is_relevant_turn_candidate(r#"{"source":"USER"}"#));
        assert!(is_relevant_turn_candidate(r#"{"source":"user"}"#));
        assert!(is_relevant_turn_candidate(r#"{"type":"PLANNER_RESPONSE"}"#));
        assert!(is_relevant_turn_candidate(r#"{"source":"MODEL"}"#));
        assert!(is_relevant_turn_candidate(r#"{"source":"model"}"#));
        assert!(!is_relevant_turn_candidate(
            r#"{"type":"GENERIC","source":"TOOL"}"#
        ));
    }
}
