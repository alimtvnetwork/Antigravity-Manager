//! Installation verification and path resolution for IDE targets.
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use super::types::{IdeStatusInfo, IdeTarget};
use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Discovers the executable path for the specified IDE target if installed on the system.
pub fn find_ide_executable(target: IdeTarget) -> Option<PathBuf> {
    match target {
        IdeTarget::Antigravity => {
            if let Ok(path) = crate::modules::process::detect_antigravity_with_diagnostics(None) {
                if path.is_file() {
                    return Some(path);
                }
            }
            find_binary_candidates(&[
                #[cfg(target_os = "windows")]
                "Antigravity.exe",
                "antigravity",
            ])
        }
        IdeTarget::Cursor => {
            let mut candidates: Vec<PathBuf> = Vec::new();

            #[cfg(target_os = "windows")]
            {
                if let Ok(local) = std::env::var("LOCALAPPDATA") {
                    candidates.push(
                        PathBuf::from(local)
                            .join("Programs")
                            .join("cursor")
                            .join("Cursor.exe"),
                    );
                }
                if let Ok(appdata) = std::env::var("APPDATA") {
                    candidates.push(
                        PathBuf::from(appdata)
                            .join("..")
                            .join("Local")
                            .join("Programs")
                            .join("cursor")
                            .join("Cursor.exe"),
                    );
                }
            }

            #[cfg(target_os = "macos")]
            {
                candidates.push(PathBuf::from(
                    "/Applications/Cursor.app/Contents/MacOS/Cursor",
                ));
                if let Some(home) = dirs::home_dir() {
                    candidates.push(
                        home.join("Applications")
                            .join("Cursor.app")
                            .join("Contents")
                            .join("MacOS")
                            .join("Cursor"),
                    );
                }
            }

            #[cfg(target_os = "linux")]
            {
                candidates.push(PathBuf::from("/usr/bin/cursor"));
                candidates.push(PathBuf::from("/snap/bin/cursor"));
                if let Some(home) = dirs::home_dir() {
                    candidates.push(home.join(".local").join("bin").join("cursor"));
                }
            }

            for p in candidates {
                if p.is_file() {
                    return Some(p);
                }
            }

            find_binary_candidates(&[
                #[cfg(target_os = "windows")]
                "Cursor.exe",
                "cursor",
            ])
        }
        IdeTarget::VsCode => {
            let mut candidates: Vec<PathBuf> = Vec::new();

            #[cfg(target_os = "windows")]
            {
                if let Ok(local) = std::env::var("LOCALAPPDATA") {
                    candidates.push(
                        PathBuf::from(local)
                            .join("Programs")
                            .join("Microsoft VS Code")
                            .join("Code.exe"),
                    );
                }
                if let Ok(pf) = std::env::var("ProgramFiles") {
                    candidates.push(PathBuf::from(pf).join("Microsoft VS Code").join("Code.exe"));
                }
                if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
                    candidates.push(
                        PathBuf::from(pf86)
                            .join("Microsoft VS Code")
                            .join("Code.exe"),
                    );
                }
            }

            #[cfg(target_os = "macos")]
            {
                candidates.push(PathBuf::from(
                    "/Applications/Visual Studio Code.app/Contents/MacOS/Electron",
                ));
                if let Some(home) = dirs::home_dir() {
                    candidates.push(
                        home.join("Applications")
                            .join("Visual Studio Code.app")
                            .join("Contents")
                            .join("MacOS")
                            .join("Electron"),
                    );
                }
            }

            #[cfg(target_os = "linux")]
            {
                candidates.push(PathBuf::from("/usr/bin/code"));
                candidates.push(PathBuf::from("/snap/bin/code"));
                if let Some(home) = dirs::home_dir() {
                    candidates.push(home.join(".local").join("bin").join("code"));
                }
            }

            for p in candidates {
                if p.is_file() {
                    return Some(p);
                }
            }

            find_binary_candidates(&[
                #[cfg(target_os = "windows")]
                "Code.exe",
                "code",
            ])
        }
    }
}

/// Searches the system PATH for the given binary names.
fn find_binary_candidates(names: &[&str]) -> Option<PathBuf> {
    for name in names {
        #[cfg(target_os = "windows")]
        let probe = Command::new("where.exe").arg(name).output();

        #[cfg(not(target_os = "windows"))]
        let probe = Command::new("which").arg(name).output();

        if let Ok(out) = probe {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let trimmed = first_line.trim();
                    let path = PathBuf::from(trimmed);
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

/// Attempts to retrieve the semantic version string of the binary.
pub fn probe_ide_version(exe_path: &Path) -> Option<String> {
    let output = Command::new(exe_path).arg("--version").output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = stdout.lines().next() {
            let ver = first_line.trim().to_string();
            if !ver.is_empty() {
                return Some(ver);
            }
        }
    }
    None
}

/// Inspects the installation and version status of a single IDE target.
pub fn get_ide_status(target: IdeTarget) -> IdeStatusInfo {
    let exe = find_ide_executable(target);
    let is_installed = exe.is_some();
    let version = exe.as_deref().and_then(probe_ide_version);
    let executable_path = exe.map(|p| p.to_string_lossy().to_string());

    IdeStatusInfo {
        target,
        display_name: target.display_name().to_string(),
        is_installed,
        version,
        executable_path,
    }
}

/// Discovers installation status across all supported IDE targets.
pub fn detect_all_ide_status() -> Result<Vec<IdeStatusInfo>, AppError> {
    let targets = [IdeTarget::Antigravity, IdeTarget::Cursor, IdeTarget::VsCode];
    let status_list = targets.into_iter().map(get_ide_status).collect();
    Ok(status_list)
}
