use std::env;
use std::path::{Path, PathBuf};

use super::*;

pub fn parse_delegate_args(args: &[String]) -> DelegateUpdateOptions {
    let mut opts = DelegateUpdateOptions {
        wait_pid: None,
        is_relaunch: true,
        target_exe: None,
        install_dir: None,
        target_version: None,
        is_delegated_worker: false,
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
            "--version" => {
                if i + 1 < args.len() {
                    opts.target_version = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--delegated-worker" => {
                opts.is_delegated_worker = true;
            }
            _ => {}
        }
        i += 1;
    }

    opts
}

pub fn resolve_default_install_dir(
    install_dir: Option<PathBuf>,
    target_exe: Option<&PathBuf>,
) -> PathBuf {
    install_dir
        .or_else(|| {
            target_exe
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .filter(|d| !d.to_string_lossy().to_lowercase().contains("agm-updater"))
        })
        .unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            {
                if let Ok(local) = env::var("LOCALAPPDATA") {
                    let cand1 = PathBuf::from(&local).join("Programs").join("agm-alim");
                    if cand1.exists() {
                        return cand1;
                    }
                    let cand2 = PathBuf::from(&local)
                        .join("Programs")
                        .join("Antigravity-Tools");
                    if cand2.exists() {
                        return cand2;
                    }
                    cand1
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
                    .unwrap_or_else(|| PathBuf::from("/usr/local/bin"))
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
