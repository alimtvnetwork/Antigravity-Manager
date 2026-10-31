//! Unit tests for WindowStateRestorer and WindowBounds.

use super::window_state_restorer::{WindowBounds, WindowStateRestorer};
use tempfile::tempdir;

#[test]
fn test_window_bounds_default() {
    let bounds = WindowBounds::default();
    assert_eq!(bounds.x, 100);
    assert_eq!(bounds.y, 100);
    assert_eq!(bounds.width, 1280);
    assert_eq!(bounds.height, 800);
    assert!(!bounds.is_maximized);
}

#[test]
fn test_window_state_save_and_load_roundtrip() {
    let temp = tempdir().expect("create temp dir");
    let custom_bounds = WindowBounds {
        x: 250,
        y: 180,
        width: 1920,
        height: 1080,
        is_maximized: true,
    };

    let save_res = WindowStateRestorer::save_bounds(temp.path(), &custom_bounds);
    assert!(save_res.is_ok());

    let loaded = WindowStateRestorer::load_bounds(temp.path());
    assert_eq!(loaded.x, 250);
    assert_eq!(loaded.y, 180);
    assert_eq!(loaded.width, 1920);
    assert_eq!(loaded.height, 1080);
    assert!(loaded.is_maximized);
}

#[test]
fn test_window_state_load_nonexistent_file_returns_default() {
    let temp = tempdir().expect("create temp dir");
    let loaded = WindowStateRestorer::load_bounds(temp.path());
    assert_eq!(loaded.x, 100);
    assert_eq!(loaded.y, 100);
    assert_eq!(loaded.width, 1280);
    assert_eq!(loaded.height, 800);
    assert!(!loaded.is_maximized);
}

#[test]
fn test_window_state_load_corrupted_file_returns_default() {
    let temp = tempdir().expect("create temp dir");
    let state_file = temp.path().join("window_state.json");
    std::fs::write(&state_file, "{ corrupted json: [ true").expect("write corrupt file");

    let loaded = WindowStateRestorer::load_bounds(temp.path());
    assert_eq!(loaded.x, 100);
    assert_eq!(loaded.y, 100);
    assert_eq!(loaded.width, 1280);
    assert_eq!(loaded.height, 800);
    assert!(!loaded.is_maximized);
}
