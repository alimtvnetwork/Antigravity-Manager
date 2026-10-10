use sysinfo::System;

use super::*;

/// Check if Antigravity is running
pub fn is_antigravity_running(target_ide: Option<&str>) -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let ide_exe_paths = get_ide_exe_paths(&system);

    let current_exe = get_current_exe_path();
    let current_pid = std::process::id();

    // Load both manual paths from config
    let config = crate::modules::config::load_app_config().ok();
    let manual_path = config
        .as_ref()
        .and_then(|c| c.antigravity_executable.as_ref())
        .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok());
    let ide_manual_path = config
        .as_ref()
        .and_then(|c| c.antigravity_ide_executable.as_ref())
        .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok());

    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();
        if pid_u32 == current_pid {
            continue;
        }

        let name = process.name().to_string_lossy().to_lowercase();
        let exe_path = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Exclude own path (handles case where manager is mistaken for Antigravity on Linux)
        if let (Some(ref my_path), Some(p_exe)) = (&current_exe, process.exe()) {
            if let Ok(p_path) = p_exe.canonicalize() {
                if my_path == &p_path {
                    continue;
                }
            }
        }

        // Common helper process exclusion logic
        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase())
            .collect::<Vec<String>>()
            .join(" ");

        let is_helper = is_helper_process(&name, &args_str, &exe_path);

        if is_helper || is_non_ide_binary(&name, &exe_path, &args_str) {
            continue;
        }

        // Recognition ref 2: If targeting IDE and ide_manual_path is configured, check it first
        if target_ide == Some("ide") {
            if let (Some(ref ide_m_path), Some(p_exe)) = (&ide_manual_path, process.exe()) {
                if let Ok(p_path) = p_exe.canonicalize() {
                    #[cfg(target_os = "macos")]
                    {
                        let m = ide_m_path.to_string_lossy();
                        let p = p_path.to_string_lossy();
                        if let (Some(mi), Some(pi)) = (m.find(".app"), p.find(".app")) {
                            if m[..mi + 4] == p[..pi + 4] {
                                return true;
                            }
                        }
                    }
                    #[cfg(not(target_os = "macos"))]
                    if ide_m_path == &p_path {
                        return true;
                    }
                }
            }
        }

        // Recognition ref 3: Priority check for manual path match (client)
        if target_ide != Some("ide") {
            if let (Some(ref m_path), Some(p_exe)) = (&manual_path, process.exe()) {
                if let Ok(p_path) = p_exe.canonicalize() {
                    // macOS: Check if within the same .app bundle
                    #[cfg(target_os = "macos")]
                    {
                        let m_path_str = m_path.to_string_lossy();
                        let p_path_str = p_path.to_string_lossy();
                        if let (Some(m_idx), Some(p_idx)) =
                            (m_path_str.find(".app"), p_path_str.find(".app"))
                        {
                            if m_path_str[..m_idx + 4] == p_path_str[..p_idx + 4] {
                                return true;
                            }
                        }
                    }

                    #[cfg(not(target_os = "macos"))]
                    if m_path == &p_path {
                        return true;
                    }
                }
            }
        }

        // 3. Strict mode: If the relevant manual path is configured, we strictly enforce it
        // and DO NOT fallback to fuzzy string matching.
        if manual_path.is_some() && target_ide != Some("ide") {
            continue;
        }
        if ide_manual_path.is_some() && target_ide == Some("ide") {
            continue;
        }

        // If checking default (target_ide != Some("ide") and not starting with instance:),
        // we MUST ignore any process whose command line points to an isolated sandbox instance
        if target_ide != Some("ide")
            && !target_ide
                .map(|t| t.starts_with("instance:"))
                .unwrap_or(false)
        {
            let is_instance_sandbox = args_str.contains(".antigravity_tools")
                || args_str.contains("/instances/")
                || args_str.contains("\\instances\\");
            if is_instance_sandbox {
                continue;
            }
        }

        // Check if the process matches target_ide
        let exe_file = exe_file_name(&exe_path);
        let has_antigravity_name = name.contains("antigravity") || exe_file.contains("antigravity");

        let is_ide_match = if target_ide == Some("ide") {
            exe_path.contains("antigravity ide")
                || exe_path.contains("antigravity-ide")
                || name.contains("antigravity ide")
                || name.contains("antigravity-ide")
                || ide_exe_paths.contains(&exe_path)
        } else {
            if ide_exe_paths.contains(&exe_path) {
                false // Explicitly immune (it is an IDE)
            } else {
                has_antigravity_name
                    && !exe_path.contains("antigravity ide")
                    && !exe_path.contains("antigravity-ide")
                    && !name.contains("antigravity ide")
                    && !name.contains("antigravity-ide")
            }
        };

        if is_ide_match {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
/// Get PID set of current process and all ancestors.
/// Only ancestors (parents/grandparents) are excluded to prevent accidentally killing
/// the launcher or shell that started the Manager.
/// Child processes spawned by the Manager (e.g., the IDE) must remain killable, so
/// descendants are intentionally NOT included here.
pub(crate) fn get_self_family_pids(system: &sysinfo::System) -> std::collections::HashSet<u32> {
    let current_pid = std::process::id();
    let mut family_pids = std::collections::HashSet::new();
    family_pids.insert(current_pid);

    // Traverse upward to find all ancestors - prevent killing the launcher/shell
    let mut next_pid = current_pid;
    // Prevent infinite loop, max depth 10
    for _ in 0..10 {
        let pid_val = sysinfo::Pid::from_u32(next_pid);
        if let Some(process) = system.process(pid_val) {
            if let Some(parent) = process.parent() {
                let parent_id = parent.as_u32();
                // Avoid cycles or duplicates
                if !family_pids.insert(parent_id) {
                    break;
                }
                next_pid = parent_id;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    family_pids
}

/// Get PIDs of all Antigravity processes (including main and helper processes)
pub(crate) fn get_antigravity_pids(target_ide: Option<&str>) -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let ide_exe_paths = get_ide_exe_paths(&system);

    // Linux: Enable family process tree exclusion
    #[cfg(target_os = "linux")]
    let family_pids = get_self_family_pids(&system);

    let mut pids = Vec::new();
    let current_pid = std::process::id();
    let current_exe = get_current_exe_path();
    let except_id = target_ide
        .and_then(|target| target.strip_prefix("instance:"))
        .unwrap_or("default");
    let (protected_pids, protected_markers) =
        crate::modules::instance::other_instance_protection(except_id);

    // Load both manual paths from config
    let config = crate::modules::config::load_app_config().ok();
    let manual_path = config
        .as_ref()
        .and_then(|c| c.antigravity_executable.as_ref())
        .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok());
    let ide_manual_path = config
        .as_ref()
        .and_then(|c| c.antigravity_ide_executable.as_ref())
        .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok());

    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();

        // Exclude own PID
        if pid_u32 == current_pid {
            continue;
        }

        // Exclude own executable path (hardened against broad name matching)
        if let (Some(ref my_path), Some(p_exe)) = (&current_exe, process.exe()) {
            if let Ok(p_path) = p_exe.canonicalize() {
                if my_path == &p_path {
                    continue;
                }
            }
        }

        let name = process.name().to_string_lossy().to_lowercase();

        #[cfg(target_os = "linux")]
        {
            // 1. Exclude family processes (self, children, parents)
            if family_pids.contains(&pid_u32) {
                continue;
            }
            // 2. Extra protection: match "tools" likely manager if not a child
            if name.contains("tools") {
                continue;
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Other platforms: exclude only self
            if pid_u32 == current_pid {
                continue;
            }
        }

        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let exe_early = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();
        if crate::modules::instance::should_spare_pid(
            pid_u32,
            &args_str,
            &exe_early,
            &name,
            &protected_pids,
            &protected_markers,
        ) {
            continue;
        }

        if is_non_ide_binary(&name, &exe_early, &args_str) {
            continue;
        }

        let is_instance_sandbox = args_str.contains(".antigravity_tools")
            || args_str.contains("/instances/")
            || args_str.contains("--user-data-dir");

        // Target discrimination: Protect instances from default operations and default from instance operations
        if let Some(t) = target_ide {
            if let Some(inst_id) = t.strip_prefix("instance:") {
                if inst_id == "default" {
                    if is_instance_sandbox {
                        continue;
                    }
                } else {
                    let id_norm = inst_id.to_lowercase();
                    let needle = format!("/instances/{}/", id_norm);
                    let needle_alt = format!("/instances/{}", id_norm);
                    let matches_inst = args_str.contains(&needle) || args_str.contains(&needle_alt);
                    if !matches_inst || !args_str.contains("--user-data-dir") {
                        continue;
                    }
                }
            } else if t == "ide" {
                if is_instance_sandbox {
                    continue;
                }
            }
        } else {
            // Default target (target_ide == None): NEVER match isolated sandbox instances!
            if is_instance_sandbox {
                continue;
            }
        }

        // Recognition ref IDE manual path: If the process exactly matches the configured IDE path,
        // NEVER add it to kill list regardless of target_ide
        if let (Some(ref ide_m_path), Some(p_exe)) = (&ide_manual_path, process.exe()) {
            if let Ok(p_path) = p_exe.canonicalize() {
                #[cfg(target_os = "macos")]
                let matches = {
                    let m = ide_m_path.to_string_lossy();
                    let p = p_path.to_string_lossy();
                    matches!(m.find(".app").zip(p.find(".app")), Some((mi, pi)) if m[..mi + 4] == p[..pi + 4])
                };
                #[cfg(not(target_os = "macos"))]
                let matches = ide_m_path == &p_path;

                if matches && target_ide != Some("ide") {
                    // This is explicitly the IDE we must NOT kill when switching client
                    continue;
                }
                if matches && target_ide == Some("ide") {
                    // This is the IDE we WANT to close
                    pids.push(pid_u32);
                    continue;
                }
            }
        }

        // Recognition ref 3: Check manual config path match (client)
        if let (Some(ref m_path), Some(p_exe)) = (&manual_path, process.exe()) {
            if let Ok(p_path) = p_exe.canonicalize() {
                #[cfg(target_os = "macos")]
                let matches = {
                    let m_path_str = m_path.to_string_lossy();
                    let p_path_str = p_path.to_string_lossy();
                    matches!(m_path_str.find(".app").zip(p_path_str.find(".app")), Some((m_idx, p_idx)) if m_path_str[..m_idx + 4] == p_path_str[..p_idx + 4])
                };
                #[cfg(not(target_os = "macos"))]
                let matches = m_path == &p_path;

                if matches {
                    #[cfg(target_os = "macos")]
                    let is_main = {
                        let is_helper_by_args = args
                            .iter()
                            .any(|arg| arg.to_string_lossy().contains("--type="));
                        let is_helper_by_name = name.contains("helper")
                            || name.contains("plugin")
                            || name.contains("renderer")
                            || name.contains("gpu")
                            || name.contains("crashpad")
                            || name.contains("utility")
                            || name.contains("audio")
                            || name.contains("sandbox");
                        !is_helper_by_args && !is_helper_by_name
                    };
                    #[cfg(not(target_os = "macos"))]
                    let is_main = true;

                    if is_main {
                        if target_ide == Some("ide") {
                            // This is explicitly the client we must NOT kill when switching IDE
                            continue;
                        } else {
                            // This is the client we WANT to close
                            pids.push(pid_u32);
                            continue;
                        }
                    }
                }
            }
        }

        // 4. Strict mode: If the relevant manual path is configured, we strictly enforce it
        // and DO NOT fallback to fuzzy string matching.
        if manual_path.is_some() && target_ide != Some("ide") {
            continue;
        }
        if ide_manual_path.is_some() && target_ide == Some("ide") {
            continue;
        }

        // Get executable path
        let exe_path = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Common helper process exclusion logic
        let is_helper = is_helper_process(&name, &args_str, &exe_path);

        // Check if the process matches target_ide
        let exe_file = exe_file_name(&exe_path);
        let has_antigravity_name = name.contains("antigravity") || exe_file.contains("antigravity");

        let is_ide_match = if target_ide == Some("ide") {
            exe_path.contains("antigravity ide")
                || exe_path.contains("antigravity-ide")
                || name.contains("antigravity ide")
                || name.contains("antigravity-ide")
                || ide_exe_paths.contains(&exe_path)
        } else {
            if ide_exe_paths.contains(&exe_path) {
                false // Explicitly immune (it is an IDE)
            } else {
                has_antigravity_name
                    && !exe_path.contains("antigravity ide")
                    && !exe_path.contains("antigravity-ide")
                    && !name.contains("antigravity ide")
                    && !name.contains("antigravity-ide")
            }
        };

        if is_ide_match && !is_helper {
            pids.push(pid_u32);
        }
    }

    if !pids.is_empty() {
        crate::modules::logger::log_info(&format!(
            "Found {} Antigravity ({:?}) processes: {:?}",
            pids.len(),
            target_ide,
            pids
        ));
    }

    pids
}
