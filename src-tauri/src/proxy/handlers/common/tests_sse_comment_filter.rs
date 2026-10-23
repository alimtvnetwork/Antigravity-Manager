#[cfg(test)]
mod tests {
    use super::super::sse_comment_filter::{contains_sse_comment, filter_sse_comments};

    #[test]
    fn test_sse_comment_filter_no_colons_fast_path() {
        let chunk = b"heartbeat plain text chunk without colon";
        assert!(!contains_sse_comment(chunk));
        assert_eq!(filter_sse_comments(chunk), chunk.to_vec());
    }

    #[test]
    fn test_sse_comment_filter_pure_comment_frames() {
        let chunk = b": keepalive telemetry\n: ping\n\n";
        assert!(contains_sse_comment(chunk));

        let filtered = filter_sse_comments(chunk);
        // Comments stripped, trailing empty line preserved
        assert_eq!(filtered, b"\n".to_vec());
    }

    #[test]
    fn test_sse_comment_filter_mixed_comments_and_data_frames() {
        let chunk = b": keepalive\ndata: {\"model\":\"claude\",\"delta\":\"hi\"}\n\n: internal trace\nevent: completion\ndata: {\"done\":true}\n\n";
        assert!(contains_sse_comment(chunk));

        let filtered = filter_sse_comments(chunk);
        let expected = b"data: {\"model\":\"claude\",\"delta\":\"hi\"}\n\nevent: completion\ndata: {\"done\":true}\n\n";
        assert_eq!(filtered, expected.to_vec());
    }

    #[test]
    fn test_sse_comment_filter_preserves_colons_inside_data_payload() {
        let chunk = b"data: {\"url\": \"https://example.com:8080/api\", \"time\": \"12:30:00\"}\n\n";
        assert!(!contains_sse_comment(chunk));

        let filtered = filter_sse_comments(chunk);
        assert_eq!(filtered, chunk.to_vec());
    }

    #[test]
    fn test_sse_comment_filter_whitespace_padded_comments() {
        let chunk = b"   : space-prefixed comment\n\t: tab-prefixed comment\ndata: payload\n\n";
        assert!(contains_sse_comment(chunk));

        let filtered = filter_sse_comments(chunk);
        assert_eq!(filtered, b"data: payload\n\n".to_vec());
    }

    #[test]
    fn test_sse_comment_filter_crlf_line_endings() {
        let chunk = b": comment line\r\ndata: active stream\r\n\r\n";
        assert!(contains_sse_comment(chunk));

        let filtered = filter_sse_comments(chunk);
        // Strips comment line, retains CRLF for data frame
        assert_eq!(filtered, b"data: active stream\r\n\r\n".to_vec());
    }

    #[test]
    fn test_sse_comment_filter_invalid_utf8_fallback() {
        let invalid_utf8_with_colon = vec![0xFF, 0xFE, b':', 0x80, b'a'];
        assert!(!contains_sse_comment(&invalid_utf8_with_colon));

        let filtered = filter_sse_comments(&invalid_utf8_with_colon);
        assert_eq!(filtered, invalid_utf8_with_colon);
    }

    #[test]
    fn test_sse_comment_filter_empty_payload() {
        let empty = b"";
        assert!(!contains_sse_comment(empty));
        assert_eq!(filter_sse_comments(empty), Vec::<u8>::new());
    }
}
