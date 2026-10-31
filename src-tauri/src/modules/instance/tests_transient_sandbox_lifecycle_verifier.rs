//! Unit tests for transient sandbox lifecycle and zero-remnants verifier.

#[cfg(test)]
mod tests {
    use super::super::transient_sandbox_lifecycle_verifier::{
        initialize_transient_sandbox, verify_sandbox_cleanup, SandboxLifecyclePhase,
        SandboxRemnantsViolation,
    };

    #[test]
    fn test_initialize_transient_sandbox_valid() {
        let state = initialize_transient_sandbox("test-sandbox-01", Some(1234))
            .expect("Valid sandbox ID should succeed");

        assert_eq!(state.sandbox_id, "test-sandbox-01");
        assert_eq!(state.phase, SandboxLifecyclePhase::Created);
        assert_eq!(state.transient_dir, "target/sandboxes/test-sandbox-01");
        assert_eq!(state.caller_pid, Some(1234));
        assert!(state.tracked_child_pids.is_empty());
    }

    #[test]
    fn test_initialize_transient_sandbox_invalid_format() {
        assert!(initialize_transient_sandbox("", None).is_err());
        assert!(initialize_transient_sandbox("bad/path", None).is_err());
        assert!(initialize_transient_sandbox("has spaces", None).is_err());
        assert!(initialize_transient_sandbox("invalid$char", None).is_err());
    }

    #[test]
    fn test_cleanup_successful_zero_remnants() {
        let mut state = initialize_transient_sandbox("clean-sandbox-1", None)
            .expect("Initialization should succeed");

        state.track_child_pid(5001);
        state.track_child_pid(5002);
        state.mark_active();
        assert!(state.phase.is_active());

        // Child PIDs 5001, 5002 are no longer running; unrelated PID 9999 is running.
        let running_pids = vec![9999];
        // Existing paths do not overlap with target/sandboxes/clean-sandbox-1.
        let existing_paths = vec!["target/sandboxes/other-sandbox/file.txt".to_string()];

        let report = verify_sandbox_cleanup(&mut state, &running_pids, &existing_paths, false);

        assert!(report.is_clean);
        assert!(report.zero_remnants_confirmed);
        assert_eq!(report.sandbox_id, "clean-sandbox-1");
        assert!(report.violations.is_empty());
        assert_eq!(state.phase, SandboxLifecyclePhase::CleanedUp);
        assert!(state.phase.is_cleaned_up());
    }

    #[test]
    fn test_cleanup_detects_orphan_child_processes() {
        let mut state = initialize_transient_sandbox("orphan-sandbox-1", None)
            .expect("Initialization should succeed");

        state.track_child_pid(6001);
        state.track_child_pid(6002);

        // Child PID 6002 is still running
        let running_pids = vec![6002, 7000];
        let existing_paths = Vec::new();

        let report = verify_sandbox_cleanup(&mut state, &running_pids, &existing_paths, false);

        assert!(!report.is_clean);
        assert!(!report.zero_remnants_confirmed);
        assert_eq!(state.phase, SandboxLifecyclePhase::FailedCleanup);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            SandboxRemnantsViolation::OrphanProcessesRemaining {
                sandbox_id,
                orphan_pids,
            } => {
                assert_eq!(sandbox_id, "orphan-sandbox-1");
                assert_eq!(orphan_pids, &vec![6002]);
            }
            _ => panic!("Expected OrphanProcessesRemaining violation"),
        }
    }

    #[test]
    fn test_cleanup_detects_filesystem_remnants() {
        let mut state = initialize_transient_sandbox("fs-remnants-sandbox", None)
            .expect("Initialization should succeed");

        let running_pids = Vec::new();
        let existing_paths = vec![
            "target/sandboxes/fs-remnants-sandbox/config.json".to_string(),
            "target/sandboxes/fs-remnants-sandbox/sub/log.txt".to_string(),
            "target/sandboxes/unrelated/config.json".to_string(),
        ];

        let report = verify_sandbox_cleanup(&mut state, &running_pids, &existing_paths, false);

        assert!(!report.is_clean);
        assert!(!report.zero_remnants_confirmed);
        assert_eq!(state.phase, SandboxLifecyclePhase::FailedCleanup);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            SandboxRemnantsViolation::FilesystemRemnantsRemaining {
                sandbox_id,
                remaining_paths,
            } => {
                assert_eq!(sandbox_id, "fs-remnants-sandbox");
                assert_eq!(remaining_paths.len(), 2);
                assert!(remaining_paths[0].contains("config.json"));
                assert!(remaining_paths[1].contains("sub/log.txt"));
            }
            _ => panic!("Expected FilesystemRemnantsRemaining violation"),
        }
    }

    #[test]
    fn test_caller_ide_protection_without_force() {
        let caller_pid = 8888;
        let mut state = initialize_transient_sandbox("caller-guard-sandbox", Some(caller_pid))
            .expect("Initialization should succeed");

        // Track a child PID that is still running
        state.track_child_pid(3001);

        // Both child PID and caller IDE process are running
        let running_pids = vec![3001, caller_pid];
        let existing_paths = Vec::new();

        let report = verify_sandbox_cleanup(&mut state, &running_pids, &existing_paths, false);

        assert!(!report.is_clean);
        assert_eq!(state.phase, SandboxLifecyclePhase::FailedCleanup);

        match &report.violations[0] {
            SandboxRemnantsViolation::CallerIdeProcessTargeted {
                caller_pid: pid,
                guidance,
            } => {
                assert_eq!(*pid, 8888);
                assert!(guidance.contains("--force"));
            }
            _ => panic!("Expected CallerIdeProcessTargeted violation"),
        }
    }

    #[test]
    fn test_bypassing_caller_protection_with_force() {
        let caller_pid = 8888;
        let mut state = initialize_transient_sandbox("caller-guard-sandbox", Some(caller_pid))
            .expect("Initialization should succeed");

        state.track_child_pid(3001);

        // Both child PID and caller IDE process are running, but force is true
        let running_pids = vec![3001, caller_pid];
        let existing_paths = Vec::new();

        let report = verify_sandbox_cleanup(&mut state, &running_pids, &existing_paths, true);

        assert!(!report.is_clean);
        // With force = true, caller self-preservation is bypassed, yielding orphan process report
        match &report.violations[0] {
            SandboxRemnantsViolation::OrphanProcessesRemaining { orphan_pids, .. } => {
                assert_eq!(orphan_pids, &vec![3001]);
            }
            _ => panic!("Expected OrphanProcessesRemaining violation when force is true"),
        }
    }

    #[test]
    fn test_violation_display_formatting() {
        let v1 = SandboxRemnantsViolation::OrphanProcessesRemaining {
            sandbox_id: "s1".to_string(),
            orphan_pids: vec![101, 102],
        };
        assert!(v1.to_string().contains("orphan child processes"));
        assert!(v1.to_string().contains("101"));

        let v2 = SandboxRemnantsViolation::FilesystemRemnantsRemaining {
            sandbox_id: "s2".to_string(),
            remaining_paths: vec!["target/sandboxes/s2/temp.db".to_string()],
        };
        assert!(v2.to_string().contains("filesystem remnants"));
        assert!(v2.to_string().contains("temp.db"));

        let v3 = SandboxRemnantsViolation::CallerIdeProcessTargeted {
            caller_pid: 9999,
            guidance: "pass --force".to_string(),
        };
        assert!(v3.to_string().contains("Caller IDE process 9999"));
        assert!(v3.to_string().contains("pass --force"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let violation = SandboxRemnantsViolation::CallerIdeProcessTargeted {
            caller_pid: 4321,
            guidance: "check force flag".to_string(),
        };
        let err: &dyn std::error::Error = &violation;
        assert!(err.to_string().contains("4321"));
    }
}
