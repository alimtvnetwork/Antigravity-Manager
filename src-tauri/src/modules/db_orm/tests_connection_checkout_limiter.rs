//! Unit tests for ConnectionCheckoutLimiter (Imp-568)

use std::sync::Arc;
use std::thread;

use super::connection_checkout_limiter::{
    ConnectionCheckoutLimiter, DEFAULT_MAX_BACKGROUND_CHECKOUTS,
};

#[test]
fn test_default_limiter_permits_up_to_capacity() {
    let limiter = ConnectionCheckoutLimiter::default();
    assert_eq!(limiter.current_background_load(), 0);

    let mut guards = Vec::new();
    for _ in 0..DEFAULT_MAX_BACKGROUND_CHECKOUTS {
        let guard = limiter.try_acquire_background_permit();
        assert!(guard.is_some(), "Expected permit to be granted within capacity");
        guards.push(guard.unwrap());
    }

    assert_eq!(limiter.current_background_load(), DEFAULT_MAX_BACKGROUND_CHECKOUTS);

    // Capacity is saturated; next acquire should return None
    let extra = limiter.try_acquire_background_permit();
    assert!(extra.is_none(), "Expected permit to be denied when saturated");
}

#[test]
fn test_guard_drop_releases_permit() {
    let limiter = ConnectionCheckoutLimiter::new(2);

    let g1 = limiter.try_acquire_background_permit();
    let g2 = limiter.try_acquire_background_permit();
    assert!(g1.is_some());
    assert!(g2.is_some());
    assert_eq!(limiter.current_background_load(), 2);

    // Saturated
    assert!(limiter.try_acquire_background_permit().is_none());

    // Drop one guard
    drop(g1);
    assert_eq!(limiter.current_background_load(), 1);

    // Can acquire again
    let g3 = limiter.try_acquire_background_permit();
    assert!(g3.is_some());
    assert_eq!(limiter.current_background_load(), 2);
}

#[test]
fn test_custom_capacity_and_zero_fallback() {
    let zero_limiter = ConnectionCheckoutLimiter::new(0);
    // Should fallback to DEFAULT_MAX_BACKGROUND_CHECKOUTS (4)
    let mut guards = Vec::new();
    for _ in 0..4 {
        guards.push(zero_limiter.try_acquire_background_permit().unwrap());
    }
    assert!(zero_limiter.try_acquire_background_permit().is_none());

    let single_limiter = ConnectionCheckoutLimiter::new(1);
    let guard = single_limiter.try_acquire_background_permit();
    assert!(guard.is_some());
    assert!(single_limiter.try_acquire_background_permit().is_none());
}

#[test]
fn test_thread_concurrent_checkout_and_drop() {
    let limiter = Arc::new(ConnectionCheckoutLimiter::new(3));
    let mut handles = Vec::new();

    for _ in 0..6 {
        let lim = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            for _ in 0..10 {
                if let Some(guard) = lim.try_acquire_background_permit() {
                    assert!(lim.current_background_load() <= 3);
                    drop(guard);
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked");
    }

    assert_eq!(limiter.current_background_load(), 0);
}
