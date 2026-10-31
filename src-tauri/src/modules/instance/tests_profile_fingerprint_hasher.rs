//! Unit tests for Profile Structural Fingerprint Hasher.

use super::profile_fingerprint_hasher::compute_profile_fingerprint;
use tempfile::tempdir;

#[test]
fn test_compute_profile_fingerprint_empty_dir() {
    let temp = tempdir().expect("create temp dir");
    let res = compute_profile_fingerprint(temp.path());
    assert!(res.is_ok());

    let hash_1 = res.unwrap();
    assert_eq!(hash_1.len(), 64); // SHA-256 hex string

    // Recomputing on unchanged directory produces exact same hash
    let hash_2 = compute_profile_fingerprint(temp.path()).unwrap();
    assert_eq!(hash_1, hash_2);
}

#[test]
fn test_compute_profile_fingerprint_detects_file_changes() {
    let temp = tempdir().expect("create temp dir");
    let user_dir = temp.path().join("User");
    std::fs::create_dir_all(&user_dir).expect("create User dir");

    let settings_path = user_dir.join("settings.json");
    std::fs::write(&settings_path, "{\"editor.fontSize\": 14}").expect("write settings");

    let initial_hash = compute_profile_fingerprint(temp.path()).expect("hash 1");

    // Modify settings content
    std::fs::write(&settings_path, "{\"editor.fontSize\": 16}").expect("modify settings");
    let modified_hash = compute_profile_fingerprint(temp.path()).expect("hash 2");

    assert_ne!(initial_hash, modified_hash);
}

#[test]
fn test_compute_profile_fingerprint_keybindings() {
    let temp = tempdir().expect("create temp dir");
    let user_dir = temp.path().join("User");
    std::fs::create_dir_all(&user_dir).expect("create User dir");

    let keybindings_path = user_dir.join("keybindings.json");
    std::fs::write(&keybindings_path, "[]").expect("write keybindings");

    let hash_with_keybindings = compute_profile_fingerprint(temp.path()).expect("hash");
    assert_eq!(hash_with_keybindings.len(), 64);
}
