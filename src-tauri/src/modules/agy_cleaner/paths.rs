use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Get the base Antigravity user data directory (~/.gemini/antigravity)
pub fn get_gemini_base_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gemini").join("antigravity"))
}

/// Get all existing candidate Gemini Antigravity base directories (antigravity, antigravity-cli, antigravity-ide)
pub fn get_gemini_candidate_dirs() -> Vec<PathBuf> {
    let mut dirs_list = Vec::new();
    if let Some(home) = dirs::home_dir() {
        for sub in &["antigravity", "antigravity-cli", "antigravity-ide"] {
            let p = home.join(".gemini").join(sub);
            if p.exists() && !dirs_list.contains(&p) {
                dirs_list.push(p);
            }
        }
    }
    if let Ok(reg) = crate::modules::instance::load_registry() {
        for inst in reg.instances {
            if !inst.is_default && inst.id != "default" {
                if let Ok(inst_home) = crate::modules::instance::get_instance_home_dir(&inst.id) {
                    for sub in &["antigravity", "antigravity-cli", "antigravity-ide"] {
                        let p = inst_home.join(".gemini").join(sub);
                        if p.exists() && !dirs_list.contains(&p) {
                            dirs_list.push(p);
                        }
                    }
                }
            }
        }
    }
    if dirs_list.is_empty() {
        if let Some(def) = get_gemini_base_dir() {
            dirs_list.push(def);
        }
    }
    dirs_list
}

/// Get the temporary staging directory for recoverable backups
pub fn get_temp_staging_dir() -> PathBuf {
    std::env::temp_dir().join("antigravity-cleaner-backup")
}

/// Safely copy or move a file/directory
pub(crate) fn safe_move_path(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if let Err(_rename_err) = fs::rename(src, dst) {
        // Fallback to copy + remove for cross-device moves
        if src.is_dir() {
            copy_dir_all(src, dst)?;
            fs::remove_dir_all(src)?;
        } else {
            fs::copy(src, dst)?;
            fs::remove_file(src)?;
        }
    }

    Ok(())
}

pub(crate) fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let dest_child = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_child)?;
        } else {
            fs::copy(entry.path(), dest_child)?;
        }
    }
    Ok(())
}

/// Calculate directory size recursively
pub fn get_dir_size_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_file() {
                total += fs::metadata(&child).map(|m| m.len()).unwrap_or(0);
            } else if child.is_dir() {
                total += get_dir_size_bytes(&child);
            }
        }
    }
    total
}
