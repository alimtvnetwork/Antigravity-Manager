#[cfg(test)]
mod tests {
    use super::super::compression_negotiator::{
        CompressionNegotiator, SelectedEncoding,
    };

    #[test]
    fn test_negotiate_sse_streaming_prefers_identity() {
        let decision = CompressionNegotiator::negotiate(Some("gzip, br"), true, Some(10000));
        assert_eq!(decision.encoding, SelectedEncoding::Identity);
        assert!(!decision.should_compress);
        assert!(decision.reason.contains("SSE streaming"));
    }

    #[test]
    fn test_negotiate_below_min_size_threshold() {
        let decision = CompressionNegotiator::negotiate(Some("br, gzip"), false, Some(512));
        assert_eq!(decision.encoding, SelectedEncoding::Identity);
        assert!(!decision.should_compress);
        assert!(decision.reason.contains("below compression threshold"));
    }

    #[test]
    fn test_negotiate_no_accept_encoding() {
        let decision_none = CompressionNegotiator::negotiate(None, false, Some(5000));
        assert_eq!(decision_none.encoding, SelectedEncoding::Identity);
        assert!(!decision_none.should_compress);

        let decision_empty = CompressionNegotiator::negotiate(Some("   "), false, Some(5000));
        assert_eq!(decision_empty.encoding, SelectedEncoding::Identity);
        assert!(!decision_empty.should_compress);
    }

    #[test]
    fn test_negotiate_brotli_priority_over_gzip() {
        let decision = CompressionNegotiator::negotiate(Some("gzip, deflate, br"), false, Some(10000));
        assert_eq!(decision.encoding, SelectedEncoding::Brotli);
        assert!(decision.should_compress);
        assert_eq!(decision.encoding.as_header_str(), Some("br"));
    }

    #[test]
    fn test_negotiate_gzip_selection() {
        let decision = CompressionNegotiator::negotiate(Some("gzip, deflate"), false, Some(10000));
        assert_eq!(decision.encoding, SelectedEncoding::Gzip);
        assert!(decision.should_compress);
        assert_eq!(decision.encoding.as_header_str(), Some("gzip"));
    }

    #[test]
    fn test_negotiate_deflate_selection() {
        let decision = CompressionNegotiator::negotiate(Some("deflate"), false, Some(10000));
        assert_eq!(decision.encoding, SelectedEncoding::Deflate);
        assert!(decision.should_compress);
        assert_eq!(decision.encoding.as_header_str(), Some("deflate"));
    }

    #[test]
    fn test_selected_encoding_header_str() {
        assert_eq!(SelectedEncoding::Identity.as_header_str(), None);
        assert_eq!(SelectedEncoding::Gzip.as_header_str(), Some("gzip"));
        assert_eq!(SelectedEncoding::Brotli.as_header_str(), Some("br"));
        assert_eq!(SelectedEncoding::Deflate.as_header_str(), Some("deflate"));
    }
}
