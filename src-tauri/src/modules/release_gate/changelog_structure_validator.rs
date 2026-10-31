//! Changelog Release Note Structure & Attribution Validator.
//!
//! Enforces release documentation discipline per AGENTS.md:
//! - Heading tag format matching `vX.Y.Z` or `vX.Y.Z-beta.N`.
//! - Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
//! - No unauthorized GitHub handles (`@...`) in release notes.
//! - Zero forbidden absolute filesystem paths (`D:\`, `C:\`, `/home/`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChangelogStructureReport {
    pub is_valid: bool,
    pub has_valid_heading: bool,
    pub has_aukgit_attribution: bool,
    pub unauthorized_handles: Vec<String>,
    pub forbidden_absolute_paths: Vec<String>,
    pub violation_reasons: Vec<String>,
}

/// Evaluates the structural compliance of a changelog release block.
pub fn validate_changelog_release_entry(
    entry_text: &str,
    expected_version: &str,
) -> ChangelogStructureReport {
    let mut has_valid_heading = false;
    let mut has_aukgit_attribution = false;
    let mut unauthorized_handles = Vec::new();
    let mut forbidden_absolute_paths = Vec::new();
    let mut violation_reasons = Vec::new();

    let clean_expected = if expected_version.starts_with('v') {
        expected_version.to_string()
    } else {
        format!("v{}", expected_version)
    };

    for line in entry_text.lines() {
        let trimmed = line.trim();

        // Heading check
        if trimmed.starts_with("## [") || trimmed.starts_with("## v") || trimmed.starts_with("## ")
        {
            if trimmed.contains(&clean_expected) {
                has_valid_heading = true;
            }
        }

        // Attribution check
        if trimmed.contains("@aukgit") {
            has_aukgit_attribution = true;
        }

        // Handle checks (@user)
        for word in trimmed.split_whitespace() {
            let clean_word =
                word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '-');
            if clean_word.starts_with('@') && clean_word.len() > 1 {
                let handle = clean_word.trim_start_matches('@');
                if handle != "aukgit" {
                    unauthorized_handles.push(clean_word.to_string());
                }
            }
        }

        // Forbidden absolute paths
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("d:\\") || lower.contains("c:\\") || lower.contains("/home/") {
            forbidden_absolute_paths.push(trimmed.to_string());
        }
    }

    if !has_valid_heading {
        violation_reasons.push(format!(
            "Missing heading matching expected version '{}'",
            clean_expected
        ));
    }
    if !has_aukgit_attribution {
        violation_reasons.push("Missing required attribution to '@aukgit'".to_string());
    }
    if !unauthorized_handles.is_empty() {
        violation_reasons.push(format!(
            "Found unauthorized user handles in release notes: {:?}",
            unauthorized_handles
        ));
    }
    if !forbidden_absolute_paths.is_empty() {
        violation_reasons.push(format!(
            "Found forbidden absolute paths in release notes: {:?}",
            forbidden_absolute_paths
        ));
    }

    let is_valid = violation_reasons.is_empty();

    ChangelogStructureReport {
        is_valid,
        has_valid_heading,
        has_aukgit_attribution,
        unauthorized_handles,
        forbidden_absolute_paths,
        violation_reasons,
    }
}
