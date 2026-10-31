//! Contributor Authorship Preservation & Cost Disclosure Guard (`Imp-699`).
//!
//! Enforces AGENTS.md Contributor Respect & Attribution discipline:
//! "Preserve authorship by preferring the contributor's own PR for squash commits,
//! or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs.
//! Disclose costs before merging: highlight affected existing behaviors and unverified
//! paths alongside improvements."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Git commit integration strategy used when merging or incorporating external contributions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommitType {
    /// Squash commit directly originating from the contributor's own PR branch.
    SquashFromContributorPr,
    /// Merge commit integrating a contributor branch; requires explicit `Co-authored-by:` trailer.
    MergeCommit,
    /// Proxy PR commit authored or landed by maintainer; requires explicit `Co-authored-by:` trailer.
    ProxyPrCommit,
}

impl CommitType {
    /// Returns true if this is a squash commit from contributor PR.
    pub fn is_squash(&self) -> bool {
        matches!(self, Self::SquashFromContributorPr)
    }

    /// Returns true if this is a merge commit.
    pub fn is_merge(&self) -> bool {
        matches!(self, Self::MergeCommit)
    }

    /// Returns true if this is a proxy PR commit.
    pub fn is_proxy_pr(&self) -> bool {
        matches!(self, Self::ProxyPrCommit)
    }

    /// Canonical string identifier for the commit integration type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SquashFromContributorPr => "squash_from_contributor_pr",
            Self::MergeCommit => "merge_commit",
            Self::ProxyPrCommit => "proxy_pr_commit",
        }
    }
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SquashFromContributorPr => write!(f, "SquashFromContributorPr"),
            Self::MergeCommit => write!(f, "MergeCommit"),
            Self::ProxyPrCommit => write!(f, "ProxyPrCommit"),
        }
    }
}

/// Disclosed costs, affected behaviors, and unverified execution paths prior to merge.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CostDisclosure {
    /// Existing features, behaviors, or contract details modified by the change.
    pub affected_existing_behaviors: Vec<String>,
    /// Execution environments, platforms, or codepaths unverified by local or CI test suites.
    pub unverified_paths: Vec<String>,
    /// Whether cost disclosure has been evaluated as satisfied.
    pub has_disclosed_costs: bool,
}

impl CostDisclosure {
    /// Creates a new `CostDisclosure` struct, automatically evaluating `has_disclosed_costs`.
    pub fn new(affected: Vec<String>, unverified: Vec<String>) -> Self {
        let has_affected = affected.iter().any(|s| !is_trivial_entry(s));
        let has_unverified = unverified.iter().any(|s| !is_trivial_entry(s));
        let has_disclosed_costs = has_affected || has_unverified;

        Self {
            affected_existing_behaviors: affected,
            unverified_paths: unverified,
            has_disclosed_costs,
        }
    }

    /// Creates an empty `CostDisclosure` struct with `has_disclosed_costs: false`.
    pub fn empty() -> Self {
        Self {
            affected_existing_behaviors: Vec::new(),
            unverified_paths: Vec::new(),
            has_disclosed_costs: false,
        }
    }

    /// Returns true if both affected behaviors and unverified paths are empty.
    pub fn is_empty(&self) -> bool {
        self.affected_existing_behaviors.is_empty() && self.unverified_paths.is_empty()
    }
}

/// Audit report validating contributor authorship preservation and cost disclosure compliance.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuthorshipPreservationReport {
    /// Whether the commit satisfies all authorship preservation and cost disclosure invariants.
    pub is_compliant: bool,
    /// Whether the commit has a valid, verified author attribution mechanism.
    pub has_valid_author: bool,
    /// Standardized `Co-authored-by:` trailer lines extracted from the commit message.
    pub coauthor_trailers: Vec<String>,
    /// Evaluated cost disclosure details.
    pub cost_disclosure: CostDisclosure,
    /// List of violations preventing compliance.
    pub violations: Vec<String>,
}

impl AuthorshipPreservationReport {
    /// Returns true if any violations were recorded.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns the number of validated `Co-authored-by:` trailers.
    pub fn trailer_count(&self) -> usize {
        self.coauthor_trailers.len()
    }
}

/// Formats a standardized git `Co-authored-by:` trailer.
pub fn format_coauthor_trailer(name: &str, email: &str) -> String {
    let clean_name = name.trim();
    let clean_email = email.trim().trim_start_matches('<').trim_end_matches('>');
    format!("Co-authored-by: {} <{}>", clean_name, clean_email)
}

/// Alias for `format_coauthor_trailer` with explicit contributor naming.
pub fn format_contributor_coauthor_trailer(name: &str, email: &str) -> String {
    format_coauthor_trailer(name, email)
}

/// Formats multiple `Co-authored-by:` trailer lines from name/email pairs.
pub fn format_coauthor_trailers(coauthors: &[(&str, &str)]) -> Vec<String> {
    coauthors
        .iter()
        .map(|(name, email)| format_coauthor_trailer(name, email))
        .collect()
}

/// Appends `Co-authored-by:` trailer lines to the end of a commit message body.
pub fn attach_coauthor_trailers(commit_msg: &str, trailers: &[String]) -> String {
    let mut out = commit_msg.trim_end().to_string();
    if !trailers.is_empty() {
        out.push_str("\n\n");
        for trailer in trailers {
            out.push_str(trailer);
            out.push('\n');
        }
    }
    out
}

/// Checks whether a string entry is a trivial or empty placeholder.
fn is_trivial_entry(entry: &str) -> bool {
    let lower = entry.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "" | "none" | "n/a" | "na" | "nil" | "-" | "--" | "no" | "nothing" | "none."
    )
}

/// Parses and extracts validated `Co-authored-by:` trailers from a commit message.
///
/// Returns valid trailers and any malformed trailer lines found.
pub fn parse_coauthor_trailers_from_msg(commit_msg: &str) -> (Vec<String>, Vec<String>) {
    let mut valid_trailers = Vec::new();
    let mut invalid_trailers = Vec::new();

    for line in commit_msg.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("co-authored-by:") {
            let content = trimmed["co-authored-by:".len()..].trim();
            if let (Some(open), Some(close)) = (content.find('<'), content.find('>')) {
                if open < close {
                    let name = content[..open].trim();
                    let email = content[open + 1..close].trim();
                    if !name.is_empty() && email.contains('@') && email.contains('.') {
                        valid_trailers.push(format!("Co-authored-by: {} <{}>", name, email));
                        continue;
                    }
                }
            }
            invalid_trailers.push(trimmed.to_string());
        }
    }

    (valid_trailers, invalid_trailers)
}

/// Attempts to parse markdown cost disclosure sections from a commit message text.
pub fn parse_cost_disclosure_from_text(text: &str) -> CostDisclosure {
    let mut affected = Vec::new();
    let mut unverified = Vec::new();

    let mut current_section: Option<&str> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.contains("affected existing behavior") || lower.contains("affected behavior") {
            current_section = Some("affected");
            continue;
        } else if lower.contains("unverified path") {
            current_section = Some("unverified");
            continue;
        } else if lower.starts_with('#') || lower.starts_with("co-authored-by:") {
            current_section = None;
            continue;
        }

        if let Some(section) = current_section {
            if !trimmed.is_empty() {
                let clean_item = trimmed
                    .trim_start_matches('-')
                    .trim_start_matches('*')
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
                    .trim();

                if !clean_item.is_empty() && !is_trivial_entry(clean_item) {
                    if section == "affected" {
                        affected.push(clean_item.to_string());
                    } else if section == "unverified" {
                        unverified.push(clean_item.to_string());
                    }
                }
            }
        }
    }

    CostDisclosure::new(affected, unverified)
}

/// Validates that an external contributor's authorship is preserved and costs are disclosed.
///
/// Rules per AGENTS.md:
/// - Squash commits: Preferred for contributor's own PR; author must be the contributor.
/// - Merge commits & Proxy PRs: Require explicit, valid `Co-authored-by:` trailers.
/// - Cost disclosure: Must highlight affected existing behaviors and unverified paths.
pub fn validate_contributor_authorship(
    commit_msg: &str,
    commit_type: CommitType,
    original_contributor: Option<&str>,
    cost_disclosure: Option<&CostDisclosure>,
) -> AuthorshipPreservationReport {
    let mut violations = Vec::new();
    let (coauthor_trailers, malformed_trailers) = parse_coauthor_trailers_from_msg(commit_msg);

    // Record any malformed trailers as violations
    for malformed in malformed_trailers {
        violations.push(format!(
            "Malformed Co-authored-by trailer syntax: '{}'",
            malformed
        ));
    }

    // 1. Resolve Cost Disclosure
    let resolved_cost_disclosure = match cost_disclosure {
        Some(cd) => {
            if cd.has_disclosed_costs {
                cd.clone()
            } else {
                // If provided cost disclosure object lacks costs, check if text has them
                let from_text = parse_cost_disclosure_from_text(commit_msg);
                if from_text.has_disclosed_costs {
                    from_text
                } else {
                    cd.clone()
                }
            }
        }
        None => parse_cost_disclosure_from_text(commit_msg),
    };

    if !resolved_cost_disclosure.has_disclosed_costs {
        violations.push(
            "Cost disclosure missing: must highlight affected existing behaviors and unverified paths before merging"
                .to_string(),
        );
    }

    // 2. Validate Authorship Preservation depending on CommitType
    let mut has_valid_author = false;

    match commit_type {
        CommitType::SquashFromContributorPr => {
            // For squash commit from contributor's own PR, contributor is the primary commit author
            if let Some(contrib) = original_contributor {
                let trimmed = contrib.trim();
                if trimmed.is_empty() {
                    violations.push(
                        "Squash commit requires valid original contributor author".to_string(),
                    );
                } else {
                    has_valid_author = true;
                }
            } else if !coauthor_trailers.is_empty() {
                has_valid_author = true;
            } else {
                violations.push(
                    "Squash commit from contributor PR requires identified original contributor"
                        .to_string(),
                );
            }
        }
        CommitType::MergeCommit => {
            // Merge commit requires explicit Co-authored-by: trailer
            if coauthor_trailers.is_empty() {
                violations.push(
                    "Merge commit requires explicit `Co-authored-by:` trailer to preserve contributor authorship"
                        .to_string(),
                );
            } else {
                has_valid_author = true;
                if let Some(contrib) = original_contributor {
                    let norm = contrib.trim().trim_start_matches('@').to_ascii_lowercase();
                    let matched = coauthor_trailers
                        .iter()
                        .any(|t| t.to_ascii_lowercase().contains(&norm));

                    if !matched {
                        violations.push(format!(
                            "Merge commit `Co-authored-by:` trailers do not attribute original contributor '{}'",
                            contrib
                        ));
                        has_valid_author = false;
                    }
                }
            }
        }
        CommitType::ProxyPrCommit => {
            // Proxy PR requires explicit Co-authored-by: trailer
            if coauthor_trailers.is_empty() {
                violations.push(
                    "Proxy PR commit missing mandatory `Co-authored-by:` trailer attributing original contributor"
                        .to_string(),
                );
            } else {
                has_valid_author = true;
                if let Some(contrib) = original_contributor {
                    let norm = contrib.trim().trim_start_matches('@').to_ascii_lowercase();
                    let matched = coauthor_trailers
                        .iter()
                        .any(|t| t.to_ascii_lowercase().contains(&norm));

                    if !matched {
                        violations.push(format!(
                            "Proxy PR `Co-authored-by:` trailers do not attribute original contributor '{}'",
                            contrib
                        ));
                        has_valid_author = false;
                    }
                }
            }
        }
    }

    let is_compliant =
        has_valid_author && resolved_cost_disclosure.has_disclosed_costs && violations.is_empty();

    AuthorshipPreservationReport {
        is_compliant,
        has_valid_author,
        coauthor_trailers,
        cost_disclosure: resolved_cost_disclosure,
        violations,
    }
}
