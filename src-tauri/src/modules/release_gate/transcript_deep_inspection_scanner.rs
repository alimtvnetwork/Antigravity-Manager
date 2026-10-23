//! Deep Transcript Inspection & Full Log Precedence Scanner.
//!
//! Enforces conversation transcript inspection discipline per AGENTS.md:
//! When inspecting conversation transcripts, always prioritize un-truncated `transcript_full.jsonl`
//! over `transcript.jsonl`. Reverse-scan across at least 10 lines, bypassing trailing `TOKEN_USAGE`,
//! `HEARTBEAT`, and `TELEMETRY` records, to extract genuine tool executions (`tool_calls`)
//! and assistant thoughts (`thinking`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_REVERSE_SCAN_DEPTH: usize = 10;
pub const TRANSCRIPT_FULL_FILENAME: &str = "transcript_full.jsonl";
pub const TRANSCRIPT_COMPACT_FILENAME: &str = "transcript.jsonl";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TranscriptInspectionReport {
    pub total_scanned: usize,
    pub bypassed_telemetry_count: usize,
    pub tool_calls_count: usize,
    pub thoughts_count: usize,
    pub has_genuine_turn: bool,
    pub selected_file_source: String,
}

/// Selects the optimal transcript log file, enforcing un-truncated log precedence.
pub fn select_transcript_file_source(
    full_log_exists: bool,
    compact_log_exists: bool,
) -> Option<&'static str> {
    if full_log_exists {
        Some(TRANSCRIPT_FULL_FILENAME)
    } else if compact_log_exists {
        Some(TRANSCRIPT_COMPACT_FILENAME)
    } else {
        None
    }
}

/// Detects whether a JSON line represents an ignorable telemetry, heartbeat, or usage record.
pub fn is_ignorable_telemetry_record(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();

    lower.contains("\"token_usage\"")
        || lower.contains("\"heartbeat\"")
        || lower.contains("\"telemetry\"")
        || lower.contains("\"metrics\"")
        || lower.contains("\"ping\"")
}

/// Scans raw JSONL transcript lines in reverse order to extract genuine execution artifacts.
pub fn scan_transcript_lines(
    lines: &[&str],
    scan_depth: usize,
    source_filename: &str,
) -> TranscriptInspectionReport {
    let mut total_scanned = 0;
    let mut bypassed_telemetry_count = 0;
    let mut tool_calls_count = 0;
    let mut thoughts_count = 0;

    let scan_limit = scan_depth.max(DEFAULT_REVERSE_SCAN_DEPTH);

    for &line in lines.iter().rev().take(scan_limit) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        total_scanned += 1;

        if is_ignorable_telemetry_record(trimmed) {
            bypassed_telemetry_count += 1;
            continue;
        }

        if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
            if let Some(tool_calls) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
                if !tool_calls.is_empty() {
                    tool_calls_count += tool_calls.len();
                }
            }

            if let Some(thinking) = parsed.get("thinking").and_then(|v| v.as_str()) {
                if !thinking.trim().is_empty() {
                    thoughts_count += 1;
                }
            }
        }
    }

    let has_genuine_turn = tool_calls_count > 0 || thoughts_count > 0;

    TranscriptInspectionReport {
        total_scanned,
        bypassed_telemetry_count,
        tool_calls_count,
        thoughts_count,
        has_genuine_turn,
        selected_file_source: source_filename.to_string(),
    }
}
