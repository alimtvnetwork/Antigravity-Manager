#[cfg(test)]
mod tests {
    use super::super::stream_early_abort_sentinel::StreamEarlyAbortSentinel;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_stream_early_abort_sentinel_initial_state() {
        let sentinel = StreamEarlyAbortSentinel::default();
        assert!(!sentinel.is_aborted());
        assert!(!sentinel.cancellation_token().is_cancelled());
    }

    #[test]
    fn test_stream_early_abort_sentinel_trigger_abort_idempotency() {
        let sentinel = StreamEarlyAbortSentinel::new();
        assert!(!sentinel.is_aborted());

        sentinel.trigger_abort();
        assert!(sentinel.is_aborted());
        assert!(sentinel.cancellation_token().is_cancelled());

        // Repeated abort triggers must be strictly idempotent
        sentinel.trigger_abort();
        assert!(sentinel.is_aborted());
        assert!(sentinel.cancellation_token().is_cancelled());
    }

    #[test]
    fn test_stream_early_abort_sentinel_direct_token_cancellation() {
        let sentinel = StreamEarlyAbortSentinel::new();
        let token = sentinel.cancellation_token();

        assert!(!sentinel.is_aborted());
        // Cancelling the token directly without calling trigger_abort
        token.cancel();

        // Dual-path check must evaluate is_aborted() to true
        assert!(sentinel.is_aborted());
    }

    #[test]
    fn test_stream_early_abort_sentinel_child_token_derivation() {
        let sentinel = StreamEarlyAbortSentinel::new();
        let child_token = sentinel.cancellation_token().child_token();

        assert!(!child_token.is_cancelled());
        sentinel.trigger_abort();

        // Child token must be cancelled automatically
        assert!(child_token.is_cancelled());
        assert!(sentinel.is_aborted());
    }

    #[test]
    fn test_stream_early_abort_sentinel_cross_thread_coordination() {
        let sentinel = StreamEarlyAbortSentinel::new();
        let sentinel_clone = sentinel.clone();

        let handle = thread::spawn(move || {
            assert!(!sentinel_clone.is_aborted());
            thread::sleep(Duration::from_millis(15));
            sentinel_clone.trigger_abort();
            assert!(sentinel_clone.is_aborted());
        });

        handle.join().expect("Worker thread succeeded");
        // SeqCst visibility ensures main thread sees abort immediately
        assert!(sentinel.is_aborted());
        assert!(sentinel.cancellation_token().is_cancelled());
    }

    #[tokio::test]
    async fn test_stream_early_abort_sentinel_tokio_select_abort() {
        let sentinel = StreamEarlyAbortSentinel::new();
        let token = sentinel.cancellation_token();

        let sentinel_trigger = sentinel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            sentinel_trigger.trigger_abort();
        });

        // Verify tokio::select responds to sentinel cancellation
        tokio::select! {
            _ = token.cancelled() => {
                assert!(sentinel.is_aborted());
            }
            _ = tokio::time::sleep(Duration::from_millis(1000)) => {
                panic!("Timeout: cancellation was not received in time");
            }
        }
    }
}
