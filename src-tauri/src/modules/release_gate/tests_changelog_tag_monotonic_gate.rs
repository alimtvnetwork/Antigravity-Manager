//! Unit tests for Changelog Tag Monotonic Gate.

#[cfg(test)]
mod tests {
    use super::super::changelog_tag_monotonic_gate::*;

    #[test]
    fn test_parse_semver_tag_success() {
        let stable = parse_semver_tag("v1.2.3").expect("valid stable tag");
        assert_eq!(stable.raw, "v1.2.3");
        assert_eq!(stable.major, 1);
        assert_eq!(stable.minor, 2);
        assert_eq!(stable.patch, 3);
        assert_eq!(stable.prerelease, None);
        assert!(!stable.is_prerelease);

        let beta = parse_semver_tag("v4.250.0-beta.1").expect("valid beta tag");
        assert_eq!(beta.raw, "v4.250.0-beta.1");
        assert_eq!(beta.major, 4);
        assert_eq!(beta.minor, 250);
        assert_eq!(beta.patch, 0);
        assert_eq!(beta.prerelease, Some("beta.1".to_string()));
        assert!(beta.is_prerelease);
    }

    #[test]
    fn test_parse_semver_tag_errors() {
        assert!(parse_semver_tag("1.2.3").is_err());
        assert!(parse_semver_tag("v1.2").is_err());
        assert!(parse_semver_tag("v1.2.x").is_err());
        assert!(parse_semver_tag("v1.2.3-").is_err());
        assert!(parse_semver_tag("").is_err());
    }

    #[test]
    fn test_compare_monotonic_versions_increments() {
        let v1_0_0 = parse_semver_tag("v1.0.0").unwrap();
        let v1_0_1 = parse_semver_tag("v1.0.1").unwrap();
        let v1_1_0 = parse_semver_tag("v1.1.0").unwrap();
        let v2_0_0 = parse_semver_tag("v2.0.0").unwrap();
        let v1_0_0_beta1 = parse_semver_tag("v1.0.0-beta.1").unwrap();
        let v1_0_0_beta2 = parse_semver_tag("v1.0.0-beta.2").unwrap();
        let v1_0_0_beta10 = parse_semver_tag("v1.0.0-beta.10").unwrap();

        // Patch increment
        assert!(compare_monotonic_versions(&v1_0_1, &v1_0_0));
        // Minor increment
        assert!(compare_monotonic_versions(&v1_1_0, &v1_0_1));
        // Major increment
        assert!(compare_monotonic_versions(&v2_0_0, &v1_1_0));
        // Beta increment
        assert!(compare_monotonic_versions(&v1_0_0_beta2, &v1_0_0_beta1));
        // Numeric beta increment (10 > 2)
        assert!(compare_monotonic_versions(&v1_0_0_beta10, &v1_0_0_beta2));
        // Prerelease to stable release
        assert!(compare_monotonic_versions(&v1_0_0, &v1_0_0_beta1));
    }

    #[test]
    fn test_compare_monotonic_versions_rejections() {
        let v1_0_0 = parse_semver_tag("v1.0.0").unwrap();
        let v1_0_1 = parse_semver_tag("v1.0.1").unwrap();
        let v1_0_0_beta1 = parse_semver_tag("v1.0.0-beta.1").unwrap();
        let v1_0_0_beta2 = parse_semver_tag("v1.0.0-beta.2").unwrap();

        // Identical versions rejected
        assert!(!compare_monotonic_versions(&v1_0_0, &v1_0_0));
        assert!(!compare_monotonic_versions(&v1_0_0_beta1, &v1_0_0_beta1));

        // Downgrades rejected
        assert!(!compare_monotonic_versions(&v1_0_0, &v1_0_1));
        assert!(!compare_monotonic_versions(&v1_0_0_beta1, &v1_0_0_beta2));
        assert!(!compare_monotonic_versions(&v1_0_0_beta1, &v1_0_0));
    }

    #[test]
    fn test_validate_stable_release_on_main() {
        let changelog = "\
# Changelog

## [v4.250.0] - 2026-10-23
### Added
- Core architecture enhancements.
";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "main",
            changelog,
            Some("v4.249.0"),
        );
        assert!(report.is_valid);
        assert!(!report.has_violations());
        assert_eq!(report.violation_count(), 0);
        assert_eq!(report.tag, "v4.250.0");
        assert_eq!(report.branch, "main");
    }

    #[test]
    fn test_validate_beta_release_on_beta() {
        let changelog = "\
# Changelog

## [v4.250.1-beta.1] - 2026-10-23
### Features
- Beta preview fixes and telemetry improvements.
";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.1-beta.1",
            "beta",
            changelog,
            Some("v4.250.0"),
        );
        assert!(report.is_valid);
        assert!(!report.has_violations());
        assert_eq!(report.violation_count(), 0);
    }

    #[test]
    fn test_cross_branch_misplacement_beta_on_main() {
        let changelog = "## [v4.250.1-beta.1] - 2026-10-23\n";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.1-beta.1",
            "main",
            changelog,
            Some("v4.250.0"),
        );
        assert!(!report.is_valid);
        assert_eq!(report.violation_count(), 1);
        match &report.violations[0] {
            MonotonicTagViolation::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                assert_eq!(tag, "v4.250.1-beta.1");
                assert_eq!(branch, "main");
                assert_eq!(expected_branch, "beta");
            }
            other => panic!("Expected CrossBranchMisplacement, got {:?}", other),
        }
    }

    #[test]
    fn test_cross_branch_misplacement_stable_on_beta() {
        let changelog = "## [v4.250.0] - 2026-10-23\n";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "beta",
            changelog,
            Some("v4.249.0"),
        );
        assert!(!report.is_valid);
        assert_eq!(report.violation_count(), 1);
        match &report.violations[0] {
            MonotonicTagViolation::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                assert_eq!(tag, "v4.250.0");
                assert_eq!(branch, "beta");
                assert_eq!(expected_branch, "main");
            }
            other => panic!("Expected CrossBranchMisplacement, got {:?}", other),
        }
    }

    #[test]
    fn test_branch_normalization_with_refs_prefix() {
        let changelog = "## [v4.250.0] - 2026-10-23\n";
        let report_main = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "refs/heads/main",
            changelog,
            None,
        );
        assert!(report_main.is_valid);

        let changelog_beta = "## [v4.250.1-beta.1] - 2026-10-23\n";
        let report_beta = validate_tag_against_changelog_and_branch(
            "v4.250.1-beta.1",
            "origin/beta",
            changelog_beta,
            None,
        );
        assert!(report_beta.is_valid);
    }

    #[test]
    fn test_non_monotonic_version_rejection() {
        let changelog = "## [v4.250.0] - 2026-10-23\n";
        // Duplicate version candidate == baseline
        let report_equal = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "main",
            changelog,
            Some("v4.250.0"),
        );
        assert!(!report_equal.is_valid);
        assert!(report_equal.violations.iter().any(|v| matches!(
            v,
            MonotonicTagViolation::NonMonotonicVersion { candidate, baseline }
                if candidate == "v4.250.0" && baseline == "v4.250.0"
        )));

        // Downgrade version candidate < baseline
        let report_downgrade = validate_tag_against_changelog_and_branch(
            "v4.249.0",
            "main",
            changelog,
            Some("v4.250.0"),
        );
        assert!(!report_downgrade.is_valid);
        assert!(report_downgrade.violations.iter().any(|v| matches!(
            v,
            MonotonicTagViolation::NonMonotonicVersion { candidate, baseline }
                if candidate == "v4.249.0" && baseline == "v4.250.0"
        )));
    }

    #[test]
    fn test_changelog_heading_mismatch() {
        let changelog = "## [v4.249.0] - 2026-10-22\n";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "main",
            changelog,
            Some("v4.249.0"),
        );
        assert!(!report.is_valid);
        assert!(report.violations.iter().any(|v| matches!(
            v,
            MonotonicTagViolation::TagHeadingMismatch { tag, expected_heading }
                if tag == "v4.250.0" && expected_heading == "## [v4.250.0]"
        )));
    }

    #[test]
    fn test_changelog_unbracketed_heading() {
        let changelog = "## v4.250.0 - 2026-10-23\n";
        let report = validate_tag_against_changelog_and_branch(
            "v4.250.0",
            "main",
            changelog,
            None,
        );
        assert!(report.is_valid);
    }

    #[test]
    fn test_violation_display_and_error() {
        let heading_err = MonotonicTagViolation::TagHeadingMismatch {
            tag: "v4.250.0".to_string(),
            expected_heading: "## [v4.250.0]".to_string(),
        };
        assert!(heading_err.to_string().contains("does not match changelog heading"));

        let non_mono_err = MonotonicTagViolation::NonMonotonicVersion {
            candidate: "v4.249.0".to_string(),
            baseline: "v4.250.0".to_string(),
        };
        assert!(non_mono_err.to_string().contains("not monotonically greater"));

        let branch_err = MonotonicTagViolation::CrossBranchMisplacement {
            tag: "v4.250.0".to_string(),
            branch: "beta".to_string(),
            expected_branch: "main".to_string(),
        };
        assert!(branch_err.to_string().contains("Cross-branch tag misplacement"));

        use std::error::Error;
        assert!(heading_err.source().is_none());
        assert!(non_mono_err.source().is_none());
        assert!(branch_err.source().is_none());
    }
}
