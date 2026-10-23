//! PR Unverified Paths & Cost Disclosure Linter (`Imp-652`).
//!
//! Enforces AGENTS.md Contributor Respect & Attribution invariant:
//! "Disclose costs before merging: highlight affected existing behaviors and
//! unverified paths alongside improvements. Own tooling and documentation gaps
//! directly rather than attributing downstream frictions to contributors."
//!
//! Audits pull request descriptions to ensure explicit, genuine disclosures for:
//! 1. Affected existing behaviors
//! 2. Unverified execution paths
//! 3. Known tooling and documentation gaps
//!
//! Detects and intercepts lazy placeholder evasions (TODO, N/A, none, TBD).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Category of cost disclosure evaluated during PR auditing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CostDisclosureCategory {
    AffectedExistingBehaviors,
    UnverifiedPaths,
    KnownToolingGaps,
    PerformanceOrMemoryImpact,
}

impl CostDisclosureCategory {
    /// Returns canonical display label for this disclosure category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AffectedExistingBehaviors => "Affected Existing Behaviors",
            Self::UnverifiedPaths => "Unverified Paths",
            Self::KnownToolingGaps => "Known Tooling Gaps",
            Self::PerformanceOrMemoryImpact => "Performance or Memory Impact",
        }
    }
}

impl std::fmt::Display for CostDisclosureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Audit findings generated when examining PR cost disclosures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnverifiedPathFinding {
    MissingUnverifiedPathsSection,
    EmptyUnverifiedPathsList,
    PlaceholderContentDetected {
        category: CostDisclosureCategory,
        snippet: String,
    },
    MissingAffectedBehaviorsSection,
}

impl std::fmt::Display for UnverifiedPathFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUnverifiedPathsSection => {
                write!(f, "Missing mandatory 'Unverified Paths' section")
            }
            Self::EmptyUnverifiedPathsList => {
                write!(f, "The 'Unverified Paths' section contains an empty list")
            }
            Self::PlaceholderContentDetected { category, snippet } => {
                write!(
                    f,
                    "Placeholder evasion detected in {} section: '{}'",
                    category, snippet
                )
            }
            Self::MissingAffectedBehaviorsSection => {
                write!(
                    f,
                    "Missing mandatory 'Affected Existing Behaviors' section"
                )
            }
        }
    }
}

impl std::error::Error for UnverifiedPathFinding {}

/// Detailed report on PR cost disclosures and findings.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CostDisclosureReport {
    pub is_disclosure_complete: bool,
    pub unverified_paths: Vec<String>,
    pub affected_behaviors: Vec<String>,
    pub known_gaps: Vec<String>,
    pub findings: Vec<UnverifiedPathFinding>,
    pub guidance: String,
}

impl CostDisclosureReport {
    /// Returns true if any audit findings were detected.
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Returns the number of audit findings detected.
    pub fn findings_count(&self) -> usize {
        self.findings.len()
    }

    /// Returns true if cost disclosures are complete and without findings.
    pub fn is_complete(&self) -> bool {
        self.is_disclosure_complete
    }
}

/// Audits a PR markdown description for mandatory cost disclosures and placeholder evasions.
pub fn audit_pr_cost_disclosures(pr_body: &str) -> CostDisclosureReport {
    let (seen, items_map) = parse_markdown_sections(pr_body);
    let mut findings = Vec::new();
    let mut unverified_paths = Vec::new();
    let mut affected_behaviors = Vec::new();
    let mut known_gaps = Vec::new();

    audit_unverified_paths(&seen, &items_map, &mut unverified_paths, &mut findings);
    audit_affected_behaviors(&seen, &items_map, &mut affected_behaviors, &mut findings);
    audit_supplementary_sections(&seen, &items_map, &mut known_gaps, &mut findings);

    let is_disclosure_complete = findings.is_empty()
        && !unverified_paths.is_empty()
        && !affected_behaviors.is_empty();
    let guidance = generate_guidance(&findings, is_disclosure_complete);

    CostDisclosureReport {
        is_disclosure_complete,
        unverified_paths,
        affected_behaviors,
        known_gaps,
        findings,
        guidance,
    }
}

/// Parses markdown lines into detected categories and extracted items.
fn parse_markdown_sections(
    markdown: &str,
) -> (
    HashSet<CostDisclosureCategory>,
    HashMap<CostDisclosureCategory, Vec<String>>,
) {
    let mut seen = HashSet::new();
    let mut items: HashMap<CostDisclosureCategory, Vec<String>> = HashMap::new();
    let mut current_cat: Option<CostDisclosureCategory> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(header_title) = extract_header_title(trimmed) {
            current_cat = classify_header(header_title);
            if let Some(cat) = current_cat {
                seen.insert(cat);
                items.entry(cat).or_default();
            }
        } else if let Some(cat) = current_cat {
            if let Some(item) = extract_item_text(trimmed) {
                items.entry(cat).or_default().push(item);
            }
        }
    }

    (seen, items)
}

/// Extracts header title from markdown headings or bold lead lines.
fn extract_header_title(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with('#') {
        Some(trimmed.trim_start_matches('#').trim().trim_matches('*').trim())
    } else if (trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4)
        || (trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > 4)
    {
        Some(trimmed.trim_matches('*').trim_matches('_').trim())
    } else {
        None
    }
}

/// Classifies markdown header title into CostDisclosureCategory.
fn classify_header(header: &str) -> Option<CostDisclosureCategory> {
    let lower = header.to_ascii_lowercase();
    let clean = lower.trim_end_matches(':').trim();

    if clean.contains("unverified path")
        || clean.contains("unverified paths")
        || clean.contains("paths not covered")
        || clean == "unverified"
        || clean.starts_with("unverified:")
    {
        Some(CostDisclosureCategory::UnverifiedPaths)
    } else if clean.contains("affected existing behavior")
        || clean.contains("affected existing behaviors")
        || clean.contains("affected behavior")
        || clean.contains("affected behaviors")
        || clean.contains("behavior alteration")
        || clean.contains("behavior alterations")
        || clean.contains("behavior changes")
    {
        Some(CostDisclosureCategory::AffectedExistingBehaviors)
    } else if clean.contains("known tooling gap")
        || clean.contains("known tooling gaps")
        || clean.contains("tooling gap")
        || clean.contains("tooling gaps")
        || clean.contains("known gap")
        || clean.contains("known gaps")
        || clean.contains("tooling and documentation gap")
        || clean.contains("documentation gap")
    {
        Some(CostDisclosureCategory::KnownToolingGaps)
    } else if clean.contains("performance or memory impact")
        || clean.contains("performance impact")
        || clean.contains("memory impact")
    {
        Some(CostDisclosureCategory::PerformanceOrMemoryImpact)
    } else {
        None
    }
}

/// Extracts cleaned bullet or paragraph item text, skipping comments.
fn extract_item_text(trimmed: &str) -> Option<String> {
    if trimmed.is_empty() || trimmed.starts_with("<!--") {
        return None;
    }

    let stripped = if let Some(rest) = trimmed.strip_prefix("- ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("* ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("+ ") {
        rest
    } else if let Some(idx) = trimmed.find(". ") {
        let prefix = &trimmed[..idx];
        if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
            &trimmed[idx + 2..]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    let item = stripped.trim();
    if item.is_empty() {
        None
    } else {
        Some(item.to_string())
    }
}

/// Audits unverified paths section presence, emptiness, and placeholders.
fn audit_unverified_paths(
    seen: &HashSet<CostDisclosureCategory>,
    items_map: &HashMap<CostDisclosureCategory, Vec<String>>,
    unverified_paths: &mut Vec<String>,
    findings: &mut Vec<UnverifiedPathFinding>,
) {
    let cat = CostDisclosureCategory::UnverifiedPaths;
    if !seen.contains(&cat) {
        findings.push(UnverifiedPathFinding::MissingUnverifiedPathsSection);
        return;
    }

    let items = items_map.get(&cat).map(|v| v.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        findings.push(UnverifiedPathFinding::EmptyUnverifiedPathsList);
        return;
    }

    for item in items {
        if is_placeholder_text(item) {
            findings.push(UnverifiedPathFinding::PlaceholderContentDetected {
                category: cat,
                snippet: item.clone(),
            });
        } else {
            unverified_paths.push(item.clone());
        }
    }
}

/// Audits affected behaviors section presence and placeholders.
fn audit_affected_behaviors(
    seen: &HashSet<CostDisclosureCategory>,
    items_map: &HashMap<CostDisclosureCategory, Vec<String>>,
    affected_behaviors: &mut Vec<String>,
    findings: &mut Vec<UnverifiedPathFinding>,
) {
    let cat = CostDisclosureCategory::AffectedExistingBehaviors;
    if !seen.contains(&cat) {
        findings.push(UnverifiedPathFinding::MissingAffectedBehaviorsSection);
        return;
    }

    let items = items_map.get(&cat).map(|v| v.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        findings.push(UnverifiedPathFinding::MissingAffectedBehaviorsSection);
        return;
    }

    for item in items {
        if is_placeholder_text(item) {
            findings.push(UnverifiedPathFinding::PlaceholderContentDetected {
                category: cat,
                snippet: item.clone(),
            });
        } else {
            affected_behaviors.push(item.clone());
        }
    }
}

/// Audits supplementary sections (known gaps, performance impacts).
fn audit_supplementary_sections(
    seen: &HashSet<CostDisclosureCategory>,
    items_map: &HashMap<CostDisclosureCategory, Vec<String>>,
    known_gaps: &mut Vec<String>,
    findings: &mut Vec<UnverifiedPathFinding>,
) {
    let cat_gaps = CostDisclosureCategory::KnownToolingGaps;
    if seen.contains(&cat_gaps) {
        if let Some(items) = items_map.get(&cat_gaps) {
            for item in items {
                if is_placeholder_text(item) {
                    findings.push(UnverifiedPathFinding::PlaceholderContentDetected {
                        category: cat_gaps,
                        snippet: item.clone(),
                    });
                } else {
                    known_gaps.push(item.clone());
                }
            }
        }
    }

    let cat_perf = CostDisclosureCategory::PerformanceOrMemoryImpact;
    if seen.contains(&cat_perf) {
        if let Some(items) = items_map.get(&cat_perf) {
            for item in items {
                if is_placeholder_text(item) {
                    findings.push(UnverifiedPathFinding::PlaceholderContentDetected {
                        category: cat_perf,
                        snippet: item.clone(),
                    });
                }
            }
        }
    }
}

/// Identifies whether extracted text is a lazy placeholder evasion.
pub fn is_placeholder_text(text: &str) -> bool {
    let clean = text
        .trim()
        .trim_matches(|c: char| {
            c == '*' || c == '_' || c == '`' || c == '[' || c == ']' || c == '<' || c == '>'
        })
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_ascii_lowercase();

    if clean.is_empty() {
        return true;
    }

    const KNOWN_PLACEHOLDERS: &[&str] = &[
        "todo",
        "tbd",
        "tba",
        "n/a",
        "na",
        "none",
        "nil",
        "null",
        "nothing",
        "not applicable",
        "no unverified paths",
        "no affected behaviors",
        "no known gaps",
        "no tooling gaps",
        "none yet",
        "nothing yet",
        "none so far",
        "wip",
        "placeholder",
        "leave blank",
        "as above",
        "-",
        "--",
        "...",
        "?",
    ];

    if KNOWN_PLACEHOLDERS.contains(&clean.as_str()) {
        return true;
    }

    if (clean.starts_with("todo") || clean.starts_with("tbd")) && clean.len() <= 30 {
        return true;
    }

    false
}

/// Generates human-actionable guidance from findings.
fn generate_guidance(findings: &[UnverifiedPathFinding], is_complete: bool) -> String {
    if is_complete {
        "PR cost disclosures are complete and compliant with AGENTS.md standards.".to_string()
    } else {
        let details: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
        format!(
            "PR cost disclosures incomplete per AGENTS.md standards: {}",
            details.join("; ")
        )
    }
}
