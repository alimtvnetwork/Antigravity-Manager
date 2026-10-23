//! No Silent Errors Justification Auditor.
//!
//! Enforces the AGENTS.md "No Silent Errors" invariant:
//! Every fallible operation must route its failure into the error module.
//! Swallowing an error — discarding it where no one can ever see it — is forbidden.
//!
//! Rust (`src-tauri/src/`):
//! - Errors flow into `AppError` (`src-tauri/src/error.rs`) and are logged via `tracing`.
//! - Never leave a bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result`
//!   without routing the `Err`.
//! - For genuinely best-effort ops (cleanup, best-effort notifies):
//!   `crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file")`
//!   plus a one-line `// Justification:` comment explaining why it is safe to continue.

use serde::{Deserialize, Serialize};

/// Recognized silent error violation variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SilentErrorViolation {
    BareRecordIgnoredWithoutJustification {
        file: String,
        line: usize,
    },
    BareLetUnderscoreResultDiscard {
        file: String,
        line: usize,
        snippet: String,
    },
    BareOkResultDiscard {
        file: String,
        line: usize,
        snippet: String,
    },
}

impl std::fmt::Display for SilentErrorViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BareRecordIgnoredWithoutJustification { file, line } => {
                write!(
                    f,
                    "{}:{}: record_ignored call lacks mandatory '// Justification:' comment",
                    file, line
                )
            }
            Self::BareLetUnderscoreResultDiscard { file, line, snippet } => {
                write!(
                    f,
                    "{}:{}: bare 'let _ = ' discards fallible result: {}",
                    file, line, snippet
                )
            }
            Self::BareOkResultDiscard { file, line, snippet } => {
                write!(
                    f,
                    "{}:{}: bare '.ok()' discards error without routing: {}",
                    file, line, snippet
                )
            }
        }
    }
}

impl std::error::Error for SilentErrorViolation {}

/// Audit report detailing silent error violations and valid justifications.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SilentErrorsAuditReport {
    pub is_clean: bool,
    pub total_sites_audited: usize,
    pub valid_justifications_count: usize,
    pub violations: Vec<SilentErrorViolation>,
}

impl SilentErrorsAuditReport {
    pub fn new() -> Self {
        Self {
            is_clean: true,
            total_sites_audited: 0,
            valid_justifications_count: 0,
            violations: Vec::new(),
        }
    }
}

impl std::fmt::Display for SilentErrorsAuditReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_clean {
            write!(
                f,
                "Silent Errors Audit Clean: {} audited sites, {} valid justifications",
                self.total_sites_audited, self.valid_justifications_count
            )
        } else {
            write!(
                f,
                "Silent Errors Audit Violations ({} found across {} sites):",
                self.violations.len(),
                self.total_sites_audited
            )?;
            for v in &self.violations {
                write!(f, "\n  - {}", v)?;
            }
            Ok(())
        }
    }
}

/// Identifies if a trimmed line is a comment.
pub fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

/// Checks whether a line or its immediate neighbors contain a valid `// Justification:` comment.
pub fn check_has_justification(lines: &[&str], line_idx: usize) -> bool {
    if lines[line_idx].contains("// Justification:") {
        return true;
    }
    let mut idx = line_idx;
    let mut checked = 0;
    while idx > 0 && checked < 3 {
        idx -= 1;
        let line = lines[idx].trim();
        if line.is_empty() || line == "{" {
            continue;
        }
        checked += 1;
        if line.contains("// Justification:") {
            return true;
        }
        if !line.starts_with("//") {
            break;
        }
    }
    false
}

/// Evaluates if an expression contains known fallible calls.
pub fn is_fallible_let_expression(line: &str) -> bool {
    const FALLIBLE_KEYWORDS: &[&str] = &[
        "fs::",
        "remove_file",
        "remove_dir",
        "write(",
        "write_all",
        "create_dir",
        "read_to_string",
        "File::",
        "Command::",
        "invoke(",
        "spawn(",
        "send(",
        "delete(",
        "rename(",
        "copy(",
        "execute(",
        "reqwest::",
        "Result",
    ];
    FALLIBLE_KEYWORDS.iter().any(|&kw| line.contains(kw))
}

/// Audits a single line within source lines and accumulates violations.
fn audit_line(
    lines: &[&str],
    line_idx: usize,
    file_name: &str,
    report: &mut SilentErrorsAuditReport,
) {
    let line = lines[line_idx];
    let trimmed = line.trim();
    if trimmed.is_empty() || is_comment_line(trimmed) {
        return;
    }

    if trimmed.contains("record_ignored") {
        report.total_sites_audited += 1;
        if check_has_justification(lines, line_idx) {
            report.valid_justifications_count += 1;
        } else {
            report.violations.push(
                SilentErrorViolation::BareRecordIgnoredWithoutJustification {
                    file: file_name.to_string(),
                    line: line_idx + 1,
                },
            );
        }
        return;
    }

    if trimmed.contains(".ok()") {
        report.total_sites_audited += 1;
        report.violations.push(SilentErrorViolation::BareOkResultDiscard {
            file: file_name.to_string(),
            line: line_idx + 1,
            snippet: trimmed.to_string(),
        });
        return;
    }

    if trimmed.contains("let _ =") && is_fallible_let_expression(trimmed) {
        report.total_sites_audited += 1;
        report.violations.push(
            SilentErrorViolation::BareLetUnderscoreResultDiscard {
                file: file_name.to_string(),
                line: line_idx + 1,
                snippet: trimmed.to_string(),
            },
        );
    }
}

/// Audits source code text for silent error swallowing patterns.
pub fn audit_source_for_silent_errors(
    source_code: &str,
    file_name: &str,
) -> SilentErrorsAuditReport {
    let mut report = SilentErrorsAuditReport::new();
    let lines: Vec<&str> = source_code.lines().collect();

    for line_idx in 0..lines.len() {
        audit_line(&lines, line_idx, file_name, &mut report);
    }

    report.is_clean = report.violations.is_empty();
    report
}
