//! No Silent Errors Linter.
//!
//! Enforces the AGENTS.md "No Silent Errors" invariant:
//! Every fallible operation must route its failure into the error module.
//! Swallowing an error — discarding it where no one can ever see it — is strictly forbidden.
//!
//! Rust (`src-tauri/src/`):
//! - Errors flow into `AppError` (`src-tauri/src/error.rs`) and are logged via `tracing`.
//! - Never leave a bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result`
//!   without routing the `Err`.
//! - For genuinely best-effort operations (cleanup, best-effort notifies):
//!   `crate::error::record_ignored(...)` plus a one-line `// Justification:` comment
//!   explaining why it is safe to continue are both required.
//!
//! TypeScript (`src/`):
//! - Failures go through the error store (`src/stores/error-store.ts`).
//! - Use `captureError`/`captureException` for user-facing failures.
//! - Use `trackWarning` for benign/best-effort failures.
//! - Bare `catch {}`, `.catch(() => {})`, or `catch (e) { console.warn(e); }` without routing are forbidden.

use serde::{Deserialize, Serialize};

/// Specific kind of silent error violation detected by the linter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SilentErrorViolationKind {
    /// Bare `let _ = <fallible>` discarding a fallible operation's result.
    BareLetUnderscore { expression: String },
    /// Calling `.ok()` on a `Result` to silently discard error without routing.
    BareOkDiscarded { expression: String },
    /// Silently falling back via `.unwrap_or_default()` on a fallible operation.
    BareUnwrapOrDefaultOnFallible { expression: String },
    /// Calling `record_ignored(...)` without the mandatory `// Justification:` comment.
    RecordIgnoredMissingJustification { line: String },
    /// Empty `catch {}` or `catch (e) {}` block discarding errors in TypeScript.
    BareCatchBlock { snippet: String },
    /// Promise `.catch(() => {})` discarding rejection in TypeScript.
    SwallowedPromiseCatch { snippet: String },
    /// Catch block logging exclusively to console without error store routing.
    ConsoleOnlyCatchWithoutErrorStore { snippet: String },
}

/// Detailed violation record with file context and actionable remediation suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentErrorViolation {
    pub line_number: usize,
    pub kind: SilentErrorViolationKind,
    pub snippet: String,
    pub suggestion: String,
}

/// Comprehensive report produced by the No Silent Errors linter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentErrorLintReport {
    pub is_clean: bool,
    pub violations: Vec<SilentErrorViolation>,
    pub best_effort_recorded_count: usize,
    pub total_lines_scanned: usize,
}

/// Checks whether a line or its immediate neighbors contain a valid `// Justification:` comment.
pub fn check_has_justification_comment(lines: &[&str], line_idx: usize) -> bool {
    // 1. Check the same line first
    if let Some(comment) = extract_justification_from_line(lines[line_idx]) {
        if !comment.trim().is_empty() {
            return true;
        }
    }

    // 2. Check preceding lines (up to 3 non-empty lines back)
    let mut lookback = line_idx;
    let mut non_empty_checked = 0;
    while lookback > 0 && non_empty_checked < 3 {
        lookback -= 1;
        let line = lines[lookback].trim();
        if line.is_empty() || line == "{" {
            continue;
        }
        non_empty_checked += 1;
        if let Some(comment) = extract_justification_from_line(line) {
            return !comment.trim().is_empty();
        }
        // If we hit code that isn't a comment, stop searching backward
        if !line.starts_with("//") {
            break;
        }
    }

    // 3. Check the next line (in case comment was placed immediately below)
    if line_idx + 1 < lines.len() {
        let next_line = lines[line_idx + 1].trim();
        if let Some(comment) = extract_justification_from_line(next_line) {
            return !comment.trim().is_empty();
        }
    }

    false
}

/// Extracts justification comment content if present.
fn extract_justification_from_line(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if let Some(pos) = lower.find("// justification:") {
        let content = &line[pos + "// justification:".len()..];
        return Some(content.trim().to_string());
    }
    None
}

/// Generates a conforming Rust best-effort snippet with record_ignored and justification.
pub fn conforming_best_effort_snippet(op_expr: &str, context: &str, justification: &str) -> String {
    format!(
        "// Justification: {}\ncrate::error::record_ignored({}, \"{}\");",
        justification.trim(),
        op_expr.trim(),
        context.trim()
    )
}

/// Generates a conforming TypeScript catch snippet routing through error-store.
pub fn conforming_typescript_catch_snippet(
    err_var: &str,
    source: &str,
    trigger_action: &str,
) -> String {
    format!(
        "catch ({}) {{\n    useErrorStore.getState().trackWarning({}, {{ source: '{}', triggerAction: '{}' }});\n}}",
        err_var.trim(),
        err_var.trim(),
        source.trim(),
        trigger_action.trim()
    )
}

/// Lints Rust source code against the No Silent Errors invariant.
pub fn lint_rust_code(content: &str) -> SilentErrorLintReport {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();
    let mut best_effort_count = 0;
    let mut in_block_comment = false;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Handle block comments /* ... */
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        // Ignore single-line comments or doc comments
        if trimmed.starts_with("//") {
            continue;
        }

        // 1. Check record_ignored
        if trimmed.contains("record_ignored(") {
            if check_has_justification_comment(&lines, idx) {
                best_effort_count += 1;
            } else {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::RecordIgnoredMissingJustification {
                        line: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Add a mandatory one-line '// Justification: <reason>' comment explaining why continuing on failure is safe.".to_string(),
                });
            }
        }

        // 2. Check bare `let _ = <fallible>`
        if (trimmed.starts_with("let _ =") || trimmed.contains("let _ ="))
            && !trimmed.contains("record_ignored")
        {
            // Check if RHS is a fallible call
            let is_fallible_call = trimmed.contains("fs::")
                || trimmed.contains("write(")
                || trimmed.contains("read(")
                || trimmed.contains("remove_")
                || trimmed.contains("delete(")
                || trimmed.contains("send(")
                || trimmed.contains("spawn(")
                || trimmed.contains("execute(")
                || trimmed.contains("query(")
                || trimmed.contains("sync_")
                || trimmed.contains("flush(")
                || trimmed.contains("Command::")
                || (trimmed.contains('(')
                    && trimmed.contains(')')
                    && !trimmed.starts_with("let _ = &")
                    && !trimmed.starts_with("let _ = format!"));

            if is_fallible_call {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::BareLetUnderscore {
                        expression: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Propagate error with '?' via map_err(|e| AppError::...), or use crate::error::record_ignored(...) with a '// Justification:' comment.".to_string(),
                });
            }
        }

        // 3. Check bare `.ok()` statement discarding errors
        if trimmed.contains(".ok()") {
            let is_propagated = trimmed.contains(".ok_or(")
                || trimmed.contains(".ok_or_else(")
                || trimmed.contains(".ok_and(")
                || trimmed.contains(".ok()?");

            let is_discarded = trimmed.ends_with(".ok();")
                || trimmed.contains(".ok();")
                || trimmed.contains("let _ =") && trimmed.contains(".ok()");

            if !is_propagated && is_discarded {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::BareOkDiscarded {
                        expression: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Do not swallow Result with .ok(); route to AppError with '?' or record_ignored with '// Justification:'.".to_string(),
                });
            }
        }

        // 4. Check bare `.unwrap_or_default()` on fallible operations
        if trimmed.contains(".unwrap_or_default()") {
            let is_fallible_source = trimmed.contains("fs::")
                || trimmed.contains("read_to_string")
                || trimmed.contains("from_str")
                || trimmed.contains("serde_json")
                || trimmed.contains("rusqlite")
                || trimmed.contains("reqwest")
                || trimmed.contains("spawn")
                || trimmed.contains("execute");

            if is_fallible_source {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::BareUnwrapOrDefaultOnFallible {
                        expression: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Handle Result explicitly with error routing instead of silently falling back via unwrap_or_default().".to_string(),
                });
            }
        }
    }

    let is_clean = violations.is_empty();
    SilentErrorLintReport {
        is_clean,
        violations,
        best_effort_recorded_count: best_effort_count,
        total_lines_scanned: lines.len(),
    }
}

/// Lints TypeScript source code against the No Silent Errors invariant.
pub fn lint_typescript_code(content: &str) -> SilentErrorLintReport {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();
    let mut best_effort_count = 0;
    let mut in_block_comment = false;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }

        // 1. Swallowed promise .catch(() => {})
        if trimmed.contains(".catch(") {
            let is_swallowed = trimmed.contains("() => {}")
                || trimmed.contains("() => { }")
                || trimmed.contains("(_e) => {}")
                || trimmed.contains("(_e) => { }")
                || trimmed.contains("(e) => {}")
                || trimmed.contains("(e) => { }")
                || trimmed.contains("e => {}")
                || trimmed.contains("err => {}")
                || trimmed.contains("(err) => {}")
                || trimmed.contains("() => null")
                || trimmed.contains("() => undefined");

            if is_swallowed {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::SwallowedPromiseCatch {
                        snippet: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Route promise failure into useErrorStore.getState().trackWarning(e, { source, triggerAction }) or captureError.".to_string(),
                });
            }
        }

        // 2. Catch blocks
        if trimmed.contains("catch {") || trimmed.contains("catch (") || trimmed.contains("catch(")
        {
            // Check if it's an empty catch block on the same line
            if trimmed.ends_with("{}") || trimmed.ends_with("{ }") {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::BareCatchBlock {
                        snippet: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Catch block must record failure via useErrorStore.getState().trackWarning or captureError.".to_string(),
                });
                continue;
            }

            // Inspect catch block body across consecutive lines (up to 8 lines)
            let mut block_lines = Vec::new();
            let mut lookahead = idx;
            let mut brace_depth = 0;
            let mut started = false;

            while lookahead < lines.len() && lookahead < idx + 8 {
                let curr = lines[lookahead];
                for ch in curr.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                        started = true;
                    } else if ch == '}' {
                        brace_depth -= 1;
                    }
                }
                block_lines.push(curr);
                if started && brace_depth == 0 {
                    break;
                }
                lookahead += 1;
            }

            let block_str = block_lines.join("\n");
            let has_error_store = block_str.contains("trackWarning")
                || block_str.contains("captureError")
                || block_str.contains("captureException")
                || block_str.contains("useErrorStore");

            let has_console_only = block_str.contains("console.warn")
                || block_str.contains("console.error")
                || block_str.contains("console.log");

            if has_error_store {
                best_effort_count += 1;
            } else if has_console_only {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::ConsoleOnlyCatchWithoutErrorStore {
                        snippet: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Do not rely exclusively on console logging in catch block; route into useErrorStore.getState().trackWarning.".to_string(),
                });
            } else if started && brace_depth == 0 && block_lines.len() <= 2 {
                violations.push(SilentErrorViolation {
                    line_number: line_num,
                    kind: SilentErrorViolationKind::BareCatchBlock {
                        snippet: trimmed.to_string(),
                    },
                    snippet: trimmed.to_string(),
                    suggestion: "Catch block is empty or missing error store dispatch.".to_string(),
                });
            }
        }
    }

    let is_clean = violations.is_empty();
    SilentErrorLintReport {
        is_clean,
        violations,
        best_effort_recorded_count: best_effort_count,
        total_lines_scanned: lines.len(),
    }
}

/// Dispatches source linting based on whether the source is Rust or TypeScript.
pub fn lint_source(content: &str, is_rust: bool) -> SilentErrorLintReport {
    if is_rust {
        lint_rust_code(content)
    } else {
        lint_typescript_code(content)
    }
}
