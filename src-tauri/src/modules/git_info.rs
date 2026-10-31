//! Git metadata extraction and telemetry parity (version, commit hash, branch, last release)

use std::process::Command;

/// Returns the application package version
pub fn get_app_version() -> &'static str {
    crate::VERSION
}

fn run_raw_git_command(args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Verify that the current directory's git repository is genuinely Antigravity-Manager,
/// preventing external workspace tags (e.g. v1.9.0) from polluting AGM version reporting.
fn is_agm_repo() -> bool {
    if let Some(toplevel) = run_raw_git_command(&["rev-parse", "--show-toplevel"]) {
        let p = std::path::Path::new(&toplevel);
        let cargo_toml = p.join("src-tauri").join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("agm-ui") || content.contains("antigravity_tools_lib") {
                    return true;
                }
            }
        }
        let root_cargo = p.join("Cargo.toml");
        if root_cargo.exists() {
            if let Ok(content) = std::fs::read_to_string(&root_cargo) {
                if content.contains("agm-ui") || content.contains("antigravity_tools_lib") {
                    return true;
                }
            }
        }
    }
    false
}

fn run_git_command(args: &[&str]) -> Option<String> {
    if !is_agm_repo() {
        return None;
    }
    run_raw_git_command(args)
}

/// Returns the short Git commit hash
pub fn get_git_hash() -> String {
    if let Some(h) = option_env!("AGM_GIT_HASH") {
        let trimmed = h.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    // Runtime fallback if binary runs inside the AGM git repository
    run_git_command(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "release".to_string())
}

/// Returns the active Git branch name
pub fn get_git_branch() -> String {
    if let Some(b) = option_env!("AGM_GIT_BRANCH") {
        let trimmed = b.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "main".to_string())
}

/// Returns the most recent release tag (e.g., v4.78.0)
pub fn get_last_release() -> String {
    if let Some(r) = option_env!("AGM_LAST_RELEASE") {
        let trimmed = r.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    run_git_command(&["describe", "--tags", "--abbrev=0"])
        .unwrap_or_else(|| format!("v{}", crate::VERSION))
}

/// Returns the full 40-character Git commit hash
pub fn get_git_full_hash() -> String {
    if let Some(h) = option_env!("AGM_GIT_FULL_HASH") {
        let trimmed = h.trim();
        if !trimmed.is_empty() && trimmed != "unknown" {
            return trimmed.to_string();
        }
    }
    run_git_command(&["rev-parse", "HEAD"]).unwrap_or_else(get_git_hash)
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
