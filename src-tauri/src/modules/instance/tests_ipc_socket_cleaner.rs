//! Unit tests for VS Code IPC Socket Cleaner.

use super::ipc_socket_cleaner::{clean_stale_ipc_sockets, is_stale_ipc_socket_name};
use tempfile::tempdir;

#[test]
fn test_is_stale_ipc_socket_name() {
    assert!(is_stale_ipc_socket_name("vscode-ipc-abc123.sock"));
    assert!(is_stale_ipc_socket_name("antigravity-ipc-worker.sock"));
    assert!(is_stale_ipc_socket_name("test_service.sock"));
    assert!(is_stale_ipc_socket_name(".org.chromium.Chromium.socket"));
    assert!(!is_stale_ipc_socket_name("regular_file.txt"));
    assert!(!is_stale_ipc_socket_name("settings.json"));
}

#[test]
fn test_clean_stale_ipc_sockets_nonexistent_dir() {
    let temp = tempdir().expect("create temp dir");
    let non_existent = temp.path().join("does_not_exist");
    let summary = clean_stale_ipc_sockets(&non_existent);
    assert_eq!(summary.files_scanned, 0);
    assert_eq!(summary.sockets_removed, 0);
}

#[test]
fn test_clean_stale_ipc_sockets_removes_matching_files() {
    let temp = tempdir().expect("create temp dir");
    let sock1 = temp.path().join("vscode-ipc-1234.sock");
    let sock2 = temp.path().join("antigravity-ipc-5678.sock");
    let keep = temp.path().join("keep_me.json");

    std::fs::write(&sock1, b"").expect("create sock1");
    std::fs::write(&sock2, b"").expect("create sock2");
    std::fs::write(&keep, b"{}").expect("create keep file");

    let summary = clean_stale_ipc_sockets(temp.path());
    assert_eq!(summary.files_scanned, 3);
    assert_eq!(summary.sockets_removed, 2);
    assert_eq!(summary.errors_encountered, 0);

    assert!(!sock1.exists());
    assert!(!sock2.exists());
    assert!(keep.exists());
}
