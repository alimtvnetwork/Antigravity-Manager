//! Author Attribution Compliance Verifier.
//! Enforces:
//! - Contributor attribution in changelogs must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).
//! - Bars any other GitHub user handles (`@...`) from release notes and changelogs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionAuditResult {
    pub is_compliant: bool,
    pub authorized_handles: Vec<String>,
    pub disallowed_handles: Vec<String>,
}

/// Scans markdown content for `@<handle>` occurrences and verifies strict exclusivity for `@aukgit`.
pub fn verify_author_attribution(content: &str) -> AttributionAuditResult {
    let mut authorized = Vec::new();
    let mut disallowed = Vec::new();

    for word in content.split_whitespace() {
        // Strip trailing punctuation like ',', '.', ')', ']'
        let clean = word.trim_matches(|c: char| c == '(' || c == ')' || c == '[' || c == ']' || c == ',' || c == '.');
        if clean.starts_with('@') && clean.len() > 1 {
            let handle = &clean[1..];
            // Valid handle characters (alphanumeric and hyphens)
            if handle.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                if handle.eq_ignore_ascii_case("aukgit") {
                    if !authorized.contains(&"@aukgit".to_string()) {
                        authorized.push("@aukgit".to_string());
                    }
                } else {
                    let formatted = format!("@{handle}");
                    if !disallowed.contains(&formatted) {
                        disallowed.push(formatted);
                    }
                }
            }
        }
    }

    AttributionAuditResult {
        is_compliant: disallowed.is_empty(),
        authorized_handles: authorized,
        disallowed_handles: disallowed,
    }
}
