//! Tests for Sole Contributor Attribution Compliance Guard.

#[cfg(test)]
mod tests {
    use super::super::sole_contributor_attribution_guard::{
        audit_changelog_attribution, AttributionViolation,
    };

    #[test]
    fn test_clean_changelog_with_only_aukgit_is_compliant() {
        let content = "\
# Changelog

## [v4.250.0] - 2026-10-23
### Features
- Enhanced release gate verification and path linting.
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content, "changelog.md");
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 1);
        assert!(report.violations.is_empty());
        assert_eq!(report.total_lines_scanned, 8);
    }

    #[test]
    fn test_extraneous_handle_flagged_with_disallowed_contributor_handle() {
        let content = "\
## [v4.250.0] - 2026-10-23
- Implemented feature X by @contributor_x
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content, "changelog.md");
        assert!(!report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 1);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::DisallowedContributorHandle {
                line_number,
                handle,
                snippet,
            } => {
                assert_eq!(*line_number, 2);
                assert_eq!(handle, "@contributor_x");
                assert!(snippet.contains("@contributor_x"));
            }
            _ => panic!("Expected DisallowedContributorHandle violation"),
        }
    }

    #[test]
    fn test_missing_aukgit_attribution_flagged() {
        let content = "\
## [v4.250.0] - 2026-10-23
- Initial release without attribution footer.
";
        let report = audit_changelog_attribution(content, "changelog_en.md");
        assert!(!report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 0);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::MissingAukgitAttribution { file_name } => {
                assert_eq!(file_name, "changelog_en.md");
            }
            _ => panic!("Expected MissingAukgitAttribution violation"),
        }
    }

    #[test]
    fn test_multiple_handles_on_single_line() {
        let content = "\
## [v4.250.0] - 2026-10-23
- Co-developed by @alice and @bob-smith with feedback from @carol_99. (Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content, "changelog.md");
        assert!(!report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 1);
        assert_eq!(report.violations.len(), 3);

        let handles: Vec<String> = report
            .violations
            .iter()
            .filter_map(|v| match v {
                AttributionViolation::DisallowedContributorHandle { handle, .. } => {
                    Some(handle.clone())
                }
                _ => None,
            })
            .collect();

        assert_eq!(
            handles,
            vec![
                "@alice".to_string(),
                "@bob-smith".to_string(),
                "@carol_99".to_string()
            ]
        );
    }

    #[test]
    fn test_violation_display_formatting() {
        let disallowed = AttributionViolation::DisallowedContributorHandle {
            line_number: 42,
            handle: "@external_dev".to_string(),
            snippet: "Fixed bug by @external_dev".to_string(),
        };
        let formatted_disallowed = format!("{disallowed}");
        assert!(formatted_disallowed.contains("@external_dev"));
        assert!(formatted_disallowed.contains("42"));
        assert!(formatted_disallowed.contains("Fixed bug by @external_dev"));

        let missing = AttributionViolation::MissingAukgitAttribution {
            file_name: "changelog.md".to_string(),
        };
        let formatted_missing = format!("{missing}");
        assert!(formatted_missing.contains("@aukgit"));
        assert!(formatted_missing.contains("changelog.md"));

        // Verify std::error::Error implementation
        let err: &dyn std::error::Error = &disallowed;
        assert_eq!(err.to_string(), formatted_disallowed);
    }

    #[test]
    fn test_case_insensitive_aukgit_attribution() {
        let content = "\
## [v4.250.0] - 2026-10-23
- Performance optimizations.
(Thanks to @AUKGIT)
";
        let report = audit_changelog_attribution(content, "changelog.md");
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_email_addresses_not_flagged_as_handles() {
        let content = "\
## [v4.250.0] - 2026-10-23
- For support contact help@example.com or security@riseup-asia.com.
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content, "changelog.md");
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_count, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_empty_content_flags_missing_attribution() {
        let report = audit_changelog_attribution("", "changelog.md");
        assert!(!report.is_compliant);
        assert_eq!(report.total_lines_scanned, 0);
        assert_eq!(report.aukgit_attributions_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            AttributionViolation::MissingAukgitAttribution { .. }
        ));
    }
}
