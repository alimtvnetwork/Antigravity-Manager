//! Unit Tests for Streaming Event Frame Parser (`Imp-690`).
//!
//! Validates W3C SSE compliance, multi-line data concatenation, CRLF normalization,
//! comment parsing, byte-fragmented buffering, trailing partial frame flushing,
//! and metric tracking.

#[cfg(test)]
mod tests {
    use super::super::streaming_event_frame_parser::{ParsedEventFrame, StreamingFrameParser};

    #[test]
    fn test_standard_single_line_sse_frame() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"data: {\"type\":\"delta\",\"text\":\"hello\"}\n\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        let frame = &frames[0];
        assert_eq!(frame.data, "{\"type\":\"delta\",\"text\":\"hello\"}");
        assert_eq!(frame.event_type, None);
        assert_eq!(frame.id, None);
        assert_eq!(frame.retry_ms, None);
        assert!(!frame.is_comment);
        assert_eq!(parser.total_frames_parsed(), 1);
        assert_eq!(parser.total_bytes_ingested(), payload.len());
        assert_eq!(parser.buffer_len(), 0);
    }

    #[test]
    fn test_multi_line_data_concatenation() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"data: Line 1\ndata: Line 2\ndata: Line 3\n\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, "Line 1\nLine 2\nLine 3");
        assert!(!frames[0].is_comment);
    }

    #[test]
    fn test_full_sse_frame_with_all_fields() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"event: message_delta\nid: evt-99182\nretry: 5000\ndata: token_abc\n\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        let frame = &frames[0];
        assert_eq!(frame.event_type.as_deref(), Some("message_delta"));
        assert_eq!(frame.id.as_deref(), Some("evt-99182"));
        assert_eq!(frame.retry_ms, Some(5000));
        assert_eq!(frame.data, "token_abc");
        assert!(!frame.is_comment);
    }

    #[test]
    fn test_single_and_multiline_sse_comments() {
        let mut parser = StreamingFrameParser::new();
        let comment_payload = b": ping heartbeat\n\n";

        let frames = parser.push_chunk(comment_payload);
        assert_eq!(frames.len(), 1);
        assert!(frames[0].is_comment);
        assert_eq!(frames[0].data, "ping heartbeat");

        let multi_comment = b": status 200\n: keep-alive\n\n";
        let frames2 = parser.push_chunk(multi_comment);
        assert_eq!(frames2.len(), 1);
        assert!(frames2[0].is_comment);
        assert_eq!(frames2[0].data, "status 200\nkeep-alive");
    }

    #[test]
    fn test_comment_with_data_preserves_data() {
        let mut parser = StreamingFrameParser::new();
        let mixed_payload = b": internal comment\ndata: actual payload\n\n";

        let frames = parser.push_chunk(mixed_payload);
        assert_eq!(frames.len(), 1);
        assert!(!frames[0].is_comment);
        assert_eq!(frames[0].data, "actual payload");
    }

    #[test]
    fn test_crlf_windows_line_endings() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"event: update\r\ndata: win32_test\r\nid: 1\r\n\r\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].event_type.as_deref(), Some("update"));
        assert_eq!(frames[0].data, "win32_test");
        assert_eq!(frames[0].id.as_deref(), Some("1"));
    }

    #[test]
    fn test_legacy_cr_line_endings() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"event: mac_classic\rdata: cr_only\r\r";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].event_type.as_deref(), Some("mac_classic"));
        assert_eq!(frames[0].data, "cr_only");
    }

    #[test]
    fn test_single_byte_fragmented_streaming() {
        let mut parser = StreamingFrameParser::new();
        let stream = b"data: fragmented token\n\n";

        let mut collected = Vec::new();
        for &byte in stream.iter() {
            let frames = parser.push_chunk(&[byte]);
            collected.extend(frames);
        }

        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].data, "fragmented token");
        assert_eq!(parser.total_bytes_ingested(), stream.len());
        assert_eq!(parser.total_frames_parsed(), 1);
        assert_eq!(parser.buffer_len(), 0);
    }

    #[test]
    fn test_multiple_frames_in_single_chunk() {
        let mut parser = StreamingFrameParser::new();
        let chunk = b"data: frame_1\n\nevent: custom\ndata: frame_2\n\ndata: frame_3\n\n";

        let frames = parser.push_chunk(chunk);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].data, "frame_1");
        assert_eq!(frames[1].event_type.as_deref(), Some("custom"));
        assert_eq!(frames[1].data, "frame_2");
        assert_eq!(frames[2].data, "frame_3");
        assert_eq!(parser.total_frames_parsed(), 3);
    }

    #[test]
    fn test_trailing_partial_frame_without_newline_flushed_by_finish() {
        let mut parser = StreamingFrameParser::new();
        let chunk = b"data: unclosed trailing data";

        let frames = parser.push_chunk(chunk);
        assert!(frames.is_empty());
        assert!(parser.has_unprocessed_bytes());
        assert_eq!(parser.incomplete_line_str().as_deref(), Some("data: unclosed trailing data"));

        let final_frames = parser.finish();
        assert_eq!(final_frames.len(), 1);
        assert_eq!(final_frames[0].data, "unclosed trailing data");
        assert_eq!(parser.buffer_len(), 0);
    }

    #[test]
    fn test_trailing_partial_frame_with_single_newline_flushed_by_finish() {
        let mut parser = StreamingFrameParser::new();
        let chunk = b"data: single trailing line\n";

        let frames = parser.push_chunk(chunk);
        assert!(frames.is_empty());
        assert!(parser.has_pending_frame());

        let final_frames = parser.finish();
        assert_eq!(final_frames.len(), 1);
        assert_eq!(final_frames[0].data, "single trailing line");
    }

    #[test]
    fn test_consecutive_empty_lines_produce_no_phantom_frames() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"\n\n\n\ndata: single\n\n\n\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, "single");
    }

    #[test]
    fn test_id_with_null_byte_ignored() {
        let mut parser = StreamingFrameParser::new();
        let payload = b"id: bad\0id\ndata: safe\n\n";

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].id, None);
        assert_eq!(frames[0].data, "safe");
    }

    #[test]
    fn test_reset_clears_state_and_metrics() {
        let mut parser = StreamingFrameParser::new();
        parser.push_chunk(b"data: first\n\n");
        assert_eq!(parser.total_frames_parsed(), 1);
        assert!(parser.total_bytes_ingested() > 0);

        parser.push_chunk(b"data: incomplete");
        assert!(parser.has_unprocessed_bytes());

        parser.reset();
        assert_eq!(parser.total_frames_parsed(), 0);
        assert_eq!(parser.total_bytes_ingested(), 0);
        assert_eq!(parser.buffer_len(), 0);
        assert!(!parser.has_pending_frame());
    }

    #[test]
    fn test_to_sse_string_roundtrip() {
        let frame = ParsedEventFrame::with_data("hello world")
            .with_event_type("chat")
            .with_id("100")
            .with_retry(3000);

        let sse_str = frame.to_sse_string();
        assert!(sse_str.contains("event: chat\n"));
        assert!(sse_str.contains("data: hello world\n"));
        assert!(sse_str.contains("id: 100\n"));
        assert!(sse_str.contains("retry: 3000\n"));
        assert!(sse_str.ends_with("\n\n"));

        let mut parser = StreamingFrameParser::new();
        let parsed = parser.push_chunk(sse_str.as_bytes());
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], frame);
    }

    #[test]
    fn test_utf8_multibyte_characters() {
        let mut parser = StreamingFrameParser::new();
        let payload = "data: 🚀 Gemini 闪电推理模型\n\n".as_bytes();

        let frames = parser.push_chunk(payload);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, "🚀 Gemini 闪电推理模型");
    }
}
