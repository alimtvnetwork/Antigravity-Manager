//! Unit tests for Transient Sandbox Lifecycle & Cleanup Sentinel (`Imp-713`, Step 299).

#[cfg(test)]
mod tests {
    use super::transient_sandbox_cleanup_sentinel::{
        evaluate_sandbox_eviction_safety, generate_transient_sandbox_id, is_transient_sandbox_id,
        plan_transient_sandbox_cleanup, plan_transient_sandbox_cleanup_with_options,
        SandboxCleanupReport, SandboxEvictionVerdict, SandboxInstanceDescriptor,
        TransientSandboxState, CALLER_IDE_FAIL_FAST_EXIT_CODE, TRANSIENT_SANDBOX_PREFIX,
    };

    #[test]
    fn test_constants_and_prefix() {
        assert_eq!(CALLER_IDE_FAIL_FAST_EXIT_CODE, 2);
        assert_eq!(TRANSIENT_SANDBOX_PREFIX, "agm-test-sandbox-");
        assert!(is_transient_sandbox_id("agm-test-sandbox-abc"));
        assert!(is_transient_sandbox_id("test-sandbox-123"));
        assert!(is_transient_sandbox_id("transient-xyz"));
        assert!(!is_transient_sandbox_id("prod-instance-01"));
        assert_eq!(
            generate_transient_sandbox_id("beta-42"),
            "agm-test-sandbox-beta-42"
        );
    }

    #[test]
    fn test_transient_sandbox_state_methods() {
        let created = TransientSandboxState::Created;
        assert_eq!(created.as_str(), "Created");
        assert!(!created.is_active());
        assert!(!created.is_purged());
        assert!(created.requires_cleanup());
        assert_eq!(created.to_string(), "Created");

        let active = TransientSandboxState::Active;
        assert_eq!(active.as_str(), "Active");
        assert!(active.is_active());
        assert!(!active.is_purged());
        assert!(active.requires_cleanup());

        let pending = TransientSandboxState::PendingCleanup;
        assert_eq!(pending.as_str(), "PendingCleanup");
        assert!(!pending.is_active());
        assert!(!pending.is_purged());
        assert!(pending.requires_cleanup());

        let purged = TransientSandboxState::Purged;
        assert_eq!(purged.as_str(), "Purged");
        assert!(!purged.is_active());
        assert!(purged.is_purged());
        assert!(!purged.requires_cleanup());

        let leaked = TransientSandboxState::OrphanRemnantsDetected;
        assert_eq!(leaked.as_str(), "OrphanRemnantsDetected");
        assert!(!leaked.is_active());
        assert!(!leaked.is_purged());
        assert!(leaked.requires_cleanup());
    }

    #[test]
    fn test_sandbox_instance_descriptor_constructors() {
        let standard = SandboxInstanceDescriptor::new(
            "agm-test-sandbox-01",
            "sandboxes/agm-test-sandbox-01",
            1792650000,
            false,
        );
        assert_eq!(standard.sandbox_id, "agm-test-sandbox-01");
        assert_eq!(standard.instance_dir, "sandboxes/agm-test-sandbox-01");
        assert_eq!(standard.created_epoch_ms, 1792650000);
        assert!(!standard.is_caller_ide);
        assert!(standard.is_transient_sandbox_id());

        let caller = SandboxInstanceDescriptor::caller_ide("caller-main", "data/profiles/caller");
        assert_eq!(caller.sandbox_id, "caller-main");
        assert!(caller.is_caller_ide);
        assert!(!caller.is_transient_sandbox_id());

        let transient = SandboxInstanceDescriptor::transient(
            "agm-test-sandbox-99",
            "sandboxes/agm-test-sandbox-99",
        );
        assert_eq!(transient.sandbox_id, "agm-test-sandbox-99");
        assert!(!transient.is_caller_ide);
        assert!(transient.is_transient_sandbox_id());
    }

    #[test]
    fn test_sandbox_eviction_verdict_properties() {
        let fail_fast = SandboxEvictionVerdict::TargetIsCallerIdeFailFast { exit_code: 2 };
        assert!(fail_fast.is_caller_ide_fail_fast());
        assert!(!fail_fast.is_safe_to_purge());
        assert!(!fail_fast.is_already_cleaned());
        assert_eq!(fail_fast.exit_code(), Some(2));
        assert!(fail_fast.to_string().contains("TargetIsCallerIdeFailFast"));

        let safe = SandboxEvictionVerdict::SafeToPurge;
        assert!(!safe.is_caller_ide_fail_fast());
        assert!(safe.is_safe_to_purge());
        assert!(!safe.is_already_cleaned());
        assert_eq!(safe.exit_code(), None);
        assert_eq!(safe.to_string(), "SafeToPurge");

        let cleaned = SandboxEvictionVerdict::AlreadyCleaned;
        assert!(!cleaned.is_caller_ide_fail_fast());
        assert!(!cleaned.is_safe_to_purge());
        assert!(cleaned.is_already_cleaned());
        assert_eq!(cleaned.exit_code(), None);
        assert_eq!(cleaned.to_string(), "AlreadyCleaned");
    }

    #[test]
    fn test_caller_ide_self_preservation_fail_fast_without_force() {
        let caller_descriptor =
            SandboxInstanceDescriptor::caller_ide("ide-worker-1", "profiles/worker-1");

        // Fails fast with exit code 2 when force is false
        let verdict = evaluate_sandbox_eviction_safety(&caller_descriptor, None, false);
        assert_eq!(
            verdict,
            SandboxEvictionVerdict::TargetIsCallerIdeFailFast {
                exit_code: CALLER_IDE_FAIL_FAST_EXIT_CODE
            }
        );

        // Even if is_caller_ide is false on descriptor, matching caller_instance_id triggers fail-fast
        let target_descriptor = SandboxInstanceDescriptor::transient(
            "dev-session-active",
            "profiles/dev-session-active",
        );
        let verdict_detected =
            evaluate_sandbox_eviction_safety(&target_descriptor, Some("dev-session-active"), false);
        assert_eq!(
            verdict_detected,
            SandboxEvictionVerdict::TargetIsCallerIdeFailFast { exit_code: 2 }
        );
    }

    #[test]
    fn test_caller_ide_overridden_with_explicit_force() {
        let caller_descriptor =
            SandboxInstanceDescriptor::caller_ide("ide-worker-force", "profiles/worker-force");

        // When force is explicitly provided, eviction is permitted
        let verdict = evaluate_sandbox_eviction_safety(&caller_descriptor, None, true);
        assert_eq!(verdict, SandboxEvictionVerdict::SafeToPurge);

        let target_descriptor =
            SandboxInstanceDescriptor::transient("my-session", "profiles/my-session");
        let verdict_detected =
            evaluate_sandbox_eviction_safety(&target_descriptor, Some("my-session"), true);
        assert_eq!(verdict_detected, SandboxEvictionVerdict::SafeToPurge);
    }

    #[test]
    fn test_transient_sandbox_safe_to_purge_and_already_cleaned() {
        let normal_sandbox = SandboxInstanceDescriptor::transient(
            "agm-test-sandbox-55",
            "temp/sandboxes/agm-test-sandbox-55",
        );
        let verdict =
            evaluate_sandbox_eviction_safety(&normal_sandbox, Some("other-caller"), false);
        assert_eq!(verdict, SandboxEvictionVerdict::SafeToPurge);

        // Empty directory indicates already cleaned
        let empty_dir_sandbox = SandboxInstanceDescriptor::transient("agm-test-sandbox-empty", "");
        let verdict_empty = evaluate_sandbox_eviction_safety(&empty_dir_sandbox, None, false);
        assert_eq!(verdict_empty, SandboxEvictionVerdict::AlreadyCleaned);

        // Marked purged string indicates already cleaned
        let purged_sandbox =
            SandboxInstanceDescriptor::transient("agm-test-sandbox-done", "<cleaned>");
        let verdict_purged = evaluate_sandbox_eviction_safety(&purged_sandbox, None, false);
        assert_eq!(verdict_purged, SandboxEvictionVerdict::AlreadyCleaned);
    }

    #[test]
    fn test_plan_transient_sandbox_cleanup_blocks_caller_ide() {
        let caller = SandboxInstanceDescriptor::caller_ide("caller-protected", "profiles/main");
        let report = plan_transient_sandbox_cleanup(&caller);

        assert!(!report.is_fully_purged);
        assert_eq!(
            report.verdict,
            SandboxEvictionVerdict::TargetIsCallerIdeFailFast { exit_code: 2 }
        );
        assert!(report.directories_removed.is_empty());
        assert!(report.processes_terminated.is_empty());
        assert!(report.summary().contains("cleanup blocked"));
    }

    #[test]
    fn test_plan_transient_sandbox_cleanup_successful_purge() {
        let sandbox = SandboxInstanceDescriptor::transient(
            "agm-test-sandbox-101",
            "target/sandboxes/agm-test-sandbox-101",
        );
        let report = plan_transient_sandbox_cleanup(&sandbox);

        assert!(report.is_fully_purged);
        assert_eq!(report.verdict, SandboxEvictionVerdict::SafeToPurge);
        assert_eq!(
            report.directories_removed,
            vec!["target/sandboxes/agm-test-sandbox-101".to_string()]
        );
        assert!(report.processes_terminated.is_empty());
        assert!(report.summary().contains("cleanly purged"));
    }

    #[test]
    fn test_plan_transient_sandbox_cleanup_with_tracked_pids_and_force() {
        let caller =
            SandboxInstanceDescriptor::caller_ide("caller-session-forced", "profiles/forced");
        let tracked_pids = vec![9001, 9002];

        let report = plan_transient_sandbox_cleanup_with_options(
            &caller,
            Some("caller-session-forced"),
            true, // force is true
            &tracked_pids,
        );

        assert!(report.is_fully_purged);
        assert_eq!(report.verdict, SandboxEvictionVerdict::SafeToPurge);
        assert_eq!(
            report.directories_removed,
            vec!["profiles/forced".to_string()]
        );
        assert_eq!(report.processes_terminated, vec![9001, 9002]);
    }

    #[test]
    fn test_plan_transient_sandbox_cleanup_already_cleaned() {
        let sandbox = SandboxInstanceDescriptor::transient("agm-test-sandbox-prior", "<purged>");
        let report = plan_transient_sandbox_cleanup(&sandbox);

        assert!(report.is_fully_purged);
        assert_eq!(report.verdict, SandboxEvictionVerdict::AlreadyCleaned);
        assert!(report.directories_removed.is_empty());
        assert!(report.processes_terminated.is_empty());
    }

    #[test]
    fn test_serde_roundtrip_all_models() {
        let descriptor = SandboxInstanceDescriptor::new(
            "agm-test-sandbox-serde",
            "sandboxes/serde",
            123456789,
            false,
        );
        let serialized_desc = serde_json::to_string(&descriptor).expect("serialize descriptor");
        let deserialized_desc: SandboxInstanceDescriptor =
            serde_json::from_str(&serialized_desc).expect("deserialize descriptor");
        assert_eq!(descriptor, deserialized_desc);

        let report = SandboxCleanupReport {
            sandbox_id: "agm-test-sandbox-serde".to_string(),
            directories_removed: vec!["sandboxes/serde".to_string()],
            processes_terminated: vec![1010, 1011],
            verdict: SandboxEvictionVerdict::SafeToPurge,
            is_fully_purged: true,
        };
        let serialized_report = serde_json::to_string(&report).expect("serialize report");
        let deserialized_report: SandboxCleanupReport =
            serde_json::from_str(&serialized_report).expect("deserialize report");
        assert_eq!(report, deserialized_report);

        let state = TransientSandboxState::OrphanRemnantsDetected;
        let serialized_state = serde_json::to_string(&state).expect("serialize state");
        let deserialized_state: TransientSandboxState =
            serde_json::from_str(&serialized_state).expect("deserialize state");
        assert_eq!(state, deserialized_state);
    }
}
