//! Unverified Paths & Cost Disclosure Guard (`AGENTS.md`).
//!
//! Enforces Contributor Respect & Attribution discipline per AGENTS.md:
//! "Disclose costs before merging: highlight affected existing behaviors and
//! unverified paths alongside improvements."
//!
//! Audits pull request bodies to ensure explicit disclosures of affected existing
//! behaviors and unverified execution paths, intercepting placeholder evasions
//! and trivial one-word non-answers on high-risk changesets.

use serde::{Deserialize, Serialize};

/// Disallowed boilerplate evasion phrases across cost disclosure sections.
pub const DISALLOWED_PLACEHOLDERS: &[&str] = &[
    "none",
    "n/a",
    "na",
    "nothing",
    "tbd",
    "todo",
    "no changes",
    "not applicable",
];

/// Category of cost disclosure section evaluated during PR auditing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CostDisclosureSection {
    AffectedExistingBehaviors,
    UnverifiedPaths,
}

impl CostDisclosureSection {
    /// Returns the human-readable canonical display label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AffectedExistingBehaviors => "Affected Existing Behaviors",
            Self::UnverifiedPaths => "Unverified Paths",
        }
    }
}

impl std::fmt::Display for CostDisclosureSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Audit findings generated when evaluating PR cost disclosure sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisclosureFinding {
    MissingRequiredSection {
        section: CostDisclosureSection,
    },
    EmptyOrTrivialDisclosure {
        section: CostDisclosureSection,
        word_count: usize,
        min_required: usize,
    },
    BoilerplatePlaceholderDetected {
        section: CostDisclosureSection,
        matched_phrase: String,
    },
}

impl std::fmt::Display for DisclosureFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredSection { section } => {
                write!(f, "Missing required cost disclosure section: '{}'", section)
            }
            Self::EmptyOrTrivialDisclosure {
                section,
                word_count,
                min_required,
            } => {
                write!(
                    f,
                    "Trivial disclosure in section '{}': {} words (minimum {} required)",
                    section, word_count, min_required
                )
            }
            Self::BoilerplatePlaceholderDetected {
                section,
                matched_phrase,
            } => {
                write!(
                    f,
                    "Boilerplate placeholder '{}' detected in section '{}'",
                    matched_phrase, section
                )
            }
        }
    }
}

impl std::error::Error for DisclosureFinding {}

/// Detailed report on cost disclosure compliance.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CostDisclosureReport {
    pub is_compliant: bool,
    pub affected_behaviors_summary: Option<String>,
    pub unverified_paths_summary: Option<String>,
    pub findings: Vec<DisclosureFinding>,
}

impl CostDisclosureReport {
    /// Returns true if disclosures are fully compliant and findings are clean.
    pub fn is_clean(&self) -> bool {
        self.is_compliant && self.findings.is_empty()
    }

    /// Returns the total count of detected findings.
    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }
}

/// Extracts title string from markdown header line.
fn extract_header_title(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with('#') {
        Some(
            trimmed
                .trim_start_matches('#')
                .trim()
                .trim_matches('*')
                .trim(),
        )
    } else if (trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4)
        || (trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > 4)
    {
        Some(trimmed.trim_matches('*').trim_matches('_').trim())
    } else {
        None
    }
}

/// Classifies markdown header string into CostDisclosureSection.
fn classify_header(header: &str) -> Option<CostDisclosureSection> {
    let lower = header.to_ascii_lowercase();
    let clean = lower.trim_end_matches(':').trim();

    if clean.contains("unverified path")
        || clean.contains("unverified paths")
        || clean.contains("paths not covered")
        || clean == "unverified"
        || clean.starts_with("unverified:")
    {
        Some(CostDisclosureSection::UnverifiedPaths)
    } else if clean.contains("affected existing behavior")
        || clean.contains("affected existing behaviors")
        || clean.contains("affected behavior")
        || clean.contains("affected behaviors")
        || clean.contains("behavior alteration")
        || clean.contains("behavior alterations")
        || clean.contains("behavior changes")
    {
        Some(CostDisclosureSection::AffectedExistingBehaviors)
    } else {
        None
    }
}

/// Detects inline bullet prefixes like `- **Affected Existing Behaviors**: ...`
fn extract_inline_labeled_line(line: &str) -> Option<(CostDisclosureSection, &str)> {
    let clean = line
        .trim()
        .trim_start_matches(|c: char| {
            c == '-' || c == '*' || c == '+' || c.is_ascii_digit() || c == '.'
        })
        .trim();

    let colon_pos = clean.find(':')?;
    let label = clean[..colon_pos]
        .trim()
        .trim_matches('*')
        .trim_matches('_')
        .trim();
    let content = clean[colon_pos + 1..].trim();

    let section = classify_header(label)?;
    Some((section, content))
}

/// Combines non-empty lines into a single whitespace-normalized summary string.
fn summarize_lines(lines: Vec<String>) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let joined = lines.join(" ").trim().to_string();
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

/// Parses markdown body and extracts text for affected behaviors and unverified paths.
fn parse_cost_disclosure_sections(markdown: &str) -> (Option<String>, Option<String>) {
    let mut affected_lines = Vec::new();
    let mut unverified_lines = Vec::new();
    let mut current_section: Option<CostDisclosureSection> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("<!--") {
            continue;
        }

        if let Some((sec, inline_content)) = extract_inline_labeled_line(trimmed) {
            current_section = Some(sec);
            if !inline_content.is_empty() {
                match sec {
                    CostDisclosureSection::AffectedExistingBehaviors => {
                        affected_lines.push(inline_content.to_string());
                    }
                    CostDisclosureSection::UnverifiedPaths => {
                        unverified_lines.push(inline_content.to_string());
                    }
                }
            }
            continue;
        }

        if let Some(header_title) = extract_header_title(trimmed) {
            let lower = header_title.to_ascii_lowercase();
            if lower.contains("cost disclosure") {
                current_section = None;
            } else if let Some(sec) = classify_header(header_title) {
                current_section = Some(sec);
            } else {
                current_section = None;
            }
            continue;
        }

        if let Some(sec) = current_section {
            let clean_line = trimmed
                .trim_start_matches(|c: char| c == '-' || c == '*' || c == '+' || c == '>')
                .trim();
            if !clean_line.is_empty() {
                match sec {
                    CostDisclosureSection::AffectedExistingBehaviors => {
                        affected_lines.push(clean_line.to_string());
                    }
                    CostDisclosureSection::UnverifiedPaths => {
                        unverified_lines.push(clean_line.to_string());
                    }
                }
            }
        }
    }

    let affected_summary = summarize_lines(affected_lines);
    let unverified_summary = summarize_lines(unverified_lines);
    (affected_summary, unverified_summary)
}

/// Checks whether the text matches known boilerplate placeholders.
fn is_boilerplate_placeholder(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    let stripped = lower
        .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
        .trim();

    for &placeholder in DISALLOWED_PLACEHOLDERS {
        if lower == placeholder
            || stripped == placeholder
            || stripped == format!("{}:", placeholder)
        {
            return true;
        }
    }
    false
}

/// Evaluates disclosure quality for a section that is present in the PR description.
fn check_present_section(
    section: CostDisclosureSection,
    trimmed: &str,
    findings: &mut Vec<DisclosureFinding>,
) {
    if is_boilerplate_placeholder(trimmed) {
        findings.push(DisclosureFinding::BoilerplatePlaceholderDetected {
            section,
            matched_phrase: trimmed.to_string(),
        });
        return;
    }

    let word_count = trimmed.split_whitespace().count();
    if word_count < 4 {
        findings.push(DisclosureFinding::EmptyOrTrivialDisclosure {
            section,
            word_count,
            min_required: 4,
        });
    }
}

/// Evaluates a single disclosure section against high-risk and completeness rules.
fn evaluate_section(
    section: CostDisclosureSection,
    content: Option<&String>,
    is_high_risk: bool,
    findings: &mut Vec<DisclosureFinding>,
) {
    match content {
        None if is_high_risk => {
            findings.push(DisclosureFinding::MissingRequiredSection { section });
        }
        None => {}
        Some(text) => check_present_section(section, text.trim(), findings),
    }
}

/// Audits pull request description markdown for mandatory cost disclosures.
pub fn audit_cost_disclosure(
    pr_body_markdown: &str,
    is_high_risk_changeset: bool,
) -> CostDisclosureReport {
    let (affected_behaviors, unverified_paths) = parse_cost_disclosure_sections(pr_body_markdown);

    let mut findings = Vec::new();
    evaluate_section(
        CostDisclosureSection::AffectedExistingBehaviors,
        affected_behaviors.as_ref(),
        is_high_risk_changeset,
        &mut findings,
    );
    evaluate_section(
        CostDisclosureSection::UnverifiedPaths,
        unverified_paths.as_ref(),
        is_high_risk_changeset,
        &mut findings,
    );

    let is_compliant = findings.is_empty();

    CostDisclosureReport {
        is_compliant,
        affected_behaviors_summary: affected_behaviors,
        unverified_paths_summary: unverified_paths,
        findings,
    }
}
