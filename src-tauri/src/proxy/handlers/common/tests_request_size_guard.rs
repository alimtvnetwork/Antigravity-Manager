//! Unit tests for RequestSizeGuard (Imp-569)

use super::request_size_guard::{
    evaluate_request_size, is_multimodal_model_name, SizeGuardOutcome,
    DEFAULT_MULTIMODAL_PAYLOAD_CEILING_BYTES, DEFAULT_TEXT_PAYLOAD_CEILING_BYTES,
};

#[test]
fn test_evaluate_request_size_under_text_ceiling_accepted() {
    let payload_size = 5 * 1024 * 1024; // 5 MB
    let result = evaluate_request_size(Some(payload_size), payload_size, false);
    assert_eq!(result, SizeGuardOutcome::Accepted);
}

#[test]
fn test_evaluate_request_size_over_text_ceiling_rejected() {
    let payload_size = 12 * 1024 * 1024; // 12 MB > 10 MB ceiling
    let result = evaluate_request_size(Some(payload_size), payload_size, false);
    assert_eq!(
        result,
        SizeGuardOutcome::RejectedTooLarge {
            actual_bytes: payload_size,
            ceiling_bytes: DEFAULT_TEXT_PAYLOAD_CEILING_BYTES,
        }
    );
}

#[test]
fn test_evaluate_request_size_multimodal_ceiling_accepted() {
    let payload_size = 35 * 1024 * 1024; // 35 MB
    // Under 50 MB multimodal ceiling
    let result = evaluate_request_size(Some(payload_size), payload_size, true);
    assert_eq!(result, SizeGuardOutcome::Accepted);
}

#[test]
fn test_evaluate_request_size_over_multimodal_ceiling_rejected() {
    let payload_size = 55 * 1024 * 1024; // 55 MB > 50 MB
    let result = evaluate_request_size(Some(payload_size), payload_size, true);
    assert_eq!(
        result,
        SizeGuardOutcome::RejectedTooLarge {
            actual_bytes: payload_size,
            ceiling_bytes: DEFAULT_MULTIMODAL_PAYLOAD_CEILING_BYTES,
        }
    );
}

#[test]
fn test_content_length_header_takes_maximum_of_header_and_body() {
    let reported_header = 15 * 1024 * 1024;
    let actual_body = 2 * 1024 * 1024;
    let result = evaluate_request_size(Some(reported_header), actual_body, false);
    assert_eq!(
        result,
        SizeGuardOutcome::RejectedTooLarge {
            actual_bytes: reported_header,
            ceiling_bytes: DEFAULT_TEXT_PAYLOAD_CEILING_BYTES,
        }
    );

    // If header is None, uses body len
    let result_none = evaluate_request_size(None, actual_body, false);
    assert_eq!(result_none, SizeGuardOutcome::Accepted);
}

#[test]
fn test_is_multimodal_model_name_detection() {
    assert!(is_multimodal_model_name("gemini-2.0-flash-exp"));
    assert!(is_multimodal_model_name("claude-3-5-sonnet-20241022"));
    assert!(is_multimodal_model_name("gpt-4o"));
    assert!(is_multimodal_model_name("gemini-1.5-pro"));
    assert!(is_multimodal_model_name("custom-vision-agent"));

    assert!(!is_multimodal_model_name("gpt-3.5-turbo"));
    assert!(!is_multimodal_model_name("text-davinci-003"));
    assert!(!is_multimodal_model_name("deepseek-coder-6.7b"));
}
