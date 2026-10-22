// Unit Test Suite for SSE Delimiter Normalizer (Imp-541)
// Validates CRLF to LF normalization, multi-chunk feeding, partial chunk buffering,
// delimiter-preserving event emission, and buffer flushing.

#[cfg(test)]
mod tests {
    use crate::proxy::handlers::common::sse_delimiter_normalizer::{
        normalize_sse_payload, SseDelimiterNormalizer,
    };

    #[test]
    fn test_sse_delimiter_normalizer_complete_events_single_chunk() {
        let mut normalizer = SseDelimiterNormalizer::new();
        let chunk = b"data: {\"type\":\"message_start\"}\n\ndata: {\"type\":\"content_block_start\"}\n\n";

        let events = normalizer.feed(chunk);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], "data: {\"type\":\"message_start\"}\n\n");
        assert_eq!(events[1], "data: {\"type\":\"content_block_start\"}\n\n");
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.flush(), None);
    }

    #[test]
    fn test_sse_delimiter_normalizer_crlf_and_cr_mixed_delimiters() {
        let mut normalizer = SseDelimiterNormalizer::new();
        let chunk = b"data: event_one\r\n\r\ndata: event_two\r\rdata: event_three\n\n";

        let events = normalizer.feed(chunk);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], "data: event_one\n\n");
        assert_eq!(events[1], "data: event_two\n\n");
        assert_eq!(events[2], "data: event_three\n\n");
        assert!(!normalizer.has_pending_bytes());
    }

    #[test]
    fn test_sse_delimiter_normalizer_fragmented_stream_across_three_chunks() {
        let mut normalizer = SseDelimiterNormalizer::new();

        // Chunk 1: Incomplete start
        let chunk1 = b"data: {\"text\": \"hello";
        let ev1 = normalizer.feed(chunk1);
        assert!(ev1.is_empty());
        assert!(normalizer.has_pending_bytes());

        // Chunk 2: Completes first event and starts second event
        let chunk2 = b" world\"}\n\ndata: {\"text\": \"second";
        let ev2 = normalizer.feed(chunk2);
        assert_eq!(ev2.len(), 1);
        assert_eq!(ev2[0], "data: {\"text\": \"hello world\"}\n\n");
        assert!(normalizer.has_pending_bytes());

        // Chunk 3: Completes second event
        let chunk3 = b" part\"}\n\n";
        let ev3 = normalizer.feed(chunk3);
        assert_eq!(ev3.len(), 1);
        assert_eq!(ev3[0], "data: {\"text\": \"second part\"}\n\n");
        assert!(!normalizer.has_pending_bytes());
    }

    #[test]
    fn test_sse_delimiter_normalizer_flush_with_unclosed_trailing_data() {
        let mut normalizer = SseDelimiterNormalizer::new();
        let chunk = b"data: {\"done\": true}";
        let events = normalizer.feed(chunk);
        assert!(events.is_empty());
        assert!(normalizer.has_pending_bytes());

        let flushed = normalizer.flush();
        assert_eq!(flushed, Some("data: {\"done\": true}\n\n".to_string()));
        assert!(!normalizer.has_pending_bytes());

        // Second flush returns None
        assert_eq!(normalizer.flush(), None);
    }

    #[test]
    fn test_normalize_sse_payload_helper() {
        let raw = "data: {\"a\":1}\r\n\r\ndata: {\"b\":2}\r\r\r\ndata: [DONE]\n\n";
        let normalized = normalize_sse_payload(raw);
        assert!(normalized.contains("data: {\"a\":1}\n\n"));
        assert!(normalized.contains("data: {\"b\":2}\n\n"));
        assert!(normalized.contains("data: [DONE]\n\n"));
    }
}
