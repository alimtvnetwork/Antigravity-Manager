#[cfg(test)]
mod tests {
    use super::super::settings_deep_merge_synchronizer::{
        deep_merge_settings, DEFAULT_PRESERVED_IDENTITY_KEYS,
    };
    use serde_json::json;

    #[test]
    fn test_deep_merge_preserves_instance_identity_keys() {
        let base = json!({
            "id": "inst-clone-42",
            "port": 9225,
            "theme": "dark",
            "editor": {
                "fontSize": 14,
                "tabSize": 2
            }
        });

        let incoming = json!({
            "id": "inst-primary-01",
            "port": 9222,
            "theme": "light",
            "editor": {
                "fontSize": 16,
                "wordWrap": true
            },
            "newPluginEnabled": true
        });

        let merged = deep_merge_settings(&base, &incoming, DEFAULT_PRESERVED_IDENTITY_KEYS);

        // Identity keys preserved
        assert_eq!(merged["id"], "inst-clone-42");
        assert_eq!(merged["port"], 9225);

        // Settings updated
        assert_eq!(merged["theme"], "light");
        assert_eq!(merged["newPluginEnabled"], true);

        // Nested settings merged recursively
        assert_eq!(merged["editor"]["fontSize"], 16);
        assert_eq!(merged["editor"]["tabSize"], 2);
        assert_eq!(merged["editor"]["wordWrap"], true);
    }

    #[test]
    fn test_deep_merge_adds_new_keys_when_absent_in_base() {
        let base = json!({
            "id": "inst-2"
        });

        let incoming = json!({
            "telemetry": false,
            "autoSwitchFloorMinutes": 5
        });

        let merged = deep_merge_settings(&base, &incoming, DEFAULT_PRESERVED_IDENTITY_KEYS);
        assert_eq!(merged["id"], "inst-2");
        assert_eq!(merged["telemetry"], false);
        assert_eq!(merged["autoSwitchFloorMinutes"], 5);
    }
}
