//! Changelog Tag Monotonic Gate (Imp-662).
//!
//! Enforces release tag monotonicity and changelog heading character-for-character
//! match per AGENTS.md Release Workflow & Commit, Tag & Push Invariant:
//! "Push stable releases to main (git tag vX.Y.Z && git push origin vX.Y.Z),
//! reserving beta exclusively for pre-releases (git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N).
//! The release gate strictly intercepts cross-branch misplacement. Tags must match CHANGELOG.md
//! headings character-for-character (including v prefix and pre-release suffix)."

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

/// Release channel classification following AGENTS.md rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseChannelType {
    Stable,
    Beta,
}

impl ReleaseChannelType {
    /// Returns true if this is a stable release channel.
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable)
    }

    /// Returns true if this is a beta preview channel.
    pub fn is_beta(&self) -> bool {
        matches!(self, Self::Beta)
    }

    /// Expected canonical branch name for this release channel.
    pub fn expected_branch(&self) -> &'static str {
        match self {
            Self::Stable => "main",
            Self::Beta => "beta",
        }
    }
}

impl fmt::Display for ReleaseChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Beta => write!(f, "Beta"),
        }
    }
}

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

impl ParsedVersionTag {
    /// Returns true if this tag represents a pre-release version.
    pub fn is_prerelease(&self) -> bool {
        self.is_prerelease
    }

    /// Returns true if this tag represents a stable release version.
    pub fn is_stable(&self) -> bool {
        !self.is_prerelease
    }

    /// Resolves the corresponding release channel type.
    pub fn release_channel(&self) -> ReleaseChannelType {
        if self.is_prerelease {
            ReleaseChannelType::Beta
        } else {
            ReleaseChannelType::Stable
        }
    }
}

/// Violations identified when validating release tags against changelogs and branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonotonicValidationViolationType {
    MissingVPrefix {
        tag: String,
    },
    HeadingMismatch {
        candidate_tag: String,
        expected_heading: String,
    },
    NonMonotonicRegression {
        candidate_tag: String,
        historical_latest_tag: String,
    },
    CrossBranchMisplacement {
        candidate_tag: String,
        branch: String,
        expected_branch: String,
    },
    InvalidSemVer {
        tag: String,
        details: String,
    },
}

pub type MonotonicTagViolation = MonotonicValidationViolationType;

impl fmt::Display for MonotonicValidationViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVPrefix { tag } => {
                write!(f, "Tag '{tag}' must start with 'v' prefix")
            }
            Self::HeadingMismatch {
                candidate_tag,
                expected_heading,
            } => {
                write!(
                    f,
                    "Tag '{candidate_tag}' does not match changelog heading character-for-character. Expected heading: '{expected_heading}'"
                )
            }
            Self::NonMonotonicRegression {
                candidate_tag,
                historical_latest_tag,
            } => {
                write!(
                    f,
                    "Candidate version '{candidate_tag}' is not monotonically greater than baseline '{historical_latest_tag}'"
                )
            }
            Self::CrossBranchMisplacement {
                candidate_tag,
                branch,
                expected_branch,
            } => {
                write!(
                    f,
                    "Cross-branch tag misplacement: tag '{candidate_tag}' placed on branch '{branch}', expected '{expected_branch}'"
                )
            }
            Self::InvalidSemVer { tag, details } => {
                write!(f, "Invalid SemVer for tag '{tag}': {details}")
            }
        }
    }
}

impl std::error::Error for MonotonicValidationViolationType {}

/// Structured validation report assessing tag monotonicity, branch placement, and changelog heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMonotonicValidationReport {
    pub is_valid: bool,
    pub has_character_match: bool,
    pub is_monotonic: bool,
    pub is_channel_compliant: bool,
    pub candidate_tag: String,
    pub latest_historical_tag: Option<String>,
    pub failure_reasons: Vec<String>,
}

impl TagMonotonicValidationReport {
    /// Returns true if all checks passed without violations.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Returns true if character match passed.
    pub fn has_character_match(&self) -> bool {
        self.has_character_match
    }

    /// Returns true if version progression is strictly monotonic.
    pub fn is_monotonic(&self) -> bool {
        self.is_monotonic
    }

    /// Returns true if release channel placement is compliant.
    pub fn is_channel_compliant(&self) -> bool {
        self.is_channel_compliant
    }

    /// Returns true if any failure reasons were recorded.
    pub fn has_failures(&self) -> bool {
        !self.failure_reasons.is_empty()
    }

    /// Returns the total count of recorded failures.
    pub fn failure_count(&self) -> usize {
        self.failure_reasons.len()
    }

    /// Compatibility helper for existing code inspecting violation status.
    pub fn has_violations(&self) -> bool {
        !self.failure_reasons.is_empty()
    }

    /// Compatibility helper for existing code checking violation count.
    pub fn violation_count(&self) -> usize {
        self.failure_reasons.len()
    }

    /// Returns candidate tag string reference.
    pub fn tag(&self) -> &str {
        &self.candidate_tag
    }
}

/// Gate enforcing changelog heading match and monotonic tag progression.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangelogTagMonotonicGate;

impl ChangelogTagMonotonicGate {
    /// Constructs a new ChangelogTagMonotonicGate instance.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates candidate tag against branch, changelog markdown content, and baseline tag.
    pub fn validate(
        &self,
        candidate_tag: &str,
        branch: &str,
        changelog_content: &str,
        latest_historical_tag: Option<&str>,
    ) -> TagMonotonicValidationReport {
        validate_tag_monotonic_progression(
            candidate_tag,
            branch,
            changelog_content,
            latest_historical_tag,
        )
    }

    /// Evaluates candidate tag against an on-disk CHANGELOG file.
    pub fn validate_file(
        &self,
        candidate_tag: &str,
        branch: &str,
        changelog_path: impl AsRef<Path>,
        latest_historical_tag: Option<&str>,
    ) -> Result<TagMonotonicValidationReport, AppError> {
        let path = changelog_path.as_ref();
        let content = fs::read_to_string(path)?;
        Ok(self.validate(candidate_tag, branch, &content, latest_historical_tag))
    }

    /// Static convenience method for validating progression.
    pub fn evaluate(
        candidate_tag: &str,
        branch: &str,
        changelog_content: &str,
        latest_historical_tag: Option<&str>,
    ) -> TagMonotonicValidationReport {
        Self::new().validate(
            candidate_tag,
            branch,
            changelog_content,
            latest_historical_tag,
        )
    }
}

/// Validates leading 'v' prefix on release tag.
fn validate_tag_prefix(tag: &str) -> Result<&str, String> {
    let trimmed = tag.trim();
    if !trimmed.starts_with('v') {
        return Err(format!("Tag '{trimmed}' must start with 'v' prefix"));
    }
    Ok(&trimmed[1..])
}

/// Splits version core from pre-release suffix (if present).
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

/// Parses a single core segment number (major, minor, or patch).
fn parse_core_segment_number(segment: &str, segment_name: &str) -> Result<u64, String> {
    segment
        .parse::<u64>()
        .map_err(|e| format!("Invalid {segment_name} segment in version: {e}"))
}

/// Parses core numeric triplet (X.Y.Z).
fn parse_core_segments(core: &str) -> Result<(u64, u64, u64), String> {
    let segments: Vec<&str> = core.split('.').collect();
    if segments.len() != 3 {
        return Err(format!(
            "Expected 3 version segments (X.Y.Z), found {} in '{core}'",
            segments.len()
        ));
    }
    let major = parse_core_segment_number(segments[0], "major")?;
    let minor = parse_core_segment_number(segments[1], "minor")?;
    let patch = parse_core_segment_number(segments[2], "patch")?;
    Ok((major, minor, patch))
}

/// Parses a release tag string into a `ParsedVersionTag`.
pub fn parse_semver_tag(tag: &str) -> Result<ParsedVersionTag, String> {
    let raw = tag.trim().to_string();
    let without_v = validate_tag_prefix(tag)?;
    let (core, prerelease) = split_core_and_prerelease(without_v)?;
    let (major, minor, patch) = parse_core_segments(core)?;
    let is_prerelease = prerelease.is_some();

    Ok(ParsedVersionTag {
        raw,
        major,
        minor,
        patch,
        prerelease,
        is_prerelease,
    })
}

/// Compares two individual pre-release identifier parts (numeric vs string).
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

/// Compares dot-delimited pre-release identifier segments according to SemVer 2.0.0.
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

/// Compares pre-release precedence when core triplets are equal.
fn compare_prerelease_precedence(cand: &Option<String>, base: &Option<String>) -> bool {
    match (cand, base) {
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (None, None) => false,
        (Some(c_pre), Some(b_pre)) => {
            compare_prerelease_segments(c_pre, b_pre) == std::cmp::Ordering::Greater
        }
    }
}

/// Compares two parsed SemVer tags. Returns true if candidate > baseline according to SemVer rules.
pub fn compare_monotonic_versions(
    candidate: &ParsedVersionTag,
    baseline: &ParsedVersionTag,
) -> bool {
    let cand_core = (candidate.major, candidate.minor, candidate.patch);
    let base_core = (baseline.major, baseline.minor, baseline.patch);

    if cand_core > base_core {
        return true;
    }
    if cand_core < base_core {
        return false;
    }

    compare_prerelease_precedence(&candidate.prerelease, &baseline.prerelease)
}

/// Normalizes git branch name by stripping remote prefixes and ref heads.
pub fn normalize_branch_name(branch: &str) -> &str {
    let trimmed = branch.trim();
    let stripped = trimmed.strip_prefix("refs/heads/").unwrap_or(trimmed);
    let stripped = stripped.strip_prefix("refs/tags/").unwrap_or(stripped);
    stripped.strip_prefix("origin/").unwrap_or(stripped)
}

/// Resolves release channel type for candidate tag.
fn resolve_candidate_channel(
    parsed: Result<&ParsedVersionTag, &String>,
    raw_tag: &str,
) -> ReleaseChannelType {
    match parsed {
        Ok(p) => p.release_channel(),
        Err(_) => {
            if raw_tag.contains("-beta") {
                ReleaseChannelType::Beta
            } else {
                ReleaseChannelType::Stable
            }
        }
    }
}

/// Evaluates branch placement compliance against channel designation.
fn evaluate_branch_compliance(
    channel: ReleaseChannelType,
    branch: &str,
    candidate_tag: &str,
) -> Result<(), String> {
    let norm = normalize_branch_name(branch);
    let expected = channel.expected_branch();
    if norm != expected {
        Err(format!(
            "Cross-branch tag misplacement: tag '{candidate_tag}' placed on branch '{branch}', expected '{expected}'"
        ))
    } else {
        Ok(())
    }
}

/// Extracts inner text from bracketed heading (e.g. `## [v4.8.2]`).
fn extract_bracketed_heading(line: &str) -> Option<&str> {
    let open_idx = line.find('[')?;
    let close_idx = line[open_idx + 1..].find(']')?;
    Some(&line[open_idx + 1..open_idx + 1 + close_idx])
}

/// Extracts first token from unbracketed heading (e.g. `## v4.8.2 - 2026-10-23`).
fn extract_unbracketed_heading(line: &str) -> Option<&str> {
    let heading_body = line.trim_start_matches('#').trim_start();
    heading_body.split_whitespace().next()
}

/// Checks whether a single changelog line matches candidate tag character-for-character.
fn changelog_line_matches_tag(line: &str, tag: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return false;
    }
    if let Some(bracketed) = extract_bracketed_heading(trimmed) {
        if bracketed == tag {
            return true;
        }
    }
    if let Some(unbracketed) = extract_unbracketed_heading(trimmed) {
        if unbracketed == tag {
            return true;
        }
    }
    false
}

/// Scans changelog markdown for character-for-character matching heading.
pub fn changelog_contains_exact_heading(changelog: &str, tag: &str) -> bool {
    changelog
        .lines()
        .any(|line| changelog_line_matches_tag(line, tag))
}

/// Evaluates whether candidate tag progresses monotonically over baseline tag.
fn evaluate_monotonic_progression(
    cand_parsed: Result<&ParsedVersionTag, &String>,
    candidate_tag: &str,
    latest_historical_tag: Option<&str>,
) -> Result<(), String> {
    let historical_str = match latest_historical_tag {
        Some(h) if !h.trim().is_empty() => h.trim(),
        _ => return Ok(()),
    };

    let cand = cand_parsed.map_err(|e| format!("Candidate tag '{candidate_tag}' invalid: {e}"))?;
    let hist = parse_semver_tag(historical_str)
        .map_err(|e| format!("Historical tag '{historical_str}' invalid: {e}"))?;

    if compare_monotonic_versions(cand, &hist) {
        Ok(())
    } else {
        Err(format!(
            "Candidate version '{candidate_tag}' is not monotonically greater than baseline '{historical_str}'"
        ))
    }
}

/// Validates tag monotonicity, branch discipline, and changelog heading character match.
pub fn validate_tag_monotonic_progression(
    candidate_tag: &str,
    branch: &str,
    changelog_markdown: &str,
    latest_historical_tag: Option<&str>,
) -> TagMonotonicValidationReport {
    let mut failure_reasons = Vec::new();
    let cand_trimmed = candidate_tag.trim();
    let cand_parsed = parse_semver_tag(cand_trimmed);

    if let Err(ref e) = cand_parsed {
        failure_reasons.push(format!("Tag format violation: {e}"));
    }

    let has_character_match = changelog_contains_exact_heading(changelog_markdown, cand_trimmed);
    if !has_character_match {
        failure_reasons.push(format!(
            "Tag '{cand_trimmed}' does not match changelog heading character-for-character. Expected heading: '## [{cand_trimmed}]'"
        ));
    }

    let channel = resolve_candidate_channel(cand_parsed.as_ref(), cand_trimmed);
    let channel_eval = evaluate_branch_compliance(channel, branch, cand_trimmed);
    let is_channel_compliant = channel_eval.is_ok();
    if let Err(msg) = channel_eval {
        failure_reasons.push(msg);
    }

    let mono_eval =
        evaluate_monotonic_progression(cand_parsed.as_ref(), cand_trimmed, latest_historical_tag);
    let is_monotonic = mono_eval.is_ok();
    if let Err(msg) = mono_eval {
        failure_reasons.push(msg);
    }

    let is_valid = failure_reasons.is_empty();

    TagMonotonicValidationReport {
        is_valid,
        has_character_match,
        is_monotonic,
        is_channel_compliant,
        candidate_tag: cand_trimmed.to_string(),
        latest_historical_tag: latest_historical_tag.map(|s| s.trim().to_string()),
        failure_reasons,
    }
}

/// Backward compatibility alias for validate_tag_monotonic_progression.
pub fn validate_tag_against_changelog_and_branch(
    tag: &str,
    branch: &str,
    changelog_markdown: &str,
    baseline_tag: Option<&str>,
) -> TagMonotonicValidationReport {
    validate_tag_monotonic_progression(tag, branch, changelog_markdown, baseline_tag)
}
