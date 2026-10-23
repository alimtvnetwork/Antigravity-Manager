// Unit Test Suite for SSE Stream Splitter & Preamble Padder (Imp-511 & Imp-481)
// Validates chunk frame splitting, buffer boundaries, remainder flushing,
// and preamble padding generation.

#[cfg(test)]
mod tests {
    use super::super::sse_stream_padder::{
        generate_sse_preamble_padding, DEFAULT_PREAMBLE_PADDING_BYTES,
    };
    use super::super::sse_stream_splitter::SseStreamSplitter;
    use bytes::Bytes;

    #[test]
    fn test_splitter_initialization_and_default() {
        let mut splitter_new = SseStreamSplitter::new();
        let mut splitter_default = SseStreamSplitter::default();

        assert_eq!(splitter_new.buffered_bytes_count(), 0);
        assert_eq!(splitter_default.buffered_bytes_count(), 0);
        assert_eq!(splitter_new.flush_remainder(), None);
        assert_eq!(splitter_default.flush_remainder(), None);
    }

    #[test]
    fn test_feed_single_complete_sse_frame() {
        let mut splitter = SseStreamSplitter::new();
        let chunk = b"data: hello world\n\n";

        let frames = splitter.feed(chunk);

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Bytes::from_static(b"data: hello world\n\n"));
        assert_eq!(splitter.buffered_bytes_count(), 0);
        assert_eq!(splitter.flush_remainder(), None);
    }

    #[test]
    fn test_feed_multiple_frames_in_single_chunk() {
        let mut splitter = SseStreamSplitter::new();
        let chunk = b"event: update\ndata: 100\n\nevent: ping\ndata: pong\n\n";

        let frames = splitter.feed(chunk);

        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames[0],
            Bytes::from_static(b"event: update\ndata: 100\n\n")
        );
        assert_eq!(
            frames[1],
            Bytes::from_static(b"event: ping\ndata: pong\n\n")
        );
        assert_eq!(splitter.buffered_bytes_count(), 0);
    }

    #[test]
    fn test_feed_frame_split_across_consecutive_chunks() {
        let mut splitter = SseStreamSplitter::new();

        let f1 = splitter.feed(b"data: first part ");
        assert!(f1.is_empty());
        assert_eq!(splitter.buffered_bytes_count(), 17);

        let f2 = splitter.feed(b"second part\n\n");
        assert_eq!(f2.len(), 1);
        assert_eq!(
            f2[0],
            Bytes::from_static(b"data: first part second part\n\n")
        );
        assert_eq!(splitter.buffered_bytes_count(), 0);
    }

    #[test]
    fn test_feed_split_at_exact_double_newline_delimiter() {
        let mut splitter = SseStreamSplitter::new();

        let f1 = splitter.feed(b"data: boundary\n");
        assert!(f1.is_empty());
        assert_eq!(splitter.buffered_bytes_count(), 15);

        let f2 = splitter.feed(b"\n");
        assert_eq!(f2.len(), 1);
        assert_eq!(f2[0], Bytes::from_static(b"data: boundary\n\n"));
        assert_eq!(splitter.buffered_bytes_count(), 0);
    }

    #[test]
    fn test_flush_remainder_with_trailing_incomplete_data() {
        let mut splitter = SseStreamSplitter::new();
        let chunk = b"data: complete\n\ndata: incomplete_tail";

        let frames = splitter.feed(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Bytes::from_static(b"data: complete\n\n"));
        assert_eq!(splitter.buffered_bytes_count(), 22);

        let remainder = splitter.flush_remainder();
        assert_eq!(
            remainder,
            Some(Bytes::from_static(b"data: incomplete_tail"))
        );
        assert_eq!(splitter.buffered_bytes_count(), 0);
        assert_eq!(splitter.flush_remainder(), None);
    }

    #[test]
    fn test_feed_consecutive_empty_frames() {
        let mut splitter = SseStreamSplitter::new();
        let frames = splitter.feed(b"\n\n\n\n");

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], Bytes::from_static(b"\n\n"));
        assert_eq!(frames[1], Bytes::from_static(b"\n\n"));
        assert_eq!(splitter.buffered_bytes_count(), 0);
    }

    #[test]
    fn test_generate_default_preamble_padding() {
        let padding = generate_sse_preamble_padding(None);

        assert!(
            padding.len() >= DEFAULT_PREAMBLE_PADDING_BYTES,
            "Padding must exceed default preamble size (got {} bytes)",
            padding.len()
        );
        assert!(padding.starts_with(b": open-stream-preamble-padding "));
        assert!(padding.ends_with(b"\n\n"));

        let padding_str = std::str::from_utf8(&padding).expect("Padding must be valid UTF-8");
        assert!(padding_str.starts_with(": open-stream-preamble-padding "));
    }

    #[test]
    fn test_generate_custom_target_padding() {
        let target = 4096;
        let padding = generate_sse_preamble_padding(Some(target));

        assert!(padding.len() >= target);
        assert!(padding.starts_with(b": open-stream-preamble-padding "));
        assert!(padding.ends_with(b"\n\n"));
    }

    #[test]
    fn test_generate_padding_clamps_to_minimum_64_bytes() {
        let padding_small = generate_sse_preamble_padding(Some(10));
        let padding_zero = generate_sse_preamble_padding(Some(0));

        assert!(padding_small.len() >= 66);
        assert!(padding_zero.len() >= 66);
        assert!(padding_small.starts_with(b": open-stream-preamble-padding "));
        assert!(padding_small.ends_with(b"\n\n"));
    }
}
