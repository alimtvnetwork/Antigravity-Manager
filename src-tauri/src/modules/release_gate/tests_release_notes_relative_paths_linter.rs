//! Unit tests for release notes & changelog relative paths linter (Imp-657).

use super::release_notes_relative_paths_linter::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_markdown_with_valid_relative_paths() {
        let content = r#"# Release Notes v4.251.0

- Updated spec in `02-spec/21-app/spec.md`
- Implemented core in `src-tauri/src/main.rs` and UI in `src/App.tsx`
- Added CLI command in `cmd/cli/main.go`
- Updated changelog in `CHANGELOG.md` and memory in `.ai-memory/plans/plan.md`
"#;
        let report = lint_release_notes(content);
        assert!(report.is_clean());
        assert_eq!(report.violation_count(), 0);
        assert!(report.violations.is_empty());
        assert!(report.verified_relative_paths >= 5);
        assert_eq!(report.total_lines, 7);
    }

    #[test]
    fn test_windows_drive_paths_detected() {
        let drive = "D";
        let path1 = format!("{drive}:\\work\\Antigravity-Manager\\src\\main.rs");
        let path2 = format!("{drive}:/work/Antigravity-Manager/02-spec/21-app/spec.md");
        let content = format!("First violation: {path1}\nSecond violation: {path2}");

        let report = lint_release_notes(&content);
        assert!(!report.is_clean());
        assert_eq!(report.violation_count(), 2);

        assert_eq!(report.violations[0].line_number, 1);
        assert_eq!(
            report.violations[0].category,
            AbsolutePathCategory::WindowsDrive
        );
        assert_eq!(report.violations[0].raw_path, path1);
        assert_eq!(
            report.violations[0].recommended_relative_path,
            "src/main.rs"
        );
        assert!(report.violations[0].snippet.contains("First violation"));

        assert_eq!(report.violations[1].line_number, 2);
        assert_eq!(
            report.violations[1].category,
            AbsolutePathCategory::WindowsDrive
        );
        assert_eq!(report.violations[1].raw_path, path2);
        assert_eq!(
            report.violations[1].recommended_relative_path,
            "02-spec/21-app/spec.md"
        );
    }

    #[test]
    fn test_unc_network_paths_detected() {
        let unc_path = format!("\\\\server\\share\\repo\\docs\\release_guide.md");
        let content = format!("Refer to UNC share at {unc_path}");

        let report = lint_release_notes(&content);
        assert!(!report.is_clean());
        assert_eq!(report.violation_count(), 1);

        let v = &report.violations[0];
        assert_eq!(v.category, AbsolutePathCategory::UncNetworkPath);
        assert_eq!(v.raw_path, unc_path);
        assert_eq!(v.recommended_relative_path, "docs/release_guide.md");
        assert_eq!(v.line_number, 1);
    }

    #[test]
    fn test_posix_home_paths_detected() {
        let linux_home = format!(
            "{}/runner/work/Antigravity-Manager/src-tauri/src/lib.rs",
            "/home"
        );
        let mac_home = format!("{}/developer/project/02-spec/21-app/spec.md", "/Users");
        let content = format!("Line 1: {linux_home}\nLine 2: {mac_home}");

        let report = lint_release_notes(&content);
        assert!(!report.is_clean());
        assert_eq!(report.violation_count(), 2);

        assert_eq!(
            report.violations[0].category,
            AbsolutePathCategory::PosixHome
        );
        assert_eq!(
            report.violations[0].recommended_relative_path,
            "src-tauri/src/lib.rs"
        );

        assert_eq!(
            report.violations[1].category,
            AbsolutePathCategory::PosixHome
        );
        assert_eq!(
            report.violations[1].recommended_relative_path,
            "02-spec/21-app/spec.md"
        );
    }

    #[test]
    fn test_posix_system_paths_detected() {
        let var_log = format!("{}/build_log.txt", "/var");
        let etc_conf = format!("{}/resolv.conf", "/etc");
        let content = format!("Logs at {var_log} and config at {etc_conf}");

        let report = lint_release_notes(&content);
        assert!(!report.is_clean());
        assert_eq!(report.violation_count(), 2);

        assert_eq!(
            report.violations[0].category,
            AbsolutePathCategory::PosixSystem
        );
        assert_eq!(
            report.violations[0].recommended_relative_path,
            "./build_log.txt"
        );

        assert_eq!(
            report.violations[1].category,
            AbsolutePathCategory::PosixSystem
        );
        assert_eq!(
            report.violations[1].recommended_relative_path,
            "./resolv.conf"
        );
    }

    #[test]
    fn test_file_uri_detected() {
        let file_uri = format!("{}D:/work/repo/.ai-memory/plans/plan.md", "file:///");
        let content = format!("Detailed memory at [plan]({file_uri})");

        let report = lint_release_notes(&content);
        assert!(!report.is_clean());
        assert_eq!(report.violation_count(), 1);

        let v = &report.violations[0];
        assert_eq!(v.category, AbsolutePathCategory::FileUri);
        assert_eq!(v.raw_path, file_uri);
        assert_eq!(v.recommended_relative_path, ".ai-memory/plans/plan.md");
    }

    #[test]
    fn test_suggest_relative_path_canonical() {
        // Windows drive with src-tauri
        let win_tauri = format!("{}:\\repo\\src-tauri\\src\\lib.rs", "C");
        assert_eq!(suggest_relative_path(&win_tauri), "src-tauri/src/lib.rs");

        // Windows drive with src
        let win_src = format!("{}:/repo/src/components/Button.tsx", "D");
        assert_eq!(suggest_relative_path(&win_src), "src/components/Button.tsx");

        // File URI with 02-spec
        let file_spec = format!("{}C:/project/02-spec/21-app/spec.md", "file:///");
        assert_eq!(suggest_relative_path(&file_spec), "02-spec/21-app/spec.md");

        // Top level file CHANGELOG.md
        let changelog = format!("{}:/work/Antigravity-Manager/CHANGELOG.md", "D");
        assert_eq!(suggest_relative_path(&changelog), "CHANGELOG.md");

        // Top level file AGENTS.md
        let agents = format!("{}/runner/work/repo/AGENTS.md", "/home");
        assert_eq!(suggest_relative_path(&agents), "AGENTS.md");

        // Arbitrary unknown path fallback
        let unknown = format!("{}:\\arbitrary_folder\\test.txt", "C");
        assert_eq!(suggest_relative_path(&unknown), "./test.txt");
    }

    #[test]
    fn test_check_release_notes_strictly_relative() {
        let clean = "All paths are in `02-spec/` and `src-tauri/`";
        assert!(check_release_notes_strictly_relative(clean).is_ok());

        let dirty = format!("Log in {}:\\work\\log.txt", "C");
        let err = check_release_notes_strictly_relative(&dirty);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().len(), 1);
    }

    #[test]
    fn test_format_summary() {
        let clean_content = "Normal markdown line with src/main.rs";
        let clean_report = lint_release_notes(clean_content);
        let summary = clean_report.format_summary();
        assert!(summary.contains("Path audit passed"));
        assert!(summary.contains("0 violations"));

        let dirty_content = format!("Path: {}:/foo/bar.txt", "C");
        let dirty_report = lint_release_notes(&dirty_content);
        let summary_dirty = dirty_report.format_summary();
        assert!(summary_dirty.contains("Path audit failed"));
        assert!(summary_dirty.contains("1 violations detected"));
    }

    #[test]
    fn test_lint_release_notes_file_io_error() {
        let non_existent_file = "non_existent_release_notes_9999.md";
        let result = lint_release_notes_file(non_existent_file);
        assert!(result.is_err());
    }
}
