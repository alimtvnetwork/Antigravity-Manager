//! Unit tests for Child Process Termination Guard.

use super::child_termination_guard::ChildTerminationGuard;

#[test]
fn test_terminate_pid_zero_returns_false() {
    let res = ChildTerminationGuard::terminate_with_ladder(0, 100);
    assert!(res.is_ok());
    assert!(!res.unwrap());
}

#[test]
fn test_terminate_dead_pid_returns_false() {
    // Arbitrary non-existent PID
    let res = ChildTerminationGuard::terminate_with_ladder(999_999_999, 50);
    assert!(res.is_ok());
    // Since PID does not exist or cannot be opened, should return false gracefully
    assert!(!res.unwrap());
}
