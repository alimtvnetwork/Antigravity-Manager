//! Sole Contributor Attribution & No Silent Errors Audit Guard (`Imp-714`, Step 300).
//!
//! Enforces release gate compliance per AGENTS.md:
//! 1. Sole Contributor Attribution:
//!    "Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit`
//!    (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other
//!    GitHub user handles (`@...`) in release changelogs or release notes, ensuring the
//!    GitHub release page contributors list only contains `aukgit` and none else."
//! 2. No Silent Errors:
//!    "Every fallible operation must route its failure into the error module. Swallowing an error
//!    — discarding it where no one can ever see it — is forbidden."
//! 3. Strictly Relative Git Paths:
//!    "Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`)
//!    in specifications, plans, commit messages, `changelog.md`, `changelog_en.md`, `README.md`,
//!    `README_EN.md`, or GitHub release notes."

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard attribution trailer mandated for release notes and changelog headers.
pub const STANDARD_ATTRIBUTION_PATTERN: &str = "(Thanks to @aukgit)";

/// Canonical GitHub handle mention pattern.
static HANDLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@([a-zA-Z0-9_\-]+)").expect("valid handle regex"));

/// Absolute path detection regular expressions.
static WINDOWS_DRIVE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b[A-Za-z]:[\\/][^\s\)\"'`<>]+"#).expect("valid windows drive regex")
});

static POSIX_HOME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:home|Users)/[^\s\)\"'`<>]+)"#).expect("valid posix home regex")
});

static POSIX_SYSTEM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:var|etc|usr|opt|tmp)/[^\s\)\"'`<>]+)"#)
        .expect("valid posix system regex")
});

static FILE_URI_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"file:///[^\s\)\"'`<>]+"#).expect("valid file uri regex"));

/// Findings discovered during changelog and release notes attribution audits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributionFinding {
    /// Valid standard attribution trailer to `@aukgit` found.
    ValidAukgitAttribution,
    /// Disallowed third-party GitHub handle found in release documentation.
    UnauthorizedHandle(String),
    /// Standard `(Thanks to @aukgit)` trailer missing.
    MissingAttributionTrailer,
}

impl AttributionFinding {
    /// Returns true if this finding represents a valid standard attribution.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::ValidAukgitAttribution)
    }

    /// Returns true if this finding represents an unauthorized handle violation.
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::UnauthorizedHandle(_))
    }

    /// Extracts the unauthorized handle name if present.
    pub fn unauthorized_handle(&self) -> Option<&str> {
        match self {
            Self::UnauthorizedHandle(h) => Some(h.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for AttributionFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidAukgitAttribution => write!(f, "Valid attribution: (Thanks to @aukgit)"),
            Self::UnauthorizedHandle(h) => {
                write!(f, "Unauthorized third-party handle detected: '{h}'")
            }
            Self::MissingAttributionTrailer => {
                write!(f, "Missing standard attribution trailer: '(Thanks to @aukgit)'")
            }
        }
    }
}

/// Findings discovered during static analysis of source code error handling patterns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SilentErrorFinding {
    /// Result discarded without error routing (`let _ = <fallible>`, `.ok();`, unrouted error).
    BareResultDiscarded { file: String, line: usize },
    /// Bare catch block without error-store tracking or justification in TypeScript/JavaScript.
    BareCatchWithoutTracking { file: String, line: usize },
    /// Documented best-effort error ignore with valid `// Justification:` comment.
    DocumentedIgnoredResult,
}

impl SilentErrorFinding {
    /// Returns true if this finding represents an unhandled silent error violation.
    pub fn is_violation(&self) -> bool {
        matches!(
            self,
            Self::BareResultDiscarded { .. } | Self::BareCatchWithoutTracking { .. }
        )
    }

    /// Returns the file path associated with the violation, if applicable.
    pub fn file(&self) -> Option<&str> {
        match self {
            Self::BareResultDiscarded { file, .. }
            | Self::BareCatchWithoutTracking { file, .. } => Some(file.as_str()),
            Self::DocumentedIgnoredResult => None,
        }
    }

    /// Returns the line number associated with the violation, if applicable.
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::BareResultDiscarded { line, .. }
            | Self::BareCatchWithoutTracking { line, .. } => Some(*line),
            Self::DocumentedIgnoredResult => None,
        }
    }
}

impl fmt::Display for SilentErrorFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BareResultDiscarded { file, line } => {
                write!(f, "Bare result discarded without error routing at {file}:{line}")
            }
            Self::BareCatchWithoutTracking { file, line } => {
                write!(f, "Bare catch block without error tracking at {file}:{line}")
            }
            Self::DocumentedIgnoredResult => {
                write!(f, "Documented best-effort ignore with valid justification")
            }
        }
    }
}

/// Comprehensive release governance and quality compliance audit report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseGovernanceReport {
    pub is_compliant: bool,
    pub attribution_findings: Vec<AttributionFinding>,
    pub silent_error_findings: Vec<SilentErrorFinding>,
    pub absolute_path_count: usize,
    pub summary: String,
}

impl ReleaseGovernanceReport {
    /// Returns true if any attribution violations exist.
    pub fn has_attribution_violations(&self) -> bool {
        self.attribution_findings.iter().any(|f| {
            matches!(
                f,
                AttributionFinding::UnauthorizedHandle(_)
                    | AttributionFinding::MissingAttributionTrailer
            )
        })
    }

    /// Returns true if any unhandled silent error violations exist.
    pub fn has_silent_error_violations(&self) -> bool {
        self.silent_error_findings.iter().any(|f| f.is_violation())
    }

    /// Returns true if any absolute path violations exist.
    pub fn has_absolute_path_violations(&self) -> bool {
        self.absolute_path_count > 0
    }
}

impl fmt::Display for ReleaseGovernanceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary)
    }
}

/// Checks whether byte index at `@` is preceded by a non-alphanumeric boundary.
fn is_handle_prefix_boundary(line: &str, byte_idx: usize) -> bool {
    if byte_idx == 0 {
        return true;
    }
    line[..byte_idx]
        .chars()
        .next_back()
        .map(|c| !c.is_alphanumeric() && c != '_')
        .unwrap_or(true)
}

/// Audits release text (e.g. changelog or release notes) for sole contributor attribution.
pub fn audit_release_attribution(text: &str) -> Vec<AttributionFinding> {
    let mut findings = Vec::new();

    // 1. Verify standard attribution trailer
    if text.contains(STANDARD_ATTRIBUTION_PATTERN) {
        findings.push(AttributionFinding::ValidAukgitAttribution);
    } else {
        findings.push(AttributionFinding::MissingAttributionTrailer);
    }

    // 2. Scan all lines for unauthorized third-party handles
    for line in text.lines() {
        for mat in HANDLE_REGEX.find_iter(line) {
            if !is_handle_prefix_boundary(line, mat.start()) {
                continue;
            }
            let raw_handle = &mat.as_str()[1..];
            if !raw_handle.eq_ignore_ascii_case("aukgit") {
                findings.push(AttributionFinding::UnauthorizedHandle(format!(
                    "@{raw_handle}"
                )));
            }
        }
    }

    findings
}

/// Extracts justification comment content if present on a line.
fn has_justification_comment(lines: &[&str], line_idx: usize) -> bool {
    let check_line = |l: &str| -> bool {
        let lower = l.to_ascii_lowercase();
        lower.contains("// justification:")
    };

    // Check current line
    if check_line(lines[line_idx]) {
        return true;
    }

    // Check up to 3 non-empty lines backward
    let mut lookback = line_idx;
    let mut non_empty = 0;
    while lookback > 0 && non_empty < 3 {
        lookback -= 1;
        let l = lines[lookback].trim();
        if l.is_empty() || l == "{" {
            continue;
        }
        non_empty += 1;
        if check_line(l) {
            return true;
        }
        if !l.starts_with("//") {
            break;
        }
    }

    // Check next line forward
    if line_idx + 1 < lines.len() && check_line(lines[line_idx + 1]) {
        return true;
    }

    false
}

/// Audits source code for No Silent Errors violations.
pub fn audit_silent_error_patterns(source_code: &str, file_name: &str) -> Vec<SilentErrorFinding> {
    let lines: Vec<&str> = source_code.lines().collect();
    let mut findings = Vec::new();
    let is_ts = file_name.ends_with(".ts")
        || file_name.ends_with(".tsx")
        || file_name.ends_with(".js")
        || file_name.ends_with(".jsx");

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Skip comments and blank lines
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // 1. Rust: record_ignored checking
        if trimmed.contains("record_ignored(") {
            if has_justification_comment(&lines, idx) {
                findings.push(SilentErrorFinding::DocumentedIgnoredResult);
            } else {
                findings.push(SilentErrorFinding::BareResultDiscarded {
                    file: file_name.to_string(),
                    line: line_num,
                });
            }
            continue;
        }

        // 2. Rust: bare `let _ = ` discarding fallible results
        if (trimmed.starts_with("let _ =") || trimmed.contains("let _ ="))
            && !trimmed.contains("record_ignored")
        {
            if has_justification_comment(&lines, idx) {
                findings.push(SilentErrorFinding::DocumentedIgnoredResult);
            } else {
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
                    findings.push(SilentErrorFinding::BareResultDiscarded {
                        file: file_name.to_string(),
                        line: line_num,
                    });
                }
            }
            continue;
        }

        // 3. Rust: bare `.ok();` discarding error without propagation
        if trimmed.ends_with(".ok();") || (trimmed.contains(".ok();") && !trimmed.contains(".ok()?")) {
            if has_justification_comment(&lines, idx) {
                findings.push(SilentErrorFinding::DocumentedIgnoredResult);
            } else {
                findings.push(SilentErrorFinding::BareResultDiscarded {
                    file: file_name.to_string(),
                    line: line_num,
                });
            }
            continue;
        }

        // 4. TypeScript: swallowed promise catch `.catch(() => {})`
        if is_ts && trimmed.contains(".catch(") {
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
                findings.push(SilentErrorFinding::BareCatchWithoutTracking {
                    file: file_name.to_string(),
                    line: line_num,
                });
                continue;
            }
        }

        // 5. TypeScript: bare catch block
        if is_ts
            && (trimmed.contains("catch {") || trimmed.contains("catch (") || trimmed.contains("catch("))
        {
            if trimmed.ends_with("{}") || trimmed.ends_with("{ }") {
                findings.push(SilentErrorFinding::BareCatchWithoutTracking {
                    file: file_name.to_string(),
                    line: line_num,
                });
                continue;
            }

            // Inspect multi-line catch block body
            let mut block_body = String::new();
            let mut depth = 0;
            let mut block_started = false;

            for block_line in lines.iter().skip(idx).take(10) {
                let bl = block_line.trim();
                for c in bl.chars() {
                    if c == '{' {
                        depth += 1;
                        block_started = true;
                    } else if c == '}' {
                        depth -= 1;
                    }
                }
                block_body.push(' ');
                block_body.push_str(bl);
                if block_started && depth == 0 {
                    break;
                }
            }

            let has_error_routing = block_body.contains("trackWarning")
                || block_body.contains("captureError")
                || block_body.contains("captureException")
                || block_body.contains("errorStore")
                || block_body.contains("throw ");

            if !has_error_routing {
                findings.push(SilentErrorFinding::BareCatchWithoutTracking {
                    file: file_name.to_string(),
                    line: line_num,
                });
            }
        }
    }

    findings
}

/// Counts absolute path occurrences in documentation or release text.
pub fn count_absolute_paths(text: &str) -> usize {
    let mut count = 0;
    count += WINDOWS_DRIVE_RE.find_iter(text).count();
    count += POSIX_HOME_RE.find_iter(text).count();
    count += POSIX_SYSTEM_RE.find_iter(text).count();
    count += FILE_URI_RE.find_iter(text).count();
    count
}

/// Generates a comprehensive release governance and quality compliance report.
pub fn generate_governance_report(
    text: &str,
    code_sources: &[(&str, &str)],
) -> ReleaseGovernanceReport {
    let attribution_findings = audit_release_attribution(text);

    let mut silent_error_findings = Vec::new();
    for (item_a, item_b) in code_sources {
        // Transparently detect which item is file name and which is source code
        let (source_code, file_name) = if item_a.contains('\n') {
            (*item_a, *item_b)
        } else if item_b.contains('\n') {
            (*item_b, *item_a)
        } else if item_a.ends_with(".rs")
            || item_a.ends_with(".ts")
            || item_a.ends_with(".tsx")
            || item_a.ends_with(".js")
        {
            (*item_b, *item_a)
        } else {
            (*item_a, *item_b)
        };

        let findings = audit_silent_error_patterns(source_code, file_name);
        silent_error_findings.extend(findings);
    }

    let absolute_path_count = count_absolute_paths(text);

    let has_unauthorized = attribution_findings
        .iter()
        .any(|f| matches!(f, AttributionFinding::UnauthorizedHandle(_)));
    let has_missing_trailer = attribution_findings
        .iter()
        .any(|f| matches!(f, AttributionFinding::MissingAttributionTrailer));
    let has_silent_violations = silent_error_findings.iter().any(|f| f.is_violation());
    let has_absolute_paths = absolute_path_count > 0;

    let is_compliant =
        !has_unauthorized && !has_missing_trailer && !has_silent_violations && !has_absolute_paths;

    let mut summary = String::new();
    summary.push_str("# Release Governance & Quality Audit Report\n\n");
    summary.push_str(&format!(
        "- **Overall Status**: {}\n",
        if is_compliant {
            "✅ COMPLIANT"
        } else {
            "❌ NON-COMPLIANT"
        }
    ));
    summary.push_str(&format!(
        "- **Attribution Findings**: {} total\n",
        attribution_findings.len()
    ));
    summary.push_str(&format!(
        "- **Silent Error Findings**: {} total ({} violations)\n",
        silent_error_findings.len(),
        silent_error_findings
            .iter()
            .filter(|f| f.is_violation())
            .count()
    ));
    summary.push_str(&format!(
        "- **Absolute Path Violations**: {}\n",
        absolute_path_count
    ));

    if !is_compliant {
        summary.push_str("\n### Violations Detected:\n");
        for af in &attribution_findings {
            if af.is_unauthorized() || matches!(af, AttributionFinding::MissingAttributionTrailer) {
                summary.push_str(&format!("- Attribution: {}\n", af));
            }
        }
        for se in &silent_error_findings {
            if se.is_violation() {
                summary.push_str(&format!("- Silent Error: {}\n", se));
            }
        }
        if has_absolute_paths {
            summary.push_str(&format!(
                "- Relative Path Invariant: {} absolute path occurrences detected in text\n",
                absolute_path_count
            ));
        }
    }

    ReleaseGovernanceReport {
        is_compliant,
        attribution_findings,
        silent_error_findings,
        absolute_path_count,
        summary,
    }
}
