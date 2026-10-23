//! Unit tests for Transient Test Sandbox Lifecycle Guard (`Imp-694`, Step 280).

#[cfg(test)]
mod tests {
    use crate::modules::release_gate::transient_test_sandbox_lifecycle_guard::{
        audit_and_update_sandbox, audit_sandbox_cleanliness, register_sandbox, track_child_process,
        SandboxCleanupReport, SandboxStatus, TestSandboxRecord, TRANSIENT_SANDBOX_PREFIX,
    };

    #[test]
    fn test_sandbox_prefix_constant() {
        assert_eq!(TRANSIENT_SANDBOX_PREFIX, "agm-test-sandbox-");
    }

    #[test]
    fn test_register_sandbox_initial_state() {
        let sandbox = register_sandbox(
            "agm-test-sandbox-9999",
            "data/sandboxes/agm-test-sandbox-9999",
            1792652100000,
        );

        assert_eq!(sandbox.sandbox_id, "agm-test-sandbox-9999");
        assert_eq!(sandbox.base_dir, "data/sandboxes/agm-test-sandbox-9999");
        assert_eq!(sandbox.created_epoch_ms, 1792652100000);
        assert!(sandbox.spawned_pids.is_empty());
        assert_eq!(sandbox.status, SandboxStatus::Created);
        assert!(!sandbox.status.is_active());
        assert!(!sandbox.status.is_cleaned());
        assert!(!sandbox.status.is_leaked());
        assert_eq!(sandbox.status.as_str(), "Created");
        assert_eq!(sandbox.status.to_string(), "Created");
    }

    #[test]
    fn test_track_child_process_transitions_to_active() {
        let mut sandbox = register_sandbox(
            "agm-test-sandbox-1",
            "data/sandboxes/agm-test-sandbox-1",
            1792652100000,
        );

        track_child_process(&mut sandbox, 50101);
        track_child_process(&mut sandbox, 50102);
        // Duplicate PID should not be added twice
        track_child_process(&mut sandbox, 50101);

        assert_eq!(sandbox.status, SandboxStatus::Active);
        assert!(sandbox.status.is_active());
        assert_eq!(sandbox.spawned_pids, vec![50101, 50102]);
    }

    #[test]
    fn test_clean_teardown_all_processes_and_files_purged() {
        let mut sandbox = register_sandbox(
            "agm-test-sandbox-clean",
            "data/sandboxes/agm-test-sandbox-clean",
            1792652100000,
        );
        track_child_process(&mut sandbox, 60101);
        track_child_process(&mut sandbox, 60102);

        // Neither 60101 nor 60102 are in active_os_pids; paths array contains no remnants of this sandbox
        let active_os_pids = [1000, 2000, 3000]; // unrelated system PIDs
        let existing_paths = ["data/sandboxes/other-sandbox/file.txt"];

        let report = audit_and_update_sandbox(&mut sandbox, &active_os_pids, &existing_paths);

        assert!(report.is_clean);
        assert_eq!(report.killed_processes_count, 2);
        assert!(report.surviving_pids.is_empty());
        assert!(report.surviving_paths.is_empty());
        assert!(report.violations.is_empty());
        assert_eq!(sandbox.status, SandboxStatus::Cleaned);
        assert!(sandbox.status.is_cleaned());
        assert!(report.summary().contains("cleanly reclaimed"));
        assert!(report.to_string().contains("cleanly reclaimed"));
    }

    #[test]
    fn test_surviving_pid_detection() {
        let mut sandbox = register_sandbox(
            "agm-test-sandbox-orphan-pid",
            "data/sandboxes/agm-test-sandbox-orphan-pid",
            1792652100000,
        );
        track_child_process(&mut sandbox, 70001);
        track_child_process(&mut sandbox, 70002);

        // 70002 is still active in the OS!
        let active_os_pids = [70002, 88888];
        let existing_paths: [&str; 0] = [];

        let report = audit_and_update_sandbox(&mut sandbox, &active_os_pids, &existing_paths);

        assert!(!report.is_clean);
        assert_eq!(report.killed_processes_count, 1);
        assert_eq!(report.surviving_pids, vec![70002]);
        assert!(report.surviving_paths.is_empty());
        assert_eq!(report.violations.len(), 1);
        assert!(report.violations[0].contains("Orphan child processes"));
        assert_eq!(sandbox.status, SandboxStatus::LeakedRemnants);
        assert!(sandbox.status.is_leaked());
        assert!(report.summary().contains("CLEANUP LEAK"));
    }

    #[test]
    fn test_surviving_path_detection() {
        let mut sandbox = register_sandbox(
            "agm-test-sandbox-leak-paths",
            "data/sandboxes/agm-test-sandbox-leak-paths",
            1792652100000,
        );
        track_child_process(&mut sandbox, 80001);

        let active_os_pids: [u32; 0] = [];
        let existing_paths = [
            "data/sandboxes/agm-test-sandbox-leak-paths/workspaceStorage/state.vscdb",
            "data/sandboxes/agm-test-sandbox-leak-paths/User/settings.json",
            "unrelated/production/path.json",
        ];

        let report = audit_and_update_sandbox(&mut sandbox, &active_os_pids, &existing_paths);

        assert!(!report.is_clean);
        assert!(report.surviving_pids.is_empty());
        assert_eq!(report.surviving_paths.len(), 2);
        assert!(report
            .surviving_paths
            .contains(&"data/sandboxes/agm-test-sandbox-leak-paths/workspaceStorage/state.vscdb".to_string()));
        assert!(report
            .surviving_paths
            .contains(&"data/sandboxes/agm-test-sandbox-leak-paths/User/settings.json".to_string()));
        assert_eq!(report.violations.len(), 1);
        assert!(report.violations[0].contains("Filesystem remnants"));
        assert_eq!(sandbox.status, SandboxStatus::LeakedRemnants);
    }

    #[test]
    fn test_both_orphan_pids_and_filesystem_remnants() {
        let mut sandbox = register_sandbox(
            "agm-test-sandbox-full-leak",
            "data/sandboxes/agm-test-sandbox-full-leak",
            1792652100000,
        );
        track_child_process(&mut sandbox, 90001);

        let active_os_pids = [90001];
        let existing_paths = ["data/sandboxes/agm-test-sandbox-full-leak/lock"];

        let report = audit_and_update_sandbox(&mut sandbox, &active_os_pids, &existing_paths);

        assert!(!report.is_clean);
        assert_eq!(report.surviving_pids, vec![90001]);
        assert_eq!(
            report.surviving_paths,
            vec!["data/sandboxes/agm-test-sandbox-full-leak/lock".to_string()]
        );
        assert_eq!(report.violations.len(), 2);
        assert_eq!(sandbox.status, SandboxStatus::LeakedRemnants);
    }

    #[test]
    fn test_cross_platform_path_matching_windows_slashes() {
        let sandbox = register_sandbox(
            "agm-test-sandbox-win",
            "C:\\sandboxes\\agm-test-sandbox-win",
            1792652100000,
        );

        let existing_paths = ["c:/sandboxes/agm-test-sandbox-win/cache.db"];
        let report = audit_sandbox_cleanliness(&sandbox, &[], &existing_paths);

        assert!(!report.is_clean);
        assert_eq!(report.surviving_paths.len(), 1);
    }
}
