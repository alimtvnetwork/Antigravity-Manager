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
        assert!(compare_versions("4.8.1", "4.8.1-beta.2")); // Official version > Pre-release version
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

    fn fixture_asset_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "Antigravity-Manager-setup.exe"
        } else if cfg!(target_os = "macos") {
            "Antigravity-Manager.dmg"
        } else {
            "Antigravity-Manager.AppImage"
        }
    }

    fn fixture_release(version: &str, has_asset: bool) -> ReleaseAssetInfo {
        let assets = if has_asset {
            vec![GitHubReleaseAsset {
                name: fixture_asset_name().to_string(),
                browser_download_url: format!(
                    "https://example.invalid/releases/{}/installer",
                    version
                ),
            }]
        } else {
            Vec::new()
        };
        ReleaseAssetInfo {
            version: version.to_string(),
            html_url: format!("https://example.invalid/releases/{}", version),
            body: None,
            published_at: None,
            is_prerelease: false,
            is_draft: false,
            assets,
        }
    }

    #[test]
    pub(crate) fn test_resolver_skips_newest_without_asset() {
        // Plan 86, subtask 002 fixture: newest lacks an asset, previous is selected.
        let releases = vec![
            fixture_release("4.300.0", false),
            fixture_release("4.299.0", true),
            fixture_release("4.298.0", true),
        ];
        let res = resolve_installable_release(&releases, UpdateChannel::Stable);
        let selected = res.selected.expect("a release is selected");
        assert_eq!(selected.version, "4.299.0");
        assert!(res.selected_asset_url.is_some());
        assert_eq!(
            res.candidates,
            vec!["4.299.0".to_string(), "4.298.0".to_string()]
        );
        assert_eq!(res.skipped.len(), 1);
        assert_eq!(res.skipped[0].version, "4.300.0");
    }

    #[test]
    pub(crate) fn test_resolver_no_installable_release() {
        // Plan 86, subtask 002 fixture: nothing installable -> nothing selected, all skipped with reasons.
        let releases = vec![
            fixture_release("4.300.0", false),
            fixture_release("4.299.0", false),
        ];
        let res = resolve_installable_release(&releases, UpdateChannel::Stable);
        assert!(res.selected.is_none());
        assert!(res.candidates.is_empty());
        assert_eq!(res.skipped.len(), 2);
        assert!(res
            .skipped
            .iter()
            .all(|s| s.reason.contains("no installer asset")));
    }

    #[test]
    pub(crate) fn test_resolver_prerelease_ignored_on_stable() {
        // Plan 86, subtask 002 fixture: prereleases are ignored on stable, allowed on beta.
        let mut pre = fixture_release("4.300.0-beta.1", true);
        pre.is_prerelease = true;
        let stable = fixture_release("4.299.0", true);
        let releases = vec![pre, stable];

        let res = resolve_installable_release(&releases, UpdateChannel::Stable);
        assert_eq!(res.selected.expect("selected").version, "4.299.0");
        assert_eq!(res.skipped.len(), 1);
        assert_eq!(res.skipped[0].version, "4.300.0-beta.1");

        let res_beta = resolve_installable_release(&releases, UpdateChannel::Beta);
        assert_eq!(
            res_beta.selected.expect("selected").version,
            "4.300.0-beta.1"
        );
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
