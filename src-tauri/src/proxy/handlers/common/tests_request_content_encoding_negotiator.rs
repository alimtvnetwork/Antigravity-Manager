#[cfg(test)]
mod tests {
    use super::super::request_content_encoding_negotiator::{
        is_encoding_supported, negotiate_content_encoding, SupportedEncoding,
    };
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn test_negotiate_gzip() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));

        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Gzip);
        assert!(is_encoding_supported(encoding));
    }

    #[test]
    fn test_negotiate_deflate() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_ENCODING,
            HeaderValue::from_static("deflate"),
        );

        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Deflate);
        assert!(is_encoding_supported(encoding));
    }

    #[test]
    fn test_negotiate_identity() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_ENCODING,
            HeaderValue::from_static("identity"),
        );

        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Identity);
        assert!(is_encoding_supported(encoding));
    }

    #[test]
    fn test_negotiate_missing_header() {
        let headers = HeaderMap::new();
        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Identity);
        assert!(is_encoding_supported(encoding));
    }

    #[test]
    fn test_negotiate_case_insensitivity() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("GZIP"));

        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Gzip);
    }

    #[test]
    fn test_negotiate_unsupported() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("br"));

        let encoding = negotiate_content_encoding(&headers);
        assert_eq!(encoding, SupportedEncoding::Unsupported);
        assert!(!is_encoding_supported(encoding));
    }
}
