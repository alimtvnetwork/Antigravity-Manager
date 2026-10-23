//! Unit tests for Published-Tip Rewrite Guard.

use crate::modules::release_gate::published_tip_rewrite_guard::{
    evaluate_rewrite_safety, DriftCheckStatus, RewriteOperation, RewritePreconditionViolation,
    RewriteVerdict,
};

#[test]
fn test_open_prs_and_no_backup_ref_blocks_rewrite() {
    let op = RewriteOperation::published("feature/auth-rework")
        .with_open_prs(vec![101, 105])
        .with_drift_status(DriftCheckStatus::ZeroDriftConfirmed);

    let verdict = evaluate_rewrite_safety(&op);
    assert!(!verdict.is_allowed);
    assert!(verdict.has_violations());
    assert_eq!(verdict.violations.len(), 2);

    let has_pr_violation = verdict.violations.iter().any(|v| {
        matches!(
            v,
            RewritePreconditionViolation::PublishedTipWithOpenPrs {
                open_pr_count: 2,
                ..
            }
        )
    });
    assert!(has_pr_violation);

    let has_backup_violation = verdict.violations.iter().any(|v| {
        matches!(
            v,
            RewritePreconditionViolation::MissingRollbackBackupRef { .. }
        )
    });
    assert!(has_backup_violation);
    assert!(verdict.guidance.contains("blocked by safety preconditions"));
}

#[test]
fn test_backup_ref_present_but_drift_not_checked_blocks_rewrite() {
    let op = RewriteOperation::published("main")
        .with_backup_ref("backup/main-pre-rewrite-1792652000")
        .with_drift_status(DriftCheckStatus::NotChecked);

    let verdict = evaluate_rewrite_safety(&op);
    assert!(!verdict.is_allowed);
    assert!(verdict.has_violations());
    assert_eq!(verdict.violations.len(), 1);

    assert!(matches!(
        verdict.violations[0],
        RewritePreconditionViolation::DriftCheckNotPerformed
    ));
    assert!(verdict.guidance.contains("Zero content drift check"));
}

#[test]
fn test_published_tip_with_backup_and_zero_drift_allowed() {
    let op = RewriteOperation::published("beta")
        .with_backup_ref("backup/beta-pre-sync-1792652000")
        .with_drift_status(DriftCheckStatus::ZeroDriftConfirmed);

    let verdict = evaluate_rewrite_safety(&op);
    assert!(verdict.is_allowed);
    assert!(!verdict.has_violations());
    assert!(verdict.violations.is_empty());
    assert!(verdict.guidance.contains("approved"));
}

#[test]
fn test_unpublished_local_branch_allowed_freely() {
    let op = RewriteOperation::local("local/experiment-fix");
    let verdict = evaluate_rewrite_safety(&op);

    assert!(verdict.is_allowed);
    assert!(!verdict.has_violations());
    assert!(verdict.violations.is_empty());
    assert!(verdict.guidance.contains("unpublished local branch"));
}

#[test]
fn test_content_drift_detected_blocks_rewrite() {
    let op = RewriteOperation::published("beta")
        .with_backup_ref("backup/beta-pre-sync-1792652000")
        .with_drift_status(DriftCheckStatus::DriftDetected {
            differing_files: vec![
                "src/modules/proxy.rs".to_string(),
                "src/error.rs".to_string(),
            ],
        });

    let verdict = evaluate_rewrite_safety(&op);
    assert!(!verdict.is_allowed);
    assert!(verdict.has_violations());
    assert_eq!(verdict.violations.len(), 1);

    assert!(matches!(
        verdict.violations[0],
        RewritePreconditionViolation::ContentDriftDetected {
            differing_files_count: 2
        }
    ));
}

#[test]
fn test_violation_descriptions() {
    let v1 = RewritePreconditionViolation::PublishedTipWithOpenPrs {
        branch: "main".to_string(),
        open_pr_count: 3,
        pr_numbers: vec![12, 14, 15],
    };
    assert!(v1.description().contains("in-flight open PR(s)"));
    assert!(v1.description().contains("main"));

    let v2 = RewritePreconditionViolation::MissingRollbackBackupRef {
        expected_ref_pattern: "backup/*".to_string(),
    };
    assert!(v2.description().contains("Missing local rollback ref"));

    let v3 = RewritePreconditionViolation::DriftCheckNotPerformed;
    assert!(v3.description().contains("diff --stat"));

    let v4 = RewritePreconditionViolation::ContentDriftDetected {
        differing_files_count: 4,
    };
    assert!(v4.description().contains("4 differing file(s)"));
}

#[test]
fn test_serde_roundtrip_rewrite_types() {
    let op = RewriteOperation::published("release/v4.9.0")
        .with_open_prs(vec![42])
        .with_backup_ref("backup/release-v4.9.0")
        .with_drift_status(DriftCheckStatus::ZeroDriftConfirmed);

    let verdict = evaluate_rewrite_safety(&op);

    let serialized_op = serde_json::to_string(&op).expect("Serialize RewriteOperation");
    let deserialized_op: RewriteOperation =
        serde_json::from_str(&serialized_op).expect("Deserialize RewriteOperation");
    assert_eq!(op, deserialized_op);

    let serialized_verdict = serde_json::to_string(&verdict).expect("Serialize RewriteVerdict");
    let deserialized_verdict: RewriteVerdict =
        serde_json::from_str(&serialized_verdict).expect("Deserialize RewriteVerdict");
    assert_eq!(verdict, deserialized_verdict);

    let drift = DriftCheckStatus::DriftDetected {
        differing_files: vec!["fileA.txt".to_string()],
    };
    let serialized_drift = serde_json::to_string(&drift).expect("Serialize DriftCheckStatus");
    let deserialized_drift: DriftCheckStatus =
        serde_json::from_str(&serialized_drift).expect("Deserialize DriftCheckStatus");
    assert_eq!(drift, deserialized_drift);
}
