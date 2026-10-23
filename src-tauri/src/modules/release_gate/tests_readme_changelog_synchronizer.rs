#[cfg(test)]
mod tests {
    use super::super::readme_changelog_synchronizer::{
        parse_release_tag_type, verify_readme_changelog_sync, ReadmeSyncViolation, ReleaseTagType,
    };

    #[test]
    fn test_parse_release_tag_type_stable() {
        assert_eq!(
            parse_release_tag_type("v4.251.0"),
            ReleaseTagType::Stable("v4.251.0".to_string())
        );
        assert_eq!(
            parse_release_tag_type("4.251.0"),
            ReleaseTagType::Stable("4.251.0".to_string())
        );
    }

    #[test]
    fn test_parse_release_tag_type_beta() {
        assert_eq!(
            parse_release_tag_type("v4.251.0-beta.1"),
            ReleaseTagType::PreReleaseBeta("v4.251.0-beta.1".to_string())
        );
        assert_eq!(
            parse_release_tag_type("v4.251.0-beta"),
            ReleaseTagType::PreReleaseBeta("v4.251.0-beta".to_string())
        );
    }

    #[test]
    fn test_parse_release_tag_type_alpha_and_rc() {
        assert_eq!(
            parse_release_tag_type("v1.0.0-alpha.1"),
            ReleaseTagType::PreReleaseBeta("v1.0.0-alpha.1".to_string())
        );
        assert_eq!(
            parse_release_tag_type("v2.0.0-rc.1"),
            ReleaseTagType::PreReleaseBeta("v2.0.0-rc.1".to_string())
        );
    }

    #[test]
    fn test_release_tag_type_helper_methods() {
        let stable = ReleaseTagType::Stable("v4.251.0".to_string());
        assert_eq!(stable.tag(), "v4.251.0");
        assert!(stable.is_stable());
        assert!(!stable.is_pre_release());

        let prerelease = ReleaseTagType::PreReleaseBeta("v4.251.0-beta.1".to_string());
        assert_eq!(prerelease.tag(), "v4.251.0-beta.1");
        assert!(!prerelease.is_stable());
        assert!(prerelease.is_pre_release());
    }

    #[test]
    fn test_stable_release_fully_synchronized() {
        let tag = "v4.251.0";
        let readme_zh = "# Antigravity Manager\n## 📝 更新日志\n### v4.251.0\n- New feature added";
        let readme_en = "# Antigravity Manager\n## 📝 Changelog\n### v4.251.0\n- New feature added";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(report.is_synchronized);
        assert_eq!(report.tag_type, ReleaseTagType::Stable("v4.251.0".to_string()));
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_stable_release_version_without_v_matches() {
        let tag = "v4.251.0";
        let readme_zh = "## 📝 更新日志\nRelease 4.251.0 notes";
        let readme_en = "## 📝 Changelog\nRelease 4.251.0 notes";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(report.is_synchronized);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_stable_release_missing_zh() {
        let tag = "v4.251.0";
        let readme_zh = "## 📝 更新日志\nOld notes only";
        let readme_en = "## 📝 Changelog\n### v4.251.0\n- New feature added";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::StableReleaseMissingReadmeSummary {
                tag: "v4.251.0".to_string(),
                missing_file: "README.md".to_string(),
            }
        );
    }

    #[test]
    fn test_stable_release_missing_en() {
        let tag = "v4.251.0";
        let readme_zh = "## 📝 更新日志\n### v4.251.0\n- Chinese summary";
        let readme_en = "## 📝 Changelog\nOld English notes only";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::StableReleaseMissingReadmeSummary {
                tag: "v4.251.0".to_string(),
                missing_file: "README_EN.md".to_string(),
            }
        );
    }

    #[test]
    fn test_stable_release_missing_both() {
        let tag = "v4.251.0";
        let readme_zh = "## 📝 更新日志\nv4.250.0 notes";
        let readme_en = "## 📝 Changelog\nv4.250.0 notes";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 2);
    }

    #[test]
    fn test_prerelease_clean_changelog_only() {
        let tag = "v4.251.0-beta.1";
        let readme_zh = "## 📝 更新日志\n### v4.250.0\n- Chinese summary";
        let readme_en = "## 📝 Changelog\n### v4.250.0\n- English summary";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(report.is_synchronized);
        assert_eq!(
            report.tag_type,
            ReleaseTagType::PreReleaseBeta("v4.251.0-beta.1".to_string())
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_prerelease_improperly_placed_in_zh() {
        let tag = "v4.251.0-beta.1";
        let readme_zh = "## 📝 更新日志\n### v4.251.0-beta.1\n- Beta leak";
        let readme_en = "## 📝 Changelog\n### v4.250.0\n- English summary";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::PreReleaseImproperlyPlacedInReadme {
                tag: "v4.251.0-beta.1".to_string(),
                found_in_file: "README.md".to_string(),
            }
        );
    }

    #[test]
    fn test_prerelease_improperly_placed_in_en() {
        let tag = "v4.251.0-beta.1";
        let readme_zh = "## 📝 更新日志\n### v4.250.0\n- Chinese summary";
        let readme_en = "## 📝 Changelog\n### v4.251.0-beta.1\n- Beta leak";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::PreReleaseImproperlyPlacedInReadme {
                tag: "v4.251.0-beta.1".to_string(),
                found_in_file: "README_EN.md".to_string(),
            }
        );
    }

    #[test]
    fn test_prerelease_improperly_placed_in_both() {
        let tag = "v4.251.0-beta.1";
        let readme_zh = "## 📝 更新日志\n### v4.251.0-beta.1\n- Beta leak";
        let readme_en = "## 📝 Changelog\n### v4.251.0-beta.1\n- Beta leak";

        let report = verify_readme_changelog_sync(tag, readme_zh, readme_en);
        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 2);
    }

    #[test]
    fn test_violation_display_formatting() {
        let v1 = ReadmeSyncViolation::StableReleaseMissingReadmeSummary {
            tag: "v4.251.0".to_string(),
            missing_file: "README.md".to_string(),
        };
        assert_eq!(
            v1.to_string(),
            "Stable release v4.251.0 is missing required release summary in README.md"
        );

        let v2 = ReadmeSyncViolation::PreReleaseImproperlyPlacedInReadme {
            tag: "v4.251.0-beta.1".to_string(),
            found_in_file: "README_EN.md".to_string(),
        };
        assert_eq!(
            v2.to_string(),
            "Pre-release v4.251.0-beta.1 should not be included in README_EN.md (exclusive to changelog)"
        );
    }
}
