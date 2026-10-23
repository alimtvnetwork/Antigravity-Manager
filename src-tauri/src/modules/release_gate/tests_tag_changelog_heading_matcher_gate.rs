//! Tests for Tag & Changelog Heading Exact Matcher Gate (Imp-632).

#[cfg(test)]
mod tests {
    use crate::modules::release_gate::tag_changelog_heading_matcher_gate::{
        find_matching_changelog_heading, is_beta_prerelease, normalize_branch,
        validate_semver_format, validate_tag_and_heading, ReleaseBranch, TagMatchingReport,
        TagMatchingViolation,
    };

    #[test]
    fn test_valid_stable_release_matching_on_main() {
        let changelog = "\
# Changelog

## [v4.250.0] - 2026-10-23
### Added
- Core architecture enhancements.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(report.is_compliant);
        assert!(!report.has_violations());
        assert_eq!(report.violation_count(), 0);
        assert_eq!(report.tag, "v4.250.0");
        assert_eq!(report.target_branch, "main");
        assert!(!report.is_beta_prerelease);
        assert_eq!(
            report.matched_heading,
            Some("## [v4.250.0] - 2026-10-23".to_string())
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_valid_beta_release_matching_on_beta() {
        let changelog = "\
# Changelog

## [v4.250.1-beta.1] - 2026-10-23
### Features
- Beta preview fixes and telemetry improvements.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.1-beta.1", "beta", changelog);
        assert!(report.is_compliant);
        assert!(!report.has_violations());
        assert_eq!(report.violation_count(), 0);
        assert_eq!(report.tag, "v4.250.1-beta.1");
        assert_eq!(report.target_branch, "beta");
        assert!(report.is_beta_prerelease);
        assert_eq!(
            report.matched_heading,
            Some("## [v4.250.1-beta.1] - 2026-10-23".to_string())
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_missing_leading_v_prefix_flagged() {
        let changelog = "\
## [4.250.0] - 2026-10-23
- Tag without v prefix.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("4.250.0", "main", changelog);
        assert!(!report.is_compliant);
        assert!(report.has_violations());
        assert!(report
            .violations
            .iter()
            .any(|v| matches!(v, TagMatchingViolation::MissingLeadingVPrefix { tag } if tag == "4.250.0")));
    }

    #[test]
    fn test_cross_branch_misplacement_for_stable_tag_on_beta() {
        let changelog = "\
## [v4.250.0] - 2026-10-23
- Stable release notes.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "beta", changelog);
        assert!(!report.is_compliant);
        assert_eq!(report.violation_count(), 1);

        match &report.violations[0] {
            TagMatchingViolation::CrossBranchTagMisplacement {
                tag,
                target_branch,
                guidance,
            } => {
                assert_eq!(tag, "v4.250.0");
                assert_eq!(target_branch, "beta");
                assert!(guidance.contains("main"));
            }
            other => panic!("Expected CrossBranchTagMisplacement, got: {:?}", other),
        }
    }

    #[test]
    fn test_cross_branch_misplacement_for_beta_tag_on_main() {
        let changelog = "\
## [v4.250.0-beta.2] - 2026-10-23
- Beta pre-release notes.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0-beta.2", "main", changelog);
        assert!(!report.is_compliant);
        assert_eq!(report.violation_count(), 1);

        match &report.violations[0] {
            TagMatchingViolation::CrossBranchTagMisplacement {
                tag,
                target_branch,
                guidance,
            } => {
                assert_eq!(tag, "v4.250.0-beta.2");
                assert_eq!(target_branch, "main");
                assert!(guidance.contains("beta"));
            }
            other => panic!("Expected CrossBranchTagMisplacement, got: {:?}", other),
        }
    }

    #[test]
    fn test_mismatched_heading_in_changelog_flagged() {
        let changelog = "\
## [v4.249.0] - 2026-10-22
- Old version notes.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(!report.is_compliant);
        assert!(report.matched_heading.is_none());

        assert!(report.violations.iter().any(|v| matches!(
            v,
            TagMatchingViolation::MismatchedChangelogHeading { tag, expected_heading }
                if tag == "v4.250.0" && expected_heading == "## [v4.250.0]"
        )));
    }

    #[test]
    fn test_unbracketed_heading_match() {
        let changelog = "\
## v4.250.0 - 2026-10-23
- Unbracketed heading format.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(report.is_compliant);
        assert_eq!(
            report.matched_heading,
            Some("## v4.250.0 - 2026-10-23".to_string())
        );
    }

    #[test]
    fn test_branch_normalization_with_refs_and_origin() {
        assert_eq!(normalize_branch("refs/heads/main"), ReleaseBranch::Main);
        assert_eq!(normalize_branch("refs/heads/beta"), ReleaseBranch::Beta);
        assert_eq!(normalize_branch("origin/main"), ReleaseBranch::Main);
        assert_eq!(normalize_branch("origin/beta"), ReleaseBranch::Beta);
        assert_eq!(
            normalize_branch("refs/heads/feature/gate"),
            ReleaseBranch::Other("feature/gate".to_string())
        );
    }

    #[test]
    fn test_invalid_semver_formats() {
        assert!(validate_semver_format("v4.invalid").is_err());
        assert!(validate_semver_format("v4.250").is_err());
        assert!(validate_semver_format("v4.250.0-beta.xyz").is_err());
        assert!(validate_semver_format("v4.250.0-beta.").is_err());
        assert!(validate_semver_format("v4.250.0").is_ok());
        assert!(validate_semver_format("v4.250.0-beta.1").is_ok());

        let changelog = "## [v4.invalid] - 2026-10-23";
        let report = validate_tag_and_heading("v4.invalid", "main", changelog);
        assert!(!report.is_compliant);
        assert!(report
            .violations
            .iter()
            .any(|v| matches!(v, TagMatchingViolation::InvalidSemVerFormat { .. })));
    }

    #[test]
    fn test_is_beta_prerelease() {
        assert!(!is_beta_prerelease("v4.250.0"));
        assert!(is_beta_prerelease("v4.250.0-beta.1"));
        assert!(is_beta_prerelease("v4.250.0-beta.12"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let original_report = TagMatchingReport {
            is_compliant: false,
            tag: "v4.250.0-beta.1".to_string(),
            target_branch: "main".to_string(),
            matched_heading: Some("## [v4.250.0-beta.1] - 2026-10-23".to_string()),
            violations: vec![TagMatchingViolation::CrossBranchTagMisplacement {
                tag: "v4.250.0-beta.1".to_string(),
                target_branch: "main".to_string(),
                guidance: "Beta pre-release tags must be placed on 'beta'".to_string(),
            }],
            is_beta_prerelease: true,
        };

        let json = serde_json::to_string(&original_report).expect("Serialization failed");
        let deserialized: TagMatchingReport =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(original_report, deserialized);
    }
}
