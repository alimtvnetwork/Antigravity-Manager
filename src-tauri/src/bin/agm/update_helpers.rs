//! update_helpers — CLI command handlers, split from agm.rs.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn resolve_gitmap_bin() -> Option<PathBuf> {
    if let Ok(out) = Command::new("gitmap").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("gitmap"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        for candidate in &[
            PathBuf::from(r"d:\work\gitmap\gitmap.exe"),
            PathBuf::from(r"C:\gitmap\gitmap.exe"),
            PathBuf::from(r".\gitmap.exe"),
        ] {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join(r"AppData\Local\gitmap-cli\gitmap.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for p in &["/usr/local/bin/gitmap", "/usr/bin/gitmap"] {
            let pb = PathBuf::from(p);
            if pb.exists() {
                return Some(pb);
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
    std::path::PathBuf::from(home)
        .join(".antigravity_tools")
        .join("update-export")
        .join("agm-update.zip")
}

pub(crate) fn download_release_zip(dest: &std::path::Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("antigravity-manager")
        .build()
        .map_err(|err| err.to_string())?;
    let release: serde_json::Value = client
        .get("https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?
        .json()
        .map_err(|err| err.to_string())?;
    let assets = release
        .get("assets")
        .and_then(|item| item.as_array())
        .ok_or_else(|| "latest release has no assets".to_string())?;
    let asset = assets
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
                name.to_lowercase().contains("x64") || name.to_lowercase().contains("amd64"),
            )
        })
        .ok_or_else(|| "no zip asset for this platform".to_string())?;
    let url = asset
        .get("browser_download_url")
        .and_then(|item| item.as_str())
        .ok_or_else(|| "release asset has no download url".to_string())?;
    let bytes = client
        .get(url)
        .send()
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?
        .bytes()
        .map_err(|err| err.to_string())?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(dest, &bytes).map_err(|err| err.to_string())?;
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
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
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
    match download_release_zip(&dest) {
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
