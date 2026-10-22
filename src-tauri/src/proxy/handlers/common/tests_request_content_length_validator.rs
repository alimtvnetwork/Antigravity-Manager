#[cfg(test)]
mod tests {
    use super::super::request_content_length_validator::{
        validate_content_length, ContentLengthValidation, DEFAULT_MAX_REQUEST_BODY_BYTES,
    };
    use axum::http::{header::CONTENT_LENGTH, HeaderMap, HeaderValue};

    #[test]
    fn test_validate_content_length_absent() {
        let headers = HeaderMap::new();
        let result = validate_content_length(&headers, None);
        assert_eq!(result, ContentLengthValidation::ChunkedOrAbsent);
    }

    #[test]
    fn test_validate_content_length_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("4096"));

        let result = validate_content_length(&headers, None);
        assert_eq!(result, ContentLengthValidation::Valid(4096));
    }

    #[test]
    fn test_validate_content_length_exceeds_default_limit() {
        let mut headers = HeaderMap::new();
        let oversize = DEFAULT_MAX_REQUEST_BODY_BYTES + 1024;
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&oversize.to_string()).unwrap());

        let result = validate_content_length(&headers, None);
        assert_eq!(
            result,
            ContentLengthValidation::ExceedsLimit {
                declared: oversize,
                limit: DEFAULT_MAX_REQUEST_BODY_BYTES,
            }
        );
    }

    #[test]
    fn test_validate_content_length_custom_limit() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("1000"));

        // When custom limit is 2000, 1000 is valid
        let ok_result = validate_content_length(&headers, Some(2000));
        assert_eq!(ok_result, ContentLengthValidation::Valid(1000));

        // When custom limit is 500, 1000 exceeds limit
        let exceed_result = validate_content_length(&headers, Some(500));
        assert_eq!(
            exceed_result,
            ContentLengthValidation::ExceedsLimit {
                declared: 1000,
                limit: 500,
            }
        );
    }

    #[test]
    fn test_validate_content_length_malformed() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("invalid_number"));

        let result = validate_content_length(&headers, None);
        match result {
            ContentLengthValidation::Malformed(msg) => {
                assert!(msg.contains("invalid_number"));
            }
            other => panic!("Expected Malformed, got {:?}", other),
        }
    }
}
