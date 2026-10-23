//! Tag & Changelog Heading Exact Matcher Gate (Imp-632).
//!
//! Enforces tag and release branch discipline per AGENTS.md:
//! "Commit, Tag & Push: Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`),
//! reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`).
//! The release gate strictly intercepts cross-branch misplacement. Tags must match
//! `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix)."

use serde::{Deserialize, Serialize};

/// Target release branch designation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseBranch {
    Main,
    Beta,
    Other(String),
}

impl ReleaseBranch {
    /// Returns the string representation of the branch name.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Beta => "beta",
            Self::Other(name) => name.as_str(),
        }
    }

    /// Returns true if the release branch is main.
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    /// Returns true if the release branch is beta.
    pub fn is_beta(&self) -> bool {
        matches!(self, Self::Beta)
    }
}

impl std::fmt::Display for ReleaseBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Violations identified when validating git tags against release branches and changelogs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagMatchingViolation {
    MissingLeadingVPrefix {
        tag: String,
    },
    MismatchedChangelogHeading {
        tag: String,
        expected_heading: String,
    },
    CrossBranchTagMisplacement {
        tag: String,
        target_branch: String,
        guidance: String,
    },
    InvalidSemVerFormat {
        tag: String,
        reason: String,
    },
}

impl std::fmt::Display for TagMatchingViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLeadingVPrefix { tag } => {
                write!(
                    f,
                    "Tag '{tag}' is missing required leading 'v' prefix (e.g. 'v4.250.0')"
                )
            }
            Self::MismatchedChangelogHeading {
                tag,
                expected_heading,
            } => {
                write!(
                    f,
                    "Tag '{tag}' does not match changelog heading character-for-character. Expected heading: '{expected_heading}'"
                )
            }
            Self::CrossBranchTagMisplacement {
                tag,
                target_branch,
                guidance,
            } => {
                write!(
                    f,
                    "Cross-branch tag misplacement: tag '{tag}' placed on branch '{target_branch}'. Guidance: {guidance}"
                )
            }
            Self::InvalidSemVerFormat { tag, reason } => {
                write!(f, "Invalid SemVer format for tag '{tag}': {reason}")
            }
        }
    }
}

impl std::error::Error for TagMatchingViolation {}

/// Summary report of tag validation against changelog heading and target branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMatchingReport {
    pub is_compliant: bool,
    pub tag: String,
    pub target_branch: String,
    pub matched_heading: Option<String>,
    pub violations: Vec<TagMatchingViolation>,
    pub is_beta_prerelease: bool,
}

impl TagMatchingReport {
    /// Returns true if all validation checks passed without violations.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Returns true if any validation violations were discovered.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns the number of recorded violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Normalizes branch references by stripping common ref prefixes.
pub fn normalize_branch(branch_name: &str) -> ReleaseBranch {
    let trimmed = branch_name.trim();
    let stripped = trimmed.strip_prefix("refs/heads/").unwrap_or(trimmed);
    let stripped = stripped.strip_prefix("origin/").unwrap_or(stripped);

    if stripped.eq_ignore_ascii_case("main") {
        ReleaseBranch::Main
    } else if stripped.eq_ignore_ascii_case("beta") {
        ReleaseBranch::Beta
    } else {
        ReleaseBranch::Other(trimmed.to_string())
    }
}

/// Determines whether a tag constitutes a beta pre-release version.
pub fn is_beta_prerelease(tag: &str) -> bool {
    tag.contains("-beta.")
}

/// Validates the dot-separated numeric segments in a version core.
fn validate_core_segments(core: &str) -> Result<(), String> {
    let segments: Vec<&str> = core.split('.').collect();
    if segments.len() != 3 {
        return Err(format!(
            "Expected 3 dot-separated version segments (X.Y.Z), found {} in '{core}'",
            segments.len()
        ));
    }

    for (idx, seg) in segments.iter().enumerate() {
        if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "Segment {} ('{seg}') is not a valid non-negative integer",
                idx + 1
            ));
        }
    }

    Ok(())
}

/// Validates SemVer format for standard and beta pre-release tags.
pub fn validate_semver_format(tag: &str) -> Result<(), String> {
    let core_version = tag.strip_prefix('v').unwrap_or(tag);
    if let Some(idx) = core_version.find("-beta.") {
        let (main_part, beta_tail) = core_version.split_at(idx);
        let beta_num_str = beta_tail.trim_start_matches("-beta.");
        if beta_num_str.is_empty() || !beta_num_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "Beta pre-release identifier must be numeric after '-beta.', found '{beta_num_str}'"
            ));
        }
        validate_core_segments(main_part)
    } else {
        validate_core_segments(core_version)
    }
}

/// Validates branch placement according to stable vs beta release rules.
fn check_branch_placement(
    tag: &str,
    target_branch: &str,
    is_beta: bool,
) -> Option<TagMatchingViolation> {
    let parsed_branch = normalize_branch(target_branch);
    if is_beta && parsed_branch != ReleaseBranch::Beta {
        Some(TagMatchingViolation::CrossBranchTagMisplacement {
            tag: tag.to_string(),
            target_branch: target_branch.to_string(),
            guidance: "Beta pre-release tags (vX.Y.Z-beta.N) must be placed on 'beta' branch, reserving 'main' exclusively for stable releases.".to_string(),
        })
    } else if !is_beta && parsed_branch != ReleaseBranch::Main {
        Some(TagMatchingViolation::CrossBranchTagMisplacement {
            tag: tag.to_string(),
            target_branch: target_branch.to_string(),
            guidance: "Stable release tags (vX.Y.Z) must be placed on 'main' branch, reserving 'beta' exclusively for pre-releases.".to_string(),
        })
    } else {
        None
    }
}

/// Attempts to match a bracketed changelog heading for the tag.
fn match_bracketed_heading(trimmed_line: &str, tag: &str) -> Option<String> {
    let open_idx = trimmed_line.find('[')?;
    let close_idx = trimmed_line[open_idx + 1..].find(']')?;
    let inside = &trimmed_line[open_idx + 1..open_idx + 1 + close_idx];
    if inside == tag {
        Some(trimmed_line.to_string())
    } else {
        None
    }
}

/// Attempts to match an unbracketed changelog heading for the tag.
fn match_unbracketed_heading(trimmed_line: &str, tag: &str) -> Option<String> {
    let heading_body = trimmed_line.trim_start_matches('#').trim_start();
    let first_token = heading_body.split_whitespace().next().unwrap_or("");
    if first_token == tag {
        Some(trimmed_line.to_string())
    } else {
        None
    }
}

/// Searches changelog markdown lines for a character-for-character heading match.
pub fn find_matching_changelog_heading(changelog_text: &str, tag: &str) -> Option<String> {
    for line in changelog_text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }

        if let Some(matched) = match_bracketed_heading(trimmed, tag) {
            return Some(matched);
        }

        if let Some(matched) = match_unbracketed_heading(trimmed, tag) {
            return Some(matched);
        }
    }
    None
}

/// Validates tag format, branch placement, and character-for-character changelog heading.
pub fn validate_tag_and_heading(
    tag: &str,
    target_branch: &str,
    changelog_text: &str,
) -> TagMatchingReport {
    let mut violations = Vec::new();

    if !tag.starts_with('v') {
        violations.push(TagMatchingViolation::MissingLeadingVPrefix {
            tag: tag.to_string(),
        });
    }

    if let Err(reason) = validate_semver_format(tag) {
        violations.push(TagMatchingViolation::InvalidSemVerFormat {
            tag: tag.to_string(),
            reason,
        });
    }

    let is_beta = is_beta_prerelease(tag);
    if let Some(misplacement) = check_branch_placement(tag, target_branch, is_beta) {
        violations.push(misplacement);
    }

    let matched_heading = find_matching_changelog_heading(changelog_text, tag);
    if matched_heading.is_none() {
        violations.push(TagMatchingViolation::MismatchedChangelogHeading {
            tag: tag.to_string(),
            expected_heading: format!("## [{tag}]"),
        });
    }

    let is_compliant = violations.is_empty();

    TagMatchingReport {
        is_compliant,
        tag: tag.to_string(),
        target_branch: target_branch.to_string(),
        matched_heading,
        violations,
        is_beta_prerelease: is_beta,
    }
}
