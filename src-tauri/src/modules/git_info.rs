//! Git metadata extraction and telemetry parity (version, commit hash, branch, last release)

use std::process::Command;

/// Returns the application package version
pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the short Git commit hash
pub fn get_git_hash() -> String {
    if let Some(h) = option_env!("AGM_GIT_HASH") {
        let trimmed = h.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    // Runtime fallback if binary runs inside a git repository
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Returns the active Git branch name
pub fn get_git_branch() -> String {
    if let Some(b) = option_env!("AGM_GIT_BRANCH") {
        let trimmed = b.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string())
}

/// Returns the most recent release tag (e.g., v4.78.0)
pub fn get_last_release() -> String {
    if let Some(r) = option_env!("AGM_LAST_RELEASE") {
        let trimmed = r.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")))
}

/// Returns the full 40-character Git commit hash
pub fn get_git_full_hash() -> String {
    if let Some(h) = option_env!("AGM_GIT_FULL_HASH") {
        let trimmed = h.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(get_git_hash)
}

/// Returns the official repository Git URL
pub fn get_repo_url() -> &'static str {
    "https://github.com/alimtvnetwork/Antigravity-Manager"
}

/// Returns the build timestamp or current compilation time
pub fn get_built_timestamp() -> String {
    if let Some(b) = option_env!("AGM_BUILD_TIME") {
        let trimmed = b.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    chrono::Utc::now().to_rfc3339()
}

/// Formatted one-line GitMap-style banner metadata
pub fn get_gitmap_style_summary() -> String {
    format!(
        "v{} (commit: {}, branch: {}, last release: {})",
        get_app_version(),
        get_git_hash(),
        get_git_branch(),
        get_last_release()
    )
}
