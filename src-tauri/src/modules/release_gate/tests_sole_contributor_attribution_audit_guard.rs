//! Unit tests for Sole Contributor Attribution & No Silent Errors Audit Guard (`Imp-714`, Step 300).

#[cfg(test)]
mod tests {
    use super::sole_contributor_attribution_audit_guard::{
        audit_release_attribution, audit_silent_error_patterns, count_absolute_paths,
        generate_governance_report, AttributionFinding, ReleaseGovernanceReport,
        SilentErrorFinding, STANDARD_ATTRIBUTION_PATTERN,
    };

    #[test]
    fn test_standard_attribution_pattern_constant() {
        assert_eq!(STANDARD_ATTRIBUTION_PATTERN, "(Thanks to @aukgit)");
    }

    #[test]
    fn test_audit_release_attribution_valid_aukgit() {
        let text = "\
# Changelog

## [v4.252.0] - 2026-10-23
### Features
- Implemented transient sandbox lifecycle sentinels.
(Thanks to @aukgit)
";
        let findings = audit_release_attribution(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0], AttributionFinding::ValidAukgitAttribution);
        assert!(findings[0].is_valid());
        assert!(!findings[0].is_unauthorized());
        assert_eq!(findings[0].unauthorized_handle(), None);
        assert!(findings[0].to_string().contains("Valid attribution"));
    }

    #[test]
    fn test_audit_release_attribution_missing_trailer() {
        let text = "\
## [v4.252.0] - 2026-10-23
- Some feature update without the standard trailer.
";
        let findings = audit_release_attribution(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0], AttributionFinding::MissingAttributionTrailer);
        assert!(!findings[0].is_valid());
        assert!(!findings[0].is_unauthorized());
        assert!(findings[0]
            .to_string()
            .contains("Missing standard attribution trailer"));
    }

    #[test]
    fn test_audit_release_attribution_unauthorized_handles() {
        let text = "\
## [v4.252.0] - 2026-10-23
- Co-authored by @alice and reviewed by @bob_dev.
(Thanks to @aukgit)
";
        let findings = audit_release_attribution(text);
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0], AttributionFinding::ValidAukgitAttribution);
        assert_eq!(
            findings[1],
            AttributionFinding::UnauthorizedHandle("@alice".to_string())
        );
        assert_eq!(
            findings[2],
            AttributionFinding::UnauthorizedHandle("@bob_dev".to_string())
        );
        assert!(findings[1].is_unauthorized());
        assert_eq!(findings[1].unauthorized_handle(), Some("@alice"));
    }

    #[test]
    fn test_email_address_not_flagged_as_handle() {
        let text = "\
Contact support at support@riseup-asia.com or admin@example.org.
(Thanks to @aukgit)
";
        let findings = audit_release_attribution(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0], AttributionFinding::ValidAukgitAttribution);
    }

    #[test]
    fn test_audit_silent_error_rust_bare_let_discard() {
        let rust_code = "\
fn save_profile(path: &Path) {
    let _ = fs::write(path, b\"data\");
}
";
        let findings = audit_silent_error_patterns(rust_code, "src/modules/profile.rs");
        assert_eq!(findings.len(), 1);
        match &findings[0] {
            SilentErrorFinding::BareResultDiscarded { file, line } => {
                assert_eq!(file, "src/modules/profile.rs");
                assert_eq!(*line, 2);
            }
            other => panic!("Expected BareResultDiscarded, got {:?}", other),
        }
        assert!(findings[0].is_violation());
        assert_eq!(findings[0].file(), Some("src/modules/profile.rs"));
        assert_eq!(findings[0].line(), Some(2));
    }

    #[test]
    fn test_audit_silent_error_rust_justified_record_ignored() {
        let rust_code = "\
fn best_effort_cleanup(tmp_path: &Path) {
    // Justification: Safe to ignore temporary file removal failure
    crate::error::record_ignored(fs::remove_file(tmp_path), \"remove temp\");
}
";
        let findings = audit_silent_error_patterns(rust_code, "src/modules/cleaner.rs");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0], SilentErrorFinding::DocumentedIgnoredResult);
        assert!(!findings[0].is_violation());
        assert_eq!(findings[0].file(), None);
        assert_eq!(findings[0].line(), None);
    }

    #[test]
    fn test_audit_silent_error_rust_unjustified_record_ignored() {
        let rust_code = "\
fn best_effort_cleanup(tmp_path: &Path) {
    crate::error::record_ignored(fs::remove_file(tmp_path), \"remove temp\");
}
";
        let findings = audit_silent_error_patterns(rust_code, "src/modules/cleaner.rs");
        assert_eq!(findings.len(), 1);
        assert!(findings[0].is_violation());
        match &findings[0] {
            SilentErrorFinding::BareResultDiscarded { line, .. } => assert_eq!(*line, 2),
            other => panic!("Expected BareResultDiscarded, got {:?}", other),
        }
    }

    #[test]
    fn test_audit_silent_error_typescript_swallowed_catch() {
        let ts_code = "\
export async function loadData() {
    await fetchApi().catch(() => {});
}
";
        let findings = audit_silent_error_patterns(ts_code, "src/api/client.ts");
        assert_eq!(findings.len(), 1);
        match &findings[0] {
            SilentErrorFinding::BareCatchWithoutTracking { file, line } => {
                assert_eq!(file, "src/api/client.ts");
                assert_eq!(*line, 2);
            }
            other => panic!("Expected BareCatchWithoutTracking, got {:?}", other),
        }
    }

    #[test]
    fn test_audit_silent_error_typescript_empty_catch_block() {
        let ts_code = "\
try {
    performAction();
} catch (e) {
    // swallowed without error store
}
";
        let findings = audit_silent_error_patterns(ts_code, "src/components/Action.tsx");
        assert_eq!(findings.len(), 1);
        assert!(findings[0].is_violation());
    }

    #[test]
    fn test_audit_silent_error_typescript_compliant_catch_block() {
        let ts_code = "\
try {
    performAction();
} catch (e) {
    useErrorStore.getState().trackWarning(e, { source: 'Action.tsx', triggerAction: 'performAction' });
}
";
        let findings = audit_silent_error_patterns(ts_code, "src/components/Action.tsx");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_count_absolute_paths_detection() {
        let text_with_paths = "\
Here are absolute paths:
1. Windows: D:\\work\\Antigravity-Manager\\src-tauri\\file.rs
2. Windows: C:/Users/Administrator/app.log
3. POSIX: /home/ubuntu/project
4. POSIX: /Users/runner/work
5. System: /var/log/syslog
6. URI: file:///C:/Users/test
";
        let count = count_absolute_paths(text_with_paths);
        assert_eq!(count, 6);

        let clean_text = "\
All references are strictly relative:
- `02-spec/21-app/spec.md`
- `.ai-memory/plans/`
- `src-tauri/src/modules/`
";
        assert_eq!(count_absolute_paths(clean_text), 0);
    }

    #[test]
    fn test_generate_governance_report_fully_compliant() {
        let changelog = "\
# Changelog

## [v4.252.0] - 2026-10-23
### Features
- Implemented sole contributor attribution and no silent errors guard.
(Thanks to @aukgit)
";
        let rust_code = "\
fn read_profile(path: &Path) -> Result<Profile, AppError> {
    let data = fs::read_to_string(path).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(serde_json::from_str(&data).map_err(|e| AppError::Json(e.to_string()))?)
}
";
        let ts_code = "\
try {
    saveOptions();
} catch (err) {
    useErrorStore.getState().trackWarning(err, { source: 'Settings' });
}
";
        let code_sources = [(rust_code, "src/profile.rs"), (ts_code, "src/settings.ts")];

        let report = generate_governance_report(changelog, &code_sources);
        assert!(report.is_compliant);
        assert!(!report.has_attribution_violations());
        assert!(!report.has_silent_error_violations());
        assert!(!report.has_absolute_path_violations());
        assert_eq!(report.absolute_path_count, 0);
        assert!(report.summary.contains("✅ COMPLIANT"));
    }

    #[test]
    fn test_generate_governance_report_flagged_violations() {
        let changelog = "\
## [v4.252.0]
- Contributed by @charlie at D:\\builds\\logs.txt
";
        let rust_code = "\
fn delete_temp(p: &Path) {
    let _ = fs::remove_file(p);
}
";
        let code_sources = [(rust_code, "src/temp.rs")];

        let report = generate_governance_report(changelog, &code_sources);
        assert!(!report.is_compliant);
        assert!(report.has_attribution_violations());
        assert!(report.has_silent_error_violations());
        assert!(report.has_absolute_path_violations());
        assert_eq!(report.absolute_path_count, 1);
        assert!(report.summary.contains("❌ NON-COMPLIANT"));
        assert!(report
            .summary
            .contains("Unauthorized third-party handle detected: '@charlie'"));
        assert!(report
            .summary
            .contains("Missing standard attribution trailer"));
    }

    #[test]
    fn test_serde_roundtrip_governance_models() {
        let finding = AttributionFinding::UnauthorizedHandle("@external".to_string());
        let ser = serde_json::to_string(&finding).expect("serialize attribution finding");
        let de: AttributionFinding =
            serde_json::from_str(&ser).expect("deserialize attribution finding");
        assert_eq!(finding, de);

        let silent_finding = SilentErrorFinding::BareResultDiscarded {
            file: "src/test.rs".to_string(),
            line: 42,
        };
        let ser_silent = serde_json::to_string(&silent_finding).expect("serialize silent finding");
        let de_silent: SilentErrorFinding =
            serde_json::from_str(&ser_silent).expect("deserialize silent finding");
        assert_eq!(silent_finding, de_silent);

        let report = ReleaseGovernanceReport {
            is_compliant: true,
            attribution_findings: vec![AttributionFinding::ValidAukgitAttribution],
            silent_error_findings: vec![SilentErrorFinding::DocumentedIgnoredResult],
            absolute_path_count: 0,
            summary: "All good".to_string(),
        };
        let ser_report = serde_json::to_string(&report).expect("serialize governance report");
        let de_report: ReleaseGovernanceReport =
            serde_json::from_str(&ser_report).expect("deserialize governance report");
        assert_eq!(report, de_report);
    }
}
