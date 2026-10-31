//! Unit tests for Single Problem Scope & PR Problem Classification Linter.

use super::pr_problem_classification_linter::{
    audit_pr_scope, classify_file_domain, FunctionalDomain, ScopeEntanglementViolation,
    ScopeLinterReport,
};

#[test]
fn test_classify_file_domain_all_domains() {
    assert_eq!(
        classify_file_domain("src-tauri/src/proxy/handler.rs"),
        FunctionalDomain::GatewayProxy
    );
    assert_eq!(
        classify_file_domain("src-tauri/src/proxy/stream.rs"),
        FunctionalDomain::GatewayProxy
    );
    assert_eq!(
        classify_file_domain("src-tauri/src/modules/release_gate/channel_gate.rs"),
        FunctionalDomain::ReleaseTooling
    );
    assert_eq!(
        classify_file_domain(".github/workflows/ci.yml"),
        FunctionalDomain::ReleaseTooling
    );
    assert_eq!(
        classify_file_domain("src-tauri/src/modules/instance/switcher.rs"),
        FunctionalDomain::InstanceManagement
    );
    assert_eq!(
        classify_file_domain("src-tauri/src/modules/db_orm/repo.rs"),
        FunctionalDomain::DatabaseOrm
    );
    assert_eq!(
        classify_file_domain("src/components/Header.tsx"),
        FunctionalDomain::UiComponents
    );
    assert_eq!(
        classify_file_domain("src/App.tsx"),
        FunctionalDomain::UiComponents
    );
    assert_eq!(
        classify_file_domain("components/Sidebar.tsx"),
        FunctionalDomain::UiComponents
    );
    assert_eq!(
        classify_file_domain("docs/architecture.md"),
        FunctionalDomain::Documentation
    );
    assert_eq!(
        classify_file_domain("README.md"),
        FunctionalDomain::Documentation
    );
}

#[test]
fn test_classify_file_domain_windows_paths() {
    assert_eq!(
        classify_file_domain("src-tauri\\src\\proxy\\handler.rs"),
        FunctionalDomain::GatewayProxy
    );
    assert_eq!(
        classify_file_domain("src\\components\\Modal.tsx"),
        FunctionalDomain::UiComponents
    );
    assert_eq!(
        classify_file_domain("docs\\api.md"),
        FunctionalDomain::Documentation
    );
}

#[test]
fn test_cohesive_pr_single_functional_domain_with_docs() {
    let files = vec![
        "src-tauri/src/proxy/handler.rs".to_string(),
        "src-tauri/src/proxy/stream.rs".to_string(),
        "docs/proxy_architecture.md".to_string(),
        "README.md".to_string(),
    ];
    let pr_body = "## Problem Classification\nFix gateway streaming timeout.";

    let report = audit_pr_scope(&files, pr_body);
    assert!(report.is_cohesive);
    assert_eq!(report.primary_domain, Some(FunctionalDomain::GatewayProxy));
    assert_eq!(report.touched_domains.len(), 2);
    assert!(report
        .touched_domains
        .contains(&FunctionalDomain::GatewayProxy));
    assert!(report
        .touched_domains
        .contains(&FunctionalDomain::Documentation));
    assert!(report.violations.is_empty());
    assert!(!report.has_violations());
    assert!(report.summary().contains("cohesive"));
}

#[test]
fn test_entangled_pr_modifying_three_disjoint_domains() {
    let files = vec![
        "src-tauri/src/proxy/handler.rs".to_string(),
        "src-tauri/src/modules/release_gate/channel_gate.rs".to_string(),
        "src/components/Header.tsx".to_string(),
    ];
    let pr_body = "## Problem Classification\nMisc fixes across multiple modules.";

    let report = audit_pr_scope(&files, pr_body);
    assert!(!report.is_cohesive);
    assert!(report.has_violations());
    assert_eq!(report.violations.len(), 1);

    match &report.violations[0] {
        ScopeEntanglementViolation::UnrelatedDomainsMixed {
            domains,
            file_count,
        } => {
            assert_eq!(*file_count, 3);
            assert_eq!(domains.len(), 3);
            assert!(domains.contains(&FunctionalDomain::GatewayProxy));
            assert!(domains.contains(&FunctionalDomain::ReleaseTooling));
            assert!(domains.contains(&FunctionalDomain::UiComponents));
        }
        _ => panic!("Expected UnrelatedDomainsMixed violation"),
    }
}

#[test]
fn test_missing_problem_classification_header() {
    let files = vec!["src-tauri/src/proxy/handler.rs".to_string()];
    let pr_body = "This PR optimizes socket streaming without providing standard headers.";

    let report = audit_pr_scope(&files, pr_body);
    assert!(!report.is_cohesive);
    assert!(report.has_violations());
    assert_eq!(report.violations.len(), 1);

    assert_eq!(
        report.violations[0],
        ScopeEntanglementViolation::MissingProblemClassificationHeader
    );
}

#[test]
fn test_accepts_lowercase_problem_classification_header() {
    let files = vec!["src-tauri/src/proxy/handler.rs".to_string()];
    let pr_body = "### problem classification: Bugfix in socket stream";

    let report = audit_pr_scope(&files, pr_body);
    assert!(report.is_cohesive);
    assert!(report.violations.is_empty());
}

#[test]
fn test_two_non_doc_domains_allowed() {
    let files = vec![
        "src-tauri/src/proxy/handler.rs".to_string(),
        "src/components/Header.tsx".to_string(),
    ];
    let pr_body = "## Problem Classification\nFull-stack gateway status pill feature.";

    let report = audit_pr_scope(&files, pr_body);
    assert!(report.is_cohesive);
    assert!(report.violations.is_empty());
}

#[test]
fn test_violation_display_and_error_trait() {
    let violation_mixed = ScopeEntanglementViolation::UnrelatedDomainsMixed {
        domains: vec![
            FunctionalDomain::GatewayProxy,
            FunctionalDomain::ReleaseTooling,
            FunctionalDomain::UiComponents,
        ],
        file_count: 3,
    };
    let formatted_mixed = format!("{}", violation_mixed);
    assert!(formatted_mixed.contains("3 unrelated domains"));
    assert!(formatted_mixed.contains("3 file(s)"));

    let violation_header = ScopeEntanglementViolation::MissingProblemClassificationHeader;
    let formatted_header = format!("{}", violation_header);
    assert!(formatted_header.contains("Problem Classification"));

    let error_ref: &dyn std::error::Error = &violation_header;
    assert!(!error_ref.to_string().is_empty());
}

#[test]
fn test_scope_linter_report_display() {
    let report = ScopeLinterReport {
        is_cohesive: false,
        primary_domain: Some(FunctionalDomain::GatewayProxy),
        touched_domains: vec![FunctionalDomain::GatewayProxy],
        violations: vec![ScopeEntanglementViolation::MissingProblemClassificationHeader],
    };
    let formatted = format!("{}", report);
    assert!(formatted.contains("entangled"));
    assert!(formatted.contains("Problem Classification"));
}

#[test]
fn test_empty_file_list() {
    let files: Vec<String> = Vec::new();
    let pr_body = "## Problem Classification\nEmpty test PR";

    let report = audit_pr_scope(&files, pr_body);
    assert!(report.is_cohesive);
    assert_eq!(report.primary_domain, None);
    assert!(report.touched_domains.is_empty());
    assert!(report.violations.is_empty());
}
