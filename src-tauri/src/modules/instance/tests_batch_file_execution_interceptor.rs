//! Unit tests for Batch File Execution Interceptor (`Imp-698`).

#[cfg(test)]
mod tests {
    use super::super::batch_file_execution_interceptor::*;

    #[test]
    fn test_classify_and_detect_batch_files_case_insensitive() {
        assert!(is_windows_batch_file("run.bat"));
        assert!(is_windows_batch_file("RUN.BAT"));
        assert!(is_windows_batch_file("script.cmd"));
        assert!(is_windows_batch_file("C:\\tools\\launcher.CMD"));
        assert!(is_windows_batch_file("\"C:\\Program Files\\app.bat\""));
        assert!(is_windows_batch_file("'run_script.cmd'"));
        assert!(is_windows_batch_file("  D:\\scripts\\start.cmd  "));

        assert_eq!(classify_script_extension("run.bat"), ScriptExtension::Bat);
        assert_eq!(classify_script_extension("RUN.BAT"), ScriptExtension::Bat);
        assert_eq!(classify_script_extension("start.cmd"), ScriptExtension::Cmd);
        assert_eq!(classify_script_extension("START.CMD"), ScriptExtension::Cmd);
    }

    #[test]
    fn test_native_executables_not_detected_as_batch() {
        assert!(!is_windows_batch_file("app.exe"));
        assert!(!is_windows_batch_file("/usr/bin/node"));
        assert!(!is_windows_batch_file("script.sh"));
        assert!(!is_windows_batch_file("binary_without_extension"));
        assert!(!is_windows_batch_file(""));

        assert_eq!(
            classify_script_extension("app.exe"),
            ScriptExtension::Other("exe".to_string())
        );
        assert_eq!(
            classify_script_extension("/usr/bin/node"),
            ScriptExtension::Other("".to_string())
        );
        assert_eq!(
            classify_script_extension("script.ps1"),
            ScriptExtension::Other("ps1".to_string())
        );
    }

    #[test]
    fn test_batch_interceptor_verdict() {
        let bat_verdict = evaluate_verdict("C:\\scripts\\setup.bat");
        assert!(bat_verdict.is_forbidden());
        match bat_verdict {
            BatchInterceptorVerdict::DirectExecutionForbidden { extension, .. } => {
                assert_eq!(extension, "bat");
            }
            _ => panic!("Expected DirectExecutionForbidden for .bat"),
        }

        let cmd_verdict = evaluate_verdict("setup.cmd");
        assert!(cmd_verdict.is_forbidden());

        let exe_verdict = evaluate_verdict("antigravity.exe");
        assert!(!exe_verdict.is_forbidden());
        assert_eq!(exe_verdict, BatchInterceptorVerdict::SafeNativeExecutable);

        let empty_verdict = evaluate_verdict("   ");
        assert_eq!(empty_verdict, BatchInterceptorVerdict::EmptyPath);
    }

    #[test]
    fn test_sanitize_batch_arg() {
        assert_eq!(sanitize_batch_arg(""), "");
        assert_eq!(sanitize_batch_arg("normal_arg"), "normal_arg");
        assert_eq!(sanitize_batch_arg("var%USER%"), "var%%USER%%");
        assert_eq!(sanitize_batch_arg("cmd1&cmd2"), "cmd1^&cmd2");
        assert_eq!(sanitize_batch_arg("val<tag>"), "val^<tag^>");
        assert_eq!(sanitize_batch_arg("pipe|target"), "pipe^|target");
        assert_eq!(sanitize_batch_arg("caret^char"), "caret^^char");
        assert_eq!(sanitize_batch_arg("quoted\"value"), "quoted\\\"value");
    }

    #[test]
    fn test_intercept_and_plan_execution() {
        let batch_path = "C:\\launcher.bat";
        let args = vec!["--port=8080".to_string(), "foo&bar".to_string()];

        let plan = intercept_and_plan_execution(batch_path, &args, false).unwrap();
        assert_eq!(plan.executable_path, "cmd.exe");
        assert!(plan.uses_shell_wrapper);
        assert_eq!(plan.shell_kind, Some("cmd.exe".to_string()));
        assert_eq!(plan.arguments[0], "/c");
        assert_eq!(plan.arguments[1], "call");
        assert_eq!(plan.arguments[2], "\"C:\\launcher.bat\"");
        assert_eq!(plan.arguments[3], "--port=8080");
        assert_eq!(plan.arguments[4], "foo^&bar");

        let native_plan = intercept_and_plan_execution("node.exe", &args, false).unwrap();
        assert_eq!(native_plan.executable_path, "node.exe");
        assert!(!native_plan.uses_shell_wrapper);
        assert_eq!(native_plan.shell_kind, None);
        assert_eq!(native_plan.arguments, args);
    }

    #[test]
    fn test_classify_execution_target() {
        let direct = classify_execution_target("C:\\App\\antigravity.exe");
        assert!(direct.is_direct_binary());
        assert!(!direct.is_batch());
        assert_eq!(direct.target_path(), "C:\\App\\antigravity.exe");

        let batch = classify_execution_target("C:\\App\\run.bat");
        assert!(batch.is_batch());
        assert!(!batch.is_direct_binary());
        assert_eq!(batch.target_path(), "C:\\App\\run.bat");

        let cmd = classify_execution_target("D:\\scripts\\deploy.cmd");
        assert!(cmd.is_batch());

        let shell = classify_execution_target("cmd.exe /c start \"\" app.bat");
        assert!(shell.is_shell_dispatch());
        assert!(!shell.is_batch());
    }

    #[test]
    fn test_quote_shell_arg_scenarios() {
        assert_eq!(quote_shell_arg("--verbose"), "--verbose");
        assert_eq!(quote_shell_arg("flag123"), "flag123");
        assert_eq!(quote_shell_arg(""), "\"\"");
        assert_eq!(
            quote_shell_arg("C:\\Program Files\\App"),
            "\"C:\\Program Files\\App\""
        );
        assert_eq!(
            quote_shell_arg("--title=\"Hello World\""),
            "\"--title=\\\"Hello World\\\"\""
        );
        assert_eq!(quote_shell_arg("A&B"), "\"A&B\"");
        assert_eq!(quote_shell_arg("pipe|target"), "\"pipe|target\"");
        assert_eq!(quote_shell_arg("redirect>file"), "\"redirect>file\"");
        assert_eq!(quote_shell_arg("var%NAME%"), "\"var%NAME%\"");
        assert_eq!(quote_shell_arg("flag!test"), "\"flag!test\"");
    }

    #[test]
    fn test_direct_binary_pass_through() {
        let target = "C:\\Program Files\\Antigravity\\antigravity.exe";
        let raw_args = ["--user-data-dir=C:\\Data", "--inspect=9222"];
        let base_binary = Some("C:\\Base\\antigravity.exe");

        let result = resolve_executable_launch(target, &raw_args, base_binary);

        assert!(!result.was_batch_intercepted);
        assert!(result.is_direct());
        assert_eq!(result.resolved_executable, target);
        assert_eq!(
            result.resolved_args,
            vec!["--user-data-dir=C:\\Data", "--inspect=9222"]
        );
        assert_eq!(
            result.strategy,
            LaunchResolutionStrategy::TargetBinaryDirect
        );
        assert_eq!(result.strategy.as_str(), "target_binary_direct");
        assert!(result.notes[0].contains("resolved without batch interception"));
    }

    #[test]
    fn test_batch_interception_with_base_binary_fallback() {
        let target = "C:\\Program Files\\Antigravity\\run.bat";
        let raw_args = ["--flag1", "value with space"];
        let base_binary = Some("C:\\Base\\antigravity.exe");

        let result = resolve_executable_launch(target, &raw_args, base_binary);

        assert!(result.was_batch_intercepted);
        assert!(result.is_intercepted());
        assert!(!result.is_direct());
        assert_eq!(result.resolved_executable, "C:\\Base\\antigravity.exe");
        assert_eq!(result.resolved_args, vec!["--flag1", "value with space"]);
        assert!(result.strategy.is_detected_base_binary());
        assert_eq!(
            result.strategy,
            LaunchResolutionStrategy::DetectedBaseBinary {
                base_path: "C:\\Base\\antigravity.exe".to_string(),
            }
        );
        assert_eq!(result.strategy.as_str(), "detected_base_binary");
        assert!(result.notes[0].contains("redirected to detected base binary"));
    }

    #[test]
    fn test_cmd_script_interception_with_case_insensitivity() {
        let target = "D:\\workspace\\START.CMD";
        let raw_args = ["--profile=default"];
        let base_binary = Some("D:\\workspace\\core.exe");

        let result = resolve_executable_launch(target, &raw_args, base_binary);

        assert!(result.was_batch_intercepted);
        assert_eq!(result.resolved_executable, "D:\\workspace\\core.exe");
        assert!(result.strategy.is_detected_base_binary());
    }

    #[test]
    fn test_batch_interception_with_shell_dispatch_fallback_no_base() {
        let target = "C:\\Program Files\\Tools\\launch.bat";
        let raw_args = ["--debug", "my folder"];
        let base_binary: Option<&str> = None;

        let result = resolve_executable_launch(target, &raw_args, base_binary);

        assert!(result.was_batch_intercepted);
        assert_eq!(result.resolved_executable, "cmd.exe");
        assert!(result.strategy.is_shell_dispatch());
        assert_eq!(result.strategy.as_str(), "windows_shell_dispatch");

        assert_eq!(result.resolved_args[0], "/c");
        assert_eq!(result.resolved_args[1], "start");
        assert_eq!(result.resolved_args[2], "\"\"");
        assert_eq!(
            result.resolved_args[3],
            "\"C:\\Program Files\\Tools\\launch.bat\""
        );
        assert_eq!(result.resolved_args[4], "--debug");
        assert_eq!(result.resolved_args[5], "\"my folder\"");

        let preview = result.command_line_preview();
        assert!(preview.starts_with("cmd.exe /c start \"\""));
        assert!(preview.contains("\"C:\\Program Files\\Tools\\launch.bat\""));
        assert!(preview.contains("\"my folder\""));
    }

    #[test]
    fn test_batch_interception_with_empty_or_batch_base_binary() {
        let target = "launch.cmd";
        let raw_args = [];
        let base_binary = Some("fallback.bat"); // Invalid base binary

        let result = resolve_executable_launch(target, &raw_args, base_binary);

        assert!(result.was_batch_intercepted);
        assert_eq!(result.resolved_executable, "cmd.exe");
        assert!(result.strategy.is_shell_dispatch());
        assert_eq!(result.resolved_args[3], "launch.cmd");
    }

    #[test]
    fn test_argument_preserving_and_quote_escaping_in_shell_dispatch() {
        let target = "run.bat";
        let raw_args = [
            "--flag",
            "spaced argument",
            "",
            "arg=\"quoted\"",
            "metachar&special",
        ];

        let result = resolve_executable_launch(target, &raw_args, None);

        assert_eq!(result.resolved_args.len(), 4 + raw_args.len());
        assert_eq!(result.resolved_args[0], "/c");
        assert_eq!(result.resolved_args[1], "start");
        assert_eq!(result.resolved_args[2], "\"\"");
        assert_eq!(result.resolved_args[3], "run.bat");
        assert_eq!(result.resolved_args[4], "--flag");
        assert_eq!(result.resolved_args[5], "\"spaced argument\"");
        assert_eq!(result.resolved_args[6], "\"\"");
        assert_eq!(result.resolved_args[7], "\"arg=\\\"quoted\\\"\"");
        assert_eq!(result.resolved_args[8], "\"metachar&special\"");
    }

    #[test]
    fn test_batch_result_display_and_helpers() {
        let target = "direct.exe";
        let result = resolve_executable_launch(target, &[], None);
        assert_eq!(result.command_line_preview(), "direct.exe");
        assert!(result.is_direct());
        assert_eq!(format!("{}", result.strategy), "TargetBinaryDirect");

        let kind = ExecutionTargetKind::WindowsBatchScript("test.bat".to_string());
        assert_eq!(format!("{}", kind), "WindowsBatchScript(test.bat)");
    }
}
