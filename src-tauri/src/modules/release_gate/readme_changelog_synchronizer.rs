//! README Changelog Synchronization Verifier.
//!
//! Enforces release documentation discipline per AGENTS.md:
//! - For stable releases, you MUST update the release summary in both `README.md`
//!   (under '## 📝 更新日志') and `README_EN.md` (under '## 📝 Changelog').
//! - Pre-release / beta versions remain exclusively in changelogs; stable releases
//!   require full synchronization across both README files.

use serde::{Deserialize, Serialize};

/// Type of release tag being validated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseTagType {
    Stable(String),
    PreReleaseBeta(String),
}

impl ReleaseTagType {
    pub fn tag(&self) -> &str {
        match self {
            Self::Stable(t) | Self::PreReleaseBeta(t) => t.as_str(),
        }
    }

    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable(_))
    }

    pub fn is_pre_release(&self) -> bool {
        matches!(self, Self::PreReleaseBeta(_))
    }
}

/// Structural violations discovered during README changelog synchronization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadmeSyncViolation {
    StableReleaseMissingReadmeSummary { tag: String, missing_file: String },
    PreReleaseImproperlyPlacedInReadme { tag: String, found_in_file: String },
}

impl std::fmt::Display for ReadmeSyncViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StableReleaseMissingReadmeSummary { tag, missing_file } => {
                write!(
                    f,
                    "Stable release {tag} is missing required release summary in {missing_file}"
                )
            }
            Self::PreReleaseImproperlyPlacedInReadme { tag, found_in_file } => {
                write!(
                    f,
                    "Pre-release {tag} should not be included in {found_in_file} (exclusive to changelog)"
                )
            }
        }
    }
}

impl std::error::Error for ReadmeSyncViolation {}

/// Report containing verification outcome and any discovered violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadmeSyncReport {
    pub is_synchronized: bool,
    pub tag_type: ReleaseTagType,
    pub violations: Vec<ReadmeSyncViolation>,
}

/// Identifies whether a tag represents a pre-release or stable release.
pub fn parse_release_tag_type(tag: &str) -> ReleaseTagType {
    let is_pre_release = tag.contains("-beta.")
        || tag.contains("-alpha.")
        || tag.contains("-rc.")
        || tag.contains("-beta")
        || tag.contains("-alpha")
        || tag.contains("-rc");

    if is_pre_release {
        ReleaseTagType::PreReleaseBeta(tag.to_string())
    } else {
        ReleaseTagType::Stable(tag.to_string())
    }
}

/// Checks whether content contains the version tag or its stripped counterpart.
fn content_contains_tag(content: &str, tag: &str) -> bool {
    let clean_version = tag.trim_start_matches('v');
    content.contains(tag) || content.contains(clean_version)
}

/// Audits stable release synchronization across README files.
fn verify_stable_readme_sync(
    tag: &str,
    readme_zh_content: &str,
    readme_en_content: &str,
    violations: &mut Vec<ReadmeSyncViolation>,
) {
    if !content_contains_tag(readme_zh_content, tag) {
        violations.push(ReadmeSyncViolation::StableReleaseMissingReadmeSummary {
            tag: tag.to_string(),
            missing_file: "README.md".to_string(),
        });
    }

    if !content_contains_tag(readme_en_content, tag) {
        violations.push(ReadmeSyncViolation::StableReleaseMissingReadmeSummary {
            tag: tag.to_string(),
            missing_file: "README_EN.md".to_string(),
        });
    }
}

/// Audits pre-release exclusivity to prevent leakage into README files.
fn verify_prerelease_readme_sync(
    tag: &str,
    readme_zh_content: &str,
    readme_en_content: &str,
    violations: &mut Vec<ReadmeSyncViolation>,
) {
    if content_contains_tag(readme_zh_content, tag) {
        violations.push(ReadmeSyncViolation::PreReleaseImproperlyPlacedInReadme {
            tag: tag.to_string(),
            found_in_file: "README.md".to_string(),
        });
    }

    if content_contains_tag(readme_en_content, tag) {
        violations.push(ReadmeSyncViolation::PreReleaseImproperlyPlacedInReadme {
            tag: tag.to_string(),
            found_in_file: "README_EN.md".to_string(),
        });
    }
}

/// Verifies synchronization of release notes in README.md and README_EN.md.
pub fn verify_readme_changelog_sync(
    tag: &str,
    readme_zh_content: &str,
    readme_en_content: &str,
) -> ReadmeSyncReport {
    let tag_type = parse_release_tag_type(tag);
    let mut violations = Vec::new();

    match &tag_type {
        ReleaseTagType::Stable(_) => {
            verify_stable_readme_sync(tag, readme_zh_content, readme_en_content, &mut violations);
        }
        ReleaseTagType::PreReleaseBeta(_) => {
            verify_prerelease_readme_sync(
                tag,
                readme_zh_content,
                readme_en_content,
                &mut violations,
            );
        }
    }

    ReadmeSyncReport {
        is_synchronized: violations.is_empty(),
        tag_type,
        violations,
    }
}
