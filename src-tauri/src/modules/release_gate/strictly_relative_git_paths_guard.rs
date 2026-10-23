//! Release Gate: Strictly Relative Git Paths Invariant Guard.
//! Enforces:
//! - All documentation, changelogs, specs, commit messages, and release notes must use strictly repository-relative Git paths.
//! - Forbids absolute Windows drive paths (e.g. `C:\`, `D:\`), POSIX home paths (`/home/`, `/Users/`),
//!   POSIX system paths (`/var/`, `/etc/`, `/usr/`), and absolute file URIs (`file:///`).

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

static FILE_URI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"file:///[^\s\)\"'`<>]+"#).expect("valid file uri regex")
});

static WINDOWS_DRIVE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b[A-Za-z]:[\\/][^\s\)\"'`<>]+"#).expect("valid windows drive regex")
});

static POSIX_HOME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:home|Users)/[^\s\)\"'`<>]+)"#).expect("valid posix home regex")
});

static POSIX_SYSTEM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:var|etc|usr)/[^\s\)\"'`<>]+)"#).expect("valid posix system regex")
});

/// Permitted repository-relative path root prefixes.
pub const VALID_RELATIVE_PATH_PREFIXES: &[&str] = &[
    "02-spec/",
    ".ai-memory/",
    "src/",
    "src-tauri/",
    "cmd/",
];

/// Classification of absolute path violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathViolationKind {
    WindowsDriveAbsolute { raw_path: String, line: usize },
    PosixHomeAbsolute { raw_path: String, line: usize },
    PosixSystemAbsolute { raw_path: String, line: usize },
    FileUriAbsolute { raw_uri: String, line: usize },
}

impl fmt::Display for PathViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowsDriveAbsolute { raw_path, line } => {
                write!(
                    f,
                    "Strict relative path violation at line {line}: Windows drive absolute path '{raw_path}'"
                )
            }
            Self::PosixHomeAbsolute { raw_path, line } => {
                write!(
                    f,
                    "Strict relative path violation at line {line}: POSIX home absolute path '{raw_path}'"
                )
            }
            Self::PosixSystemAbsolute { raw_path, line } => {
                write!(
                    f,
                    "Strict relative path violation at line {line}: POSIX system absolute path '{raw_path}'"
                )
            }
            Self::FileUriAbsolute { raw_uri, line } => {
                write!(
                    f,
                    "Strict relative path violation at line {line}: Absolute file URI '{raw_uri}'"
                )
            }
        }
    }
}

impl std::error::Error for PathViolationKind {}

/// Audit report containing violation details and verified relative path statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictRelativePathsReport {
    pub is_clean: bool,
    pub total_lines_scanned: usize,
    pub violations: Vec<PathViolationKind>,
    pub verified_relative_paths_count: usize,
}

fn trim_trailing_punctuation(s: &str) -> &str {
    s.trim_end_matches(&['.', ',', ';', ':', '!', '?'][..])
}

fn detect_file_uris(
    line_num: usize,
    line: &mut String,
    violations: &mut Vec<PathViolationKind>,
) {
    for mat in FILE_URI_RE.find_iter(line) {
        let raw = trim_trailing_punctuation(mat.as_str());
        violations.push(PathViolationKind::FileUriAbsolute {
            raw_uri: raw.to_string(),
            line: line_num,
        });
    }
    *line = FILE_URI_RE.replace_all(line, " ").to_string();
}

fn detect_windows_drive_paths(
    line_num: usize,
    line: &str,
    violations: &mut Vec<PathViolationKind>,
) {
    for mat in WINDOWS_DRIVE_RE.find_iter(line) {
        let raw = trim_trailing_punctuation(mat.as_str());
        violations.push(PathViolationKind::WindowsDriveAbsolute {
            raw_path: raw.to_string(),
            line: line_num,
        });
    }
}

fn detect_posix_home_paths(
    line_num: usize,
    line: &str,
    violations: &mut Vec<PathViolationKind>,
) {
    for cap in POSIX_HOME_RE.captures_iter(line) {
        if let Some(mat) = cap.get(1) {
            let raw = trim_trailing_punctuation(mat.as_str());
            violations.push(PathViolationKind::PosixHomeAbsolute {
                raw_path: raw.to_string(),
                line: line_num,
            });
        }
    }
}

fn detect_posix_system_paths(
    line_num: usize,
    line: &str,
    violations: &mut Vec<PathViolationKind>,
) {
    for cap in POSIX_SYSTEM_RE.captures_iter(line) {
        if let Some(mat) = cap.get(1) {
            let raw = trim_trailing_punctuation(mat.as_str());
            violations.push(PathViolationKind::PosixSystemAbsolute {
                raw_path: raw.to_string(),
                line: line_num,
            });
        }
    }
}

fn is_valid_relative_path(token: &str) -> bool {
    VALID_RELATIVE_PATH_PREFIXES.iter().any(|&prefix| {
        token.starts_with(prefix) || token.starts_with(&prefix.replace('/', "\\"))
    })
}

fn count_line_relative_paths(line: &str) -> usize {
    line.split_whitespace()
        .filter(|word| {
            let trimmed = word.trim_matches(|c: char| {
                matches!(
                    c,
                    '`' | '(' | ')' | '[' | ']' | '<' | '>' | '"' | '\'' | ',' | ';'
                )
            });
            is_valid_relative_path(trimmed)
        })
        .count()
}

fn scan_line_for_violations(
    line_num: usize,
    line: &str,
    violations: &mut Vec<PathViolationKind>,
) {
    let mut sanitized_line = line.to_string();
    detect_file_uris(line_num, &mut sanitized_line, violations);
    detect_windows_drive_paths(line_num, &sanitized_line, violations);
    detect_posix_home_paths(line_num, &sanitized_line, violations);
    detect_posix_system_paths(line_num, &sanitized_line, violations);
}

/// Scans text line-by-line and enforces repository-relative Git path hygiene.
pub fn audit_text_for_absolute_paths(text: &str) -> StrictRelativePathsReport {
    let mut violations = Vec::new();
    let mut verified_relative_paths_count = 0;
    let mut total_lines_scanned = 0;

    for (idx, line) in text.lines().enumerate() {
        total_lines_scanned += 1;
        let line_num = idx + 1;
        scan_line_for_violations(line_num, line, &mut violations);
        verified_relative_paths_count += count_line_relative_paths(line);
    }

    let is_clean = violations.is_empty();
    StrictRelativePathsReport {
        is_clean,
        total_lines_scanned,
        violations,
        verified_relative_paths_count,
    }
}

/// Validates commit messages, returning Err on the first absolute path violation found.
pub fn validate_commit_message_paths(message: &str) -> Result<(), PathViolationKind> {
    let report = audit_text_for_absolute_paths(message);
    if let Some(first_violation) = report.violations.into_iter().next() {
        Err(first_violation)
    } else {
        Ok(())
    }
}
