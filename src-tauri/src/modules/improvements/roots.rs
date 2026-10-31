//! Resolve flywheel ledger and dispatch roots without requiring AGM source cwd.
//! Packaged GUI cwd is the install dir, so `01-prompts/30-improvements` is optional.

use crate::error::AppResult;
use std::collections::HashSet;
use std::fs;
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

const TASK_HINT_CAP: usize = 16;

fn skip_task_stem(stem: &str) -> bool {
    matches!(
        stem.to_ascii_lowercase().as_str(),
        "readme" | "index" | "changelog" | "license"
    )
}

fn skip_task_dir(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "completed" | "archived" | "closed" | "changelog"
    )
}

fn push_md_stems(dir: &Path, out: &mut Vec<String>, seen: &mut HashSet<String>, recurse: bool) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if out.len() >= TASK_HINT_CAP {
            return;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            if skip_task_dir(name) {
                continue;
            }
            if recurse {
                push_md_stems(&path, out, seen, false);
            }
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if skip_task_stem(stem) {
            continue;
        }
        let key = stem.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(stem.to_string());
        }
    }
}

/// Pending plans, issues, and open questions in any project workspace.
/// Used so Detail/Next can break down tasks without AGM's `01-prompts` tree.
pub fn collect_workspace_task_hints(workspace: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let dirs = [
        workspace.join(".ai-memory/plans/pending"),
        workspace.join(".ai-memory/issues"),
        workspace.join(".ai-memory/plans"),
        workspace.join(".ai-memory/ambiguous-questions"),
        workspace.join(".ai-memory/plans/subtasks"),
    ];
    for dir in dirs {
        if out.len() >= TASK_HINT_CAP {
            break;
        }
        let recurse = dir
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| matches!(n, "subtasks" | "pending" | "ambiguous-questions"));
        push_md_stems(&dir, &mut out, &mut seen, recurse);
    }
    out
}

/// Union of task hints across every folder bound to the instance, plus the dispatch root.
pub fn collect_instance_task_hints(instance_id: &str) -> Vec<String> {
    let inst = normalize_instance_id(instance_id);
    let mut roots: Vec<PathBuf> = crate::modules::instance::get_instance_workspace_paths(&inst)
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let dispatch = dispatch_root(&inst);
    if !roots.iter().any(|r| r == &dispatch) {
        roots.insert(0, dispatch);
    }
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        for hint in collect_workspace_task_hints(&root) {
            let key = hint.to_ascii_lowercase();
            if seen.insert(key) {
                out.push(hint);
                if out.len() >= TASK_HINT_CAP {
                    return out;
                }
            }
        }
    }
    out
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

    #[test]
    fn test_collect_workspace_task_hints_any_project() {
        let base = unique_tmp("tasks");
        let pending = base.join(".ai-memory").join("plans").join("pending");
        fs::create_dir_all(&pending).expect("create pending");
        fs::write(pending.join("01-fix-login.md"), "# login\n").expect("write pending");
        fs::write(
            base.join(".ai-memory").join("plans").join("readme.md"),
            "# skip\n",
        )
        .expect("write readme");
        let completed = base.join(".ai-memory").join("plans").join("completed");
        fs::create_dir_all(&completed).expect("create completed");
        fs::write(completed.join("99-old-done.md"), "# done\n").expect("write completed");
        fs::write(
            base.join(".ai-memory")
                .join("issues")
                .join("66-open-switch.md"),
            "# issue\n",
        )
        .expect("write issue");
        let hints = collect_workspace_task_hints(&base);
        assert!(
            hints.iter().any(|h| h.contains("01-fix-login")),
            "expected pending task, got {hints:?}"
        );
        assert!(
            hints.iter().any(|h| h.contains("66-open-switch")),
            "expected open issue, got {hints:?}"
        );
        assert!(
            !hints.iter().any(|h| h.eq_ignore_ascii_case("readme")),
            "readme must not be a task hint: {hints:?}"
        );
        assert!(
            !hints.iter().any(|h| h.contains("old-done")),
            "completed plans must not be task hints: {hints:?}"
        );
        crate::error::record_ignored(fs::remove_dir_all(&base), "cleanup flywheel task fixture");
    }
}
