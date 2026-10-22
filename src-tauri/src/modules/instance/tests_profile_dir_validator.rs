//! Unit tests for Profile Sandboxed Directory Validator.

use super::profile_dir_validator::{validate_profile_directory, ProfileDirValidationError};
use tempfile::tempdir;

#[test]
fn test_validate_profile_directory_valid() {
    let base_temp = tempdir().expect("create base temp dir");
    let profile_temp = base_temp.path().join("profiles").join("inst-01");
    std::fs::create_dir_all(&profile_temp).expect("create profile dir");

    let res = validate_profile_directory(&profile_temp, base_temp.path());
    assert!(res.is_ok());
}

#[test]
fn test_validate_profile_directory_not_found() {
    let base_temp = tempdir().expect("create base temp dir");
    let non_existent = base_temp.path().join("profiles").join("missing");

    let res = validate_profile_directory(&non_existent, base_temp.path());
    assert!(matches!(res, Err(ProfileDirValidationError::NotFound(_))));
}

#[test]
fn test_validate_profile_directory_not_a_directory() {
    let base_temp = tempdir().expect("create base temp dir");
    let file_path = base_temp.path().join("file.txt");
    std::fs::write(&file_path, "not a directory").expect("write file");

    let res = validate_profile_directory(&file_path, base_temp.path());
    assert!(matches!(res, Err(ProfileDirValidationError::NotADirectory(_))));
}

#[test]
fn test_validate_profile_directory_path_traversal() {
    let base_temp = tempdir().expect("create base temp dir");
    let other_temp = tempdir().expect("create other temp dir");

    let res = validate_profile_directory(other_temp.path(), base_temp.path());
    assert!(matches!(
        res,
        Err(ProfileDirValidationError::PathTraversalDetected { .. })
    ));
}
