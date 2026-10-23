#[cfg(test)]
mod tests {
    use super::super::batch_file_spawn_guard::{
        is_batch_file_extension, resolve_windows_execution_command, validate_executable_path,
        ExecutablePathViolation,
    };

    #[test]
    fn test_is_batch_file_extension() {
        assert!(is_batch_file_extension("run.bat"));
        assert!(is_batch_file_extension("tools\\launcher.cmd"));
        assert!(is_batch_file_extension("SCRIPT.BAT"));
        assert!(is_batch_file_extension("start.CMD"));
        assert!(!is_batch_file_extension("code.exe"));
        assert!(!is_batch_file_extension("app.sh"));
        assert!(!is_batch_file_extension("cmd.exe"));
    }

    #[test]
    fn test_validate_executable_path_rejects_batch_files() {
        let res = validate_executable_path("C:\\app\\launch.bat");
        match res {
            Err(ExecutablePathViolation::DirectBatchExecutionForbidden { path, suggestion }) => {
                assert_eq!(path, "C:\\app\\launch.bat");
                assert!(suggestion.contains("cmd.exe /c start"));
            }
            _ => panic!("Expected DirectBatchExecutionForbidden"),
        }
    }

    #[test]
    fn test_validate_executable_path_accepts_binaries() {
        assert!(validate_executable_path("C:\\Program Files\\IDE\\code.exe").is_ok());
        assert!(validate_executable_path("/usr/bin/code").is_ok());
    }

    #[test]
    fn test_resolve_windows_execution_command_wraps_batch_file() {
        let (exe, args) = resolve_windows_execution_command("launch.bat", &["--profile", "default"]);
        assert_eq!(exe, "cmd.exe");
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "start");
        assert_eq!(args[2], "");
        assert_eq!(args[3], "launch.bat");
        assert_eq!(args[4], "--profile");
        assert_eq!(args[5], "default");
    }

    #[test]
    fn test_resolve_windows_execution_command_passes_binary_directly() {
        let (exe, args) = resolve_windows_execution_command("antigravity.exe", &["--headless"]);
        assert_eq!(exe, "antigravity.exe");
        assert_eq!(args, vec!["--headless"]);
    }
}
