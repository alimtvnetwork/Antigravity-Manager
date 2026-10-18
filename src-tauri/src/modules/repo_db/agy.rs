//! Repo DB: agy

use super::detection::{
    effective_image_uris, extract_image_payload_or_path, extract_image_uris_or_paths,
    extract_screenshot_id_token, resume_task_document,
};
use super::models::ActivePrompt;
use super::prompts_crud::save_or_requeue_prompt;
use super::state::get_active_agy_workers;
use super::text_utils::extract_clean_user_prompt;
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use uuid::Uuid;

fn truncate_utf8(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

/// `agy` resumes a conversation with `--conversation`. A synthetic `conv-` id means
/// "start a new turn", and a project key (`name__instance`) is not a conversation id.
fn agy_conversation_id(session: &str) -> Option<&str> {
    let trimmed = session.trim();
    if trimmed.is_empty()
        || trimmed == "-"
        || trimmed.starts_with("conv-")
        || trimmed.starts_with("proj-")
        || trimmed.starts_with("p-")
        || trimmed.starts_with("queued-")
    {
        return None;
    }
    if trimmed.contains("__") || trimmed.contains('\\') || trimmed.contains('/') {
        return None;
    }
    Some(trimmed)
}

/// Terminate any active `agy` background worker processes scoped to `instance_id`
/// before switching or closing an instance so stale workers cannot block post-switch dispatch.
pub fn terminate_agy_workers_for_instance(instance_id: &str) -> usize {
    let clean_inst = if instance_id.trim().is_empty() || instance_id == "__default__" {
        "default".to_string()
    } else {
        crate::modules::instance::resolve_instance_id(instance_id.trim())
            .unwrap_or_else(|_| instance_id.trim().to_string())
    };
    let is_default = clean_inst == "default";

    let mut workers = match get_active_agy_workers().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let matching_keys: Vec<(String, u32)> = workers
        .iter()
        .filter_map(|(ws_key, &pid)| {
            let prefix = ws_key.split(':').next().unwrap_or("");
            let matches = prefix == clean_inst
                || (is_default
                    && (prefix == "default" || prefix == "__default__" || prefix.is_empty()));
            if matches {
                Some((ws_key.clone(), pid))
            } else {
                None
            }
        })
        .collect();

    if matching_keys.is_empty() {
        return 0;
    }

    let pids: Vec<sysinfo::Pid> = matching_keys
        .iter()
        .map(|(_, pid)| sysinfo::Pid::from_u32(*pid))
        .collect();
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&pids),
        sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );

    let mut terminated = 0;
    for (ws_key, pid) in matching_keys {
        if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid)) {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Terminating active agy worker (pid {pid}) for instance '{}' ({ws_key}) before switch/close",
                clean_inst
            ));
            proc.kill();
            terminated += 1;
        }
        workers.remove(&ws_key);
    }

    terminated
}

/// Terminate all active `agy` worker processes across all instances during application shutdown.
pub fn terminate_all_active_agy_workers() -> usize {
    let mut workers = match get_active_agy_workers().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if workers.is_empty() {
        return 0;
    }
    let pids: Vec<sysinfo::Pid> = workers
        .values()
        .map(|&pid| sysinfo::Pid::from_u32(pid))
        .collect();
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&pids),
        sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );
    let mut terminated = 0;
    for (ws_key, pid) in workers.drain() {
        if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid)) {
            crate::modules::logger::log_info(&format!(
                "[Shutdown] Terminating active agy worker (pid {pid}) for workspace {ws_key}"
            ));
            proc.kill();
            terminated += 1;
        }
    }
    terminated
}

/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Uses `--conversation` (not `--resume`, which agy rejects) and `-p`.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> Result<(), String> {
    spawn_or_replace_prompt_via_agy(prompt, false)
}

/// Spawn `agy` CLI to execute a prompt in a workspace. When `replace_existing`
/// is true (explicit user Send/Rerun), any lingering headless `agy` worker for
/// the same `(instance_id, repo_path)` is terminated before spawning the new worker.
pub fn spawn_or_replace_prompt_via_agy(
    prompt: &ActivePrompt,
    replace_existing: bool,
) -> Result<(), String> {
    if std::env::var("AGM_SKIP_AGY_SPAWN").as_deref() == Ok("1") {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] AGM_SKIP_AGY_SPAWN=1 set: skipped agy spawn for prompt '{}'",
            prompt.id
        ));
        return Ok(());
    }

    let ws_dir = PathBuf::from(&prompt.repo_path);
    if !ws_dir.exists() {
        let message = format!("workspace directory does not exist: '{}'", prompt.repo_path);
        crate::modules::logger::log_warn(&format!("[RepoDB] {message}"));
        return Err(message);
    }

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    let prompt_to_run = if clean_prompt.trim().is_empty() {
        prompt.prompt_content.trim().to_string()
    } else {
        clean_prompt
    };
    if prompt_to_run.trim().is_empty() {
        let message = format!("prompt content for '{}' is empty", prompt.id);
        crate::modules::logger::log_warn(&format!("[RepoDB] {message}"));
        return Err(message);
    }

    let ws_key = format!("{}:{}", prompt.instance_id, prompt.repo_path);
    {
        let mut workers = match get_active_agy_workers().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(&existing_pid) = workers.get(&ws_key) {
            let mut sys = sysinfo::System::new();
            let target_pid = sysinfo::Pid::from_u32(existing_pid);
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            );
            if let Some(proc) = sys.process(target_pid) {
                if replace_existing {
                    crate::modules::logger::log_info(&format!(
                        "[RepoDB] Replacing existing agy worker (pid {existing_pid}) in workspace '{}' for explicit Send of prompt '{}'",
                        prompt.repo_path, prompt.id
                    ));
                    proc.kill();
                    workers.remove(&ws_key);
                } else {
                    let message = format!(
                        "agy is already running for this workspace (pid {existing_pid}); prompt '{}' was not sent again",
                        prompt.id
                    );
                    crate::modules::logger::log_info(&format!("[RepoDB] {message}"));
                    return Err(message);
                }
            } else {
                workers.remove(&ws_key);
            }
        }
    }

    if let Some(agy_bin) = crate::modules::process::get_antigravity_cli_executable_path() {
        let mut cmd = std::process::Command::new(&agy_bin);
        cmd.current_dir(&ws_dir);
        cmd.arg("--dangerously-skip-permissions");

        if let Some(ref sess) = prompt.session_id {
            if let Some(conversation_id) = agy_conversation_id(sess) {
                cmd.arg("--conversation").arg(conversation_id);
            }
        }

        cmd.arg("-p").arg(truncate_utf8(&prompt_to_run, 24000));

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        if prompt.instance_id != "default" && !prompt.instance_id.is_empty() {
            if let Ok(registry) = crate::modules::instance::load_registry() {
                if let Some(inst) = registry
                    .instances
                    .iter()
                    .find(|i| i.id == prompt.instance_id || i.name == prompt.instance_id)
                {
                    if let Some(ref acc_id) = inst.bound_account_id {
                        if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                            cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                            cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
                        }
                    }
                }
            }

            if let Ok(inst_home) =
                crate::modules::instance::get_instance_home_dir(&prompt.instance_id)
            {
                #[cfg(target_os = "windows")]
                {
                    cmd.env("USERPROFILE", &inst_home);
                }
                cmd.env("HOME", &inst_home);
                cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
                cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
                cmd.env("SSH_TTY", "pty/0");
                cmd.env("WSL_DISTRO_NAME", "antigravity-isolated");
                cmd.env("DOCKER_CONTAINER", "1");
            }
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                std::thread::sleep(std::time::Duration::from_millis(400));
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let mut err_msg = String::new();
                        if let Some(mut reader) = child.stderr.take() {
                            use std::io::Read;
                            if let Err(read_err) = reader.read_to_string(&mut err_msg) {
                                err_msg.push_str(&format!(" (stderr read failed: {read_err})"));
                            }
                        }
                        if status.success() {
                            let _ = crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
                                &prompt.instance_id,
                                &prompt_to_run,
                            );
                            return Ok(());
                        }
                        let detail = err_msg.trim();
                        let message = if detail.is_empty() {
                            format!("agy exited {status}")
                        } else {
                            format!("agy exited {status}: {detail}")
                        };
                        crate::modules::logger::log_error(&format!(
                            "[RepoDB] {message} for prompt '{}'",
                            prompt.id
                        ));
                        return Err(message);
                    }
                    Ok(None) => {}
                    Err(wait_err) => {
                        return Err(format!("agy wait failed: {wait_err}"));
                    }
                }

                let child_pid = child.id();
                {
                    let mut workers = match get_active_agy_workers().lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    workers.insert(ws_key.clone(), child_pid);
                }
                let _ =
                    crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
                        &prompt.instance_id,
                        &prompt_to_run,
                    );
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                    prompt.id, child_pid, prompt.instance_id, prompt.repo_path, prompt_to_run
                ));

                let prompt_id = prompt.id.clone();
                let ws_key_clone = ws_key.clone();
                let inst_id_clone = prompt.instance_id.clone();
                let repo_path_clone = prompt.repo_path.clone();
                let monitor_res = std::thread::Builder::new()
                    .name(format!("agy-monitor-{}", prompt_id))
                    .spawn(move || {
                        let mut stderr_reader = child.stderr.take();
                        match child.wait() {
                            Ok(status) => {
                                {
                                    let mut workers = match get_active_agy_workers().lock() {
                                        Ok(guard) => guard,
                                        Err(poisoned) => poisoned.into_inner(),
                                    };
                                    workers.remove(&ws_key_clone);
                                }
                                if !status.success() {
                                    let mut err_msg = String::new();
                                    if let Some(mut reader) = stderr_reader {
                                        use std::io::Read;
                                        if let Err(read_err) = reader.read_to_string(&mut err_msg) {
                                            err_msg.push_str(&format!(
                                                " (stderr read failed: {read_err})"
                                            ));
                                        }
                                    }
                                    let err_str = err_msg.trim();
                                    tracing::warn!(
                                        target: "repo_db",
                                        prompt_id = %prompt_id,
                                        exit_code = ?status.code(),
                                        stderr = %err_str,
                                        "agy CLI process exited with non-zero status"
                                    );
                                    crate::modules::logger::log_warn(&format!(
                                        "[RepoDB] agy CLI for prompt '{}' exited with status {:?}, stderr: {}",
                                        prompt_id, status.code(), err_str
                                    ));
                                } else {
                                    tracing::debug!(
                                        target: "repo_db",
                                        prompt_id = %prompt_id,
                                        "agy CLI process exited successfully"
                                    );
                                    if let Ok(conn) = super::schema::connect_db() {
                                        let now = Utc::now().timestamp();
                                        // Justification: best-effort completion status update; prevents finished prompts from being re-backed-up
                                        crate::error::record_ignored(
                                            conn.execute(
                                                "UPDATE active_prompts SET status = 'completed', updated_at = ?1 WHERE id = ?2 AND status IN ('running', 'dispatched', 'in_flight')",
                                                rusqlite::params![now, &prompt_id],
                                            ),
                                            "mark prompt completed on agy exit",
                                        );
                                    }
                                    if let Ok(mut map) = super::state::get_memory_prompts_map().lock() {
                                        if let Some(p) = map.get_mut(&prompt_id) {
                                            p.status = "completed".to_string();
                                            p.updated_at = Utc::now().timestamp();
                                        }
                                    }
                                }
                                drain_next_queued_prompt_for_workspace(&inst_id_clone, &repo_path_clone);
                            }
                            Err(e) => {
                                {
                                    let mut workers = match get_active_agy_workers().lock() {
                                        Ok(guard) => guard,
                                        Err(poisoned) => poisoned.into_inner(),
                                    };
                                    workers.remove(&ws_key_clone);
                                }
                                tracing::warn!(
                                    target: "repo_db",
                                    prompt_id = %prompt_id,
                                    error = %e,
                                    "Failed to wait on agy CLI child process"
                                );
                                drain_next_queued_prompt_for_workspace(&inst_id_clone, &repo_path_clone);
                            }
                        }
                    });

                if let Err(e) = monitor_res {
                    tracing::warn!(target: "repo_db", error = %e, "Failed to spawn agy monitor thread");
                }

                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    target: "repo_db",
                    prompt_id = %prompt.id,
                    error = %e,
                    "Failed to spawn agy execution"
                );
                let message = format!("failed to start agy for prompt '{}': {e}", prompt.id);
                crate::modules::logger::log_error(&format!("[RepoDB] {message}"));
                Err(message)
            }
        }
    } else {
        let message = "agy executable was not found".to_string();
        crate::modules::logger::log_warn(&format!("[RepoDB] {message}"));
        Err(message)
    }
}

/// Helper to execute a clipboard helper process, explicitly write data, drop stdin to signal EOF,
/// and wait with a timeout to prevent deadlocks or indefinite hanging.
fn run_clipboard_helper_process(
    mut cmd: std::process::Command,
    data: &[u8],
    helper_name: &'static str,
) -> bool {
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let write_res = stdin.write_all(data);
                crate::error::record_ignored(
                    write_res,
                    &format!("write clipboard stdin ({helper_name})"),
                );
                // Explicitly drop stdin so EOF is sent to the child process pipe
                drop(stdin);
            }

            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(3);
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => return status.success(),
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            tracing::warn!(
                                target: "repo_db",
                                helper = helper_name,
                                "Clipboard helper timed out after 3 seconds, terminating process"
                            );
                            let _ = child.kill();
                            return false;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(e) => {
                        tracing::warn!(
                            target: "repo_db",
                            helper = helper_name,
                            error = %e,
                            "Failed to wait for clipboard helper"
                        );
                        return false;
                    }
                }
            }
        }
        Err(e) => {
            tracing::debug!(
                target: "repo_db",
                helper = helper_name,
                error = %e,
                "Clipboard helper command not available or failed to spawn"
            );
            false
        }
    }
}

/// Cross-platform helper to copy text content to system clipboard
pub fn copy_to_system_clipboard(content: &str) {
    if content.trim().is_empty() {
        return;
    }
    let text = content.to_string();
    let spawn_res = std::thread::Builder::new()
        .name("clipboard-writer".into())
        .spawn(move || {
            let data = text.as_bytes();
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                let mut cmd = std::process::Command::new("clip");
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                run_clipboard_helper_process(cmd, data, "clip");
            }
            #[cfg(target_os = "macos")]
            {
                let cmd = std::process::Command::new("pbcopy");
                run_clipboard_helper_process(cmd, data, "pbcopy");
            }
            #[cfg(target_os = "linux")]
            {
                let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
                let mut has_copied = false;
                if is_wayland {
                    let cmd = std::process::Command::new("wl-copy");
                    has_copied = run_clipboard_helper_process(cmd, data, "wl-copy");
                }
                if !has_copied {
                    let mut cmd = std::process::Command::new("xclip");
                    cmd.args(["-selection", "clipboard"]);
                    has_copied = run_clipboard_helper_process(cmd, data, "xclip");
                }
                if !has_copied {
                    let mut cmd = std::process::Command::new("xsel");
                    cmd.args(["--clipboard", "--input"]);
                    run_clipboard_helper_process(cmd, data, "xsel");
                }
            }
        });

    if let Err(e) = spawn_res {
        tracing::warn!(target: "repo_db", error = %e, "Failed to spawn detached clipboard thread");
    }
}

/// Copy text on the calling thread and return the helper failure.
/// The webview Clipboard API rejects with "Document is not focused" when the
/// OS window is not the foreground window. `clip` / `pbcopy` / `wl-copy` do not
/// need that focus.
pub fn copy_text_to_clipboard_sync(content: &str) -> Result<(), String> {
    if content.is_empty() {
        return Err("clipboard text is empty".to_string());
    }
    let data = content.as_bytes();
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("clip");
        cmd.creation_flags(0x08000000);
        return run_clipboard_helper_result(cmd, data, "clip");
    }
    #[cfg(target_os = "macos")]
    {
        let cmd = std::process::Command::new("pbcopy");
        return run_clipboard_helper_result(cmd, data, "pbcopy");
    }
    #[cfg(target_os = "linux")]
    {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let mut errors: Vec<String> = Vec::new();
        if is_wayland {
            let cmd = std::process::Command::new("wl-copy");
            match run_clipboard_helper_result(cmd, data, "wl-copy") {
                Ok(()) => return Ok(()),
                Err(e) => errors.push(e),
            }
        }
        let mut xclip = std::process::Command::new("xclip");
        xclip.args(["-selection", "clipboard"]);
        match run_clipboard_helper_result(xclip, data, "xclip") {
            Ok(()) => return Ok(()),
            Err(e) => errors.push(e),
        }
        let mut xsel = std::process::Command::new("xsel");
        xsel.args(["--clipboard", "--input"]);
        match run_clipboard_helper_result(xsel, data, "xsel") {
            Ok(()) => return Ok(()),
            Err(e) => errors.push(e),
        }
        return Err(format!(
            "linux clipboard helpers failed: {}",
            errors.join(" | ")
        ));
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = data;
        Err("clipboard helper is not implemented for this operating system".to_string())
    }
}

fn run_clipboard_helper_result(
    mut cmd: std::process::Command,
    data: &[u8],
    helper_name: &'static str,
) -> Result<(), String> {
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn clipboard helper {helper_name}: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(data)
            .map_err(|e| format!("write clipboard stdin ({helper_name}): {e}"))?;
        drop(stdin);
    } else {
        return Err(format!("clipboard helper {helper_name} has no stdin pipe"));
    }

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                return Err(format!(
                    "clipboard helper {helper_name} exited with {status}"
                ));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let kill_note = match child.kill() {
                        Ok(()) => String::new(),
                        Err(kill_err) => format!("; kill failed: {kill_err}"),
                    };
                    return Err(format!(
                        "clipboard helper {helper_name} timed out after 3 seconds{kill_note}"
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                return Err(format!("wait for clipboard helper {helper_name}: {e}"));
            }
        }
    }
}

/// Send a prompt immediately for a specific instance, verifying process liveness,
/// caching PID, updating SQLite split DB, writing .antigravity_resume_task.json, and executing via agy.
pub fn send_prompt_now_for_instance(
    instance_id: &str,
    repo_path: &str,
    prompt_content: &str,
    conversation_id: Option<&str>,
) -> Result<ActivePrompt, String> {
    crate::modules::instance::scan_and_cache_all_running_instances();
    copy_to_system_clipboard(prompt_content);

    let clean_inst = if instance_id.trim().is_empty() || instance_id == "__default__" {
        "default"
    } else {
        instance_id.trim()
    };
    let canonical_inst = crate::modules::instance::resolve_instance_id(clean_inst)
        .unwrap_or_else(|_| clean_inst.to_string());

    // 1. Ensure instance is running with smart process cache and adaptive cold-start stabilization
    let was_cold_started =
        match crate::modules::instance::smart_run::ensure_instance_running_smart_detailed(
            &canonical_inst,
            Some(repo_path),
        ) {
            Ok(res) => {
                if res.was_cold_started {
                    crate::modules::logger::log_info(&format!(
                        "[ColdStartDispatch] Instance '{}' was cold-started (stabilized in {}ms, pid: {:?}). Waiting 1500ms settlement buffer for IDE window/extension initialization...",
                        canonical_inst, res.stabilization_ms, res.pid
                    ));
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    let (still_running, _, pids) =
                        crate::modules::instance::is_instance_process_running_smart(
                            &canonical_inst,
                        );
                    if still_running {
                        crate::modules::instance::focus_running_instance(&pids, Some(repo_path));
                    }
                }
                res.was_cold_started
            }
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] ensure_instance_running_smart_detailed warning for '{}': {}",
                    canonical_inst, e
                ));
                false
            }
        };

    // 2. Generate prompt ID and record in DB
    let prompt_id = format!("p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let clean_repo_name = Path::new(repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "project".to_string());
    let now = Utc::now().timestamp();

    let (extracted_img, img_paths) = extract_image_payload_or_path(prompt_content);
    let mut initial_img = extracted_img.clone();
    if let Some(ss_token) = extract_screenshot_id_token(prompt_content) {
        if let Some(ref cur) = initial_img {
            if !cur.contains(&ss_token) {
                initial_img = Some(format!("{};screenshot_id={}", cur, ss_token));
            }
        } else {
            initial_img = Some(format!("screenshot_id={}", ss_token));
        }
    }

    let active_prompt = ActivePrompt {
        id: prompt_id,
        project_id: clean_repo_name.clone(),
        instance_id: canonical_inst.clone(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_content.to_string(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: conversation_id.map(|s| s.to_string()),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: initial_img,
    };

    save_or_requeue_prompt(&active_prompt)?;

    // 3. Write resume task files to repo
    if !repo_path.trim().is_empty() {
        let ws_dir = PathBuf::from(repo_path);
        if ws_dir.exists() {
            let (extracted_uris, img_paths) = extract_image_uris_or_paths(prompt_content);
            let final_uris =
                effective_image_uris(active_prompt.image_payload.as_deref(), extracted_uris);
            let mut payload = resume_task_document(&active_prompt, "dispatched", now, &img_paths);
            if !final_uris.is_empty() {
                // Legacy single-string format for one URI; JSON array for many.
                payload["image_payload"] = if final_uris.len() == 1 {
                    serde_json::json!(final_uris[0].as_str())
                } else {
                    serde_json::json!(final_uris)
                };
            } else if !img_paths.is_empty() {
                // Legacy fallback: paths-only prompts previously stored the
                // joined paths in image_payload; preserve that behavior.
                payload["image_payload"] = serde_json::json!(img_paths.join(";"));
            }
            let serialized = serde_json::to_string_pretty(&payload).unwrap_or_default();
            // Justification: resume snapshot is a convenience file; the agy dispatch result below is the authoritative outcome
            crate::error::record_ignored(
                fs::write(ws_dir.join(".antigravity_resume_task.json"), &serialized),
                "write resume task snapshot",
            );
            // Justification: resume snapshot is a convenience file; the agy dispatch result below is the authoritative outcome
            crate::error::record_ignored(
                fs::write(
                    ws_dir.join(format!(".antigravity_resume_task.{}.json", canonical_inst)),
                    &serialized,
                ),
                "write per-instance resume task snapshot",
            );
        }
    }

    // 4. Dispatch: Prioritize live DevTools CDP submission if IDE is running, falling back to agy CLI.
    let clean_prompt = extract_clean_user_prompt(prompt_content);
    let fallback_prompt = if clean_prompt.trim().is_empty() {
        prompt_content.trim()
    } else {
        clean_prompt.as_str()
    };

    let mut dispatched_via_cdp = false;
    let (is_running, _, pids) =
        crate::modules::instance::is_instance_process_running_smart(&canonical_inst);
    if is_running {
        crate::modules::instance::focus_running_instance(&pids, Some(repo_path));
        let max_cdp_attempts = if was_cold_started { 3 } else { 1 };
        for attempt in 1..=max_cdp_attempts {
            if crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
                &canonical_inst,
                fallback_prompt,
            ) {
                dispatched_via_cdp = true;
                break;
            }
            if attempt < max_cdp_attempts {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }

    if dispatched_via_cdp {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Direct CDP dispatch succeeded for prompt '{}' on instance '{}' (cold-start: {})",
            active_prompt.id, canonical_inst, was_cold_started
        ));
        crate::modules::audit::record_if_active(
            "backend",
            "repo_db",
            "cdp_dispatch",
            &format!("instance:{canonical_inst}"),
            serde_json::json!({
                "promptId": active_prompt.id,
                "instanceId": canonical_inst,
                "repoPath": repo_path,
                "dispatch": "cdp_direct",
                "coldStarted": was_cold_started,
            }),
            "ok",
            "Prompt dispatched directly via live DevTools CDP into running IDE",
        );
    } else {
        // Fallback: Dispatch via agy CLI execution.
        // Explicit user Send/Rerun replaces any lingering agy worker for this workspace.
        if let Err(spawn_err) = spawn_or_replace_prompt_via_agy(&active_prompt, true) {
            tracing::warn!(
                target: "repo_db",
                prompt_id = %active_prompt.id,
                instance_id = %canonical_inst,
                repo_path = %repo_path,
                error = %spawn_err,
                "agy CLI dispatch failed and direct CDP was unavailable"
            );
            let focus_note = match crate::modules::instance::ensure_instance_running_for_dispatch(
                &canonical_inst,
                Some(repo_path),
            ) {
                Ok(pid) => format!("IDE focused (pid {pid})"),
                Err(focus_err) => format!("IDE focus failed: {focus_err}"),
            };
            return Err(format!(
                "{spawn_err}; direct CDP dispatch unavailable; {focus_note}"
            ));
        } else {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] send_prompt_now_for_instance dispatched prompt '{}' for instance '{}' in '{}' (cold-start: {}) via agy CLI",
                active_prompt.id, canonical_inst, repo_path, was_cold_started
            ));
        }
    }

    Ok(active_prompt)
}

/// Enqueue a prompt into the FIFO queue for a specific instance,
/// saving to SQLite split DB and writing .antigravity_resume_task.json with status queued.
pub fn enqueue_prompt_for_instance(
    instance_id: &str,
    repo_path: &str,
    prompt_content: &str,
    conversation_id: Option<&str>,
) -> Result<ActivePrompt, String> {
    let clean_prompt = extract_clean_user_prompt(prompt_content);
    let prompt_to_check = if clean_prompt.trim().is_empty() {
        prompt_content.trim()
    } else {
        clean_prompt.as_str()
    };
    if prompt_to_check.is_empty() {
        return Err("prompt content is empty".to_string());
    }
    crate::modules::instance::scan_and_cache_all_running_instances();
    copy_to_system_clipboard(prompt_content);

    let clean_inst = if instance_id.trim().is_empty() || instance_id == "__default__" {
        "default"
    } else {
        instance_id.trim()
    };
    let canonical_inst = crate::modules::instance::resolve_instance_id(clean_inst)
        .unwrap_or_else(|_| clean_inst.to_string());

    let prompt_id = format!("queued-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let clean_repo_name = Path::new(repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "project".to_string());
    let now = Utc::now().timestamp();

    let (extracted_img, img_paths) = extract_image_payload_or_path(prompt_content);
    let mut initial_img = extracted_img.clone();
    if let Some(ss_token) = extract_screenshot_id_token(prompt_content) {
        if let Some(ref cur) = initial_img {
            if !cur.contains(&ss_token) {
                initial_img = Some(format!("{};screenshot_id={}", cur, ss_token));
            }
        } else {
            initial_img = Some(format!("screenshot_id={}", ss_token));
        }
    }

    let active_prompt = ActivePrompt {
        id: prompt_id,
        project_id: clean_repo_name,
        instance_id: canonical_inst.clone(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_content.to_string(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: conversation_id.map(|s| s.to_string()),
        status: "queued".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: initial_img,
    };

    save_or_requeue_prompt(&active_prompt)?;

    if !repo_path.trim().is_empty() {
        let ws_dir = PathBuf::from(repo_path);
        if ws_dir.exists() {
            let (extracted_uris, img_paths) = extract_image_uris_or_paths(prompt_content);
            let final_uris =
                effective_image_uris(active_prompt.image_payload.as_deref(), extracted_uris);
            let mut payload = resume_task_document(&active_prompt, "queued", now, &img_paths);
            if !final_uris.is_empty() {
                // Legacy single-string format for one URI; JSON array for many.
                payload["image_payload"] = if final_uris.len() == 1 {
                    serde_json::json!(final_uris[0].as_str())
                } else {
                    serde_json::json!(final_uris)
                };
            } else if !img_paths.is_empty() {
                // Legacy fallback: paths-only prompts previously stored the
                // joined paths in image_payload; preserve that behavior.
                payload["image_payload"] = serde_json::json!(img_paths.join(";"));
            }
            payload["auto_boot"] = serde_json::json!(false);
            let serialized = serde_json::to_string_pretty(&payload).unwrap_or_default();
            // Justification: resume snapshot is a convenience file; the queued state is already recorded
            crate::error::record_ignored(
                fs::write(ws_dir.join(".antigravity_resume_task.json"), &serialized),
                "write resume task snapshot",
            );
            // Justification: resume snapshot is a convenience file; the queued state is already recorded
            crate::error::record_ignored(
                fs::write(
                    ws_dir.join(format!(".antigravity_resume_task.{}.json", canonical_inst)),
                    &serialized,
                ),
                "write per-instance resume task snapshot",
            );
        } else {
            tracing::warn!(
                target: "repo_db",
                prompt_id = %active_prompt.id,
                repo_path = %repo_path,
                "Workspace directory does not exist for enqueue_prompt_for_instance; skipping resume task write"
            );
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] Workspace directory does not exist for prompt '{}': '{}'",
                active_prompt.id, repo_path
            ));
        }
    }

    invalidate_prompt_tree_cache(Some(&canonical_inst));

    crate::modules::logger::log_info(&format!(
        "[RepoDB] enqueue_prompt_for_instance recorded prompt '{}' for instance '{}' in '{}'",
        active_prompt.id, canonical_inst, repo_path
    ));

    Ok(active_prompt)
}

/// Drain the oldest FIFO `queued` prompt (`ORDER BY created_at ASC, id ASC`)
/// for `(instance_id, repo_path)` when an `agy` worker finishes.
pub fn drain_next_queued_prompt_for_workspace(instance_id: &str, repo_path: &str) {
    let conn = match super::schema::connect_db() {
        Ok(c) => c,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] drain_next_queued_prompt_for_workspace connect_db failed: {err}"
            ));
            return;
        }
    };
    let mut stmt = match conn.prepare(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts
         WHERE instance_id = ?1 AND repo_path = ?2 AND status = 'queued'
         ORDER BY created_at ASC, id ASC
         LIMIT 1",
    ) {
        Ok(s) => s,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] drain_next_queued_prompt_for_workspace prepare failed: {err}"
            ));
            return;
        }
    };
    let next_prompt: Option<ActivePrompt> = stmt
        .query_row(rusqlite::params![instance_id, repo_path], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .ok();

    let Some(mut prompt) = next_prompt else {
        return;
    };

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    let prompt_to_check = if clean_prompt.trim().is_empty() {
        prompt.prompt_content.trim()
    } else {
        clean_prompt.as_str()
    };
    if prompt_to_check.is_empty() {
        // Justification: purge corrupt empty queued prompt row so it cannot block the FIFO queue
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM active_prompts WHERE id = ?1",
                rusqlite::params![&prompt.id],
            ),
            "delete empty queued prompt row",
        );
        return;
    }

    let now = Utc::now().timestamp();
    prompt.status = "running".to_string();
    prompt.updated_at = now;
    if let Err(err) = conn.execute(
        "UPDATE active_prompts SET status = 'running', updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, &prompt.id],
    ) {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] drain_next_queued_prompt_for_workspace update status failed for '{}': {err}",
            prompt.id
        ));
        return;
    }

    if let Ok(mut map) = super::state::get_memory_prompts_map().lock() {
        if let Some(p) = map.get_mut(&prompt.id) {
            p.status = "running".to_string();
            p.updated_at = now;
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Auto-draining next FIFO queued prompt '{}' for instance '{}' in '{}'",
        prompt.id, instance_id, repo_path
    ));

    if let Err(err) = spawn_or_replace_prompt_via_agy(&prompt, false) {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Auto-drain spawn failed for queued prompt '{}': {err}",
            prompt.id
        ));
    }
    invalidate_prompt_tree_cache(Some(instance_id));
}

#[cfg(test)]
mod agy_flag_tests {
    use super::agy_conversation_id;

    #[test]
    fn conversation_flag_keeps_real_ids_and_skips_synthetic_keys() {
        assert_eq!(agy_conversation_id("cid-e2e-probe"), Some("cid-e2e-probe"));
        assert_eq!(agy_conversation_id("  abc  "), Some("abc"));
        assert_eq!(agy_conversation_id("conv-11111111"), None);
        assert_eq!(agy_conversation_id("-"), None);
        assert_eq!(agy_conversation_id("p-12345678"), None);
        assert_eq!(agy_conversation_id("queued-12345678"), None);
        assert_eq!(agy_conversation_id("repo__default"), None);
        assert_eq!(agy_conversation_id("C:/repo"), None);
        assert_eq!(agy_conversation_id(""), None);
    }
}
