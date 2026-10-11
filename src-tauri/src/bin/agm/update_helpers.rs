//! update_helpers — CLI command handlers, split from agm.rs.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn resolve_gitmap_bin() -> Option<PathBuf> {
    if let Some(env_bin) = env::var_os("GITMAP_BIN") {
        let pb = PathBuf::from(env_bin);
        if pb.exists() {
            return Some(pb);
        }
    }
    if let Ok(out) = Command::new("gitmap").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("gitmap"));
        }
    }

    let bin_name = if cfg!(target_os = "windows") {
        "gitmap.exe"
    } else {
        "gitmap"
    };

    for rel in &[
        PathBuf::from(".").join(bin_name),
        PathBuf::from("..").join("gitmap").join(bin_name),
    ] {
        if rel.exists() {
            return Some(rel.clone());
        }
    }

    if let Ok(curr_exe) = env::current_exe() {
        if let Some(parent) = curr_exe.parent() {
            for cand in &[
                parent.join(bin_name),
                parent.join("..").join("gitmap").join(bin_name),
            ] {
                if cand.exists() {
                    return Some(cand.clone());
                }
            }
        }
    }

    if let Some(cargo_home) = env::var_os("CARGO_HOME") {
        let cand = PathBuf::from(cargo_home).join("bin").join(bin_name);
        if cand.exists() {
            return Some(cand);
        }
    }

    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let real_local =
            antigravity_tools_lib::modules::delegate_updater::strip_instance_sandbox_local_appdata(
                Path::new(&local_app_data),
            );
        for cand in &[
            real_local.join("gitmap-cli").join(bin_name),
            real_local.join("Programs").join("gitmap").join(bin_name),
        ] {
            if cand.exists() {
                return Some(cand.clone());
            }
        }
    }

    let raw_home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .ok()
        .or_else(|| dirs::home_dir().map(|h| h.to_string_lossy().to_string()));
    if let Some(home_str) = raw_home {
        let real_home = PathBuf::from(
            antigravity_tools_lib::modules::delegate_updater::unwrap_sandbox_path(&home_str),
        );
        for cand in &[
            real_home.join(".cargo").join("bin").join(bin_name),
            real_home.join(".local").join("bin").join(bin_name),
            real_home
                .join("AppData")
                .join("Local")
                .join("gitmap-cli")
                .join(bin_name),
            real_home.join("work").join("gitmap").join(bin_name),
            real_home.join("git-work").join("gitmap").join(bin_name),
        ] {
            if cand.exists() {
                return Some(cand.clone());
            }
        }
    }

    None
}

pub(crate) fn forward_to_gitmap_ssh(subargs: &[String]) -> bool {
    if let Some(gitmap_bin) = resolve_gitmap_bin() {
        let mut cmd = Command::new(gitmap_bin);
        cmd.arg("ssh");
        for a in subargs {
            cmd.arg(a);
        }
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        match cmd.status() {
            Ok(status) => {
                if !status.success() {
                    let code = status.code().unwrap_or(1);
                    std::process::exit(code);
                }
                return true;
            }
            Err(e) => {
                eprintln!("[WARN] Failed to spawn gitmap: {}", e);
            }
        }
    }
    false
}

pub(crate) fn default_update_export_path() -> std::path::PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let real_home = antigravity_tools_lib::modules::delegate_updater::unwrap_sandbox_path(&home);
    std::path::PathBuf::from(real_home)
        .join(".antigravity_tools")
        .join("update-export")
        .join("agm-update.zip")
}

pub(crate) fn download_release_zip(
    dest: &std::path::Path,
    target_version: Option<&str>,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("antigravity-manager")
        .build()
        .map_err(|err| err.to_string())?;

    let mut api_urls = Vec::new();
    let mut direct_zip_urls = Vec::new();
    if let Some(ver) = target_version {
        let clean = crate::update_cmds::normalize_version_str(ver);
        api_urls.push(format!(
            "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/v{}",
            clean
        ));
        if cfg!(windows) {
            direct_zip_urls.push(format!(
                "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{0}/agm-alim_{0}_windows_x64.zip",
                clean
            ));
        }
        if let Some(short_ver) = clean.strip_suffix(".0") {
            if short_ver.contains('.') {
                api_urls.push(format!(
                    "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/v{}",
                    short_ver
                ));
                if cfg!(windows) {
                    direct_zip_urls.push(format!(
                        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{0}/agm-alim_{0}_windows_x64.zip",
                        short_ver
                    ));
                }
            }
        }
    } else {
        api_urls.push(
            "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest"
                .to_string(),
        );
    }

    let mut resolved_download_url: Option<String> = None;
    let mut last_err = String::new();

    for release_url in &api_urls {
        match client
            .get(release_url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .and_then(|r| r.error_for_status())
        {
            Ok(resp) => {
                if let Ok(release) = resp.json::<serde_json::Value>() {
                    if let Some(assets) = release.get("assets").and_then(|item| item.as_array()) {
                        if let Some(asset) = assets
                            .iter()
                            .filter(|item| {
                                release_zip_matches_platform(
                                    item.get("name")
                                        .and_then(|name| name.as_str())
                                        .unwrap_or(""),
                                )
                            })
                            .max_by_key(|item| {
                                let name = item
                                    .get("name")
                                    .and_then(|name| name.as_str())
                                    .unwrap_or("");
                                usize::from(
                                    name.to_lowercase().contains("x64")
                                        || name.to_lowercase().contains("amd64"),
                                )
                            })
                        {
                            if let Some(u) = asset
                                .get("browser_download_url")
                                .and_then(|item| item.as_str())
                            {
                                resolved_download_url = Some(u.to_string());
                                break;
                            }
                        } else {
                            last_err = "no zip asset for this platform".to_string();
                        }
                    } else {
                        last_err = "release has no assets".to_string();
                    }
                }
            }
            Err(err) => {
                last_err = err.to_string();
            }
        }
    }

    let mut candidate_download_urls = Vec::new();
    if let Some(primary) = resolved_download_url {
        candidate_download_urls.push(primary);
    }
    for fallback_url in direct_zip_urls {
        if !candidate_download_urls.contains(&fallback_url) {
            candidate_download_urls.push(fallback_url);
        }
    }

    if candidate_download_urls.is_empty() {
        return Err(if last_err.is_empty() {
            "no zip asset for this platform".to_string()
        } else {
            last_err
        });
    }

    let mut downloaded_bytes = None;
    for url in &candidate_download_urls {
        match client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes())
        {
            Ok(b) if !b.is_empty() => {
                downloaded_bytes = Some(b);
                break;
            }
            Ok(_) => {
                last_err = format!("empty response from {}", url);
            }
            Err(err) => {
                last_err = err.to_string();
            }
        }
    }

    let bytes = downloaded_bytes.ok_or(last_err)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(dest, &bytes).map_err(|err| err.to_string())?;
    Ok(())
}

pub(crate) fn release_zip_matches_platform(name: &str) -> bool {
    let lower = name.to_lowercase();
    if !lower.ends_with(".zip") {
        return false;
    }
    #[cfg(windows)]
    {
        lower.contains("windows") || lower.contains("win")
    }
    #[cfg(target_os = "macos")]
    {
        lower.contains("darwin") || lower.contains("macos") || lower.contains("osx")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        lower.contains("linux")
    }
}

pub(crate) fn register_powershell_profile_function(exe_path: &Path) {
    let script = format!(
        "$profilePath = $PROFILE; \
         if ($profilePath -and (Test-Path -Path $profilePath)) {{ \
             $content = Get-Content -LiteralPath $profilePath -Raw; \
             if ($content -notmatch 'function agm\\b') {{ \
                 $entry = \"`n# agm & adm command wrappers`nfunction agm {{ & '{exe}' @args }}`nfunction adm {{ & '{exe}' @args }}`n\"; \
                 Add-Content -LiteralPath $profilePath -Value $entry; \
             }} \
         }}",
        exe = exe_path.to_string_lossy().replace('\'', "''")
    );
    // Justification: best-effort PowerShell profile wrapper registration; non-fatal if powershell is unavailable
    antigravity_tools_lib::error::record_ignored(
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output(),
        "register_powershell_profile_function",
    );
}

pub(crate) fn cmd_update_export_zip(args: &[String]) {
    let mut output: Option<std::path::PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "-o" || args[index] == "--output" {
            if let Some(next) = args.get(index + 1) {
                output = Some(std::path::PathBuf::from(next));
                index += 2;
                continue;
            }
        }
        index += 1;
    }
    let dest = output.unwrap_or_else(default_update_export_path);
    let target_version = crate::update_cmds::extract_target_version(args);
    match download_release_zip(&dest, target_version.as_deref()) {
        Ok(()) => {
            println!("EXPORT_ZIP={}", dest.display());
            println!(
                "[OK] Release zip saved. This command does not install or replace the running app."
            );
        }
        Err(err) => {
            eprintln!("[FAIL] export-zip: {}", err);
            std::process::exit(1);
        }
    }
}
