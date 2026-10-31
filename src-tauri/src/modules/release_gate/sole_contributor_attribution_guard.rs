//! Sole Contributor Attribution Compliance Guard.
//!
//! Enforces contributor attribution discipline per AGENTS.md:
//! "Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit`
//! (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other
//! GitHub user handles (`@...`) in release changelogs or release notes, ensuring the
//! GitHub release page contributors list only contains `aukgit` and none else."

use serde::{Deserialize, Serialize};

/// Violations identified during changelog author attribution audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributionViolation {
    DisallowedContributorHandle {
        line_number: usize,
        handle: String,
        snippet: String,
    },
    MissingAukgitAttribution {
        file_name: String,
    },
}

impl std::fmt::Display for AttributionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisallowedContributorHandle {
                line_number,
                handle,
                snippet,
            } => {
                write!(
                    f,
                    "Disallowed contributor handle '{}' at line {}: '{}'",
                    handle, line_number, snippet
                )
            }
            Self::MissingAukgitAttribution { file_name } => {
                write!(
                    f,
                    "Missing mandatory attribution to '@aukgit' in '{}'",
                    file_name
                )
            }
        }
    }
}

impl std::error::Error for AttributionViolation {}

/// Summary report of changelog attribution compliance audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionAuditReport {
    pub is_compliant: bool,
    pub total_lines_scanned: usize,
    pub aukgit_attributions_count: usize,
    pub violations: Vec<AttributionViolation>,
}

impl AttributionAuditReport {
    /// Creates a report with line scan count initialized.
    pub fn new(total_lines: usize) -> Self {
        Self {
            is_compliant: false,
            total_lines_scanned: total_lines,
            aukgit_attributions_count: 0,
            violations: Vec::new(),
        }
    }
}

/// Checks whether a byte index at '@' is preceded by a non-alphanumeric boundary.
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

/// Scans a single line for github handle occurrences and audits them.
fn scan_line_handles(
    line: &str,
    line_number: usize,
    aukgit_count: &mut usize,
    violations: &mut Vec<AttributionViolation>,
) {
    static HANDLE_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"@([a-zA-Z0-9_\-]+)").expect("valid handle regex")
    });

    for mat in HANDLE_REGEX.find_iter(line) {
        if !is_handle_prefix_boundary(line, mat.start()) {
            continue;
        }
        let raw_handle = &mat.as_str()[1..];
        if raw_handle.eq_ignore_ascii_case("aukgit") {
            *aukgit_count += 1;
        } else {
            violations.push(AttributionViolation::DisallowedContributorHandle {
                line_number,
                handle: format!("@{raw_handle}"),
                snippet: line.trim().to_string(),
            });
        }
    }
}

/// Audits changelog markdown content for exclusive `@aukgit` attribution compliance.
pub fn audit_changelog_attribution(content: &str, file_name: &str) -> AttributionAuditReport {
    let mut aukgit_count = 0;
    let mut violations = Vec::new();
    let total_lines = content.lines().count();

    for (idx, line) in content.lines().enumerate() {
        scan_line_handles(line, idx + 1, &mut aukgit_count, &mut violations);
    }

    if aukgit_count == 0 {
        violations.push(AttributionViolation::MissingAukgitAttribution {
            file_name: file_name.to_string(),
        });
    }

    let is_compliant = violations.is_empty();

    AttributionAuditReport {
        is_compliant,
        total_lines_scanned: total_lines,
        aukgit_attributions_count: aukgit_count,
        violations,
    }
}
