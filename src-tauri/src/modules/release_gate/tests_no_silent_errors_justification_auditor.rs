#[cfg(test)]
mod tests {
    use super::super::no_silent_errors_justification_auditor::{
        audit_source_for_silent_errors, check_has_justification, is_comment_line,
        is_fallible_let_expression, SilentErrorViolation, SilentErrorsAuditReport,
    };

    #[test]
    fn test_record_ignored_with_preceding_justification_passes() {
        let code = r#"
            fn cleanup_temp(tmp: &std::path::Path) {
                // Justification: safe to ignore temp file removal failure on cleanup
                crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file");
            }
        "#;
        let report = audit_source_for_silent_errors(code, "test.rs");
        assert!(report.is_clean);
        assert_eq!(report.total_sites_audited, 1);
        assert_eq!(report.valid_justifications_count, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_record_ignored_with_inline_justification_passes() {
        let code = r#"
            fn notify() {
                crate::error::record_ignored(send_metric(), "metric"); // Justification: best effort telemetry
            }
        "#;
        let report = audit_source_for_silent_errors(code, "test.rs");
        assert!(report.is_clean);
        assert_eq!(report.total_sites_audited, 1);
        assert_eq!(report.valid_justifications_count, 1);
    }

    #[test]
    fn test_record_ignored_missing_justification_fails() {
        let code = r#"
            fn cleanup(p: &std::path::Path) {
                crate::error::record_ignored(std::fs::remove_file(p), "cleanup");
            }
        "#;
        let report = audit_source_for_silent_errors(code, "cleanup.rs");
        assert!(!report.is_clean);
        assert_eq!(report.total_sites_audited, 1);
        assert_eq!(report.valid_justifications_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0],
            SilentErrorViolation::BareRecordIgnoredWithoutJustification { line: 3, .. }
        ));
    }

    #[test]
    fn test_bare_let_underscore_fallible_fails() {
        let code = r#"
            fn write_cache(path: &str, data: &[u8]) {
                let _ = std::fs::remove_file(path);
            }
        "#;
        let report = audit_source_for_silent_errors(code, "cache.rs");
        assert!(!report.is_clean);
        assert_eq!(report.total_sites_audited, 1);
        assert_eq!(report.violations.len(), 1);
        if let SilentErrorViolation::BareLetUnderscoreResultDiscard { snippet, line, .. } =
            &report.violations[0]
        {
            assert_eq!(*line, 3);
            assert!(snippet.contains("remove_file"));
        } else {
            panic!("Expected BareLetUnderscoreResultDiscard violation");
        }
    }

    #[test]
    fn test_bare_ok_result_discard_fails() {
        let code = r#"
            fn process() {
                std::fs::read_to_string("config.toml").ok();
            }
        "#;
        let report = audit_source_for_silent_errors(code, "process.rs");
        assert!(!report.is_clean);
        assert_eq!(report.total_sites_audited, 1);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0],
            SilentErrorViolation::BareOkResultDiscard { line: 3, .. }
        ));
    }

    #[test]
    fn test_clean_source_code_passes() {
        let code = r#"
            fn safe_write(path: &Path, content: &str) -> Result<(), AppError> {
                std::fs::write(path, content).map_err(|e| AppError::Io(e.to_string()))?;
                let x = 42;
                let _ = x;
                Ok(())
            }
        "#;
        let report = audit_source_for_silent_errors(code, "safe.rs");
        assert!(report.is_clean);
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_display_and_error_formatting() {
        let v1 = SilentErrorViolation::BareRecordIgnoredWithoutJustification {
            file: "mod.rs".to_string(),
            line: 15,
        };
        let v2 = SilentErrorViolation::BareLetUnderscoreResultDiscard {
            file: "lib.rs".to_string(),
            line: 22,
            snippet: "let _ = fs::write(&p, d);".to_string(),
        };
        let v3 = SilentErrorViolation::BareOkResultDiscard {
            file: "main.rs".to_string(),
            line: 30,
            snippet: "file.sync().ok();".to_string(),
        };

        let err1: &dyn std::error::Error = &v1;
        let err2: &dyn std::error::Error = &v2;
        let err3: &dyn std::error::Error = &v3;

        assert!(err1.to_string().contains("mandatory '// Justification:'"));
        assert!(err2.to_string().contains("bare 'let _ = '"));
        assert!(err3.to_string().contains("bare '.ok()'"));

        let mut report = SilentErrorsAuditReport::new();
        report.is_clean = false;
        report.total_sites_audited = 3;
        report.violations = vec![v1, v2, v3];

        let report_str = format!("{}", report);
        assert!(report_str.contains("Violations (3 found across 3 sites)"));
    }

    #[test]
    fn test_comment_lines_and_fallible_helpers() {
        assert!(is_comment_line("// this is a comment"));
        assert!(is_comment_line("/* block comment */"));
        assert!(is_comment_line("* inner block"));
        assert!(!is_comment_line("let x = 1;"));

        assert!(is_fallible_let_expression("let _ = std::fs::remove_file(p);"));
        assert!(is_fallible_let_expression("let _ = invoke(\"cmd\");"));
        assert!(!is_fallible_let_expression("let _ = 123;"));
    }
}
