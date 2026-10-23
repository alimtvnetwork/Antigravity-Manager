//! Unit tests for branch release channel gate (Imp-656).

#[cfg(test)]
mod tests {
    use super::branch_release_channel_gate::*;

    const SAMPLE_CHANGELOG: &str = r#"# Changelog

## [v4.251.0] - 2026-10-23

### Added
- Implemented branch release channel gate.

---

## [v4.250.0-beta.1] - 2026-10-20

### Added
- Beta preview improvements.

---

## [4.249.0] - 2026-10-15

### Fixed
- Legacy format heading without v prefix.
"#;

    #[test]
    fn test_stable_release_on_main_valid() {
        let report = validate_branch_release_channel("v4.251.0", "main", Some(SAMPLE_CHANGELOG));
        assert!(report.is_valid());
        assert!(!report.has_violations());
        assert_eq!(report.channel, BranchReleaseChannel::Stable);
        assert_eq!(
            report.matched_heading.as_deref(),
            Some("## [v4.251.0] - 2026-10-23")
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_stable_release_on_beta_rejected() {
        let report = validate_branch_release_channel("v4.251.0", "beta", Some(SAMPLE_CHANGELOG));
        assert!(!report.is_valid());
        assert!(report.has_violations());
        assert_eq!(report.channel, BranchReleaseChannel::Stable);

        let msgs = report.violation_messages();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("branch 'beta' is forbidden"));
        assert!(msgs[0].contains("Expected 'main'"));
    }

    #[test]
    fn test_stable_release_on_feature_branch_rejected() {
        let report =
            validate_branch_release_channel("v4.251.0", "feature/my-feat", Some(SAMPLE_CHANGELOG));
        assert!(!report.is_valid());
        assert!(report.has_violations());

        let msgs = report.violation_messages();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("branch 'feature/my-feat' is forbidden"));
    }

    #[test]
    fn test_preview_beta_release_on_beta_valid() {
        let report =
            validate_branch_release_channel("v4.250.0-beta.1", "beta", Some(SAMPLE_CHANGELOG));
        assert!(report.is_valid());
        assert_eq!(report.channel, BranchReleaseChannel::Beta);
        assert_eq!(
            report.matched_heading.as_deref(),
            Some("## [v4.250.0-beta.1] - 2026-10-20")
        );
    }

    #[test]
    fn test_preview_beta_release_on_main_rejected() {
        let report =
            validate_branch_release_channel("v4.250.0-beta.1", "main", Some(SAMPLE_CHANGELOG));
        assert!(!report.is_valid());
        assert_eq!(report.channel, BranchReleaseChannel::Beta);

        let msgs = report.violation_messages();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("branch 'main' is forbidden"));
        assert!(msgs[0].contains("Expected 'beta'"));
    }

    #[test]
    fn test_preview_beta_release_on_feature_branch_rejected() {
        let report = validate_branch_release_channel(
            "v4.250.0-beta.1",
            "fix/beta-bug",
            Some(SAMPLE_CHANGELOG),
        );
        assert!(!report.is_valid());

        let msgs = report.violation_messages();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("branch 'fix/beta-bug' is forbidden"));
    }

    #[test]
    fn test_missing_leading_v_prefix() {
        let report = validate_branch_release_channel("4.251.0", "main", None);
        assert!(!report.is_valid());

        match &report.violations[0] {
            BranchReleaseViolation::MissingVPrefix { tag } => {
                assert_eq!(tag, "4.251.0");
            }
            _ => panic!("Expected MissingVPrefix violation"),
        }
    }

    #[test]
    fn test_invalid_semver_format() {
        let report1 = validate_branch_release_channel("v4.abc.0", "main", None);
        assert!(!report1.is_valid());
        match &report1.violations[0] {
            BranchReleaseViolation::InvalidSemVerFormat { tag, reason } => {
                assert_eq!(tag, "v4.abc.0");
                assert!(reason.contains("numeric digits"));
            }
            _ => panic!("Expected InvalidSemVerFormat"),
        }

        let report2 = validate_branch_release_channel("v4.250.0-beta.xyz", "beta", None);
        assert!(!report2.is_valid());
        match &report2.violations[0] {
            BranchReleaseViolation::InvalidSemVerFormat { tag, reason } => {
                assert_eq!(tag, "v4.250.0-beta.xyz");
                assert!(reason.contains("must be numeric after '-beta.'"));
            }
            _ => panic!("Expected InvalidSemVerFormat for beta suffix"),
        }
    }

    #[test]
    fn test_changelog_heading_exact_match_character_for_character() {
        let result = find_changelog_heading_exact(SAMPLE_CHANGELOG, "v4.251.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "## [v4.251.0] - 2026-10-23");
    }

    #[test]
    fn test_changelog_heading_character_mismatch_without_v() {
        // SAMPLE_CHANGELOG has "## [4.249.0] - 2026-10-15" (missing 'v')
        let result = find_changelog_heading_exact(SAMPLE_CHANGELOG, "v4.249.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            BranchReleaseViolation::ChangelogHeadingCharacterMismatch {
                tag,
                expected_heading,
                actual_heading,
            } => {
                assert_eq!(tag, "v4.249.0");
                assert_eq!(expected_heading, "## [v4.249.0]");
                assert_eq!(actual_heading, "## [4.249.0] - 2026-10-15");
            }
            _ => panic!("Expected ChangelogHeadingCharacterMismatch"),
        }
    }

    #[test]
    fn test_changelog_heading_not_found() {
        let result = find_changelog_heading_exact(SAMPLE_CHANGELOG, "v9.999.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            BranchReleaseViolation::ChangelogHeadingNotFound {
                tag,
                expected_heading,
            } => {
                assert_eq!(tag, "v9.999.0");
                assert_eq!(expected_heading, "## [v9.999.0]");
            }
            _ => panic!("Expected ChangelogHeadingNotFound"),
        }
    }

    #[test]
    fn test_branch_ref_normalization() {
        let b1 = normalize_git_branch("refs/heads/main");
        assert_eq!(b1, ReleaseGateBranch::Main);

        let b2 = normalize_git_branch("origin/main");
        assert_eq!(b2, ReleaseGateBranch::Main);

        let b3 = normalize_git_branch("refs/heads/beta");
        assert_eq!(b3, ReleaseGateBranch::Beta);

        let b4 = normalize_git_branch("origin/beta");
        assert_eq!(b4, ReleaseGateBranch::Beta);

        let b5 = normalize_git_branch("refs/heads/feature/awesome");
        assert_eq!(
            b5,
            ReleaseGateBranch::Feature("feature/awesome".to_string())
        );
    }

    #[test]
    fn test_unbracketed_changelog_heading_match() {
        let unbracketed_changelog = "## v4.255.0 - 2026-10-25\n- Unbracketed release.";
        let result = find_changelog_heading_exact(unbracketed_changelog, "v4.255.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "## v4.255.0 - 2026-10-25");
    }

    #[test]
    fn test_validate_branch_release_channel_file_io_error() {
        let non_existent_path = "non_existent_changelog_file_9999.md";
        let result =
            validate_branch_release_channel_file("v4.251.0", "main", non_existent_path);
        assert!(result.is_err());
    }
}
