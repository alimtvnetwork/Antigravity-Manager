//! Unit tests for Hourly Switch Entropy & Oscillation Guard (Imp-555).

use super::hourly_entropy_guard::HourlyEntropyGuard;
use std::time::Duration;

#[test]
fn test_hourly_entropy_guard_empty() {
    let mut guard = HourlyEntropyGuard::default();
    assert_eq!(guard.calculate_entropy(), 1.0);
    assert!(!guard.is_oscillating());
}

#[test]
fn test_hourly_entropy_guard_single_account_low_entropy() {
    let mut guard = HourlyEntropyGuard::new(Duration::from_secs(3600), 0.35);

    for _ in 0..6 {
        guard.record_switch("account-a");
    }

    // Only 1 distinct account -> distinct_count <= 1 -> entropy = 0.0
    let entropy = guard.calculate_entropy();
    assert_eq!(entropy, 0.0);
    // Since count >= 5 and entropy < 0.35 -> is_oscillating = true
    assert!(guard.is_oscillating());
}

#[test]
fn test_hourly_entropy_guard_diverse_accounts_high_entropy() {
    let mut guard = HourlyEntropyGuard::new(Duration::from_secs(3600), 0.35);

    guard.record_switch("acc-1");
    guard.record_switch("acc-2");
    guard.record_switch("acc-3");
    guard.record_switch("acc-4");
    guard.record_switch("acc-5");

    let entropy = guard.calculate_entropy();
    // Uniform distribution across 5 accounts -> entropy should be close to 1.0
    assert!(entropy > 0.9);
    assert!(!guard.is_oscillating());
}

#[test]
fn test_hourly_entropy_guard_below_minimum_switch_threshold() {
    let mut guard = HourlyEntropyGuard::new(Duration::from_secs(3600), 0.35);

    // 4 switches (less than 5 required for oscillation flag)
    for _ in 0..4 {
        guard.record_switch("account-loop");
    }

    // Even though entropy is 0.0, switches < 5 -> not flagged as oscillating yet
    assert_eq!(guard.calculate_entropy(), 0.0);
    assert!(!guard.is_oscillating());
}
