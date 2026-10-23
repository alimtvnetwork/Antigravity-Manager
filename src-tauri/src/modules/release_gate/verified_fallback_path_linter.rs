//! Verified Fallback Path & Risky Path Replacement Linter.
//!
//! Enforces risky path safety per AGENTS.md:
//! - "Keep an explicit fallback to the verified path whenever you replace it
//!    (e.g. '0 matches found → full restart'). A silent no-op is worse than the
//!    failure it was meant to fix."
//! - "Order side effects to fail before the point of no return: write credentials
//!    before killing a process, validate before deleting."
//! - "Detect broadly, act narrowly: a matcher may recognize a whole class of
//!    problems, while its effect stays inside the intended data — not across line
//!    breaks, tags, or other clauses. Bound every wait with a timeout."

use serde::{Deserialize, Serialize};
use std::cmp;
use std::fmt;

/// Violations identified when auditing risky path replacements and fallbacks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackLinterViolation {
    /// Replacement construct has no explicit fallback branch to verified execution.
    MissingVerifiedFallbackBranch {
        file: String,
        line: usize,
        construct: String,
    },
    /// A zero-matches branch returns silently without executing verified fallback.
    SilentNoopOnZeroMatches { file: String, line: usize },
}

impl fmt::Display for FallbackLinterViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVerifiedFallbackBranch {
                file,
                line,
                construct,
            } => {
                write!(
                    f,
                    "{}:{}: Replacement construct '{}' lacks verified fallback branch",
                    file, line, construct
                )
            }
            Self::SilentNoopOnZeroMatches { file, line } => {
                write!(
                    f,
                    "{}:{}: Forbidden silent no-op return on zero matches without explicit fallback",
                    file, line
                )
            }
        }
    }
}

impl std::error::Error for FallbackLinterViolation {}

/// Comprehensive audit report detailing fallback safety across replacement sites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackAuditReport {
    pub is_safe: bool,
    pub replacement_sites_scanned: usize,
    pub verified_fallbacks_found: usize,
    pub violations: Vec<FallbackLinterViolation>,
}

impl FallbackAuditReport {
    /// Constructs a clean fallback audit report.
    pub fn new() -> Self {
        Self {
            is_safe: true,
            replacement_sites_scanned: 0,
            verified_fallbacks_found: 0,
            violations: Vec::new(),
        }
    }

    /// Returns true if any fallback violations were detected.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Formats a concise summary of the fallback safety audit.
    pub fn summary(&self) -> String {
        if self.is_safe {
            format!(
                "Safe: {} replacement site(s) scanned, {} verified fallback(s) confirmed",
                self.replacement_sites_scanned, self.verified_fallbacks_found
            )
        } else {
            format!(
                "Unsafe: {} violation(s) detected across {} replacement site(s)",
                self.violations.len(),
                self.replacement_sites_scanned
            )
        }
    }
}

impl Default for FallbackAuditReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FallbackAuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())?;
        for v in &self.violations {
            write!(f, "\n  - {}", v)?;
        }
        Ok(())
    }
}

/// Identifies if a trimmed source code line is a comment.
pub fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

/// Detects if a line specifies a zero-matches or empty check.
pub fn is_zero_matches_check(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let has_if = lower.contains("if ") || lower.contains("if(");
    let is_zero_or_empty = lower.contains("is_empty()")
        || lower.contains("count == 0")
        || lower.contains("matches == 0")
        || lower.contains("found == 0")
        || lower.contains(".len() == 0");
    has_if && is_zero_or_empty
}

/// Detects if a line initiates a pattern matching construct on replacement results.
pub fn is_pattern_match_construct(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let trimmed = lower.trim_start();
    trimmed.starts_with("match ")
        && (trimmed.contains("result")
            || trimmed.contains("matches")
            || trimmed.contains("replacement")
            || trimmed.contains("res"))
}

/// Checks if a code window invokes verified fallback, restart, or reload logic.
pub fn has_fallback_invocation(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("fallback") || lower.contains("restart") || lower.contains("reload")
}

/// Checks if a code window contains an unhandled silent no-op return.
pub fn has_silent_noop_return(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("return ok(())")
        || lower.contains("return ok(vec![])")
        || lower.contains("return ok(none)")
        || lower.contains("return ok(0)")
        || lower.contains("return;")
}

/// Extracts a lookahead code window of up to 8 lines starting at the target index.
pub fn extract_block_window(lines: &[&str], start_idx: usize) -> String {
    let end_idx = cmp::min(start_idx + 8, lines.len());
    lines[start_idx..end_idx].join("\n")
}

/// Evaluates a zero-matches check site for safe fallback execution.
fn evaluate_zero_matches_site(
    lines: &[&str],
    line_idx: usize,
    file_name: &str,
    report: &mut FallbackAuditReport,
) {
    report.replacement_sites_scanned += 1;
    let block = extract_block_window(lines, line_idx);
    if has_fallback_invocation(&block) {
        report.verified_fallbacks_found += 1;
    } else if has_silent_noop_return(&block) {
        report
            .violations
            .push(FallbackLinterViolation::SilentNoopOnZeroMatches {
                file: file_name.to_string(),
                line: line_idx + 1,
            });
    }
}

/// Evaluates a pattern match construct site for explicit fallback branches.
fn evaluate_pattern_match_site(
    lines: &[&str],
    line_idx: usize,
    trimmed: &str,
    file_name: &str,
    report: &mut FallbackAuditReport,
) {
    report.replacement_sites_scanned += 1;
    let block = extract_block_window(lines, line_idx);
    if has_fallback_invocation(&block) {
        report.verified_fallbacks_found += 1;
    } else if has_silent_noop_return(&block) {
        report
            .violations
            .push(FallbackLinterViolation::SilentNoopOnZeroMatches {
                file: file_name.to_string(),
                line: line_idx + 1,
            });
    } else {
        report
            .violations
            .push(FallbackLinterViolation::MissingVerifiedFallbackBranch {
                file: file_name.to_string(),
                line: line_idx + 1,
                construct: trimmed.to_string(),
            });
    }
}

/// Inspects a candidate line for replacement sites and verifies fallback safety.
fn inspect_fallback_site(
    lines: &[&str],
    line_idx: usize,
    trimmed: &str,
    file_name: &str,
    report: &mut FallbackAuditReport,
) {
    if is_zero_matches_check(trimmed) {
        evaluate_zero_matches_site(lines, line_idx, file_name, report);
    } else if is_pattern_match_construct(trimmed) {
        evaluate_pattern_match_site(lines, line_idx, trimmed, file_name, report);
    }
}

/// Audits source code for verified fallback paths and detects forbidden silent no-ops.
pub fn audit_fallback_paths(source_code: &str, file_name: &str) -> FallbackAuditReport {
    let mut report = FallbackAuditReport::new();
    let lines: Vec<&str> = source_code.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) {
            continue;
        }
        inspect_fallback_site(&lines, line_idx, trimmed, file_name, &mut report);
    }

    report.is_safe = report.violations.is_empty();
    report
}
