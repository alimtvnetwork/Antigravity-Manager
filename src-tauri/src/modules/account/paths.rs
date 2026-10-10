use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, RwLock};

use super::*;

// ... existing constants ...
pub(crate) const DATA_DIR: &str = ".antigravity_tools";

pub(crate) const LOCATION_POINTER_FILE: &str = ".antigravity_tools_location";

pub(crate) const ACCOUNTS_INDEX: &str = "accounts.json";

pub(crate) const ACCOUNTS_DIR: &str = "accounts";

pub(crate) const DATA_DIR_POINTER_FILE: &str = "data_dir.txt";

/// Get data directory bootstrap pointer file path (stored in system config dir)
pub fn get_data_dir_pointer_file() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("antigravity-tools").join(DATA_DIR_POINTER_FILE))
}

pub(crate) static DATA_DIR_OVERRIDE: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

pub(crate) fn data_dir_override_slot() -> &'static RwLock<Option<PathBuf>> {
    DATA_DIR_OVERRIDE.get_or_init(|| RwLock::new(None))
}

pub(crate) fn location_pointer_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
    Ok(home.join(LOCATION_POINTER_FILE))
}

pub(crate) fn default_data_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
    Ok(home.join(DATA_DIR))
}

pub(crate) fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("failed_to_create_data_dir: {}", e))?;
    }
    Ok(())
}

/// Strip Windows `\\?\` / `\\?\UNC\` prefixes and quotes so paths stay portable
/// across Windows, Linux, macOS and Docker (`ABV_DATA_DIR=/app/data`).
pub(crate) fn strip_extended_path_prefix(input: &str) -> String {
    let s = input
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '\u{feff}');
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", rest);
    }
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    if let Some(rest) = s.strip_prefix("//?/UNC/") {
        return format!("//{}", rest);
    }
    if let Some(rest) = s.strip_prefix("//?/") {
        return rest.to_string();
    }
    s.to_string()
}

pub(crate) fn expand_user_path(input: &str) -> Option<PathBuf> {
    if input == "~" || input.starts_with("~/") || input.starts_with("~\\") {
        let home = dirs::home_dir()?;
        let rest = input
            .trim_start_matches('~')
            .trim_start_matches(['/', '\\']);
        return Some(if rest.is_empty() {
            home
        } else {
            home.join(rest)
        });
    }
    None
}

/// Normalize a data-dir path for persistence, env vars and UI display.
pub fn normalize_data_dir_path(path: impl AsRef<Path>) -> PathBuf {
    let raw = path.as_ref().to_string_lossy();
    let stripped = strip_extended_path_prefix(&raw);
    if let Some(expanded) = expand_user_path(&stripped) {
        return expanded;
    }
    PathBuf::from(stripped)
}

/// Human-readable path without Windows verbatim prefixes.
pub fn format_data_dir_path(path: &Path) -> String {
    normalize_data_dir_path(path).to_string_lossy().into_owned()
}

pub(crate) fn resolve_existing_path(path: &Path) -> PathBuf {
    let normalized = normalize_data_dir_path(path);
    match normalized.canonicalize() {
        Ok(canon) => normalize_data_dir_path(canon),
        Err(_) => normalized,
    }
}

pub(crate) fn path_compare_key(path: &Path) -> String {
    let mut s = resolve_existing_path(path)
        .to_string_lossy()
        .replace('\\', "/");
    while s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    #[cfg(windows)]
    {
        s = s.to_ascii_lowercase();
    }
    s
}

pub(crate) fn paths_equivalent(a: &Path, b: &Path) -> bool {
    path_compare_key(a) == path_compare_key(b)
}

pub(crate) fn is_nested_data_dir(inner: &Path, outer: &Path) -> bool {
    let inner_key = path_compare_key(inner);
    let outer_key = path_compare_key(outer);
    inner_key != outer_key && inner_key.starts_with(&(outer_key + "/"))
}

pub(crate) fn persist_clean_env(dir: &Path) {
    std::env::set_var("ABV_DATA_DIR", format_data_dir_path(dir));
}

pub(crate) fn read_location_pointer() -> Option<PathBuf> {
    let path = location_pointer_path().ok()?;
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cleaned = normalize_data_dir_path(trimmed);
    if format_data_dir_path(&cleaned) != trimmed {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(write_location_pointer(&cleaned), "write_location_pointer");
    }
    Some(cleaned)
}

pub(crate) fn write_location_pointer(dir: &Path) -> Result<(), String> {
    let pointer = location_pointer_path()?;
    fs::write(&pointer, format_data_dir_path(dir).as_bytes())
        .map_err(|e| format!("Failed to write data directory pointer: {}", e))
}

pub(crate) fn is_default_data_dir(dir: &Path) -> bool {
    default_data_dir()
        .map(|d| paths_equivalent(&d, dir) || d == dir)
        .unwrap_or(false)
}

pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create target data directory: {}", e))?;
    for entry in
        fs::read_dir(src).map_err(|e| format!("Failed to read source data directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read data directory entry: {}", e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to read entry type: {}", e))?;
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create target subdirectory: {}", e))?;
            }
            fs::copy(&from, &to)
                .map_err(|e| format!("Failed to copy file {}: {}", from.display(), e))?;
        }
    }
    Ok(())
}

pub(crate) fn dir_is_empty(path: &Path) -> Result<bool, String> {
    let mut entries =
        fs::read_dir(path).map_err(|e| format!("Failed to read target directory: {}", e))?;
    Ok(entries.next().is_none())
}

pub(crate) fn apply_data_dir(dir: &Path) -> Result<(), String> {
    let dir = normalize_data_dir_path(dir);
    ensure_dir(&dir)?;
    if is_default_data_dir(&dir) {
        if let Ok(pointer) = location_pointer_path() {
            // Justification: best-effort cleanup; a leftover file is harmless
            crate::error::record_ignored(fs::remove_file(pointer), "remove_file");
        }
    } else {
        write_location_pointer(&dir)?;
    }
    if let Ok(mut guard) = data_dir_override_slot().write() {
        *guard = Some(dir.clone());
    }
    persist_clean_env(&dir);
    Ok(())
}

/// Get data directory path
pub fn get_data_dir() -> Result<PathBuf, String> {
    // 1. Process env (tests, Docker, and in-process override after migrate)
    if let Ok(env_path) = std::env::var("ABV_DATA_DIR") {
        if !env_path.trim().is_empty() {
            let data_dir = normalize_data_dir_path(&env_path);
            ensure_dir(&data_dir)?;
            if format_data_dir_path(&data_dir) != env_path {
                persist_clean_env(&data_dir);
            }
            return Ok(data_dir);
        }
    }

    // 2. Runtime override (pointer already loaded this session)
    if let Ok(guard) = data_dir_override_slot().read() {
        if let Some(ref path) = *guard {
            let data_dir = normalize_data_dir_path(path);
            ensure_dir(&data_dir)?;
            return Ok(data_dir);
        }
    }

    // 3. Pointer file outside the data dir so deleting the old folder still finds the new path
    if let Some(path) = read_location_pointer() {
        ensure_dir(&path)?;
        if let Ok(mut guard) = data_dir_override_slot().write() {
            *guard = Some(path.clone());
        }
        return Ok(path);
    }

    // 4. Default ~/.antigravity_tools
    let data_dir = default_data_dir()?;
    ensure_dir(&data_dir)?;
    Ok(data_dir)
}

/// Move the data directory to `new_dir`, persist the location, and switch all runtime lookups.
pub fn migrate_data_dir(new_dir: PathBuf) -> Result<PathBuf, String> {
    let new_dir = normalize_data_dir_path(new_dir);
    let new_dir = if new_dir.as_os_str().is_empty() {
        return Err("Target data directory cannot be empty".to_string());
    } else if new_dir.is_absolute() {
        new_dir
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to resolve relative path: {}", e))?
            .join(new_dir)
    };

    let old_dir = normalize_data_dir_path(get_data_dir()?);
    if paths_equivalent(&old_dir, &new_dir) {
        apply_data_dir(&old_dir)?;
        return Ok(resolve_existing_path(&old_dir));
    }

    if is_nested_data_dir(&new_dir, &old_dir) {
        return Err("Cannot migrate data directory into itself".to_string());
    }

    if new_dir.exists() {
        if new_dir.is_file() {
            return Err("Target path already exists and is not a directory".to_string());
        }
        if !dir_is_empty(&new_dir)? {
            return Err(
                "Target directory is not empty, please select an empty directory or new path"
                    .to_string(),
            );
        }
        copy_dir_recursive(&old_dir, &new_dir)?;
        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(&old_dir), "remove_dir_all");
    } else if let Some(parent) = new_dir.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create target parent directory: {}", e))?;
        match fs::rename(&old_dir, &new_dir) {
            Ok(()) => {}
            Err(_) => {
                copy_dir_recursive(&old_dir, &new_dir)?;
                // Justification: best-effort cleanup; a leftover directory is harmless
                crate::error::record_ignored(fs::remove_dir_all(&old_dir), "remove_dir_all");
            }
        }
    } else {
        return Err("Target path is invalid".to_string());
    }

    let resolved = resolve_existing_path(&new_dir);
    apply_data_dir(&resolved)?;
    Ok(resolved)
}
