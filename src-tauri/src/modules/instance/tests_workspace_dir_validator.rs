//! Unit tests for Workspace Directory Validator.

use super::workspace_dir_validator::WorkspaceDirValidator;
use tempfile::tempdir;

#[test]
fn test_validate_workspace_nonexistent() {
    let temp = tempdir().expect("create temp dir");
    let missing = temp.path().join("missing_workspace");

    let report = WorkspaceDirValidator::validate_workspace(&missing).expect("validate workspace");
    assert!(!report.exists);
    assert!(!report.is_directory);
    assert!(!report.is_writable);
    assert!(report.detected_project_markers.is_empty());
}

#[test]
fn test_validate_workspace_valid_directory() {
    let temp = tempdir().expect("create temp dir");

    let report =
        WorkspaceDirValidator::validate_workspace(temp.path()).expect("validate workspace");
    assert!(report.exists);
    assert!(report.is_directory);
    assert!(report.is_writable);
}

#[test]
fn test_validate_workspace_detects_markers() {
    let temp = tempdir().expect("create temp dir");

    // Create candidate project markers
    std::fs::create_dir_all(temp.path().join(".git")).expect("create .git");
    std::fs::write(temp.path().join("Cargo.toml"), b"[package]").expect("create Cargo.toml");

    let report =
        WorkspaceDirValidator::validate_workspace(temp.path()).expect("validate workspace");
    assert!(report.exists);
    assert!(report
        .detected_project_markers
        .contains(&".git".to_string()));
    assert!(report
        .detected_project_markers
        .contains(&"Cargo.toml".to_string()));
    assert!(!report
        .detected_project_markers
        .contains(&"package.json".to_string()));
}
