//! Unit tests for Maintainer Staging Protocol Guard.

use crate::modules::release_gate::staging_protocol_guard::{
    evaluate_staging_protocol, ChangeSetSummary, StagingDecidingFactor, StagingReport,
    StagingVerdict,
};

#[test]
fn test_schema_migration_requires_staging() {
    let summary = ChangeSetSummary::new(2, 45, 10).with_schema_migration(true);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Required);
    assert!(report.is_beta_staging_required);
    assert!(!report.is_beta_staging_recommended);
    assert!(report.is_staging_needed());
    assert!(report.requires_maintainer_confirmation());
    assert_eq!(report.recommended_branch_target(), "origin/beta");
    assert!(report
        .deciding_factors
        .iter()
        .any(|f| matches!(f, StagingDecidingFactor::SchemaMigrationDetected)));
    assert!(report.guidance.contains("beta"));
}

#[test]
fn test_trivial_one_line_fix_unnecessary() {
    let summary = ChangeSetSummary::new(1, 1, 1).with_one_line_fix(true);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Unnecessary);
    assert!(!report.is_beta_staging_required);
    assert!(!report.is_beta_staging_recommended);
    assert!(!report.is_staging_needed());
    assert!(!report.requires_maintainer_confirmation());
    assert_eq!(report.recommended_branch_target(), "origin/main");
    assert!(report
        .deciding_factors
        .iter()
        .any(|f| matches!(f, StagingDecidingFactor::TrivialOneLineFix)));
    assert!(report.guidance.contains("trivial one-line fix"));
}

#[test]
fn test_docs_only_unnecessary() {
    let summary = ChangeSetSummary::new(4, 120, 30).with_docs_only(true);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Unnecessary);
    assert!(!report.is_beta_staging_required);
    assert!(!report.is_beta_staging_recommended);
    assert!(!report.is_staging_needed());
    assert!(!report.requires_maintainer_confirmation());
    assert_eq!(report.recommended_branch_target(), "origin/main");
    assert!(report
        .deciding_factors
        .iter()
        .any(|f| matches!(f, StagingDecidingFactor::DocumentationOnly)));
    assert!(report.guidance.contains("documentation-only"));
}

#[test]
fn test_multi_module_without_schema_recommended() {
    let summary = ChangeSetSummary::new(5, 50, 20);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Recommended);
    assert!(!report.is_beta_staging_required);
    assert!(report.is_beta_staging_recommended);
    assert!(report.is_staging_needed());
    assert!(report.requires_maintainer_confirmation());
    assert_eq!(report.recommended_branch_target(), "origin/beta");
    assert!(report.deciding_factors.iter().any(|f| matches!(
        f,
        StagingDecidingFactor::MultiModuleChangeset { modules_count: 5 }
    )));
}

#[test]
fn test_large_refactor_over_500_lines_required() {
    let summary = ChangeSetSummary::new(3, 400, 150);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Required);
    assert!(report.is_beta_staging_required);
    assert!(report.requires_maintainer_confirmation());
    assert!(report.deciding_factors.iter().any(|f| matches!(
        f,
        StagingDecidingFactor::NonTrivialCodebaseRefactor { lines_changed: 550 }
    )));
}

#[test]
fn test_moderate_refactor_over_100_lines_recommended() {
    let summary = ChangeSetSummary::new(2, 80, 50);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Recommended);
    assert!(report.is_beta_staging_recommended);
    assert!(report.requires_maintainer_confirmation());
    assert!(report.deciding_factors.iter().any(|f| matches!(
        f,
        StagingDecidingFactor::NonTrivialCodebaseRefactor { lines_changed: 130 }
    )));
}

#[test]
fn test_public_surface_change_required() {
    let summary = ChangeSetSummary::new(1, 15, 5).with_public_surface("Tauri IPC Route");
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Required);
    assert!(report.is_beta_staging_required);
    assert!(report.deciding_factors.iter().any(|f| {
        match f {
            StagingDecidingFactor::PublicSurfaceModified { surface_type } => {
                surface_type == "Tauri IPC Route"
            }
            _ => false,
        }
    }));
}

#[test]
fn test_feature_flag_introduced_required() {
    let summary = ChangeSetSummary::new(2, 30, 0).with_feature_flag("enable_thinking_v2");
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Required);
    assert!(report.is_beta_staging_required);
    assert!(report.deciding_factors.iter().any(|f| {
        match f {
            StagingDecidingFactor::FeatureFlagIntroduced { flag_name } => {
                flag_name == "enable_thinking_v2"
            }
            _ => false,
        }
    }));
}

#[test]
fn test_release_machinery_change_required() {
    let summary = ChangeSetSummary::new(1, 10, 5).with_release_machinery(true);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Required);
    assert!(report.is_beta_staging_required);
    assert!(report
        .deciding_factors
        .iter()
        .any(|f| matches!(f, StagingDecidingFactor::ReleaseMachineryModified)));
}

#[test]
fn test_small_localized_change_unnecessary() {
    let summary = ChangeSetSummary::new(2, 20, 10);
    let report = evaluate_staging_protocol(&summary);

    assert_eq!(report.verdict, StagingVerdict::Unnecessary);
    assert!(!report.is_beta_staging_required);
    assert!(!report.is_beta_staging_recommended);
    assert!(!report.is_staging_needed());
    assert!(!report.requires_maintainer_confirmation());
    assert_eq!(report.recommended_branch_target(), "origin/main");
    assert!(report.deciding_factors.is_empty());
}

#[test]
fn test_staging_verdict_helpers() {
    let req = StagingVerdict::Required;
    assert_eq!(req.as_str(), "required");
    assert_eq!(format!("{}", req), "required");
    assert!(req.is_required());
    assert!(!req.is_recommended());
    assert!(!req.is_unnecessary());

    let rec = StagingVerdict::Recommended;
    assert_eq!(rec.as_str(), "recommended");
    assert_eq!(format!("{}", rec), "recommended");
    assert!(!rec.is_required());
    assert!(rec.is_recommended());
    assert!(!rec.is_unnecessary());

    let unnec = StagingVerdict::Unnecessary;
    assert_eq!(unnec.as_str(), "unnecessary");
    assert_eq!(format!("{}", unnec), "unnecessary");
    assert!(!unnec.is_required());
    assert!(!unnec.is_recommended());
    assert!(unnec.is_unnecessary());
}

#[test]
fn test_staging_deciding_factor_descriptions() {
    let f1 = StagingDecidingFactor::SchemaMigrationDetected;
    assert!(f1.description().contains("schema migration"));

    let f2 = StagingDecidingFactor::PublicSurfaceModified {
        surface_type: "REST API".to_string(),
    };
    assert!(f2.description().contains("REST API"));

    let f3 = StagingDecidingFactor::FeatureFlagIntroduced {
        flag_name: "use_v2".to_string(),
    };
    assert!(f3.description().contains("use_v2"));

    let f4 = StagingDecidingFactor::ReleaseMachineryModified;
    assert!(f4.description().contains("Release machinery"));

    let f5 = StagingDecidingFactor::MultiModuleChangeset { modules_count: 7 };
    assert!(f5.description().contains("7"));

    let f6 = StagingDecidingFactor::NonTrivialCodebaseRefactor { lines_changed: 800 };
    assert!(f6.description().contains("800"));

    let f7 = StagingDecidingFactor::TrivialOneLineFix;
    assert!(f7.description().contains("single-line"));

    let f8 = StagingDecidingFactor::DocumentationOnly;
    assert!(f8.description().contains("Documentation-only"));
}

#[test]
fn test_serde_roundtrip_staging_types() {
    let summary = ChangeSetSummary::new(4, 600, 20)
        .with_schema_migration(true)
        .with_public_surface("CLI parser");

    let report = evaluate_staging_protocol(&summary);

    let serialized_report = serde_json::to_string(&report).expect("Serialize StagingReport");
    let deserialized_report: StagingReport =
        serde_json::from_str(&serialized_report).expect("Deserialize StagingReport");
    assert_eq!(report, deserialized_report);

    let serialized_summary = serde_json::to_string(&summary).expect("Serialize ChangeSetSummary");
    let deserialized_summary: ChangeSetSummary =
        serde_json::from_str(&serialized_summary).expect("Deserialize ChangeSetSummary");
    assert_eq!(summary, deserialized_summary);

    let verdict = StagingVerdict::Required;
    let serialized_verdict = serde_json::to_string(&verdict).expect("Serialize StagingVerdict");
    assert_eq!(serialized_verdict, "\"required\"");
    let deserialized_verdict: StagingVerdict =
        serde_json::from_str(&serialized_verdict).expect("Deserialize StagingVerdict");
    assert_eq!(verdict, deserialized_verdict);
}
