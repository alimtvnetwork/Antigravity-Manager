use crate::modules::logger;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::*;

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
pub(crate) struct UpdaterJson {
    pub(crate) version: String,
    pub(crate) notes: Option<String>,
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
pub(crate) struct PackageJson {
    pub(crate) version: String,
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
