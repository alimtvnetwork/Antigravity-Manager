//! Unit tests for PR Unverified Paths & Cost Disclosure Linter (`Imp-652`).

use super::pr_unverified_paths_linter::{
    audit_pr_cost_disclosures, is_placeholder_text, CostDisclosureCategory, CostDisclosureReport,
    UnverifiedPathFinding,
};

#[test]
fn test_complete_valid_pr_body_passes_clean() {
    let pr_body = r#"
## Problem Classification
Refactor proxy handler pipeline for headroom evaluation.

## Affected Existing Behaviors
- Alters default token budget expansion ceiling from 32k to 64k.
- Increases heartbeat ping frequency to 15s.

## Unverified Paths
- Windows Job Object fallback ladder was verified on Win11, but unverified on Windows Server 2019.
- Redis cluster failover under high concurrent request load.

## Known Tooling Gaps
- Local mock runner does not support live SSE chunk packet loss emulation.
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(report.is_complete());
    assert!(report.is_disclosure_complete);
    assert!(!report.has_findings());
    assert_eq!(report.findings_count(), 0);
    assert_eq!(report.affected_behaviors.len(), 2);
    assert_eq!(report.unverified_paths.len(), 2);
    assert_eq!(report.known_gaps.len(), 1);
    assert!(report.guidance.contains("complete and compliant"));
}

#[test]
fn test_missing_unverified_paths_section_is_flagged() {
    let pr_body = r#"
## Problem Classification
Bug fix in model router.

## Affected Existing Behaviors
- Changes regex fallback order.
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(!report.is_disclosure_complete);
    assert!(report.has_findings());
    assert!(report
        .findings
        .contains(&UnverifiedPathFinding::MissingUnverifiedPathsSection));
}

#[test]
fn test_missing_affected_behaviors_section_is_flagged() {
    let pr_body = r#"
## Problem Classification
Bug fix in model router.

## Unverified Paths
- Tested on macOS only; unverified on Linux aarch64.
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(!report.is_disclosure_complete);
    assert!(report.has_findings());
    assert!(report
        .findings
        .contains(&UnverifiedPathFinding::MissingAffectedBehaviorsSection));
}

#[test]
fn test_placeholder_content_todo_is_flagged() {
    let pr_body = r#"
## Affected Existing Behaviors
- Alters routing rules.

## Unverified Paths
- TODO: verify on Windows
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(!report.is_disclosure_complete);
    assert!(report.has_findings());
    let found = report.findings.iter().any(|f| match f {
        UnverifiedPathFinding::PlaceholderContentDetected { category, snippet } => {
            *category == CostDisclosureCategory::UnverifiedPaths && snippet.contains("TODO")
        }
        _ => false,
    });
    assert!(
        found,
        "Expected PlaceholderContentDetected with TODO snippet"
    );
}

#[test]
fn test_placeholder_content_na_and_none_are_flagged() {
    let pr_body = r#"
## Affected Existing Behaviors
- N/A

## Unverified Paths
- None.
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(!report.is_disclosure_complete);
    assert_eq!(report.findings_count(), 2);

    let na_found = report.findings.iter().any(|f| match f {
        UnverifiedPathFinding::PlaceholderContentDetected { category, snippet } => {
            *category == CostDisclosureCategory::AffectedExistingBehaviors && snippet == "N/A"
        }
        _ => false,
    });
    let none_found = report.findings.iter().any(|f| match f {
        UnverifiedPathFinding::PlaceholderContentDetected { category, snippet } => {
            *category == CostDisclosureCategory::UnverifiedPaths && snippet == "None."
        }
        _ => false,
    });

    assert!(na_found, "Expected N/A placeholder in affected behaviors");
    assert!(none_found, "Expected None. placeholder in unverified paths");
}

#[test]
fn test_empty_unverified_paths_section_is_flagged() {
    let pr_body = r#"
## Affected Existing Behaviors
- Normalizes authorization headers.

## Unverified Paths

## Known Tooling Gaps
- None recorded.
"#;

    let report = audit_pr_cost_disclosures(pr_body);

    assert!(!report.is_disclosure_complete);
    assert!(report.has_findings());
    assert!(report
        .findings
        .contains(&UnverifiedPathFinding::EmptyUnverifiedPathsList));
}

#[test]
fn test_is_placeholder_text_helper() {
    assert!(is_placeholder_text("TODO"));
    assert!(is_placeholder_text("todo: check"));
    assert!(is_placeholder_text("tbd"));
    assert!(is_placeholder_text("N/A"));
    assert!(is_placeholder_text("na"));
    assert!(is_placeholder_text("none"));
    assert!(is_placeholder_text("None."));
    assert!(is_placeholder_text("**none**"));
    assert!(is_placeholder_text("`N/A`"));
    assert!(is_placeholder_text(""));

    assert!(!is_placeholder_text("Windows Job object fallback path"));
    assert!(!is_placeholder_text(
        "Alters default timeout from 30s to 45s"
    ));
}

#[test]
fn test_serde_roundtrip_serialization() {
    let report = CostDisclosureReport {
        is_disclosure_complete: true,
        unverified_paths: vec!["Path A".to_string()],
        affected_behaviors: vec!["Behavior B".to_string()],
        known_gaps: vec!["Gap C".to_string()],
        findings: vec![],
        guidance: "PR cost disclosures are complete and compliant with AGENTS.md standards."
            .to_string(),
    };

    let serialized = serde_json::to_string(&report).expect("Serialization must succeed");
    let deserialized: CostDisclosureReport =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");

    assert_eq!(report, deserialized);
}

#[test]
fn test_serde_roundtrip_with_findings() {
    let report = CostDisclosureReport {
        is_disclosure_complete: false,
        unverified_paths: vec![],
        affected_behaviors: vec![],
        known_gaps: vec![],
        findings: vec![
            UnverifiedPathFinding::MissingUnverifiedPathsSection,
            UnverifiedPathFinding::PlaceholderContentDetected {
                category: CostDisclosureCategory::AffectedExistingBehaviors,
                snippet: "TODO".to_string(),
            },
        ],
        guidance: "Incomplete disclosures".to_string(),
    };

    let serialized = serde_json::to_string(&report).expect("Serialization must succeed");
    let deserialized: CostDisclosureReport =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");

    assert_eq!(report, deserialized);
}
