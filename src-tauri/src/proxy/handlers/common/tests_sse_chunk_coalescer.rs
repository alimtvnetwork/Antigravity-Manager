#[cfg(test)]
mod tests {
    use super::super::sse_chunk_coalescer::SseChunkCoalescer;
    use bytes::Bytes;

    #[test]
    fn test_sse_chunk_coalescer_initial_state_and_default() {
        let coalescer = SseChunkCoalescer::new();
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (0, 0));
        assert_eq!(coalescer.finish(), None);

        let default_coalescer = SseChunkCoalescer::default();
        assert_eq!(default_coalescer.buffered_len(), 0);
        assert_eq!(default_coalescer.stats(), (0, 0));
    }

    #[test]
    fn test_sse_chunk_coalescer_single_complete_frame_lf() {
        let mut coalescer = SseChunkCoalescer::new();
        let chunk = b"data: {\"text\":\"hello world\"}\n\n";

        let frames = coalescer.push_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Bytes::from_static(chunk));
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (chunk.len(), 1));
        assert_eq!(coalescer.finish(), None);
    }

    #[test]
    fn test_sse_chunk_coalescer_single_complete_frame_crlf() {
        let mut coalescer = SseChunkCoalescer::new();
        let chunk = b"data: {\"text\":\"crlf message\"}\r\n\r\n";

        let frames = coalescer.push_chunk(chunk);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Bytes::from_static(chunk));
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (chunk.len(), 1));
    }

    #[test]
    fn test_sse_chunk_coalescer_multiple_complete_frames_single_push() {
        let mut coalescer = SseChunkCoalescer::new();
        let chunk = b"data: frame 1\n\nevent: ping\ndata: frame 2\r\n\r\ndata: frame 3\n\n";

        let frames = coalescer.push_chunk(chunk);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], Bytes::from_static(b"data: frame 1\n\n"));
        assert_eq!(frames[1], Bytes::from_static(b"event: ping\ndata: frame 2\r\n\r\n"));
        assert_eq!(frames[2], Bytes::from_static(b"data: frame 3\n\n"));
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (chunk.len(), 3));
    }

    #[test]
    fn test_sse_chunk_coalescer_fragmented_frame_across_multiple_pushes() {
        let mut coalescer = SseChunkCoalescer::new();

        // Push incomplete fragment 1
        let f1 = coalescer.push_chunk(b"data: frag");
        assert!(f1.is_empty());
        assert_eq!(coalescer.buffered_len(), 10);

        // Push incomplete fragment 2
        let f2 = coalescer.push_chunk(b"mented pay");
        assert!(f2.is_empty());
        assert_eq!(coalescer.buffered_len(), 20);

        // Push final fragment completing the event
        let f3 = coalescer.push_chunk(b"load\n\n");
        assert_eq!(f3.len(), 1);
        assert_eq!(f3[0], Bytes::from_static(b"data: fragmented payload\n\n"));
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (26, 1));
    }

    #[test]
    fn test_sse_chunk_coalescer_delimiter_split_across_chunk_boundary() {
        let mut coalescer = SseChunkCoalescer::new();

        // Boundary split along LF: first chunk ends with single '\n'
        let f1 = coalescer.push_chunk(b"data: first part\n");
        assert!(f1.is_empty());
        assert_eq!(coalescer.buffered_len(), 17);

        // Second chunk starts with '\n', completing the first frame, and contains a second frame
        let f2 = coalescer.push_chunk(b"\ndata: second part\n\n");
        assert_eq!(f2.len(), 2);
        assert_eq!(f2[0], Bytes::from_static(b"data: first part\n\n"));
        assert_eq!(f2[1], Bytes::from_static(b"data: second part\n\n"));
        assert_eq!(coalescer.buffered_len(), 0);

        // Boundary split along CRLF: chunk 1 ends in "\r\n\r"
        let f3 = coalescer.push_chunk(b"data: crlf split\r\n\r");
        assert!(f3.is_empty());
        let f4 = coalescer.push_chunk(b"\n");
        assert_eq!(f4.len(), 1);
        assert_eq!(f4[0], Bytes::from_static(b"data: crlf split\r\n\r\n"));
        assert_eq!(coalescer.buffered_len(), 0);
    }

    #[test]
    fn test_sse_chunk_coalescer_finish_with_residual_incomplete_frame() {
        let mut coalescer = SseChunkCoalescer::new();
        let frames = coalescer.push_chunk(b"data: complete\n\ndata: trailing residual");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Bytes::from_static(b"data: complete\n\n"));
        assert_eq!(coalescer.buffered_len(), 24);

        let residual = coalescer.finish();
        assert_eq!(residual, Some(Bytes::from_static(b"data: trailing residual")));
    }

    #[test]
    fn test_sse_chunk_coalescer_empty_push_chunk() {
        let mut coalescer = SseChunkCoalescer::new();
        let frames = coalescer.push_chunk(&[]);
        assert!(frames.is_empty());
        assert_eq!(coalescer.buffered_len(), 0);
        assert_eq!(coalescer.stats(), (0, 0));
    }
}
