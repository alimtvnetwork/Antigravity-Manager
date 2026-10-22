//! Unit tests for StreamingTimeoutSentinel (Imp-565)

use std::thread::sleep;
use std::time::Duration;

use super::streaming_timeout_sentinel::{
    StreamingTimeoutConfig, StreamingTimeoutError, StreamingTimeoutSentinel,
};

#[test]
fn test_default_sentinel_initialization() {
    let mut sentinel = StreamingTimeoutSentinel::new(None);
    assert_eq!(sentinel.chunks_received(), 0);
    assert_eq!(sentinel.bytes_received(), 0);
    assert!(sentinel.check_liveness().is_ok());
}

#[test]
fn test_record_chunk_metrics() {
    let mut sentinel = StreamingTimeoutSentinel::new(None);
    sentinel.record_chunk(128);
    sentinel.record_chunk(256);
    sentinel.record_chunk(64);

    assert_eq!(sentinel.chunks_received(), 3);
    assert_eq!(sentinel.bytes_received(), 448);
    assert!(sentinel.check_liveness().is_ok());
}

#[test]
fn test_inactivity_timeout_detection() {
    let config = StreamingTimeoutConfig {
        max_inactivity_duration: Duration::from_millis(15),
        total_stream_timeout: Duration::from_secs(10),
    };
    let mut sentinel = StreamingTimeoutSentinel::new(Some(config));

    // Sleep past the inactivity limit
    sleep(Duration::from_millis(25));

    let result = sentinel.check_liveness();
    match result {
        Err(StreamingTimeoutError::InactivityTimeout { silence_secs: _, chunks_received }) => {
            assert_eq!(chunks_received, 0);
        }
        other => panic!("Expected InactivityTimeout, got {:?}", other),
    }

    // Subsequent check should fail with AlreadyAborted
    assert_eq!(sentinel.check_liveness(), Err(StreamingTimeoutError::AlreadyAborted));
}

#[test]
fn test_record_chunk_resets_inactivity() {
    let config = StreamingTimeoutConfig {
        max_inactivity_duration: Duration::from_millis(40),
        total_stream_timeout: Duration::from_secs(10),
    };
    let mut sentinel = StreamingTimeoutSentinel::new(Some(config));

    sleep(Duration::from_millis(20));
    sentinel.record_chunk(100);
    sleep(Duration::from_millis(20));

    // Inactivity was reset by record_chunk, so total elapsed inactivity is <= 20ms < 40ms
    assert!(sentinel.check_liveness().is_ok());
    assert_eq!(sentinel.chunks_received(), 1);
    assert_eq!(sentinel.bytes_received(), 100);
}

#[test]
fn test_total_stream_timeout() {
    let config = StreamingTimeoutConfig {
        max_inactivity_duration: Duration::from_millis(100),
        total_stream_timeout: Duration::from_millis(25),
    };
    let mut sentinel = StreamingTimeoutSentinel::new(Some(config));

    // Keep active with chunks so inactivity doesn't trigger
    sleep(Duration::from_millis(15));
    sentinel.record_chunk(50);
    sleep(Duration::from_millis(20));

    let result = sentinel.check_liveness();
    match result {
        Err(StreamingTimeoutError::TotalDurationTimeout { bytes_received, .. }) => {
            assert_eq!(bytes_received, 50);
        }
        other => panic!("Expected TotalDurationTimeout, got {:?}", other),
    }
}
