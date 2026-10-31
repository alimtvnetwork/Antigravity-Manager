//! Tests for Headless & CLI Parity Guard.

#[cfg(test)]
mod tests {
    use super::super::headless_cli_parity_guard::{
        canonical_lifecycle_registry, evaluate_parity, OperationEntry, ParityReport,
        ParityViolation,
    };

    #[test]
    fn test_gui_only_lifecycle_operation_flagged() {
        let entries = vec![OperationEntry::gui_only(
            "instance_duplication",
            Some("copy_instance_with_options".to_string()),
        )];

        let report = evaluate_parity(&entries);
        assert!(!report.is_clean);
        assert_eq!(report.paired_operations_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            ParityViolation::GuiOnlyOperation { operation } if operation == "instance_duplication"
        ));
    }

    #[test]
    fn test_cli_only_lifecycle_operation_flagged() {
        let entries = vec![OperationEntry::cli_only(
            "quick_profile_dump",
            Some("dump_profile_route".to_string()),
        )];

        let report = evaluate_parity(&entries);
        assert!(!report.is_clean);
        assert_eq!(report.paired_operations_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            ParityViolation::CliOnlyOperation { operation } if operation == "quick_profile_dump"
        ));
    }

    #[test]
    fn test_missing_shared_service_route_flagged() {
        let mut entry = OperationEntry::new("sync_settings");
        entry.has_gui_entry = true;
        entry.has_cli_entry = true;
        entry.service_route = None;

        let report = evaluate_parity(&[entry]);
        assert!(!report.is_clean);
        assert_eq!(report.paired_operations_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            ParityViolation::MissingSharedServiceRoute { operation } if operation == "sync_settings"
        ));
    }

    #[test]
    fn test_empty_service_route_flagged() {
        let mut entry = OperationEntry::new("sync_settings");
        entry.has_gui_entry = true;
        entry.has_cli_entry = true;
        entry.service_route = Some("   ".to_string());

        let report = evaluate_parity(&[entry]);
        assert!(!report.is_clean);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            ParityViolation::MissingSharedServiceRoute { operation } if operation == "sync_settings"
        ));
    }

    #[test]
    fn test_divergent_service_route_flagged() {
        let mut entry = OperationEntry::new("profile_sync");
        entry.has_gui_entry = true;
        entry.has_cli_entry = true;
        entry.service_route = Some("gui_only_sync_adapter".to_string());

        let report = evaluate_parity(&[entry]);
        assert!(!report.is_clean);
        assert_eq!(report.paired_operations_count, 0);
        assert_eq!(report.violations.len(), 1);
        assert!(matches!(
            &report.violations[0],
            ParityViolation::DivergentServiceRoute { operation, .. } if operation == "profile_sync"
        ));
    }

    #[test]
    fn test_fully_paired_registry_passes_clean() {
        let entries = vec![
            OperationEntry::paired("instance_duplication", "copy_instance_with_options"),
            OperationEntry::paired("settings_synchronization", "sync_settings_deep_merge"),
            OperationEntry::paired("instance_stop", "stop_instance"),
        ];

        let report = evaluate_parity(&entries);
        assert!(
            report.is_clean,
            "Expected fully paired operations to be clean, got: {:?}",
            report.violations
        );
        assert_eq!(report.paired_operations_count, 3);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_canonical_lifecycle_registry_passes() {
        let registry = canonical_lifecycle_registry();
        let report = evaluate_parity(&registry);

        assert!(
            report.is_clean,
            "Expected canonical lifecycle registry to be clean, got: {:?}",
            report.violations
        );
        assert_eq!(report.paired_operations_count, registry.len());
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_serde_roundtrip_parity_report() {
        let entries = vec![
            OperationEntry::paired("op_ok", "shared_route"),
            OperationEntry::gui_only("op_gui", Some("route".to_string())),
        ];
        let report = evaluate_parity(&entries);

        assert!(!report.is_clean);
        assert_eq!(report.paired_operations_count, 1);
        assert_eq!(report.violations.len(), 1);

        let json = serde_json::to_string(&report).expect("serialize parity report");
        let deserialized: ParityReport =
            serde_json::from_str(&json).expect("deserialize parity report");
        assert_eq!(report, deserialized);
    }
}
