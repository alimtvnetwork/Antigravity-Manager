//! Unit tests for rollback reference and Git history drift verifier.

#[cfg(test)]
mod tests {
    use super::super::rollback_reference_drift_verifier::{
        parse_git_diff_stat, verify_history_rewrite_safety, RollbackRefViolation,
    };

    #[test]
    fn test_safe_history_rewrite_all_criteria_met() {
        let existing_refs = vec![
            "refs/heads/main".to_string(),
            "backup/feat-task-204-1729660000".to_string(),
        ];
        let diff_stat = "";
        let open_prs = Vec::new();

        let report = verify_history_rewrite_safety("main", &existing_refs, diff_stat, &open_prs);

        assert!(report.is_safe);
        assert_eq!(report.target_branch, "main");
        assert_eq!(
            report.backup_ref,
            Some("backup/feat-task-204-1729660000".to_string())
        );
        assert!(report.diff_stat.is_zero_drift);
        assert_eq!(report.open_pr_count, 0);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_safe_history_rewrite_refs_heads_backup() {
        let existing_refs = vec!["refs/heads/backup/v4-hotfix".to_string()];
        let diff_stat = "0 files changed, 0 insertions(+), 0 deletions(-)";
        let open_prs = Vec::new();

        let report =
            verify_history_rewrite_safety("release/v4", &existing_refs, diff_stat, &open_prs);

        assert!(report.is_safe);
        assert_eq!(
            report.backup_ref,
            Some("refs/heads/backup/v4-hotfix".to_string())
        );
        assert!(report.diff_stat.is_zero_drift);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_missing_backup_ref_violation() {
        let existing_refs = vec![
            "refs/heads/main".to_string(),
            "refs/heads/feature-auth".to_string(),
        ];
        let diff_stat = "";
        let open_prs = Vec::new();

        let report = verify_history_rewrite_safety("main", &existing_refs, diff_stat, &open_prs);

        assert!(!report.is_safe);
        assert_eq!(report.backup_ref, None);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            RollbackRefViolation::MissingBackupReference {
                expected_prefix,
                target_branch,
            } => {
                assert_eq!(expected_prefix, "backup/");
                assert_eq!(target_branch, "main");
            }
            _ => panic!("Expected MissingBackupReference violation"),
        }
    }

    #[test]
    fn test_content_drift_violation() {
        let existing_refs = vec!["backup/main-checkpoint".to_string()];
        let diff_stat = "3 files changed, 14 insertions(+), 6 deletions(-)";
        let open_prs = Vec::new();

        let report = verify_history_rewrite_safety("main", &existing_refs, diff_stat, &open_prs);

        assert!(!report.is_safe);
        assert!(!report.diff_stat.is_zero_drift);
        assert_eq!(report.diff_stat.files_changed, 3);
        assert_eq!(report.diff_stat.insertions, 14);
        assert_eq!(report.diff_stat.deletions, 6);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            RollbackRefViolation::ContentDriftDetected {
                files_changed,
                insertions,
                deletions,
                diff_stat: recorded_stat,
            } => {
                assert_eq!(*files_changed, 3);
                assert_eq!(*insertions, 14);
                assert_eq!(*deletions, 6);
                assert!(recorded_stat.contains("3 files changed"));
            }
            _ => panic!("Expected ContentDriftDetected violation"),
        }
    }

    #[test]
    fn test_in_flight_prs_violation() {
        let existing_refs = vec!["backup/beta-checkpoint".to_string()];
        let diff_stat = "";
        let open_prs = vec![
            "PR #120: Fix dark mode flicker".to_string(),
            "PR #125: Add token budget limiter".to_string(),
        ];

        let report = verify_history_rewrite_safety("beta", &existing_refs, diff_stat, &open_prs);

        assert!(!report.is_safe);
        assert_eq!(report.open_pr_count, 2);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            RollbackRefViolation::InFlightPrsExist {
                target_branch,
                open_prs: prs,
            } => {
                assert_eq!(target_branch, "beta");
                assert_eq!(prs.len(), 2);
            }
            _ => panic!("Expected InFlightPrsExist violation"),
        }
    }

    #[test]
    fn test_multiple_violations_aggregated() {
        let existing_refs = vec!["refs/heads/feature-orphan".to_string()];
        let diff_stat = "1 file changed, 2 insertions(+)";
        let open_prs = vec!["PR #99: In-flight PR".to_string()];

        let report = verify_history_rewrite_safety("main", &existing_refs, diff_stat, &open_prs);

        assert!(!report.is_safe);
        assert_eq!(report.violations.len(), 3);
    }

    #[test]
    fn test_parse_git_diff_stat_formats() {
        // Empty string
        let empty = parse_git_diff_stat("");
        assert!(empty.is_zero_drift);
        assert_eq!(empty.files_changed, 0);

        // Zero changes explicitly stated
        let zero = parse_git_diff_stat(" 0 files changed");
        assert!(zero.is_zero_drift);
        assert_eq!(zero.files_changed, 0);

        // Single insertion only
        let ins_only = parse_git_diff_stat(" 1 file changed, 10 insertions(+)");
        assert!(!ins_only.is_zero_drift);
        assert_eq!(ins_only.files_changed, 1);
        assert_eq!(ins_only.insertions, 10);
        assert_eq!(ins_only.deletions, 0);

        // Single deletion only
        let del_only = parse_git_diff_stat(" 2 files changed, 5 deletions(-)");
        assert!(!del_only.is_zero_drift);
        assert_eq!(del_only.files_changed, 2);
        assert_eq!(del_only.insertions, 0);
        assert_eq!(del_only.deletions, 5);

        // Multi-line full git diff stat output
        let multi_line = r#"
 src/components/Header.tsx | 12 +++++++++---
 src/utils/format.ts       |  4 ++--
 2 files changed, 11 insertions(+), 5 deletions(-)
"#;
        let summary = parse_git_diff_stat(multi_line);
        assert!(!summary.is_zero_drift);
        assert_eq!(summary.files_changed, 2);
        assert_eq!(summary.insertions, 11);
        assert_eq!(summary.deletions, 5);
    }

    #[test]
    fn test_violation_display_formatting() {
        let v1 = RollbackRefViolation::MissingBackupReference {
            expected_prefix: "backup/".to_string(),
            target_branch: "main".to_string(),
        };
        assert!(v1.to_string().contains("Missing backup reference"));
        assert!(v1.to_string().contains("backup/"));
        assert!(v1.to_string().contains("main"));

        let v2 = RollbackRefViolation::ContentDriftDetected {
            diff_stat: "1 file changed".to_string(),
            files_changed: 1,
            insertions: 5,
            deletions: 2,
        };
        assert!(v2.to_string().contains("Content drift detected"));
        assert!(v2.to_string().contains("1 files changed, +5 -2"));

        let v3 = RollbackRefViolation::InFlightPrsExist {
            target_branch: "release".to_string(),
            open_prs: vec!["PR #1".to_string()],
        };
        assert!(v3.to_string().contains("In-flight PRs exist"));
        assert!(v3.to_string().contains("release"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let violation = RollbackRefViolation::MissingBackupReference {
            expected_prefix: "backup/".to_string(),
            target_branch: "main".to_string(),
        };
        let err: &dyn std::error::Error = &violation;
        assert!(err.to_string().contains("Missing backup reference"));
    }
}
