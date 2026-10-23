//! Deep Transcript Inspection & Full Log Precedence Reverse Scanner.
//!
//! Enforces transcript inspection discipline per AGENTS.md:
//! When inspecting conversation transcripts, always prioritize un-truncated
//! `transcript_full.jsonl` over `transcript.jsonl`. Reverse-scan across at least
//! 10 lines, bypassing trailing `TOKEN_USAGE`, `HEARTBEAT`, and `TELEMETRY` records,
//! to extract genuine tool executions (`tool_calls`) and assistant thoughts (`thinking`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NOISE_RECORD_TYPES: &[&str] = &[
    "TOKEN_USAGE",
    "HEARTBEAT",
    "TELEMETRY",
    "METRICS",
    "PING",
];
pub const MINIMUM_REVERSE_SCAN_LINES: usize = 10;
pub const TRANSCRIPT_FULL_FILENAME: &str = "transcript_full.jsonl";
pub const TRANSCRIPT_COMPACT_FILENAME: &str = "transcript.jsonl";

/// Distinct record variants identified during transcript reverse scanning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptRecordType {
    ToolCalls(Vec<String>),
    ThinkingContent(String),
    UserPrompt(String),
    IgnoredNoise(String),
}

impl std::fmt::Display for TranscriptRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolCalls(calls) => write!(f, "ToolCalls({:?})", calls),
            Self::ThinkingContent(t) => write!(f, "ThinkingContent({} chars)", t.len()),
            Self::UserPrompt(p) => write!(f, "UserPrompt({} chars)", p.len()),
            Self::IgnoredNoise(n) => write!(f, "IgnoredNoise({})", n),
        }
    }
}

/// Extracted execution artifact with line coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedExecutionArtifact {
    pub line_number: usize,
    pub artifact_type: TranscriptRecordType,
}

impl ExtractedExecutionArtifact {
    pub fn new(line_number: usize, artifact_type: TranscriptRecordType) -> Self {
        Self {
            line_number,
            artifact_type,
        }
    }

    pub fn is_genuine(&self) -> bool {
        !matches!(self.artifact_type, TranscriptRecordType::IgnoredNoise(_))
    }
}

impl std::fmt::Display for ExtractedExecutionArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line_number, self.artifact_type)
    }
}

/// Comprehensive report produced by reverse scanning conversation transcripts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseScanReport {
    pub is_full_log_preferred: bool,
    pub total_lines_scanned: usize,
    pub noise_records_skipped: usize,
    pub extracted_artifacts: Vec<ExtractedExecutionArtifact>,
    pub found_genuine_execution: bool,
}

impl ReverseScanReport {
    pub fn new(is_full_log_preferred: bool) -> Self {
        Self {
            is_full_log_preferred,
            total_lines_scanned: 0,
            noise_records_skipped: 0,
            extracted_artifacts: Vec::new(),
            found_genuine_execution: false,
        }
    }
}

impl std::fmt::Display for ReverseScanReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ReverseScanReport: preferred_full={}, scanned={}, skipped_noise={}, artifacts={}, genuine={}",
            self.is_full_log_preferred,
            self.total_lines_scanned,
            self.noise_records_skipped,
            self.extracted_artifacts.len(),
            self.found_genuine_execution
        )
    }
}

/// Selects the optimal transcript file, enforcing un-truncated log precedence.
pub fn select_transcript_file(full_exists: bool, compact_exists: bool) -> Option<&'static str> {
    if full_exists {
        Some(TRANSCRIPT_FULL_FILENAME)
    } else if compact_exists {
        Some(TRANSCRIPT_COMPACT_FILENAME)
    } else {
        None
    }
}

/// Evaluates if a given transcript log line matches recognized noise types.
pub fn is_noise_record(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    NOISE_RECORD_TYPES.iter().any(|&noise| upper.contains(noise))
}

/// Extracts tool call names or string representations from parsed JSON Value.
fn extract_tool_calls_from_json(val: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(calls) = val.get("tool_calls").and_then(|v| v.as_array()) {
        for call in calls {
            if let Some(name) = call.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            } else if let Some(s) = call.as_str() {
                names.push(s.to_string());
            } else {
                names.push(call.to_string());
            }
        }
    }
    names
}

/// Extracts tool calls with JSON parsing and substring fallback.
fn extract_tool_calls_from_raw(line: &str) -> Vec<String> {
    if let Ok(val) = serde_json::from_str::<Value>(line) {
        let tools = extract_tool_calls_from_json(&val);
        if !tools.is_empty() {
            return tools;
        }
    }
    vec!["tool_call".to_string()]
}

/// Extracts thinking thoughts with JSON parsing and line fallback.
fn extract_thinking_from_raw(line: &str) -> String {
    if let Ok(val) = serde_json::from_str::<Value>(line) {
        if let Some(thinking) = val.get("thinking").and_then(|v| v.as_str()) {
            return thinking.to_string();
        }
    }
    line.to_string()
}

/// Extracts user prompt content with JSON parsing and line fallback.
fn extract_user_prompt_from_raw(line: &str) -> String {
    if let Ok(val) = serde_json::from_str::<Value>(line) {
        if let Some(content) = val.get("content").and_then(|v| v.as_str()) {
            return content.to_string();
        }
    }
    line.to_string()
}

/// Scans a single line and appends recognized genuine artifacts.
fn scan_single_line(line: &str, line_number: usize, artifacts: &mut Vec<ExtractedExecutionArtifact>) {
    if line.contains("\"tool_calls\"") {
        let tools = extract_tool_calls_from_raw(line);
        artifacts.push(ExtractedExecutionArtifact::new(
            line_number,
            TranscriptRecordType::ToolCalls(tools),
        ));
    }
    if line.contains("\"thinking\"") {
        let thought = extract_thinking_from_raw(line);
        artifacts.push(ExtractedExecutionArtifact::new(
            line_number,
            TranscriptRecordType::ThinkingContent(thought),
        ));
    }
    if line.contains("\"USER_INPUT\"") {
        let prompt = extract_user_prompt_from_raw(line);
        artifacts.push(ExtractedExecutionArtifact::new(
            line_number,
            TranscriptRecordType::UserPrompt(prompt),
        ));
    }
}

/// Scans lines backwards up to `max_scan_lines` (minimum 10 lines).
pub fn reverse_scan_transcript(lines: &[String], max_scan_lines: usize) -> ReverseScanReport {
    let mut report = ReverseScanReport::new(true);
    let scan_depth = max_scan_lines.max(MINIMUM_REVERSE_SCAN_LINES);
    let total_lines = lines.len();
    let start_idx = total_lines.saturating_sub(scan_depth);

    for idx in (start_idx..total_lines).rev() {
        let raw_line = &lines[idx];
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        report.total_lines_scanned += 1;
        let line_num = idx + 1;

        if is_noise_record(trimmed) {
            report.noise_records_skipped += 1;
            continue;
        }
        scan_single_line(trimmed, line_num, &mut report.extracted_artifacts);
    }

    report.found_genuine_execution = !report.extracted_artifacts.is_empty();
    report
}

/// Helper to scan string slice inputs.
pub fn reverse_scan_transcript_str(lines: &[&str], max_scan_lines: usize) -> ReverseScanReport {
    let owned_lines: Vec<String> = lines.iter().map(|&s| s.to_string()).collect();
    reverse_scan_transcript(&owned_lines, max_scan_lines)
}
