//! Repo DB: agy

use super::detection::{extract_image_payload_or_path, resume_task_document};
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

/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Strips prompt envelope wrappers and executes cleanly via -p without failing on GUI trajectory IDs.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool {
    let ws_dir = PathBuf::from(&prompt.repo_path);
    if !ws_dir.exists() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Workspace directory does not exist: '{}', skipping agy spawn",
            prompt.repo_path
        ));
        return false;
    }

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    if clean_prompt.trim().is_empty() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Prompt content for '{}' is empty, skipping agy spawn",
            prompt.id
        ));
        return false;
    }

    let ws_key = format!("{}:{}", prompt.instance_id, prompt.repo_path);
    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        if let Some(&existing_pid) = workers.get(&ws_key) {
            let mut sys = sysinfo::System::new();
            let target_pid = sysinfo::Pid::from_u32(existing_pid);
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            );
            if sys.process(target_pid).is_some() {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] An agy worker (PID: {}) is already active for workspace '{}'. Prompt '{}' was not sent again.",
                    existing_pid, ws_key, prompt.id
                ));
                return false;
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
            let s = sess.trim();
            if !s.is_empty() && !s.starts_with("conv-") {
                cmd.arg("--resume").arg(s);
            }
        }

        if clean_prompt.len() <= 24000 {
            cmd.arg("-p").arg(&clean_prompt);
        } else {
            cmd.arg("-p").arg(&clean_prompt[..24000]);
        }

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
                    if !inst.data_dir.trim().is_empty() {
                        cmd.arg("--user-data-dir").arg(&inst.data_dir);
                    }
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
                let child_pid = child.id();
                {
                    let mut workers = get_active_agy_workers().lock().unwrap();
                    workers.insert(ws_key.clone(), child_pid);
                }
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                    prompt.id, child_pid, prompt.instance_id, prompt.repo_path, clean_prompt
                ));

                let prompt_id = prompt.id.clone();
                let ws_key_clone = ws_key.clone();
                let monitor_res = std::thread::Builder::new()
                    .name(format!("agy-monitor-{}", prompt_id))
                    .spawn(move || {
                        let mut stderr_reader = child.stderr.take();
                        match child.wait() {
                            Ok(status) => {
                                {
                                    let mut workers = get_active_agy_workers().lock().unwrap();
                                    workers.remove(&ws_key_clone);
                                }
                                if !status.success() {
                                    let mut err_msg = String::new();
                                    if let Some(mut reader) = stderr_reader {
                                        use std::io::Read;
                                        let _ = reader.read_to_string(&mut err_msg);
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
                                }
                            }
                            Err(e) => {
                                {
                                    let mut workers = get_active_agy_workers().lock().unwrap();
                                    workers.remove(&ws_key_clone);
                                }
                                tracing::warn!(
                                    target: "repo_db",
                                    prompt_id = %prompt_id,
                                    error = %e,
                                    "Failed to wait on agy CLI child process"
                                );
                            }
                        }
                    });

                if let Err(e) = monitor_res {
                    tracing::warn!(target: "repo_db", error = %e, "Failed to spawn agy monitor thread");
                }

                true
            }
            Err(e) => {
                tracing::warn!(
                    target: "repo_db",
                    prompt_id = %prompt.id,
                    error = %e,
                    "Failed to spawn agy execution"
                );
                crate::modules::logger::log_error(&format!(
                    "[RepoDB] Failed to spawn agy execution for prompt '{}': {}",
                    prompt.id, e
                ));
                false
            }
        }
    } else {
        crate::modules::logger::log_warn(
            "[RepoDB] agy executable not found, cannot resume prompt via CLI",
        );
        false
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

    // 1. Ensure instance is running with smart process cache (reuses existing process without reopening)
    if let Err(e) =
        crate::modules::instance::ensure_instance_running_smart(&canonical_inst, Some(repo_path))
    {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] ensure_instance_running_smart warning for '{}': {}",
            canonical_inst, e
        ));
    }

    // 2. Generate prompt ID and record in DB
    let prompt_id = format!("p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let clean_repo_name = Path::new(repo_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "project".to_string());
    let now = Utc::now().timestamp();

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
        image_payload: None,
    };

    save_or_requeue_prompt(&active_prompt)?;

    // 3. Write resume task files to repo
    if !repo_path.trim().is_empty() {
        let ws_dir = PathBuf::from(repo_path);
        if ws_dir.exists() {
            let (extracted_img, img_paths) = extract_image_payload_or_path(prompt_content);
            let final_img = active_prompt.image_payload.clone().or(extracted_img);
            let mut payload = resume_task_document(&active_prompt, "dispatched", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
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

    // 4. Dispatch via agy CLI execution with Tier 2 graceful degradation.
    // If agy CLI executable is absent or another worker is active, the prompt
    // has already been safely persisted to SQLite, copied to system clipboard,
    // and written to .antigravity_resume_task.json. We ensure the IDE instance
    // is running/focused and complete the dispatch successfully.
    let spawned = spawn_prompt_via_agy(&active_prompt);
    if !spawned {
        tracing::warn!(
            target: "repo_db",
            prompt_id = %active_prompt.id,
            instance_id = %canonical_inst,
            repo_path = %repo_path,
            "agy CLI dispatch failed or inactive; degraded to workspace resume task & clipboard fallback"
        );
        let _ = crate::modules::instance::ensure_instance_running_for_dispatch(
            &canonical_inst,
            Some(repo_path),
        );
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] agy CLI bridge inactive or busy for prompt '{}'; saved to .antigravity_resume_task.json & clipboard, instance '{}' focused",
            active_prompt.id, canonical_inst
        ));
    } else {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] send_prompt_now_for_instance dispatched prompt '{}' for instance '{}' in '{}'",
            active_prompt.id, canonical_inst, repo_path
        ));
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
        image_payload: None,
    };

    save_or_requeue_prompt(&active_prompt)?;

    if !repo_path.trim().is_empty() {
        let ws_dir = PathBuf::from(repo_path);
        if ws_dir.exists() {
            let (extracted_img, img_paths) = extract_image_payload_or_path(prompt_content);
            let final_img = active_prompt.image_payload.clone().or(extracted_img);
            let mut payload = resume_task_document(&active_prompt, "queued", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
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
