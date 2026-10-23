//! GitHub Release Notes Parity Guard (`Imp-689`).
//!
//! Enforces AGENTS.md Release Discipline & Attribution invariant:
//! "Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit`
//! (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other
//! GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub
//! release page contributors list only contains `aukgit` and none else.
//! Synchronize README Changelog: For stable releases, you MUST update the release summary in both
//! `README.md` and `README_EN.md`.
//! Strictly Relative Git Paths Invariant: Never output or record absolute filesystem paths
//! (e.g. `D:\...`, `C:\...`, `/home/...`) in specifications, plans, commit messages, `changelog.md`,
//! `changelog_en.md`, `README.md`, `README_EN.md`, or GitHub release notes."

use serde::{Deserialize, Serialize};
use std::fmt;

/// The only permitted GitHub attribution handle across release changelogs and notes.
pub const SOLE_PERMITTED_AUTHOR_HANDLE: &str = "@aukgit";

/// Known absolute path drive and directory prefixes forbidden in release notes.
pub const FORBIDDEN_ABSOLUTE_PREFIXES: &[&str] = &[
    "c:\\", "c:/", "d:\\", "d:/", "e:\\", "e:/", "/home/", "/users/", "/etc/", "/var/",
    "/tmp/", "/usr/", "/root/", "/opt/", "file://",
];

/// Granular violations detected during GitHub release notes validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseNotesViolation {
    /// A third-party GitHub handle other than `@aukgit` was found.
    UnauthorizedContributorHandle(String),
    /// An absolute filesystem path or file URI was found in release notes text.
    AbsolutePathFound {
        /// Extracted absolute path snippet.
        path: String,
        /// 1-indexed line number where the path was detected.
        line: usize,
    },
    /// Mandatory attribution to `@aukgit` was missing.
    MissingAukgitAttribution,
    /// Release notes or changelog heading failed to match the targeted Git tag.
    MissingChangelogHeadingMatch {
        /// Targeted release tag.
        tag: String,
    },
}

impl fmt::Display for ReleaseNotesViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnauthorizedContributorHandle(handle) => {
                write!(
                    f,
                    "Unauthorized contributor handle '{handle}'. Attribution must attribute strictly '{SOLE_PERMITTED_AUTHOR_HANDLE}'."
                )
            }
            Self::AbsolutePathFound { path, line } => {
                write!(
                    f,
                    "Absolute filesystem path '{path}' detected at line {line}. Strictly relative paths invariant violated."
                )
            }
            Self::MissingAukgitAttribution => {
                write!(
                    f,
                    "Missing mandatory attribution to '{SOLE_PERMITTED_AUTHOR_HANDLE}' ('(Thanks to {SOLE_PERMITTED_AUTHOR_HANDLE})')."
                )
            }
            Self::MissingChangelogHeadingMatch { tag } => {
                write!(
                    f,
                    "Release heading mismatch: release notes or changelog heading does not match tag '{tag}'."
                )
            }
        }
    }
}

/// Audit report produced after validating GitHub release notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseNotesParityReport {
    /// Targeted Git release tag.
    pub tag: String,
    /// True if release notes satisfy all path hygiene, single attribution, and heading rules.
    pub is_compliant: bool,
    /// Granular list of detected policy violations.
    pub violations: Vec<ReleaseNotesViolation>,
    /// True if the mandatory `@aukgit` attribution is present.
    pub authorized_attribution: bool,
}

impl fmt::Display for ReleaseNotesParityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReleaseNotesParityReport: tag={}, compliant={}, authorized_attribution={}, violations={}",
            self.tag,
            self.is_compliant,
            self.authorized_attribution,
            self.violations.len()
        )
    }
}

/// Helper function to trim and extract a path token from a snippet.
fn extract_path_token(snippet: &str) -> String {
    let end = snippet
        .find(|c: char| {
            c.is_whitespace()
                || c == '"'
                || c == '\''
                || c == ')'
                || c == ']'
                || c == '`'
                || c == '>'
                || c == '<'
        })
        .unwrap_or(snippet.len());

    snippet[..end]
        .trim_end_matches(|c: char| c == '.' || c == ',' || c == ';')
        .to_string()
}

/// Extracts all absolute filesystem paths and file URIs from a line of text.
fn extract_absolute_paths_from_line(line: &str) -> Vec<String> {
    let mut results = Vec::new();
    let lower = line.to_ascii_lowercase();
    let bytes = line.as_bytes();

    // 1. file:// scheme detection
    let mut search_start = 0;
    while let Some(pos) = lower[search_start..].find("file://") {
        let actual_pos = search_start + pos;
        let token = extract_path_token(&line[actual_pos..]);
        if !token.is_empty() && !results.contains(&token) {
            results.push(token.clone());
        }
        search_start = actual_pos + token.len().max(1);
    }

    // 2. Windows drive letters (e.g. C:\, C:/, D:\, D:/)
    for i in 0..bytes.len() {
        if bytes[i].is_ascii_alphabetic()
            && i + 2 < bytes.len()
            && bytes[i + 1] == b':'
            && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/')
        {
            let is_start = i == 0
                || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'/');
            if is_start {
                let token = extract_path_token(&line[i..]);
                if !token.is_empty() && !results.contains(&token) {
                    results.push(token);
                }
            }
        }
    }

    // 3. Unix root prefixes (/home/, /users/, /etc/, /var/, etc.)
    let unix_prefixes = [
        "/home/", "/users/", "/etc/", "/var/", "/tmp/", "/usr/", "/root/", "/opt/",
    ];
    for prefix in unix_prefixes {
        let mut u_start = 0;
        while let Some(pos) = lower[u_start..].find(prefix) {
            let actual_pos = u_start + pos;
            let token = extract_path_token(&line[actual_pos..]);
            if !token.is_empty() && !results.contains(&token) {
                results.push(token.clone());
            }
            u_start = actual_pos + token.len().max(1);
        }
    }

    results
}

/// Extracts all GitHub-style `@handle` mentions from a line of text, ignoring emails.
fn extract_handles_from_line(line: &str) -> Vec<String> {
    let mut handles = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        if chars[i] == '@' {
            let is_boundary = i == 0
                || (!chars[i - 1].is_ascii_alphanumeric()
                    && chars[i - 1] != '.'
                    && chars[i - 1] != '_'
                    && chars[i - 1] != '-');

            if is_boundary && i + 1 < len && chars[i + 1].is_ascii_alphanumeric() {
                let start = i;
                i += 1;
                while i < len
                    && (chars[i].is_ascii_alphanumeric()
                        || chars[i] == '-'
                        || chars[i] == '_')
                {
                    i += 1;
                }
                let handle: String = chars[start..i].iter().collect();
                handles.push(handle);
                continue;
            }
        }
        i += 1;
    }

    handles
}

/// Validates GitHub release notes content against tag matching, author attribution, and path hygiene.
pub fn validate_release_notes(
    tag: &str,
    release_notes: &str,
    changelog_heading: Option<&str>,
) -> ReleaseNotesParityReport {
    let mut violations = Vec::new();
    let mut authorized_attribution = false;
    let lines: Vec<&str> = release_notes.lines().collect();

    // 1. Scan line-by-line for absolute paths and author handles
    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;

        // Path scanning
        let found_paths = extract_absolute_paths_from_line(line);
        for path in found_paths {
            violations.push(ReleaseNotesViolation::AbsolutePathFound {
                path,
                line: line_num,
            });
        }

        // Handle scanning
        let found_handles = extract_handles_from_line(line);
        for handle in found_handles {
            if handle.eq_ignore_ascii_case(SOLE_PERMITTED_AUTHOR_HANDLE) {
                authorized_attribution = true;
            } else {
                violations.push(ReleaseNotesViolation::UnauthorizedContributorHandle(handle));
            }
        }
    }

    // 2. Ensure authorized attribution is present
    if !authorized_attribution {
        violations.push(ReleaseNotesViolation::MissingAukgitAttribution);
    }

    // 3. Heading and tag parity validation
    let top_release_heading = lines.iter().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            Some(
                trimmed
                    .trim_start_matches('#')
                    .trim()
                    .trim_matches('[')
                    .trim_matches(']')
                    .trim()
                    .to_string(),
            )
        } else {
            None
        }
    });

    let mut heading_matched = false;

    if let Some(ch_heading) = changelog_heading {
        let clean_ch = ch_heading
            .trim()
            .trim_start_matches('#')
            .trim()
            .trim_matches('[')
            .trim_matches(']')
            .trim();

        if clean_ch == tag.trim() {
            heading_matched = true;
        } else {
            violations.push(ReleaseNotesViolation::MissingChangelogHeadingMatch {
                tag: tag.to_string(),
            });
        }
    } else if let Some(ref rel_h) = top_release_heading {
        if rel_h == tag.trim() {
            heading_matched = true;
        } else {
            violations.push(ReleaseNotesViolation::MissingChangelogHeadingMatch {
                tag: tag.to_string(),
            });
        }
    } else {
        violations.push(ReleaseNotesViolation::MissingChangelogHeadingMatch {
            tag: tag.to_string(),
        });
    }

    if heading_matched {
        if let Some(ref rel_h) = top_release_heading {
            if rel_h != tag.trim() {
                violations.push(ReleaseNotesViolation::MissingChangelogHeadingMatch {
                    tag: tag.to_string(),
                });
            }
        }
    }

    ReleaseNotesParityReport {
        tag: tag.to_string(),
        is_compliant: violations.is_empty(),
        violations,
        authorized_attribution,
    }
}

/// Convenience backward-compatible wrapper auditing release notes against an expected tag.
pub fn audit_release_notes_parity(
    release_notes: &str,
    expected_tag: &str,
) -> ReleaseNotesParityReport {
    validate_release_notes(expected_tag, release_notes, None)
}
