#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

use super::*;

/// Close Antigravity processes
// timeout_secs 仅用于 macos/Linux 分支（graceful_timeout），Windows 分支不使用参数
#[cfg_attr(target_os = "windows", allow(unused_variables))]
pub fn close_antigravity(timeout_secs: u64, target_ide: Option<&str>) -> Result<(), String> {
    crate::modules::logger::log_info(&format!("Closing Antigravity ({:?})...", target_ide));

    #[cfg(target_os = "windows")]
    {
        // Windows: Precise tree kill by PID to eliminate parent and all Electron helpers
        let pids = get_antigravity_pids(target_ide);
        if !pids.is_empty() {
            crate::modules::logger::log_info(&format!(
                "Precisely closing {} identified processes on Windows (taskkill /F /T)...",
                pids.len()
            ));
            for pid in &pids {
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("taskkill")
                        .args(["/F", "/T", "/PID", &pid.to_string()])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .output(),
                    "spawn taskkill",
                );
            }
            thread::sleep(Duration::from_millis(300));
        }

        // Extra cleanup: If closing Antigravity (classic/client), also sweep any orphan language_server processes
        // that belong to the antigravity installation to prevent port/mutex locks blocking restarts.
        if target_ide != Some("ide") {
            sweep_orphan_language_servers();
        }

        // Safety fallback: re-sweep any lingering target PIDs if needed
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

        // Drain verification loop: wait up to 2500ms for all processes to completely exit
        let start_wait = std::time::Instant::now();
        let max_wait = Duration::from_millis(2500);
        let mut system = System::new();
        loop {
            system.refresh_processes(sysinfo::ProcessesToUpdate::All);
            let has_alive = pids
                .iter()
                .any(|&pid| system.process(sysinfo::Pid::from_u32(pid)).is_some());
            if !has_alive {
                break;
            }
            if start_wait.elapsed() > max_wait {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        // Clean stale lockfiles after process termination
        clean_antigravity_lockfiles(target_ide);
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Optimize closing strategy to avoid "Window terminated unexpectedly" popups
        // Strategy: SEND SIGTERM to main process only, let it coordinate closing children

        let pids = get_antigravity_pids(target_ide);
        if !pids.is_empty() {
            // 1. Identify main process (PID)
            let mut system = System::new();
            system.refresh_processes(sysinfo::ProcessesToUpdate::All);

            let mut main_pid = None;

            // Load manual configuration path as highest priority reference
            let manual_path = crate::modules::config::load_app_config()
                .ok()
                .and_then(|c| c.antigravity_executable)
                .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok());

            crate::modules::logger::log_info("Analyzing process list to identify main process:");
            for pid_u32 in &pids {
                let pid = sysinfo::Pid::from_u32(*pid_u32);
                if let Some(process) = system.process(pid) {
                    let name = process.name().to_string_lossy();
                    let args = process.cmd();
                    let args_str = args
                        .iter()
                        .map(|arg| arg.to_string_lossy().into_owned())
                        .collect::<Vec<String>>()
                        .join(" ");

                    crate::modules::logger::log_info(&format!(
                        " - PID: {} | Name: {} | Args: {}",
                        pid_u32, name, args_str
                    ));

                    // 1. Priority to manual path matching
                    if let (Some(ref m_path), Some(p_exe)) = (&manual_path, process.exe()) {
                        if let Ok(p_path) = p_exe.canonicalize() {
                            let m_path_str = m_path.to_string_lossy();
                            let p_path_str = p_path.to_string_lossy();
                            if let (Some(m_idx), Some(p_idx)) =
                                (m_path_str.find(".app"), p_path_str.find(".app"))
                            {
                                if m_path_str[..m_idx + 4] == p_path_str[..p_idx + 4] {
                                    // Deep validation: even if path matches, must exclude Helper keywords and arguments
                                    let is_helper_by_args = args_str.contains("--type=");
                                    let is_helper_by_name = name.to_lowercase().contains("helper")
                                        || name.to_lowercase().contains("plugin")
                                        || name.to_lowercase().contains("renderer")
                                        || name.to_lowercase().contains("gpu")
                                        || name.to_lowercase().contains("crashpad")
                                        || name.to_lowercase().contains("utility")
                                        || name.to_lowercase().contains("audio")
                                        || name.to_lowercase().contains("sandbox")
                                        || name.to_lowercase().contains("language_server");

                                    if !is_helper_by_args && !is_helper_by_name {
                                        main_pid = Some(pid_u32);
                                        crate::modules::logger::log_info(&format!(
                                            "   => Identified as main process (manual path match)"
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // 2. Feature analysis matching (fallback)
                    let is_helper_by_name = name.to_lowercase().contains("helper")
                        || name.to_lowercase().contains("crashpad")
                        || name.to_lowercase().contains("utility")
                        || name.to_lowercase().contains("audio")
                        || name.to_lowercase().contains("sandbox")
                        || name.to_lowercase().contains("language_server")
                        || name.to_lowercase().contains("plugin")
                        || name.to_lowercase().contains("renderer");

                    let is_helper_by_args = args_str.contains("--type=");

                    if !is_helper_by_name && !is_helper_by_args {
                        if main_pid.is_none() {
                            main_pid = Some(pid_u32);
                            crate::modules::logger::log_info(&format!(
                                "   => Identified as main process (Name/Args analysis)"
                            ));
                        }
                    } else {
                        crate::modules::logger::log_info(&format!(
                            "   => Identified as helper process (Helper/Args)"
                        ));
                    }
                }
            }

            // Phase 1: Graceful exit (SIGTERM)
            if let Some(pid) = main_pid {
                crate::modules::logger::log_info(&format!(
                    "Sending SIGTERM to main process PID: {}",
                    pid
                ));
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("kill")
                        .args(["-15", &pid.to_string()])
                        .output(),
                    "spawn kill",
                );
            } else {
                crate::modules::logger::log_warn(
                    "No main process identified, sending SIGTERM to all associated processes",
                );
                for pid in &pids {
                    // Justification: best-effort process spawn; failure logged
                    crate::error::record_ignored(
                        Command::new("kill")
                            .args(["-15", &pid.to_string()])
                            .output(),
                        "spawn kill",
                    );
                }
            }

            // Wait for graceful exit (max 70% of timeout_secs)
            let graceful_timeout = (timeout_secs * 7) / 10;
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(graceful_timeout) {
                if !is_antigravity_running(target_ide) {
                    crate::modules::logger::log_info("All Antigravity processes gracefully closed");
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(500));
            }

            // Phase 2: Force kill (SIGKILL) - targeting all remaining processes (Helpers)
            if is_antigravity_running(target_ide) {
                let remaining_pids = get_antigravity_pids(target_ide);
                if !remaining_pids.is_empty() {
                    crate::modules::logger::log_warn(&format!(
                        "Graceful exit timeout, force killing {} remaining processes (SIGKILL)",
                        remaining_pids.len()
                    ));
                    for pid in &remaining_pids {
                        let output = Command::new("kill").args(["-9", &pid.to_string()]).output();

                        if let Ok(result) = output {
                            if !result.status.success() {
                                let error = String::from_utf8_lossy(&result.stderr);
                                if !error.contains("No such process") {
                                    // "No matching processes" for killall, "No such process" for kill
                                    crate::modules::logger::log_error(&format!(
                                        "SIGKILL process {} failed: {}",
                                        pid, error
                                    ));
                                }
                            }
                        }
                    }
                    thread::sleep(Duration::from_secs(1));
                }

                // Final check
                if !is_antigravity_running(target_ide) {
                    crate::modules::logger::log_info("All processes exited after forced cleanup");
                    return Ok(());
                }
            } else {
                crate::modules::logger::log_info("All processes exited after SIGTERM");
                return Ok(());
            }
        } else {
            // Only consider not running when pids is empty, don't error here as it might already be closed
            crate::modules::logger::log_info("Antigravity not running, no need to close");
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: precise closing
        let pids = get_antigravity_pids(target_ide);
        if !pids.is_empty() {
            let mut system = System::new();
            system.refresh_processes(sysinfo::ProcessesToUpdate::All);

            let mut main_pid = None;

            for pid_u32 in &pids {
                let pid = sysinfo::Pid::from_u32(*pid_u32);
                if let Some(process) = system.process(pid) {
                    let name = process.name().to_string_lossy();
                    let args = process.cmd();
                    let args_str = args
                        .iter()
                        .map(|arg| arg.to_string_lossy().into_owned())
                        .collect::<Vec<String>>()
                        .join(" ");

                    let is_helper_by_name = name.to_lowercase().contains("helper")
                        || name.to_lowercase().contains("crashpad")
                        || name.to_lowercase().contains("utility")
                        || name.to_lowercase().contains("audio")
                        || name.to_lowercase().contains("sandbox")
                        || name.to_lowercase().contains("plugin")
                        || name.to_lowercase().contains("renderer");

                    let is_helper_by_args = args_str.contains("--type=");

                    if !is_helper_by_name && !is_helper_by_args {
                        main_pid = Some(pid_u32);
                        break;
                    }
                }
            }

            // Phase 1: SIGTERM
            if let Some(pid) = main_pid {
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("kill")
                        .args(["-15", &pid.to_string()])
                        .output(),
                    "spawn kill",
                );
            } else {
                crate::modules::logger::log_warn(
                    "No clear Linux main process identified, sending SIGTERM to all associated processes",
                );
                for pid in &pids {
                    // Justification: best-effort process spawn; failure logged
                    crate::error::record_ignored(
                        Command::new("kill")
                            .args(["-15", &pid.to_string()])
                            .output(),
                        "spawn kill",
                    );
                }
            }

            // Wait for graceful exit
            let graceful_timeout = (timeout_secs * 7) / 10;
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(graceful_timeout) {
                if !is_antigravity_running(target_ide) {
                    crate::modules::logger::log_info("Antigravity gracefully closed");
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(500));
            }

            // Phase 2: SIGKILL
            if is_antigravity_running(target_ide) {
                let remaining_pids = get_antigravity_pids(target_ide);
                if !remaining_pids.is_empty() {
                    crate::modules::logger::log_warn(&format!(
                        "Graceful exit timeout, force killing {} remaining processes (SIGKILL)",
                        remaining_pids.len()
                    ));
                    for pid in &remaining_pids {
                        // Justification: best-effort process spawn; failure logged
                        crate::error::record_ignored(
                            Command::new("kill").args(["-9", &pid.to_string()]).output(),
                            "spawn kill",
                        );
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        } else {
            crate::modules::logger::log_info(
                "No Antigravity processes found to close (possibly filtered or not running)",
            );
        }
    }

    // Final check with polling retry window (max 3 seconds, 150ms interval) to tolerate OS cleanup latency
    let final_check_start = std::time::Instant::now();
    let final_check_timeout = Duration::from_secs(3);

    while final_check_start.elapsed() < final_check_timeout {
        if !is_antigravity_running(target_ide) {
            crate::modules::logger::log_info("Antigravity closed successfully");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(150));
    }

    // If still running after 3 seconds, perform one last sweep kill on all remaining PIDs
    let remaining_pids = get_antigravity_pids(target_ide);
    if !remaining_pids.is_empty() {
        crate::modules::logger::log_warn(&format!(
            "Still running after timeout, attempting final sweep kill on PIDs: {:?}",
            remaining_pids
        ));
        for pid in &remaining_pids {
            #[cfg(target_os = "windows")]
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output(),
                "spawn taskkill",
            );

            #[cfg(not(target_os = "windows"))]
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(
                Command::new("kill").args(["-9", &pid.to_string()]).output(),
                "spawn kill",
            );
        }
        thread::sleep(Duration::from_millis(300));
    }

    if is_antigravity_running(target_ide) {
        return Err(
            "Unable to close Antigravity process, please close manually and retry".to_string(),
        );
    }

    crate::modules::logger::log_info("Antigravity closed successfully");
    Ok(())
}
