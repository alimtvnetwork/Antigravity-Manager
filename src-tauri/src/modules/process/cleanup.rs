use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

use super::*;

/// Clean stale lockfiles (Electron lockfile, code.lock, Singleton*) from data directories
pub fn clean_antigravity_lockfiles(target_ide: Option<&str>) {
    let mut candidate_dirs: Vec<std::path::PathBuf> = Vec::new();

    // 1) From active process user-data-dir
    if let Some(user_data_dir) = get_user_data_dir_from_process(target_ide) {
        candidate_dirs.push(user_data_dir);
    }

    // If targeting a specific instance, only clean that instance's directories and return
    if let Some(t) = target_ide {
        if let Some(inst_id) = t.strip_prefix("instance:") {
            if inst_id != "default" {
                if let Ok(instances_dir) = crate::modules::instance::get_instances_dir() {
                    let inst_root = instances_dir.join(inst_id);
                    candidate_dirs.push(inst_root.join("data"));
                    candidate_dirs.push(inst_root);
                }
            }
        }
    }

    // 2) Standard platform folders (for default client or IDE)
    let is_specific_instance = target_ide
        .map(|t| t.starts_with("instance:") && t != "instance:default")
        .unwrap_or(false);

    if !is_specific_instance {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let appdata_path = std::path::PathBuf::from(appdata);
                candidate_dirs.push(appdata_path.join("Antigravity"));
                candidate_dirs.push(appdata_path.join("Antigravity IDE"));
                candidate_dirs.push(appdata_path.join("antigravity"));
                candidate_dirs.push(appdata_path.join("antigravity-ide"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                let app_sup = home.join("Library/Application Support");
                candidate_dirs.push(app_sup.join("Antigravity"));
                candidate_dirs.push(app_sup.join("Antigravity IDE"));
                candidate_dirs.push(app_sup.join("antigravity"));
                candidate_dirs.push(app_sup.join("antigravity-ide"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let config_dir = home.join(".config");
                candidate_dirs.push(config_dir.join("Antigravity"));
                candidate_dirs.push(config_dir.join("Antigravity IDE"));
                candidate_dirs.push(config_dir.join("antigravity"));
                candidate_dirs.push(config_dir.join("antigravity-ide"));
            }
        }
    }

    for dir in candidate_dirs {
        if !dir.exists() {
            continue;
        }

        let lockfile = dir.join("lockfile");
        if lockfile.exists() {
            // Justification: best-effort cleanup; a leftover file is harmless
            crate::error::record_ignored(std::fs::remove_file(&lockfile), "remove_file");
            crate::modules::logger::log_info(&format!(
                "[Lockfile] Cleaned stale Electron lockfile: {:?}",
                lockfile
            ));
        }

        let code_lock = dir.join("code.lock");
        if code_lock.exists() {
            // Justification: best-effort cleanup; a leftover file is harmless
            crate::error::record_ignored(std::fs::remove_file(&code_lock), "remove_file");
            crate::modules::logger::log_info(&format!(
                "[Lockfile] Cleaned stale code.lock: {:?}",
                code_lock
            ));
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                let is_lock = fname == "lockfile"
                    || fname.starts_with("singleton")
                    || fname.ends_with(".lock")
                    || fname == "code.lock";
                if is_lock {
                    // Justification: best-effort cleanup; a leftover file is harmless
                    crate::error::record_ignored(std::fs::remove_file(entry.path()), "remove_file");
                }
            }
        }
    }
}

/// Extra cleanup: Kill orphan language_server processes located inside the Antigravity installation
pub fn sweep_orphan_language_servers() {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let exe_path = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        if (name.contains("language_server") || exe_path.contains("language_server"))
            && exe_path.contains("antigravity")
            && !exe_path.contains("antigravity ide")
            && !exe_path.contains("antigravity-ide")
        {
            if let Some(parent) = process.parent() {
                if system.process(parent).is_some() {
                    // Parent process is still active; this language_server is not an orphan
                    continue;
                }
            }

            let pid_u32 = pid.as_u32();
            crate::modules::logger::log_info(&format!(
                "Sweeping orphan language_server process (PID: {}, Path: {})",
                pid_u32, exe_path
            ));
            #[cfg(target_os = "windows")]
            {
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("taskkill")
                        .args(["/F", "/PID", &pid_u32.to_string()])
                        .creation_flags(0x08000000)
                        .output(),
                    "spawn taskkill",
                );
            }

            #[cfg(not(target_os = "windows"))]
            {
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("kill")
                        .args(["-9", &pid_u32.to_string()])
                        .output(),
                    "spawn kill",
                );
            }
        }
    }
}

/// `language_server` subprocess determination.
///
/// Coverage: Windows/Linux is `language_server` / `language_server.exe`, macOS is
/// `language_server_macos` / `language_server_macos_arm`.
///
/// Note: Narrow detection (language server only), distinct from broad helper detection.
pub(crate) fn is_language_server_process(name: &str, exe_path: &str) -> bool {
    name.to_lowercase().contains("language_server")
        || exe_path.to_lowercase().contains("language_server")
}

/// Force kill a single process (Windows uses `taskkill /F`, other platforms use `kill -9`).
pub(crate) fn force_kill_pid(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output(),
            "spawn taskkill",
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("kill").args(["-9", &pid.to_string()]).output(),
            "spawn kill",
        );
    }
}

/// Collect language_server subprocess PIDs for the target IDE.
pub(crate) fn language_server_subprocess_pids(
    system: &System,
    target_ide: Option<&str>,
) -> Vec<u32> {
    let roots: Vec<u32> = get_antigravity_pids(target_ide)
        .into_iter()
        .filter(|pid| {
            system
                .process(sysinfo::Pid::from_u32(*pid))
                .map(|process| {
                    let name = process.name().to_string_lossy().to_string();
                    let args = process
                        .cmd()
                        .iter()
                        .map(|s| s.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let exe = process
                        .exe()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    !is_helper_process(&name, &args, &exe)
                })
                .unwrap_or(false)
        })
        .collect();

    if roots.is_empty() {
        return Vec::new();
    }

    // Breadth-first traversal of all descendants
    let mut descendants: Vec<u32> = Vec::new();
    let mut frontier: Vec<u32> = roots.clone();
    let mut visited: std::collections::HashSet<u32> = roots.into_iter().collect();

    while let Some(parent) = frontier.pop() {
        for (pid, process) in system.processes() {
            if process.parent().map(|p| p.as_u32()) != Some(parent) {
                continue;
            }
            let pid_u32 = pid.as_u32();
            if visited.insert(pid_u32) {
                descendants.push(pid_u32);
                frontier.push(pid_u32);
            }
        }
    }

    descendants
        .into_iter()
        .filter(|pid_u32| {
            system
                .process(sysinfo::Pid::from_u32(*pid_u32))
                .map(|process| {
                    let name = process.name().to_string_lossy().to_string();
                    let exe = process
                        .exe()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    is_language_server_process(&name, &exe)
                })
                .unwrap_or(false)
        })
        .collect()
}

/// Hot-switch account: terminate only the language_server subprocess, preserving the main IDE window.
pub fn kill_language_server_subprocesses(target_ide: Option<&str>) -> Result<usize, String> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let pids = language_server_subprocess_pids(&system, target_ide);
    for pid_u32 in &pids {
        crate::modules::logger::log_info(&format!(
            "[HotSwitch] Terminating language_server subprocess (PID: {}, target: {:?})",
            pid_u32, target_ide
        ));
        force_kill_pid(*pid_u32);
    }

    Ok(pids.len())
}

/// Wait for language_server to respawn after hot switch.
pub fn wait_for_language_server_respawn(target_ide: Option<&str>, timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

    while std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(500));

        let mut system = System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All);
        if !language_server_subprocess_pids(&system, target_ide).is_empty() {
            return true;
        }
    }

    false
}
