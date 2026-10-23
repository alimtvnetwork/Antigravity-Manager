//! GitHub Release Notes Parity Guard (`Imp-689`).
//!
//! Enforces AGENTS.md Release Discipline & Attribution invariant:
//! "Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit`
//! (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other
//! GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub
//! release page contributors list only contains `aukgit` and none else.
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
];

/// Violations detected when auditing GitHub release notes content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseNotesViolation {
    /// An absolute filesystem path was detected in release notes text.
    AbsolutePathDetected {
        line_number: usize,
        detected_path: String,
    },
    /// A third-party GitHub handle other than `@aukgit` was found.
    ForeignContributorHandle {
        line_number: usize,
        detected_handle: String,
    },
    /// Top heading fails to match the expected Git release tag character-for-character.
    TagHeadingMismatch {
        expected_tag: String,
        found_heading: Option<String>,
    },
}

impl fmt::Display for ReleaseNotesViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsolutePathDetected { line_number, detected_path } => {
                write!(f, "Absolute path at line {line_number}: '{detected_path}'")
            }
            Self::ForeignContributorHandle { line_number, detected_handle } => {
                write!(
                    f,
                    "Foreign contributor handle '{detected_handle}' at line {line_number}; only '{SOLE_PERMITTED_AUTHOR_HANDLE}' is permitted"
                )
            }
            Self::TagHeadingMismatch { expected_tag, found_heading } => {
                write!(
                    f,
                    "Release heading mismatch: expected '{expected_tag}', found '{:?}'",
                    found_heading
                )
            }
        }
    }
}

/// Audit report produced after inspecting GitHub release notes markdown text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseNotesAuditReport {
    /// True if release notes satisfy all path hygiene and single-contributor rules.
    pub is_compliant: bool,
    /// Total lines evaluated.
    pub total_lines_audited: usize,
    /// Detected tag heading.
    pub detected_heading: Option<String>,
    /// Granular list of detected violations.
    pub violations: Vec<ReleaseNotesViolation>,
}

/// Audits release notes markdown content for strictly relative paths and single attribution.
pub fn audit_release_notes_parity(
    release_notes_content: &str,
    expected_tag: &str,
) -> ReleaseNotesAuditReport {
    let mut violations = Vec::new();
    let mut detected_heading = None;
    let lines: Vec<&str> = release_notes_content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // 1. Tag heading match (first markdown heading)
        if detected_heading.is_none() && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            let heading_text = trimmed.trim_start_matches('#').trim();
            detected_heading = Some(heading_text.to_string());
            if heading_text != expected_tag {
                violations.push(ReleaseNotesViolation::TagHeadingMismatch {
                    expected_tag: expected_tag.to_string(),
                    found_heading: Some(heading_text.to_string()),
                });
            }
        }

        // 2. Absolute path check
        let lower = trimmed.to_ascii_lowercase();
        for prefix in FORBIDDEN_ABSOLUTE_PREFIXES {
            if let Some(pos) = lower.find(prefix) {
                let snippet = &trimmed[pos..(pos + 30).min(trimmed.len())];
                violations.push(ReleaseNotesViolation::AbsolutePathDetected {
                    line_number: line_num,
                    detected_path: snippet.to_string(),
                });
                break;
            }
        }

        // 3. Contributor attribution check (detect foreign @handles)
        for word in trimmed.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '-');
            if clean_word.starts_with('@') && clean_word.len() > 1 {
                if clean_word != SOLE_PERMITTED_AUTHOR_HANDLE {
                    violations.push(ReleaseNotesViolation::ForeignContributorHandle {
                        line_number: line_num,
                        detected_handle: clean_word.to_string(),
                    });
                }
            }
        }
    }

    if detected_heading.is_none() {
        violations.push(ReleaseNotesViolation::TagHeadingMismatch {
            expected_tag: expected_tag.to_string(),
            found_heading: None,
        });
    }

    ReleaseNotesAuditReport {
        is_compliant: violations.is_empty(),
        total_lines_audited: lines.len(),
        detected_heading,
        violations,
    }
}
