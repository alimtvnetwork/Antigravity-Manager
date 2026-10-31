// Unit Test Suite for Client Request Cancellation Watcher (Imp-542)
// Validates initial uncancelled state, atomic cancellation signaling,
// thread-safe flag cloning, and idempotence of cancel invocations.

#[cfg(test)]
mod tests {
    use crate::proxy::handlers::common::request_cancellation_watcher::RequestCancellationWatcher;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_request_cancellation_watcher_default_state() {
        let watcher = RequestCancellationWatcher::default();
        assert!(!watcher.is_cancelled());
        assert!(!watcher.flag().load(Ordering::SeqCst));
    }

    #[test]
    fn test_request_cancellation_watcher_cancel_triggers_flag() {
        let watcher = RequestCancellationWatcher::new();
        assert!(!watcher.is_cancelled());

        watcher.cancel();
        assert!(watcher.is_cancelled());
        assert!(watcher.flag().load(Ordering::SeqCst));

        // Multiple cancellations are idempotent
        watcher.cancel();
        assert!(watcher.is_cancelled());
    }

    #[test]
    fn test_request_cancellation_watcher_cross_thread_coordination() {
        let watcher = RequestCancellationWatcher::new();
        let flag_clone = watcher.flag();

        let handle = std::thread::spawn(move || {
            assert!(!flag_clone.load(Ordering::SeqCst));
            // Simulate external cancellation from client abort
            flag_clone.store(true, Ordering::SeqCst);
        });

        handle.join().unwrap();
        assert!(watcher.is_cancelled());
    }
}
