use crate::modules::logger;
use crate::utils::command::CommandExtWrapper;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";
const GITHUB_RELEASES_API_URL: &str =
    "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases?per_page=15";
const GITHUB_RAW_URL: &str =
    "https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/package.json";
const JSDELIVR_URL: &str =
    "https://cdn.jsdelivr.net/gh/alimtvnetwork/Antigravity-Manager@main/package.json";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_CHECK_INTERVAL_HOURS: u64 = 24;

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
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub const STABLE_UPDATER_JSON_URL: &str =
    "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json";
pub const PREVIEW_UPDATER_JSON_URL: &str =
    "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/preview/updater.json";

pub fn get_upstream_proxy_url() -> Option<String> {
    if let Ok(config) = crate::modules::config::load_app_config() {
        if config.proxy.upstream_proxy.enabled && !config.proxy.upstream_proxy.url.trim().is_empty()
        {
            let url = config.proxy.upstream_proxy.url.trim();
            let normalized = if !url.contains("://") {
                format!("http://{}", url)
            } else {
                url.to_string()
            };
            return Some(normalized);
        }
    }

    // 兜底：若未显式配置上游代理，尝试从系统环境变量获取代理 (HTTPS_PROXY / HTTP_PROXY / ALL_PROXY)
    for env_var in &[
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ] {
        if let Ok(val) = std::env::var(env_var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                let normalized = if !trimmed.contains("://") {
                    format!("http://{}", trimmed)
                } else {
                    trimmed.to_string()
                };
                return Some(normalized);
            }
        }
    }

    None
}

/// Check for updates with improved strategy:
/// 1. Check updater.json (Source of Truth for Auto-Update)
/// 2. Fallback to GitHub API (Informational)
/// Check for updates with improved strategy:
/// 1. Check updater.json based on selected channel (Stable vs Beta)
/// 2. Fallback to GitHub API (Release or Pre-release)
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let settings = load_update_settings().unwrap_or_default();
    let mut info = check_for_updates_internal(settings.update_channel).await?;
    info.proxy_url = get_upstream_proxy_url();
    info.channel = Some(settings.update_channel);
    Ok(info)
}

async fn check_for_updates_internal(channel: UpdateChannel) -> Result<UpdateInfo, String> {
    // 1. Try updater.json first (Critical for functional Auto-Update)
    match check_updater_json_channel(channel).await {
        Ok(info) => return Ok(info),
        Err(e) => {
            logger::log_warn(&format!(
                "{:?} updater.json check failed: {}. Trying fallbacks...",
                channel, e
            ));
        }
    }

    // 2. Try GitHub API
    match check_github_api_channel(channel).await {
        Ok(info) => return Ok(info),
        Err(e) => {
            logger::log_warn(&format!(
                "GitHub API ({:?}) check failed: {}. Trying static fallbacks...",
                channel, e
            ));
        }
    }

    // 3. Try GitHub Raw (only applies to stable/main)
    if channel == UpdateChannel::Stable {
        if let Ok(info) = check_static_url(GITHUB_RAW_URL, "GitHub Raw").await {
            return Ok(info);
        }
        if let Ok(info) = check_static_url(JSDELIVR_URL, "jsDelivr").await {
            return Ok(info);
        }
    }

    Err(format!(
        "Failed to fetch updates for {:?} channel. Please check network/proxy settings.",
        channel
    ))
}

#[derive(Debug, Deserialize)]
struct UpdaterJson {
    version: String,
    notes: Option<String>,
    pub_date: Option<String>,
}

async fn create_client() -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .user_agent("Antigravity-Manager")
        .timeout(std::time::Duration::from_secs(10));

    // Load config to check for upstream proxy
    if let Ok(config) = crate::modules::config::load_app_config() {
        if config.proxy.upstream_proxy.enabled && !config.proxy.upstream_proxy.url.is_empty() {
            logger::log_info(&format!(
                "Update checker using upstream proxy: {}",
                config.proxy.upstream_proxy.url
            ));
            match reqwest::Proxy::all(&config.proxy.upstream_proxy.url) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    logger::log_warn(&format!(
                        "Failed to parse proxy URL '{}': {}",
                        config.proxy.upstream_proxy.url, e
                    ));
                }
            }
        }
    }

    builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

async fn check_updater_json_channel(channel: UpdateChannel) -> Result<UpdateInfo, String> {
    let client = create_client().await?;
    let target_url = match channel {
        UpdateChannel::Stable => STABLE_UPDATER_JSON_URL,
        UpdateChannel::Beta => PREVIEW_UPDATER_JSON_URL,
    };

    logger::log_info(&format!(
        "Checking for updates via {:?} updater.json ({})...",
        channel, target_url
    ));

    let response = client.get(target_url).send().await;

    let (response, actual_url) = match response {
        Ok(res) if res.status().is_success() => (res, target_url.to_string()),
        other => {
            if channel == UpdateChannel::Beta {
                logger::log_info("Preview updater.json endpoint unavailable, checking latest prerelease assets from GitHub API...");
                if let Ok(asset_url) = fetch_prerelease_updater_json_url(&client).await {
                    let res = client
                        .get(&asset_url)
                        .send()
                        .await
                        .map_err(|e| format!("Request failed: {}", e))?;
                    (res, asset_url)
                } else {
                    let err_msg = match other {
                        Ok(res) => format!("status {}", res.status()),
                        Err(e) => e.to_string(),
                    };
                    return Err(format!("updater.json returned {}", err_msg));
                }
            } else {
                let err_msg = match other {
                    Ok(res) => format!("status {}", res.status()),
                    Err(e) => e.to_string(),
                };
                return Err(format!("updater.json returned {}", err_msg));
            }
        }
    };

    let updater_info: UpdaterJson = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse updater.json: {}", e))?;

    let latest_version = updater_info.version.trim_start_matches('v').to_string();
    let current_version = CURRENT_VERSION.to_string();
    let has_update = compare_versions(&latest_version, &current_version);

    if has_update {
        logger::log_info(&format!(
            "New version found ({:?} updater.json): {} (Current: {})",
            channel, latest_version, current_version
        ));
    } else {
        logger::log_info(&format!(
            "Up to date ({:?} updater.json): {} (Matches {})",
            channel, current_version, latest_version
        ));
    }

    let download_url = format!(
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v{}",
        latest_version
    );

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        download_url,
        release_notes: updater_info
            .notes
            .unwrap_or_else(|| "Release notes available on GitHub.".to_string()),
        published_at: updater_info
            .pub_date
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        source: Some(format!("{:?} updater.json", channel)),
        proxy_url: None,
        channel: Some(channel),
        updater_json_url: Some(actual_url),
    })
}

async fn fetch_prerelease_updater_json_url(client: &reqwest::Client) -> Result<String, String> {
    let response = client
        .get(GITHUB_RELEASES_API_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to query releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Releases API returned status {}",
            response.status()
        ));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    for release in releases {
        if release.prerelease {
            if let Some(asset) = release
                .assets
                .into_iter()
                .find(|a| a.name.eq_ignore_ascii_case("updater.json"))
            {
                return Ok(asset.browser_download_url);
            }
        }
    }

    Err("No updater.json found in latest pre-releases".to_string())
}

async fn check_github_api_channel(channel: UpdateChannel) -> Result<UpdateInfo, String> {
    let client = create_client().await?;
    logger::log_info(&format!(
        "Checking for updates via GitHub API ({:?} channel)...",
        channel
    ));

    let release = match channel {
        UpdateChannel::Stable => {
            let response = client
                .get(GITHUB_API_URL)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("GitHub API returned status: {}", response.status()));
            }

            response
                .json::<GitHubRelease>()
                .await
                .map_err(|e| format!("Failed to parse release info: {}", e))?
        }
        UpdateChannel::Beta => {
            let response = client
                .get(GITHUB_RELEASES_API_URL)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("GitHub API returned status: {}", response.status()));
            }

            let releases: Vec<GitHubRelease> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse releases: {}", e))?;

            // 优先查找最新的 pre-release
            let latest_pre = releases
                .into_iter()
                .find(|r| r.prerelease)
                .ok_or_else(|| "No pre-release found on GitHub".to_string())?;

            latest_pre
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let current_version = CURRENT_VERSION.to_string();
    let has_update = compare_versions(&latest_version, &current_version);

    if has_update {
        logger::log_info(&format!(
            "New version found (API {:?}): {} (Current: {})",
            channel, latest_version, current_version
        ));
    } else {
        logger::log_info(&format!(
            "Up to date (API {:?}): {} (Matches {})",
            channel, current_version, latest_version
        ));
    }

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        download_url: release.html_url,
        release_notes: release.body.unwrap_or_default(),
        published_at: release
            .published_at
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        source: Some(format!("GitHub API ({:?})", channel)),
        proxy_url: None,
        channel: Some(channel),
        updater_json_url: match channel {
            UpdateChannel::Beta => Some(PREVIEW_UPDATER_JSON_URL.to_string()),
            UpdateChannel::Stable => Some(STABLE_UPDATER_JSON_URL.to_string()),
        },
    })
}

#[derive(Deserialize)]
struct PackageJson {
    version: String,
}

async fn check_static_url(url: &str, source_name: &str) -> Result<UpdateInfo, String> {
    let client = create_client().await?;

    logger::log_info(&format!("Checking for updates via {}...", source_name));

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "{} returned status: {}",
            source_name,
            response.status()
        ));
    }

    let package_json: PackageJson = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse package.json: {}", e))?;

    let latest_version = package_json.version;
    let current_version = CURRENT_VERSION.to_string();
    let has_update = compare_versions(&latest_version, &current_version);

    if has_update {
        logger::log_info(&format!(
            "New version found ({}): {} (Current: {})",
            source_name, latest_version, current_version
        ));
    } else {
        logger::log_info(&format!(
            "Up to date ({}): {} (Matches {})",
            source_name, current_version, latest_version
        ));
    }

    // fallback sources generally don't provide release notes or download specific URL, construct generic
    let download_url =
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest".to_string();
    let release_notes = format!(
        "New version detected via {}. Please check release page for details.",
        source_name
    );

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        download_url,
        release_notes,
        published_at: Utc::now().to_rfc3339(), // Approximate time
        source: Some(source_name.to_string()),
        proxy_url: None,
        channel: Some(UpdateChannel::Stable),
        updater_json_url: Some(STABLE_UPDATER_JSON_URL.to_string()),
    })
}

/// Compare two semantic versions (supports pre-release tags like "4.8.1-beta.2" vs "4.8.1-beta.1")
pub fn compare_versions(latest: &str, current: &str) -> bool {
    let parse_semver = |v: &str| -> (Vec<u32>, Option<(String, u32)>) {
        let clean = v.trim().trim_start_matches('v');
        if let Some((main_part, pre_part)) = clean.split_once('-') {
            let nums: Vec<u32> = main_part
                .split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
            // 解析预发布段，如 beta.2 -> ("beta", 2)
            let pre_info = if let Some((tag, num_str)) = pre_part.split_once('.') {
                Some((tag.to_lowercase(), num_str.parse::<u32>().unwrap_or(0)))
            } else {
                Some((pre_part.to_lowercase(), 0))
            };
            (nums, pre_info)
        } else {
            let nums: Vec<u32> = clean
                .split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
            (nums, None)
        }
    };

    let (latest_nums, latest_pre) = parse_semver(latest);
    let (current_nums, current_pre) = parse_semver(current);

    // 1. 先比较主版本号 [major, minor, patch]
    for i in 0..latest_nums.len().max(current_nums.len()) {
        let l = latest_nums.get(i).copied().unwrap_or(0);
        let c = current_nums.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    // 2. 主版本号完全相同时，检查 pre-release (标准 SemVer 规则：无 pre-release > 有 pre-release)
    match (latest_pre, current_pre) {
        (None, Some(_)) => true,  // e.g. latest 4.8.1 正式版 > current 4.8.1-beta.2
        (Some(_), None) => false, // e.g. latest 4.8.1-beta.2 < current 4.8.1 正式版
        (Some((l_tag, l_num)), Some((c_tag, c_num))) => {
            if l_tag != c_tag {
                l_tag > c_tag
            } else {
                l_num > c_num // e.g. beta.2 > beta.1
            }
        }
        (None, None) => false, // 完全相同版本
    }
}

/// Check if enough time has passed since last check
pub fn should_check_for_updates(settings: &UpdateSettings) -> bool {
    if !settings.auto_check {
        return false;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed_hours = (now - settings.last_check_time) / 3600;
    let interval = if settings.check_interval_hours > 0 {
        settings.check_interval_hours
    } else {
        DEFAULT_CHECK_INTERVAL_HOURS
    };
    elapsed_hours >= interval
}

/// Load update settings from config file
pub fn load_update_settings() -> Result<UpdateSettings, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    if !settings_path.exists() {
        return Ok(UpdateSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

/// Save update settings to config file
pub fn save_update_settings(settings: &UpdateSettings) -> Result<(), String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))
}

/// Update last check time
pub fn update_last_check_time() -> Result<(), String> {
    let mut settings = load_update_settings()?;
    settings.last_check_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    save_update_settings(&settings)
}

/// Detect if the app was installed via Homebrew Cask (macOS only)
pub fn is_homebrew_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        let caskroom_paths = [
            "/opt/homebrew/Caskroom/antigravity-tools",
            "/usr/local/Caskroom/antigravity-tools",
        ];

        for path in &caskroom_paths {
            if std::path::Path::new(path).exists() {
                logger::log_info(&format!("Detected Homebrew Cask installation at: {}", path));
                return true;
            }
        }
    }

    false
}

/// Detect if the app is currently running as an AppImage (Linux only).
///
/// The AppImage runtime always sets the `APPIMAGE` environment variable to the
/// absolute path of the source `.AppImage` file before mounting and executing the
/// bundled application. This is the canonical way to detect an AppImage execution
/// context without inspecting the filesystem.
///
/// This is used to gate Tauri's native auto-updater on Linux: Tauri's updater plugin
/// only supports AppImage bundles on Linux. Attempting to use it on RPM/DEB-installed
/// binaries results in an `ENOEXEC` error because the downloaded artifact is an
/// AppImage that cannot be executed without FUSE support (or proper permissions).
pub fn is_appimage_running() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("APPIMAGE").is_ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Execute `brew upgrade --cask antigravity-tools` with timeout (macOS only)
#[cfg(not(target_os = "macos"))]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    Err("brew_not_supported".to_string())
}

#[cfg(target_os = "macos")]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    logger::log_info("Starting Homebrew Cask upgrade for antigravity-tools...");

    // Find brew binary
    let brew_path = if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
        "/opt/homebrew/bin/brew"
    } else if std::path::Path::new("/usr/local/bin/brew").exists() {
        "/usr/local/bin/brew"
    } else {
        return Err("brew_not_found".to_string());
    };

    // 3 min timeout to prevent hanging
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        tokio::process::Command::new(brew_path)
            .args(["upgrade", "--cask", "antigravity-tools"])
            .output(),
    )
    .await;

    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            logger::log_error(&format!("Failed to execute brew upgrade: {}", e));
            return Err("brew_exec_failed".to_string());
        }
        Err(_) => {
            logger::log_error("Homebrew upgrade timed out after 3 minutes");
            return Err("brew_timeout".to_string());
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        logger::log_info(&format!("Homebrew upgrade succeeded: {}", stdout));
        Ok(stdout)
    } else {
        logger::log_error(&format!(
            "brew upgrade failed - stdout: {} stderr: {}",
            stdout, stderr
        ));
        // Return structured error key for frontend i18n
        if stderr.contains("already installed") || stdout.contains("already installed") {
            Err("brew_already_latest".to_string())
        } else {
            Err("brew_upgrade_failed".to_string())
        }
    }
}

/// Check for updates by executing our official installer shell script
pub async fn check_update_via_script() -> Result<UpdateInfo, String> {
    logger::log_info("Executing installer shell script to check for updates...");

    #[cfg(target_os = "windows")]
    {
        let mut ps_cmd = tokio::process::Command::new("powershell");
        ps_cmd.creation_flags_windows().args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
        ]);

        if std::path::Path::new("install.ps1").exists() {
            ps_cmd.args(["-File", ".\\install.ps1", "-CheckUpdate"]);
        } else {
            ps_cmd.args([
                "-Command",
                "& { $script = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($script)) -CheckUpdate }",
            ]);
        }

        let output = ps_cmd
            .output()
            .await
            .map_err(|e| format!("Failed to run powershell update check: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !stdout.is_empty() {
            if let Some(start_idx) = stdout.find('{') {
                if let Some(end_idx) = stdout.rfind('}') {
                    let json_str = &stdout[start_idx..=end_idx];
                    if let Ok(info) = serde_json::from_str::<UpdateInfo>(json_str) {
                        return Ok(info);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let script_cmd = if std::path::Path::new("install.sh").exists() {
            "./install.sh --check-update 2>/dev/null".to_string()
        } else {
            "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --check-update 2>/dev/null".to_string()
        };

        let output = tokio::process::Command::new("bash")
            .args(["-c", &script_cmd])
            .output()
            .await
            .map_err(|e| format!("Failed to run bash update check: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !stdout.is_empty() {
            if let Some(start_idx) = stdout.find('{') {
                if let Some(end_idx) = stdout.rfind('}') {
                    let json_str = &stdout[start_idx..=end_idx];
                    if let Ok(info) = serde_json::from_str::<UpdateInfo>(json_str) {
                        return Ok(info);
                    }
                }
            }
        }
    }

    logger::log_warn(
        "Script update check did not yield parseable result, falling back to check_for_updates",
    );
    check_for_updates().await
}

/// Run official installer to update the tool to the latest version
pub async fn run_installer_update() -> Result<String, String> {
    logger::log_info("Starting 3-stage delegated CLI updater for application update...");

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate current executable path: {}", e))?;
    let current_dir = current_exe
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let my_pid = std::process::id();
    let my_dir = current_dir.to_string_lossy().to_string();

    match crate::modules::delegate_updater::prepare_isolated_update_cli(my_pid) {
        Ok(temp_agm) => {
            logger::log_info(&format!(
                "Prepared isolated Update CLI copy at: {:?}",
                temp_agm
            ));

            match crate::modules::delegate_updater::spawn_delegated_update_cli(
                &temp_agm,
                my_pid,
                &current_dir,
                &current_exe,
                true,
                None,
            ) {
                Ok(child_pid) => {
                    logger::log_info(&format!(
                        "Launched delegated Update CLI process (PID: {}). Scheduling UI exit in 600ms...",
                        child_pid
                    ));
                    crate::modules::notification_hub::notify_system_updated(
                        CURRENT_VERSION,
                        "latest (delegated update CLI launched)",
                        Some("Delegated Update CLI (`agm-update-cli`) launched in isolated temp folder. It will run `agm update` and then `agm open-ui` to reopen the UI automatically."),
                    );

                    // Schedule unconditional clean exit after returning IPC response so tray_enabled prevent_exit() cannot hold file locks
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(650));
                        std::process::exit(0);
                    });

                    return Ok(
                        "Delegated Update CLI started (`agm-update-cli` -> `agm update` -> `agm open-ui`). Closing UI to apply update and reopen automatically."
                            .to_string(),
                    );
                }
                Err(e) => {
                    logger::log_error(&format!(
                        "Failed to spawn delegated Update CLI ({}), falling back to direct installer",
                        e
                    ));
                }
            }
        }
        Err(e) => {
            logger::log_warn(&format!(
                "Failed to prepare isolated Update CLI ({}), falling back to direct installer",
                e
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Fallback: On Windows, launch the installer in a visible, detached PowerShell process
        let dir_arg = format!("-InstallDir \"{}\"", my_dir);
        let ps_body = if current_dir.join("install.ps1").exists() {
            format!("Write-Host 'Updating Antigravity Tools...' -ForegroundColor Cyan; .\\install.ps1 -Update -NoLaunch {}", dir_arg)
        } else {
            format!("Write-Host 'Updating Antigravity Tools from official release...' -ForegroundColor Cyan; $s = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($s)) -Update -NoLaunch {}", dir_arg)
        };

        let mut spawn_cmd = std::process::Command::new("cmd.exe");
        spawn_cmd.args([
            "/c",
            "start",
            "Antigravity Tools Updater",
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_body,
        ]);

        match spawn_cmd.spawn() {
            Ok(_) => {
                logger::log_info("Successfully launched visible detached installer process.");
                crate::modules::notification_hub::notify_system_updated(
                    CURRENT_VERSION,
                    "latest (installer launched)",
                    Some("Official Windows installer update launched. Package download and binary replacement in progress."),
                );
                Ok(
                    "Official installer launched successfully! Check the update console window."
                        .to_string(),
                )
            }
            Err(e) => {
                logger::log_error(&format!("Failed to spawn updater process: {}", e));
                Err(format!("Failed to spawn updater: {}", e))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = if std::path::Path::new("install.sh").exists() {
            "bash ./install.sh --update".to_string()
        } else {
            "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --update".to_string()
        };

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tokio::process::Command::new("bash")
                .args(["-c", &cmd])
                .output(),
        )
        .await;

        let output = match result {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => return Err(format!("Failed to execute installer: {}", e)),
            Err(_) => return Err("Installer timed out after 5 minutes".to_string()),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            logger::log_info(&format!("Installer update succeeded: {}", stdout));
            crate::modules::notification_hub::notify_system_updated(
                CURRENT_VERSION,
                "latest",
                Some("Installer update script executed successfully."),
            );
            Ok(stdout)
        } else {
            let error_detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else if !stdout.trim().is_empty() {
                let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
                lines
                    .iter()
                    .rev()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                format!("Process exited with status code {:?}", output.status.code())
            };

            logger::log_error(&format!(
                "Installer update failed - stdout: {} stderr: {}",
                stdout, stderr
            ));
            Err(format!("Installer update failed: {}", error_detail))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
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
    fn test_should_check_for_updates() {
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
    fn update_settings_deserialize_missing_auto_check_and_last_check_time() {
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
