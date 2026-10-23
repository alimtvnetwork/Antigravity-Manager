//! Cargo Clippy Zero-Warnings Preflight Gate (`Imp-651`).
//!
//! Enforces pre-flight quality gates per AGENTS.md:
//! "Pre-flight Checks: Run the essentials before submitting PRs or release tags:
//! - `cd src-tauri && cargo clippy --all-targets --all-features` (comprehensive Rust gate,
//!   already includes compilation — no separate `cargo check` needed).
//!   Must verify zero warnings and zero compilation errors."

use serde::{Deserialize, Serialize};

/// Severity level of a compiler or clippy diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClippyDiagnosticLevel {
    Error,
    Warning,
    Help,
    Note,
}

impl ClippyDiagnosticLevel {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    pub fn is_help(&self) -> bool {
        matches!(self, Self::Help)
    }

    pub fn is_note(&self) -> bool {
        matches!(self, Self::Note)
    }
}

/// Structured diagnostic item parsed from compiler or clippy output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClippyDiagnostic {
    pub level: ClippyDiagnosticLevel,
    pub message: String,
    pub code: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub is_fatal: bool,
}

impl ClippyDiagnostic {
    pub fn is_fatal(&self) -> bool {
        self.is_fatal
    }

    pub fn is_warning(&self) -> bool {
        self.level.is_warning()
    }

    pub fn is_error(&self) -> bool {
        self.level.is_error()
    }
}

/// Aggregate audit report for cargo clippy preflight evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClippyAuditReport {
    pub is_clean: bool,
    pub total_diagnostics: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<ClippyDiagnostic>,
}

impl ClippyAuditReport {
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    pub fn is_clean(&self) -> bool {
        self.is_clean
    }
}

#[derive(Deserialize)]
struct CargoJsonEnvelope {
    reason: Option<String>,
    message: Option<CargoJsonMessage>,
}

#[derive(Deserialize)]
struct CargoJsonMessage {
    level: String,
    message: String,
    code: Option<CargoJsonCode>,
    spans: Option<Vec<CargoJsonSpan>>,
}

#[derive(Deserialize)]
struct CargoJsonCode {
    code: String,
}

#[derive(Deserialize)]
struct CargoJsonSpan {
    file_name: String,
    line_start: usize,
}

/// Identifies compiler summary footers that should not be counted as separate diagnostics.
pub fn is_compiler_summary_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("error: aborting due to") {
        return true;
    }
    if trimmed.starts_with("error: could not compile") {
        return true;
    }
    if trimmed.starts_with("warning:")
        && (trimmed.ends_with("emitted")
            || (trimmed.contains("generated ") && trimmed.contains("warning")))
    {
        return true;
    }
    false
}

/// Parses the severity level and code from a text diagnostic header line.
fn parse_level_tail(
    level: ClippyDiagnosticLevel,
    rest: &str,
) -> Option<(ClippyDiagnosticLevel, Option<String>, String)> {
    if let Some(msg) = rest.strip_prefix(": ") {
        Some((level, None, msg.trim().to_string()))
    } else if rest.starts_with('[') {
        let closing = rest.find(']')?;
        let code = rest[1..closing].trim().to_string();
        let after_bracket = &rest[closing + 1..];
        let msg = after_bracket
            .strip_prefix(": ")
            .unwrap_or(after_bracket)
            .trim();
        Some((level, Some(code), msg.to_string()))
    } else {
        None
    }
}

/// Parses human-readable compiler diagnostic header lines.
pub fn parse_text_diagnostic_header(
    line: &str,
) -> Option<(ClippyDiagnosticLevel, Option<String>, String)> {
    let trimmed = line.trim();
    if is_compiler_summary_line(trimmed) {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("error") {
        parse_level_tail(ClippyDiagnosticLevel::Error, rest)
    } else if let Some(rest) = trimmed.strip_prefix("warning") {
        parse_level_tail(ClippyDiagnosticLevel::Warning, rest)
    } else if let Some(rest) = trimmed.strip_prefix("help:") {
        Some((ClippyDiagnosticLevel::Help, None, rest.trim().to_string()))
    } else if let Some(rest) = trimmed.strip_prefix("note:") {
        Some((ClippyDiagnosticLevel::Note, None, rest.trim().to_string()))
    } else {
        None
    }
}

/// Parses source location `--> path:line:col` from text compiler output.
pub fn parse_source_location(line: &str) -> Option<(String, usize)> {
    let trimmed = line.trim();
    let after_arrow = trimmed.strip_prefix("-->")?.trim();
    let parts: Vec<&str> = after_arrow.split(':').collect();

    if parts.len() >= 3 && parts[0].len() == 1 && parts[0].chars().all(|c| c.is_ascii_alphabetic())
    {
        // Windows drive letter format: C:\path\to\file:line:col
        let file_path = format!("{}:{}", parts[0], parts[1]);
        let line_num = parts[2].trim().parse::<usize>().ok()?;
        Some((file_path, line_num))
    } else if parts.len() >= 2 {
        let file_path = parts[0].trim().to_string();
        let line_num = parts[1].trim().parse::<usize>().ok()?;
        Some((file_path, line_num))
    } else {
        None
    }
}

/// Extracts lint rule code from compiler note lines (e.g. `#[warn(clippy::...)]`).
pub fn extract_code_from_note(line: &str) -> Option<String> {
    let trimmed = line.trim();
    for prefix in &["#[warn(", "#[deny(", "`#[warn(", "`#[deny("] {
        if let Some(pos) = trimmed.find(prefix) {
            let start = pos + prefix.len();
            let after = &trimmed[start..];
            if let Some(end) = after.find(')') {
                let code = after[..end].trim().trim_matches('`').trim();
                if !code.is_empty() {
                    return Some(code.to_string());
                }
            }
        }
    }
    None
}

/// Parses a single JSON line into a structured diagnostic item.
pub fn parse_json_diagnostic_line(line: &str) -> Option<ClippyDiagnostic> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') {
        return None;
    }

    let envelope = serde_json::from_str::<CargoJsonEnvelope>(trimmed).ok()?;
    if envelope.reason.as_deref() != Some("compiler-message") {
        return None;
    }

    let msg = envelope.message?;
    let level = match msg.level.as_str() {
        "error" => ClippyDiagnosticLevel::Error,
        "warning" => ClippyDiagnosticLevel::Warning,
        "help" => ClippyDiagnosticLevel::Help,
        "note" => ClippyDiagnosticLevel::Note,
        _ => return None,
    };

    let is_fatal = level.is_error();
    let code = msg.code.map(|c| c.code);
    let span = msg.spans.and_then(|s| s.into_iter().next());
    let file_path = span.as_ref().map(|s| s.file_name.clone());
    let line_number = span.as_ref().map(|s| s.line_start);

    Some(ClippyDiagnostic {
        level,
        message: msg.message,
        code,
        file_path,
        line_number,
        is_fatal,
    })
}

/// Parses full text compiler output into structured diagnostics.
pub fn parse_text_diagnostics(output: &str) -> Vec<ClippyDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut current: Option<ClippyDiagnostic> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((level, code, msg)) = parse_text_diagnostic_header(trimmed) {
            if let Some(prev) = current.take() {
                diagnostics.push(prev);
            }
            let is_fatal = level.is_error();
            current = Some(ClippyDiagnostic {
                level,
                message: msg,
                code,
                file_path: None,
                line_number: None,
                is_fatal,
            });
            continue;
        }

        if let Some(ref mut diag) = current {
            if diag.file_path.is_none() {
                if let Some((path, line_no)) = parse_source_location(trimmed) {
                    diag.file_path = Some(path);
                    diag.line_number = Some(line_no);
                    continue;
                }
            }

            if diag.code.is_none() {
                if let Some(code) = extract_code_from_note(trimmed) {
                    diag.code = Some(code);
                }
            }
        }
    }

    if let Some(prev) = current {
        diagnostics.push(prev);
    }

    diagnostics
}

/// Parses JSON line compiler output into structured diagnostics.
pub fn parse_json_diagnostics(output: &str) -> Vec<ClippyDiagnostic> {
    let mut diagnostics = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            if let Some(diag) = parse_json_diagnostic_line(trimmed) {
                diagnostics.push(diag);
            }
        }
    }
    diagnostics
}

/// Parses either JSON or human-readable text compiler diagnostics.
pub fn parse_clippy_diagnostics(output: &str) -> Vec<ClippyDiagnostic> {
    let json_diags = parse_json_diagnostics(output);
    if !json_diags.is_empty() {
        json_diags
    } else {
        parse_text_diagnostics(output)
    }
}

/// Evaluates cargo clippy compiler output, producing a clean/dirty audit report.
pub fn parse_clippy_output(output: &str) -> ClippyAuditReport {
    let diagnostics = parse_clippy_diagnostics(output);
    let mut error_count = 0;
    let mut warning_count = 0;

    for diag in &diagnostics {
        match diag.level {
            ClippyDiagnosticLevel::Error => error_count += 1,
            ClippyDiagnosticLevel::Warning => warning_count += 1,
            _ => {}
        }
    }

    let total_diagnostics = diagnostics.len();
    let is_clean = error_count == 0 && warning_count == 0;

    ClippyAuditReport {
        is_clean,
        total_diagnostics,
        error_count,
        warning_count,
        diagnostics,
    }
}
