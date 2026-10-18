use std::env;
use std::path::{Path, PathBuf};

use super::*;

/// Normalize version string: e.g., "4.162" or "v4.162" -> "4.162.0",
/// stripping any leading URL prefixes if present.
pub fn normalize_version_string(raw: &str) -> String {
    let clean = raw.trim().trim_matches(['\'', '"']);
    let stripped_url = if let Some(pos) = clean.find("/releases/tag/") {
        let after = &clean[pos + "/releases/tag/".len()..];
        after.split(['?', '#', '/']).next().unwrap_or(after)
    } else if let Some(pos) = clean.find("/releases/download/") {
        let after = &clean[pos + "/releases/download/".len()..];
        after.split(['?', '#', '/']).next().unwrap_or(after)
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        let path_only = clean.split(['?', '#']).next().unwrap_or(clean);
        path_only.rsplit('/').next().unwrap_or(clean)
    } else {
        clean
    };

    let clean = stripped_url
        .trim()
        .trim_matches(['\'', '"'])
        .trim_start_matches(['v', 'V'])
        .trim();
    let base = clean.split('-').next().unwrap_or(clean);
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() == 2
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        if let Some(suffix_idx) = clean.find('-') {
            format!("{}.0{}", &clean[..suffix_idx], &clean[suffix_idx..])
        } else {
            format!("{}.0", clean)
        }
    } else {
        clean.to_string()
    }
}

/// Extract and normalize version from a token that may be a plain version string ("4.226.3", "v4.226.3")
/// or a full GitHub release URL (e.g. "https://github.com/.../releases/tag/v4.226.3").
pub fn extract_version_from_token(token: &str) -> Option<String> {
    let clean = token.trim().trim_matches(['\'', '"']);
    if clean.is_empty() {
        return None;
    }

    if let Some(pos) = clean.find("/releases/tag/") {
        let after_tag = &clean[pos + "/releases/tag/".len()..];
        let tag_part = after_tag.split(['?', '#', '/']).next().unwrap_or(after_tag);
        let tag_clean = tag_part.trim().trim_start_matches(['v', 'V']).trim();
        if !tag_clean.is_empty()
            && tag_clean
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(normalize_version_string(tag_clean));
        }
    }

    if let Some(pos) = clean.find("/releases/download/") {
        let after_dl = &clean[pos + "/releases/download/".len()..];
        let tag_part = after_dl.split(['?', '#', '/']).next().unwrap_or(after_dl);
        let tag_clean = tag_part.trim().trim_start_matches(['v', 'V']).trim();
        if !tag_clean.is_empty()
            && tag_clean
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(normalize_version_string(tag_clean));
        }
    }

    if clean.starts_with("http://") || clean.starts_with("https://") {
        let path_only = clean.split(['?', '#']).next().unwrap_or(clean);
        if let Some(last_seg) = path_only.rsplit('/').next() {
            let seg_clean = last_seg.trim().trim_start_matches(['v', 'V']).trim();
            if !seg_clean.is_empty()
                && seg_clean
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && seg_clean.contains('.')
            {
                return Some(normalize_version_string(seg_clean));
            }
        }
    }

    let v = clean.trim_start_matches(['v', 'V']).trim();
    if v.contains('.')
        && v.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        return Some(normalize_version_string(v));
    }

    None
}

/// Unwrap an Antigravity multi-instance sandbox path back to the real host user profile root.
/// If `raw_path` contains `.antigravity_tools/instances/` or `.antigravity_tools\instances\`,
/// returns the host user profile prefix before that marker; otherwise returns `raw_path` unchanged.
pub fn unwrap_sandbox_path(raw_path: &str) -> String {
    let normalized = raw_path.replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    let marker = "\\.antigravity_tools\\instances\\";
    if let Some(idx) = lower.find(marker) {
        let prefix = &raw_path[..idx];
        if !prefix.is_empty() {
            return prefix.to_string();
        }
    }
    let alt_marker = ".antigravity_tools\\instances\\";
    if let Some(idx) = lower.find(alt_marker) {
        if idx > 0 {
            let prefix = raw_path[..idx].trim_end_matches(['\\', '/']);
            if !prefix.is_empty() {
                return prefix.to_string();
            }
        }
    }
    raw_path.to_string()
}

/// Strip `.antigravity_tools/instances/<id>/...` sandbox redirection from a `LOCALAPPDATA` or profile path,
/// returning `<real_user_root>/AppData/Local` when sandboxed or `raw_path` unchanged otherwise.
pub fn strip_instance_sandbox_local_appdata(raw_path: &Path) -> PathBuf {
    let raw_str = raw_path.to_string_lossy();
    let unwrapped = unwrap_sandbox_path(&raw_str);
    if unwrapped.len() != raw_str.len() {
        return PathBuf::from(unwrapped).join("AppData").join("Local");
    }
    raw_path.to_path_buf()
}

pub fn is_instance_sandbox_path(path: &Path) -> bool {
    let raw_str = path.to_string_lossy();
    unwrap_sandbox_path(&raw_str).len() != raw_str.len()
}

pub fn parse_delegate_args(args: &[String]) -> DelegateUpdateOptions {
    let mut opts = DelegateUpdateOptions {
        wait_pid: None,
        is_relaunch: true,
        target_exe: None,
        install_dir: None,
        target_version: None,
        is_delegated_worker: false,
        is_pinned: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--wait-pid" => {
                if i + 1 < args.len() {
                    opts.wait_pid = args[i + 1].parse::<u32>().ok();
                    i += 1;
                }
            }
            "--relaunch" => {
                opts.is_relaunch = true;
            }
            "--no-relaunch" | "--no-launch" => {
                opts.is_relaunch = false;
            }
            "--target-exe" => {
                if i + 1 < args.len() {
                    opts.target_exe = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--install-dir" => {
                if i + 1 < args.len() {
                    opts.install_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--version" | "-v" => {
                if i + 1 < args.len() {
                    if let Some(v) = extract_version_from_token(&args[i + 1]) {
                        opts.target_version = Some(v);
                    } else {
                        opts.target_version = Some(normalize_version_string(&args[i + 1]));
                    }
                    i += 1;
                }
            }
            "--delegated-worker" => {
                opts.is_delegated_worker = true;
            }
            "-isPinned" | "--is-pinned" | "-ispinned" => {
                opts.is_pinned = true;
            }
            arg if arg.starts_with("--version=") || arg.starts_with("-v=") => {
                let prefix = if arg.starts_with("--version=") {
                    "--version="
                } else {
                    "-v="
                };
                let raw_val = arg.trim_start_matches(prefix);
                if let Some(v) = extract_version_from_token(raw_val) {
                    opts.target_version = Some(v);
                } else {
                    opts.target_version = Some(normalize_version_string(raw_val));
                }
            }
            arg if arg.starts_with("--to=")
                || arg.starts_with("-to=")
                || arg.starts_with("--revert=")
                || arg.starts_with("-revert=")
                || arg.starts_with("--downgrade=")
                || arg.starts_with("-downgrade=") =>
            {
                let prefix_len = arg.find('=').map(|idx| idx + 1).unwrap_or(0);
                let raw_val = &arg[prefix_len..];
                if let Some(v) = extract_version_from_token(raw_val) {
                    opts.target_version = Some(v);
                } else if !raw_val.trim().is_empty() {
                    opts.target_version = Some(normalize_version_string(raw_val));
                }
            }
            arg if arg.eq_ignore_ascii_case("to")
                || arg.eq_ignore_ascii_case("--to")
                || arg.eq_ignore_ascii_case("-to")
                || arg.eq_ignore_ascii_case("revert")
                || arg.eq_ignore_ascii_case("--revert")
                || arg.eq_ignore_ascii_case("-revert")
                || arg.eq_ignore_ascii_case("downgrade")
                || arg.eq_ignore_ascii_case("--downgrade")
                || arg.eq_ignore_ascii_case("-downgrade") =>
            {
                if i + 1 < args.len() {
                    if let Some(v) = extract_version_from_token(&args[i + 1]) {
                        opts.target_version = Some(v);
                        i += 1;
                    } else {
                        let v = args[i + 1]
                            .trim()
                            .trim_matches(['\'', '"'])
                            .trim_start_matches(['v', 'V']);
                        if !v.is_empty()
                            && v.chars()
                                .next()
                                .map(|c| c.is_ascii_digit())
                                .unwrap_or(false)
                        {
                            opts.target_version = Some(normalize_version_string(v));
                            i += 1;
                        }
                    }
                }
            }
            _ => {
                if opts.target_version.is_none() && !args[i].starts_with('-') {
                    if let Some(v) = extract_version_from_token(&args[i]) {
                        opts.target_version = Some(v);
                    }
                }
            }
        }
        i += 1;
    }

    if opts.target_version.is_some() {
        opts.is_pinned = true;
    }

    opts
}

pub fn parse_update_args(args: &[String]) -> DelegateUpdateOptions {
    parse_delegate_args(args)
}

pub fn resolve_default_install_dir(
    install_dir: Option<PathBuf>,
    target_exe: Option<&PathBuf>,
) -> PathBuf {
    install_dir
        .map(|dir| {
            if is_instance_sandbox_path(&dir) {
                strip_instance_sandbox_local_appdata(&dir)
                    .join("Programs")
                    .join("agm-alim")
            } else {
                dir
            }
        })
        .or_else(|| {
            target_exe
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .filter(|d| {
                    !d.to_string_lossy().to_lowercase().contains("agm-updater")
                        && !is_instance_sandbox_path(d)
                })
        })
        .unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            {
                if let Ok(local) = env::var("LOCALAPPDATA") {
                    let real_local = strip_instance_sandbox_local_appdata(Path::new(&local));
                    let cand1 = real_local.join("Programs").join("agm-alim");
                    if cand1.exists() {
                        return cand1;
                    }
                    let cand2 = real_local.join("Programs").join("Antigravity-Tools");
                    if cand2.exists() {
                        return cand2;
                    }
                    cand1
                } else if let Ok(profile) = env::var("USERPROFILE") {
                    let real_local = strip_instance_sandbox_local_appdata(Path::new(&profile));
                    real_local.join("Programs").join("agm-alim")
                } else {
                    PathBuf::from(".")
                }
            }
            #[cfg(target_os = "macos")]
            {
                let app_dir = PathBuf::from("/Applications");
                if app_dir.exists() {
                    app_dir
                } else {
                    dirs::home_dir()
                        .map(|h| h.join("Applications"))
                        .unwrap_or_else(|| PathBuf::from("/Applications"))
                }
            }
            #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
            {
                dirs::home_dir()
                    .map(|h| h.join(".local/bin"))
                    .unwrap_or_else(|| PathBuf::from("."))
            }
        })
}

pub fn resolve_default_target_exe(
    target_exe: Option<PathBuf>,
    resolved_install_dir: &Path,
) -> PathBuf {
    if let Some(exe) = target_exe {
        let exe_str = exe.to_string_lossy().to_lowercase();
        if !exe_str.contains("agm-update-cli")
            && !exe_str.contains("agm-updater")
            && !exe_str.contains(".trash")
            && !is_instance_sandbox_path(&exe)
        {
            return exe;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let exe_cand = resolved_install_dir.join("agm-alim.exe");
        if exe_cand.exists() {
            exe_cand
        } else {
            resolved_install_dir.join("agm-alim.exe")
        }
    }
    #[cfg(target_os = "macos")]
    {
        let candidates = [
            PathBuf::from("/Applications/Antigravity Manager Tools.app"),
            dirs::home_dir()
                .map(|h| h.join("Applications/Antigravity Manager Tools.app"))
                .unwrap_or_default(),
            PathBuf::from("/Applications/Antigravity Tools.app"),
            PathBuf::from("/Applications/agm-alim.app"),
        ];
        for cand in &candidates {
            if cand.exists() && cand.is_dir() {
                let cand_str = cand.to_string_lossy().to_lowercase();
                if !cand_str.contains(".trash") {
                    return cand.clone();
                }
            }
        }
        if let Ok(curr_exe) = env::current_exe() {
            let mut ancestor = curr_exe.parent();
            while let Some(parent) = ancestor {
                if parent.extension().and_then(|s| s.to_str()) == Some("app") {
                    let parent_str = parent.to_string_lossy().to_lowercase();
                    if !parent_str.contains(".trash") {
                        return parent.to_path_buf();
                    }
                }
                ancestor = parent.parent();
            }
        }
        PathBuf::from("/Applications/Antigravity Manager Tools.app")
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        resolved_install_dir.join("agm-alim")
    }
}
