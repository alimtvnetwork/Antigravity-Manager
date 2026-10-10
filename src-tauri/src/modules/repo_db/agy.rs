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
            .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                let child_pid = child.id();
                {
                    let mut workers = get_active_agy_workers().lock().unwrap();
                    workers.insert(ws_key, child_pid);
                }
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                    prompt.id, child_pid, prompt.instance_id, prompt.repo_path, clean_prompt
                ));
                true
            }
            Err(e) => {
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

/// Cross-platform helper to copy text content to system clipboard
pub fn copy_to_system_clipboard(content: &str) {
    if content.trim().is_empty() {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(mut child) = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                crate::error::record_ignored(
                    std::io::Write::write_all(stdin, content.as_bytes()),
                    "write clipboard stdin (clip)",
                );
            }
            // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
            crate::error::record_ignored(child.wait(), "wait for clipboard helper (clip)");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                crate::error::record_ignored(
                    std::io::Write::write_all(stdin, content.as_bytes()),
                    "write clipboard stdin (pbcopy)",
                );
            }
            // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
            crate::error::record_ignored(child.wait(), "wait for clipboard helper (pbcopy)");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let mut has_copied = false;
        if is_wayland {
            if let Ok(mut child) = std::process::Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                    crate::error::record_ignored(
                        std::io::Write::write_all(stdin, content.as_bytes()),
                        "write clipboard stdin (wl-copy)",
                    );
                }
                if let Ok(status) = child.wait() {
                    has_copied = status.success();
                }
            }
        }
        if !has_copied {
            if let Ok(mut child) = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                    crate::error::record_ignored(
                        std::io::Write::write_all(stdin, content.as_bytes()),
                        "write clipboard stdin (xclip)",
                    );
                }
                if let Ok(status) = child.wait() {
                    has_copied = status.success();
                }
            }
        }
        if !has_copied {
            if let Ok(mut child) = std::process::Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                    crate::error::record_ignored(
                        std::io::Write::write_all(stdin, content.as_bytes()),
                        "write clipboard stdin (xsel)",
                    );
                }
                // Justification: clipboard copy is fire-and-forget; the caller has no recovery path
                crate::error::record_ignored(child.wait(), "wait for clipboard helper (xsel)");
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

    // 4. Dispatch via agy CLI execution. A failed spawn must surface as an
    // error: returning Ok here previously made the UI report "Dispatched!"
    // while nothing reached the IDE (agy binary missing, workspace dir
    // missing, or another worker active all return false from the spawner).
    let spawned = spawn_prompt_via_agy(&active_prompt);
    if !spawned {
        return Err(format!(
            "prompt '{}' recorded but agy CLI dispatch failed for instance '{}' (agy executable not found, workspace directory missing, or another dispatch already active for this workspace — see logs)",
            active_prompt.id, canonical_inst
        ));
    }
    crate::modules::logger::log_info(&format!(
        "[RepoDB] send_prompt_now_for_instance dispatched prompt '{}' for instance '{}' in '{}'",
        active_prompt.id, canonical_inst, repo_path
    ));

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
        }
    }

    invalidate_prompt_tree_cache(Some(&canonical_inst));

    crate::modules::logger::log_info(&format!(
        "[RepoDB] enqueue_prompt_for_instance recorded prompt '{}' for instance '{}' in '{}'",
        active_prompt.id, canonical_inst, repo_path
    ));

    Ok(active_prompt)
}
