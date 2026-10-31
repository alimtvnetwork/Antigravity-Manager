#[cfg(test)]
mod tests {
    use super::super::response_content_type_matcher::{
        classify_content_type, normalize_content_type, CanonicalContentType,
    };
    use axum::http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};

    #[test]
    fn test_classify_content_type_sse() {
        assert_eq!(
            classify_content_type(Some("text/event-stream")),
            CanonicalContentType::ServerSentEvents
        );
        assert_eq!(
            classify_content_type(Some("text/event-stream; charset=utf-8")),
            CanonicalContentType::ServerSentEvents
        );
    }

    #[test]
    fn test_classify_content_type_json() {
        assert_eq!(
            classify_content_type(Some("application/json")),
            CanonicalContentType::Json
        );
        assert_eq!(
            classify_content_type(Some("application/problem+json")),
            CanonicalContentType::Json
        );
        // None falls back to Json
        assert_eq!(classify_content_type(None), CanonicalContentType::Json);
    }

    #[test]
    fn test_classify_content_type_text_and_binary() {
        assert_eq!(
            classify_content_type(Some("text/plain; charset=iso-8859-1")),
            CanonicalContentType::PlainText
        );
        assert_eq!(
            classify_content_type(Some("application/octet-stream")),
            CanonicalContentType::Binary
        );
        assert_eq!(
            classify_content_type(Some("image/png")),
            CanonicalContentType::Binary
        );
        assert_eq!(
            classify_content_type(Some("application/x-custom")),
            CanonicalContentType::Other
        );
    }

    #[test]
    fn test_normalize_content_type_modifies_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));

        normalize_content_type(&mut headers, CanonicalContentType::ServerSentEvents);

        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            "text/event-stream; charset=utf-8"
        );
    }

    #[test]
    fn test_standard_header_values() {
        assert_eq!(
            CanonicalContentType::ServerSentEvents.standard_header_value(),
            "text/event-stream; charset=utf-8"
        );
        assert_eq!(
            CanonicalContentType::Json.standard_header_value(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            CanonicalContentType::PlainText.standard_header_value(),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            CanonicalContentType::Binary.standard_header_value(),
            "application/octet-stream"
        );
    }
}
