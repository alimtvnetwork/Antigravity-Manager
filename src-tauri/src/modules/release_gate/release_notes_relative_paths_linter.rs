//! Release Gate: Release Notes & Changelog Relative Paths Linter (Imp-657).
//!
//! Enforces AGENTS.md Strictly Relative Git Paths Invariant:
//! "Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`)
//! in specifications, plans, commit messages, `changelog.md`, `changelog_en.md`, `README.md`,
//! `README_EN.md`, or GitHub release notes. All file references MUST be repository-relative Git paths
//! (e.g., `02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`)."

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

static FILE_URI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"file://(?:/[^\s\)\"'`<>]+|localhost/[^\s\)\"'`<>]+)"#)
        .expect("valid file uri regex")
});

static WINDOWS_DRIVE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b[A-Za-z]:[\\/][^\s\)\"'`<>]+"#).expect("valid windows drive regex")
});

static UNC_NETWORK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w\\])(\\\\[a-zA-Z0-9._\-?]+[\\/][^\s\)\"'`<>]+)"#)
        .expect("valid unc network regex")
});

static POSIX_HOME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:home|Users)/[^\s\)\"'`<>]+)"#)
        .expect("valid posix home regex")
});

static POSIX_SYSTEM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:^|[^\w/])(/(?:var|etc|usr|opt|tmp|private)/[^\s\)\"'`<>]+)"#)
        .expect("valid posix system regex")
});

/// Recognized repository root directories for deriving relative path recommendations.
pub const CANONICAL_REPO_PREFIXES: &[&str] = &[
    "02-spec/",
    ".ai-memory/",
    "src-tauri/",
    "src/",
    "cmd/",
    "docs/",
    "scripts/",
    "crates/",
    "packages/",
    ".agents/",
    ".github/",
    "tests/",
    "public/",
];

/// Recognized repository top-level files for deriving relative path recommendations.
pub const CANONICAL_REPO_FILES: &[&str] = &[
    "CHANGELOG.md",
    "changelog.md",
    "changelog_en.md",
    "README.md",
    "README_EN.md",
    "AGENTS.md",
    "package.json",
    "Cargo.toml",
    "Cargo.lock",
    "tsconfig.json",
    "vite.config.ts",
];

/// Classification category for detected absolute paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbsolutePathCategory {
    WindowsDrive,
    UncNetworkPath,
    PosixHome,
    PosixSystem,
    FileUri,
}

impl fmt::Display for AbsolutePathCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowsDrive => write!(f, "Windows drive absolute path"),
            Self::UncNetworkPath => write!(f, "UNC network absolute path"),
            Self::PosixHome => write!(f, "POSIX user home absolute path"),
            Self::PosixSystem => write!(f, "POSIX system absolute path"),
            Self::FileUri => write!(f, "Absolute file URI"),
        }
    }
}

/// Detailed violation record for an absolute path found in markdown or release notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelativePathViolation {
    pub line_number: usize,
    pub raw_path: String,
    pub category: AbsolutePathCategory,
    pub snippet: String,
    pub recommended_relative_path: String,
}

impl fmt::Display for RelativePathViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Line {}: {} '{}' -> recommended: '{}'",
            self.line_number, self.category, self.raw_path, self.recommended_relative_path
        )
    }
}

impl std::error::Error for RelativePathViolation {}

/// Structured audit report for relative path hygiene in release notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelativePathAuditReport {
    pub is_clean: bool,
    pub total_lines: usize,
    pub violations: Vec<RelativePathViolation>,
    pub verified_relative_paths: usize,
}

impl RelativePathAuditReport {
    pub fn is_clean(&self) -> bool {
        self.is_clean
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    pub fn format_summary(&self) -> String {
        if self.is_clean {
            format!(
                "Path audit passed: {} lines scanned, {} valid relative paths verified, 0 violations.",
                self.total_lines, self.verified_relative_paths
            )
        } else {
            format!(
                "Path audit failed: {} violations detected across {} lines.",
                self.violations.len(),
                self.total_lines
            )
        }
    }
}

fn trim_trailing_punctuation(s: &str) -> &str {
    s.trim_end_matches(&['.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'', '`'][..])
}

fn create_snippet(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.chars().count() > 120 {
        let mut s: String = trimmed.chars().take(117).collect();
        s.push_str("...");
        s
    } else {
        trimmed.to_string()
    }
}

/// Analyzes an absolute path and generates a recommended repository-relative Git path.
pub fn suggest_relative_path(raw_path: &str) -> String {
    let trimmed = raw_path.trim();

    // Strip URI protocol if present
    let stripped = if let Some(rest) = trimmed.strip_prefix("file://localhost/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("file:///") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("file://") {
        rest
    } else {
        trimmed
    };

    // Normalize separators to forward slash
    let normalized = stripped.replace('\\', "/");

    // Check for canonical repo prefixes
    for &prefix in CANONICAL_REPO_PREFIXES {
        if let Some(pos) = normalized.find(prefix) {
            return normalized[pos..].to_string();
        }
        let prefix_no_slash = prefix.trim_end_matches('/');
        if let Some(pos) = normalized.find(&format!("/{prefix_no_slash}/")) {
            return normalized[pos + 1..].to_string();
        }
    }

    // Check for canonical top-level repo files
    for &file in CANONICAL_REPO_FILES {
        if normalized.ends_with(&format!("/{file}")) || normalized == file {
            return file.to_string();
        }
    }

    // Fallback: extract the trailing file name or component
    let filename = normalized
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("file");

    format!("./{filename}")
}

fn scan_line(line_number: usize, line: &str, violations: &mut Vec<RelativePathViolation>) {
    let snippet = create_snippet(line);
    let mut working = line.to_string();

    // 1. Absolute file URIs
    for mat in FILE_URI_RE.find_iter(&working) {
        let raw = trim_trailing_punctuation(mat.as_str());
        violations.push(RelativePathViolation {
            line_number,
            raw_path: raw.to_string(),
            category: AbsolutePathCategory::FileUri,
            snippet: snippet.clone(),
            recommended_relative_path: suggest_relative_path(raw),
        });
    }
    working = FILE_URI_RE.replace_all(&working, " ").to_string();

    // 2. UNC network paths
    for cap in UNC_NETWORK_RE.captures_iter(&working) {
        if let Some(mat) = cap.get(1) {
            let raw = trim_trailing_punctuation(mat.as_str());
            violations.push(RelativePathViolation {
                line_number,
                raw_path: raw.to_string(),
                category: AbsolutePathCategory::UncNetworkPath,
                snippet: snippet.clone(),
                recommended_relative_path: suggest_relative_path(raw),
            });
        }
    }
    working = UNC_NETWORK_RE.replace_all(&working, " ").to_string();

    // 3. Windows drive letter paths
    for mat in WINDOWS_DRIVE_RE.find_iter(&working) {
        let raw = trim_trailing_punctuation(mat.as_str());
        violations.push(RelativePathViolation {
            line_number,
            raw_path: raw.to_string(),
            category: AbsolutePathCategory::WindowsDrive,
            snippet: snippet.clone(),
            recommended_relative_path: suggest_relative_path(raw),
        });
    }
    working = WINDOWS_DRIVE_RE.replace_all(&working, " ").to_string();

    // 4. POSIX user home paths
    for cap in POSIX_HOME_RE.captures_iter(&working) {
        if let Some(mat) = cap.get(1) {
            let raw = trim_trailing_punctuation(mat.as_str());
            violations.push(RelativePathViolation {
                line_number,
                raw_path: raw.to_string(),
                category: AbsolutePathCategory::PosixHome,
                snippet: snippet.clone(),
                recommended_relative_path: suggest_relative_path(raw),
            });
        }
    }
    working = POSIX_HOME_RE.replace_all(&working, " ").to_string();

    // 5. POSIX system absolute paths
    for cap in POSIX_SYSTEM_RE.captures_iter(&working) {
        if let Some(mat) = cap.get(1) {
            let raw = trim_trailing_punctuation(mat.as_str());
            violations.push(RelativePathViolation {
                line_number,
                raw_path: raw.to_string(),
                category: AbsolutePathCategory::PosixSystem,
                snippet: snippet.clone(),
                recommended_relative_path: suggest_relative_path(raw),
            });
        }
    }
}

fn count_verified_relative_paths(line: &str) -> usize {
    line.split_whitespace()
        .filter(|word| {
            let cleaned = word.trim_matches(|c: char| {
                matches!(
                    c,
                    '`' | '(' | ')' | '[' | ']' | '<' | '>' | '"' | '\'' | ',' | ';' | ':' | '.'
                )
            });
            CANONICAL_REPO_PREFIXES.iter().any(|&prefix| {
                cleaned.starts_with(prefix) || cleaned.starts_with(&prefix.replace('/', "\\"))
            }) || CANONICAL_REPO_FILES.iter().any(|&file| cleaned == file)
        })
        .count()
}

/// Lints markdown text or release notes for strictly repository-relative Git path compliance.
pub fn lint_release_notes(content: &str) -> RelativePathAuditReport {
    let mut violations = Vec::new();
    let mut verified_relative_paths = 0;
    let mut total_lines = 0;

    for (idx, line) in content.lines().enumerate() {
        total_lines += 1;
        let line_number = idx + 1;
        scan_line(line_number, line, &mut violations);
        verified_relative_paths += count_verified_relative_paths(line);
    }

    let is_clean = violations.is_empty();
    RelativePathAuditReport {
        is_clean,
        total_lines,
        violations,
        verified_relative_paths,
    }
}

/// Validates that release notes text is strictly relative, returning an error list on failure.
pub fn check_release_notes_strictly_relative(
    content: &str,
) -> Result<(), Vec<RelativePathViolation>> {
    let report = lint_release_notes(content);
    if report.is_clean {
        Ok(())
    } else {
        Err(report.violations)
    }
}

/// Lints an on-disk release notes or changelog file for relative path compliance.
pub fn lint_release_notes_file(
    path: impl AsRef<Path>,
) -> Result<RelativePathAuditReport, std::io::Error> {
    let content = fs::read_to_string(path.as_ref())?;
    Ok(lint_release_notes(&content))
}
