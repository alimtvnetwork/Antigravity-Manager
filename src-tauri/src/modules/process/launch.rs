#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::thread;
use std::time::Duration;

use super::*;

/// Clean AppImage-specific environment variables before spawning external processes on Linux
#[cfg(target_os = "linux")]
pub fn clean_appimage_env(cmd: &mut Command) {
    let appimage_vars = [
        "APPIMAGE",
        "APPDIR",
        "ARGV0",
        "OWD",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "GTK_PATH",
        "GIO_EXTRA_MODULES",
        "GI_TYPELIB_PATH",
        "QT_PLUGIN_PATH",
        "QT_QPA_PLATFORM_PLUGIN_PATH",
    ];
    for var in &appimage_vars {
        cmd.env_remove(var);
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        let filtered_dirs: Vec<&str> = xdg_data_dirs
            .split(':')
            .filter(|dir| !dir.starts_with("/tmp/.mount_"))
            .collect();
        cmd.env("XDG_DATA_DIRS", filtered_dirs.join(":"));
    }
}

/// Helper to construct argument list for macOS `open` command.
///
/// Ensures `-n` and `-a` are passed, and that `--args` is added BEFORE any application flags
/// (such as `--new-window` or custom args), so they are never interpreted as `open` flags.
pub(crate) fn format_macos_open_args(
    app_identifier: &str,
    args: Option<&[String]>,
    new_window: bool,
) -> Vec<String> {
    let mut cmd_args = vec![
        "-n".to_string(),
        "-a".to_string(),
        app_identifier.to_string(),
    ];
    let valid_args: Vec<&str> = args
        .map(|a| {
            a.iter()
                .map(|s| s.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();

    if !valid_args.is_empty() || new_window {
        cmd_args.push("--args".to_string());
        for arg in valid_args {
            cmd_args.push(arg.to_string());
        }
        if new_window && !cmd_args.iter().any(|a| a == "--new-window") {
            cmd_args.push("--new-window".to_string());
        }
    }
    cmd_args
}

/// Start Antigravity with optional snapshot path & args fallback
#[allow(unused_mut)]
pub fn start_antigravity_with_fallback_path(
    target_ide: Option<&str>,
    preferred_path: Option<&std::path::Path>,
    preferred_args: Option<&[String]>,
) -> Result<(), String> {
    crate::modules::logger::log_info(&format!(
        "Starting Antigravity ({:?}, preferred_path: {:?})...",
        target_ide, preferred_path
    ));

    // Clean any stale lockfiles before starting new instance
    clean_antigravity_lockfiles(target_ide);

    // Prefer manually specified path and args from configuration
    let config = crate::modules::config::load_app_config().ok();
    let manual_path = if target_ide == Some("ide") {
        config
            .as_ref()
            .and_then(|c| c.antigravity_ide_executable.clone())
    } else {
        config
            .as_ref()
            .and_then(|c| c.antigravity_executable.clone())
    };
    let raw_args = config
        .and_then(|c| c.antigravity_args.clone())
        .or_else(|| preferred_args.map(|a| a.to_vec()));
    let args = raw_args.map(|a| sanitize_restart_args(&a));

    if let Some(mut path_str) = manual_path {
        let mut path = std::path::PathBuf::from(&path_str);

        #[cfg(target_os = "macos")]
        {
            // Fault tolerance: If path is inside .app bundle (e.g. misselected Helper), auto-correct to .app directory
            if let Some(app_idx) = path_str.find(".app") {
                let corrected_app = &path_str[..app_idx + 4];
                if corrected_app != path_str {
                    crate::modules::logger::log_info(&format!(
                        "Detected macOS path inside .app bundle, auto-correcting to: {}",
                        corrected_app
                    ));
                    path_str = corrected_app.to_string();
                    path = std::path::PathBuf::from(&path_str);
                }
            }
        }

        if path.exists() {
            crate::modules::logger::log_info(&format!(
                "Starting with manual configuration path: {}",
                path_str
            ));

            #[cfg(target_os = "macos")]
            {
                // macOS: if .app directory, use open
                if path_str.ends_with(".app") || path.is_dir() {
                    let mut cmd = Command::new("open");
                    cmd.env("RUST_BACKTRACE", "1");
                    let open_args = format_macos_open_args(&path_str, args.as_deref(), true);
                    cmd.args(&open_args);

                    let output = cmd.output().map_err(|e| {
                        let bt = std::backtrace::Backtrace::capture();
                        format!("Startup failed (open): {} | Stack: {:?}", e, bt)
                    })?;
                    if !output.status.success() {
                        let err_msg = String::from_utf8_lossy(&output.stderr);
                        let bt = std::backtrace::Backtrace::capture();
                        return Err(format!(
                            "Startup failed (open exit code: {:?}): {} | Stack: {:?}",
                            output.status.code(),
                            err_msg.trim(),
                            bt
                        ));
                    }
                } else {
                    let mut cmd = Command::new(&path_str);
                    cmd.env("RUST_BACKTRACE", "1");

                    // Add startup arguments
                    if let Some(ref args) = args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }
                    cmd.arg("--new-window");

                    cmd.spawn().map_err(|e| {
                        let bt = std::backtrace::Backtrace::capture();
                        format!("Startup failed (direct): {} | Stack: {:?}", e, bt)
                    })?;
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                let mut cmd = Command::new(&path_str);
                cmd.env("RUST_BACKTRACE", "1");

                if let Some(parent) = path.parent() {
                    cmd.current_dir(parent);
                }

                // Add startup arguments (preserve workspace folders without forcing blank window)
                if let Some(ref args) = args {
                    for arg in args {
                        cmd.arg(arg);
                    }
                }

                #[cfg(target_os = "windows")]
                {
                    cmd.creation_flags(0x00000200 | 0x01000000); // CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB
                }

                #[cfg(target_os = "linux")]
                clean_appimage_env(&mut cmd);

                cmd.spawn().map_err(|e| format!("Startup failed: {}", e))?;
            }

            crate::modules::logger::log_info(&format!(
                "Antigravity startup command sent (manual path: {}, args: {:?})",
                path_str, args
            ));
            return Ok(());
        } else {
            crate::modules::logger::log_warn(&format!(
                "Manual configuration path does not exist: {}, falling back to auto-detection",
                path_str
            ));
        }
    }

    // 次优：如果切换前捕获到了运行中进程的真实有效路径，优先使用它以防止非标准安装路径丢失
    if let Some(pref_path) = preferred_path {
        let pref_str = pref_path.to_string_lossy().to_lowercase();
        if pref_path.exists() && !pref_str.contains(".trash") {
            crate::modules::logger::log_info(&format!(
                "Starting with preferred snapshot process path: {:?}",
                pref_path
            ));

            #[cfg(target_os = "macos")]
            {
                let path_str = pref_path.to_string_lossy();
                let app_target = if let Some(app_idx) = path_str.find(".app") {
                    &path_str[..app_idx + 4]
                } else {
                    &*path_str
                };

                let mut cmd = Command::new("open");
                cmd.env("RUST_BACKTRACE", "1");
                let open_args = format_macos_open_args(app_target, args.as_deref(), false);
                cmd.args(&open_args);

                let output = cmd.output().map_err(|e| {
                    let bt = std::backtrace::Backtrace::capture();
                    format!("Execute open command failed: {} | Stack: {:?}", e, bt)
                })?;
                if !output.status.success() {
                    let err_msg = String::from_utf8_lossy(&output.stderr);
                    let bt = std::backtrace::Backtrace::capture();
                    return Err(format!(
                        "Startup failed (open exit code: {:?}): {} | Stack: {:?}",
                        output.status.code(),
                        err_msg.trim(),
                        bt
                    ));
                }
                crate::modules::logger::log_info(
                    "Antigravity startup command sent (macOS open snapshot path)",
                );
                return Ok(());
            }

            #[cfg(not(target_os = "macos"))]
            {
                let mut cmd = Command::new(pref_path);
                cmd.env("RUST_BACKTRACE", "1");

                if let Some(parent) = pref_path.parent() {
                    cmd.current_dir(parent);
                }

                if let Some(ref args) = args {
                    for arg in args {
                        cmd.arg(arg);
                    }
                }

                #[cfg(target_os = "windows")]
                {
                    cmd.creation_flags(0x00000200 | 0x01000000); // CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB
                }

                #[cfg(target_os = "linux")]
                clean_appimage_env(&mut cmd);

                cmd.spawn().map_err(|e| {
                    format!(
                        "Startup failed (preferred snapshot path {:?}): {}",
                        pref_path, e
                    )
                })?;
                crate::modules::logger::log_info(&format!(
                    "Antigravity startup command sent (snapshot path: {:?})",
                    pref_path
                ));
                return Ok(());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Improvement: Use output() to wait for open command completion and capture "app not found" error
        let mut cmd = Command::new("open");
        cmd.env("RUST_BACKTRACE", "1");
        let app_name = if target_ide == Some("ide") {
            "Antigravity IDE"
        } else {
            "Antigravity"
        };
        let open_args = format_macos_open_args(app_name, args.as_deref(), true);
        cmd.args(&open_args);

        let output = cmd.output().map_err(|e| {
            let bt = std::backtrace::Backtrace::capture();
            format!("Execute open command failed: {} | Stack: {:?}", e, bt)
        })?;
        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            let bt = std::backtrace::Backtrace::capture();
            return Err(format!(
                "Startup failed (open exit code: {:?}): {} | Stack: {:?}",
                output.status.code(),
                err_msg.trim(),
                bt
            ));
        }

        crate::modules::logger::log_info("Antigravity startup command sent (macOS open)");
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows/Linux Auto-detection and Startup
        match detect_antigravity_with_diagnostics(target_ide) {
            Ok(detected_path) => {
                let mut cmd = Command::new(&detected_path);
                cmd.env("RUST_BACKTRACE", "1");

                if let Some(parent) = detected_path.parent() {
                    cmd.current_dir(parent);
                }

                // Add startup arguments (preserve workspace folders without forcing blank window)
                if let Some(ref args) = args {
                    for arg in args {
                        cmd.arg(arg);
                    }
                }

                #[cfg(target_os = "windows")]
                {
                    cmd.creation_flags(0x00000200 | 0x01000000); // CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB
                }

                #[cfg(target_os = "linux")]
                clean_appimage_env(&mut cmd);

                cmd.spawn().map_err(|e| {
                    format!("Startup failed (detected path {:?}): {}", detected_path, e)
                })?;

                crate::modules::logger::log_info(&format!(
                    "Antigravity startup command sent (detected path: {:?})",
                    detected_path
                ));
                Ok(())
            }
            Err(diag_err) => {
                crate::modules::logger::log_error(&format!("[IDE Discovery Failed] {}", diag_err));
                Err(format!("{}", diag_err))
            }
        }
    }
}

/// Emergency repair: Tree-kill all stuck Antigravity/Electron processes, purge all lockfiles, and cleanly relaunch Antigravity
pub fn clean_and_restart_workspace(target_ide: Option<&str>) -> Result<String, String> {
    crate::modules::logger::log_info(
        "[Workspace] Executing emergency clean_and_restart_workspace...",
    );

    // 1. Force tree-kill all Antigravity processes
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(close_antigravity(5, target_ide), "close_antigravity");

    // 2. Extra safety sweep: purge any remaining target PIDs on Windows
    #[cfg(target_os = "windows")]
    {
        let remaining_pids = get_antigravity_pids(target_ide);
        for pid in remaining_pids {
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output(),
                "spawn taskkill",
            );
        }
    }

    // 3. Purge all lockfiles
    clean_antigravity_lockfiles(target_ide);

    // 4. Brief pause to ensure file handles are unmapped
    std::thread::sleep(Duration::from_millis(350));

    // 5. Clean relaunch
    start_antigravity(target_ide)?;

    crate::modules::logger::log_info("[Workspace] Clean and restart completed successfully.");
    Ok(
        "Stuck processes terminated, lockfiles purged, and Antigravity cleanly relaunched."
            .to_string(),
    )
}

/// Start Antigravity (wrapper using default discovery)
pub fn start_antigravity(target_ide: Option<&str>) -> Result<(), String> {
    start_antigravity_with_fallback_path(target_ide, None, None)
}
