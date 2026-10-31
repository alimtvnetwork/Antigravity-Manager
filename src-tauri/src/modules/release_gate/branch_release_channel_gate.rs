//! Release Gate: Branch Release Channel Separation Gate (Imp-656).
//!
//! Enforces AGENTS.md Release Channel Separation & Discipline:
//! - Stable Releases (`vX.Y.Z`): Allowed exclusively on `main` branch. Rejected if on `beta` or feature branches.
//! - Preview / Beta Releases (`vX.Y.Z-beta.N`): Allowed exclusively on `beta` branch. Rejected if on `main`.
//! - Character-for-character match with `CHANGELOG.md` headings (including `v` prefix and pre-release suffix).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

/// Branch release channel category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchReleaseChannel {
    Stable,
    Beta,
}

impl fmt::Display for BranchReleaseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Beta => write!(f, "Beta/Preview"),
        }
    }
}

/// Normalized git release branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseGateBranch {
    Main,
    Beta,
    Feature(String),
}

impl ReleaseGateBranch {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Beta => "beta",
            Self::Feature(name) => name.as_str(),
        }
    }

    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    pub fn is_beta(&self) -> bool {
        matches!(self, Self::Beta)
    }

    pub fn is_feature(&self) -> bool {
        matches!(self, Self::Feature(_))
    }
}

impl fmt::Display for ReleaseGateBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Violations identified during release channel and branch gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchReleaseViolation {
    MissingVPrefix {
        tag: String,
    },
    InvalidSemVerFormat {
        tag: String,
        reason: String,
    },
    BranchChannelMismatch {
        tag: String,
        current_branch: String,
        channel: BranchReleaseChannel,
        expected_branch: String,
        guidance: String,
    },
    ChangelogHeadingNotFound {
        tag: String,
        expected_heading: String,
    },
    ChangelogHeadingCharacterMismatch {
        tag: String,
        expected_heading: String,
        actual_heading: String,
    },
}

impl fmt::Display for BranchReleaseViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVPrefix { tag } => {
                write!(
                    f,
                    "Release tag '{tag}' is missing required leading 'v' prefix (expected format: 'vX.Y.Z' or 'vX.Y.Z-beta.N')"
                )
            }
            Self::InvalidSemVerFormat { tag, reason } => {
                write!(f, "Invalid SemVer format in release tag '{tag}': {reason}")
            }
            Self::BranchChannelMismatch {
                tag,
                current_branch,
                channel,
                expected_branch,
                guidance,
            } => {
                write!(
                    f,
                    "Branch release channel mismatch for tag '{tag}' ({channel}): branch '{current_branch}' is forbidden. Expected '{expected_branch}'. {guidance}"
                )
            }
            Self::ChangelogHeadingNotFound {
                tag,
                expected_heading,
            } => {
                write!(
                    f,
                    "Tag '{tag}' not found in CHANGELOG.md headings. Expected exact heading matching '{expected_heading}'"
                )
            }
            Self::ChangelogHeadingCharacterMismatch {
                tag,
                expected_heading,
                actual_heading,
            } => {
                write!(
                    f,
                    "Tag '{tag}' failed character-for-character match with changelog heading. Expected '{expected_heading}', found '{actual_heading}'"
                )
            }
        }
    }
}

impl std::error::Error for BranchReleaseViolation {}

/// Comprehensive audit report for branch and release channel compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseChannelValidationReport {
    pub is_valid: bool,
    pub tag: String,
    pub current_branch: String,
    pub channel: BranchReleaseChannel,
    pub matched_heading: Option<String>,
    pub violations: Vec<BranchReleaseViolation>,
}

impl ReleaseChannelValidationReport {
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    pub fn violation_messages(&self) -> Vec<String> {
        self.violations.iter().map(|v| v.to_string()).collect()
    }
}

fn validate_core_triplet(triplet: &str) -> Result<(), String> {
    let parts: Vec<&str> = triplet.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Expected 3 dot-separated integers (X.Y.Z), found {} segments in '{triplet}'",
            parts.len()
        ));
    }
    for (idx, seg) in parts.iter().enumerate() {
        if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "Version segment {} ('{seg}') must be non-negative numeric digits",
                idx + 1
            ));
        }
    }
    Ok(())
}

/// Parses the release tag into a release channel, validating SemVer integrity.
pub fn parse_release_channel(tag: &str) -> Result<BranchReleaseChannel, BranchReleaseViolation> {
    let tag = tag.trim();
    if !tag.starts_with('v') {
        return Err(BranchReleaseViolation::MissingVPrefix {
            tag: tag.to_string(),
        });
    }

    let core = &tag[1..];
    if let Some(idx) = core.find("-beta.") {
        let (triplet, beta_suffix) = core.split_at(idx);
        let beta_num = beta_suffix.trim_start_matches("-beta.");
        if beta_num.is_empty() || !beta_num.chars().all(|c| c.is_ascii_digit()) {
            return Err(BranchReleaseViolation::InvalidSemVerFormat {
                tag: tag.to_string(),
                reason: format!(
                    "Beta pre-release identifier must be numeric after '-beta.', got '{beta_num}'"
                ),
            });
        }
        if let Err(reason) = validate_core_triplet(triplet) {
            return Err(BranchReleaseViolation::InvalidSemVerFormat {
                tag: tag.to_string(),
                reason,
            });
        }
        Ok(BranchReleaseChannel::Beta)
    } else if let Some(idx) = core.find("-beta") {
        let (triplet, _) = core.split_at(idx);
        if let Err(reason) = validate_core_triplet(triplet) {
            return Err(BranchReleaseViolation::InvalidSemVerFormat {
                tag: tag.to_string(),
                reason,
            });
        }
        Ok(BranchReleaseChannel::Beta)
    } else if let Some(idx) = core.find('-') {
        let (triplet, _) = core.split_at(idx);
        if let Err(reason) = validate_core_triplet(triplet) {
            return Err(BranchReleaseViolation::InvalidSemVerFormat {
                tag: tag.to_string(),
                reason,
            });
        }
        Ok(BranchReleaseChannel::Beta)
    } else {
        if let Err(reason) = validate_core_triplet(core) {
            return Err(BranchReleaseViolation::InvalidSemVerFormat {
                tag: tag.to_string(),
                reason,
            });
        }
        Ok(BranchReleaseChannel::Stable)
    }
}

/// Normalizes branch names by stripping remote prefixes and ref heads.
pub fn normalize_git_branch(branch: &str) -> ReleaseGateBranch {
    let trimmed = branch.trim();
    let stripped = trimmed.strip_prefix("refs/heads/").unwrap_or(trimmed);
    let stripped = stripped.strip_prefix("refs/tags/").unwrap_or(stripped);
    let stripped = stripped.strip_prefix("origin/").unwrap_or(stripped);

    if stripped.eq_ignore_ascii_case("main") {
        ReleaseGateBranch::Main
    } else if stripped.eq_ignore_ascii_case("beta") {
        ReleaseGateBranch::Beta
    } else {
        ReleaseGateBranch::Feature(stripped.to_string())
    }
}

/// Searches changelog markdown for character-for-character matching headings.
pub fn find_changelog_heading_exact(
    changelog_content: &str,
    tag: &str,
) -> Result<String, BranchReleaseViolation> {
    let tag = tag.trim();
    let tag_without_v = tag.strip_prefix('v').unwrap_or(tag);
    let mut near_miss_heading = None;

    for line in changelog_content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }

        // Bracketed heading: e.g. ## [v4.250.0] or ## [v4.250.0] - 2026-10-23
        if let (Some(open_idx), Some(close_idx)) = (trimmed.find('['), trimmed.find(']')) {
            if open_idx < close_idx {
                let inside = &trimmed[open_idx + 1..close_idx];
                if inside == tag {
                    return Ok(trimmed.to_string());
                } else if inside == tag_without_v {
                    near_miss_heading = Some(trimmed.to_string());
                }
            }
        }

        // Unbracketed heading: e.g. ## v4.250.0 or ## v4.250.0 - 2026-10-23
        let heading_content = trimmed.trim_start_matches('#').trim();
        let first_token = heading_content.split_whitespace().next().unwrap_or("");
        if first_token == tag {
            return Ok(trimmed.to_string());
        } else if first_token == tag_without_v {
            near_miss_heading = Some(trimmed.to_string());
        }
    }

    if let Some(actual) = near_miss_heading {
        Err(BranchReleaseViolation::ChangelogHeadingCharacterMismatch {
            tag: tag.to_string(),
            expected_heading: format!("## [{tag}]"),
            actual_heading: actual,
        })
    } else {
        Err(BranchReleaseViolation::ChangelogHeadingNotFound {
            tag: tag.to_string(),
            expected_heading: format!("## [{tag}]"),
        })
    }
}

/// Evaluates tag and branch compliance according to release guidelines.
pub fn validate_branch_release_channel(
    tag: &str,
    branch: &str,
    changelog_content: Option<&str>,
) -> ReleaseChannelValidationReport {
    let tag = tag.trim();
    let branch_str = branch.trim();
    let mut violations = Vec::new();

    let channel_result = parse_release_channel(tag);
    let channel = match &channel_result {
        Ok(c) => *c,
        Err(v) => {
            violations.push(v.clone());
            if tag.contains("-beta") {
                BranchReleaseChannel::Beta
            } else {
                BranchReleaseChannel::Stable
            }
        }
    };

    let normalized_branch = normalize_git_branch(branch_str);

    match channel {
        BranchReleaseChannel::Stable => {
            if !normalized_branch.is_main() {
                violations.push(BranchReleaseViolation::BranchChannelMismatch {
                    tag: tag.to_string(),
                    current_branch: branch_str.to_string(),
                    channel,
                    expected_branch: "main".to_string(),
                    guidance: "Stable releases ('vX.Y.Z') are allowed exclusively on 'main' branch, reserving 'beta' exclusively for pre-releases.".to_string(),
                });
            }
        }
        BranchReleaseChannel::Beta => {
            if !normalized_branch.is_beta() {
                violations.push(BranchReleaseViolation::BranchChannelMismatch {
                    tag: tag.to_string(),
                    current_branch: branch_str.to_string(),
                    channel,
                    expected_branch: "beta".to_string(),
                    guidance: "Pre-releases ('vX.Y.Z-beta.N') are allowed exclusively on 'beta' branch, reserving 'main' exclusively for stable releases.".to_string(),
                });
            }
        }
    }

    let mut matched_heading = None;
    if let Some(changelog) = changelog_content {
        match find_changelog_heading_exact(changelog, tag) {
            Ok(heading) => {
                matched_heading = Some(heading);
            }
            Err(v) => {
                violations.push(v);
            }
        }
    }

    let is_valid = violations.is_empty();

    ReleaseChannelValidationReport {
        is_valid,
        tag: tag.to_string(),
        current_branch: branch_str.to_string(),
        channel,
        matched_heading,
        violations,
    }
}

/// Validates branch release channel compliance against an on-disk CHANGELOG file.
pub fn validate_branch_release_channel_file(
    tag: &str,
    branch: &str,
    changelog_path: impl AsRef<Path>,
) -> Result<ReleaseChannelValidationReport, std::io::Error> {
    let content = fs::read_to_string(changelog_path.as_ref())?;
    Ok(validate_branch_release_channel(tag, branch, Some(&content)))
}
