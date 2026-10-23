//! Unit tests for pipeline request header sanitizer.

#[cfg(test)]
mod tests {
    use super::pipeline_request_header_sanitizer::*;
    use std::collections::HashMap;

    #[test]
    fn test_clean_headers_preserved() {
        let mut headers = HashMap::new();
        let token = format!("{}-{}", "Bearer", "valid_auth_token_98765");
        headers.insert("authorization".to_string(), token.clone());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-goog-api-key".to_string(), "gemini-api-key-sample".to_string());
        headers.insert("user-agent".to_string(), "Antigravity/4.0".to_string());
        headers.insert("accept".to_string(), "application/json".to_string());

        let report = sanitize_pipeline_request_headers(&headers);

        assert!(report.is_clean);
        assert!(!report.has_violations);
        assert_eq!(report.total_input_headers, 5);
        assert_eq!(report.retained_headers.len(), 5);
        assert!(report.stripped_headers.is_empty());
        assert!(report.normalized_headers.is_empty());
        assert_eq!(
            report.retained_headers.get("authorization").map(String::as_str),
            Some(token.as_str())
        );
    }

    #[test]
    fn test_hop_by_hop_headers_stripped() {
        let mut headers = HashMap::new();
        headers.insert("connection".to_string(), "keep-alive".to_string());
        headers.insert("keep-alive".to_string(), "timeout=5".to_string());
        headers.insert("transfer-encoding".to_string(), "chunked".to_string());
        headers.insert("host".to_string(), "api.example.com".to_string());
        headers.insert("content-length".to_string(), "1024".to_string());

        let report = sanitize_pipeline_request_headers(&headers);

        assert!(!report.is_clean);
        assert!(report.has_violations);
        assert_eq!(report.stripped_headers.len(), 5);
        assert!(report.retained_headers.is_empty());

        for record in &report.stripped_headers {
            match &record.action {
                SanitizedHeaderAction::Stripped { category, reason } => {
                    assert_eq!(*category, HeaderSanitizationCategory::HopByHopTransport);
                    assert!(reason.contains("Hop-by-hop"));
                }
                _ => panic!("Expected Stripped action for hop-by-hop header"),
            }
        }
    }

    #[test]
    fn test_risky_internal_metadata_stripped() {
        let mut headers = HashMap::new();
        headers.insert("x-goog-user-project".to_string(), "secret-proj".to_string());
        headers.insert("x-antigravity-internal".to_string(), "true".to_string());
        headers.insert("cf-connecting-ip".to_string(), "1.2.3.4".to_string());
        headers.insert("x-real-ip".to_string(), "5.6.7.8".to_string());
        headers.insert("x-forwarded-for".to_string(), "10.0.0.1".to_string());
        headers.insert("x-forwarded-proto".to_string(), "https".to_string());
        headers.insert("x-agm-instance-id".to_string(), "inst-001".to_string());
        headers.insert("x-internal-routing-secret".to_string(), "route-key".to_string());

        let report = sanitize_pipeline_request_headers(&headers);

        assert!(!report.is_clean);
        assert!(report.has_violations);
        assert_eq!(report.stripped_headers.len(), 8);
        assert!(report.retained_headers.is_empty());

        for record in &report.stripped_headers {
            match &record.action {
                SanitizedHeaderAction::Stripped { category, reason } => {
                    assert_eq!(*category, HeaderSanitizationCategory::RiskyInternalTelemetry);
                    assert!(reason.contains("Internal telemetry"));
                }
                _ => panic!("Expected Stripped action for internal header"),
            }
        }
    }

    #[test]
    fn test_untrusted_client_metadata_stripped() {
        let mut headers = HashMap::new();
        headers.insert("cookie".to_string(), "session=secret".to_string());
        headers.insert("cookie2".to_string(), "v=2".to_string());
        headers.insert("sec-ch-ua".to_string(), "\"Google Chrome\";v=\"125\"".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());

        let report = sanitize_pipeline_request_headers(&headers);

        assert!(!report.is_clean);
        assert!(report.has_violations);
        assert_eq!(report.stripped_headers.len(), 4);
        assert!(report.retained_headers.is_empty());

        for record in &report.stripped_headers {
            match &record.action {
                SanitizedHeaderAction::Stripped { category, reason } => {
                    assert_eq!(*category, HeaderSanitizationCategory::UntrustedClientMetadata);
                    assert!(reason.contains("Untrusted client metadata"));
                }
                _ => panic!("Expected Stripped action for untrusted metadata"),
            }
        }
    }

    #[test]
    fn test_casing_normalization() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer token-xyz".to_string());
        headers.insert("X-Goog-Api-Key".to_string(), "my-key".to_string());

        let report = sanitize_pipeline_request_headers(&headers);

        assert!(report.is_clean);
        assert!(!report.has_violations);
        assert_eq!(report.normalized_headers.len(), 3);
        assert_eq!(report.retained_headers.len(), 3);

        assert!(report.retained_headers.contains_key("content-type"));
        assert!(report.retained_headers.contains_key("authorization"));
        assert!(report.retained_headers.contains_key("x-goog-api-key"));

        let norm_keys: Vec<&str> = report
            .normalized_headers
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert!(norm_keys.contains(&"Content-Type"));
        assert!(norm_keys.contains(&"Authorization"));
        assert!(norm_keys.contains(&"X-Goog-Api-Key"));
    }

    #[test]
    fn test_categorize_header_direct() {
        assert_eq!(
            categorize_header("connection"),
            HeaderSanitizationCategory::HopByHopTransport
        );
        assert_eq!(
            categorize_header("Transfer-Encoding"),
            HeaderSanitizationCategory::HopByHopTransport
        );
        assert_eq!(
            categorize_header("x-agm-worker"),
            HeaderSanitizationCategory::RiskyInternalTelemetry
        );
        assert_eq!(
            categorize_header("x-internal-flag"),
            HeaderSanitizationCategory::RiskyInternalTelemetry
        );
        assert_eq!(
            categorize_header("cf-connecting-ip"),
            HeaderSanitizationCategory::RiskyInternalTelemetry
        );
        assert_eq!(
            categorize_header("Cookie"),
            HeaderSanitizationCategory::UntrustedClientMetadata
        );
        assert_eq!(
            categorize_header("sec-ch-ua-mobile"),
            HeaderSanitizationCategory::UntrustedClientMetadata
        );
        assert_eq!(
            categorize_header("content-type"),
            HeaderSanitizationCategory::AllowedProtocolHeader
        );
    }

    #[test]
    fn test_display_and_helpers() {
        let cat = HeaderSanitizationCategory::HopByHopTransport;
        assert_eq!(format!("{cat}"), "HopByHopTransport");

        let action = SanitizedHeaderAction::Stripped {
            reason: "test reason".to_string(),
            category: HeaderSanitizationCategory::RiskyInternalTelemetry,
        };
        assert!(format!("{action}").contains("test reason"));

        let norm_action = SanitizedHeaderAction::NormalizedCasing {
            original: "Header-Key".to_string(),
            canonical: "header-key".to_string(),
        };
        assert!(format!("{norm_action}").contains("Header-Key"));

        assert!(is_mandatory_allowed_prefix("x-goog-custom"));
        assert!(is_mandatory_allowed_prefix("Authorization"));
        assert!(!is_mandatory_allowed_prefix("custom-unknown"));
    }
}
