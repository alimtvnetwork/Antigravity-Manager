//! Unit tests for Multi-Protocol Response Stream Delimiter Normalizer (Imp-633).

#[cfg(test)]
mod tests {
    use crate::proxy::handlers::common::response_stream_delimiter_normalizer::{
        StreamDelimiterNormalizer, StreamFrame,
    };

    #[test]
    fn test_standard_single_frame_chunk_parsing() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello world\"}}]}\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames[0].data_payload,
            "{\"choices\":[{\"delta\":{\"content\":\"Hello world\"}}]}"
        );
        assert_eq!(frames[0].event_type, None);
        assert!(!frames[0].is_terminal);
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.frames_normalized_count(), 1);
    }

    #[test]
    fn test_windows_crlf_delimiter_parsing() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: {\"id\":\"chat-123\",\"content\":\"CRLF test\"}\r\n\r\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames[0].data_payload,
            "{\"id\":\"chat-123\",\"content\":\"CRLF test\"}"
        );
        assert_eq!(frames[0].event_type, None);
        assert!(!frames[0].is_terminal);
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.frames_normalized_count(), 1);
    }

    #[test]
    fn test_split_chunk_parsing_across_multiple_feeds() {
        let mut normalizer = StreamDelimiterNormalizer::new();

        // Feed partial chunk 1
        let chunk1 = b"data: {\"par";
        let frames1 = normalizer.feed_chunk(chunk1);
        assert!(frames1.is_empty());
        assert!(normalizer.has_pending_bytes());
        assert_eq!(normalizer.frames_normalized_count(), 0);

        // Feed partial chunk 2 completing the frame
        let chunk2 = b"t\": 1}\n\n";
        let frames2 = normalizer.feed_chunk(chunk2);
        assert_eq!(frames2.len(), 1);
        assert_eq!(frames2[0].data_payload, "{\"part\": 1}");
        assert!(!frames2[0].is_terminal);
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.frames_normalized_count(), 1);
    }

    #[test]
    fn test_multiple_sse_frames_in_single_chunk() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: {\"seq\": 1}\n\ndata: {\"seq\": 2}\n\ndata: {\"seq\": 3}\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].data_payload, "{\"seq\": 1}");
        assert_eq!(frames[1].data_payload, "{\"seq\": 2}");
        assert_eq!(frames[2].data_payload, "{\"seq\": 3}");
        assert_eq!(normalizer.frames_normalized_count(), 3);
        assert!(!normalizer.has_pending_bytes());
    }

    #[test]
    fn test_terminal_done_sentinel_detection() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: [DONE]\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data_payload, "[DONE]");
        assert!(frames[0].is_terminal);
        assert_eq!(frames[0].event_type, None);
    }

    #[test]
    fn test_event_type_and_terminal_event() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"event: delta\ndata: {\"text\":\"hi\"}\n\nevent: done\ndata: {}\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 2);

        assert_eq!(frames[0].event_type(), Some("delta"));
        assert!(frames[0].has_event_type());
        assert_eq!(frames[0].data_payload(), "{\"text\":\"hi\"}");
        assert!(!frames[0].is_terminal());

        assert_eq!(frames[1].event_type(), Some("done"));
        assert!(frames[1].is_terminal());
    }

    #[test]
    fn test_multiline_data_and_comment_filtering() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b": keepalive ping\ndata: line 1\ndata: line 2\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data_payload, "line 1\nline 2");
        assert_eq!(frames[0].event_type, None);
    }

    #[test]
    fn test_flush_remaining_unclosed_fragment() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: {\"unclosed\": true}";

        let frames = normalizer.feed_chunk(chunk);
        assert!(frames.is_empty());
        assert!(normalizer.has_pending_bytes());

        let flushed = normalizer.flush_remaining();
        assert!(flushed.is_some());
        let frame = flushed.unwrap();
        assert_eq!(frame.data_payload, "{\"unclosed\": true}");
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.frames_normalized_count(), 1);
    }

    #[test]
    fn test_reset_functionality() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        normalizer.feed_chunk(b"data: first\n\ndata: partial");
        assert_eq!(normalizer.frames_normalized_count(), 1);
        assert!(normalizer.has_pending_bytes());

        normalizer.reset();
        assert_eq!(normalizer.frames_normalized_count(), 0);
        assert!(!normalizer.has_pending_bytes());
        assert_eq!(normalizer.buffer_len(), 0);
    }

    #[test]
    fn test_malformed_trailing_delimiters_stripped() {
        let mut normalizer = StreamDelimiterNormalizer::new();
        let chunk = b"data: clean\r\n\r\n\n\n\r\n\n";

        let frames = normalizer.feed_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data_payload, "clean");
        assert_eq!(normalizer.flush_remaining(), None);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original_frame = StreamFrame::new(
            Some("message".to_string()),
            "{\"content\":\"test\"}".to_string(),
            35,
            false,
        );

        let json = serde_json::to_string(&original_frame).expect("Serialize frame");
        let deserialized: StreamFrame = serde_json::from_str(&json).expect("Deserialize frame");
        assert_eq!(original_frame, deserialized);

        let mut original_normalizer = StreamDelimiterNormalizer::new();
        original_normalizer.feed_chunk(b"data: item\n\n");
        let norm_json = serde_json::to_string(&original_normalizer).expect("Serialize norm");
        let norm_deserialized: StreamDelimiterNormalizer =
            serde_json::from_str(&norm_json).expect("Deserialize norm");
        assert_eq!(original_normalizer.buffer, norm_deserialized.buffer);
        assert_eq!(
            original_normalizer.frames_normalized_count,
            norm_deserialized.frames_normalized_count
        );
    }
}
