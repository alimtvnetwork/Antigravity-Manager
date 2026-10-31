//! Tests for Sole Contributor Attribution Release Enforcer.

#[cfg(test)]
mod tests {
    use super::super::sole_contributor_attribution_enforcer::{
        audit_changelog_attribution, audit_release_notes, audit_release_notes_contributors,
        extract_handles_from_text, AttributionReport, AttributionViolation,
        STANDARD_ATTRIBUTION_PATTERN,
    };

    #[test]
    fn test_valid_changelog_with_aukgit_passes_clean() {
        let content = "\
# Changelog

## [v4.251.0] - 2026-10-23
### Features
- Implemented pipeline decoupling and protocol unification.
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content);
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_found, 1);
        assert!(!report.has_third_party_handles);
        assert!(report.violations.is_empty());
        assert_eq!(report.scanned_lines_count, 8);
    }

    #[test]
    fn test_third_party_handle_alice_flagged() {
        let content = "\
## [v4.251.0] - 2026-10-23
- Implemented by @alice with review assistance.
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content);
        assert!(!report.is_compliant);
        assert!(report.has_third_party_handles);
        assert_eq!(report.aukgit_attributions_found, 1);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::ThirdPartyHandleDetected {
                handle,
                line_number,
            } => {
                assert_eq!(handle, "@alice");
                assert_eq!(*line_number, 2);
            }
            _ => panic!("Expected ThirdPartyHandleDetected violation"),
        }
    }

    #[test]
    fn test_third_party_handle_bob_flagged() {
        let content = "\
## [v4.251.0] - 2026-10-23
- Co-developed by @bob
(Thanks to @aukgit)
";
        let report = audit_changelog_attribution(content);
        assert!(!report.is_compliant);
        assert!(report.has_third_party_handles);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::ThirdPartyHandleDetected {
                handle,
                line_number,
            } => {
                assert_eq!(handle, "@bob");
                assert_eq!(*line_number, 2);
            }
            _ => panic!("Expected ThirdPartyHandleDetected violation"),
        }
    }

    #[test]
    fn test_missing_standard_attribution_format_flagged() {
        let content = "\
## [v4.251.0] - 2026-10-23
- Added critical error boundaries.
Thanks to @aukgit
";
        let report = audit_changelog_attribution(content);
        assert!(!report.is_compliant);
        assert_eq!(report.aukgit_attributions_found, 1);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::MissingStandardAttributionPattern {
                line_number,
                expected,
            } => {
                assert_eq!(*line_number, 3);
                assert_eq!(expected, STANDARD_ATTRIBUTION_PATTERN);
            }
            _ => panic!("Expected MissingStandardAttributionPattern violation"),
        }
    }

    #[test]
    fn test_clean_text_with_no_handles_passes_clean() {
        let content = "\
## [v4.251.0] - 2026-10-23
### Fixes
- Resolved socket timeout race condition.
- Cleaned up temp build directory remnants.
";
        let report = audit_changelog_attribution(content);
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_found, 0);
        assert!(!report.has_third_party_handles);
        assert!(report.violations.is_empty());
        assert_eq!(report.scanned_lines_count, 6);
    }

    #[test]
    fn test_release_notes_with_non_aukgit_contributor() {
        let release_notes = "Release v4.251.0 contributors: @aukgit, @charlie";
        let report = audit_release_notes(release_notes);

        assert!(!report.is_compliant);
        assert!(report.has_third_party_handles);
        assert_eq!(report.aukgit_attributions_found, 1);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            AttributionViolation::NonAukgitContributorInReleaseNotes { handle } => {
                assert_eq!(handle, "@charlie");
            }
            _ => panic!("Expected NonAukgitContributorInReleaseNotes"),
        }
    }

    #[test]
    fn test_audit_release_notes_contributors_slice() {
        let contributors = vec!["aukgit", "@david", "aukgit"];
        let violations = audit_release_notes_contributors(&contributors);

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            AttributionViolation::NonAukgitContributorInReleaseNotes { handle } => {
                assert_eq!(handle, "@david");
            }
            _ => panic!("Expected NonAukgitContributorInReleaseNotes"),
        }
    }

    #[test]
    fn test_extract_handles_from_text() {
        let text = "Thanks to @aukgit and feedback from @eve-ops, but ignore contact@example.com";
        let handles = extract_handles_from_text(text);

        assert_eq!(handles, vec!["@aukgit".to_string(), "@eve-ops".to_string()]);
    }

    #[test]
    fn test_case_insensitive_aukgit_attribution() {
        let content = "\
## [v4.251.0] - 2026-10-23
- Performance enhancement.
(Thanks to @AUKGIT)
";
        let report = audit_changelog_attribution(content);
        assert!(report.is_compliant);
        assert_eq!(report.aukgit_attributions_found, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_display_formatting_and_error_impl() {
        let v1 = AttributionViolation::ThirdPartyHandleDetected {
            handle: "@mallory".to_string(),
            line_number: 14,
        };
        assert_eq!(
            v1.to_string(),
            "Third-party handle '@mallory' detected at line 14"
        );

        let v2 = AttributionViolation::MissingStandardAttributionPattern {
            line_number: 9,
            expected: "(Thanks to @aukgit)".to_string(),
        };
        assert_eq!(
            v2.to_string(),
            "Missing standard attribution pattern '(Thanks to @aukgit)' at line 9"
        );

        let v3 = AttributionViolation::NonAukgitContributorInReleaseNotes {
            handle: "@trent".to_string(),
        };
        assert_eq!(
            v3.to_string(),
            "Non-aukgit contributor '@trent' found in release notes"
        );

        let err: &dyn std::error::Error = &v1;
        assert_eq!(err.to_string(), v1.to_string());
    }

    #[test]
    fn test_serde_roundtrip_violations_and_report() {
        let report = AttributionReport {
            is_compliant: false,
            violations: vec![
                AttributionViolation::ThirdPartyHandleDetected {
                    handle: "@stranger".to_string(),
                    line_number: 22,
                },
                AttributionViolation::MissingStandardAttributionPattern {
                    line_number: 25,
                    expected: "(Thanks to @aukgit)".to_string(),
                },
                AttributionViolation::NonAukgitContributorInReleaseNotes {
                    handle: "@outsider".to_string(),
                },
            ],
            scanned_lines_count: 50,
            aukgit_attributions_found: 2,
            has_third_party_handles: true,
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: AttributionReport =
            serde_json::from_str(&json).expect("deserialize report");

        assert_eq!(report, deserialized);
    }
}
