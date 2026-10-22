//! Unit tests for Dynamic Connection Pool Sizer (Imp-554).

use super::dynamic_pool_sizer::{DynamicPoolSizer, MAX_IDLE_CONNECTIONS, MIN_IDLE_CONNECTIONS};

#[test]
fn test_dynamic_pool_sizer_initial_state() {
    let sizer = DynamicPoolSizer::new();
    assert_eq!(sizer.current_limit(), MIN_IDLE_CONNECTIONS);
    assert_eq!(MIN_IDLE_CONNECTIONS, 5);
    assert_eq!(MAX_IDLE_CONNECTIONS, 50);
}

#[test]
fn test_dynamic_pool_sizer_request_scaling() {
    let sizer = DynamicPoolSizer::new();

    // 1 active request: (1 + 5) = 6
    let limit = sizer.on_request_started();
    assert_eq!(limit, 6);
    assert_eq!(sizer.current_limit(), 6);

    // 5 active requests: (5 + 5) = 10
    for _ in 1..5 {
        sizer.on_request_started();
    }
    assert_eq!(sizer.current_limit(), 10);
}

#[test]
fn test_dynamic_pool_sizer_upper_bound_clamping() {
    let sizer = DynamicPoolSizer::new();

    // Start 100 requests (exceeding MAX_IDLE_CONNECTIONS = 50)
    for _ in 0..100 {
        sizer.on_request_started();
    }
    assert_eq!(sizer.current_limit(), MAX_IDLE_CONNECTIONS);
}

#[test]
fn test_dynamic_pool_sizer_request_completion() {
    let sizer = DynamicPoolSizer::new();

    // Start 10 requests -> limit: 15
    for _ in 0..10 {
        sizer.on_request_started();
    }
    assert_eq!(sizer.current_limit(), 15);

    // Finish 5 requests -> active = 5 -> target = 10
    for _ in 0..5 {
        sizer.on_request_finished();
    }
    assert_eq!(sizer.current_limit(), 10);

    // Finish remaining 5 requests -> active = 0 -> target = 5 (MIN_IDLE)
    for _ in 0..5 {
        sizer.on_request_finished();
    }
    assert_eq!(sizer.current_limit(), MIN_IDLE_CONNECTIONS);

    // Further finishes saturate without underflow
    sizer.on_request_finished();
    assert_eq!(sizer.current_limit(), MIN_IDLE_CONNECTIONS);
}
