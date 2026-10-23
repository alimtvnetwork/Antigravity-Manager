//! Unit tests for Multi-Stage Launcher Dispatcher (`Imp-628`).

#[cfg(test)]
mod tests {
    use crate::modules::instance::multi_stage_launcher_dispatcher::{
        dispatch_and_report, extract_batch_extension, is_disallowed_batch_script,
        plan_multi_stage_launch, resolve_job_breakaway_mode, JobBreakawayMode, LaunchDispatchPlan,
        LaunchDispatchReport, LaunchStage, LauncherSecurityViolation, LauncherTargetSpec,
    };

    #[test]
    fn test_stage1_selected_when_valid_target_binary_provided() {
        let spec = LauncherTargetSpec::new()
            .with_target("bin/antigravity.exe")
            .with_args(vec!["--profile".to_string(), "p1".to_string()])
            .with_shell_fallback(false);

        let plan =
            plan_multi_stage_launch(&spec, false).expect("Stage 1 launch plan should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage1TargetBinary);
        assert_eq!(plan.executable, "bin/antigravity.exe");
        assert_eq!(plan.resolved_args, vec!["--profile", "p1"]);
        assert_eq!(plan.breakaway_mode, JobBreakawayMode::BreakawayRequested);
        assert!(!plan.is_shell_stage());
        assert!(plan.is_breakaway_requested());
        assert!(!plan.is_breakaway_denied_fallback());
    }

    #[test]
    fn test_stage1_under_restricted_job_object_steps_down_breakaway() {
        let spec = LauncherTargetSpec::new()
            .with_target("bin/antigravity.exe")
            .with_args(vec!["--headless".to_string()]);

        let plan =
            plan_multi_stage_launch(&spec, true).expect("Stage 1 restricted launch should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage1TargetBinary);
        assert_eq!(plan.executable, "bin/antigravity.exe");
        assert_eq!(
            plan.breakaway_mode,
            JobBreakawayMode::BreakawayDeniedFallback
        );
        assert!(plan.is_breakaway_denied_fallback());
        assert!(!plan.is_breakaway_requested());
    }

    #[test]
    fn test_stage2_fallback_when_target_binary_is_none() {
        let spec = LauncherTargetSpec::new()
            .with_base("base/antigravity.exe")
            .with_args(vec!["--inspect".to_string()])
            .with_shell_fallback(false);

        let plan = plan_multi_stage_launch(&spec, false).expect("Stage 2 fallback should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage2BaseBinary);
        assert_eq!(plan.executable, "base/antigravity.exe");
        assert_eq!(plan.resolved_args, vec!["--inspect"]);
        assert_eq!(plan.breakaway_mode, JobBreakawayMode::BreakawayRequested);
        assert!(!plan.is_shell_stage());
    }

    #[test]
    fn test_stage2_fallback_when_target_binary_is_empty_string() {
        let spec = LauncherTargetSpec::new()
            .with_target("")
            .with_base("base/antigravity.exe")
            .with_args(vec!["--port".to_string(), "9000".to_string()]);

        let plan = plan_multi_stage_launch(&spec, false)
            .expect("Stage 2 empty target fallback should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage2BaseBinary);
        assert_eq!(plan.executable, "base/antigravity.exe");
    }

    #[test]
    fn test_stage2_under_restricted_job_object_steps_down_breakaway() {
        let spec = LauncherTargetSpec::new()
            .with_base("base/antigravity.exe")
            .with_args(vec!["--verbose".to_string()]);

        let plan =
            plan_multi_stage_launch(&spec, true).expect("Stage 2 restricted launch should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage2BaseBinary);
        assert_eq!(
            plan.breakaway_mode,
            JobBreakawayMode::BreakawayDeniedFallback
        );
        assert!(plan.is_breakaway_denied_fallback());
    }

    #[test]
    fn test_stage3_windows_shell_fallback_when_target_and_base_are_none() {
        let spec = LauncherTargetSpec::new()
            .with_args(vec!["--data-dir".to_string(), "profiles/p1".to_string()])
            .with_shell_fallback(true);

        let plan =
            plan_multi_stage_launch(&spec, false).expect("Stage 3 shell fallback should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage3WindowsShell);
        assert_eq!(plan.executable, "cmd.exe");
        assert_eq!(
            plan.resolved_args,
            vec!["/c", "start", "\"\"", "--data-dir", "profiles/p1"]
        );
        assert_eq!(
            plan.breakaway_mode,
            JobBreakawayMode::BreakawayNotApplicable
        );
        assert!(plan.is_shell_stage());
    }

    #[test]
    fn test_stage3_windows_shell_fallback_when_target_and_base_are_empty_strings() {
        let spec = LauncherTargetSpec::new()
            .with_target("")
            .with_base("")
            .with_shell_fallback(true);

        let plan = plan_multi_stage_launch(&spec, false).expect("Stage 3 fallback should succeed");

        assert_eq!(plan.selected_stage, LaunchStage::Stage3WindowsShell);
        assert_eq!(plan.executable, "cmd.exe");
        assert_eq!(plan.resolved_args, vec!["/c", "start", "\"\""]);
        assert_eq!(
            plan.breakaway_mode,
            JobBreakawayMode::BreakawayNotApplicable
        );
    }

    #[test]
    fn test_target_batch_file_disallowed_bat() {
        let spec = LauncherTargetSpec::new()
            .with_target("scripts/launch.bat")
            .with_shell_fallback(true);

        let err = plan_multi_stage_launch(&spec, false)
            .expect_err("Direct .bat execution must be rejected");

        match err {
            LauncherSecurityViolation::DirectBatchFileExecutionDisallowed { path, extension } => {
                assert_eq!(path, "scripts/launch.bat");
                assert_eq!(extension, ".bat");
            }
            _ => panic!("Expected DirectBatchFileExecutionDisallowed"),
        }
        assert!(err.is_batch_disallowed());
        assert!(!err.is_all_stages_failed());
    }

    #[test]
    fn test_target_batch_file_disallowed_cmd() {
        let spec = LauncherTargetSpec::new()
            .with_target("scripts/start.cmd")
            .with_shell_fallback(true);

        let err = plan_multi_stage_launch(&spec, false)
            .expect_err("Direct .cmd execution must be rejected");

        match err {
            LauncherSecurityViolation::DirectBatchFileExecutionDisallowed { path, extension } => {
                assert_eq!(path, "scripts/start.cmd");
                assert_eq!(extension, ".cmd");
            }
            _ => panic!("Expected DirectBatchFileExecutionDisallowed"),
        }
    }

    #[test]
    fn test_target_batch_file_case_insensitive() {
        let spec_upper_bat = LauncherTargetSpec::new().with_target("LAUNCH.BAT");
        let err_bat = plan_multi_stage_launch(&spec_upper_bat, false).unwrap_err();
        assert!(matches!(
            err_bat,
            LauncherSecurityViolation::DirectBatchFileExecutionDisallowed { ref extension, .. } if extension == ".bat"
        ));

        let spec_upper_cmd = LauncherTargetSpec::new().with_target("LAUNCH.CMD");
        let err_cmd = plan_multi_stage_launch(&spec_upper_cmd, false).unwrap_err();
        assert!(matches!(
            err_cmd,
            LauncherSecurityViolation::DirectBatchFileExecutionDisallowed { ref extension, .. } if extension == ".cmd"
        ));
    }

    #[test]
    fn test_base_batch_file_disallowed_when_target_is_none() {
        let spec = LauncherTargetSpec::new().with_base("base/launcher.bat");

        let err = plan_multi_stage_launch(&spec, false)
            .expect_err("Base .bat execution must be rejected");

        match err {
            LauncherSecurityViolation::DirectBatchFileExecutionDisallowed { path, extension } => {
                assert_eq!(path, "base/launcher.bat");
                assert_eq!(extension, ".bat");
            }
            _ => panic!("Expected DirectBatchFileExecutionDisallowed"),
        }
    }

    #[test]
    fn test_all_stages_failed_when_shell_fallback_disabled() {
        let spec = LauncherTargetSpec::new()
            .with_args(vec!["--inspect".to_string()])
            .with_shell_fallback(false);

        let err = plan_multi_stage_launch(&spec, false)
            .expect_err("All stages must fail when shell disabled");

        match err {
            LauncherSecurityViolation::AllLaunchStagesFailed {
                ref attempted_stages,
                ref error_details,
            } => {
                assert_eq!(attempted_stages.len(), 3);
                assert_eq!(attempted_stages[0], LaunchStage::Stage1TargetBinary);
                assert_eq!(attempted_stages[1], LaunchStage::Stage2BaseBinary);
                assert_eq!(attempted_stages[2], LaunchStage::Stage3WindowsShell);
                assert_eq!(error_details.len(), 3);
            }
            _ => panic!("Expected AllLaunchStagesFailed"),
        }
        assert!(err.is_all_stages_failed());
        assert!(!err.is_batch_disallowed());
    }

    #[test]
    fn test_all_stages_failed_when_both_target_and_base_are_empty() {
        let spec = LauncherTargetSpec::new()
            .with_target("")
            .with_base("")
            .with_shell_fallback(false);

        let err = plan_multi_stage_launch(&spec, false)
            .expect_err("Empty targets must fail without shell");
        assert!(err.is_all_stages_failed());
    }

    #[test]
    fn test_is_disallowed_batch_script_classification() {
        assert!(is_disallowed_batch_script("run.bat"));
        assert!(is_disallowed_batch_script("run.cmd"));
        assert!(is_disallowed_batch_script("run.BAT"));
        assert!(is_disallowed_batch_script("run.CMD"));
        assert!(is_disallowed_batch_script("  run.bat  "));
        assert!(is_disallowed_batch_script("path/to/script.bat"));

        assert!(!is_disallowed_batch_script("run.exe"));
        assert!(!is_disallowed_batch_script("run.sh"));
        assert!(!is_disallowed_batch_script("cmd.exe"));
        assert!(!is_disallowed_batch_script(""));
        assert!(!is_disallowed_batch_script("   "));
    }

    #[test]
    fn test_extract_batch_extension() {
        assert_eq!(extract_batch_extension("run.bat"), ".bat");
        assert_eq!(extract_batch_extension("run.cmd"), ".cmd");
        assert_eq!(extract_batch_extension("RUN.BAT"), ".bat");
        assert_eq!(extract_batch_extension("RUN.CMD"), ".cmd");
    }

    #[test]
    fn test_resolve_job_breakaway_mode() {
        assert_eq!(
            resolve_job_breakaway_mode(true),
            JobBreakawayMode::BreakawayDeniedFallback
        );
        assert_eq!(
            resolve_job_breakaway_mode(false),
            JobBreakawayMode::BreakawayRequested
        );
    }

    #[test]
    fn test_violation_display_formatting_and_error_trait() {
        let violation_batch = LauncherSecurityViolation::DirectBatchFileExecutionDisallowed {
            path: "test.bat".to_string(),
            extension: ".bat".to_string(),
        };
        let formatted_batch = format!("{}", violation_batch);
        assert!(formatted_batch.contains("Direct execution of Windows batch file is disallowed"));
        assert!(formatted_batch.contains("test.bat"));
        assert!(formatted_batch.contains(".bat"));

        let violation_failed = LauncherSecurityViolation::AllLaunchStagesFailed {
            attempted_stages: vec![LaunchStage::Stage1TargetBinary],
            error_details: vec!["Target binary missing".to_string()],
        };
        let formatted_failed = format!("{}", violation_failed);
        assert!(formatted_failed.contains("All launch stages failed"));
        assert!(formatted_failed.contains("Target binary missing"));
    }

    #[test]
    fn test_dispatch_and_report_success() {
        let spec = LauncherTargetSpec::new().with_target("bin/app.exe");
        let report = dispatch_and_report(&spec, false);

        assert!(report.is_success);
        assert!(report.is_successful());
        assert!(report.plan.is_some());
        assert!(report.violation.is_none());
    }

    #[test]
    fn test_dispatch_and_report_failure() {
        let spec = LauncherTargetSpec::new().with_shell_fallback(false);
        let report = dispatch_and_report(&spec, false);

        assert!(!report.is_success);
        assert!(!report.is_successful());
        assert!(report.plan.is_none());
        assert!(report.violation.is_some());
    }

    #[test]
    fn test_launch_stage_and_breakaway_mode_methods() {
        assert!(LaunchStage::Stage1TargetBinary.is_stage1());
        assert!(!LaunchStage::Stage1TargetBinary.is_stage2());
        assert!(!LaunchStage::Stage1TargetBinary.is_stage3());

        assert!(LaunchStage::Stage2BaseBinary.is_stage2());
        assert!(LaunchStage::Stage3WindowsShell.is_stage3());

        assert_eq!(
            LaunchStage::Stage1TargetBinary.as_str(),
            "stage_1_target_binary"
        );
        assert_eq!(
            format!("{}", LaunchStage::Stage1TargetBinary),
            "stage_1_target_binary"
        );

        assert!(JobBreakawayMode::BreakawayRequested.is_breakaway_requested());
        assert!(JobBreakawayMode::BreakawayDeniedFallback.is_breakaway_denied_fallback());
        assert!(JobBreakawayMode::BreakawayNotApplicable.is_breakaway_not_applicable());

        assert_eq!(
            JobBreakawayMode::BreakawayRequested.as_str(),
            "breakaway_requested"
        );
        assert_eq!(
            format!("{}", JobBreakawayMode::BreakawayRequested),
            "breakaway_requested"
        );
    }

    #[test]
    fn test_launcher_target_spec_builder() {
        let spec = LauncherTargetSpec::new()
            .with_target("target.exe")
            .with_base("base.exe")
            .with_args(["-a", "-b"])
            .with_shell_fallback(true);

        assert_eq!(spec.target_binary_path, Some("target.exe".to_string()));
        assert_eq!(spec.detected_base_binary_path, Some("base.exe".to_string()));
        assert_eq!(spec.arguments, vec!["-a", "-b"]);
        assert!(spec.allow_shell_fallback);
    }
}
