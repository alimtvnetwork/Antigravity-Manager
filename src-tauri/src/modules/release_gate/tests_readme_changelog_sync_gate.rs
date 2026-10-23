//! Tests for README Changelog Synchronization Gate.

#[cfg(test)]
mod tests {
    use super::super::readme_changelog_sync_gate::{
        contains_version, extract_section, extract_version,
        verify_readme_changelog_synchronization, version_matches, ReadmeSyncReport,
        ReadmeSyncViolation, ReleaseChannel, CHINESE_CHANGELOG_HEADER, ENGLISH_CHANGELOG_HEADER,
    };

    #[test]
    fn test_stable_release_fully_synchronized_passes_clean() {
        let target_version = "v4.251.0";
        let changelog_summary = "### [v4.251.0] - 2026-10-23\n- Feature notes for v4.251.0";
        let readme_zh = "\
# Antigravity Manager
## 📝 更新日志
### v4.251.0 (2026-10-23)
- 增加了发布门禁检查逻辑
## 🤝 贡献者
";
        let readme_en = "\
# Antigravity Manager
## 📝 Changelog
### v4.251.0 (2026-10-23)
- Added release gate verification logic
## 🤝 Contributing
";

        let report = verify_readme_changelog_synchronization(
            target_version,
            changelog_summary,
            readme_zh,
            readme_en,
            &ReleaseChannel::Stable,
        );

        assert!(report.is_synchronized);
        assert!(!report.is_exempt_from_readme);
        assert_eq!(report.target_version, "v4.251.0");
        assert_eq!(report.channel, ReleaseChannel::Stable);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_stable_release_missing_chinese_readme_update_flagged() {
        let target_version = "v4.251.0";
        let changelog_summary = "### [v4.251.0]\n- New release notes";
        let readme_zh = "\
# Antigravity Manager
## 📝 更新日志
### v4.250.0 (2026-10-20)
- 旧版本说明
## 🤝 贡献者
";
        let readme_en = "\
# Antigravity Manager
## 📝 Changelog
### v4.251.0 (2026-10-23)
- Added new release notes
## 🤝 Contributing
";

        let report = verify_readme_changelog_synchronization(
            target_version,
            changelog_summary,
            readme_zh,
            readme_en,
            &ReleaseChannel::Stable,
        );

        assert!(!report.is_synchronized);
        assert!(!report.is_exempt_from_readme);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::MissingReleaseSummaryInReadme {
                readme_lang: "zh".to_string(),
                expected_version: "v4.251.0".to_string(),
            }
        );
    }

    #[test]
    fn test_stable_release_missing_english_readme_update_flagged() {
        let target_version = "v4.251.0";
        let changelog_summary = "### [v4.251.0]\n- New release notes";
        let readme_zh = "\
# Antigravity Manager
## 📝 更新日志
### v4.251.0 (2026-10-23)
- 新版本说明
## 🤝 贡献者
";
        let readme_en = "\
# Antigravity Manager
## 📝 Changelog
### v4.250.0 (2026-10-20)
- Outdated English notes
## 🤝 Contributing
";

        let report = verify_readme_changelog_synchronization(
            target_version,
            changelog_summary,
            readme_zh,
            readme_en,
            &ReleaseChannel::Stable,
        );

        assert!(!report.is_synchronized);
        assert!(!report.is_exempt_from_readme);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::MissingReleaseSummaryInReadme {
                readme_lang: "en".to_string(),
                expected_version: "v4.251.0".to_string(),
            }
        );
    }

    #[test]
    fn test_beta_prerelease_exempt_from_readme_sync() {
        let target_version = "v4.251.0-beta.1";
        let changelog_summary = "### [v4.251.0-beta.1]\n- Beta preview notes";
        // Both README files still have older stable release notes
        let readme_zh = "## 📝 更新日志\n### v4.250.0\n- Previous stable";
        let readme_en = "## 📝 Changelog\n### v4.250.0\n- Previous stable";

        let report = verify_readme_changelog_synchronization(
            target_version,
            changelog_summary,
            readme_zh,
            readme_en,
            &ReleaseChannel::BetaPreRelease,
        );

        assert!(report.is_synchronized);
        assert!(report.is_exempt_from_readme);
        assert_eq!(report.target_version, "v4.251.0-beta.1");
        assert_eq!(report.channel, ReleaseChannel::BetaPreRelease);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_missing_changelog_sections_flagged() {
        let target_version = "v4.251.0";
        let changelog_summary = "### [v4.251.0]\n- Summary";
        let readme_zh_no_section = "# Title\nNo changelog section here";
        let readme_en_no_section = "# Title\nNo changelog section here";

        let report = verify_readme_changelog_synchronization(
            target_version,
            changelog_summary,
            readme_zh_no_section,
            readme_en_no_section,
            &ReleaseChannel::Stable,
        );

        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 2);
        assert!(report
            .violations
            .contains(&ReadmeSyncViolation::MissingChineseReadmeChangelogSection));
        assert!(report
            .violations
            .contains(&ReadmeSyncViolation::MissingEnglishReadmeChangelogSection));
    }

    #[test]
    fn test_mismatched_changelog_summary_version_flagged() {
        let target_version = "v4.251.0";
        let changelog_summary = "### [v4.250.0]\n- Outdated changelog summary";
        let readme_zh = format!("{CHINESE_CHANGELOG_HEADER}\n### v4.251.0\n- Chinese notes");
        let readme_en = format!("{ENGLISH_CHANGELOG_HEADER}\n### v4.251.0\n- English notes");

        let report = verify_readme_changelog_synchronization(
            target_version,
            &changelog_summary,
            &readme_zh,
            &readme_en,
            &ReleaseChannel::Stable,
        );

        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::MismatchedVersionSummary {
                expected_version: "v4.251.0".to_string(),
                found_version: Some("v4.250.0".to_string()),
            }
        );
    }

    #[test]
    fn test_empty_changelog_summary_flagged() {
        let target_version = "v4.251.0";
        let readme_zh = format!("{CHINESE_CHANGELOG_HEADER}\n### v4.251.0\n- Notes");
        let readme_en = format!("{ENGLISH_CHANGELOG_HEADER}\n### v4.251.0\n- Notes");

        let report = verify_readme_changelog_synchronization(
            target_version,
            "   ",
            &readme_zh,
            &readme_en,
            &ReleaseChannel::Stable,
        );

        assert!(!report.is_synchronized);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0],
            ReadmeSyncViolation::MismatchedVersionSummary {
                expected_version: "v4.251.0".to_string(),
                found_version: None,
            }
        );
    }

    #[test]
    fn test_version_normalization_and_matching() {
        assert!(version_matches("v4.251.0", "4.251.0"));
        assert!(version_matches("4.251.0", "v4.251.0"));
        assert!(version_matches("v4.251.0", "v4.251.0"));
        assert!(!version_matches("v4.251.0", "v4.250.0"));

        let sample = "## 📝 更新日志\n### 4.251.0 (2026-10-23)";
        assert!(contains_version(sample, "v4.251.0"));
        assert!(contains_version(sample, "4.251.0"));

        assert_eq!(
            extract_version("release-v4.251.0-final"),
            Some("v4.251.0".to_string())
        );
    }

    #[test]
    fn test_release_channel_helpers() {
        assert!(ReleaseChannel::Stable.is_stable());
        assert!(!ReleaseChannel::Stable.is_beta_prerelease());

        assert!(ReleaseChannel::BetaPreRelease.is_beta_prerelease());
        assert!(!ReleaseChannel::BetaPreRelease.is_stable());

        assert_eq!(
            ReleaseChannel::detect_from_version_or_tag("v4.251.0"),
            ReleaseChannel::Stable
        );
        assert_eq!(
            ReleaseChannel::detect_from_version_or_tag("v4.251.0-beta.1"),
            ReleaseChannel::BetaPreRelease
        );
        assert_eq!(
            ReleaseChannel::detect_from_version_or_tag("v1.0.0-rc.2"),
            ReleaseChannel::BetaPreRelease
        );
    }

    #[test]
    fn test_extract_section_boundaries() {
        let content = "\
# Title
## 📝 更新日志
Section Content Line 1
Section Content Line 2
## Next Section
Other Content
";
        let section = extract_section(content, "## 📝 更新日志");
        assert!(section.is_some());
        let text = section.unwrap();
        assert!(text.contains("Section Content Line 1"));
        assert!(!text.contains("Other Content"));
    }

    #[test]
    fn test_violation_display_formatting_and_error() {
        let v1 = ReadmeSyncViolation::MissingChineseReadmeChangelogSection;
        assert_eq!(
            v1.to_string(),
            "Missing Chinese changelog section '## 📝 更新日志' in README.md"
        );

        let v2 = ReadmeSyncViolation::MissingEnglishReadmeChangelogSection;
        assert_eq!(
            v2.to_string(),
            "Missing English changelog section '## 📝 Changelog' in README_EN.md"
        );

        let v3 = ReadmeSyncViolation::MismatchedVersionSummary {
            expected_version: "v4.251.0".to_string(),
            found_version: Some("v4.250.0".to_string()),
        };
        assert_eq!(
            v3.to_string(),
            "Mismatched version summary: expected 'v4.251.0', found 'Some(\"v4.250.0\")'"
        );

        let v4 = ReadmeSyncViolation::MissingReleaseSummaryInReadme {
            readme_lang: "zh".to_string(),
            expected_version: "v4.251.0".to_string(),
        };
        assert_eq!(
            v4.to_string(),
            "Missing release summary for 'v4.251.0' in zh README"
        );

        let err: &dyn std::error::Error = &v1;
        assert_eq!(err.to_string(), v1.to_string());
    }

    #[test]
    fn test_serde_roundtrip() {
        let report = ReadmeSyncReport {
            is_synchronized: false,
            target_version: "v4.251.0".to_string(),
            channel: ReleaseChannel::Stable,
            violations: vec![
                ReadmeSyncViolation::MissingChineseReadmeChangelogSection,
                ReadmeSyncViolation::MissingReleaseSummaryInReadme {
                    readme_lang: "en".to_string(),
                    expected_version: "v4.251.0".to_string(),
                },
            ],
            is_exempt_from_readme: false,
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: ReadmeSyncReport =
            serde_json::from_str(&json).expect("deserialize report");

        assert_eq!(report, deserialized);
    }
}
