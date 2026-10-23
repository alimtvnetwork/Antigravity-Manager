//! README Changelog Synchronization Gate.
//!
//! Enforces release documentation discipline per AGENTS.md:
//! "Synchronize README Changelog (同步首页更新日志): For stable releases, you MUST update the release
//! summary in both `README.md` (under '## 📝 更新日志') and `README_EN.md` (under '## 📝 Changelog').
//! Never update only `CHANGELOG.md` while leaving `README.md` / `README_EN.md` with outdated release notes.
//! Pre-release / beta versions remain exclusively in changelogs; stable releases require full synchronization
//! across both README files."

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Regex matching SemVer versions with optional leading 'v'.
static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"v?(\d+\.\d+\.\d+(-[a-zA-Z0-9\.]+)?)").expect("valid version regex"));

/// Section header for changelog in Chinese README.
pub const CHINESE_CHANGELOG_HEADER: &str = "## 📝 更新日志";

/// Section header for changelog in English README.
pub const ENGLISH_CHANGELOG_HEADER: &str = "## 📝 Changelog";

/// Target release channel for compliance determination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseChannel {
    Stable,
    BetaPreRelease,
}

impl ReleaseChannel {
    /// Returns true if this channel represents a stable production release.
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable)
    }

    /// Returns true if this channel represents a beta pre-release.
    pub fn is_beta_prerelease(&self) -> bool {
        matches!(self, Self::BetaPreRelease)
    }

    /// Infers the release channel from a version or tag string.
    pub fn detect_from_version_or_tag(version: &str) -> Self {
        if version.contains("-beta") || version.contains("-alpha") || version.contains("-rc") {
            Self::BetaPreRelease
        } else {
            Self::Stable
        }
    }
}

/// Violations identified during README changelog synchronization verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadmeSyncViolation {
    MissingChineseReadmeChangelogSection,
    MissingEnglishReadmeChangelogSection,
    MismatchedVersionSummary {
        expected_version: String,
        found_version: Option<String>,
    },
    MissingReleaseSummaryInReadme {
        readme_lang: String,
        expected_version: String,
    },
}

impl std::fmt::Display for ReadmeSyncViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingChineseReadmeChangelogSection => {
                write!(
                    f,
                    "Missing Chinese changelog section '## 📝 更新日志' in README.md"
                )
            }
            Self::MissingEnglishReadmeChangelogSection => {
                write!(
                    f,
                    "Missing English changelog section '## 📝 Changelog' in README_EN.md"
                )
            }
            Self::MismatchedVersionSummary {
                expected_version,
                found_version,
            } => {
                write!(
                    f,
                    "Mismatched version summary: expected '{expected_version}', found '{found_version:?}'"
                )
            }
            Self::MissingReleaseSummaryInReadme {
                readme_lang,
                expected_version,
            } => {
                write!(
                    f,
                    "Missing release summary for '{expected_version}' in {readme_lang} README"
                )
            }
        }
    }
}

impl std::error::Error for ReadmeSyncViolation {}

/// Summary report of README changelog synchronization verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadmeSyncReport {
    pub is_synchronized: bool,
    pub target_version: String,
    pub channel: ReleaseChannel,
    pub violations: Vec<ReadmeSyncViolation>,
    pub is_exempt_from_readme: bool,
}

impl ReadmeSyncReport {
    /// Creates a report initialized for the target version and channel.
    pub fn new(target_version: &str, channel: ReleaseChannel, is_exempt_from_readme: bool) -> Self {
        Self {
            is_synchronized: true,
            target_version: target_version.to_string(),
            channel,
            violations: Vec::new(),
            is_exempt_from_readme,
        }
    }
}

/// Normalizes two version strings and determines if they represent the same version.
pub fn version_matches(v1: &str, v2: &str) -> bool {
    let clean1 = v1.trim().trim_start_matches('v');
    let clean2 = v2.trim().trim_start_matches('v');
    clean1 == clean2
}

/// Checks whether a text slice mentions a specific version, with or without leading 'v'.
pub fn contains_version(text: &str, target_version: &str) -> bool {
    let clean = target_version.trim().trim_start_matches('v');
    let with_v = format!("v{clean}");
    text.contains(&with_v) || text.contains(clean)
}

/// Extracts the first SemVer version substring found in a text slice.
pub fn extract_version(text: &str) -> Option<String> {
    VERSION_REGEX.find(text).map(|m| m.as_str().to_string())
}

/// Extracts the markdown subsection starting at `header` and ending at the next level-2 header.
pub fn extract_section<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    let header_idx = content.find(header)?;
    let after_header = &content[header_idx + header.len()..];
    let end_idx = after_header.find("\n## ").unwrap_or(after_header.len());
    Some(&after_header[..end_idx])
}

/// Validates that changelog summary text is present and consistent with target version.
fn audit_changelog_summary(
    target_version: &str,
    changelog_summary: &str,
    violations: &mut Vec<ReadmeSyncViolation>,
) {
    if changelog_summary.trim().is_empty() {
        violations.push(ReadmeSyncViolation::MismatchedVersionSummary {
            expected_version: target_version.to_string(),
            found_version: None,
        });
        return;
    }

    if let Some(summary_ver) = extract_version(changelog_summary) {
        if !version_matches(&summary_ver, target_version) {
            violations.push(ReadmeSyncViolation::MismatchedVersionSummary {
                expected_version: target_version.to_string(),
                found_version: Some(summary_ver),
            });
        }
    }
}

/// Audits a language-specific README content slice for section and version presence.
fn audit_readme_language_sync(
    readme_text: &str,
    header: &str,
    lang_code: &str,
    target_version: &str,
    violations: &mut Vec<ReadmeSyncViolation>,
) {
    if !readme_text.contains(header) {
        match lang_code {
            "zh" => violations.push(ReadmeSyncViolation::MissingChineseReadmeChangelogSection),
            "en" => violations.push(ReadmeSyncViolation::MissingEnglishReadmeChangelogSection),
            _ => violations.push(ReadmeSyncViolation::MissingReleaseSummaryInReadme {
                readme_lang: lang_code.to_string(),
                expected_version: target_version.to_string(),
            }),
        }
        return;
    }

    let section = extract_section(readme_text, header).unwrap_or("");
    if !contains_version(section, target_version) {
        violations.push(ReadmeSyncViolation::MissingReleaseSummaryInReadme {
            readme_lang: lang_code.to_string(),
            expected_version: target_version.to_string(),
        });
    }
}

/// Verifies synchronization of release notes between changelog and README files.
///
/// Rules:
/// - If `channel == ReleaseChannel::BetaPreRelease`: returns `is_synchronized = true, is_exempt_from_readme = true`.
/// - If `channel == ReleaseChannel::Stable`:
///   - Checks that `readme_zh_text` contains `## 📝 更新日志` and mentions `target_version`.
///   - Checks that `readme_en_text` contains `## 📝 Changelog` and mentions `target_version`.
///   - Checks summaries. If either is missing or outdated, flags violation.
pub fn verify_readme_changelog_synchronization(
    target_version: &str,
    changelog_summary: &str,
    readme_zh_text: &str,
    readme_en_text: &str,
    channel: &ReleaseChannel,
) -> ReadmeSyncReport {
    if *channel == ReleaseChannel::BetaPreRelease {
        return ReadmeSyncReport {
            is_synchronized: true,
            target_version: target_version.to_string(),
            channel: *channel,
            violations: Vec::new(),
            is_exempt_from_readme: true,
        };
    }

    let mut violations = Vec::new();

    audit_changelog_summary(target_version, changelog_summary, &mut violations);
    audit_readme_language_sync(
        readme_zh_text,
        CHINESE_CHANGELOG_HEADER,
        "zh",
        target_version,
        &mut violations,
    );
    audit_readme_language_sync(
        readme_en_text,
        ENGLISH_CHANGELOG_HEADER,
        "en",
        target_version,
        &mut violations,
    );

    let is_synchronized = violations.is_empty();

    ReadmeSyncReport {
        is_synchronized,
        target_version: target_version.to_string(),
        channel: *channel,
        violations,
        is_exempt_from_readme: false,
    }
}
