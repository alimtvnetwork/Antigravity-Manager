#[cfg(test)]
mod tests {
    use super::super::manifest_sync::{verify_manifest_versions, ManifestSyncReport};

    #[test]
    fn test_manifest_versions_all_matching() {
        let report = verify_manifest_versions(Some("4.250.0"), Some("4.250.0"), Some("4.250.0"));

        assert!(report.is_synced);
        assert_eq!(report.package_version.as_deref(), Some("4.250.0"));
        assert_eq!(report.cargo_version.as_deref(), Some("4.250.0"));
        assert_eq!(report.tauri_version.as_deref(), Some("4.250.0"));
    }

    #[test]
    fn test_manifest_versions_mismatched_cargo() {
        let report = verify_manifest_versions(Some("4.250.0"), Some("4.249.0"), Some("4.250.0"));

        assert!(!report.is_synced);
    }

    #[test]
    fn test_manifest_versions_mismatched_tauri() {
        let report = verify_manifest_versions(Some("4.250.0"), Some("4.250.0"), Some("4.249.0"));

        assert!(!report.is_synced);
    }

    #[test]
    fn test_manifest_versions_missing_file() {
        let report = verify_manifest_versions(Some("4.250.0"), None, Some("4.250.0"));

        assert!(!report.is_synced);
        assert_eq!(report.cargo_version, None);
    }

    #[test]
    fn test_manifest_versions_trimmed_whitespace() {
        let report =
            verify_manifest_versions(Some("  4.250.0  "), Some("4.250.0\n"), Some("4.250.0"));

        assert!(report.is_synced);
        assert_eq!(report.package_version.as_deref(), Some("4.250.0"));
    }
}
