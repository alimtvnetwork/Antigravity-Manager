//! Pull Request Template Compliance Linter.
//!
//! Enforces PR template alignment per AGENTS.md:
//! Validates that every pull request description comprehensively addresses the 4 mandatory
//! sections: problem classification, behavior alterations, unverified paths, and rollback strategy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PrTemplateComplianceReport {
    pub is_compliant: bool,
    pub has_problem_classification: bool,
    pub has_behavior_alterations: bool,
    pub has_unverified_paths: bool,
    pub has_rollback_strategy: bool,
    pub missing_sections: Vec<String>,
}

/// Evaluates whether a PR description satisfies the mandatory 4-part AGENTS.md template.
pub fn lint_pr_description(description_markdown: &str) -> PrTemplateComplianceReport {
    let lower = description_markdown.to_ascii_lowercase();

    let has_problem_classification = lower.contains("problem classification")
        || lower.contains("## problem")
        || lower.contains("### problem");

    let has_behavior_alterations = lower.contains("behavior alteration")
        || lower.contains("behavior changes")
        || lower.contains("## behavior")
        || lower.contains("### behavior");

    let has_unverified_paths = lower.contains("unverified path")
        || lower.contains("unverified")
        || lower.contains("paths not covered");

    let has_rollback_strategy = lower.contains("rollback strategy")
        || lower.contains("rollback plan")
        || lower.contains("revert strategy");

    let mut missing_sections = Vec::new();
    if !has_problem_classification {
        missing_sections.push("Problem Classification".to_string());
    }
    if !has_behavior_alterations {
        missing_sections.push("Behavior Alterations".to_string());
    }
    if !has_unverified_paths {
        missing_sections.push("Unverified Paths".to_string());
    }
    if !has_rollback_strategy {
        missing_sections.push("Rollback Strategy".to_string());
    }

    let is_compliant = missing_sections.is_empty();

    PrTemplateComplianceReport {
        is_compliant,
        has_problem_classification,
        has_behavior_alterations,
        has_unverified_paths,
        has_rollback_strategy,
        missing_sections,
    }
}
