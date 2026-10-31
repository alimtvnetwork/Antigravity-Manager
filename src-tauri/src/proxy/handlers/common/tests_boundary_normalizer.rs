#[cfg(test)]
mod tests {
    use super::super::boundary_normalizer::StreamBoundaryNormalizer;

    #[test]
    fn test_boundary_normalizer_single_complete_chunk() {
        let mut normalizer = StreamBoundaryNormalizer::new();
        let events = normalizer.feed_chunk("data: {\"text\":\"hello\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "data: {\"text\":\"hello\"}");
        assert_eq!(normalizer.flush_remaining(), None);
    }

    #[test]
    fn test_boundary_normalizer_fragmented_across_chunks() {
        let mut normalizer = StreamBoundaryNormalizer::new();

        let events1 = normalizer.feed_chunk("data: {\"response\":");
        assert!(events1.is_empty());

        let events2 = normalizer.feed_chunk(" \"streamed_text\"}\n\n");
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0], "data: {\"response\": \"streamed_text\"}");
    }

    #[test]
    fn test_boundary_normalizer_multiple_events_in_single_chunk() {
        let mut normalizer = StreamBoundaryNormalizer::new();
        let chunk = "event: message\ndata: 1\n\nevent: message\ndata: 2\n\n";
        let events = normalizer.feed_chunk(chunk);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], "event: message\ndata: 1");
        assert_eq!(events[1], "event: message\ndata: 2");
    }

    #[test]
    fn test_boundary_normalizer_flush_trailing() {
        let mut normalizer = StreamBoundaryNormalizer::new();
        normalizer.feed_chunk("data: unfinished_event");

        let flushed = normalizer.flush_remaining();
        assert_eq!(flushed.as_deref(), Some("data: unfinished_event"));

        // Buffer should be empty on subsequent flush
        assert_eq!(normalizer.flush_remaining(), None);
    }
}
