//! Sole Contributor Attribution Release Enforcer.
//!
//! Enforces contributor attribution discipline per AGENTS.md:
//! "Documentation & Attribution: Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit`
//! (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`)
//! in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else."

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Regex matching GitHub handle mentions `@<handle>`.
static HANDLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@([a-zA-Z0-9_\-]+)").expect("valid handle regex"));

/// Standard attribution pattern required by release gate.
pub const STANDARD_ATTRIBUTION_PATTERN: &str = "(Thanks to @aukgit)";

/// Discovered violations during author attribution audits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributionViolation {
    ThirdPartyHandleDetected {
        handle: String,
        line_number: usize,
    },
    MissingStandardAttributionPattern {
        line_number: usize,
        expected: String,
    },
    NonAukgitContributorInReleaseNotes {
        handle: String,
    },
}

impl std::fmt::Display for AttributionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThirdPartyHandleDetected {
                handle,
                line_number,
            } => {
                write!(
                    f,
                    "Third-party handle '{handle}' detected at line {line_number}"
                )
            }
            Self::MissingStandardAttributionPattern {
                line_number,
                expected,
            } => {
                write!(
                    f,
                    "Missing standard attribution pattern '{expected}' at line {line_number}"
                )
            }
            Self::NonAukgitContributorInReleaseNotes { handle } => {
                write!(
                    f,
                    "Non-aukgit contributor '{handle}' found in release notes"
                )
            }
        }
    }
}

impl std::error::Error for AttributionViolation {}

/// Summary report of changelog or release note attribution audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionReport {
    pub is_compliant: bool,
    pub violations: Vec<AttributionViolation>,
    pub scanned_lines_count: usize,
    pub aukgit_attributions_found: usize,
    pub has_third_party_handles: bool,
}

impl AttributionReport {
    /// Creates a new default attribution report with the given line count.
    pub fn new(scanned_lines_count: usize) -> Self {
        Self {
            is_compliant: true,
            violations: Vec::new(),
            scanned_lines_count,
            aukgit_attributions_found: 0,
            has_third_party_handles: false,
        }
    }
}

/// Checks whether the character preceding '@' is a non-alphanumeric boundary.
///
/// Prevents false-positive matches for email addresses like `dev@example.com`.
fn is_handle_prefix_boundary(text: &str, byte_idx: usize) -> bool {
    if byte_idx == 0 {
        return true;
    }
    text[..byte_idx]
        .chars()
        .next_back()
        .map(|c| !c.is_alphanumeric() && c != '_')
        .unwrap_or(true)
}

/// Extracts all GitHub-style `@handle` mentions from a text slice.
pub fn extract_handles_from_text(text: &str) -> Vec<String> {
    let mut handles = Vec::new();
    for mat in HANDLE_REGEX.find_iter(text) {
        if is_handle_prefix_boundary(text, mat.start()) {
            handles.push(mat.as_str().to_string());
        }
    }
    handles
}

/// Checks whether a line satisfies standard attribution pattern `(Thanks to @aukgit)`.
fn line_has_standard_attribution_format(line: &str) -> bool {
    line.to_ascii_lowercase()
        .contains(&STANDARD_ATTRIBUTION_PATTERN.to_ascii_lowercase())
}

/// Audits a line for handles and validates compliance with sole contributor rules.
fn audit_changelog_line(line: &str, line_number: usize, report: &mut AttributionReport) {
    let mut line_has_aukgit = false;

    for mat in HANDLE_REGEX.find_iter(line) {
        if !is_handle_prefix_boundary(line, mat.start()) {
            continue;
        }

        let raw_handle = &mat.as_str()[1..];
        if raw_handle.eq_ignore_ascii_case("aukgit") {
            line_has_aukgit = true;
            report.aukgit_attributions_found += 1;
        } else {
            report.has_third_party_handles = true;
            report
                .violations
                .push(AttributionViolation::ThirdPartyHandleDetected {
                    handle: mat.as_str().to_string(),
                    line_number,
                });
        }
    }

    if line_has_aukgit && !line_has_standard_attribution_format(line) {
        report
            .violations
            .push(AttributionViolation::MissingStandardAttributionPattern {
                line_number,
                expected: STANDARD_ATTRIBUTION_PATTERN.to_string(),
            });
    } else if !line_has_aukgit
        && line.to_ascii_lowercase().contains("thanks to")
        && !line_has_standard_attribution_format(line)
    {
        report
            .violations
            .push(AttributionViolation::MissingStandardAttributionPattern {
                line_number,
                expected: STANDARD_ATTRIBUTION_PATTERN.to_string(),
            });
    }
}

/// Audits changelog markdown content for exclusive `@aukgit` attribution compliance.
///
/// Scans for regex pattern `@([a-zA-Z0-9_\-]+)`:
/// - If any handle other than `aukgit` is found, flags `ThirdPartyHandleDetected`.
/// - Verifies proper format `(Thanks to @aukgit)`.
/// - Returns `is_compliant = true` only when all handles are `@aukgit` and formatting is correct.
pub fn audit_changelog_attribution(changelog_text: &str) -> AttributionReport {
    let lines: Vec<&str> = changelog_text.lines().collect();
    let mut report = AttributionReport::new(lines.len());

    for (idx, line) in lines.iter().enumerate() {
        audit_changelog_line(line, idx + 1, &mut report);
    }

    report.is_compliant = report.violations.is_empty();
    report
}

/// Audits release notes text against contributor guidelines.
pub fn audit_release_notes(release_notes: &str) -> AttributionReport {
    let lines: Vec<&str> = release_notes.lines().collect();
    let mut report = AttributionReport::new(lines.len());

    for handle in extract_handles_from_text(release_notes) {
        let clean = handle.trim_start_matches('@');
        if clean.eq_ignore_ascii_case("aukgit") {
            report.aukgit_attributions_found += 1;
        } else {
            report.has_third_party_handles = true;
            report
                .violations
                .push(AttributionViolation::NonAukgitContributorInReleaseNotes {
                    handle: handle.clone(),
                });
        }
    }

    report.is_compliant = report.violations.is_empty();
    report
}

/// Audits a raw list of contributor handles for release gate authorization.
pub fn audit_release_notes_contributors(contributors: &[&str]) -> Vec<AttributionViolation> {
    let mut violations = Vec::new();

    for contributor in contributors {
        let clean = contributor.trim().trim_start_matches('@');
        if !clean.eq_ignore_ascii_case("aukgit") {
            let formatted = if contributor.starts_with('@') {
                contributor.to_string()
            } else {
                format!("@{contributor}")
            };
            violations.push(AttributionViolation::NonAukgitContributorInReleaseNotes {
                handle: formatted,
            });
        }
    }

    violations
}
