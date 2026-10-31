//! Unit tests for Instance Process Vitality Scanner.

use super::vitality_scanner::{is_process_alive, verify_instance_pids};

#[test]
fn test_pid_zero_is_never_alive() {
    assert!(!is_process_alive(0));
}

#[test]
fn test_current_process_is_always_alive() {
    let current_pid = std::process::id();
    assert!(is_process_alive(current_pid));
}

#[test]
fn test_verify_instance_pids_batch() {
    let current_pid = std::process::id();
    let dead_pid = 999_999_999;

    let results = verify_instance_pids(&[0, current_pid, dead_pid]);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (0, false));
    assert_eq!(results[1], (current_pid, true));
    assert_eq!(results[2], (dead_pid, false));
}
