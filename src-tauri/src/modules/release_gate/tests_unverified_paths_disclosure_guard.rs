//! Unit tests for Unverified Paths & Cost Disclosure Guard (`AGENTS.md`).

use super::unverified_paths_disclosure_guard::{
    audit_cost_disclosure, CostDisclosureSection, DisclosureFinding, DISALLOWED_PLACEHOLDERS,
};

#[test]
fn test_fully_compliant_pr_body_with_substantive_disclosure() {
    let markdown = r#"
## Summary
Refactor token queue and retry logic across cluster workers.

## Affected Existing Behaviors
Alters the exponential backoff floor from 100 milliseconds to 500 milliseconds.

## Unverified Paths
Full multi-region cluster failover was not verified against live network links.
"#;

    let report = audit_cost_disclosure(markdown, true);

    assert!(report.is_compliant);
    assert!(report.is_clean());
    assert!(report.findings.is_empty());
    assert_eq!(report.total_findings(), 0);
    assert!(report.affected_behaviors_summary.is_some());
    assert!(report.unverified_paths_summary.is_some());

    assert!(report
        .affected_behaviors_summary
        .as_ref()
        .unwrap()
        .contains("exponential backoff floor"));
    assert!(report
        .unverified_paths_summary
        .as_ref()
        .unwrap()
        .contains("multi-region cluster failover"));
}

#[test]
fn test_missing_unverified_paths_on_high_risk_changeset() {
    let markdown = r#"
## Summary
Major migration of core routing table.

## Affected Existing Behaviors
Modifies how active routes are indexed and looked up in memory.
"#;

    let report = audit_cost_disclosure(markdown, true);

    assert!(!report.is_compliant);
    assert!(!report.is_clean());
    assert_eq!(report.findings.len(), 1);

    match &report.findings[0] {
        DisclosureFinding::MissingRequiredSection { section } => {
            assert_eq!(*section, CostDisclosureSection::UnverifiedPaths);
        }
        other => panic!("Expected MissingRequiredSection, got: {:?}", other),
    }
}

#[test]
fn test_placeholder_evasions_flagged_as_boilerplate() {
    let markdown_na = r#"
## Affected Existing Behaviors
Alters the buffer allocation size during streaming response processing.

## Unverified Paths
N/A
"#;

    let report_na = audit_cost_disclosure(markdown_na, true);
    assert!(!report_na.is_compliant);
    assert_eq!(report_na.findings.len(), 1);

    match &report_na.findings[0] {
        DisclosureFinding::BoilerplatePlaceholderDetected {
            section,
            matched_phrase,
        } => {
            assert_eq!(*section, CostDisclosureSection::UnverifiedPaths);
            assert_eq!(matched_phrase, "N/A");
        }
        other => panic!("Expected BoilerplatePlaceholderDetected, got: {:?}", other),
    }

    let markdown_none = r#"
## Affected Existing Behaviors
None

## Unverified Paths
Verified all paths thoroughly in the test suite and production preview.
"#;

    let report_none = audit_cost_disclosure(markdown_none, true);
    assert!(!report_none.is_compliant);
    assert!(matches!(
        &report_none.findings[0],
        DisclosureFinding::BoilerplatePlaceholderDetected {
            section: CostDisclosureSection::AffectedExistingBehaviors,
            ..
        }
    ));
}

#[test]
fn test_trivial_one_word_content_flagged() {
    let markdown = r#"
## Affected Existing Behaviors
Alters the default timeout settings for websocket keepalive heartbeats.

## Unverified Paths
Tested
"#;

    let report = audit_cost_disclosure(markdown, true);
    assert!(!report.is_compliant);
    assert_eq!(report.findings.len(), 1);

    match &report.findings[0] {
        DisclosureFinding::EmptyOrTrivialDisclosure {
            section,
            word_count,
            min_required,
        } => {
            assert_eq!(*section, CostDisclosureSection::UnverifiedPaths);
            assert_eq!(*word_count, 1);
            assert_eq!(*min_required, 4);
        }
        other => panic!("Expected EmptyOrTrivialDisclosure, got: {:?}", other),
    }
}

#[test]
fn test_normal_changeset_passing_when_disclosures_are_present() {
    let markdown = r#"
## Affected Existing Behaviors
Slight adjustment to logging verbosity level in debug mode.

## Unverified Paths
Edge case with corrupted configuration file was skipped.
"#;

    // Normal changeset: is_high_risk_changeset = false
    let report = audit_cost_disclosure(markdown, false);
    assert!(report.is_compliant);
    assert!(report.is_clean());
    assert!(report.findings.is_empty());
}

#[test]
fn test_normal_changeset_without_disclosures_passes() {
    let markdown = r#"
## Summary
Fix typo in documentation file.
"#;

    // Low-risk changesets do not require cost disclosures
    let report = audit_cost_disclosure(markdown, false);
    assert!(report.is_compliant);
    assert!(report.findings.is_empty());
}

#[test]
fn test_high_risk_missing_both_sections_flags_both() {
    let markdown = r#"
## Summary
Rewrite of the process spawning subsystem.
"#;

    let report = audit_cost_disclosure(markdown, true);
    assert!(!report.is_compliant);
    assert_eq!(report.findings.len(), 2);
    assert!(report.findings.contains(&DisclosureFinding::MissingRequiredSection {
        section: CostDisclosureSection::AffectedExistingBehaviors,
    }));
    assert!(report.findings.contains(&DisclosureFinding::MissingRequiredSection {
        section: CostDisclosureSection::UnverifiedPaths,
    }));
}

#[test]
fn test_inline_bullet_disclosure_parsing() {
    let markdown = r#"
## Cost Disclosure
- **Affected Existing Behaviors**: Updates retry backoff strategy from linear to exponential.
- **Unverified Paths**: Dynamic failover under severe high-latency network packet loss conditions.
"#;

    let report = audit_cost_disclosure(markdown, true);
    assert!(report.is_compliant);
    assert!(report.findings.is_empty());
    assert!(report.affected_behaviors_summary.is_some());
    assert!(report.unverified_paths_summary.is_some());
}

#[test]
fn test_all_disallowed_placeholders_are_detected() {
    for &placeholder in DISALLOWED_PLACEHOLDERS {
        let markdown = format!(
            "## Affected Existing Behaviors\n{}\n\n## Unverified Paths\nVerified across all platforms and tested locally.\n",
            placeholder
        );
        let report = audit_cost_disclosure(&markdown, true);
        assert!(
            !report.is_compliant,
            "Placeholder '{}' should have failed audit",
            placeholder
        );
        assert!(matches!(
            &report.findings[0],
            DisclosureFinding::BoilerplatePlaceholderDetected {
                section: CostDisclosureSection::AffectedExistingBehaviors,
                ..
            }
        ));
    }
}

#[test]
fn test_display_and_error_formatting_of_findings() {
    let f1 = DisclosureFinding::MissingRequiredSection {
        section: CostDisclosureSection::UnverifiedPaths,
    };
    assert!(f1
        .to_string()
        .contains("Missing required cost disclosure section: 'Unverified Paths'"));

    let f2 = DisclosureFinding::EmptyOrTrivialDisclosure {
        section: CostDisclosureSection::AffectedExistingBehaviors,
        word_count: 2,
        min_required: 4,
    };
    assert!(f2
        .to_string()
        .contains("Trivial disclosure in section 'Affected Existing Behaviors': 2 words"));

    let f3 = DisclosureFinding::BoilerplatePlaceholderDetected {
        section: CostDisclosureSection::UnverifiedPaths,
        matched_phrase: "TBD".to_string(),
    };
    assert!(f3
        .to_string()
        .contains("Boilerplate placeholder 'TBD' detected"));

    // Verify std::error::Error trait
    let err_box: Box<dyn std::error::Error> = Box::new(f1);
    assert!(!err_box.to_string().is_empty());

    // Verify CostDisclosureSection display
    assert_eq!(
        CostDisclosureSection::AffectedExistingBehaviors.to_string(),
        "Affected Existing Behaviors"
    );
    assert_eq!(
        CostDisclosureSection::UnverifiedPaths.to_string(),
        "Unverified Paths"
    );
}
