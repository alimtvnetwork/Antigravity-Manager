//! Tests for Cross-Platform Compatibility Guard.

#[cfg(test)]
mod tests {
    use super::super::cross_platform_compat_guard::{
        evaluate_file, evaluate_files, CompatReport, PlatformRuleViolation, SourceFile,
    };

    #[test]
    fn test_unguarded_windows_drive_letter_flagged() {
        let code = r#"
            pub fn read_config() -> String {
                let p = "C:\\data\\config.json";
                std::fs::read_to_string(p).unwrap_or_default()
            }
        "#;
        let file = SourceFile::new("src/config.rs", code);
        let report = evaluate_file(&file);

        assert!(!report.is_clean);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].rule_id,
            "CROSS_PLATFORM_NO_HARDCODED_DRIVE"
        );
        assert_eq!(report.findings[0].severity, "error");
        assert!(matches!(
            report.findings[0].violation,
            PlatformRuleViolation::HardcodedDriveLetterPath { .. }
        ));
    }

    #[test]
    fn test_cfg_target_os_windows_gated_drive_letter_passes() {
        let code = r#"
            #[cfg(target_os = "windows")]
            pub fn read_windows_default() -> String {
                let p = "C:\\data\\config.json";
                std::fs::read_to_string(p).unwrap_or_default()
            }
        "#;
        let file = SourceFile::new("src/win_config.rs", code);
        let report = evaluate_file(&file);

        assert!(
            report.is_clean,
            "Expected clean report for cfg-gated drive letter, got: {:?}",
            report.findings
        );
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_pathbuf_join_construction_passes() {
        let code = r#"
            use std::path::{Path, PathBuf};

            pub fn resolve_path(base: &Path, file_name: &str) -> PathBuf {
                base.join("subfolder").join(file_name)
            }
        "#;
        let file = SourceFile::new("src/path_helper.rs", code);
        let report = evaluate_file(&file);

        assert!(
            report.is_clean,
            "Expected clean report for PathBuf construction, got: {:?}",
            report.findings
        );
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_bare_cmd_spawn_flagged() {
        let code = r#"
            pub fn launch_shell() {
                let _ = std::process::Command::new("cmd.exe").arg("/c").spawn();
            }
        "#;
        let file = SourceFile::new("src/launcher.rs", code);
        let report = evaluate_file(&file);

        assert!(!report.is_clean);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].rule_id, "CROSS_PLATFORM_NO_BARE_SHELL");
        assert!(matches!(
            report.findings[0].violation,
            PlatformRuleViolation::OSSpecificShellInvocation { .. }
        ));
    }

    #[test]
    fn test_cfg_windows_gated_cmd_passes() {
        let code = r#"
            #[cfg(windows)]
            pub fn launch_win_shell() {
                let _ = std::process::Command::new("cmd.exe").arg("/c").spawn();
            }
        "#;
        let file = SourceFile::new("src/win_launcher.rs", code);
        let report = evaluate_file(&file);

        assert!(
            report.is_clean,
            "Expected clean report for cfg-gated cmd.exe, got: {:?}",
            report.findings
        );
    }

    #[test]
    fn test_bare_unix_shell_flagged() {
        let code = r#"
            pub fn run_bash() {
                std::process::Command::new("/bin/bash").spawn();
            }
        "#;
        let file = SourceFile::new("src/unix.rs", code);
        let report = evaluate_file(&file);

        assert!(!report.is_clean);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].rule_id, "CROSS_PLATFORM_NO_BARE_SHELL");
    }

    #[test]
    fn test_literal_path_separator_assumption_flagged() {
        let code = r#"
            pub fn make_path(dir: &str, file: &str) -> String {
                format!("{}\\{}", dir, file)
            }
        "#;
        let file = SourceFile::new("src/format_path.rs", code);
        let report = evaluate_file(&file);

        assert!(!report.is_clean);
        assert_eq!(
            report.findings[0].rule_id,
            "CROSS_PLATFORM_NO_LITERAL_SEPARATOR"
        );
        assert!(matches!(
            report.findings[0].violation,
            PlatformRuleViolation::LiteralPathSeparatorAssumption { .. }
        ));
    }

    #[test]
    fn test_unconditional_platform_api_flagged() {
        let code = r#"
            pub fn configure_job() {
                let _flags = CREATE_BREAKAWAY_FROM_JOB;
            }
        "#;
        let file = SourceFile::new("src/job.rs", code);
        let report = evaluate_file(&file);

        assert!(!report.is_clean);
        assert_eq!(
            report.findings[0].rule_id,
            "CROSS_PLATFORM_NO_UNCONDITIONAL_PLATFORM_API"
        );
        assert!(matches!(
            report.findings[0].violation,
            PlatformRuleViolation::UnconditionalPlatformApi { .. }
        ));
    }

    #[test]
    fn test_serde_roundtrip_compat_report() {
        let file1 = SourceFile::new("src/good.rs", "fn ok() {}");
        let file2 = SourceFile::new("src/bad.rs", "fn bad() { let p = \"C:\\data\"; }");
        let report = evaluate_files(&[file1, file2]);

        assert!(!report.is_clean);
        assert_eq!(report.total_findings, 1);
        assert_eq!(report.file_reports.len(), 2);

        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: CompatReport = serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(report, deserialized);
    }
}
