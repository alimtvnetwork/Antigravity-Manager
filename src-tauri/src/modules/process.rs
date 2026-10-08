use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Get normalized path of the current running executable
fn get_current_exe_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
}

/// Helper to extract executable paths of Antigravity IDE instances
/// Uses config path as primary, and falls back to cmd() arg scanning (works on macOS/Linux)
fn get_ide_exe_paths(system: &System) -> std::collections::HashSet<String> {
    let mut immune_exe_paths = std::collections::HashSet::new();

    // Primary: load from explicit config setting (most reliable on Windows)
    if let Ok(config) = crate::modules::config::load_app_config() {
        if let Some(ide_path) = config.antigravity_ide_executable {
            if let Ok(canonical) = std::path::PathBuf::from(&ide_path).canonicalize() {
                immune_exe_paths.insert(canonical.to_string_lossy().to_lowercase());
            } else {
                immune_exe_paths.insert(ide_path.to_lowercase());
            }
        }
    }

    // Fallback: scan process cmd() args (works on macOS/Linux, may be empty on Windows)
    for (_pid, process) in system.processes() {
        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase())
            .collect::<Vec<String>>()
            .join(" ");

        if args_str.contains("antigravity ide") || args_str.contains("antigravity-ide") {
            if let Some(exe_path) = process.exe().and_then(|p| p.to_str()) {
                immune_exe_paths.insert(exe_path.to_lowercase());
            }
        }

        // Structural signature of Antigravity IDE:
        // Executable parent folder contains resources/bin/language_server.exe (or language_server on Unix)
        // or resources/app.asar (Electron-based VS Code IDE distribution).
        if let Some(exe) = process.exe() {
            let path_str = exe.to_string_lossy().to_lowercase();
            if path_str.contains("antigravity") {
                if let Some(parent) = exe.parent() {
                    let has_ls = parent
                        .join("resources")
                        .join("bin")
                        .join("language_server.exe")
                        .exists()
                        || parent
                            .join("resources")
                            .join("bin")
                            .join("language_server")
                            .exists();
                    let has_app_asar = parent.join("resources").join("app.asar").exists();
                    if has_ls || has_app_asar {
                        immune_exe_paths.insert(path_str);
                    }
                }
            }
        }
    }
    immune_exe_paths
}

/// Check if a process with the given name is running (case-insensitive, strips .exe on Windows).
pub fn is_process_running_by_name(target_name: &str) -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let target_lower = target_name.to_lowercase();
    for (_pid, process) in system.processes() {
        let mut name = process.name().to_string_lossy().to_lowercase();
        if name.ends_with(".exe") {
            name.truncate(name.len() - 4);
        }
        if name == target_lower {
            return true;
        }
    }
    false
}

pub(crate) const NON_IDE_BINARIES: &[&str] = &[
    "esbuild",
    "esbuild.exe",
    "cargo",
    "cargo.exe",
    "rustc",
    "rustc.exe",
    "rust-analyzer",
    "rust-analyzer.exe",
    "node",
    "node.exe",
    "npm",
    "npm.cmd",
    "npm.exe",
    "pnpm",
    "pnpm.cmd",
    "pnpm.exe",
    "yarn",
    "yarn.cmd",
    "yarn.exe",
    "bun",
    "bun.exe",
    "git",
    "git.exe",
    "python",
    "python.exe",
    "python3",
    "python3.exe",
    "powershell",
    "powershell.exe",
    "pwsh",
    "pwsh.exe",
    "cmd",
    "cmd.exe",
    "bash",
    "bash.exe",
    "sh",
    "sh.exe",
    "conhost",
    "conhost.exe",
    "tar",
    "tar.exe",
    "curl",
    "curl.exe",
    "aria2c",
    "aria2c.exe",
    "agm",
    "agm.exe",
];

pub(crate) fn is_non_ide_binary(name: &str, exe_path: &str, args_str: &str) -> bool {
    let name_lower = name.to_lowercase();
    let exe_lower = exe_path.to_lowercase().replace('\\', "/");
    let args_lower = args_str.to_lowercase().replace('\\', "/");

    // Check exact binary name or binary with extension
    if NON_IDE_BINARIES
        .iter()
        .any(|&b| name_lower == b || name_lower == format!("{}.exe", b))
    {
        return true;
    }

    // Check if executable path points into build or package manager directories
    if exe_lower.contains("/node_modules/")
        || exe_lower.contains("/target/debug/")
        || exe_lower.contains("/target/release/")
        || exe_lower.contains("/.cargo/")
        || exe_lower.contains("/.rustup/")
    {
        return true;
    }

    // Check command line arguments for package manager execution
    if args_lower.contains("node_modules")
        && (name_lower.contains("node") || name_lower.contains("esbuild"))
    {
        return true;
    }

    false
}

pub(crate) fn exe_file_name(exe_path: &str) -> &str {
    let normalized = exe_path.trim_end_matches(['/', '\\']);
    normalized
        .rsplit_once(['/', '\\'])
        .map(|(_, file)| file)
        .unwrap_or(normalized)
}

/// Helper process discriminator to filter out sub-processes, audio/gpu/renderers, crashpads, and language servers
pub(crate) fn is_helper_process(name: &str, args_str: &str, exe_path: &str) -> bool {
    if is_non_ide_binary(name, exe_path, args_str) {
        return true;
    }

    let name_lower = name.to_lowercase();
    let args_lower = args_str.to_lowercase();
    let exe_lower = exe_path.to_lowercase();

    args_lower.contains("--type=")
        || args_lower.contains("node-ipc")
        || args_lower.contains("nodeipc")
        || args_lower.contains("max-old-space-size")
        || args_lower.contains("node_modules")
        || args_lower.contains("--standalone")
        || args_lower.contains("--subclient_type")
        || args_lower.contains("--override_ide_name")
        || args_lower.contains("embedded-browser-webview")
        || args_lower.contains("webview")
        || name_lower.contains("webview")
        || exe_lower.contains("webview")
        || name_lower.starts_with("agm")
        || exe_lower.contains("agm-alim")
        || exe_lower.ends_with("agm.exe")
        || name_lower.contains("helper")
        || name_lower.contains("plugin")
        || name_lower.contains("renderer")
        || name_lower.contains("gpu")
        || name_lower.contains("crashpad")
        || name_lower.contains("utility")
        || name_lower.contains("audio")
        || name_lower.contains("sandbox")
        || name_lower.contains("language_server")
        || args_lower.contains("language_server")
        || exe_lower.contains("crashpad")
        || exe_lower.contains("helper")
        || exe_lower.contains("language_server")
}

/// Sanitize restart arguments to prevent internal engine/language_server arguments
/// (such as --standalone or --override_ide_name) from leaking into IDE relaunch commands.
pub(crate) fn sanitize_restart_args(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|arg| {
            let lower = arg.trim().to_lowercase();
            !lower.is_empty()
                && !lower.starts_with("--standalone")
                && !lower.starts_with("--override_ide_name")
                && !lower.starts_with("--subclient_type")
                && !lower.contains("language_server")
        })
        .cloned()
        .collect()
}

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
fn get_self_family_pids(system: &sysinfo::System) -> std::collections::HashSet<u32> {
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
            let _ = std::fs::remove_file(&lockfile);
            crate::modules::logger::log_info(&format!(
                "[Lockfile] Cleaned stale Electron lockfile: {:?}",
                lockfile
            ));
        }

        let code_lock = dir.join("code.lock");
        if code_lock.exists() {
            let _ = std::fs::remove_file(&code_lock);
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
                    let _ = std::fs::remove_file(entry.path());
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
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &pid_u32.to_string()])
                    .creation_flags(0x08000000)
                    .output();
            }

            #[cfg(not(target_os = "windows"))]
            {
                let _ = Command::new("kill")
                    .args(["-9", &pid_u32.to_string()])
                    .output();
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
fn force_kill_pid(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(0x08000000)
            .output();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
}

/// Collect language_server subprocess PIDs for the target IDE.
fn language_server_subprocess_pids(system: &System, target_ide: Option<&str>) -> Vec<u32> {
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
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
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
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
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
                let _ = Command::new("kill")
                    .args(["-15", &pid.to_string()])
                    .output();
            } else {
                crate::modules::logger::log_warn(
                    "No main process identified, sending SIGTERM to all associated processes",
                );
                for pid in &pids {
                    let _ = Command::new("kill")
                        .args(["-15", &pid.to_string()])
                        .output();
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
                let _ = Command::new("kill")
                    .args(["-15", &pid.to_string()])
                    .output();
            } else {
                crate::modules::logger::log_warn(
                    "No clear Linux main process identified, sending SIGTERM to all associated processes",
                );
                for pid in &pids {
                    let _ = Command::new("kill")
                        .args(["-15", &pid.to_string()])
                        .output();
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
                        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
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
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();

            #[cfg(not(target_os = "windows"))]
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
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
    let _ = close_antigravity(5, target_ide);

    // 2. Extra safety sweep: purge any remaining target PIDs on Windows
    #[cfg(target_os = "windows")]
    {
        let remaining_pids = get_antigravity_pids(target_ide);
        for pid in remaining_pids {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
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

fn get_process_info(target_ide: Option<&str>) -> (Option<std::path::PathBuf>, Option<Vec<String>>) {
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

/// Get Antigravity executable path (cross-platform)
///
/// Search strategy (highest to lowest priority):
/// 1. Get path from running process (most reliable, supports any location)
/// Comprehensive detection engine that returns either the found PathBuf or a detailed AppError with audit trail and stack trace
pub fn detect_antigravity_with_diagnostics(
    target_ide: Option<&str>,
) -> Result<std::path::PathBuf, crate::error::AppError> {
    let mut audit_log = Vec::new();

    // Strategy 1: Check running processes (supports any custom location)
    if let Some(path) = get_path_from_running_process(target_ide) {
        crate::modules::logger::log_info(&format!(
            "[IDE Discovery] Located via running process: {:?}",
            path
        ));
        return Ok(path);
    }
    audit_log.push("No active Antigravity process found in process table".to_string());

    // Strategy 2: Check config paths (supports user-configured locations)
    if let Ok(config) = crate::modules::config::load_app_config() {
        let manual = if target_ide == Some("ide") {
            config.antigravity_ide_executable.as_ref()
        } else {
            config.antigravity_executable.as_ref()
        };

        if let Some(p) = manual {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                crate::modules::logger::log_info(&format!(
                    "[IDE Discovery] Located via user configuration: {:?}",
                    path
                ));
                return Ok(path);
            }
            audit_log.push(format!(
                "Configured path '{:?}' does not exist on disk",
                path
            ));
        } else {
            audit_log.push("No custom executable path configured in Settings".to_string());
        }
    }

    // Strategy 3: Standard installation locations, PATH, and Desktop entries
    let (found, checked) = audit_standard_locations(target_ide);
    if let Some(path) = found {
        crate::modules::logger::log_info(&format!(
            "[IDE Discovery] Located via filesystem search: {:?}",
            path
        ));
        return Ok(path);
    }

    let stack = std::backtrace::Backtrace::capture().to_string();
    let diag_summary = format!(
        "Audit Log:\n{}\n\nTested {} candidate locations across PATH, standard directories, Snap, Flatpak, and .desktop entries.\nNone matched an existing executable binary.",
        audit_log.join("\n"),
        checked.len()
    );

    Err(crate::error::AppError::IdeNotFound {
        message: format!(
            "Could not locate Antigravity executable on this system (tested {} locations).",
            checked.len()
        ),
        target_ide: target_ide.map(|s| s.to_string()),
        searched_locations: checked,
        diagnostics: diag_summary,
        stack_trace: stack,
    })
}

/// Fallback wrapper returning Option for backward compatibility
pub fn get_antigravity_executable_path(target_ide: Option<&str>) -> Option<std::path::PathBuf> {
    detect_antigravity_with_diagnostics(target_ide).ok()
}

/// Resolve the platform-specific path to the IDE discovery log:
/// `{data_local_dir}/antigravity/ide-discovery.log`
pub fn get_ide_discovery_log_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|d| d.join("antigravity").join("ide-discovery.log"))
}

/// Append a structured JSON diagnostic entry to `{data_local_dir}/antigravity/ide-discovery.log`.
/// Never propagates I/O errors, only logging failures via `tracing::warn!`.
pub fn append_ide_discovery_log(
    found: bool,
    status: Option<&str>,
    path: &str,
    method: &str,
    checked_paths: Option<&[String]>,
    backtrace: &str,
) {
    let log_path = match get_ide_discovery_log_path() {
        Some(p) => p,
        None => return,
    };

    if let Some(parent) = log_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "[IDE Discovery] Failed to create log directory {:?}: {}",
                parent,
                e
            );
            return;
        }
    }

    let mut entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "found": found,
        "status": status.unwrap_or(if found { "SUCCESS" } else { "FAILED" }),
        "path": path,
        "method": method,
        "backtrace": backtrace,
    });
    if let Some(paths) = checked_paths {
        entry["checked_paths"] = serde_json::json!(paths);
    }

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(mut file) => {
            use std::io::Write;
            let mut line = match serde_json::to_string(&entry) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("[IDE Discovery] Failed to serialize discovery log: {}", e);
                    return;
                }
            };
            line.push('\n');
            if let Err(e) = file.write_all(line.as_bytes()) {
                tracing::warn!(
                    "[IDE Discovery] Failed to write to log file {:?}: {}",
                    log_path,
                    e
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                "[IDE Discovery] Failed to open log file {:?}: {}",
                log_path,
                e
            );
        }
    }
}

/// Discover Antigravity IDE on first-time startup or when unconfigured,
/// persisting the located executable into gui_config.json.
pub fn discover_and_persist_initial_ide_info() -> Option<std::path::PathBuf> {
    let mut config = crate::modules::config::load_app_config().unwrap_or_default();

    if let Some(ref exe_str) = config.antigravity_executable {
        if !exe_str.trim().is_empty() {
            let path = std::path::PathBuf::from(exe_str);
            if path.exists() {
                crate::modules::logger::log_info(&format!(
                    "[IDE Discovery] Configured Antigravity IDE executable already exists: {:?}",
                    path
                ));
                append_ide_discovery_log(
                    true,
                    Some("SUCCESS"),
                    exe_str,
                    "existing_config",
                    None,
                    "",
                );
                return Some(path);
            }
        }
    }

    match detect_antigravity_with_diagnostics(None) {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            config.antigravity_executable = Some(path_str.clone());
            if let Err(e) = crate::modules::config::save_app_config(&config) {
                crate::modules::logger::log_warn(&format!(
                    "[IDE Discovery] Failed to persist discovered IDE to config: {}",
                    e
                ));
            }
            crate::modules::logger::log_info(&format!(
                "[IDE Discovery] First-time startup located Antigravity IDE: {:?}",
                path
            ));
            append_ide_discovery_log(
                true,
                Some("SUCCESS"),
                &path_str,
                "initial_discovery",
                None,
                "",
            );
            Some(path)
        }
        Err(err) => {
            let trace = std::backtrace::Backtrace::capture();
            let (diagnostics, original_stack, searched) = match err {
                crate::error::AppError::IdeNotFound {
                    ref diagnostics,
                    ref stack_trace,
                    ref searched_locations,
                    ..
                } => (
                    diagnostics.as_str(),
                    stack_trace.as_str(),
                    Some(searched_locations.as_slice()),
                ),
                _ => ("No detailed diagnostics available", "", None),
            };
            crate::modules::logger::log_warn(&format!(
                "[IDE Discovery] No Antigravity IDE detected during initial discovery.\nDiagnostics:\n{}\nStack trace:\n{:?}\nOriginal Stack:\n{}",
                diagnostics, trace, original_stack
            ));
            let trace_str = format!("{:?}\nOriginal Stack:\n{}", trace, original_stack);
            append_ide_discovery_log(
                false,
                Some("FAILED"),
                "",
                "initial_discovery",
                searched,
                &trace_str,
            );
            None
        }
    }
}

/// Helper to construct candidate search paths for macOS IDE discovery.
pub(crate) fn get_macos_candidate_paths(
    folder_name: &str,
    home_dir: Option<&std::path::Path>,
) -> Vec<String> {
    let mut candidates = vec![
        format!("/Applications/{}.app", folder_name),
        format!(
            "/Applications/{}.app/Contents/MacOS/{}",
            folder_name, folder_name
        ),
        format!(
            "/Applications/{}.app/Contents/MacOS/Antigravity",
            folder_name
        ),
        format!("/Applications/{}.app/Contents/MacOS/Electron", folder_name),
    ];

    let resolved_home: Option<std::path::PathBuf> = match home_dir {
        Some(h) => Some(h.to_path_buf()),
        None => dirs::home_dir(),
    };

    if let Some(home) = resolved_home {
        let home_str = home.to_string_lossy().replace('\\', "/");
        let home_clean = home_str.trim_end_matches('/');
        let user_app = format!("{}/Applications/{}.app", home_clean, folder_name);
        candidates.push(user_app.clone());
        candidates.push(format!("{}/Contents/MacOS/{}", user_app, folder_name));
        candidates.push(format!("{}/Contents/MacOS/Antigravity", user_app));
        candidates.push(format!("{}/Contents/MacOS/Electron", user_app));
    }

    candidates
}

/// Audit standard installation locations and collect diagnostics
fn audit_standard_locations(target_ide: Option<&str>) -> (Option<std::path::PathBuf>, Vec<String>) {
    let mut checked = Vec::new();

    let folder_names: &[&str] = if target_ide == Some("ide") {
        &["Antigravity IDE", "Antigravity", "antigravity"]
    } else if target_ide == Some("code") || target_ide == Some("cursor") {
        &["Antigravity", "antigravity"]
    } else if target_ide == Some("classic") {
        &["Antigravity", "antigravity"]
    } else {
        &["Antigravity", "antigravity", "Antigravity IDE"]
    };

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir();
        for folder_name in folder_names {
            let candidates = get_macos_candidate_paths(folder_name, home.as_deref());
            for c in candidates {
                checked.push(c.clone());
                if !c.to_lowercase().contains(".trash") {
                    let p = std::path::PathBuf::from(c);
                    if p.exists() {
                        return (Some(p), checked);
                    }
                }
            }
        }

        // Spotlight mdfind fallback for non-standard directory installations
        for folder_name in folder_names {
            let query = format!("kMDItemFSName == '{}.app'", folder_name);
            checked.push(format!("mdfind: {}", query));
            if let Ok(output) = std::process::Command::new("mdfind")
                .env("RUST_BACKTRACE", "1")
                .arg(&query)
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.to_lowercase().contains(".trash") {
                            let p = std::path::PathBuf::from(trimmed);
                            if p.exists() {
                                return (Some(p), checked);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let local_appdata = env::var("LOCALAPPDATA").ok();
        let program_files =
            env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 =
            env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

        for folder_name in folder_names {
            let mut candidates = Vec::new();

            if let Some(ref local) = local_appdata {
                candidates.push(
                    std::path::PathBuf::from(local)
                        .join("Programs")
                        .join(folder_name)
                        .join(format!("{}.exe", folder_name)),
                );
            }

            candidates.push(
                std::path::PathBuf::from(&program_files)
                    .join(folder_name)
                    .join(format!("{}.exe", folder_name)),
            );

            candidates.push(
                std::path::PathBuf::from(&program_files_x86)
                    .join(folder_name)
                    .join(format!("{}.exe", folder_name)),
            );

            for path in candidates {
                let path_str = path.to_string_lossy().to_string();
                checked.push(path_str);
                if path.exists() {
                    return (Some(path), checked);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Direct APPIMAGE environment check
        if let Ok(appimage) = std::env::var("APPIMAGE") {
            let path = std::path::PathBuf::from(&appimage);
            checked.push(format!("$APPIMAGE: {}", appimage));
            if path.is_file() {
                return (Some(path), checked);
            }
        }

        let exe_names: &[&str] = if target_ide == Some("ide") {
            &[
                "antigravity-ide",
                "Antigravity-IDE",
                "antigravity",
                "Antigravity",
            ]
        } else if target_ide == Some("cursor") {
            &["cursor", "antigravity", "Antigravity"]
        } else if target_ide == Some("code") {
            &["code", "antigravity", "Antigravity"]
        } else {
            &[
                "antigravity",
                "Antigravity",
                "antigravity-ide",
                "Antigravity-IDE",
                "google-antigravity",
                "Google-Antigravity",
                "agy",
            ]
        };

        // 2. PATH resolution
        if let Some(path) = resolve_linux_path_env(exe_names, &mut checked) {
            return (Some(path), checked);
        }

        // 3. Standard filesystem locations & AppImages
        for folder_name in folder_names {
            if let Some(path) = resolve_linux_standard_paths(folder_name, exe_names, &mut checked) {
                return (Some(path), checked);
            }
        }

        // 4. Desktop entry parsing (.desktop)
        if let Some(path) = resolve_linux_desktop_entry(exe_names, &mut checked) {
            return (Some(path), checked);
        }
    }

    (None, checked)
}

#[cfg(target_os = "linux")]
fn clean_desktop_exec_command(exec_cmd: &str) -> Option<String> {
    let trimmed = exec_cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Handle quotes: e.g. "/opt/Antigravity/antigravity" %U or '...'
    let unquoted = if (trimmed.starts_with('"') && trimmed[1..].contains('"'))
        || (trimmed.starts_with('\'') && trimmed[1..].contains('\''))
    {
        let quote_char = trimmed.chars().next().unwrap();
        let end_idx = trimmed[1..]
            .find(quote_char)
            .map(|i| i + 1)
            .unwrap_or(trimmed.len());
        &trimmed[1..end_idx]
    } else {
        trimmed.split_whitespace().next().unwrap_or("")
    };

    // If starts with env or /usr/bin/env, skip to next token
    let binary = if unquoted.ends_with("/env") || unquoted == "env" {
        let after_env = trimmed.trim_start_matches(unquoted).trim();
        after_env.split_whitespace().next().unwrap_or("")
    } else {
        unquoted
    };

    if binary.is_empty() {
        None
    } else {
        Some(binary.to_string())
    }
}

#[cfg(target_os = "linux")]
fn resolve_linux_path_env(
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for p in std::env::split_paths(&path_var) {
        for exe in exe_names {
            let candidate = p.join(exe);
            checked.push(candidate.to_string_lossy().to_string());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn resolve_linux_desktop_entry(
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let mut dirs_to_check = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        std::path::PathBuf::from("/usr/local/share/applications"),
        std::path::PathBuf::from("/var/lib/snapd/desktop/applications"),
        std::path::PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        dirs_to_check.push(home.join(".local/share/applications"));
        dirs_to_check.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    for dir in dirs_to_check {
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(f) => f.to_string_lossy().to_lowercase(),
                None => continue,
            };
            if !file_name.ends_with(".desktop") {
                continue;
            }

            let matches_target = exe_names
                .iter()
                .any(|name| file_name.contains(&name.to_lowercase()));
            if matches_target {
                checked.push(format!(".desktop: {}", path.to_string_lossy()));
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(exec_cmd) = trimmed.strip_prefix("Exec=") {
                            if let Some(clean_cmd) = clean_desktop_exec_command(exec_cmd) {
                                let binary_path = std::path::PathBuf::from(&clean_cmd);
                                checked.push(format!("  -> Exec: {}", clean_cmd));
                                if binary_path.is_file() {
                                    return Some(binary_path);
                                }
                                // If relative or bare command, check PATH
                                if let Some(found) = resolve_linux_path_env(&[&clean_cmd], checked)
                                {
                                    return Some(found);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn resolve_linux_standard_paths(
    folder_name: &str,
    exe_names: &[&str],
    checked: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let folder_lower = folder_name.to_lowercase().replace(' ', "-");
    let folder_lower_simple = folder_name.to_lowercase().replace(' ', "");

    for exe in exe_names {
        let mut candidates = vec![
            std::path::PathBuf::from(format!("/usr/bin/{}", exe)),
            std::path::PathBuf::from(format!("/usr/local/bin/{}", exe)),
            std::path::PathBuf::from(format!("/snap/bin/{}", exe)),
            std::path::PathBuf::from(format!("/var/lib/snapd/snap/bin/{}", exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_lower, exe)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_lower_simple, exe)),
            std::path::PathBuf::from(format!("/opt/Google/Antigravity/{}", exe)),
            std::path::PathBuf::from(format!("/opt/google-antigravity/{}", exe)),
            std::path::PathBuf::from(format!("/usr/share/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/usr/share/{}/{}", folder_lower, exe)),
            std::path::PathBuf::from(format!("/usr/lib/{}/{}", folder_name, exe)),
            std::path::PathBuf::from(format!("/usr/lib/{}/{}", folder_lower, exe)),
        ];

        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join(format!(".local/bin/{}", exe)));
            candidates.push(home.join(format!("bin/{}", exe)));
            candidates.push(home.join(format!(".local/share/{}/{}", folder_name, exe)));
            candidates.push(home.join(format!(".local/share/{}/{}", folder_lower, exe)));
            candidates.push(home.join(format!("Applications/{}.AppImage", folder_name)));
            candidates.push(home.join(format!("Applications/{}.AppImage", folder_lower)));
            candidates.push(home.join("Applications/Antigravity.AppImage"));
            candidates.push(home.join("Applications/antigravity.AppImage"));
            candidates.push(home.join("Downloads/Antigravity.AppImage"));
            candidates.push(home.join("Downloads/antigravity.AppImage"));
            candidates.push(home.join("Desktop/Antigravity.AppImage"));
            candidates.push(home.join("Desktop/antigravity.AppImage"));
        }

        for path in candidates {
            checked.push(path.to_string_lossy().to_string());
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// 获取 Antigravity CLI (agy) 的安装/可执行文件路径
pub fn get_antigravity_cli_executable_path() -> Option<std::path::PathBuf> {
    // 1. 优先从配置查询
    if let Ok(config) = crate::modules::config::load_app_config() {
        if let Some(ref p) = config.antigravity_cli_executable {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 2. 检查标准用户本地目录 ~/.local/bin/agy 或 ~/.local/bin/agy.exe
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local").join("bin");
        let path = if cfg!(target_os = "windows") {
            local_bin.join("agy.exe")
        } else {
            local_bin.join("agy")
        };
        if path.exists() {
            return Some(path);
        }

        // Check Windows standard %LOCALAPPDATA%\agy\bin\agy.exe
        if let Some(local_app_data) = dirs::data_local_dir() {
            let win_path = local_app_data.join("agy").join("bin").join("agy.exe");
            if win_path.exists() {
                return Some(win_path);
            }
        }

        // Check ~/.gemini/antigravity/bin/agy.exe
        let gemini_bin = home.join(".gemini").join("antigravity").join("bin");
        let gemini_path = if cfg!(target_os = "windows") {
            gemini_bin.join("agy.exe")
        } else {
            gemini_bin.join("agy")
        };
        if gemini_path.exists() {
            return Some(gemini_path);
        }
    }

    // 3. 在系统环境变量 PATH 中查找
    let cmd = if cfg!(target_os = "windows") {
        "agy.exe"
    } else {
        "agy"
    };
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            let p_cmd = p.join(cmd);
            if p_cmd.exists() {
                return Some(p_cmd);
            }
        }
    }

    None
}

/// Bring any instance window in the process list to the foreground
#[cfg(target_os = "windows")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if pids.is_empty() {
        return false;
    }
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let pid_list = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let ps_cmd = format!(
        r#"$pids = @({});
$ws = New-Object -ComObject WScript.Shell;
$act = $false;
foreach ($p in $pids) {{
    if ($ws.AppActivate($p)) {{ $act = $true; break }}
}}
if (-not $act) {{
    foreach ($p in $pids) {{
        $proc = Get-Process -Id $p -ErrorAction SilentlyContinue;
        if ($proc -and $proc.MainWindowHandle -ne 0) {{
            Add-Type '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);' -Name 'Win32Focus' -Namespace 'Antigravity';
            [Antigravity.Win32Focus]::ShowWindow($proc.MainWindowHandle, 9);
            [Antigravity.Win32Focus]::SetForegroundWindow($proc.MainWindowHandle);
            $act = $true;
            break;
        }}
    }}
}}
exit $(if ($act) {{ 0 }} else {{ 1 }})"#,
        pid_list
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// Bring an instance window to the foreground by PID
#[cfg(target_os = "windows")]
pub fn focus_instance_process(pid: u32) -> bool {
    focus_instance_pids(&[pid])
}

#[cfg(target_os = "macos")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if let Some(&pid) = pids.first() {
        focus_instance_process(pid)
    } else {
        false
    }
}

#[cfg(target_os = "macos")]
pub fn focus_instance_process(pid: u32) -> bool {
    let script = format!(
        "tell application \"System Events\" to set frontmost of (first process whose unix id is {}) to true",
        pid
    );
    let output = Command::new("osascript").args(["-e", &script]).output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if let Some(&pid) = pids.first() {
        focus_instance_process(pid)
    } else {
        false
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_process(pid: u32) -> bool {
    let output = Command::new("xdotool")
        .args(["search", "--pid", &pid.to_string(), "windowactivate"])
        .output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_pids(_pids: &[u32]) -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_process(_pid: u32) -> bool {
    false
}

/// Bring Antigravity IDE/Classic window to the foreground and ensure it is visible/restored
pub fn focus_antigravity_window(target_ide: Option<&str>) -> bool {
    let pids = get_antigravity_pids(target_ide);
    focus_instance_pids(&pids)
}

/// Bring an instance window with matching workspace title to the foreground
#[cfg(target_os = "windows")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    if pids.is_empty() {
        return false;
    }
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let pid_list = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let safe_repo = repo_name.replace('\'', "''").replace('"', "");

    let ps_cmd = format!(
        r#"$pids = @({});
$repo = '{}';
$act = $false;

if (-not ([System.Management.Automation.PSTypeName]'Antigravity.Win32WorkspaceWindow').Type) {{
    Add-Type @'
    using System;
    using System.Runtime.InteropServices;
    using System.Text;

    namespace Antigravity {{
        public class Win32WorkspaceWindow {{
            [DllImport("user32.dll")]
            public static extern bool SetForegroundWindow(IntPtr hWnd);

            [DllImport("user32.dll")]
            public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

            [DllImport("user32.dll")]
            public static extern bool BringWindowToTop(IntPtr hWnd);

            [DllImport("user32.dll")]
            public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

            [DllImport("user32.dll")]
            public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

            [DllImport("user32.dll")]
            public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

            public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
        }}
    }}
'@;
}}

# 1. First pass: Search top-level windows belonging to target PIDs matching repo name
[Antigravity.Win32WorkspaceWindow]::EnumWindows({{
    param($hwnd, $lparam)
    $procId = 0;
    [Antigravity.Win32WorkspaceWindow]::GetWindowThreadProcessId($hwnd, [ref]$procId);
    if ($pids -contains $procId) {{
        $sb = New-Object System.Text.StringBuilder 512;
        [Antigravity.Win32WorkspaceWindow]::GetWindowText($hwnd, $sb, 512) | Out-Null;
        $title = $sb.ToString();
        if ($title -and ($title -like "*$repo*")) {{
            [Antigravity.Win32WorkspaceWindow]::ShowWindow($hwnd, 9);
            [Antigravity.Win32WorkspaceWindow]::BringWindowToTop($hwnd);
            [Antigravity.Win32WorkspaceWindow]::SetForegroundWindow($hwnd);
            $global:act = $true;
            $script:act = $true;
            return $false;
        }}
    }}
    return $true;
}}, [IntPtr]::Zero);

# 2. Second pass: Fallback to any MainWindowHandle if exact repo title not found
if (-not $global:act -and -not $script:act -and -not $act) {{
    foreach ($p in $pids) {{
        $proc = Get-Process -Id $p -ErrorAction SilentlyContinue;
        if ($proc -and $proc.MainWindowHandle -ne 0) {{
            [Antigravity.Win32WorkspaceWindow]::ShowWindow($proc.MainWindowHandle, 9);
            [Antigravity.Win32WorkspaceWindow]::BringWindowToTop($proc.MainWindowHandle);
            [Antigravity.Win32WorkspaceWindow]::SetForegroundWindow($proc.MainWindowHandle);
            $global:act = $true;
            $script:act = $true;
            $act = $true;
            break;
        }}
    }}
}}
exit $(if ($global:act -or $script:act -or $act) {{ 0 }} else {{ 1 }})"#,
        pid_list, safe_repo
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    let script = format!(
        r#"tell application "System Events"
            set matched to false
            repeat with p in {{{}}}
                set procList to (processes whose unix id is p)
                if (count of procList) > 0 then
                    set theProc to item 1 of procList
                    tell theProc
                        repeat with w in windows
                            if name of w contains "{}" then
                                set frontmost to true
                                perform action "AXRaise" of w
                                set matched to true
                                exit repeat
                            end if
                        end repeat
                    end tell
                    if matched then exit repeat
                end if
            end repeat
            if not matched and (count of pids) > 0 then
                set frontmost of (first process whose unix id is (item 1 of pids)) to true
            end if
        end tell"#,
        pids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(","),
        repo_name.replace('"', "\\\"")
    );
    let output = Command::new("osascript").args(["-e", &script]).output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    let wm_status = Command::new("wmctrl")
        .args(["-a", repo_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if wm_status {
        return true;
    }
    if let Some(&pid) = pids.first() {
        Command::new("xdotool")
            .args(["search", "--pid", &pid.to_string(), "windowactivate"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_workspace_window(_pids: &[u32], _repo_name: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_helper_process_detection() {
        // Normal main processes
        assert!(!is_helper_process(
            "Antigravity",
            "/Applications/Antigravity.app/Contents/MacOS/Antigravity",
            "/Applications/Antigravity.app/Contents/MacOS/Antigravity"
        ));
        assert!(!is_helper_process(
            "Antigravity.exe",
            "C:\\Program Files\\Antigravity\\Antigravity.exe",
            "C:\\Program Files\\Antigravity\\Antigravity.exe"
        ));

        // Language server / engine processes (must be detected as helper)
        assert!(is_helper_process(
            "language_server",
            "--standalone --override_ide_name antigravity --subclient_type hub",
            "/Applications/Antigravity.app/Contents/Resources/bin/language_server"
        ));
        assert!(is_helper_process(
            "language_server.exe",
            "--standalone",
            "C:\\Antigravity\\resources\\bin\\language_server.exe"
        ));
        assert!(is_helper_process(
            "Antigravity",
            "--type=utility --utility-sub-type=audio.mojom.AudioService",
            "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper.app/Contents/MacOS/Antigravity Helper"
        ));
        assert!(is_helper_process(
            "Antigravity",
            "--type=renderer",
            "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper (Renderer).app"
        ));
        assert!(is_helper_process(
            "crashpad_handler",
            "",
            "/Applications/Antigravity.app/Contents/Frameworks/Electron Framework.framework/Helpers/chrome_crashpad_handler"
        ));
    }

    #[test]
    fn test_is_language_server_process_narrow_detection() {
        // Hot-switch locator must be narrow: only match language server, never renderer / gpu / crashpad
        assert!(is_language_server_process(
            "language_server.exe",
            "C:\\Users\\me\\AppData\\Local\\Programs\\Antigravity IDE\\resources\\bin\\language_server.exe"
        ));
        assert!(is_language_server_process(
            "language_server",
            "/Applications/Antigravity.app/Contents/Resources/bin/language_server"
        ));
        // macOS process name might be truncated -> matching path is sufficient
        assert!(is_language_server_process(
            "language_server",
            "/Applications/Antigravity IDE.app/Contents/Resources/bin/language_server_macos_arm"
        ));
        // Case-insensitive
        assert!(is_language_server_process(
            "LANGUAGE_SERVER.EXE",
            "C:\\Antigravity\\bin\\Language_Server.exe"
        ));

        // Negative examples: other Antigravity processes must never match
        for (name, exe) in [
            ("Antigravity.exe", "C:\\Antigravity\\Antigravity.exe"),
            (
                "Antigravity",
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity",
            ),
            (
                "Antigravity Helper",
                "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper.app/Contents/MacOS/Antigravity Helper",
            ),
            (
                "Antigravity Helper (Renderer)",
                "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper (Renderer).app",
            ),
            (
                "crashpad_handler",
                "/Applications/Antigravity.app/Contents/Frameworks/Electron Framework.framework/Helpers/chrome_crashpad_handler",
            ),
            ("node", "/usr/local/bin/node"),
        ] {
            assert!(
                !is_language_server_process(name, exe),
                "{name} should not be identified as language_server"
            );
        }
    }

    #[test]
    fn test_sanitize_restart_args() {
        let dirty_args = vec![
            "--standalone".to_string(),
            "--override_ide_name".to_string(),
            "antigravity".to_string(),
            "--subclient_type".to_string(),
            "hub".to_string(),
            "--user-data-dir=/tmp/test".to_string(),
            "/path/to/project".to_string(),
        ];

        let cleaned = sanitize_restart_args(&dirty_args);
        assert!(!cleaned.contains(&"--standalone".to_string()));
        assert!(!cleaned.contains(&"--override_ide_name".to_string()));
        assert!(!cleaned.contains(&"--subclient_type".to_string()));
        assert!(cleaned.contains(&"--user-data-dir=/tmp/test".to_string()));
        assert!(cleaned.contains(&"/path/to/project".to_string()));
    }

    #[test]
    fn test_is_non_ide_binary_filters_build_tools() {
        assert!(is_non_ide_binary(
            "esbuild.exe",
            "D:/work/Antigravity-Manager/node_modules/esbuild/esbuild.exe",
            ""
        ));
        assert!(is_non_ide_binary(
            "cargo.exe",
            "C:/Users/User/.cargo/bin/cargo.exe",
            "test"
        ));
        assert!(is_non_ide_binary(
            "rustc.exe",
            "C:/Users/User/.cargo/bin/rustc.exe",
            ""
        ));
        assert!(is_non_ide_binary(
            "node.exe",
            "C:/Program Files/nodejs/node.exe",
            "vite"
        ));
        assert!(!is_non_ide_binary(
            "antigravity.exe",
            "C:/Program Files/Antigravity/antigravity.exe",
            ""
        ));
        assert!(!is_non_ide_binary(
            "antigravity",
            "/usr/bin/antigravity",
            ""
        ));
    }

    #[test]
    fn test_format_macos_open_args_with_no_args() {
        let args = format_macos_open_args("Antigravity", None, true);
        assert_eq!(
            args,
            vec!["-n", "-a", "Antigravity", "--args", "--new-window"]
        );
    }

    #[test]
    fn test_format_macos_open_args_with_custom_args() {
        let custom = vec![
            "--user-data-dir=/tmp/test".to_string(),
            "/path/to/workspace".to_string(),
        ];
        let args = format_macos_open_args("/Applications/Antigravity.app", Some(&custom), true);
        assert_eq!(
            args,
            vec![
                "-n",
                "-a",
                "/Applications/Antigravity.app",
                "--args",
                "--user-data-dir=/tmp/test",
                "/path/to/workspace",
                "--new-window"
            ]
        );
    }

    #[test]
    fn test_format_macos_open_args_snapshot_no_new_window() {
        let snapshot_args = vec!["--user-data-dir=/tmp/snapshot".to_string()];
        let args = format_macos_open_args(
            "/Applications/Antigravity IDE.app",
            Some(&snapshot_args),
            false,
        );
        assert_eq!(
            args,
            vec![
                "-n",
                "-a",
                "/Applications/Antigravity IDE.app",
                "--args",
                "--user-data-dir=/tmp/snapshot"
            ]
        );

        let no_args = format_macos_open_args("Antigravity", None, false);
        assert_eq!(no_args, vec!["-n", "-a", "Antigravity"]);
    }

    #[test]
    fn test_get_macos_candidate_paths() {
        let home = std::path::Path::new("/Users/developer");
        let paths = get_macos_candidate_paths("Antigravity", Some(home));
        assert_eq!(
            paths,
            vec![
                "/Applications/Antigravity.app".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity.app/Contents/MacOS/Electron".to_string(),
                "/Users/developer/Applications/Antigravity.app".to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity.app/Contents/MacOS/Electron".to_string(),
            ]
        );
    }

    #[test]
    fn test_get_macos_candidate_paths_ide() {
        let home = std::path::Path::new("/Users/developer");
        let paths = get_macos_candidate_paths("Antigravity IDE", Some(home));
        assert_eq!(
            paths,
            vec![
                "/Applications/Antigravity IDE.app".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity IDE".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity".to_string(),
                "/Applications/Antigravity IDE.app/Contents/MacOS/Electron".to_string(),
                "/Users/developer/Applications/Antigravity IDE.app".to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity IDE"
                    .to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Antigravity"
                    .to_string(),
                "/Users/developer/Applications/Antigravity IDE.app/Contents/MacOS/Electron"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn test_discover_and_persist_initial_ide_info_runs_without_panic() {
        let _ = discover_and_persist_initial_ide_info();
    }

    #[test]
    fn test_append_ide_discovery_log_runs_without_panic() {
        append_ide_discovery_log(
            false,
            Some("FAILED"),
            "/path/to/test",
            "test_method",
            Some(&["/checked/1".to_string(), "/checked/2".to_string()]),
            "test_trace",
        );
    }
}
