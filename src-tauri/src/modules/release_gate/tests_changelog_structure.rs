#[cfg(test)]
mod tests {
    use super::super::changelog_structure_validator::{
        validate_changelog_release_entry, ChangelogStructureReport,
    };

    #[test]
    fn test_valid_changelog_entry() {
        let entry = r#"
## [v4.250.0] - 2026-10-23
### Features
- Enhanced release gate verification and path linting.
(Thanks to @aukgit)
"#;
        let report = validate_changelog_release_entry(entry, "4.250.0");
        assert!(report.is_valid);
        assert!(report.has_valid_heading);
        assert!(report.has_aukgit_attribution);
        assert!(report.unauthorized_handles.is_empty());
        assert!(report.forbidden_absolute_paths.is_empty());
    }

    #[test]
    fn test_missing_heading() {
        let entry = r#"
### Features
- Enhanced release gate verification.
(Thanks to @aukgit)
"#;
        let report = validate_changelog_release_entry(entry, "4.250.0");
        assert!(!report.is_valid);
        assert!(!report.has_valid_heading);
    }

    #[test]
    fn test_missing_aukgit_attribution() {
        let entry = r#"
## [v4.250.0] - 2026-10-23
### Features
- Enhanced release gate verification.
"#;
        let report = validate_changelog_release_entry(entry, "4.250.0");
        assert!(!report.is_valid);
        assert!(!report.has_aukgit_attribution);
    }

    #[test]
    fn test_unauthorized_handle_detected() {
        let entry = r#"
## [v4.250.0] - 2026-10-23
- Implemented by @somecontributor.
(Thanks to @aukgit)
"#;
        let report = validate_changelog_release_entry(entry, "4.250.0");
        assert!(!report.is_valid);
        assert_eq!(report.unauthorized_handles, vec!["@somecontributor"]);
    }

    #[test]
    fn test_forbidden_absolute_path_detected() {
        let entry = r#"
## [v4.250.0] - 2026-10-23
- Fixed file at D:\work\Antigravity-Manager\src\App.tsx.
(Thanks to @aukgit)
"#;
        let report = validate_changelog_release_entry(entry, "4.250.0");
        assert!(!report.is_valid);
        assert_eq!(report.forbidden_absolute_paths.len(), 1);
    }
}
