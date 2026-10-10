use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_compare_versions() {
        assert!(compare_versions("3.3.36", "3.3.35"));
        assert!(compare_versions("3.4.0", "3.3.35"));
        assert!(compare_versions("4.0.3", "3.3.35"));
        assert!(!compare_versions("3.3.34", "3.3.35"));
        assert!(!compare_versions("3.3.35", "3.3.35"));

        // Pre-release tests
        assert!(compare_versions("4.8.1-beta.2", "4.8.1-beta.1"));
        assert!(!compare_versions("4.8.1-beta.1", "4.8.1-beta.2"));
        assert!(compare_versions("4.8.1", "4.8.1-beta.2")); // 正式版 > 预发布版
        assert!(!compare_versions("4.8.1-beta.2", "4.8.1"));
        assert!(compare_versions("4.8.2-beta.1", "4.8.1"));
    }

    #[test]
    pub(crate) fn test_should_check_for_updates() {
        let mut settings = UpdateSettings::default();
        assert!(should_check_for_updates(&settings));

        settings.last_check_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(!should_check_for_updates(&settings));

        settings.auto_check = false;
        assert!(!should_check_for_updates(&settings));
    }

    #[test]
    pub(crate) fn update_settings_deserialize_missing_auto_check_and_last_check_time() {
        let json = r#"{
  "check_interval_hours": 24,
  "notify_on_update": true,
  "notify_via_email": true,
  "notify_via_telegram": true,
  "last_known_version": "4.108.0",
  "update_channel": "stable"
}"#;
        let settings: UpdateSettings = serde_json::from_str(json).expect("partial settings JSON");
        assert!(settings.auto_check);
        assert_eq!(settings.last_check_time, 0);
    }
}
