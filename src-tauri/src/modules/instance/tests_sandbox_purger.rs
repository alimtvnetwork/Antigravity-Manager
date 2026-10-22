//! Unit tests for Ephemeral Sandbox Purger (Track B Step 25)

use std::fs;
use std::path::PathBuf;

use super::sandbox_purger::{purge_ephemeral_sandboxes, SandboxPurgeReport};

fn create_temp_test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "agm_test_sandbox_{}_{}",
        name,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&dir), "create test fixture dir");
    dir
}

#[test]
fn test_purge_non_existent_directory_returns_empty_report() {
    let non_existent = std::env::temp_dir().join("agm_non_existent_dir_probe_12345");
    if non_existent.exists() {
        // Justification: Clean existing fixture directory if present
        crate::error::record_ignored(fs::remove_dir_all(&non_existent), "cleanup existing dir");
    }

    let report = purge_ephemeral_sandboxes(&non_existent).expect("Expected Ok report on missing dir");
    assert!(report.deleted_paths.is_empty());
    assert!(report.skipped_locked_paths.is_empty());
    assert_eq!(report.total_reclaimed_bytes, 0);
}

#[test]
fn test_purge_regular_directories_are_preserved() {
    let root = create_temp_test_dir("preserve_regular");
    let regular_a = root.join("production_profile");
    let regular_b = root.join("user-workspace-data");
    let regular_c = root.join("default_settings");

    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&regular_a), "create regular_a");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&regular_b), "create regular_b");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&regular_c), "create regular_c");

    let report = purge_ephemeral_sandboxes(&root).expect("Expected Ok purge");
    assert_eq!(report.deleted_paths.len(), 0);
    assert_eq!(report.total_reclaimed_bytes, 0);

    assert!(regular_a.exists());
    assert!(regular_b.exists());
    assert!(regular_c.exists());

    // Justification: Best-effort test cleanup
    crate::error::record_ignored(fs::remove_dir_all(&root), "clean test root");
}

#[test]
fn test_purge_ephemeral_sandboxes_by_prefix_and_reclaims_bytes() {
    let root = create_temp_test_dir("purge_prefixes");

    // Create ephemeral directories
    let sandbox_a = root.join(".antigravity-sandbox-worker-1");
    let sandbox_b = root.join(".test-profile-alpha");
    let sandbox_c = root.join("agm-test-e2e-run");
    let preserved = root.join("real-developer-profile");

    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&sandbox_a), "create sandbox_a");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&sandbox_b), "create sandbox_b");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&sandbox_c), "create sandbox_c");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&preserved), "create preserved");

    // Write files to verify byte reclamation
    let payload = vec![0u8; 1024]; // 1 KB
    // Justification: Test fixture file write
    crate::error::record_ignored(fs::write(sandbox_a.join("state.bin"), &payload), "write state.bin");
    // Justification: Test fixture file write
    crate::error::record_ignored(fs::write(sandbox_b.join("config.json"), &payload), "write config.json");
    // Justification: Test fixture file write
    crate::error::record_ignored(fs::write(sandbox_c.join("cache.dat"), &payload), "write cache.dat");
    // Justification: Test fixture file write
    crate::error::record_ignored(fs::write(preserved.join("important.txt"), &payload), "write important.txt");

    let report = purge_ephemeral_sandboxes(&root).expect("Expected Ok purge");

    assert_eq!(report.deleted_paths.len(), 3);
    assert!(report.total_reclaimed_bytes >= 3072);

    assert!(!sandbox_a.exists(), ".antigravity-sandbox- should have been removed");
    assert!(!sandbox_b.exists(), ".test-profile- should have been removed");
    assert!(!sandbox_c.exists(), "agm-test- should have been removed");
    assert!(preserved.exists(), "Non-ephemeral profile must be preserved");

    // Justification: Best-effort test cleanup
    crate::error::record_ignored(fs::remove_dir_all(&root), "clean test root");
}

#[test]
fn test_purge_ephemeral_sandboxes_empty_sandbox_folder() {
    let root = create_temp_test_dir("purge_empty");
    let empty_sandbox = root.join(".test-profile-empty");
    // Justification: Test fixture directory creation in temporary directory
    crate::error::record_ignored(fs::create_dir_all(&empty_sandbox), "create empty_sandbox");

    let report = purge_ephemeral_sandboxes(&root).expect("Expected Ok purge");
    assert_eq!(report.deleted_paths.len(), 1);
    assert_eq!(report.total_reclaimed_bytes, 0);
    assert!(!empty_sandbox.exists());

    // Justification: Best-effort test cleanup
    crate::error::record_ignored(fs::remove_dir_all(&root), "clean test root");
}
