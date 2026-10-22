//! Unit tests for ConnectionAgeSentinel (Imp-574)

use std::thread::sleep;
use std::time::Duration;

use super::connection_age_sentinel::{ConnectionAgeSentinel, DEFAULT_MAX_CONNECTION_AGE};

#[test]
fn test_default_sentinel_initialization() {
    let sentinel = ConnectionAgeSentinel::default();
    assert!(sentinel.sweep_stale_connections().is_empty());
}

#[test]
fn test_register_and_touch_within_age() {
    let sentinel = ConnectionAgeSentinel::new(Duration::from_secs(60));
    sentinel.register_connection("conn-1".to_string(), "https://upstream.internal".to_string());

    // Connection is freshly registered, should not retire
    let should_retire = sentinel.touch_connection("conn-1");
    assert!(!should_retire);

    // Non-existent connection returns false
    assert!(!sentinel.touch_connection("conn-unknown"));
}

#[test]
fn test_touch_connection_exceeding_max_age_returns_true() {
    let sentinel = ConnectionAgeSentinel::new(Duration::from_millis(15));
    sentinel.register_connection("conn-short-lived".to_string(), "https://upstream.internal".to_string());

    // Sleep past max_age
    sleep(Duration::from_millis(25));

    let should_retire = sentinel.touch_connection("conn-short-lived");
    assert!(should_retire, "Connection past max age must signal retirement on touch");
}

#[test]
fn test_sweep_stale_connections_removes_expired_entries() {
    let sentinel = ConnectionAgeSentinel::new(Duration::from_millis(30));
    sentinel.register_connection("conn-old".to_string(), "https://old.internal".to_string());

    sleep(Duration::from_millis(35));
    sentinel.register_connection("conn-fresh".to_string(), "https://fresh.internal".to_string());

    let swept = sentinel.sweep_stale_connections();
    assert_eq!(swept, vec!["conn-old".to_string()]);

    // Old connection has been removed from sentinel
    assert!(!sentinel.touch_connection("conn-old"));
    // Fresh connection remains active
    assert!(!sentinel.touch_connection("conn-fresh"));
}

#[test]
fn test_remove_connection_explicit() {
    let sentinel = ConnectionAgeSentinel::new(Duration::from_secs(10));
    sentinel.register_connection("conn-dropped".to_string(), "https://drop.internal".to_string());

    sentinel.remove_connection("conn-dropped");
    assert!(!sentinel.touch_connection("conn-dropped"));
    assert!(sentinel.sweep_stale_connections().is_empty());
}
