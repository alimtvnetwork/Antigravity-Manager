//! Unit tests for Cargo Clippy Zero-Warnings Preflight Gate (`Imp-651`).

#[cfg(test)]
mod tests {
    use crate::modules::release_gate::preflight_clippy_gate::{
        extract_code_from_note, is_compiler_summary_line, parse_clippy_output,
        parse_json_diagnostic_line, parse_source_location, parse_text_diagnostic_header,
        ClippyAuditReport, ClippyDiagnostic, ClippyDiagnosticLevel,
    };

    #[test]
    fn test_clean_output_passes_audit_report() {
        let clean_output = r#"
            Checking agm-ui v4.251.0 (D:\work\Antigravity-Manager\src-tauri)
            Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.15s
        "#;

        let report = parse_clippy_output(clean_output);

        assert!(report.is_clean());
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 0);
        assert_eq!(report.total_diagnostics, 0);
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
    }

    #[test]
    fn test_empty_output_passes_audit_report() {
        let report = parse_clippy_output("");

        assert!(report.is_clean());
        assert_eq!(report.total_diagnostics, 0);
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
    }

    #[test]
    fn test_clippy_warning_fails_audit_report() {
        let warning_output = r#"
warning: redundant clone
  --> src/modules/instance.rs:42:15
   |
42 |     let b = a.clone();
   |               ^^^^^^^ help: remove this
   |
   = note: `#[warn(clippy::redundant_clone)]` on by default
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#redundant_clone

warning: `agm-ui` (lib) generated 1 warning (run `cargo clippy --fix` to apply 1 suggestion)
        "#;

        let report = parse_clippy_output(warning_output);

        assert!(!report.is_clean());
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.total_diagnostics, 1);
        assert!(report.has_warnings());
        assert!(!report.has_errors());

        let diag = &report.diagnostics[0];
        assert_eq!(diag.level, ClippyDiagnosticLevel::Warning);
        assert!(diag.is_warning());
        assert!(!diag.is_fatal());
        assert_eq!(diag.message, "redundant clone");
        assert_eq!(diag.code, Some("clippy::redundant_clone".to_string()));
        assert_eq!(diag.file_path, Some("src/modules/instance.rs".to_string()));
        assert_eq!(diag.line_number, Some(42));
    }

    #[test]
    fn test_compiler_error_fails_audit_report() {
        let error_output = r#"
error[E0425]: cannot find value `foo` in this scope
  --> src/modules/release_gate.rs:18:9
   |
18 |     foo();
   |     ^^^ not found in this scope

error: aborting due to 1 previous error
error: could not compile `agm-ui` (bin "agm-ui") due to 1 previous error
        "#;

        let report = parse_clippy_output(error_output);

        assert!(!report.is_clean());
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 0);
        assert_eq!(report.total_diagnostics, 1);
        assert!(report.has_errors());
        assert!(!report.has_warnings());

        let diag = &report.diagnostics[0];
        assert_eq!(diag.level, ClippyDiagnosticLevel::Error);
        assert!(diag.is_error());
        assert!(diag.is_fatal());
        assert_eq!(diag.code, Some("E0425".to_string()));
        assert_eq!(
            diag.file_path,
            Some("src/modules/release_gate.rs".to_string())
        );
        assert_eq!(diag.line_number, Some(18));
    }

    #[test]
    fn test_mixed_warnings_and_errors_detected() {
        let mixed_output = r#"
warning[clippy::needless_return]: unneeded `return` statement
  --> src/main.rs:25:5
   |
25 |     return 10;
   |     ^^^^^^^^^

error[E0308]: mismatched types
  --> src/main.rs:40:12
   |
40 |     let x: u32 = "str";
   |            ---   ^^^^^ expected `u32`, found `&str`

error: aborting due to 1 previous error; 1 warning emitted
        "#;

        let report = parse_clippy_output(mixed_output);

        assert!(!report.is_clean());
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.total_diagnostics, 2);
        assert!(report.has_errors());
        assert!(report.has_warnings());
    }

    #[test]
    fn test_json_lines_output_parsing() {
        let json_output = r#"
{"reason":"compiler-artifact","package_id":"agm-ui 4.251.0"}
{"reason":"compiler-message","package_id":"agm-ui 4.251.0","message":{"children":[],"code":{"code":"clippy::needless_borrow","explanation":null},"level":"warning","message":"this expression borrows a value","spans":[{"file_name":"src/foo.rs","line_start":33}]}}
{"reason":"compiler-message","package_id":"agm-ui 4.251.0","message":{"children":[],"code":{"code":"E0425","explanation":null},"level":"error","message":"cannot find value","spans":[{"file_name":"src/bar.rs","line_start":88}]}}
        "#;

        let report = parse_clippy_output(json_output);

        assert!(!report.is_clean());
        assert_eq!(report.total_diagnostics, 2);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.error_count, 1);
        assert!(report.has_errors());
        assert!(report.has_warnings());

        assert_eq!(
            report.diagnostics[0].code,
            Some("clippy::needless_borrow".to_string())
        );
        assert_eq!(
            report.diagnostics[0].file_path,
            Some("src/foo.rs".to_string())
        );
        assert_eq!(report.diagnostics[0].line_number, Some(33));

        assert_eq!(report.diagnostics[1].code, Some("E0425".to_string()));
        assert_eq!(
            report.diagnostics[1].file_path,
            Some("src/bar.rs".to_string())
        );
        assert_eq!(report.diagnostics[1].line_number, Some(88));
        assert!(report.diagnostics[1].is_fatal);
    }

    #[test]
    fn test_windows_drive_letter_source_location() {
        let loc = parse_source_location("--> C:\\work\\repo\\src\\lib.rs:55:12");
        assert_eq!(
            loc,
            Some(("C:\\work\\repo\\src\\lib.rs".to_string(), 55))
        );

        let posix_loc = parse_source_location("--> src/main.rs:102:4");
        assert_eq!(posix_loc, Some(("src/main.rs".to_string(), 102)));

        assert_eq!(parse_source_location("invalid line"), None);
    }

    #[test]
    fn test_extract_code_from_note() {
        assert_eq!(
            extract_code_from_note(" = note: `#[warn(clippy::needless_return)]` on by default"),
            Some("clippy::needless_return".to_string())
        );
        assert_eq!(
            extract_code_from_note(" = note: #[deny(clippy::all)] on by default"),
            Some("clippy::all".to_string())
        );
        assert_eq!(extract_code_from_note(" = note: standard note text"), None);
    }

    #[test]
    fn test_is_compiler_summary_line() {
        assert!(is_compiler_summary_line(
            "error: aborting due to 1 previous error"
        ));
        assert!(is_compiler_summary_line(
            "error: could not compile `agm-ui`"
        ));
        assert!(is_compiler_summary_line("warning: 1 warning emitted"));
        assert!(is_compiler_summary_line(
            "warning: `agm-ui` generated 2 warnings"
        ));
        assert!(!is_compiler_summary_line("warning: unused variable"));
        assert!(!is_compiler_summary_line("error[E0425]: cannot find value"));
    }

    #[test]
    fn test_parse_text_diagnostic_header() {
        let (level, code, msg) =
            parse_text_diagnostic_header("warning: unused variable: `a`").unwrap();
        assert_eq!(level, ClippyDiagnosticLevel::Warning);
        assert_eq!(code, None);
        assert_eq!(msg, "unused variable: `a`");

        let (level2, code2, msg2) =
            parse_text_diagnostic_header("error[E0425]: cannot find value `x`").unwrap();
        assert_eq!(level2, ClippyDiagnosticLevel::Error);
        assert_eq!(code2, Some("E0425".to_string()));
        assert_eq!(msg2, "cannot find value `x`");
    }

    #[test]
    fn test_serde_roundtrip_report_and_diagnostics() {
        let report = ClippyAuditReport {
            is_clean: false,
            total_diagnostics: 1,
            error_count: 0,
            warning_count: 1,
            diagnostics: vec![ClippyDiagnostic {
                level: ClippyDiagnosticLevel::Warning,
                message: "redundant clone".to_string(),
                code: Some("clippy::redundant_clone".to_string()),
                file_path: Some("src/main.rs".to_string()),
                line_number: Some(10),
                is_fatal: false,
            }],
        };

        let json = serde_json::to_string(&report).expect("Serialize ClippyAuditReport");
        let decoded: ClippyAuditReport =
            serde_json::from_str(&json).expect("Deserialize ClippyAuditReport");

        assert_eq!(report, decoded);
    }

    #[test]
    fn test_serde_roundtrip_levels() {
        let levels = [
            ClippyDiagnosticLevel::Error,
            ClippyDiagnosticLevel::Warning,
            ClippyDiagnosticLevel::Help,
            ClippyDiagnosticLevel::Note,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).expect("Serialize level");
            let decoded: ClippyDiagnosticLevel =
                serde_json::from_str(&json).expect("Deserialize level");
            assert_eq!(level, decoded);
        }
    }
}
