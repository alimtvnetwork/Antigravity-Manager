//! Release asset resolution (plan 86, subtask 002).
//!
//! Walks releases newest-first and selects the first one that is
//! installable on this platform, recording every skipped release with its
//! reason so the user can be told which versions were skipped and why.

use super::update_types::SkippedVersion;
use super::*;

/// Platform asset view of one release, decoupled from the GitHub DTO so the
/// resolution logic is unit-testable with fixtures.
#[derive(Debug, Clone)]
pub struct ReleaseAssetInfo {
    pub version: String,
    pub html_url: String,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub is_prerelease: bool,
    pub is_draft: bool,
    pub assets: Vec<GitHubReleaseAsset>,
}

impl From<&GitHubRelease> for ReleaseAssetInfo {
    fn from(release: &GitHubRelease) -> Self {
        Self {
            version: release.tag_name.trim_start_matches('v').to_string(),
            html_url: release.html_url.clone(),
            body: release.body.clone(),
            published_at: release.published_at.clone(),
            is_prerelease: release.prerelease,
            is_draft: release.draft,
            assets: release.assets.clone(),
        }
    }
}

/// Outcome of resolving a release list.
#[derive(Debug, Clone, Default)]
pub struct ReleaseResolution {
    /// The newest installable release, when one exists.
    pub selected: Option<ReleaseAssetInfo>,
    /// Download URL of the platform asset on the selected release.
    pub selected_asset_url: Option<String>,
    /// Versions of all installable releases, newest first.
    pub candidates: Vec<String>,
    /// Releases that were skipped, with reasons, newest first.
    pub skipped: Vec<SkippedVersion>,
}

/// True when an asset file name is an installer for the current platform.
/// The suffix table lives here only (plan 86, subtask 002).
pub fn is_platform_asset_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    #[cfg(target_os = "windows")]
    {
        name.ends_with(".exe") || name.ends_with(".msi")
    }
    #[cfg(target_os = "macos")]
    {
        name.ends_with(".dmg") || name.ends_with(".app.tar.gz") || name.ends_with(".pkg")
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        name.ends_with(".appimage") || name.ends_with(".deb") || name.ends_with(".rpm")
    }
}

/// The platform asset of a release, when one exists with a usable URL.
pub fn platform_asset(assets: &[GitHubReleaseAsset]) -> Option<&GitHubReleaseAsset> {
    assets.iter().find(|asset| {
        !asset.browser_download_url.trim().is_empty() && is_platform_asset_name(&asset.name)
    })
}

/// Resolve the newest installable release from a newest-first list.
/// Drafts are always skipped; prereleases are skipped on the stable
/// channel. Releases without a platform asset are skipped with a reason.
pub fn resolve_installable_release(
    releases: &[ReleaseAssetInfo],
    channel: UpdateChannel,
) -> ReleaseResolution {
    let mut resolution = ReleaseResolution::default();
    for release in releases {
        if release.is_draft {
            resolution.skipped.push(SkippedVersion {
                version: release.version.clone(),
                reason: "draft release".to_string(),
            });
            continue;
        }
        if release.is_prerelease && channel == UpdateChannel::Stable {
            resolution.skipped.push(SkippedVersion {
                version: release.version.clone(),
                reason: "pre-release on the stable channel".to_string(),
            });
            continue;
        }
        match platform_asset(&release.assets) {
            Some(asset) => {
                resolution.candidates.push(release.version.clone());
                if resolution.selected.is_none() {
                    resolution.selected_asset_url = Some(asset.browser_download_url.clone());
                    resolution.selected = Some(release.clone());
                }
            }
            None => {
                resolution.skipped.push(SkippedVersion {
                    version: release.version.clone(),
                    reason: "no installer asset for this platform".to_string(),
                });
            }
        }
    }
    resolution
}
