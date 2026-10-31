#[cfg(test)]
mod tests {
    use super::super::json_payload_sanitizer::JsonPayloadSanitizer;
    use serde_json::Value;

    #[test]
    fn test_sanitize_raw_json_clean_passthrough() {
        let clean = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let out = JsonPayloadSanitizer::sanitize_raw_json(clean);
        assert_eq!(out, clean);
    }

    #[test]
    fn test_sanitize_raw_json_unescaped_control_characters() {
        // Embed raw byte 0x07 (BEL) and 0x01 (SOH) inside string value
        let raw = format!("{{\"text\": \"hello{}\"}}", '\x07');
        let sanitized = JsonPayloadSanitizer::sanitize_raw_json(&raw);
        assert!(sanitized.contains("\\u0007"));

        // Verify result parses as valid JSON
        let val: Value = serde_json::from_str(&sanitized).unwrap();
        assert_eq!(val["text"], "hello\x07");
    }

    #[test]
    fn test_strip_trailing_commas_in_objects_and_arrays() {
        let malformed_obj = r#"{"a": 1, "b": 2,}"#;
        let cleaned_obj = JsonPayloadSanitizer::strip_trailing_commas(malformed_obj);
        assert_eq!(cleaned_obj, r#"{"a": 1, "b": 2}"#);

        let malformed_arr = r#"[1, 2, 3,]"#;
        let cleaned_arr = JsonPayloadSanitizer::strip_trailing_commas(malformed_arr);
        assert_eq!(cleaned_arr, r#"[1, 2, 3]"#);
    }

    #[test]
    fn test_strip_trailing_commas_preserves_string_commas() {
        let json_with_commas = r#"{"msg": "comma, inside, string", "items": [1, 2]}"#;
        let cleaned = JsonPayloadSanitizer::strip_trailing_commas(json_with_commas);
        assert_eq!(cleaned, json_with_commas);
    }

    #[test]
    fn test_sanitize_and_clean_full_pipeline() {
        let dirty = format!("{{\"prompt\": \"line 1{}line 2\", \"extra\": 42,}}", '\x01');
        let cleaned = JsonPayloadSanitizer::sanitize_and_clean(&dirty);

        let parsed: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(parsed["extra"], 42);
        assert!(parsed["prompt"].as_str().unwrap().contains("line 1"));
    }
}
