use crate::modules::process;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::fs;
use std::path::PathBuf;

/// Antigravity version information
#[derive(Debug, Clone)]
pub struct AntigravityVersion {
    pub short_version: String,
    #[allow(dead_code)] // Reserved for build/diagnostic output
    pub bundle_version: String,
}

/// Extracts the first semantic version number (X.Y.Z) from any string
fn extract_semver(raw: &str) -> Option<String> {
    for token in raw.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let t = token.trim_matches(|c: char| c == '"' || c == '\'' || c == '(' || c == ')');
        if t.is_empty() {
            continue;
        }
        let mut parts = t.split('.');
        let p1 = parts.next();
        let p2 = parts.next();
        let p3 = parts.next();
        if p1.is_some()
            && p2.is_some()
            && p3.is_some()
            && [p1.unwrap(), p2.unwrap(), p3.unwrap()]
                .iter()
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(t.to_string());
        }
    }
    None
}

/// Detect Antigravity version (cross-platform, supports pre-snapshot path priority)
pub fn get_antigravity_version_with_path(
    target_ide: Option<&str>,
    preferred_path: Option<&std::path::Path>,
) -> Result<AntigravityVersion, String> {
    // 1. Preferentially use the pre-capture path, if not, explore the Antigravity executable file path
    let exe_path = preferred_path
        .map(|p| p.to_path_buf())
        .or_else(|| process::get_antigravity_executable_path(target_ide))
        .ok_or("Unable to locate Antigravity executable")?;

    // 2. Read version information based on the platform
    #[cfg(target_os = "macos")]
    {
        get_version_macos(&exe_path)
    }

    #[cfg(target_os = "windows")]
    {
        get_version_windows(&exe_path)
    }

    #[cfg(target_os = "linux")]
    {
        get_version_linux(&exe_path)
    }
}

/// Detect Antigravity version (cross-platform)
pub fn get_antigravity_version(target_ide: Option<&str>) -> Result<AntigravityVersion, String> {
    get_antigravity_version_with_path(target_ide, None)
}

/// macOS: Reading version from Info.plist
#[cfg(target_os = "macos")]
fn get_version_macos(exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    use plist::Value;

    // exe_path may be /Applications/Antigravity.app or an internal executable
    // Need to find the .app directory
    let path_str = exe_path.to_string_lossy();
    let app_path = if let Some(idx) = path_str.find(".app") {
        PathBuf::from(&path_str[..idx + 4])
    } else {
        exe_path.clone()
    };

    let info_plist_path = app_path.join("Contents/Info.plist");
    if !info_plist_path.exists() {
        return Err(format!("Info.plist not found: {:?}", info_plist_path));
    }

    let content =
        fs::read(&info_plist_path).map_err(|e| format!("Failed to read Info.plist: {}", e))?;

    let plist: Value =
        plist::from_bytes(&content).map_err(|e| format!("Failed to parse Info.plist: {}", e))?;

    let dict = plist
        .as_dictionary()
        .ok_or("Info.plist is not a dictionary")?;

    let short_version = dict
        .get("CFBundleShortVersionString")
        .and_then(|v| v.as_string())
        .ok_or("CFBundleShortVersionString not found")?;

    let bundle_version = dict
        .get("CFBundleVersion")
        .and_then(|v| v.as_string())
        .unwrap_or(short_version);

    Ok(AntigravityVersion {
        short_version: short_version.to_string(),
        bundle_version: bundle_version.to_string(),
    })
}

/// Windows: Reading version from executable metadata
#[cfg(target_os = "windows")]
fn get_version_windows(exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    use crate::utils::command::CommandExtWrapper;
    use std::process::Command;

    // Windows: Use PowerShell to read file version information
    let mut cmd = Command::new("powershell");
    let output = cmd
        .creation_flags_windows()
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!(
                "(Get-Item '{}').VersionInfo.FileVersion",
                exe_path.display()
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    if !output.status.success() {
        return Err("Failed to read version from executable".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if version.is_empty() {
        return Err("Version information not found in executable".to_string());
    }

    Ok(AntigravityVersion {
        short_version: version.clone(),
        bundle_version: version,
    })
}

/// Linux: read from package.json or --version parameter
#[cfg(target_os = "linux")]
fn get_version_linux(exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    use std::process::Command;

    // Method 1 (Priority): Try to read package.json from the installation directory to avoid accidentally launching the GUI when executing the executable file
    if let Some(parent) = exe_path.parent() {
        let package_json = parent.join("resources/app/package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                        return Ok(AntigravityVersion {
                            short_version: version.to_string(),
                            bundle_version: version.to_string(),
                        });
                    }
                }
            }
        }
    }

    // Method 2 (cover): Try executing --version (only executed if it cannot be obtained from package.json)
    let output = Command::new(exe_path).arg("--version").output();

    if let Ok(result) = output {
        if result.status.success() {
            let raw_version = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if !raw_version.is_empty() {
                let version = extract_semver(&raw_version).unwrap_or_else(|| {
                    raw_version
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                });
                return Ok(AntigravityVersion {
                    short_version: version.clone(),
                    bundle_version: raw_version,
                });
            }
        }
    }

    Err("Unable to determine Antigravity version on Linux".to_string())
}

/// Determine whether it is a new version (>= 1.16.5)
pub fn is_new_version(version: &AntigravityVersion) -> bool {
    compare_version(&version.short_version, "1.16.5") >= std::cmp::Ordering::Equal
}

/// Compare version numbers
pub fn compare_version(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..parts1.len().max(parts2.len()) {
        let p1 = parts1.get(i).unwrap_or(&0);
        let p2 = parts2.get(i).unwrap_or(&0);
        match p1.cmp(p2) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert_eq!(
            compare_version("1.16.5", "1.16.4"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_version("1.16.5", "1.16.5"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_version("1.16.4", "1.16.5"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_version("1.17.0", "1.16.5"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_version("2.0.0", "1.16.5"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_is_new_version() {
        let old = AntigravityVersion {
            short_version: "1.16.4".to_string(),
            bundle_version: "1.16.4".to_string(),
        };
        assert!(!is_new_version(&old));

        let new = AntigravityVersion {
            short_version: "1.16.5".to_string(),
            bundle_version: "1.16.5".to_string(),
        };
        assert!(is_new_version(&new));

        let newer = AntigravityVersion {
            short_version: "1.17.0".to_string(),
            bundle_version: "1.17.0".to_string(),
        };
        assert!(is_new_version(&newer));
    }

    #[test]
    fn test_extract_semver_from_messy_output() {
        let raw = "1.107.0\n1504c8cc4b34dbfbb4a97ebe954b3da2b5634516\nx64";
        assert_eq!(extract_semver(raw), Some("1.107.0".to_string()));
    }
}
