#[cfg(test)]
mod tests {
    use super::super::stream_flusher::{
        StreamFlusher, DEFAULT_SSE_KEEPALIVE_INTERVAL, SSE_KEEPALIVE_BYTES,
    };
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_stream_flusher_default_initialization() {
        let flusher = StreamFlusher::default();
        // Immediately after creation, keepalive should not be due
        assert!(!flusher.should_emit_keepalive());
        assert_eq!(DEFAULT_SSE_KEEPALIVE_INTERVAL, Duration::from_secs(15));
        assert_eq!(SSE_KEEPALIVE_BYTES, b": keepalive\n\n");
    }

    #[test]
    fn test_stream_flusher_custom_interval_trigger() {
        // Fast interval for deterministic testing
        let mut flusher = StreamFlusher::new(Some(Duration::from_millis(20)));
        assert!(!flusher.should_emit_keepalive());

        // Wait beyond interval
        sleep(Duration::from_millis(30));
        assert!(flusher.should_emit_keepalive());

        // Consuming keepalive frame resets the timer
        let frame = flusher.get_keepalive_frame();
        assert_eq!(frame, b": keepalive\n\n");
        assert!(!flusher.should_emit_keepalive());
    }

    #[test]
    fn test_stream_flusher_record_data_sent_resets_idle_timer() {
        let mut flusher = StreamFlusher::new(Some(Duration::from_millis(30)));

        sleep(Duration::from_millis(15));
        assert!(!flusher.should_emit_keepalive());

        // Data activity occurs, resetting the clock
        flusher.record_data_sent();

        // Sleeping another 20ms would exceed 30ms total, but only 20ms since record_data_sent
        sleep(Duration::from_millis(20));
        assert!(!flusher.should_emit_keepalive());

        // Wait another 15ms (total 35ms since record_data_sent)
        sleep(Duration::from_millis(15));
        assert!(flusher.should_emit_keepalive());
    }

    #[test]
    fn test_stream_flusher_zero_duration_boundary() {
        // Boundary case: ZERO duration interval must always emit immediately
        let flusher = StreamFlusher::new(Some(Duration::ZERO));
        assert!(flusher.should_emit_keepalive());
    }

    #[test]
    fn test_stream_flusher_large_duration_boundary() {
        // Boundary case: 1 hour duration never emits prematurely
        let flusher = StreamFlusher::new(Some(Duration::from_secs(3600)));
        assert!(!flusher.should_emit_keepalive());
    }

    #[test]
    fn test_stream_flusher_consecutive_keepalive_emission_cycles() {
        let mut flusher = StreamFlusher::new(Some(Duration::from_millis(15)));

        for _ in 0..3 {
            sleep(Duration::from_millis(20));
            assert!(flusher.should_emit_keepalive());
            let frame = flusher.get_keepalive_frame();
            assert_eq!(frame, SSE_KEEPALIVE_BYTES);
            assert!(!flusher.should_emit_keepalive());
        }
    }

    #[test]
    fn test_stream_flusher_sse_protocol_syntax_validity() {
        let frame_str = std::str::from_utf8(SSE_KEEPALIVE_BYTES).expect("Valid UTF-8");
        // SSE comments must begin with ':' and end with double newline "\n\n"
        assert!(frame_str.starts_with(':'));
        assert!(frame_str.ends_with("\n\n"));
        assert_eq!(frame_str, ": keepalive\n\n");
    }
}
