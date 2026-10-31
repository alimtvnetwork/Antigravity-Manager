#[cfg(test)]
mod tests {
    use super::super::no_silent_errors_linter::{
        check_has_justification_comment, conforming_best_effort_snippet,
        conforming_typescript_catch_snippet, lint_rust_code, lint_source, lint_typescript_code,
        SilentErrorViolationKind,
    };

    #[test]
    fn test_rust_bare_let_underscore_detected() {
        let code = r#"
            fn write_cache(path: &str, data: &[u8]) {
                let _ = std::fs::write(&path, data);
            }
        "#;
        let report = lint_rust_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::BareLetUnderscore { .. }
        ));
    }

    #[test]
    fn test_rust_record_ignored_with_justification_passes() {
        let code = r#"
            fn cleanup_temp(tmp: &Path) {
                // Justification: safe to ignore temp file removal failure on cleanup
                crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file");
            }
        "#;
        let report = lint_rust_code(code);
        assert!(
            report.is_clean,
            "Expected clean report, got: {:?}",
            report.violations
        );
        assert_eq!(report.best_effort_recorded_count, 1);
    }

    #[test]
    fn test_rust_record_ignored_missing_justification_detected() {
        let code = r#"
            fn cleanup_temp(tmp: &Path) {
                crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file");
            }
        "#;
        let report = lint_rust_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::RecordIgnoredMissingJustification { .. }
        ));
    }

    #[test]
    fn test_rust_record_ignored_empty_justification_detected() {
        let code = r#"
            fn cleanup_temp(tmp: &Path) {
                // Justification:
                crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file");
            }
        "#;
        let report = lint_rust_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::RecordIgnoredMissingJustification { .. }
        ));
    }

    #[test]
    fn test_rust_bare_ok_discarded_detected() {
        let code = r#"
            fn flush_data(file: &mut File) {
                file.flush().ok();
            }
        "#;
        let report = lint_rust_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::BareOkDiscarded { .. }
        ));
    }

    #[test]
    fn test_rust_ok_or_else_not_flagged() {
        let code = r#"
            fn get_config(opt: Option<String>) -> Result<String, AppError> {
                let val = opt.ok_or_else(|| AppError::Config("missing".into()))?;
                Ok(val)
            }
        "#;
        let report = lint_rust_code(code);
        assert!(
            report.is_clean,
            "ok_or_else should not be flagged: {:?}",
            report.violations
        );
    }

    #[test]
    fn test_rust_proper_propagation_passes() {
        let code = r#"
            fn save_config(path: &Path, data: &[u8]) -> Result<(), AppError> {
                std::fs::write(&path, data).map_err(|e| AppError::Io(format!("write {path:?}: {e}")))?;
                Ok(())
            }
        "#;
        let report = lint_rust_code(code);
        assert!(
            report.is_clean,
            "Proper error propagation should be clean: {:?}",
            report.violations
        );
    }

    #[test]
    fn test_rust_unwrap_or_default_on_fallible_detected() {
        let code = r#"
            fn load_string(path: &str) -> String {
                std::fs::read_to_string(path).unwrap_or_default()
            }
        "#;
        let report = lint_rust_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::BareUnwrapOrDefaultOnFallible { .. }
        ));
    }

    #[test]
    fn test_typescript_swallowed_promise_catch_detected() {
        let code = r#"
            async function saveSettings() {
                await invoke('save_config').catch(() => {});
            }
        "#;
        let report = lint_typescript_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::SwallowedPromiseCatch { .. }
        ));
    }

    #[test]
    fn test_typescript_bare_catch_block_detected() {
        let code = r#"
            try {
                doSomething();
            } catch {}
        "#;
        let report = lint_typescript_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::BareCatchBlock { .. }
        ));
    }

    #[test]
    fn test_typescript_console_only_catch_detected() {
        let code = r#"
            try {
                doSomething();
            } catch (e) {
                console.warn(e);
            }
        "#;
        let report = lint_typescript_code(code);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            report.violations[0].kind,
            SilentErrorViolationKind::ConsoleOnlyCatchWithoutErrorStore { .. }
        ));
    }

    #[test]
    fn test_typescript_proper_track_warning_passes() {
        let code = r#"
            try {
                doSomething();
            } catch (e) {
                useErrorStore.getState().trackWarning(e, { source: 'MyComp.save', triggerAction: 'save_x' });
            }
        "#;
        let report = lint_typescript_code(code);
        assert!(
            report.is_clean,
            "Proper error store trackWarning should be clean: {:?}",
            report.violations
        );
        assert_eq!(report.best_effort_recorded_count, 1);
    }

    #[test]
    fn test_helper_snippets() {
        let rust_snippet = conforming_best_effort_snippet(
            "std::fs::remove_file(&tmp)",
            "remove temp file",
            "safe cleanup on exit",
        );
        assert!(rust_snippet.contains("// Justification: safe cleanup on exit"));
        assert!(rust_snippet.contains("crate::error::record_ignored"));

        let ts_snippet = conforming_typescript_catch_snippet("e", "SettingsPanel", "save_key");
        assert!(ts_snippet.contains("useErrorStore.getState().trackWarning"));
        assert!(ts_snippet.contains("SettingsPanel"));
        assert!(ts_snippet.contains("save_key"));
    }

    #[test]
    fn test_comments_and_docstrings_are_ignored() {
        let code = r#"
            // Example of forbidden pattern:
            // let _ = std::fs::write(&path, data);
            // file.flush().ok();
            /*
             * let _ = std::fs::remove_file(&path);
             */
            fn real_function() -> Result<(), AppError> {
                Ok(())
            }
        "#;
        let report = lint_rust_code(code);
        assert!(
            report.is_clean,
            "Comments containing bad examples should not trigger violations: {:?}",
            report.violations
        );
    }

    #[test]
    fn test_check_has_justification_comment() {
        let lines = vec![
            "fn run() {",
            "    // Justification: safe to ignore in testing",
            "    crate::error::record_ignored(op(), \"op\");",
            "}",
        ];
        assert!(check_has_justification_comment(&lines, 2));

        let bad_lines = vec![
            "fn run() {",
            "    let x = 1;",
            "    crate::error::record_ignored(op(), \"op\");",
            "}",
        ];
        assert!(!check_has_justification_comment(&bad_lines, 2));
    }

    #[test]
    fn test_lint_source_dispatch() {
        let rust_code = "fn f() { let _ = std::fs::write(\"a\", b\"b\"); }";
        let r_rep = lint_source(rust_code, true);
        assert!(!r_rep.is_clean);

        let ts_code = "async () => { await doIt().catch(() => {}); }";
        let ts_rep = lint_source(ts_code, false);
        assert!(!ts_rep.is_clean);
    }
}
