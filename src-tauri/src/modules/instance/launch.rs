//! IDE launch pipeline with extra workspaces.
use super::*;
use crate::error::AppError;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub(crate) fn launch_instance_inner_with_extra_workspaces(
    instance_id: &str,
    reinject_prompts: bool,
    extra_workspaces: Option<&[String]>,
) -> Result<(), crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    let inst_config = registry.instances[pos].clone();
    let data_dir = registry.instances[pos].data_dir.clone();
    let is_default = registry.instances[pos].is_default;
    let custom_exe = registry.instances[pos].executable_path.clone();
    let extensions_dir = registry.instances[pos].extensions_dir.clone();
    let bound_acc = registry.instances[pos].bound_account_id.clone();
    let bound_email = registry.instances[pos].bound_email.clone();
    registry.instances[pos].last_used = chrono::Utc::now().timestamp();
    registry.active_instance_id = instance_id.to_string();
    save_registry(&registry).map_err(crate::error::AppError::Config)?;

    let target_data_path = PathBuf::from(&data_dir);

    // Snapshot bound workspace folders BEFORE closing existing instance processes
    let mut workspace_folders = get_instance_workspace_folders(instance_id, &data_dir);
    if let Some(extras) = extra_workspaces {
        for folder in extras {
            if Path::new(folder).exists() && !workspace_folders.contains(folder) {
                workspace_folders.push(folder.clone());
            }
        }
    }

    // Determine executable FIRST while running processes are alive for discovery
    let exe_path = resolve_launch_executable(instance_id, is_default, custom_exe.as_deref())?;

    // Guard against killing running instances:
    // If the instance process is verified to be alive and running, NEVER kill it with close_instance!
    let (is_already_running, _, _) = is_instance_process_running_smart(instance_id);
    if !is_already_running {
        // Guard: do not kill running processes when !is_already_running if another instance is alive
        let (other_pids, _) = other_instance_protection(instance_id);
        let has_other_alive = other_pids.iter().any(|&p| is_pid_alive_targeted(p));
        if !has_other_alive {
            // Close only the existing process for THIS target instance if running, allowing OS to unmap locks
            // Justification: best-effort close; close is idempotent and re-attempted on the next action
            crate::error::record_ignored(close_instance(instance_id), "close instance");
            std::thread::sleep(std::time::Duration::from_millis(300));
        } else {
            crate::modules::logger::log_info(&format!(
                "[Instance] Another instance is currently alive; bypassing close_instance for '{}' to protect active processes.",
                instance_id
            ));
        }
        // Clean any orphaned lock files in the target instance data directory
        clear_stale_instance_lockfiles(&target_data_path);
    } else {
        crate::modules::logger::log_info(&format!(
            "[Instance] Target instance '{}' is already running; bypassing close_instance and preserving active process locks.",
            instance_id
        ));
    }

    // If an account is bound to this instance profile, sync credentials and inject token directly into isolated state.vscdb and system keyring AFTER process exit.
    let resolved_account = if let Some(ref account_id) = bound_acc {
        crate::modules::account::load_account(account_id)
            .ok()
            .or_else(|| {
                bound_email.as_ref().and_then(|em| {
                    crate::modules::account::get_account_by_email(em)
                        .ok()
                        .flatten()
                })
            })
    } else if let Some(ref email) = bound_email {
        crate::modules::account::get_account_by_email(email)
            .ok()
            .flatten()
    } else if is_default {
        crate::modules::account::get_current_account()
            .ok()
            .flatten()
    } else {
        None
    };

    if let Some(account) = resolved_account.as_ref() {
        // Justification: best-effort credential seeding on launch; the account flow re-derives credentials on demand
        crate::error::record_ignored(
            inject_account_credentials(
                instance_id,
                &data_dir,
                is_default,
                account,
                &CredentialInjectOptions::for_launch(),
            ),
            "inject account credentials",
        );
    }

    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&inst_config),
        "inject instance settings",
    );

    let exe_str = exe_path.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        // Ensure candidate executable paths for Antigravity also check user-level $HOME/Applications/Antigravity.app:
        // If exe_str is not found at /Applications/Antigravity.app, check $HOME/Applications/Antigravity.app.
        let mut exe_str = exe_str;
        if !Path::new(&exe_str).exists() {
            if exe_str == "/Applications/Antigravity.app"
                || exe_str.starts_with("/Applications/Antigravity.app")
            {
                if let Some(home) = dirs::home_dir() {
                    let user_app = home.join("Applications").join("Antigravity.app");
                    if user_app.exists() {
                        exe_str = user_app.to_string_lossy().to_string();
                    }
                }
            }
            if !Path::new(&exe_str).exists() {
                for candidate in get_macos_candidate_paths() {
                    if candidate.exists() {
                        exe_str = candidate.to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }

        let is_app_bundle = exe_str.ends_with(".app") || Path::new(&exe_str).is_dir();

        let mut app_args = Vec::new();
        let has_custom_data = !is_default;
        let inst_home_opt = if has_custom_data {
            app_args.push(format!("--user-data-dir={}", data_dir));
            app_args.push("--password-store=basic".to_string());
            app_args.push("--remote-debugging-port=0".to_string());
            let home_opt = get_instance_home_dir(instance_id).ok();
            write_keyring_bypass_markers(&target_data_path, home_opt.as_deref());
            if let Some(ref inst_home) = home_opt {
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(inst_home), "create directory");
                let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
                let gemini_dir = inst_home.join(".gemini").join("antigravity");
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(
                    fs::create_dir_all(&gemini_ide_dir),
                    "create directory",
                );
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(&gemini_dir), "create directory");

                // Ensure app_storage.json has ide-install-wizard-shown: true and reflects the bound account
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                // Justification: best-effort storage sync; re-synced on every launch and account switch
                crate::error::record_ignored(
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos),
                    "sync app_storage.json",
                );
            } else {
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                // Justification: best-effort storage sync; re-synced on every launch and account switch
                crate::error::record_ignored(
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos),
                    "sync app_storage.json",
                );
            }
            home_opt
        } else {
            None
        };

        if let Some(ref ext_dir) = extensions_dir {
            app_args.push(format!("--extensions-dir={}", ext_dir));
        }
        if !workspace_folders.is_empty() {
            for folder in &workspace_folders {
                app_args.push(folder.clone());
            }
        }

        let args = if app_args.is_empty() {
            None
        } else {
            Some(app_args)
        };

        let mut cmd = if is_app_bundle {
            let mut c = Command::new("open");
            let open_args =
                crate::modules::process::format_macos_open_args(&exe_str, args.as_deref(), true);
            c.args(&open_args);
            c
        } else {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                let mut permissions = metadata.permissions();
                let mode = permissions.mode();
                if mode & 0o111 == 0 {
                    permissions.set_mode(mode | 0o755);
                    // Justification: permission hardening; the file stays usable with its existing mode if this fails
                    crate::error::record_ignored(
                        std::fs::set_permissions(&exe_path, permissions),
                        "set file permissions",
                    );
                }
            }
            let mut c = Command::new(&exe_str);
            if let Some(parent) = exe_path.parent() {
                c.current_dir(parent);
            }
            if let Some(ref arg_list) = args {
                for arg in arg_list {
                    c.arg(arg);
                }
            }
            c.arg("--new-window");
            c
        };

        if has_custom_data {
            if let Some(ref inst_home) = inst_home_opt {
                cmd.env("HOME", inst_home);
            }
            cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
            cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
            cmd.env("SSH_TTY", "pty/0");
            if let Some(ref acc) = resolved_account {
                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
            }
        }

        cmd.env("RUST_BACKTRACE", "1");

        if is_app_bundle {
            cmd.stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
        } else {
            cmd.stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }

        let child = cmd.spawn().map_err(|e| {
            let trace = std::backtrace::Backtrace::capture();
            let trace_str = format!("{}. Backtrace:\n{:?}", e, trace);
            crate::modules::process::append_ide_discovery_log(
                false,
                Some("FAILED"),
                &exe_str,
                "launch_attempt",
                None,
                &trace_str,
            );
            crate::modules::logger::log_error(&format!(
                "[Instance] Failed to spawn macOS instance process (exe: {}, is_app_bundle: {}): {}. Backtrace:\n{:?}",
                exe_str, is_app_bundle, e, trace
            ));
            crate::error::AppError::Process(format!(
                "Failed to spawn macOS instance process: {} (trace: {:?})",
                e, trace
            ))
        })?;
        let child_pid = child.id();

        if is_app_bundle {
            match child.wait_with_output() {
                Ok(output) => {
                    if !output.status.success() {
                        let err_msg = String::from_utf8_lossy(&output.stderr);
                        let trace = std::backtrace::Backtrace::capture();
                        let trace_str = format!(
                            "macOS open exited with code {:?}: {}. Backtrace:\n{:?}",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        );
                        crate::modules::process::append_ide_discovery_log(
                            false,
                            Some("FAILED"),
                            &exe_str,
                            "open_command_exit",
                            None,
                            &trace_str,
                        );
                        crate::modules::logger::log_error(&format!(
                            "[Instance] macOS open command failed (code {:?}): {}. Backtrace:\n{:?}",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        ));
                        return Err(crate::error::AppError::Process(format!(
                            "Failed to open macOS application bundle (exit code {:?}): {} (trace: {:?})",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        )));
                    }
                }
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Failed to wait on macOS open child process: {}",
                        e
                    ));
                }
            }
        }
        // /usr/bin/open exits quickly (~20ms).
        // Discover the true Antigravity process PID post-launch to prevent storing transient wrapper PID.
        std::thread::sleep(std::time::Duration::from_millis(500));
        let real_pids = find_pids_for_data_dir(&data_dir, is_default);
        let actual_pid = real_pids.first().copied().unwrap_or(child_pid);
        // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
        crate::error::record_ignored(
            record_instance_pid(instance_id, actual_pid, &data_dir),
            "record instance pid",
        );
        if reinject_prompts {
            wait_for_instance_prompt_channel(instance_id);
            // Justification: background restore; prompts are re-restored on the next launch
            crate::error::record_ignored(
                crate::modules::backup_prompts_db::restore_running_prompts(
                    Some(instance_id),
                    false,
                    None,
                ),
                "restore running prompts",
            );
            // Justification: background resend; the scheduler re-attempts on its next tick
            crate::error::record_ignored(
                crate::modules::repo_db::resend_running_commands_for_instance(
                    Some(instance_id),
                    20,
                ),
                "resend running commands",
            );
            // Justification: background dispatch; the scheduler re-attempts on its next tick
            crate::error::record_ignored(
                crate::modules::repo_db::dispatch_running_prompts(instance_id),
                "dispatch running prompts",
            );
            // Justification: intentionally discards the ensured-goals count; the function is infallible and the scheduler re-verifies on its next tick
            let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        }
        crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(&exe_str);
        cmd.env("RUST_BACKTRACE", "1");

        let has_custom_data = !is_default;
        if has_custom_data {
            cmd.arg(format!("--user-data-dir={}", data_dir));
            cmd.arg("--password-store=basic");
            cmd.arg("--remote-debugging-port=0");
            let inst_home_opt = get_instance_home_dir(instance_id).ok();
            write_keyring_bypass_markers(&target_data_path, inst_home_opt.as_deref());

            if let Some(ref inst_home) = inst_home_opt {
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(inst_home), "create directory");
                let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
                let gemini_dir = inst_home.join(".gemini").join("antigravity");
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(
                    fs::create_dir_all(&gemini_ide_dir),
                    "create directory",
                );
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(&gemini_dir), "create directory");

                // Ensure app_storage.json has ide-install-wizard-shown: true and reflects the bound account
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                // Justification: best-effort storage sync; re-synced on every launch and account switch
                crate::error::record_ignored(
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos),
                    "sync app_storage.json",
                );

                #[cfg(not(target_os = "windows"))]
                {
                    cmd.env("HOME", inst_home);
                }
                #[cfg(target_os = "windows")]
                {
                    cmd.env("USERPROFILE", inst_home);
                    cmd.env("APPDATA", inst_home.join("AppData").join("Roaming"));
                    cmd.env("LOCALAPPDATA", inst_home.join("AppData").join("Local"));
                }
            }
            if let Some(ref acc) = resolved_account {
                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
            }
        }
        if let Some(ref ext_dir) = extensions_dir {
            cmd.arg(format!("--extensions-dir={}", ext_dir));
        }
        if !workspace_folders.is_empty() {
            for folder in &workspace_folders {
                cmd.arg(folder);
            }
        } else {
            cmd.arg("--new-window");
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        {
            // 0x00000200: CREATE_NEW_PROCESS_GROUP
            // 0x01000000: CREATE_BREAKAWAY_FROM_JOB
            // Note: NEVER add 0x08000000 (CREATE_NO_WINDOW) to a GUI application!
            cmd.creation_flags(0x00000200 | 0x01000000);
        }

        #[cfg(target_os = "linux")]
        {
            crate::modules::process::clean_appimage_env(&mut cmd);
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                let mut permissions = metadata.permissions();
                let mode = permissions.mode();
                if mode & 0o111 == 0 {
                    permissions.set_mode(mode | 0o755);
                    // Justification: permission hardening; the file stays usable with its existing mode if this fails
                    crate::error::record_ignored(
                        std::fs::set_permissions(&exe_path, permissions),
                        "set file permissions",
                    );
                }
            }
        }

        // Justification: settings injection is best-effort; the instance still launches with defaults
        crate::error::record_ignored(
            inject_instance_settings(&inst_config),
            "inject instance settings",
        );

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!("Failed to spawn instance process: {}", e))
        })?;
        // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
        record_spawned_instance_process(instance_id, &data_dir, reinject_prompts, child.id())?;
        Ok(())
    }
}
