use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_is_helper_process_detection() {
        // Normal main processes
        assert!(!is_helper_process(
            "Antigravity",
            "/Applications/Antigravity.app/Contents/MacOS/Antigravity",
            "/Applications/Antigravity.app/Contents/MacOS/Antigravity"
        ));
        assert!(!is_helper_process(
            "Antigravity.exe",
            "C:\\Program Files\\Antigravity\\Antigravity.exe",
            "C:\\Program Files\\Antigravity\\Antigravity.exe"
        ));

        // Language server / engine processes (must be detected as helper)
        assert!(is_helper_process(
            "language_server",
            "--standalone --override_ide_name antigravity --subclient_type hub",
            "/Applications/Antigravity.app/Contents/Resources/bin/language_server"
        ));
        assert!(is_helper_process(
            "language_server.exe",
            "--standalone",
            "C:\\Antigravity\\resources\\bin\\language_server.exe"
        ));
        assert!(is_helper_process(
            "Antigravity",
            "--type=utility --utility-sub-type=audio.mojom.AudioService",
            "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper.app/Contents/MacOS/Antigravity Helper"
        ));
        assert!(is_helper_process(
            "Antigravity",
            "--type=renderer",
            "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper (Renderer).app"
        ));
        assert!(is_helper_process(
            "crashpad_handler",
            "",
            "/Applications/Antigravity.app/Contents/Frameworks/Electron Framework.framework/Helpers/chrome_crashpad_handler"
        ));
    }

    #[test]
    pub(crate) fn test_is_language_server_process_narrow_detection() {
        // Hot-switch locator must be narrow: only match language server, never renderer / gpu / crashpad
        assert!(is_language_server_process(
            "language_server.exe",
            "C:\\Users\\me\\AppData\\Local\\Programs\\Antigravity IDE\\resources\\bin\\language_server.exe"
        ));
        assert!(is_language_server_process(
            "language_server",
            "/Applications/Antigravity.app/Contents/Resources/bin/language_server"
        ));
        // macOS process name might be truncated -> matching path is sufficient
        assert!(is_language_server_process(
            "language_server",
            "/Applications/Antigravity IDE.app/Contents/Resources/bin/language_server_macos_arm"
        ));
        // Case-insensitive
        assert!(is_language_server_process(
            "LANGUAGE_SERVER.EXE",
            "C:\\Antigravity\\bin\\Language_Server.exe"
        ));

        // Negative examples: other Antigravity processes must never match
        for (name, exe) in [
            ("Antigravity.exe", "C:\\Antigravity\\Antigravity.exe"),
            (
                "Antigravity",
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity",
            ),
            (
                "Antigravity Helper",
                "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper.app/Contents/MacOS/Antigravity Helper",
            ),
            (
                "Antigravity Helper (Renderer)",
                "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper (Renderer).app",
            ),
            (
                "crashpad_handler",
                "/Applications/Antigravity.app/Contents/Frameworks/Electron Framework.framework/Helpers/chrome_crashpad_handler",
            ),
            ("node", "/usr/local/bin/node"),
        ] {
            assert!(
                !is_language_server_process(name, exe),
                "{name} should not be identified as language_server"
            );
        }
    }

    #[test]
    pub(crate) fn test_sanitize_restart_args() {
        let dirty_args = vec![
            "--standalone".to_string(),
            "--override_ide_name".to_string(),
            "antigravity".to_string(),
            "--subclient_type".to_string(),
            "hub".to_string(),
            "--user-data-dir=/tmp/test".to_string(),
            "/path/to/project".to_string(),
        ];

        let cleaned = sanitize_restart_args(&dirty_args);
        assert!(!cleaned.contains(&"--standalone".to_string()));
        assert!(!cleaned.contains(&"--override_ide_name".to_string()));
        assert!(!cleaned.contains(&"--subclient_type".to_string()));
        assert!(cleaned.contains(&"--user-data-dir=/tmp/test".to_string()));
        assert!(cleaned.contains(&"/path/to/project".to_string()));
    }

    #[test]
    pub(crate) fn test_is_non_ide_binary_filters_build_tools() {
        assert!(is_non_ide_binary(
            "esbuild.exe",
            "D:/work/Antigravity-Manager/node_modules/esbuild/esbuild.exe",
            ""
        ));
        assert!(is_non_ide_binary(
            "cargo.exe",
            "C:/Users/User/.cargo/bin/cargo.exe",
            "test"
        ));
        assert!(is_non_ide_binary(
            "rustc.exe",
            "C:/Users/User/.cargo/bin/rustc.exe",
            ""
        ));
        assert!(is_non_ide_binary(
            "node.exe",
            "C:/Program Files/nodejs/node.exe",
            "vite"
        ));
        assert!(!is_non_ide_binary(
            "antigravity.exe",
            "C:/Program Files/Antigravity/antigravity.exe",
            ""
        ));
        assert!(!is_non_ide_binary(
            "antigravity",
            "/usr/bin/antigravity",
            ""
        ));
    }

    #[test]
    pub(crate) fn test_format_macos_open_args_with_no_args() {
        let args = format_macos_open_args("Antigravity", None, true);
        assert_eq!(
            args,
            vec!["-n", "-a", "Antigravity", "--args", "--new-window"]
        );
    }

    #[test]
    pub(crate) fn test_format_macos_open_args_with_custom_args() {
        let custom = vec![
            "--user-data-dir=/tmp/test".to_string(),
            "/path/to/workspace".to_string(),
        ];
        let args = format_macos_open_args("/Applications/Antigravity.app", Some(&custom), true);
        assert_eq!(
            args,
            vec![
                "-n",
                "-a",
                "/Applications/Antigravity.app",
                "--args",
                "--user-data-dir=/tmp/test",
                "/path/to/workspace",
                "--new-window"
            ]
        );
    }

    #[test]
    pub(crate) fn test_format_macos_open_args_snapshot_no_new_window() {
        let snapshot_args = vec!["--user-data-dir=/tmp/snapshot".to_string()];
        let args = format_macos_open_args(
            "/Applications/Antigravity IDE.app",
            Some(&snapshot_args),
            false,
        );
        assert_eq!(
            args,
            vec![
                "-n",
                "-a",
                "/Applications/Antigravity IDE.app",
                "--args",
                "--user-data-dir=/tmp/snapshot"
            ]
        );

        let no_args = format_macos_open_args("Antigravity", None, false);
        assert_eq!(no_args, vec!["-n", "-a", "Antigravity"]);
    }

    #[test]
    pub(crate) fn test_get_macos_candidate_paths() {
        let home = std::path::Path::new("/Users/developer");
        let paths = get_macos_candidate_paths("Antigravity", Some(home));
        assert_eq!(
            paths,
            vec![
                "/Applications/Antigravity.app".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Electron".to_string(),
                "/Users/developer/Applications/Antigravity.app".to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Electron".to_string(),
            ]
        );
    }

    #[test]
    pub(crate) fn test_get_macos_candidate_paths_ide() {
        let home = std::path::Path::new("/Users/developer");
        let paths = get_macos_candidate_paths("Antigravity IDE", Some(home));
        assert_eq!(
            paths,
            vec![
                "/Applications/Antigravity IDE.app".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity IDE".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Electron".to_string(),
                "/Users/developer/Applications/Antigravity IDE.app".to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity IDE"
                    .to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Electron"
                    .to_string(),
            ]
        );
    }

    #[test]
    pub(crate) fn test_discover_and_persist_initial_ide_info_runs_without_panic() {
        // Justification: non-Result return value intentionally discarded — no error channel to track
        let _ = discover_and_persist_initial_ide_info();
    }

    #[test]
    pub(crate) fn test_append_ide_discovery_log_runs_without_panic() {
        append_ide_discovery_log(
            false,
            Some("FAILED"),
            "/path/to/test",
            "test_method",
            Some(&["/checked/1".to_string(), "/checked/2".to_string()]),
            "test_trace",
        );
    }
}
