//! Unit tests for Upstream HTTP/2 Flow Controller (Imp-558).

use super::http2_flow_controller::{
    tune_http2_flow_control, DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE,
    DEFAULT_INITIAL_STREAM_WINDOW_SIZE, DEFAULT_MAX_CONCURRENT_STREAMS,
    MAX_STREAM_WINDOW_SIZE, MIN_STREAM_WINDOW_SIZE,
};

#[test]
fn test_tune_http2_flow_control_default() {
    let cfg = tune_http2_flow_control(None, None);
    assert_eq!(cfg.stream_window_size, DEFAULT_INITIAL_STREAM_WINDOW_SIZE);
    assert_eq!(cfg.connection_window_size, DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE);
    assert_eq!(cfg.max_concurrent_streams, DEFAULT_MAX_CONCURRENT_STREAMS);
    assert!(cfg.adaptive_tuning);
}

#[test]
fn test_tune_http2_flow_control_small_payload() {
    let cfg = tune_http2_flow_control(Some(10 * 1024), None);
    assert_eq!(cfg.stream_window_size, 128 * 1024);
    assert_eq!(cfg.connection_window_size, DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE);
}

#[test]
fn test_tune_http2_flow_control_large_payload() {
    let large_size = 8 * 1024 * 1024; // 8MB
    let cfg = tune_http2_flow_control(Some(large_size), None);
    assert_eq!(cfg.stream_window_size, 8 * 1024 * 1024);
    assert_eq!(cfg.connection_window_size, 32 * 1024 * 1024); // 8MB * 4
}

#[test]
fn test_tune_http2_flow_control_concurrency_scaling() {
    let cfg = tune_http2_flow_control(None, Some(256));
    assert_eq!(cfg.max_concurrent_streams, 256);
}

#[test]
fn test_tune_http2_flow_control_bounds_clamping() {
    let cfg = tune_http2_flow_control(Some(100 * 1024 * 1024), Some(2000));
    assert_eq!(cfg.stream_window_size, MAX_STREAM_WINDOW_SIZE);
    assert_eq!(cfg.max_concurrent_streams, 512); // clamped at 512
}
