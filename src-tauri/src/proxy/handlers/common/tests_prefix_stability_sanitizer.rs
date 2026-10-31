//! Unit Tests for Pipeline Prefix Stability & Header Sanitizer (`Imp-625`)

#[cfg(test)]
mod tests {
    use super::super::prefix_stability_sanitizer::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_sanitize_strips_forbidden_internal_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-agm-internal".to_string(), "true".to_string());
        headers.insert(
            "x-antigravity-internal".to_string(),
            "cluster-node-1".to_string(),
        );
        headers.insert("x-proxy-forwarded".to_string(), "10.0.0.1".to_string());
        headers.insert(
            "authorization".to_string(),
            "Bearer secret-token".to_string(),
        );

        let policy = PrefixStabilityPolicy::default();
        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 1);
        assert_eq!(
            clean.get("authorization").map(String::as_str),
            Some("Bearer secret-token")
        );
        assert!(!clean.contains_key("x-agm-internal"));
        assert!(!clean.contains_key("x-antigravity-internal"));
        assert!(!clean.contains_key("x-proxy-forwarded"));

        assert!(!report.is_stable);
        assert!(report.has_stripped_headers);
        assert_eq!(report.normalized_headers_count, 1);
        assert_eq!(
            report.stripped_headers,
            vec![
                "x-agm-internal".to_string(),
                "x-antigravity-internal".to_string(),
                "x-proxy-forwarded".to_string(),
            ]
        );
    }

    #[test]
    fn test_sanitize_strips_disallowed_goog_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-goog-api-client".to_string(), "gl-rust/0.1.0".to_string());
        headers.insert(
            "x-goog-api-key".to_string(),
            "AIzaSyInternalSecret".to_string(),
        );
        headers.insert("content-type".to_string(), "application/json".to_string());

        let policy = PrefixStabilityPolicy::default();
        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 1);
        assert_eq!(
            clean.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert!(!clean.contains_key("x-goog-api-client"));
        assert!(!clean.contains_key("x-goog-api-key"));

        assert!(!report.is_stable);
        assert!(report.has_stripped_headers);
        assert_eq!(
            report.stripped_headers,
            vec![
                "x-goog-api-client".to_string(),
                "x-goog-api-key".to_string(),
            ]
        );
    }

    #[test]
    fn test_sanitize_preserves_legitimate_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer sk-live".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("accept".to_string(), "text/event-stream".to_string());
        headers.insert("user-agent".to_string(), "AGM-Client/4.0".to_string());

        let policy = PrefixStabilityPolicy::default();
        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 4);
        assert_eq!(
            clean.get("authorization").map(String::as_str),
            Some("Bearer sk-live")
        );
        assert_eq!(
            clean.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(
            clean.get("accept").map(String::as_str),
            Some("text/event-stream")
        );
        assert_eq!(
            clean.get("user-agent").map(String::as_str),
            Some("AGM-Client/4.0")
        );

        assert!(report.is_stable);
        assert!(!report.has_stripped_headers);
        assert!(report.stripped_headers.is_empty());
        assert_eq!(report.normalized_headers_count, 4);
    }

    #[test]
    fn test_sanitize_case_insensitivity() {
        let mut headers = HashMap::new();
        headers.insert(
            "X-Goog-Api-Client".to_string(),
            "gl-python/3.11".to_string(),
        );
        headers.insert("X-AGM-INTERNAL".to_string(), "1".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("User-Agent".to_string(), "CustomClient".to_string());

        let policy = PrefixStabilityPolicy::default();
        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 2);
        assert_eq!(
            clean.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(
            clean.get("user-agent").map(String::as_str),
            Some("CustomClient")
        );

        assert_eq!(
            report.stripped_headers,
            vec![
                "x-agm-internal".to_string(),
                "x-goog-api-client".to_string(),
            ]
        );
    }

    #[test]
    fn test_sanitize_allowed_custom_headers_override() {
        let mut headers = HashMap::new();
        headers.insert("x-proxy-forwarded".to_string(), "192.168.1.1".to_string());
        headers.insert("x-agm-internal".to_string(), "true".to_string());

        let policy = PrefixStabilityPolicy::default()
            .with_allowed_headers(vec!["x-proxy-forwarded".to_string()]);

        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 1);
        assert_eq!(
            clean.get("x-proxy-forwarded").map(String::as_str),
            Some("192.168.1.1")
        );
        assert!(!clean.contains_key("x-agm-internal"));

        assert_eq!(report.stripped_headers, vec!["x-agm-internal".to_string()]);
    }

    #[test]
    fn test_sanitize_strip_goog_disabled() {
        let mut headers = HashMap::new();
        headers.insert("x-goog-api-client".to_string(), "gl-rust/0.1.0".to_string());
        headers.insert("authorization".to_string(), "Bearer token".to_string());

        let policy = PrefixStabilityPolicy::default().with_strip_disallowed_goog_headers(false);

        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert_eq!(clean.len(), 2);
        assert!(clean.contains_key("x-goog-api-client"));
        assert!(clean.contains_key("authorization"));
        assert!(report.is_stable);
        assert!(!report.has_stripped_headers);
    }

    #[test]
    fn test_deterministic_json_key_sorting_unordered_objects() {
        let obj1 = json!({
            "zebra": 10,
            "apple": "fruit",
            "middle": true,
            "beta": [1, 2, 3]
        });

        let obj2 = json!({
            "apple": "fruit",
            "beta": [1, 2, 3],
            "middle": true,
            "zebra": 10
        });

        let ser1 = serialize_deterministic_json(&obj1).expect("serialization succeeds");
        let ser2 = serialize_deterministic_json(&obj2).expect("serialization succeeds");

        assert_eq!(
            ser1, ser2,
            "Unordered object keys must serialize to identical strings"
        );
        assert_eq!(
            ser1,
            r#"{"apple":"fruit","beta":[1,2,3],"middle":true,"zebra":10}"#
        );
    }

    #[test]
    fn test_deterministic_json_nested_structures() {
        let complex1 = json!({
            "meta": {
                "tags": ["b", "a"],
                "author": { "name": "Alice", "id": 1 }
            },
            "data": [
                { "z": 9, "a": 1 },
                { "k": 2, "b": 8 }
            ]
        });

        let complex2 = json!({
            "data": [
                { "a": 1, "z": 9 },
                { "b": 8, "k": 2 }
            ],
            "meta": {
                "author": { "id": 1, "name": "Alice" },
                "tags": ["b", "a"]
            }
        });

        let ser1 = serialize_deterministic_json(&complex1).expect("serialization succeeds");
        let ser2 = serialize_deterministic_json(&complex2).expect("serialization succeeds");

        assert_eq!(
            ser1, ser2,
            "Nested structures must recursively serialize deterministically"
        );
    }

    #[test]
    fn test_empty_headers_stable() {
        let headers = HashMap::new();
        let policy = PrefixStabilityPolicy::default();
        let (clean, report) = sanitize_request_headers(&headers, &policy);

        assert!(clean.is_empty());
        assert!(report.is_stable);
        assert!(!report.has_stripped_headers);
        assert_eq!(report.normalized_headers_count, 0);
    }

    #[test]
    fn test_serde_roundtrip_policy_and_report() {
        let policy = PrefixStabilityPolicy {
            strip_disallowed_goog_headers: false,
            allowed_custom_headers: vec!["x-custom-1".to_string()],
            deterministic_key_order: true,
        };
        let policy_str = serde_json::to_string(&policy).expect("serialize policy");
        let deserialized_policy: PrefixStabilityPolicy =
            serde_json::from_str(&policy_str).expect("deserialize policy");
        assert_eq!(policy, deserialized_policy);

        let report = PrefixSanitizationReport::new(vec!["x-agm-internal".to_string()], 3);
        let report_str = serde_json::to_string(&report).expect("serialize report");
        let deserialized_report: PrefixSanitizationReport =
            serde_json::from_str(&report_str).expect("deserialize report");
        assert_eq!(report, deserialized_report);
    }
}
