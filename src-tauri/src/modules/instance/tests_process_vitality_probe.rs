//! Unit tests for Process Vitality Probe.

use super::process_vitality_probe::ProcessVitalityProbe;

#[test]
fn test_vitality_probe_pid_zero() {
    assert!(!ProcessVitalityProbe::is_pid_alive(0));
}

#[test]
fn test_vitality_probe_current_process() {
    let current_pid = std::process::id();
    assert!(ProcessVitalityProbe::is_pid_alive(current_pid));
}

#[test]
fn test_vitality_probe_dead_pid() {
    assert!(!ProcessVitalityProbe::is_pid_alive(999_999_999));
}
