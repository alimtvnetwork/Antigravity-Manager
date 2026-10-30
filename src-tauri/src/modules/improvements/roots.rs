//! Resolve flywheel ledger and dispatch roots without requiring AGM source cwd.
//! Packaged GUI cwd is the install dir, so `01-prompts/30-improvements` is optional.

use crate::error::AppResult;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const IMPROVEMENTS_DIR: &str = "01-prompts/30-improvements";

pub fn normalize_instance_id(instance_id: &str) -> String {
    let trimmed = instance_id.trim();
    if trimmed.is_empty() || trimmed == "__default__" {
        "default".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn has_improvements_dir(path: &Path) -> bool {
    path.join(IMPROVEMENTS_DIR).is_dir()
}

pub fn walk_up_for_improvements(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if has_improvements_dir(&dir) {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn git_toplevel() -> Option<PathBuf> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--show-toplevel"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(PathBuf::from(text))
}

fn env_override_root() -> Option<PathBuf> {
    for key in ["AGM_IMPROVEMENTS_ROOT", "AGM_REPO_ROOT"] {
        let Ok(raw) = std::env::var(key) else {
            continue;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = PathBuf::from(trimmed);
        if has_improvements_dir(&path) {
            return Some(path);
        }
        if let Some(found) = walk_up_for_improvements(&path) {
            return Some(found);
        }
    }
    None
}

/// AGM source tree that contains `01-prompts/30-improvements` (optional).
pub fn find_source_repo() -> Option<PathBuf> {
    if let Some(env_root) = env_override_root() {
        return Some(env_root);
    }
    if let Some(git) = git_toplevel() {
        if has_improvements_dir(&git) {
            return Some(git);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(found) = walk_up_for_improvements(&cwd) {
            return Some(found);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(found) = walk_up_for_improvements(parent) {
                return Some(found);
            }
        }
    }
    None
}

fn app_data_dir() -> PathBuf {
    match crate::modules::account::get_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::warn!("improvements ledger falling back to temp dir: {e}");
            std::env::temp_dir().join("antigravity_tools")
        }
    }
}

/// Workspace the flywheel acts on: instance folders, then source repo, then app data.
pub fn dispatch_root(instance_id: &str) -> PathBuf {
    let inst = normalize_instance_id(instance_id);
    let folders = crate::modules::instance::get_instance_workspace_paths(&inst);
    if let Some(first) = folders.into_iter().next() {
        return PathBuf::from(first);
    }
    if let Some(source) = find_source_repo() {
        return source;
    }
    app_data_dir()
}

/// Ledger lives on the dispatch root so each instance workspace has its own flywheel.
pub fn ledger_root(instance_id: &str) -> PathBuf {
    dispatch_root(instance_id)
}

/// Never fails. Packaged GUI without a source checkout uses app data.
pub fn repo_root() -> AppResult<PathBuf> {
    Ok(ledger_root("default"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_tmp(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agm-fw-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn test_normalize_instance_id() {
        assert_eq!(normalize_instance_id(""), "default");
        assert_eq!(normalize_instance_id("  "), "default");
        assert_eq!(normalize_instance_id("__default__"), "default");
        assert_eq!(normalize_instance_id("v2-3127"), "v2-3127");
    }

    #[test]
    fn test_walk_up_finds_and_misses_marker() {
        let base = unique_tmp("walk");
        let nested = base.join("a").join("b");
        let marker = nested.join(IMPROVEMENTS_DIR);
        fs::create_dir_all(&marker).expect("create marker");
        assert_eq!(
            walk_up_for_improvements(&nested).as_deref(),
            Some(nested.as_path())
        );
        let empty = base.join("empty");
        fs::create_dir_all(&empty).expect("create empty");
        assert!(walk_up_for_improvements(&empty).is_none());
        // Justification: leftover temp dirs do not change the walk-up assertion
        crate::error::record_ignored(fs::remove_dir_all(&base), "cleanup flywheel walk fixture");
    }

    #[test]
    fn test_repo_root_never_config_error() {
        let root = repo_root().expect("repo_root must succeed without 01-prompts");
        assert!(!root.as_os_str().is_empty());
    }
}
