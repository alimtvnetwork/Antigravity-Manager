//! Unit tests for Sandboxed Storage Obsolete File Cleaner.

use super::sandboxed_storage_cleaner::sweep_obsolete_instance_files;
use tempfile::tempdir;

#[test]
fn test_sweep_obsolete_instance_files_empty_dir() {
    let temp = tempdir().expect("create temp dir");
    let report = sweep_obsolete_instance_files(temp.path()).expect("sweep files");
    assert!(report.removed_files.is_empty());
    assert_eq!(report.bytes_reclaimed, 0);
}

#[test]
fn test_sweep_obsolete_instance_files_removes_obsolete_patterns() {
    let temp = tempdir().expect("create temp dir");

    let obsolete_bak = temp.path().join("config.json.bak");
    let obsolete_old = temp.path().join("state.old");
    let obsolete_tmp = temp.path().join("temp-snapshot-123.tmp");
    let persistent_file = temp.path().join("settings.json");

    std::fs::write(&obsolete_bak, b"backup data").expect("write bak");
    std::fs::write(&obsolete_old, b"old data").expect("write old");
    std::fs::write(&obsolete_tmp, b"temp data").expect("write tmp");
    std::fs::write(&persistent_file, b"persistent data").expect("write persistent");

    let report = sweep_obsolete_instance_files(temp.path()).expect("sweep files");

    assert_eq!(report.removed_files.len(), 3);
    assert!(report.bytes_reclaimed > 0);

    assert!(!obsolete_bak.exists());
    assert!(!obsolete_old.exists());
    assert!(!obsolete_tmp.exists());
    assert!(persistent_file.exists());
}
