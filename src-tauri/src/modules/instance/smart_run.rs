//! Ensure-running / focus-or-launch smart entry points.
use super::*;
use crate::error::AppError;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Smart ensure instance is running:
/// Checks process cache and OS PID table. If already running, reuses without reopening.
/// If not running or cached PID is closed, launches instance, waits 800ms, invalidates cache,
/// force refreshes scan, verifies the newly spawned PID, and caches it.
pub fn ensure_instance_running_smart(
    instance_id: &str,
    workspace_path: Option<&str>,
) -> Result<(bool, Option<u32>), crate::error::AppError> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let (is_running, primary_pid, pids) = is_instance_process_running_smart(&canonical_id);
    if is_running {
        crate::modules::logger::log_info(&format!(
            "[SmartProcessCache] Instance '{}' ({}) is verified running (PID: {:?}). Zero relaunch guarantee active.",
            instance_id, canonical_id, primary_pid
        ));
        focus_running_instance(&pids, workspace_path);
        return Ok((true, primary_pid));
    }

    crate::modules::logger::log_info(&format!(
        "[SmartProcessCache] Instance '{}' ({}) confirmed offline across OS. Initiating cold launch...",
        instance_id, canonical_id
    ));
    if let Some(ws) = workspace_path {
        let ws_vec = vec![ws.to_string()];
        launch_instance_with_workspaces(&canonical_id, Some(&ws_vec), true)?;
    } else {
        launch_instance(&canonical_id)?;
    }

    std::thread::sleep(std::time::Duration::from_millis(800));
    invalidate_instance_process_cache(&canonical_id);
    force_refresh_process_cache();
    let (is_now_running, new_pid, _) = is_instance_process_running_smart(&canonical_id);

    Ok((is_now_running, new_pid))
}

/// Focus an already running instance window, or launch it if not running
pub fn focus_or_launch_instance(instance_id: &str) -> Result<bool, crate::error::AppError> {
    focus_or_launch_instance_with_workspace(instance_id, None)
}

/// Focus an already running instance window, or launch it with optional workspace path if not running
pub fn focus_or_launch_instance_with_workspace(
    instance_id: &str,
    workspace_path: Option<&str>,
) -> Result<bool, crate::error::AppError> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let (is_running, _) = ensure_instance_running_smart(&canonical_id, workspace_path)?;
    Ok(is_running)
}

/// Focus an already running instance window with matching workspace, or launch it targeted at workspace
pub fn focus_or_launch_workspace(
    instance_id: &str,
    repo_path: &str,
    repo_name: &str,
) -> Result<bool, crate::error::AppError> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let target_path = if !repo_path.is_empty() {
        repo_path
    } else {
        repo_name
    };
    let (is_running, _) = ensure_instance_running_smart(&canonical_id, Some(target_path))?;
    Ok(is_running)
}

/// Launch a specific instance with multi-window isolation, bound workspace folder restoration, and custom/cloned executable support
pub fn launch_instance(instance_id: &str) -> Result<(), crate::error::AppError> {
    launch_instance_inner(instance_id, true)
}

/// Launch during an account switch. Prompt restore stays in the switch step that runs after the process is up.
pub fn launch_instance_without_prompt_reinject(
    instance_id: &str,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner(instance_id, false)
}

/// Launch a specific instance with extra/migrated workspace folders and prompt reinjection control
pub fn launch_instance_with_workspaces(
    instance_id: &str,
    extra_workspaces: Option<&[String]>,
    reinject_prompts: bool,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner_with_extra_workspaces(instance_id, reinject_prompts, extra_workspaces)
}

fn launch_instance_inner(
    instance_id: &str,
    reinject_prompts: bool,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner_with_extra_workspaces(instance_id, reinject_prompts, None)
}

/// Helper to construct candidate search paths for macOS instance launch.
pub(crate) fn get_macos_candidate_paths() -> Vec<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/Applications/Antigravity.app"),
        PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity"),
    ];

    if let Some(home) = dirs::home_dir() {
        let user_app = home.join("Applications").join("Antigravity.app");
        candidates.push(user_app.clone());
        let user_macos_bin = user_app.join("Contents").join("MacOS").join("Antigravity");
        candidates.push(user_macos_bin);
    }

    candidates
}
