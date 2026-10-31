//! Pull Request Template Compliance Guard (`Imp-203`).
//!
//! Enforces PR template alignment invariant per AGENTS.md PR Scope & Grouping:
//! "Review & Template Alignment: Route every PR through peer review and complete
//! `.github/pull_request_template.md` (problem classification, behavior alterations,
//! unverified paths, and rollback strategy)."
//!
//! Validates PR descriptions to ensure all four mandatory sections are present
//! and contain non-trivial content (>= 3 words).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimum word count required for a PR template section to be considered complete.
pub const MINIMUM_SECTION_WORDS: usize = 3;

/// The four mandatory sections required in pull request descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrTemplateSection {
    ProblemClassification,
    BehaviorAlterations,
    UnverifiedPaths,
    RollbackStrategy,
}

impl PrTemplateSection {
    /// List of all four mandatory PR template sections in standard evaluation order.
    pub const ALL: [PrTemplateSection; 4] = [
        PrTemplateSection::ProblemClassification,
        PrTemplateSection::BehaviorAlterations,
        PrTemplateSection::UnverifiedPaths,
        PrTemplateSection::RollbackStrategy,
    ];

    /// Returns the human-readable canonical title of the section.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProblemClassification => "Problem Classification",
            Self::BehaviorAlterations => "Behavior Alterations",
            Self::UnverifiedPaths => "Unverified Paths",
            Self::RollbackStrategy => "Rollback Strategy",
        }
    }

    /// Alias for canonical title.
    pub fn title(&self) -> &'static str {
        self.as_str()
    }
}

impl std::fmt::Display for PrTemplateSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Audit findings generated during PR template compliance inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateAuditFinding {
    MissingSection {
        section: PrTemplateSection,
    },
    IncompleteSection {
        section: PrTemplateSection,
        minimum_words: usize,
        found_words: usize,
    },
}

impl std::fmt::Display for TemplateAuditFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSection { section } => {
                write!(f, "Missing mandatory PR template section: {}", section)
            }
            Self::IncompleteSection {
                section,
                minimum_words,
                found_words,
            } => {
                write!(
                    f,
                    "Incomplete PR template section: {} (found {} words, minimum {} required)",
                    section, found_words, minimum_words
                )
            }
        }
    }
}

impl std::error::Error for TemplateAuditFinding {}

/// Detailed compliance report from auditing a PR description markdown.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PrTemplateAuditReport {
    pub is_compliant: bool,
    pub problem_classification: Option<String>,
    pub behavior_alterations: Option<String>,
    pub unverified_paths: Option<String>,
    pub rollback_strategy: Option<String>,
    pub findings: Vec<TemplateAuditFinding>,
}

impl PrTemplateAuditReport {
    /// Returns true if the PR description complies with all template requirements.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Returns true if any compliance findings were flagged.
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Returns a list of sections marked as completely missing.
    pub fn missing_sections(&self) -> Vec<PrTemplateSection> {
        self.findings
            .iter()
            .filter_map(|f| match f {
                TemplateAuditFinding::MissingSection { section } => Some(*section),
                _ => None,
            })
            .collect()
    }

    /// Returns a list of sections marked as incomplete (too few words).
    pub fn incomplete_sections(&self) -> Vec<PrTemplateSection> {
        self.findings
            .iter()
            .filter_map(|f| match f {
                TemplateAuditFinding::IncompleteSection { section, .. } => Some(*section),
                _ => None,
            })
            .collect()
    }
}

type EvaluatedSections = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// Evaluates all 4 template sections against parsed content.
fn evaluate_all_sections(
    sections: &HashMap<PrTemplateSection, String>,
    findings: &mut Vec<TemplateAuditFinding>,
) -> EvaluatedSections {
    (
        evaluate_section(PrTemplateSection::ProblemClassification, sections, findings),
        evaluate_section(PrTemplateSection::BehaviorAlterations, sections, findings),
        evaluate_section(PrTemplateSection::UnverifiedPaths, sections, findings),
        evaluate_section(PrTemplateSection::RollbackStrategy, sections, findings),
    )
}

/// Audits a pull request description against the mandatory AGENTS.md template sections.
pub fn audit_pr_description(description_markdown: &str) -> PrTemplateAuditReport {
    let sections = parse_markdown_sections(description_markdown);
    let mut findings = Vec::new();
    let (prob, beh, unv, roll) = evaluate_all_sections(&sections, &mut findings);
    let is_compliant = findings.is_empty();

    PrTemplateAuditReport {
        is_compliant,
        problem_classification: prob,
        behavior_alterations: beh,
        unverified_paths: unv,
        rollback_strategy: roll,
        findings,
    }
}

/// Parses markdown text into a map of detected template sections and their content.
fn parse_markdown_sections(markdown: &str) -> HashMap<PrTemplateSection, String> {
    let mut sections: HashMap<PrTemplateSection, String> = HashMap::new();
    let mut current_section: Option<PrTemplateSection> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(header_title) = extract_header_title(trimmed) {
            current_section = classify_header_title(header_title);
            if let Some(sec) = current_section {
                sections.entry(sec).or_default();
            }
        } else if let Some(sec) = current_section {
            append_section_line(&mut sections, sec, line);
        }
    }

    sections
}

/// Extracts the clean title from a markdown header or bold title line.
fn extract_header_title(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with('#') {
        Some(
            trimmed
                .trim_start_matches('#')
                .trim()
                .trim_matches('*')
                .trim(),
        )
    } else if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
        Some(trimmed.trim_matches('*').trim())
    } else {
        None
    }
}

/// Classifies a header title into its corresponding PrTemplateSection.
fn classify_header_title(header_title: &str) -> Option<PrTemplateSection> {
    let lower = header_title.to_ascii_lowercase();
    if lower.contains("problem classification")
        || lower == "problem"
        || lower.starts_with("problem:")
        || lower.contains("Current issues")
    {
        Some(PrTemplateSection::ProblemClassification)
    } else if lower.contains("behavior alteration")
        || lower.contains("behavior alterations")
        || lower.contains("behavior changes")
        || lower == "behavior"
        || lower.starts_with("behavior:")
        || lower.contains("changes caused")
    {
        Some(PrTemplateSection::BehaviorAlterations)
    } else if lower.contains("unverified path")
        || lower.contains("unverified paths")
        || lower == "unverified"
        || lower.starts_with("unverified:")
        || lower.contains("paths not covered")
    {
        Some(PrTemplateSection::UnverifiedPaths)
    } else if lower.contains("rollback strategy")
        || lower.contains("rollback plan")
        || lower.contains("revert strategy")
        || lower == "rollback"
        || lower.starts_with("rollback:")
    {
        Some(PrTemplateSection::RollbackStrategy)
    } else {
        None
    }
}

/// Appends a line of content to the target section in the accumulator map.
fn append_section_line(
    sections: &mut HashMap<PrTemplateSection, String>,
    section: PrTemplateSection,
    line: &str,
) {
    let entry = sections.entry(section).or_default();
    if !entry.is_empty() {
        entry.push('\n');
    }
    entry.push_str(line);
}

/// Evaluates a section's content and records any compliance findings.
fn evaluate_section(
    section: PrTemplateSection,
    sections: &HashMap<PrTemplateSection, String>,
    findings: &mut Vec<TemplateAuditFinding>,
) -> Option<String> {
    match sections.get(&section) {
        Some(raw_content) => {
            let cleaned = strip_html_comments_and_clean(raw_content);
            let words = count_words(&cleaned);
            if words < MINIMUM_SECTION_WORDS {
                findings.push(TemplateAuditFinding::IncompleteSection {
                    section,
                    minimum_words: MINIMUM_SECTION_WORDS,
                    found_words: words,
                });
            }
            Some(cleaned)
        }
        None => {
            findings.push(TemplateAuditFinding::MissingSection { section });
            None
        }
    }
}

/// Strips HTML comment blocks (e.g. `<!-- ... -->`) and trims leading/trailing whitespace.
fn strip_html_comments_and_clean(content: &str) -> String {
    let mut cleaned_lines = Vec::new();
    let mut in_comment = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            continue;
        }
        if trimmed.starts_with("<!--") {
            in_comment = true;
            continue;
        }
        if in_comment {
            if trimmed.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        cleaned_lines.push(trimmed);
    }

    cleaned_lines.join(" ")
}

/// Counts whitespace-delimited words in the provided text.
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}
