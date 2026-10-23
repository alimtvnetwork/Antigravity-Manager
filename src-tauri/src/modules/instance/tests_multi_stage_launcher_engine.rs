//! Unit tests for Multi-Stage Launcher Engine & Batch Execution Guard (`Imp-650`).

#[cfg(test)]
mod tests {
    use crate::modules::instance::multi_stage_launcher_engine::{
        build_launch_plan, build_shell_dispatch_args, detect_executable_kind, is_batch_script,
        resolve_breakaway_flags, ExecutableKind, ExecutionStage, LaunchPlan,
        CREATE_BREAKAWAY_FROM_JOB, ERROR_ACCESS_DENIED,
    };

    #[test]
    fn test_native_binary_with_breakaway_enabled() {
        let args = vec!["--user-data-dir=data".to_string(), "--headless".to_string()];
        let plan = build_launch_plan(
            "target/bin/antigravity.exe",
            &args,
            None,
            true,
            None,
        );

        assert_eq!(plan.program, "target/bin/antigravity.exe");
        assert_eq!(plan.args, args);
        assert_eq!(plan.creation_flags, CREATE_BREAKAWAY_FROM_JOB);
        assert!(plan.has_breakaway());
        assert!(!plan.is_batch_guarded());
        assert_eq!(plan.stage, ExecutionStage::TargetBinary);
        assert_eq!(plan.executable_kind, ExecutableKind::NativeBinary);
        assert!(plan.stage.is_target_binary());
        assert!(plan.executable_kind.is_native());
    }

    #[test]
    fn test_native_binary_with_breakaway_disabled() {
        let args = vec!["--version".to_string()];
        let plan = build_launch_plan("antigravity.exe", &args, None, false, None);

        assert_eq!(plan.program, "antigravity.exe");
        assert_eq!(plan.creation_flags, 0);
        assert!(!plan.has_breakaway());
        assert_eq!(plan.stage, ExecutionStage::TargetBinary);
    }

    #[test]
    fn test_os_error_5_fallback_removes_breakaway_flag() {
        let args = vec!["--port=9000".to_string()];
        let plan = build_launch_plan(
            "target/tools/app.exe",
            &args,
            None,
            true,
            Some(ERROR_ACCESS_DENIED),
        );

        assert_eq!(plan.program, "target/tools/app.exe");
        assert_eq!(plan.args, args);
        assert_eq!(plan.creation_flags, 0);
        assert!(!plan.has_breakaway());
        assert_eq!(plan.stage, ExecutionStage::TargetBinary);
        assert_eq!(plan.executable_kind, ExecutableKind::NativeBinary);
        assert!(!plan.is_batch_guarded());
    }

    #[test]
    fn test_batch_script_bat_routing_to_detected_base_binary() {
        let args = vec!["--profile".to_string(), "worker".to_string()];
        let base_binary = "target/base/antigravity.exe";
        let plan = build_launch_plan(
            "scripts/launcher.bat",
            &args,
            Some(base_binary),
            true,
            None,
        );

        assert_eq!(plan.program, base_binary);
        assert_eq!(plan.args, args);
        assert_eq!(plan.stage, ExecutionStage::BaseBinaryFallback);
        assert!(plan.stage.is_base_binary_fallback());
        assert_eq!(plan.executable_kind, ExecutableKind::NativeBinary);
        assert!(plan.is_batch_guarded());
        assert!(plan.has_breakaway());
        assert_eq!(plan.creation_flags, CREATE_BREAKAWAY_FROM_JOB);
    }

    #[test]
    fn test_batch_script_cmd_routing_to_detected_base_binary() {
        let args = vec!["--sandbox".to_string()];
        let base_binary = "target/base/antigravity.exe";
        let plan = build_launch_plan(
            "scripts/runner.CMD",
            &args,
            Some(base_binary),
            true,
            None,
        );

        assert_eq!(plan.program, base_binary);
        assert_eq!(plan.args, args);
        assert_eq!(plan.stage, ExecutionStage::BaseBinaryFallback);
        assert!(plan.is_batch_guarded());
    }

    #[test]
    fn test_batch_script_routing_to_windows_shell_dispatch_when_base_binary_is_none() {
        let args = vec!["--port".to_string(), "8080".to_string()];
        let plan = build_launch_plan("scripts/run.bat", &args, None, true, None);

        assert_eq!(plan.program, "cmd.exe");
        assert_eq!(
            plan.args,
            vec![
                "/c".to_string(),
                "start".to_string(),
                "scripts/run.bat".to_string(),
                "--port".to_string(),
                "8080".to_string(),
            ]
        );
        assert_eq!(plan.stage, ExecutionStage::ShellDispatchFallback);
        assert!(plan.stage.is_shell_dispatch_fallback());
        assert_eq!(plan.executable_kind, ExecutableKind::ShellWrapper);
        assert!(plan.executable_kind.is_shell_wrapper());
        assert!(plan.is_batch_guarded());
        assert!(plan.has_breakaway());
    }

    #[test]
    fn test_batch_script_routing_to_shell_dispatch_when_base_binary_is_empty_or_whitespace() {
        let args = vec!["--arg".to_string()];
        let plan_empty = build_launch_plan("start.cmd", &args, Some(""), true, None);
        assert_eq!(plan_empty.program, "cmd.exe");
        assert_eq!(plan_empty.stage, ExecutionStage::ShellDispatchFallback);

        let plan_whitespace = build_launch_plan("start.cmd", &args, Some("   "), true, None);
        assert_eq!(plan_whitespace.program, "cmd.exe");
        assert_eq!(plan_whitespace.stage, ExecutionStage::ShellDispatchFallback);
    }

    #[test]
    fn test_case_insensitive_extension_checks() {
        assert!(is_batch_script("run.bat"));
        assert!(is_batch_script("RUN.BAT"));
        assert!(is_batch_script("runner.Bat"));
        assert!(is_batch_script("start.cmd"));
        assert!(is_batch_script("START.CMD"));
        assert!(is_batch_script("Script.Cmd"));
        assert!(is_batch_script("\"quoted\\path.bat\""));
        assert!(is_batch_script("'single_quoted.cmd'"));

        assert!(!is_batch_script("app.exe"));
        assert!(!is_batch_script("code.sh"));
        assert!(!is_batch_script("deploy.ps1"));
        assert!(!is_batch_script("cmd.exe"));
        assert!(!is_batch_script("batch_file.js"));
        assert!(!is_batch_script(""));
    }

    #[test]
    fn test_detect_executable_kind() {
        assert_eq!(
            detect_executable_kind("runner.bat"),
            ExecutableKind::WindowsBatchScript
        );
        assert_eq!(
            detect_executable_kind("RUNNER.CMD"),
            ExecutableKind::WindowsBatchScript
        );
        assert_eq!(
            detect_executable_kind("cmd.exe"),
            ExecutableKind::ShellWrapper
        );
        assert_eq!(
            detect_executable_kind("windows/system32/cmd.exe"),
            ExecutableKind::ShellWrapper
        );
        assert_eq!(
            detect_executable_kind("powershell.exe"),
            ExecutableKind::ShellWrapper
        );
        assert_eq!(
            detect_executable_kind("app.exe"),
            ExecutableKind::NativeBinary
        );
        assert_eq!(
            detect_executable_kind("bin/code"),
            ExecutableKind::NativeBinary
        );
    }

    #[test]
    fn test_resolve_breakaway_flags_matrix() {
        assert_eq!(
            resolve_breakaway_flags(true, None),
            (CREATE_BREAKAWAY_FROM_JOB, true)
        );
        assert_eq!(resolve_breakaway_flags(false, None), (0, false));
        assert_eq!(
            resolve_breakaway_flags(true, Some(ERROR_ACCESS_DENIED)),
            (0, false)
        );
        assert_eq!(
            resolve_breakaway_flags(true, Some(2)),
            (CREATE_BREAKAWAY_FROM_JOB, true)
        );
    }

    #[test]
    fn test_build_shell_dispatch_args_construction() {
        let args = vec!["--flag".to_string(), "val".to_string()];
        let dispatch = build_shell_dispatch_args("scripts/launch.bat", &args);
        assert_eq!(dispatch.len(), 4);
        assert_eq!(dispatch[0], "/c");
        assert_eq!(dispatch[1], "start");
        assert_eq!(dispatch[2], "scripts/launch.bat");
        assert_eq!(dispatch[3], "--flag");
    }

    #[test]
    fn test_serde_roundtrip_launch_plan() {
        let plan = LaunchPlan {
            program: "cmd.exe".to_string(),
            args: vec!["/c".to_string(), "start".to_string(), "run.bat".to_string()],
            creation_flags: CREATE_BREAKAWAY_FROM_JOB,
            stage: ExecutionStage::ShellDispatchFallback,
            executable_kind: ExecutableKind::ShellWrapper,
            is_batch_guarded: true,
            has_breakaway: true,
        };

        let json = serde_json::to_string(&plan).expect("Serialize LaunchPlan");
        let decoded: LaunchPlan = serde_json::from_str(&json).expect("Deserialize LaunchPlan");

        assert_eq!(plan, decoded);
    }

    #[test]
    fn test_serde_roundtrip_enums() {
        let stages = [
            ExecutionStage::TargetBinary,
            ExecutionStage::BaseBinaryFallback,
            ExecutionStage::ShellDispatchFallback,
        ];
        for stage in stages {
            let json = serde_json::to_string(&stage).expect("Serialize ExecutionStage");
            let decoded: ExecutionStage =
                serde_json::from_str(&json).expect("Deserialize ExecutionStage");
            assert_eq!(stage, decoded);
        }

        let kinds = [
            ExecutableKind::NativeBinary,
            ExecutableKind::WindowsBatchScript,
            ExecutableKind::ShellWrapper,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("Serialize ExecutableKind");
            let decoded: ExecutableKind =
                serde_json::from_str(&json).expect("Deserialize ExecutableKind");
            assert_eq!(kind, decoded);
        }
    }
}
