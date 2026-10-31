//! Unit tests for Transient Sandbox Manager (`Imp-621`).

use super::transient_sandbox_manager::{
    clean_transient_sandbox, create_transient_sandbox, evaluate_caller_protection,
    sweep_stale_sandboxes, CallerProtectionVerdict, SUBDIR_USER_DATA, SUBDIR_WORKSPACE_STORAGE,
    TRANSIENT_SANDBOX_PREFIX,
};
use std::fs;

#[test]
fn test_caller_protection_blocks_matching_id() {
    let verdict = evaluate_caller_protection(
        "inst_default",
        Some("inst_default"),
        Some(1001),
        Some(2002),
        false,
    );
    assert!(!verdict.is_allowed());
    match verdict {
        CallerProtectionVerdict::BlockedCallerProtection { caller_id, reason } => {
            assert_eq!(caller_id, "inst_default");
            assert!(reason.contains("Termination blocked without --force"));
        }
        _ => panic!("Expected BlockedCallerProtection"),
    }
}

#[test]
fn test_caller_protection_blocks_matching_pid() {
    let verdict = evaluate_caller_protection(
        "inst_other",
        Some("inst_caller"),
        Some(5555),
        Some(5555),
        false,
    );
    assert!(!verdict.is_allowed());
}

#[test]
fn test_caller_protection_allows_with_force() {
    let verdict = evaluate_caller_protection(
        "inst_default",
        Some("inst_default"),
        Some(1001),
        Some(1001),
        true, // force = true
    );
    assert!(verdict.is_allowed());
}

#[test]
fn test_caller_protection_allows_distinct_instance() {
    let verdict = evaluate_caller_protection(
        "inst_sandbox_1",
        Some("inst_active_dev"),
        Some(4001),
        Some(9999),
        false,
    );
    assert!(verdict.is_allowed());
}

#[test]
fn test_transient_sandbox_lifecycle() {
    let temp_root = std::env::temp_dir().join(format!("agm_test_sbx_{}", uuid::Uuid::new_v4()));

    // 1. Create sandbox
    let mut record = create_transient_sandbox(&temp_root, 18888).expect("create sandbox failed");
    assert!(record.sandbox_id.starts_with(TRANSIENT_SANDBOX_PREFIX));
    assert!(record.profile_dir.exists());
    assert!(record.workspace_storage_dir.exists());

    let settings_path = record
        .profile_dir
        .join(SUBDIR_USER_DATA)
        .join("settings.json");
    assert!(settings_path.exists());

    // 2. Add sample file in workspaceStorage
    let dummy_ws_file = record.workspace_storage_dir.join("state.vscdb");
    fs::write(&dummy_ws_file, b"sample db content").expect("write sample ws db");

    // 3. Clean with preserve_workspace_storage = true
    let rep_preserved =
        clean_transient_sandbox(&mut record, true).expect("clean with preservation");
    assert!(rep_preserved.is_success);
    assert!(rep_preserved.workspace_storage_preserved);
    assert!(dummy_ws_file.exists());
    assert!(!settings_path.exists()); // settings was scrubbed

    // 4. Clean with preserve_workspace_storage = false (full purge)
    record.is_cleaned = false; // Reset to allow full purge
    let rep_purged = clean_transient_sandbox(&mut record, false).expect("full purge");
    assert!(rep_purged.is_success);
    assert!(!record.profile_dir.exists());

    // Cleanup temp root
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_sweep_stale_sandboxes() {
    let temp_root = std::env::temp_dir().join(format!("agm_sweep_test_{}", uuid::Uuid::new_v4()));
    let sbx1_dir = temp_root.join(format!("{}old_1", TRANSIENT_SANDBOX_PREFIX));
    let sbx2_dir = temp_root.join("permanent_instance_not_test");

    fs::create_dir_all(&sbx1_dir).expect("create sbx1");
    fs::create_dir_all(&sbx2_dir).expect("create sbx2");

    // Set max_age = 0 with current_epoch_s = 100000 to force sweep of sbx1
    let reports = sweep_stale_sandboxes(&temp_root, 0, 100_000).expect("sweep sandboxes");
    assert_eq!(reports.len(), 1);
    assert!(!sbx1_dir.exists());
    assert!(sbx2_dir.exists()); // permanent instance preserved

    let _ = fs::remove_dir_all(&temp_root);
}
