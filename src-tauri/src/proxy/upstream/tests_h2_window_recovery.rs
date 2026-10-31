//! Unit tests for H2WindowRecoveryTracker (Imp-570)

use std::sync::Arc;
use std::thread;

use super::h2_window_recovery::{H2WindowRecoveryTracker, DEFAULT_H2_STALL_THRESHOLD};

#[test]
fn test_default_tracker_initialization() {
    let tracker = H2WindowRecoveryTracker::default();
    assert_eq!(tracker.stall_count(), 0);
    assert!(!tracker.should_downgrade_to_http1());
}

#[test]
fn test_record_stall_increments_counter() {
    let tracker = H2WindowRecoveryTracker::new(None);
    assert_eq!(tracker.record_stall(), 1);
    assert_eq!(tracker.record_stall(), 2);
    assert_eq!(tracker.stall_count(), 2);
    assert!(!tracker.should_downgrade_to_http1());
}

#[test]
fn test_threshold_downgrade_trigger() {
    let tracker = H2WindowRecoveryTracker::new(Some(DEFAULT_H2_STALL_THRESHOLD));

    for _ in 0..(DEFAULT_H2_STALL_THRESHOLD - 1) {
        tracker.record_stall();
        assert!(!tracker.should_downgrade_to_http1());
    }

    // Reach threshold
    tracker.record_stall();
    assert!(tracker.should_downgrade_to_http1());
    assert_eq!(tracker.stall_count(), DEFAULT_H2_STALL_THRESHOLD);
}

#[test]
fn test_record_success_resets_counter() {
    let tracker = H2WindowRecoveryTracker::new(Some(2));
    tracker.record_stall();
    tracker.record_stall();
    assert!(tracker.should_downgrade_to_http1());

    tracker.record_success();
    assert_eq!(tracker.stall_count(), 0);
    assert!(!tracker.should_downgrade_to_http1());
}

#[test]
fn test_custom_threshold_single_stall() {
    let tracker = H2WindowRecoveryTracker::new(Some(1));
    assert!(!tracker.should_downgrade_to_http1());

    tracker.record_stall();
    assert!(tracker.should_downgrade_to_http1());
}

#[test]
fn test_concurrent_stall_recording() {
    let tracker = Arc::new(H2WindowRecoveryTracker::new(Some(100)));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let t = Arc::clone(&tracker);
        handles.push(thread::spawn(move || {
            for _ in 0..10 {
                t.record_stall();
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked");
    }

    assert_eq!(tracker.stall_count(), 100);
    assert!(tracker.should_downgrade_to_http1());
}
