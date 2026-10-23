//! Contributor Cost Disclosure & Attribution Linter.
//!
//! Enforces contributor attribution and cost disclosure discipline per AGENTS.md:
//! - "Contributor Respect & Attribution: Preserve authorship by preferring the contributor's
//!   own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge
//!   commits and proxy PRs."
//! - "Disclose costs before merging: highlight affected existing behaviors and unverified
//!   paths alongside improvements."
//! - "Own tooling and documentation gaps directly rather than attributing downstream frictions
//!   to contributors."
//! - "Accompany every rejection with a concrete file-by-file accept/drop breakdown and an
//!   actionable path forward."
//!
//! Provides:
//! 1. PR body linting for mandatory Cost Disclosure (affected behaviors, unverified paths, improvements).
//! 2. Git `Co-authored-by:` trailer validation and proxy PR attribution enforcement.
//! 3. Structured file-by-file Accept/Drop breakdown formatting and rejection response compliance linting.

use serde::{Deserialize, Serialize};
use super::coauthor_attribution_verifier::{parse_coauthor_trailers, CoAuthor};

pub use super::coauthor_attribution_verifier::CoAuthor;

/// File disposition status in a PR review breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDisposition {
    Accept,
    Drop,
    NeedsRevision,
}

impl std::fmt::Display for FileDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "Accept"),
            Self::Drop => write!(f, "Drop"),
            Self::NeedsRevision => write!(f, "Needs Revision"),
        }
    }
}

/// A file-by-file review decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDecision {
    pub file_path: String,
    pub disposition: FileDisposition,
    pub rationale: String,
}

/// Structured breakdown accompanying PR rejection or partial merge feedback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionBreakdown {
    pub contributor_handle: String,
    pub pr_reference: String,
    pub file_decisions: Vec<FileDecision>,
    pub actionable_path_forward: String,
    pub owned_tooling_or_doc_gaps: Vec<String>,
}

/// Report evaluating whether a rejection comment satisfies AGENTS.md standards.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RejectionReviewReport {
    pub is_compliant: bool,
    pub has_file_breakdown: bool,
    pub has_actionable_path_forward: bool,
    pub has_owned_tooling_gaps: bool,
    pub missing_requirements: Vec<String>,
}

/// Report evaluating PR body compliance with Cost Disclosure and Attribution requirements.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContributorPrComplianceReport {
    pub is_merge_ready: bool,
    pub has_cost_disclosure: bool,
    pub has_affected_behaviors: bool,
    pub has_unverified_paths: bool,
    pub has_improvements: bool,
    pub has_valid_coauthors: bool,
    pub coauthors: Vec<CoAuthor>,
    pub proxy_missing_coauthor: bool,
    pub missing_sections: Vec<String>,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Formats a standardized file-by-file rejection or review breakdown per AGENTS.md.
pub fn format_rejection_breakdown(breakdown: &RejectionBreakdown) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "### Contributor Review Feedback for PR {}\n\n",
        breakdown.pr_reference.trim()
    ));
    out.push_str(&format!(
        "Thank you {} for your contribution to Antigravity Manager!\n\n",
        breakdown.contributor_handle.trim()
    ));

    out.push_str("#### File-by-File Disposition Breakdown\n\n");
    out.push_str("| File | Disposition | Rationale |\n");
    out.push_str("| :--- | :--- | :--- |\n");

    for file in &breakdown.file_decisions {
        let disp_str = match file.disposition {
            FileDisposition::Accept => "**Accept**",
            FileDisposition::Drop => "**Drop**",
            FileDisposition::NeedsRevision => "**Needs Revision**",
        };
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            file.file_path.trim().replace('\\', "/"),
            disp_str,
            file.rationale.trim()
        ));
    }
    out.push('\n');

    if !breakdown.owned_tooling_or_doc_gaps.is_empty() {
        out.push_str("#### Tooling & Documentation Gaps (Maintainer Owned)\n\n");
        out.push_str(
            "We take direct ownership of the following tooling and documentation gaps that contributed to friction in this PR:\n",
        );
        for gap in &breakdown.owned_tooling_or_doc_gaps {
            out.push_str(&format!("- {}\n", gap.trim()));
        }
        out.push('\n');
    }

    out.push_str("#### Actionable Path Forward\n\n");
    out.push_str(breakdown.actionable_path_forward.trim());
    out.push('\n');

    out
}

/// Lints a rejection comment to ensure it fulfills AGENTS.md contributor respect standards:
/// file-by-file breakdown, actionable path forward, and maintainer-owned tooling/doc gaps.
pub fn lint_rejection_response(markdown: &str) -> RejectionReviewReport {
    let lower = markdown.to_ascii_lowercase();

    let has_file_breakdown = (lower.contains("file-by-file") || lower.contains("file breakdown") || lower.contains("| file |"))
        && (lower.contains("accept") || lower.contains("drop"));

    let has_actionable_path_forward = lower.contains("actionable path forward")
        || lower.contains("path forward")
        || lower.contains("next steps");

    let has_owned_tooling_gaps = lower.contains("tooling")
        && (lower.contains("gap") || lower.contains("friction") || lower.contains("ownership") || lower.contains("maintainer owned"));

    let mut missing_requirements = Vec::new();
    if !has_file_breakdown {
        missing_requirements.push("File-by-file accept/drop breakdown".to_string());
    }
    if !has_actionable_path_forward {
        missing_requirements.push("Actionable path forward".to_string());
    }
    if !has_owned_tooling_gaps {
        missing_requirements.push("Maintainer-owned tooling or documentation gaps".to_string());
    }

    let is_compliant = missing_requirements.is_empty();

    RejectionReviewReport {
        is_compliant,
        has_file_breakdown,
        has_actionable_path_forward,
        has_owned_tooling_gaps,
        missing_requirements,
    }
}

/// Detects whether PR context indicates a proxy PR submitted on behalf of another contributor.
pub fn detect_is_proxy_pr(pr_body: &str) -> bool {
    let lower = pr_body.to_ascii_lowercase();
    lower.contains("proxy pr")
        || lower.contains("proxy for @")
        || lower.contains("proxy submission")
        || lower.contains("on behalf of @")
        || lower.contains("on behalf of contributor")
        || lower.contains("co-authored pr")
}

/// Evaluates PR body markdown for Cost Disclosure and Attribution compliance per AGENTS.md.
pub fn lint_contributor_pr(pr_body: &str, is_proxy_pr: bool) -> ContributorPrComplianceReport {
    let lower = pr_body.to_ascii_lowercase();
    let mut missing_sections = Vec::new();
    let mut violations = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Cost Disclosure Top-Level Section Check
    let has_cost_disclosure = lower.contains("cost disclosure")
        || lower.contains("## cost")
        || lower.contains("### cost")
        || lower.contains("costs & trade-offs")
        || lower.contains("costs and trade-offs");

    if !has_cost_disclosure {
        missing_sections.push("Cost Disclosure".to_string());
        violations.push("Missing required Cost Disclosure section in PR description".to_string());
        recommendations.push("Add a '## Cost Disclosure' section detailing affected behaviors and unverified paths".to_string());
    }

    // 2. Affected Existing Behaviors Check
    let affected_content = extract_section_content(
        pr_body,
        &["affected existing behaviors", "affected behaviors", "existing behaviors", "behavior alterations"],
    );
    let has_affected_behaviors = match affected_content {
        Some(ref content) if is_substantive(content) => true,
        Some(_) => {
            violations.push("Section 'Affected Existing Behaviors' contains trivial placeholder instead of substantive disclosure".to_string());
            recommendations.push("Provide concrete descriptions of affected behaviors, altered defaults, or performance characteristics".to_string());
            false
        }
        None => {
            missing_sections.push("Affected Existing Behaviors".to_string());
            violations.push("Missing 'Affected Existing Behaviors' disclosure".to_string());
            recommendations.push("Specify which existing behaviors or defaults change with this PR".to_string());
            false
        }
    };

    // 3. Unverified Paths Check
    let unverified_content = extract_section_content(
        pr_body,
        &["unverified paths", "unverified", "paths not covered", "untested edge cases"],
    );
    let has_unverified_paths = match unverified_content {
        Some(ref content) if is_substantive(content) => true,
        Some(_) => {
            violations.push("Section 'Unverified Paths' contains trivial placeholder instead of substantive disclosure".to_string());
            recommendations.push("Detail environments, OS variants, or edge conditions not verified in local testing".to_string());
            false
        }
        None => {
            missing_sections.push("Unverified Paths".to_string());
            violations.push("Missing 'Unverified Paths' disclosure".to_string());
            recommendations.push("List paths, high-concurrency scenarios, or edge cases that remain unverified".to_string());
            false
        }
    };

    // 4. Improvements Check
    let improvements_content = extract_section_content(
        pr_body,
        &["improvements", "key improvements", "benefits", "enhancements", "problem classification"],
    );
    let has_improvements = match improvements_content {
        Some(ref content) if is_substantive(content) => true,
        Some(_) => {
            violations.push("Section 'Improvements' contains trivial placeholder".to_string());
            false
        }
        None => {
            missing_sections.push("Improvements".to_string());
            violations.push("Missing 'Improvements' disclosure".to_string());
            false
        }
    };

    // 5. Co-authored-by Trailer Validation
    let trailer_report = parse_coauthor_trailers(pr_body);
    let has_valid_coauthors = trailer_report.has_coauthors;
    let coauthors = trailer_report.coauthors;

    for inv in &trailer_report.invalid_trailers {
        violations.push(format!("Malformed Co-authored-by trailer: '{inv}'"));
        recommendations.push("Format trailers as: Co-authored-by: Name <email@example.com>".to_string());
    }

    let is_proxy = is_proxy_pr || detect_is_proxy_pr(pr_body);
    let mut proxy_missing_coauthor = false;

    if is_proxy && coauthors.is_empty() {
        proxy_missing_coauthor = true;
        violations.push(
            "Proxy PR submitted on behalf of contributor must attach at least one valid 'Co-authored-by: Name <email>' trailer to preserve authorship per AGENTS.md"
                .to_string(),
        );
        recommendations.push(
            "Add 'Co-authored-by: Contributor Name <contributor@example.com>' trailer to PR description"
                .to_string(),
        );
    }

    let is_merge_ready = missing_sections.is_empty()
        && violations.is_empty()
        && has_cost_disclosure
        && has_affected_behaviors
        && has_unverified_paths
        && has_improvements
        && !proxy_missing_coauthor;

    ContributorPrComplianceReport {
        is_merge_ready,
        has_cost_disclosure,
        has_affected_behaviors,
        has_unverified_paths,
        has_improvements,
        has_valid_coauthors,
        coauthors,
        proxy_missing_coauthor,
        missing_sections,
        violations,
        recommendations,
    }
}

/// Extracts the body content beneath specific markdown section header variants.
fn extract_section_content(markdown: &str, header_variants: &[&str]) -> Option<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let lower_lines: Vec<String> = lines.iter().map(|l| l.to_ascii_lowercase()).collect();

    for (idx, line) in lower_lines.iter().enumerate() {
        for &variant in header_variants {
            let trimmed = line.trim();
            let is_header = trimmed.starts_with('#') || trimmed.starts_with("**");
            if is_header && trimmed.contains(variant) {
                let mut collected = Vec::new();
                for next_line in &lines[idx + 1..] {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with('#') {
                        break;
                    }
                    if !next_trimmed.is_empty() {
                        collected.push(next_trimmed);
                    }
                }
                let content = collected.join(" ").trim().to_string();
                if !content.is_empty() {
                    return Some(content);
                }
            }
        }
    }
    None
}

/// Checks whether extracted section content is substantive (not empty or trivial placeholder).
fn is_substantive(content: &str) -> bool {
    let trimmed = content.trim().to_ascii_lowercase();
    if trimmed.len() < 10 {
        return false;
    }
    let trivial_placeholders = [
        "n/a", "none", "no", "nil", "todo", "tbd", "nothing", "-", "not applicable",
        "none so far", "n / a",
    ];
    if trivial_placeholders
        .iter()
        .any(|&p| trimmed == p || trimmed == format!("{p}."))
    {
        return false;
    }
    true
}
