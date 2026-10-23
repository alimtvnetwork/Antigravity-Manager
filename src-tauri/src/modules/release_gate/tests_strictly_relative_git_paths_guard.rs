//! Unit tests for strictly relative git paths guard.

use super::strictly_relative_git_paths_guard::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_relative_paths() {
        let content = "Updated spec in `02-spec/21-app/spec.md` and codebase in `src-tauri/src/main.rs` and `cmd/cli/main.go`.";
        let report = audit_text_for_absolute_paths(content);

        assert!(report.is_clean);
        assert_eq!(report.total_lines_scanned, 1);
        assert!(report.violations.is_empty());
        assert_eq!(report.verified_relative_paths_count, 3);
    }

    #[test]
    fn test_windows_drive_path_detected() {
        let drive_c = format!("{}:\\Users\\admin\\repo\\file.rs", "C");
        let drive_d = format!("{}:/work/Antigravity-Manager/spec.md", "D");
        let content = format!("First line: {drive_c}\nSecond line: {drive_d}");

        let report = audit_text_for_absolute_paths(&content);

        assert!(!report.is_clean);
        assert_eq!(report.total_lines_scanned, 2);
        assert_eq!(report.violations.len(), 2);

        match &report.violations[0] {
            PathViolationKind::WindowsDriveAbsolute { raw_path, line } => {
                assert_eq!(*line, 1);
                assert_eq!(raw_path, &drive_c);
            }
            _ => panic!("Expected WindowsDriveAbsolute violation on line 1"),
        }

        match &report.violations[1] {
            PathViolationKind::WindowsDriveAbsolute { raw_path, line } => {
                assert_eq!(*line, 2);
                assert_eq!(raw_path, &drive_d);
            }
            _ => panic!("Expected WindowsDriveAbsolute violation on line 2"),
        }
    }

    #[test]
    fn test_posix_home_path_detected() {
        let home_linux = format!("{}/runner/work/Antigravity-Manager", "/home");
        let home_mac = format!("{}/developer/projects/app", "/Users");
        let content = format!("Linux: {home_linux}\nMac: {home_mac}");

        let report = audit_text_for_absolute_paths(&content);

        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 2);

        match &report.violations[0] {
            PathViolationKind::PosixHomeAbsolute { raw_path, line } => {
                assert_eq!(*line, 1);
                assert_eq!(raw_path, &home_linux);
            }
            _ => panic!("Expected PosixHomeAbsolute on line 1"),
        }

        match &report.violations[1] {
            PathViolationKind::PosixHomeAbsolute { raw_path, line } => {
                assert_eq!(*line, 2);
                assert_eq!(raw_path, &home_mac);
            }
            _ => panic!("Expected PosixHomeAbsolute on line 2"),
        }
    }

    #[test]
    fn test_posix_system_path_detected() {
        let etc_path = format!("{}/hosts", "/etc");
        let var_path = format!("{}/log/syslog", "/var");
        let usr_path = format!("{}/local/bin/agm", "/usr");
        let content = format!("Checking {etc_path}, {var_path}, and {usr_path}");

        let report = audit_text_for_absolute_paths(&content);

        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 3);

        let system_paths: Vec<&str> = report
            .violations
            .iter()
            .filter_map(|v| match v {
                PathViolationKind::PosixSystemAbsolute { raw_path, .. } => Some(raw_path.as_str()),
                _ => None,
            })
            .collect();

        assert!(system_paths.contains(&etc_path.as_str()));
        assert!(system_paths.contains(&var_path.as_str()));
        assert!(system_paths.contains(&usr_path.as_str()));
    }

    #[test]
    fn test_file_uri_detected() {
        let file_uri = format!("{}C:/path/to/source.rs", "file:///");
        let content = format!("Reference: [link]({file_uri})");

        let report = audit_text_for_absolute_paths(&content);

        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            PathViolationKind::FileUriAbsolute { raw_uri, line } => {
                assert_eq!(*line, 1);
                assert_eq!(raw_uri, &file_uri);
            }
            _ => panic!("Expected FileUriAbsolute violation"),
        }
    }

    #[test]
    fn test_commit_message_validation() {
        let clean_msg = "feat: update 02-spec/21-app/spec.md and src/main.rs";
        assert!(validate_commit_message_paths(clean_msg).is_ok());

        let bad_msg_linux = format!("fix: see {}/runner/work/err.log", "/home");
        let result_linux = validate_commit_message_paths(&bad_msg_linux);
        assert!(result_linux.is_err());
        match result_linux.unwrap_err() {
            PathViolationKind::PosixHomeAbsolute { line, .. } => assert_eq!(line, 1),
            _ => panic!("Expected PosixHomeAbsolute"),
        }

        let bad_msg_win = format!("docs: refer to {}:/docs/guide.md", "D");
        let result_win = validate_commit_message_paths(&bad_msg_win);
        assert!(result_win.is_err());
        match result_win.unwrap_err() {
            PathViolationKind::WindowsDriveAbsolute { line, .. } => assert_eq!(line, 1),
            _ => panic!("Expected WindowsDriveAbsolute"),
        }
    }

    #[test]
    fn test_violation_display_formatting() {
        let v1 = PathViolationKind::WindowsDriveAbsolute {
            raw_path: format!("{}:\\test\\file.rs", "C"),
            line: 4,
        };
        assert!(format!("{v1}").contains("line 4"));
        assert!(format!("{v1}").contains("Windows drive absolute path"));

        let v2 = PathViolationKind::PosixHomeAbsolute {
            raw_path: format!("{}/user/project", "/home"),
            line: 7,
        };
        assert!(format!("{v2}").contains("line 7"));
        assert!(format!("{v2}").contains("POSIX home absolute path"));

        let v3 = PathViolationKind::PosixSystemAbsolute {
            raw_path: format!("{}/hosts", "/etc"),
            line: 12,
        };
        assert!(format!("{v3}").contains("line 12"));
        assert!(format!("{v3}").contains("POSIX system absolute path"));

        let v4 = PathViolationKind::FileUriAbsolute {
            raw_uri: format!("{}opt/data", "file:///"),
            line: 19,
        };
        assert!(format!("{v4}").contains("line 19"));
        assert!(format!("{v4}").contains("Absolute file URI"));
    }
}
