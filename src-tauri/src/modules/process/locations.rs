use std::process::Command;

use super::*;

/// Helper to construct candidate search paths for macOS IDE discovery.
pub(crate) fn get_macos_candidate_paths(
    folder_name: &str,
    home_dir: Option<&std::path::Path>,
) -> Vec<String> {
    let mut candidates = vec![
        format!("/Applications/{}.app", folder_name),
        format!(
            "/Applications/{}.app/Contents/MacOS/{}",
            folder_name, folder_name
        ),
        format!(
            "/Applications/{}.app/Contents/MacOS/Antigravity",
            folder_name
        ),
        format!("/Applications/{}.app/Contents/MacOS/Electron", folder_name),
    ];

    let resolved_home: Option<std::path::PathBuf> = match home_dir {
        Some(h) => Some(h.to_path_buf()),
        None => dirs::home_dir(),
    };

    if let Some(home) = resolved_home {
        let home_str = home.to_string_lossy().replace('\\', "/");
        let home_clean = home_str.trim_end_matches('/');
        let user_app = format!("{}/Applications/{}.app", home_clean, folder_name);
        candidates.push(user_app.clone());
        candidates.push(format!("{}/Contents/MacOS/{}", user_app, folder_name));
        candidates.push(format!("{}/Contents/MacOS/Antigravity", user_app));
        candidates.push(format!("{}/Contents/MacOS/Electron", user_app));
    }

    candidates
}

/// Audit standard installation locations and collect diagnostics
pub(crate) fn audit_standard_locations(
    target_ide: Option<&str>,
) -> (Option<std::path::PathBuf>, Vec<String>) {
    let mut checked = Vec::new();

    let folder_names: &[&str] = if target_ide == Some("ide") {
        &["Antigravity IDE", "Antigravity", "antigravity"]
    } else if target_ide == Some("code") || target_ide == Some("cursor") {
        &["Antigravity", "antigravity"]
    } else if target_ide == Some("classic") {
        &["Antigravity", "antigravity"]
    } else {
        &["Antigravity", "antigravity", "Antigravity IDE"]
    };

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir();
        for folder_name in folder_names {
            let candidates = get_macos_candidate_paths(folder_name, home.as_deref());
            for c in candidates {
                checked.push(c.clone());
                if !c.to_lowercase().contains(".trash") {
                    let p = std::path::PathBuf::from(c);
                    if p.exists() {
                        return (Some(p), checked);
                    }
                }
            }
        }

        // Spotlight mdfind fallback for non-standard directory installations
        for folder_name in folder_names {
            let query = format!("kMDItemFSName == '{}.app'", folder_name);
            checked.push(format!("mdfind: {}", query));
            if let Ok(output) = std::process::Command::new("mdfind")
                .env("RUST_BACKTRACE", "1")
                .arg(&query)
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.to_lowercase().contains(".trash") {
                            let p = std::path::PathBuf::from(trimmed);
                            if p.exists() {
                                return (Some(p), checked);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let local_appdata = env::var("LOCALAPPDATA").ok();
        let program_files =
            env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 =
            env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

        for folder_name in folder_names {
            let mut candidates = Vec::new();

            if let Some(ref local) = local_appdata {
                candidates.push(
                    std::path::PathBuf::from(local)
                        .join("Programs")
                        .join(folder_name)
                        .join(format!("{}.exe", folder_name)),
                );
            }

            candidates.push(
                std::path::PathBuf::from(&program_files)
                    .join(folder_name)
                    .join(format!("{}.exe", folder_name)),
            );

            candidates.push(
                std::path::PathBuf::from(&program_files_x86)
                    .join(folder_name)
                    .join(format!("{}.exe", folder_name)),
            );

            for path in candidates {
                let path_str = path.to_string_lossy().to_string();
                checked.push(path_str);
                if path.exists() {
                    return (Some(path), checked);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Direct APPIMAGE environment check
        if let Ok(appimage) = std::env::var("APPIMAGE") {
            let path = std::path::PathBuf::from(&appimage);
            checked.push(format!("$APPIMAGE: {}", appimage));
            if path.is_file() {
                return (Some(path), checked);
            }
        }

        let exe_names: &[&str] = if target_ide == Some("ide") {
            &[
                "antigravity-ide",
                "Antigravity-IDE",
                "antigravity",
                "Antigravity",
            ]
        } else if target_ide == Some("cursor") {
            &["cursor", "antigravity", "Antigravity"]
        } else if target_ide == Some("code") {
            &["code", "antigravity", "Antigravity"]
        } else {
            &[
                "antigravity",
                "Antigravity",
                "antigravity-ide",
                "Antigravity-IDE",
                "google-antigravity",
                "Google-Antigravity",
                "agy",
            ]
        };

        // 2. PATH resolution
        if let Some(path) = resolve_linux_path_env(exe_names, &mut checked) {
            return (Some(path), checked);
        }

        // 3. Standard filesystem locations & AppImages
        for folder_name in folder_names {
            if let Some(path) = resolve_linux_standard_paths(folder_name, exe_names, &mut checked) {
                return (Some(path), checked);
            }
        }

        // 4. Desktop entry parsing (.desktop)
        if let Some(path) = resolve_linux_desktop_entry(exe_names, &mut checked) {
            return (Some(path), checked);
        }
    }

    (None, checked)
}

#[cfg(target_os = "linux")]
pub(crate) fn clean_desktop_exec_command(exec_cmd: &str) -> Option<String> {
    let trimmed = exec_cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Handle quotes: e.g. "/opt/Antigravity/antigravity" %U or '...'
    let unquoted = if (trimmed.starts_with('"') && trimmed[1..].contains('"'))
        || (trimmed.starts_with('\'') && trimmed[1..].contains('\''))
    {
        let quote_char = trimmed.chars().next().unwrap();
        let end_idx = trimmed[1..]
            .find(quote_char)
            .map(|i| i + 1)
            .unwrap_or(trimmed.len());
        &trimmed[1..end_idx]
    } else {
        trimmed.split_whitespace().next().unwrap_or("")
    };

    // If starts with env or /usr/bin/env, skip to next token
    let binary = if unquoted.ends_with("/env") || unquoted == "env" {
        let after_env = trimmed.trim_start_matches(unquoted).trim();
        after_env.split_whitespace().next().unwrap_or("")
    } else {
        unquoted
    };

    if binary.is_empty() {
        None
    } else {
        Some(binary.to_string())
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn resolve_linux_path_env(
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for p in std::env::split_paths(&path_var) {
        for exe in exe_names {
            let candidate = p.join(exe);
            checked.push(candidate.to_string_lossy().to_string());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub(crate) fn resolve_linux_desktop_entry(
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let mut dirs_to_check = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        std::path::PathBuf::from("/usr/local/share/applications"),
        std::path::PathBuf::from("/var/lib/snapd/desktop/applications"),
        std::path::PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        dirs_to_check.push(home.join(".local/share/applications"));
        dirs_to_check.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    for dir in dirs_to_check {
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(f) => f.to_string_lossy().to_lowercase(),
                None => continue,
            };
            if !file_name.ends_with(".desktop") {
                continue;
            }

            let matches_target = exe_names
                .iter()
                .any(|name| file_name.contains(&name.to_lowercase()));
            if matches_target {
                checked.push(format!(".desktop: {}", path.to_string_lossy()));
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(exec_cmd) = trimmed.strip_prefix("Exec=") {
                            if let Some(clean_cmd) = clean_desktop_exec_command(exec_cmd) {
                                let binary_path = std::path::PathBuf::from(&clean_cmd);
                                checked.push(format!("  -> Exec: {}", clean_cmd));
                                if binary_path.is_file() {
                                    return Some(binary_path);
                                }
                                // If relative or bare command, check PATH
                                if let Some(found) = resolve_linux_path_env(&[&clean_cmd], checked)
                                {
                                    return Some(found);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub(crate) fn resolve_linux_standard_paths(
    folder_name: &str,
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let folder_lower = folder_name.to_lowercase().replace(' ', "-");
    let folder_lower_simple = folder_name.to_lowercase().replace(' ', "");

    for exe in exe_names {
        let mut candidates = vec![
            std::path::PathBuf::from(format!("/usr/bin/{}", exe)),
            std::path::PathBuf::from(format!("/usr/local/bin/{}", exe)),
            std::path::PathBuf::from(format!("/snap/bin/{}", exe)),
            std::path::PathBuf::from(format!("/var/lib/snapd/snap/bin/{}", exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_lower, exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_lower_simple, exe)),
            std::path::PathBuf::from(format!("/opt/Google/Antigravity/{}", exe)),
            std::path::PathBuf::from(format!("/opt/google-antigravity/{}", exe)),
            std::path::PathBuf::from(format!("/usr/share/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/usr/share/{}/{}", folder_lower, exe)),
            std::path::PathBuf::from(format!("/usr/lib/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/usr/lib/{}/{}", folder_lower, exe)),
        ];

        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join(format!(".local/bin/{}", exe)));
            candidates.push(home.join(format!("bin/{}", exe)));
            candidates.push(home.join(format!(".local/share/{}/{}", folder_name, exe)));
            candidates.push(home.join(format!(".local/share/{}/{}", folder_lower, exe)));
            candidates.push(home.join(format!("Applications/{}.AppImage", folder_name)));
            candidates.push(home.join(format!("Applications/{}.AppImage", folder_lower)));
            candidates.push(home.join("Applications/Antigravity.AppImage"));
            candidates.push(home.join("Applications/antigravity.AppImage"));
            candidates.push(home.join("Downloads/Antigravity.AppImage"));
            candidates.push(home.join("Downloads/antigravity.AppImage"));
            candidates.push(home.join("Desktop/Antigravity.AppImage"));
            candidates.push(home.join("Desktop/antigravity.AppImage"));
        }

        for path in candidates {
            checked.push(path.to_string_lossy().to_string());
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// 获取 Antigravity CLI (agy) 的安装/可执行文件路径
pub fn get_antigravity_cli_executable_path() -> Option<std::path::PathBuf> {
    // 1. 优先从配置查询
    if let Ok(config) = crate::modules::config::load_app_config() {
        if let Some(ref p) = config.antigravity_cli_executable {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 2. 检查标准用户本地目录 ~/.local/bin/agy 或 ~/.local/bin/agy.exe
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local").join("bin");
        let path = if cfg!(target_os = "windows") {
            local_bin.join("agy.exe")
        } else {
            local_bin.join("agy")
        };
        if path.exists() {
            return Some(path);
        }

        // Check Windows standard %LOCALAPPDATA%\agy\bin\agy.exe
        if let Some(local_app_data) = dirs::data_local_dir() {
            let win_path = local_app_data.join("agy").join("bin").join("agy.exe");
            if win_path.exists() {
                return Some(win_path);
            }
        }

        // Check ~/.gemini/antigravity/bin/agy.exe
        let gemini_bin = home.join(".gemini").join("antigravity").join("bin");
        let gemini_path = if cfg!(target_os = "windows") {
            gemini_bin.join("agy.exe")
        } else {
            gemini_bin.join("agy")
        };
        if gemini_path.exists() {
            return Some(gemini_path);
        }
    }

    // 3. 在系统环境变量 PATH 中查找
    let cmd = if cfg!(target_os = "windows") {
        "agy.exe"
    } else {
        "agy"
    };
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            let p_cmd = p.join(cmd);
            if p_cmd.exists() {
                return Some(p_cmd);
            }
        }
    }

    None
}
