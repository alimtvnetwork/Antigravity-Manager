use serde::{Deserialize, Serialize};

use super::*;

pub(crate) const GITHUB_API_URL: &str =
    "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";

pub(crate) const GITHUB_RELEASES_API_URL: &str =
    "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases?per_page=15";

pub(crate) const GITHUB_RAW_URL: &str =
    "https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/package.json";

pub(crate) const JSDELIVR_URL: &str =
    "https://cdn.jsdelivr.net/gh/alimtvnetwork/Antigravity-Manager@main/package.json";

pub(crate) const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DEFAULT_CHECK_INTERVAL_HOURS: u64 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        if CURRENT_VERSION.contains('-') {
            UpdateChannel::Beta
        } else {
            UpdateChannel::Stable
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String, // previously release_url
    #[serde(default)]
    pub release_notes: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub channel: Option<UpdateChannel>,
    #[serde(default)]
    pub updater_json_url: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    #[serde(default = "default_true")]
    pub auto_check: bool,
    #[serde(default)]
    pub last_check_time: u64,
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
    #[serde(default = "default_true")]
    pub notify_on_update: bool,
    #[serde(default = "default_true")]
    pub notify_via_email: bool,
    #[serde(default = "default_true")]
    pub notify_via_telegram: bool,
    #[serde(default)]
    pub last_known_version: String,
    #[serde(default)]
    pub update_channel: UpdateChannel,
}

fn default_check_interval() -> u64 {
    DEFAULT_CHECK_INTERVAL_HOURS
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            last_check_time: 0,
            check_interval_hours: DEFAULT_CHECK_INTERVAL_HOURS,
            notify_on_update: true,
            notify_via_email: true,
            notify_via_telegram: true,
            last_known_version: env!("CARGO_PKG_VERSION").to_string(),
            update_channel: UpdateChannel::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitHubRelease {
    pub(crate) tag_name: String,
    pub(crate) html_url: String,
    pub(crate) body: Option<String>,
    pub(crate) published_at: Option<String>,
    #[serde(default)]
    pub(crate) prerelease: bool,
    #[serde(default)]
    pub(crate) assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitHubReleaseAsset {
    pub(crate) name: String,
    pub(crate) browser_download_url: String,
}

pub const STABLE_UPDATER_JSON_URL: &str =
    "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json";

pub const PREVIEW_UPDATER_JSON_URL: &str =
    "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/preview/updater.json";
