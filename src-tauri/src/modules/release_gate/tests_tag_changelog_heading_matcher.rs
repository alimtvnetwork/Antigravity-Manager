//! Tests for Tag & Changelog Heading Matcher.

#[cfg(test)]
mod tests {
    use super::super::tag_changelog_heading_matcher::{
        is_prerelease_tag, parse_release_branch, validate_tag_and_heading, ReleaseBranch,
        TagMatchViolation,
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
        assert!(report.is_valid);
        assert_eq!(report.tag, "v4.250.0");
        assert_eq!(report.branch, "main");
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
        assert!(report.is_valid);
        assert_eq!(report.tag, "v4.250.1-beta.1");
        assert_eq!(report.branch, "beta");
        assert_eq!(
            report.matched_heading,
            Some("## [v4.250.1-beta.1] - 2026-10-23".to_string())
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_cross_branch_misplacement_for_stable_tag_on_beta() {
        let changelog = "\
## [v4.250.0] - 2026-10-23
- Stable release notes.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "beta", changelog);
        assert!(!report.is_valid);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            TagMatchViolation::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                assert_eq!(tag, "v4.250.0");
                assert_eq!(branch, "beta");
                assert_eq!(expected_branch, "main");
            }
            _ => panic!("Expected CrossBranchMisplacement violation"),
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
        assert!(!report.is_valid);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            TagMatchViolation::CrossBranchMisplacement {
                tag,
                branch,
                expected_branch,
            } => {
                assert_eq!(tag, "v4.250.0-beta.2");
                assert_eq!(branch, "main");
                assert_eq!(expected_branch, "beta");
            }
            _ => panic!("Expected CrossBranchMisplacement violation"),
        }
    }

    #[test]
    fn test_tag_heading_mismatch_absent_from_changelog() {
        let changelog = "\
# Changelog

## [v4.249.0] - 2026-10-20
- Older release entry.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(!report.is_valid);
        assert!(report.matched_heading.is_none());
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            TagMatchViolation::TagHeadingMismatch {
                tag,
                expected_heading,
                guidance,
            } => {
                assert_eq!(tag, "v4.250.0");
                assert_eq!(expected_heading, "## [v4.250.0]");
                assert!(guidance.contains("v4.250.0"));
            }
            _ => panic!("Expected TagHeadingMismatch violation"),
        }
    }

    #[test]
    fn test_unbracketed_heading_match() {
        let changelog = "\
# Changelog

## v4.250.0 - 2026-10-23
- Stable release with direct unbracketed heading.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(report.is_valid);
        assert_eq!(
            report.matched_heading,
            Some("## v4.250.0 - 2026-10-23".to_string())
        );
    }

    #[test]
    fn test_partial_version_prefix_does_not_falsely_match() {
        let changelog = "\
# Changelog

## [v4.250.00] - 2026-10-23
- Different release entry.
(Thanks to @aukgit)
";
        let report = validate_tag_and_heading("v4.250.0", "main", changelog);
        assert!(!report.is_valid);
        assert!(report.matched_heading.is_none());
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            TagMatchViolation::TagHeadingMismatch { .. }
        ));
    }

    #[test]
    fn test_violation_display_formatting() {
        let heading_mismatch = TagMatchViolation::TagHeadingMismatch {
            tag: "v1.0.0".to_string(),
            expected_heading: "## [v1.0.0]".to_string(),
            guidance: "Add heading to CHANGELOG.md".to_string(),
        };
        let formatted_mismatch = format!("{heading_mismatch}");
        assert!(formatted_mismatch.contains("v1.0.0"));
        assert!(formatted_mismatch.contains("## [v1.0.0]"));

        let cross_branch = TagMatchViolation::CrossBranchMisplacement {
            tag: "v1.0.0".to_string(),
            branch: "beta".to_string(),
            expected_branch: "main".to_string(),
        };
        let formatted_branch = format!("{cross_branch}");
        assert!(formatted_branch.contains("v1.0.0"));
        assert!(formatted_branch.contains("beta"));
        assert!(formatted_branch.contains("main"));

        // Verify std::error::Error trait
        let err: &dyn std::error::Error = &cross_branch;
        assert_eq!(err.to_string(), formatted_branch);
    }

    #[test]
    fn test_parse_release_branch_variations() {
        assert_eq!(parse_release_branch("main"), ReleaseBranch::Main);
        assert_eq!(parse_release_branch("refs/heads/main"), ReleaseBranch::Main);
        assert_eq!(parse_release_branch("origin/main"), ReleaseBranch::Main);
        assert_eq!(parse_release_branch("MAIN"), ReleaseBranch::Main);

        assert_eq!(parse_release_branch("beta"), ReleaseBranch::Beta);
        assert_eq!(parse_release_branch("refs/heads/beta"), ReleaseBranch::Beta);
        assert_eq!(parse_release_branch("origin/beta"), ReleaseBranch::Beta);
        assert_eq!(parse_release_branch("BETA"), ReleaseBranch::Beta);

        assert!(matches!(
            parse_release_branch("feature/release-gate"),
            ReleaseBranch::Other(name) if name == "feature/release-gate"
        ));
    }

    #[test]
    fn test_is_prerelease_tag_variations() {
        assert!(is_prerelease_tag("v1.2.3-beta.0"));
        assert!(is_prerelease_tag("v1.2.3-beta.12"));
        assert!(is_prerelease_tag("v1.2.3-alpha.1"));
        assert!(is_prerelease_tag("v1.2.3-rc.1"));

        assert!(!is_prerelease_tag("v1.2.3"));
        assert!(!is_prerelease_tag("v2.0.0"));
        assert!(!is_prerelease_tag("v1.2.3-final"));
    }
}
