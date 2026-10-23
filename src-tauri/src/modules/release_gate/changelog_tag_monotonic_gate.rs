//! Changelog Tag Monotonic Gate.
//!
//! Enforces release tag monotonicity and changelog heading character-for-character
//! match per AGENTS.md Release Workflow:
//! "Commit, Tag & Push: Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`),
//! reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`).
//! The release gate strictly intercepts cross-branch misplacement. Tags must match `CHANGELOG.md`
//! headings character-for-character (including `v` prefix and pre-release suffix)."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Parsed semantic version tag details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedVersionTag {
    pub raw: String,
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
    pub is_prerelease: bool,
}

/// Violations identified when evaluating release tags against changelogs and branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonotonicTagViolation {
    TagHeadingMismatch {
        tag: String,
        expected_heading: String,
    },
    NonMonotonicVersion {
        candidate: String,
        baseline: String,
    },
    CrossBranchMisplacement {
        tag: String,
        branch: String,
        expected_branch: String,
    },
}

impl fmt::Display for MonotonicTagViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TagHeadingMismatch {
                tag,
                expected_heading,
            } => {
                write!(
                    f,
                    "Tag '{tag}' does not match changelog heading character-for-character. Expected heading: '{expected_heading}'"
                )
            }
            Self::NonMonotonicVersion {
                candidate,
                baseline,
            } => {
                write!(
                    f,
                    "Candidate version '{candidate}' is not monotonically greater than baseline '{baseline}'"
                )
            }
            Self::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                write!(
                    f,
                    "Cross-branch tag misplacement: tag '{tag}' placed on branch '{branch}', expected '{expected_branch}'"
                )
            }
        }
    }
}

impl std::error::Error for MonotonicTagViolation {}

/// Structured validation report assessing tag monotonicity, branch placement, and changelog heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMonotonicValidationReport {
    pub is_valid: bool,
    pub tag: String,
    pub branch: String,
    pub violations: Vec<MonotonicTagViolation>,
}

impl TagMonotonicValidationReport {
    /// Returns true if all checks passed without violations.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Returns true if any validation violations were recorded.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns the total count of recorded violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Parses a release tag string into a `ParsedVersionTag`.
pub fn parse_semver_tag(tag: &str) -> Result<ParsedVersionTag, String> {
    let trimmed = tag.trim();
    if !trimmed.starts_with('v') {
        return Err(format!("Tag '{trimmed}' must start with 'v' prefix"));
    }

    let without_v = &trimmed[1..];
    let (core, prerelease) = split_core_and_prerelease(without_v)?;
    let (major, minor, patch) = parse_core_segments(core)?;
    let is_prerelease = prerelease.is_some();

    Ok(ParsedVersionTag {
        raw: trimmed.to_string(),
        major,
        minor,
        patch,
        prerelease,
        is_prerelease,
    })
}

fn split_core_and_prerelease(version_str: &str) -> Result<(&str, Option<String>), String> {
    match version_str.find('-') {
        Some(idx) => {
            let (core, rest) = version_str.split_at(idx);
            let prerelease = &rest[1..];
            if prerelease.is_empty() {
                return Err("Prerelease identifier cannot be empty".to_string());
            }
            Ok((core, Some(prerelease.to_string())))
        }
        None => Ok((version_str, None)),
    }
}

fn parse_core_segments(core: &str) -> Result<(u64, u64, u64), String> {
    let segments: Vec<&str> = core.split('.').collect();
    if segments.len() != 3 {
        return Err(format!(
            "Expected 3 version segments (X.Y.Z), found {} in '{core}'",
            segments.len()
        ));
    }

    let major = segments[0]
        .parse::<u64>()
        .map_err(|e| format!("Invalid major segment in '{core}': {e}"))?;
    let minor = segments[1]
        .parse::<u64>()
        .map_err(|e| format!("Invalid minor segment in '{core}': {e}"))?;
    let patch = segments[2]
        .parse::<u64>()
        .map_err(|e| format!("Invalid patch segment in '{core}': {e}"))?;

    Ok((major, minor, patch))
}

fn compare_prerelease_identifiers(cand: &str, base: &str) -> std::cmp::Ordering {
    let cand_num = cand.parse::<u64>().ok();
    let base_num = base.parse::<u64>().ok();
    match (cand_num, base_num) {
        (Some(cn), Some(bn)) => cn.cmp(&bn),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => cand.cmp(base),
    }
}

fn compare_prerelease_segments(cand: &str, base: &str) -> std::cmp::Ordering {
    let cand_parts: Vec<&str> = cand.split('.').collect();
    let base_parts: Vec<&str> = base.split('.').collect();
    let min_len = cand_parts.len().min(base_parts.len());

    for i in 0..min_len {
        let ord = compare_prerelease_identifiers(cand_parts[i], base_parts[i]);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    cand_parts.len().cmp(&base_parts.len())
}

/// Compares two parsed SemVer tags. Returns true if candidate > baseline according to SemVer rules.
pub fn compare_monotonic_versions(candidate: &ParsedVersionTag, baseline: &ParsedVersionTag) -> bool {
    let core_cand = (candidate.major, candidate.minor, candidate.patch);
    let core_base = (baseline.major, baseline.minor, baseline.patch);

    if core_cand > core_base {
        return true;
    }
    if core_cand < core_base {
        return false;
    }

    match (&candidate.prerelease, &baseline.prerelease) {
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (None, None) => false,
        (Some(c_pre), Some(b_pre)) => {
            compare_prerelease_segments(c_pre, b_pre) == std::cmp::Ordering::Greater
        }
    }
}

fn normalize_branch_name(branch: &str) -> &str {
    let trimmed = branch.trim();
    let stripped = trimmed.strip_prefix("refs/heads/").unwrap_or(trimmed);
    stripped.strip_prefix("origin/").unwrap_or(stripped)
}

fn changelog_contains_heading(changelog: &str, tag: &str) -> bool {
    for line in changelog.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }

        if let Some(open) = trimmed.find('[') {
            if let Some(close) = trimmed[open + 1..].find(']') {
                let inside = &trimmed[open + 1..open + 1 + close];
                if inside == tag {
                    return true;
                }
            }
        }

        let heading_content = trimmed.trim_start_matches('#').trim_start();
        let first_token = heading_content.split_whitespace().next().unwrap_or("");
        if first_token == tag {
            return true;
        }
    }
    false
}

fn evaluate_branch_placement(
    parsed: Result<&ParsedVersionTag, &String>,
    tag: &str,
    branch: &str,
) -> Option<MonotonicTagViolation> {
    let norm = normalize_branch_name(branch);
    let (is_prerelease, expected) = match parsed {
        Ok(p) => (p.is_prerelease, if p.is_prerelease { "beta" } else { "main" }),
        Err(_) => (tag.contains("-beta"), if tag.contains("-beta") { "beta" } else { "main" }),
    };

    if (is_prerelease && norm != "beta") || (!is_prerelease && norm != "main") {
        Some(MonotonicTagViolation::CrossBranchMisplacement {
            tag: tag.to_string(),
            branch: branch.to_string(),
            expected_branch: expected.to_string(),
        })
    } else {
        None
    }
}

fn evaluate_monotonicity(
    cand_parsed: Result<&ParsedVersionTag, &String>,
    candidate_tag: &str,
    baseline_tag: Option<&str>,
) -> Option<MonotonicTagViolation> {
    let base_str = baseline_tag?;
    let base_parsed = parse_semver_tag(base_str);

    match (cand_parsed, &base_parsed) {
        (Ok(cand), Ok(base)) if compare_monotonic_versions(cand, base) => None,
        _ => Some(MonotonicTagViolation::NonMonotonicVersion {
            candidate: candidate_tag.to_string(),
            baseline: base_str.to_string(),
        }),
    }
}

/// Validates a git tag against release branch discipline, changelog headings, and baseline monotonicity.
pub fn validate_tag_against_changelog_and_branch(
    tag: &str,
    branch: &str,
    changelog_markdown: &str,
    baseline_tag: Option<&str>,
) -> TagMonotonicValidationReport {
    let mut violations = Vec::new();
    let candidate_parsed = parse_semver_tag(tag);

    if let Some(v) = evaluate_branch_placement(candidate_parsed.as_ref(), tag, branch) {
        violations.push(v);
    }

    if let Some(v) = evaluate_monotonicity(candidate_parsed.as_ref(), tag, baseline_tag) {
        violations.push(v);
    }

    if !changelog_contains_heading(changelog_markdown, tag) {
        violations.push(MonotonicTagViolation::TagHeadingMismatch {
            tag: tag.to_string(),
            expected_heading: format!("## [{tag}]"),
        });
    }

    let is_valid = violations.is_empty();
    TagMonotonicValidationReport {
        is_valid,
        tag: tag.to_string(),
        branch: branch.to_string(),
        violations,
    }
}
