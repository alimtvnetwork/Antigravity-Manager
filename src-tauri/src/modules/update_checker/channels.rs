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

    // Tip: If the upstream proxy is not explicitly configured, try to get the proxy from the system environment variable (HTTPS_PROXY / HTTP_PROXY / ALL_PROXY)
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
    #[serde(default)]
    pub(crate) platforms: Option<serde_json::Value>,
}

pub(crate) fn has_platform_asset_in_release(assets: &[GitHubReleaseAsset]) -> bool {
    platform_asset(assets).is_some()
}

/// Download URL of this platform's asset inside an updater.json `platforms`
/// map, when present (plan 86).
fn platform_asset_url_in_updater_json(platforms: &Option<serde_json::Value>) -> Option<String> {
    let Some(serde_json::Value::Object(map)) = platforms else {
        return None;
    };
    for (key, val) in map {
        let k = key.to_ascii_lowercase();
        let os_match = if cfg!(target_os = "windows") {
            k.contains("windows") || k.contains("win")
        } else if cfg!(target_os = "macos") {
            k.contains("darwin") || k.contains("macos") || k.contains("osx")
        } else {
            k.contains("linux")
        };
        if !os_match {
            continue;
        }
        if let Some(url) = val.get("url").and_then(|u| u.as_str()) {
            let lower = url.trim().to_ascii_lowercase();
            let suffix_ok = if cfg!(target_os = "windows") {
                lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".nsis.zip")
            } else if cfg!(target_os = "macos") {
                lower.ends_with(".dmg") || lower.ends_with(".app.tar.gz") || lower.ends_with(".pkg")
            } else {
                lower.ends_with(".appimage")
                    || lower.ends_with(".appimage.tar.gz")
                    || lower.ends_with(".deb")
                    || lower.ends_with(".rpm")
            };
            if suffix_ok {
                return Some(url.trim().to_string());
            }
        }
    }
    None
}

pub(crate) async fn verify_tag_has_platform_asset(version: &str) -> bool {
    let Ok(client) = create_client().await else {
        return false;
    };
    let clean = version.trim().trim_start_matches('v');
    let tag_url = format!(
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/v{}",
        clean
    );
    let Ok(resp) = client.get(&tag_url).send().await else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    let Ok(release) = resp.json::<GitHubRelease>().await else {
        return false;
    };
    has_platform_asset_in_release(&release.assets)
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
    let is_newer = compare_versions(&latest_version, &current_version);
    let has_asset = if is_newer {
        verify_tag_has_platform_asset(&latest_version).await
    } else {
        false
    };
    let has_update = is_newer && has_asset;
    // Skipped-version reporting (plan 86): a newer version without a
    // platform asset is reported, never silently dropped.
    let skipped_versions = if is_newer && !has_asset {
        vec![SkippedVersion {
            version: latest_version.clone(),
            reason: "no installer asset for this platform".to_string(),
        }]
    } else {
        Vec::new()
    };
    let asset_url = if has_update {
        platform_asset_url_in_updater_json(&updater_info.platforms)
    } else {
        None
    };
    let candidates = if has_update {
        vec![latest_version.clone()]
    } else {
        Vec::new()
    };

    if has_update {
        logger::log_info(&format!(
            "New version found ({:?} updater.json, platform asset verified): {} (Current: {})",
            channel, latest_version, current_version
        ));
    } else if is_newer {
        logger::log_warn(&format!(
            "Skipping version {} ({:?} updater.json): no installer asset found for this OS yet",
            latest_version, channel
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
        asset_verified: has_update,
        asset_url,
        candidates,
        skipped_versions,
        source_errors: Vec::new(),
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

    if releases.is_empty() {
        return Err("No releases found on GitHub".to_string());
    }

    // Resolve the newest installable release (plan 86, subtask 002): newer
    // releases without a platform asset are skipped with reasons and an
    // older verified release is offered instead.
    let infos: Vec<ReleaseAssetInfo> = releases.iter().map(ReleaseAssetInfo::from).collect();
    let resolution = resolve_installable_release(&infos, channel);
    let newest_version = infos.first().map(|r| r.version.clone()).unwrap_or_default();
    let current_version = CURRENT_VERSION.to_string();

    let selected = resolution.selected.clone();
    let latest_version = selected
        .as_ref()
        .map(|r| r.version.clone())
        .unwrap_or_else(|| newest_version.clone());
    let is_newer = compare_versions(&latest_version, &current_version);
    let has_update = selected.is_some() && is_newer;

    // Only report skips for versions newer than the running one.
    let skipped_versions: Vec<SkippedVersion> = resolution
        .skipped
        .iter()
        .filter(|s| compare_versions(&s.version, &current_version))
        .cloned()
        .collect();
    for skipped in &skipped_versions {
        logger::log_warn(&format!(
            "Skipping release {} (API {:?}): {}",
            skipped.version, channel, skipped.reason
        ));
    }

    if has_update {
        logger::log_info(&format!(
            "New version found (API {:?}, platform asset verified): {} (Current: {})",
            channel, latest_version, current_version
        ));
    } else {
        logger::log_info(&format!(
            "Up to date (API {:?}): {} (Matches {})",
            channel, current_version, latest_version
        ));
    }

    let (download_url, release_notes, published_at) = match &selected {
        Some(sel) => (
            sel.html_url.clone(),
            sel.body.clone().unwrap_or_default(),
            sel.published_at
                .clone()
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
        ),
        None => (
            releases[0].html_url.clone(),
            releases[0].body.clone().unwrap_or_default(),
            releases[0]
                .published_at
                .clone()
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
        ),
    };

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        download_url,
        release_notes,
        published_at,
        source: Some(format!("GitHub API ({:?})", channel)),
        proxy_url: None,
        channel: Some(channel),
        updater_json_url: match channel {
            UpdateChannel::Beta => Some(PREVIEW_UPDATER_JSON_URL.to_string()),
            UpdateChannel::Stable => Some(STABLE_UPDATER_JSON_URL.to_string()),
        },
        asset_verified: selected.is_some(),
        asset_url: resolution.selected_asset_url.clone(),
        candidates: resolution.candidates.clone(),
        skipped_versions,
        source_errors: Vec::new(),
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
    let is_newer = compare_versions(&latest_version, &current_version);
    let has_asset = if is_newer {
        verify_tag_has_platform_asset(&latest_version).await
    } else {
        false
    };
    let has_update = is_newer && has_asset;
    // Skipped-version reporting (plan 86), same contract as the other sources.
    let skipped_versions = if is_newer && !has_asset {
        vec![SkippedVersion {
            version: latest_version.clone(),
            reason: "no installer asset for this platform".to_string(),
        }]
    } else {
        Vec::new()
    };
    let candidates = if has_update {
        vec![latest_version.clone()]
    } else {
        Vec::new()
    };

    if has_update {
        logger::log_info(&format!(
            "New version found ({}, platform asset verified): {} (Current: {})",
            source_name, latest_version, current_version
        ));
    } else if is_newer {
        logger::log_warn(&format!(
            "Skipping version {} ({}): no release asset found for this OS yet",
            latest_version, source_name
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
        asset_verified: has_update,
        asset_url: None,
        candidates,
        skipped_versions,
        source_errors: Vec::new(),
    })
}
