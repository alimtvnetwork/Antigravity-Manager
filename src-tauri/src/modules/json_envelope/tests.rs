use serde_json::Value;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    pub(crate) fn test_envelope_serialization_and_unpacking() {
        let envelope = JsonEnvelope::new("agm/test-type", json!({ "key": "value", "count": 42 }))
            .with_description("Unit test payload");

        let serialized = serde_json::to_string(&envelope).expect("serialize envelope");
        let parsed: Value = serde_json::from_str(&serialized).expect("deserialize Value");

        let (attrs_opt, data) = unpack_envelope(parsed);
        assert!(attrs_opt.is_some());
        let attrs = attrs_opt.unwrap();
        assert_eq!(attrs.data_type, "agm/test-type");
        assert_eq!(attrs.version, "2.0");
        assert_eq!(attrs.description.as_deref(), Some("Unit test payload"));
        assert_eq!(data["key"], "value");
        assert_eq!(data["count"], 42);
    }

    #[test]
    pub(crate) fn test_variable_interpolation_and_chaining() {
        let raw = r#"{
            "attributes": {
                "type": "agm/supabase-endpoints",
                "version": "2.0",
                "workDirectory": {
                    "path": "${workDir}",
                    "defaultPath": "D:\\work",
                    "isApplied": true,
                    "isEnforced": false
                }
            },
            "variables": {
                "workDir": "D:\\work",
                "repoDir": "${workDir}\\antigravity-manager",
                "baseUrl": "https://pezjuuddecbyfmqxytrv.supabase.co",
                "restUrl": "${baseUrl}/rest/v1/"
            },
            "data": {
                "endpoint": "${restUrl}",
                "dir": "${repoDir}"
            }
        }"#;

        let (data, attrs) = extract_payload::<Value>(raw).expect("extract_payload");
        assert_eq!(attrs.version, "2.0");
        assert_eq!(
            attrs.work_directory.map(|w| w.path),
            Some("D:\\work".to_string())
        );
        assert_eq!(
            data["endpoint"].as_str().unwrap(),
            "https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/"
        );
        assert_eq!(
            data["dir"].as_str().unwrap(),
            "D:\\work\\antigravity-manager"
        );
    }

    #[test]
    pub(crate) fn test_work_directory_polymorphism() {
        let raw_str = r#"{
            "attributes": {
                "type": "agm/test",
                "workDirectory": "D:\\custom\\path"
            },
            "data": { "ok": true }
        }"#;
        let (_, attrs) =
            extract_payload::<Value>(raw_str).expect("extract flat workDirectory string");
        assert_eq!(
            attrs.work_directory.map(|w| w.path),
            Some("D:\\custom\\path".to_string())
        );

        let raw_obj = r#"{
            "attributes": {
                "type": "agm/test",
                "workDirectory": {
                    "path": "${workDir}",
                    "defaultPath": "D:\\work",
                    "isApplied": true,
                    "isEnforced": false
                }
            },
            "variables": {
                "workDir": "D:\\work"
            },
            "data": { "ok": true }
        }"#;
        let (_, attrs2) = extract_payload::<Value>(raw_obj).expect("extract obj workDirectory");
        assert_eq!(
            attrs2.work_directory.map(|w| w.path),
            Some("D:\\work".to_string())
        );
    }
}
