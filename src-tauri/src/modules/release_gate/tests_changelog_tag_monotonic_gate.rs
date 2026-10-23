//! Unit tests for Changelog Tag Monotonic Gate (Imp-662).

#[cfg(test)]
mod tests {
    use super::super::changelog_tag_monotonic_gate::*;
    use std::fs;

    #[test]
    fn test_parse_semver_tag_valid_stable_and_beta() {
        let stable = parse_semver_tag("v1.2.3").expect("valid stable tag");
        assert_eq!(stable.raw, "v1.2.3");
        assert_eq!(stable.major, 1);
        assert_eq!(stable.minor, 2);
        assert_eq!(stable.patch, 3);
        assert_eq!(stable.prerelease, None);
        assert!(!stable.is_prerelease());
        assert!(stable.is_stable());
        assert_eq!(stable.release_channel(), ReleaseChannelType::Stable);

        let beta = parse_semver_tag("v4.250.0-beta.1").expect("valid beta tag");
        assert_eq!(beta.raw, "v4.250.0-beta.1");
        assert_eq!(beta.major, 4);
        assert_eq!(beta.minor, 250);
        assert_eq!(beta.patch, 0);
        assert_eq!(beta.prerelease, Some("beta.1".to_string()));
        assert!(beta.is_prerelease());
        assert!(!beta.is_stable());
        assert_eq!(beta.release_channel(), ReleaseChannelType::Beta);
    }

    #[test]
    fn test_parse_semver_tag_errors_missing_v_and_malformed() {
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
        // Numeric beta comparison (10 > 2)
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
    fn test_exact_character_matching_changelog_headings() {
        let changelog = "\
# Changelog

## [v4.8.2] - 2026-10-23
### Added
- Feature details

## v4.8.1 - 2026-10-20
### Fixed
- Patch fixes

## [v4.8.0-beta.1] - 2026-10-15
### Beta
- Initial preview
";
        assert!(changelog_contains_exact_heading(changelog, "v4.8.2"));
        assert!(changelog_contains_exact_heading(changelog, "v4.8.1"));
        assert!(changelog_contains_exact_heading(changelog, "v4.8.0-beta.1"));
        assert!(!changelog_contains_exact_heading(changelog, "4.8.2"));
        assert!(!changelog_contains_exact_heading(changelog, "v4.8.0"));
    }

    #[test]
    fn test_mismatched_heading_rejection() {
        // Missing 'v' prefix in heading
        let cl_missing_v = "## [4.8.2] - 2026-10-23\n";
        let report_no_v =
            validate_tag_monotonic_progression("v4.8.2", "main", cl_missing_v, Some("v4.8.1"));
        assert!(!report_no_v.is_valid());
        assert!(!report_no_v.has_character_match());
        assert!(report_no_v
            .failure_reasons
            .iter()
            .any(|r| r.contains("does not match changelog heading")));

        // Missing '-beta' suffix in heading
        let cl_missing_beta = "## [v4.8.2] - 2026-10-23\n";
        let report_no_beta = validate_tag_monotonic_progression(
            "v4.8.2-beta.1",
            "beta",
            cl_missing_beta,
            Some("v4.8.1"),
        );
        assert!(!report_no_beta.is_valid());
        assert!(!report_no_beta.has_character_match());

        // Extra '-beta' suffix in heading when candidate is stable
        let cl_extra_beta = "## [v4.8.2-beta.1] - 2026-10-23\n";
        let report_extra_beta =
            validate_tag_monotonic_progression("v4.8.2", "main", cl_extra_beta, Some("v4.8.1"));
        assert!(!report_extra_beta.is_valid());
        assert!(!report_extra_beta.has_character_match());
    }

    #[test]
    fn test_monotonic_version_progression_verification() {
        let changelog = "## [v4.8.2] - 2026-10-23\n";
        let report =
            validate_tag_monotonic_progression("v4.8.2", "main", changelog, Some("v4.8.1"));
        assert!(report.is_valid());
        assert!(report.has_character_match());
        assert!(report.is_monotonic());
        assert!(report.is_channel_compliant());
        assert_eq!(report.failure_count(), 0);
        assert_eq!(report.candidate_tag, "v4.8.2");
        assert_eq!(report.latest_historical_tag, Some("v4.8.1".to_string()));

        // Verification with no baseline tag (first release)
        let report_no_hist = validate_tag_monotonic_progression("v4.8.2", "main", changelog, None);
        assert!(report_no_hist.is_valid());
        assert!(report_no_hist.is_monotonic());
    }

    #[test]
    fn test_non_monotonic_version_regression_interception() {
        let changelog = "## [v4.8.0] - 2026-10-23\n";
        // Downgrade candidate < historical
        let report_downgrade =
            validate_tag_monotonic_progression("v4.8.0", "main", changelog, Some("v4.8.1"));
        assert!(!report_downgrade.is_valid());
        assert!(!report_downgrade.is_monotonic());
        assert!(report_downgrade
            .failure_reasons
            .iter()
            .any(|r| r.contains("not monotonically greater")));

        // Duplicate candidate == historical
        let changelog_dup = "## [v4.8.1] - 2026-10-23\n";
        let report_dup =
            validate_tag_monotonic_progression("v4.8.1", "main", changelog_dup, Some("v4.8.1"));
        assert!(!report_dup.is_valid());
        assert!(!report_dup.is_monotonic());
    }

    #[test]
    fn test_stable_tag_on_main_vs_beta() {
        let changelog = "## [v4.8.2] - 2026-10-23\n";

        // Allowed on main
        let report_main =
            validate_tag_monotonic_progression("v4.8.2", "main", changelog, Some("v4.8.1"));
        assert!(report_main.is_valid());
        assert!(report_main.is_channel_compliant());

        // Allowed with refs prefix
        let report_refs = validate_tag_monotonic_progression(
            "v4.8.2",
            "refs/heads/main",
            changelog,
            Some("v4.8.1"),
        );
        assert!(report_refs.is_valid());
        assert!(report_refs.is_channel_compliant());

        // Rejected on beta
        let report_beta =
            validate_tag_monotonic_progression("v4.8.2", "beta", changelog, Some("v4.8.1"));
        assert!(!report_beta.is_valid());
        assert!(!report_beta.is_channel_compliant());
        assert!(report_beta
            .failure_reasons
            .iter()
            .any(|r| r.contains("Cross-branch tag misplacement")));

        // Rejected on feature branch
        let report_feature = validate_tag_monotonic_progression(
            "v4.8.2",
            "feature/new-gate",
            changelog,
            Some("v4.8.1"),
        );
        assert!(!report_feature.is_valid());
        assert!(!report_feature.is_channel_compliant());
    }

    #[test]
    fn test_beta_tag_on_beta_vs_main() {
        let changelog = "## [v4.8.2-beta.1] - 2026-10-23\n";

        // Allowed on beta
        let report_beta =
            validate_tag_monotonic_progression("v4.8.2-beta.1", "beta", changelog, Some("v4.8.1"));
        assert!(report_beta.is_valid());
        assert!(report_beta.is_channel_compliant());

        // Allowed with origin prefix
        let report_origin = validate_tag_monotonic_progression(
            "v4.8.2-beta.1",
            "origin/beta",
            changelog,
            Some("v4.8.1"),
        );
        assert!(report_origin.is_valid());
        assert!(report_origin.is_channel_compliant());

        // Rejected on main
        let report_main =
            validate_tag_monotonic_progression("v4.8.2-beta.1", "main", changelog, Some("v4.8.1"));
        assert!(!report_main.is_valid());
        assert!(!report_main.is_channel_compliant());
        assert!(report_main
            .failure_reasons
            .iter()
            .any(|r| r.contains("Cross-branch tag misplacement")));
    }

    #[test]
    fn test_gate_struct_evaluate_and_instance() {
        let changelog = "## [v4.8.2] - 2026-10-23\n";
        let gate = ChangelogTagMonotonicGate::new();

        let report1 = gate.validate("v4.8.2", "main", changelog, Some("v4.8.1"));
        assert!(report1.is_valid());

        let report2 =
            ChangelogTagMonotonicGate::evaluate("v4.8.2", "main", changelog, Some("v4.8.1"));
        assert!(report2.is_valid());
        assert_eq!(report1, report2);
    }

    #[test]
    fn test_report_serialization_and_deserialization() {
        let report = TagMonotonicValidationReport {
            is_valid: true,
            has_character_match: true,
            is_monotonic: true,
            is_channel_compliant: true,
            candidate_tag: "v4.8.2".to_string(),
            latest_historical_tag: Some("v4.8.1".to_string()),
            failure_reasons: Vec::new(),
        };

        let json = serde_json::to_string(&report).expect("serializable");
        let restored: TagMonotonicValidationReport =
            serde_json::from_str(&json).expect("deserializable");
        assert_eq!(report, restored);
        assert!(restored.is_valid());
        assert_eq!(restored.tag(), "v4.8.2");
    }

    #[test]
    fn test_validate_file_missing_and_existing() {
        let gate = ChangelogTagMonotonicGate::new();
        let non_existent = "target/test_missing_changelog_nonexistent.md";
        let result_err = gate.validate_file("v4.8.2", "main", non_existent, None);
        assert!(result_err.is_err());

        let tmp_path = std::env::temp_dir().join("test_changelog_tag_monotonic_gate.md");
        fs::write(&tmp_path, "## [v4.8.2] - 2026-10-23\n").expect("write temp file");

        let result_ok = gate.validate_file("v4.8.2", "main", &tmp_path, Some("v4.8.1"));
        assert!(result_ok.is_ok());
        let report = result_ok.unwrap();
        assert!(report.is_valid());

        // Justification: Best-effort test artifact cleanup must record any ignored failure.
        crate::error::record_ignored(fs::remove_file(&tmp_path), "clean up test changelog file");
    }
}
