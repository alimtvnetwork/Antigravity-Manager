#[cfg(test)]
mod tests {
    use super::super::path_hygiene::lint_path_hygiene;

    #[test]
    fn test_relative_paths_are_clean() {
        let content = "Updated spec in `02-spec/21-app/spec.md` and plan in `.ai-memory/plans/completed/plan.md`.";
        let report = lint_path_hygiene(content);

        assert!(report.is_clean);
        assert!(report.absolute_path_violations.is_empty());
    }

    #[test]
    fn test_windows_drive_letter_is_flagged() {
        let content = "File located at D:\\work\\Antigravity-Manager\\src\\main.rs";
        let report = lint_path_hygiene(content);

        assert!(!report.is_clean);
        assert_eq!(report.absolute_path_violations.len(), 1);
        assert!(report.absolute_path_violations[0].contains("D:\\work"));
    }

    #[test]
    fn test_unix_home_path_is_flagged() {
        let content = "Log written to `/home/user/project/log.txt`";
        let report = lint_path_hygiene(content);

        assert!(!report.is_clean);
        assert_eq!(report.absolute_path_violations.len(), 1);
        assert!(report.absolute_path_violations[0].contains("/home/user/project/log.txt"));
    }

    #[test]
    fn test_multiple_absolute_paths_collected() {
        let content = "Path 1: C:/repo/src\nPath 2: /Users/admin/docs";
        let report = lint_path_hygiene(content);

        assert!(!report.is_clean);
        assert_eq!(report.absolute_path_violations.len(), 2);
    }
}
