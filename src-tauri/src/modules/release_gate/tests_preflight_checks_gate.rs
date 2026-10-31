//! Unit tests for Preflight Checks Gate (`Imp-640`).

#[cfg(test)]
mod tests {
    use crate::modules::release_gate::preflight_checks_gate::{
        evaluate_preflight_results, format_preflight_summary, PreflightCheckKind,
        PreflightCheckResult, PreflightCheckStatus, PreflightReport,
    };

    #[test]
    fn test_all_blocking_checks_passed_is_ready_for_release() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 120, true),
            PreflightCheckResult::passed(PreflightCheckKind::CargoClippyAllFeatures, 4500, true),
            PreflightCheckResult::passed(PreflightCheckKind::NpmRunBuild, 3200, true),
        ];

        let report = evaluate_preflight_results(&results);

        assert!(report.is_ready_for_release);
        assert_eq!(report.total_checks, 3);
        assert_eq!(report.passed_count, 3);
        assert_eq!(report.failed_count, 0);
        assert!(!report.has_failures());
        assert!(!report.has_blocking_failures());
        assert_eq!(report.skipped_count(), 0);
    }

    #[test]
    fn test_one_blocking_check_failed_blocks_release() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 110, true),
            PreflightCheckResult::failed(
                PreflightCheckKind::CargoClippyAllFeatures,
                "clippy found 2 warnings in src-tauri/src/error.rs",
                3800,
                true,
            ),
            PreflightCheckResult::passed(PreflightCheckKind::NpmRunBuild, 2900, true),
        ];

        let report = evaluate_preflight_results(&results);

        assert!(!report.is_ready_for_release);
        assert_eq!(report.total_checks, 3);
        assert_eq!(report.passed_count, 2);
        assert_eq!(report.failed_count, 1);
        assert!(report.has_failures());
        assert!(report.has_blocking_failures());
    }

    #[test]
    fn test_non_blocking_check_failed_passes_with_warning() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 100, true),
            PreflightCheckResult::passed(PreflightCheckKind::CargoClippyAllFeatures, 4100, true),
            PreflightCheckResult::failed(
                PreflightCheckKind::CustomStep("cargo audit".to_string()),
                "advisory warning found",
                800,
                false,
            ),
        ];

        let report = evaluate_preflight_results(&results);

        assert!(report.is_ready_for_release);
        assert_eq!(report.total_checks, 3);
        assert_eq!(report.passed_count, 2);
        assert_eq!(report.failed_count, 1);
        assert!(report.has_failures());
        assert!(!report.has_blocking_failures());
    }

    #[test]
    fn test_skipped_check_does_not_block_release() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 95, true),
            PreflightCheckResult::passed(PreflightCheckKind::CargoClippyAllFeatures, 3900, true),
            PreflightCheckResult::skipped(
                PreflightCheckKind::NpmRunBuild,
                "no frontend files modified in this changeset",
                true,
            ),
        ];

        let report = evaluate_preflight_results(&results);

        assert!(report.is_ready_for_release);
        assert_eq!(report.total_checks, 3);
        assert_eq!(report.passed_count, 2);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.skipped_count(), 1);
        assert!(!report.has_failures());
    }

    #[test]
    fn test_check_status_helpers() {
        let pass = PreflightCheckStatus::Passed;
        assert!(pass.is_passed());
        assert!(!pass.is_failed());
        assert!(!pass.is_skipped());
        assert_eq!(pass.error_message(), None);
        assert_eq!(pass.skip_reason(), None);

        let fail = PreflightCheckStatus::Failed {
            error: "build syntax error".to_string(),
        };
        assert!(!fail.is_passed());
        assert!(fail.is_failed());
        assert!(!fail.is_skipped());
        assert_eq!(fail.error_message(), Some("build syntax error"));
        assert_eq!(fail.skip_reason(), None);

        let skip = PreflightCheckStatus::Skipped {
            reason: "trivial edit".to_string(),
        };
        assert!(!skip.is_passed());
        assert!(!skip.is_failed());
        assert!(skip.is_skipped());
        assert_eq!(skip.error_message(), None);
        assert_eq!(skip.skip_reason(), Some("trivial edit"));
    }

    #[test]
    fn test_check_kind_command_strings_and_display() {
        let fmt = PreflightCheckKind::CargoFmtCheck;
        assert_eq!(fmt.command_str(), "cd src-tauri && cargo fmt -- --check");
        assert_eq!(fmt.to_string(), "cargo fmt -- --check");

        let clippy = PreflightCheckKind::CargoClippyAllFeatures;
        assert_eq!(
            clippy.command_str(),
            "cd src-tauri && cargo clippy --all-targets --all-features"
        );
        assert_eq!(
            clippy.to_string(),
            "cargo clippy --all-targets --all-features"
        );

        let npm = PreflightCheckKind::NpmRunBuild;
        assert_eq!(npm.command_str(), "npm run build");
        assert_eq!(npm.to_string(), "npm run build");

        let custom = PreflightCheckKind::CustomStep("cargo test --no-run".to_string());
        assert_eq!(custom.command_str(), "cargo test --no-run");
        assert_eq!(custom.to_string(), "custom: cargo test --no-run");
    }

    #[test]
    fn test_format_summary_all_passed() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 100, true),
            PreflightCheckResult::passed(PreflightCheckKind::CargoClippyAllFeatures, 4000, true),
        ];
        let report = evaluate_preflight_results(&results);
        let summary = format_preflight_summary(&report);

        assert!(summary.contains("READY FOR RELEASE"));
        assert!(summary.contains("Summary: Total: 2, Passed: 2, Failed: 0, Skipped: 0"));
        assert!(summary.contains("All pre-flight checks passed successfully."));
    }

    #[test]
    fn test_format_summary_blocked_failure() {
        let results = vec![PreflightCheckResult::failed(
            PreflightCheckKind::CargoFmtCheck,
            "formatting diff detected",
            50,
            true,
        )];
        let report = evaluate_preflight_results(&results);
        let summary = format_preflight_summary(&report);

        assert!(summary.contains("BLOCKED FROM RELEASE"));
        assert!(summary.contains("BLOCKING"));
        assert!(summary.contains("FAILED: formatting diff detected"));
        assert!(summary.contains("Fix violations before tagging or submitting PR."));
    }

    #[test]
    fn test_format_summary_passed_with_warnings() {
        let results = vec![
            PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 100, true),
            PreflightCheckResult::failed(
                PreflightCheckKind::CustomStep("linter".to_string()),
                "soft warning",
                50,
                false,
            ),
        ];
        let report = evaluate_preflight_results(&results);
        let summary = format_preflight_summary(&report);

        assert!(summary.contains("READY FOR RELEASE"));
        assert!(summary.contains("OPTIONAL"));
        assert!(summary.contains("Passed with warnings"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let report = PreflightReport {
            is_ready_for_release: true,
            total_checks: 2,
            passed_count: 1,
            failed_count: 0,
            results: vec![
                PreflightCheckResult::passed(PreflightCheckKind::CargoFmtCheck, 150, true),
                PreflightCheckResult::skipped(
                    PreflightCheckKind::NpmRunBuild,
                    "no frontend changes",
                    true,
                ),
            ],
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: PreflightReport =
            serde_json::from_str(&json).expect("deserialize report");

        assert_eq!(report, deserialized);
    }
}
