use sysinfo::System;

use super::*;

pub(crate) fn get_process_info(
    target_ide: Option<&str>,
) -> (Option<std::path::PathBuf>, Option<Vec<String>>) {
    let mut system = System::new_all();
    system.refresh_all();

    let current_exe = get_current_exe_path();
    let current_pid = std::process::id();

    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();
        if pid_u32 == current_pid {
            continue;
        }

        // Exclude manager process itself
        if let (Some(ref my_path), Some(p_exe)) = (&current_exe, process.exe()) {
            if let Ok(p_path) = p_exe.canonicalize() {
                if my_path == &p_path {
                    continue;
                }
            }
        }

        let name = process.name().to_string_lossy().to_lowercase();

        // Get executable path and command line arguments
        if let Some(exe) = process.exe() {
            let mut args = process.cmd().iter();
            let exe_path = args
                .next()
                .map_or(exe.to_string_lossy(), |arg| arg.to_string_lossy())
                .to_lowercase();

            // Extract actual arguments from command line (skipping exe path)
            let args = args
                .map(|arg| arg.to_string_lossy().to_lowercase())
                .collect::<Vec<String>>();

            let args_str = args.join(" ");

            // Common helper process exclusion logic (strictly excludes language_server and sub-processes)
            let is_helper = is_helper_process(&name, &args_str, &exe_path);
            if is_helper
                || is_non_ide_binary(&name, &exe_path, &args_str)
                || exe_path.contains(".trash")
            {
                continue;
            }

            // Sanitize snapshot arguments to prevent engine parameters like --standalone from leaking into relaunch
            let clean_args = sanitize_restart_args(&args);
            let path = Some(exe.to_path_buf());
            let args = Some(clean_args);

            // Target discrimination: Protect instances from default operations and default from instance operations
            if let Some(t) = target_ide {
                if let Some(inst_id) = t.strip_prefix("instance:") {
                    if inst_id == "default" {
                        let is_instance_sandbox = args_str.contains(".antigravity_tools")
                            || args_str.contains("/instances/")
                            || args_str.contains("\\instances\\");
                        if is_instance_sandbox {
                            continue;
                        }
                    } else {
                        let id_norm = inst_id.to_lowercase();
                        let needle = format!("/instances/{}/", id_norm);
                        let needle_alt = format!("/instances/{}", id_norm);
                        let matches_inst =
                            args_str.contains(&needle) || args_str.contains(&needle_alt);
                        if !matches_inst || !args_str.contains("--user-data-dir") {
                            continue;
                        }
                    }
                } else if t == "ide" {
                    let is_instance_sandbox = args_str.contains(".antigravity_tools")
                        || args_str.contains("/instances/")
                        || args_str.contains("\\instances\\");
                    if is_instance_sandbox {
                        continue;
                    }
                }
            } else {
                // Default target (target_ide == None): NEVER match isolated sandbox instances!
                let is_instance_sandbox = args_str.contains(".antigravity_tools")
                    || args_str.contains("/instances/")
                    || args_str.contains("\\instances\\");
                if is_instance_sandbox {
                    continue;
                }
            }

            let exe_file = exe_file_name(&exe_path);
            let has_antigravity_name =
                name.contains("antigravity") || exe_file.contains("antigravity");

            let is_ide_match = if target_ide == Some("ide") {
                exe_path.contains("antigravity ide")
                    || exe_path.contains("antigravity-ide")
                    || name.contains("antigravity ide")
                    || name.contains("antigravity-ide")
            } else {
                has_antigravity_name
                    && !exe_path.contains("antigravity ide")
                    && !exe_path.contains("antigravity-ide")
                    && !name.contains("antigravity ide")
                    && !name.contains("antigravity-ide")
            };

            if is_ide_match && !is_helper {
                #[cfg(target_os = "macos")]
                {
                    if !exe_path.contains("frameworks") {
                        if let Some(app_idx) = exe_path.find(".app") {
                            let app_path_str = &exe.to_string_lossy()[..app_idx + 4];
                            let path = Some(std::path::PathBuf::from(app_path_str));
                            return (path, args);
                        }
                    }
                    return (path, args);
                }

                #[cfg(target_os = "windows")]
                {
                    return (path, args);
                }

                #[cfg(target_os = "linux")]
                {
                    return (path, args);
                }
            }
        }
    }
    (None, None)
}

/// Get Antigravity executable path from running processes
///
/// Most reliable method to find installation anywhere
pub fn get_path_from_running_process(target_ide: Option<&str>) -> Option<std::path::PathBuf> {
    let (path, _) = get_process_info(target_ide);
    path
}

/// Get Antigravity startup arguments from running processes
pub fn get_args_from_running_process(target_ide: Option<&str>) -> Option<Vec<String>> {
    let (_, args) = get_process_info(target_ide);
    args
}

/// Get --user-data-dir argument value (if exists)
pub fn get_user_data_dir_from_process(target_ide: Option<&str>) -> Option<std::path::PathBuf> {
    let is_instance_target = target_ide
        .map(|t| t.starts_with("instance:"))
        .unwrap_or(false);

    // Prefer getting startup arguments from config
    if let Ok(config) = crate::modules::config::load_app_config() {
        if let Some(args) = config.antigravity_args {
            // Check arguments in config
            for i in 0..args.len() {
                if args[i] == "--user-data-dir" && i + 1 < args.len() {
                    // Next argument is the path
                    let path = std::path::PathBuf::from(&args[i + 1]);
                    if path.exists() {
                        let path_str = path.to_string_lossy().to_lowercase();
                        let is_instance_path = path_str.contains(".antigravity_tools")
                            || path_str.contains("/instances/")
                            || path_str.contains("\\instances\\");
                        if !is_instance_path || is_instance_target {
                            return Some(path);
                        }
                    }
                } else if args[i].starts_with("--user-data-dir=") {
                    // Argument and value in same string, e.g. --user-data-dir=/path/to/data
                    let parts: Vec<&str> = args[i].splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let path_str = parts[1];
                        let path = std::path::PathBuf::from(path_str);
                        if path.exists() {
                            let path_str = path.to_string_lossy().to_lowercase();
                            let is_instance_path = path_str.contains(".antigravity_tools")
                                || path_str.contains("/instances/")
                                || path_str.contains("\\instances\\");
                            if !is_instance_path || is_instance_target {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }

    // If not in config, get arguments from running process
    if let Some(args) = get_args_from_running_process(target_ide) {
        for i in 0..args.len() {
            if args[i] == "--user-data-dir" && i + 1 < args.len() {
                // Next argument is the path
                let path = std::path::PathBuf::from(&args[i + 1]);
                if path.exists() {
                    let path_str = path.to_string_lossy().to_lowercase();
                    let is_instance_path = path_str.contains(".antigravity_tools")
                        || path_str.contains("/instances/")
                        || path_str.contains("\\instances\\");
                    if !is_instance_path || is_instance_target {
                        return Some(path);
                    }
                }
            } else if args[i].starts_with("--user-data-dir=") {
                // Argument and value in same string, e.g. --user-data-dir=/path/to/data
                let parts: Vec<&str> = args[i].splitn(2, '=').collect();
                if parts.len() == 2 {
                    let path_str = parts[1];
                    let path = std::path::PathBuf::from(path_str);
                    if path.exists() {
                        let path_str = path.to_string_lossy().to_lowercase();
                        let is_instance_path = path_str.contains(".antigravity_tools")
                            || path_str.contains("/instances/")
                            || path_str.contains("\\instances\\");
                        if !is_instance_path || is_instance_target {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    None
}
