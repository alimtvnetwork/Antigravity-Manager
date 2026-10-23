//! Tag & Changelog Heading Matcher.
//!
//! Enforces tag and release branch discipline per AGENTS.md:
//! "Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`),
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
pub enum TagMatchViolation {
    TagHeadingMismatch {
        tag: String,
        expected_heading: String,
        guidance: String,
    },
    CrossBranchMisplacement {
        tag: String,
        branch: String,
        expected_branch: String,
    },
}

impl std::fmt::Display for TagMatchViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TagHeadingMismatch {
                tag,
                expected_heading,
                guidance,
            } => {
                write!(
                    f,
                    "Tag heading mismatch for tag '{tag}': expected heading '{expected_heading}'. Guidance: {guidance}"
                )
            }
            Self::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                write!(
                    f,
                    "Cross-branch misplacement: tag '{tag}' placed on branch '{branch}', but expected branch '{expected_branch}'"
                )
            }
        }
    }
}

impl std::error::Error for TagMatchViolation {}

/// Summary report of tag validation against changelog heading and target branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMatchReport {
    pub is_valid: bool,
    pub tag: String,
    pub branch: String,
    pub matched_heading: Option<String>,
    pub violations: Vec<TagMatchViolation>,
}

/// Parses a branch reference into a standardized `ReleaseBranch`.
pub fn parse_release_branch(branch_name: &str) -> ReleaseBranch {
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

/// Determines whether a tag constitutes a pre-release version.
pub fn is_prerelease_tag(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    lower.contains("-beta.") || lower.contains("-alpha.") || lower.contains("-rc.")
}

/// Validates branch alignment for stable vs pre-release tags.
fn validate_branch_alignment(
    tag: &str,
    branch: &str,
    is_prerelease: bool,
) -> Option<TagMatchViolation> {
    let parsed = parse_release_branch(branch);
    if is_prerelease && parsed != ReleaseBranch::Beta {
        Some(TagMatchViolation::CrossBranchMisplacement {
            tag: tag.to_string(),
            branch: branch.to_string(),
            expected_branch: "beta".to_string(),
        })
    } else if !is_prerelease && parsed != ReleaseBranch::Main {
        Some(TagMatchViolation::CrossBranchMisplacement {
            tag: tag.to_string(),
            branch: branch.to_string(),
            expected_branch: "main".to_string(),
        })
    } else {
        None
    }
}

/// Scans changelog markdown lines for a character-for-character heading match.
fn find_character_matching_heading(content: &str, tag: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }
        if let Some(open) = trimmed.find('[') {
            if let Some(close) = trimmed[open + 1..].find(']') {
                let inside = &trimmed[open + 1..open + 1 + close];
                if inside == tag {
                    return Some(trimmed.to_string());
                }
            }
        }
        let heading_body = trimmed.trim_start_matches('#').trim_start();
        let first_token = heading_body.split_whitespace().next().unwrap_or("");
        if first_token == tag {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Validates tag alignment with release branch and character-for-character changelog heading.
pub fn validate_tag_and_heading(
    tag: &str,
    branch: &str,
    changelog_content: &str,
) -> TagMatchReport {
    let mut violations = Vec::new();
    let is_prerelease = is_prerelease_tag(tag);

    if let Some(violation) = validate_branch_alignment(tag, branch, is_prerelease) {
        violations.push(violation);
    }

    let matched_heading = find_character_matching_heading(changelog_content, tag);
    if matched_heading.is_none() {
        violations.push(TagMatchViolation::TagHeadingMismatch {
            tag: tag.to_string(),
            expected_heading: format!("## [{tag}]"),
            guidance: format!(
                "Add a heading matching '{tag}' character-for-character, e.g. '## [{tag}] - YYYY-MM-DD'"
            ),
        });
    }

    let is_valid = violations.is_empty();

    TagMatchReport {
        is_valid,
        tag: tag.to_string(),
        branch: branch.to_string(),
        matched_heading,
        violations,
    }
}
