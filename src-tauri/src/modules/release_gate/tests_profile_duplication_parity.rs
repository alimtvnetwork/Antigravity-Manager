#[cfg(test)]
mod tests {
    use super::super::profile_duplication_parity_guard::{
        validate_duplication_parity, DuplicationParityReport, InstanceCopyOptions,
    };

    #[test]
    fn test_valid_duplication_parity_compliant() {
        let opts = InstanceCopyOptions {
            source_id: "inst-primary".to_string(),
            target_id: "inst-clone-1".to_string(),
            preserve_workspace_storage: true,
            sync_settings: true,
            is_shared_service_route: true,
        };

        let report = validate_duplication_parity(&opts);
        assert!(report.is_parity_compliant);
        assert!(report.preserves_workspace_storage);
        assert!(report.is_shared_service_route);
        assert!(report.violation_reasons.is_empty());
    }

    #[test]
    fn test_invalid_duplication_missing_workspace_storage_preservation() {
        let opts = InstanceCopyOptions {
            source_id: "inst-primary".to_string(),
            target_id: "inst-clone-1".to_string(),
            preserve_workspace_storage: false,
            sync_settings: true,
            is_shared_service_route: true,
        };

        let report = validate_duplication_parity(&opts);
        assert!(!report.is_parity_compliant);
        assert!(!report.preserves_workspace_storage);
        assert!(report.violation_reasons[0].contains("preserve_workspace_storage"));
    }

    #[test]
    fn test_invalid_duplication_not_shared_service_route() {
        let opts = InstanceCopyOptions {
            source_id: "inst-primary".to_string(),
            target_id: "inst-clone-1".to_string(),
            preserve_workspace_storage: true,
            sync_settings: true,
            is_shared_service_route: false,
        };

        let report = validate_duplication_parity(&opts);
        assert!(!report.is_parity_compliant);
        assert!(report.violation_reasons[0].contains("copy_instance_with_options"));
    }

    #[test]
    fn test_invalid_duplication_identical_source_and_target() {
        let opts = InstanceCopyOptions {
            source_id: "inst-primary".to_string(),
            target_id: "inst-primary".to_string(),
            preserve_workspace_storage: true,
            sync_settings: true,
            is_shared_service_route: true,
        };

        let report = validate_duplication_parity(&opts);
        assert!(!report.is_parity_compliant);
        assert!(report.violation_reasons[0].contains("cannot be identical"));
    }
}
