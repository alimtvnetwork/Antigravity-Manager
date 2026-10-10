use sysinfo::System;

use super::*;

pub(crate) fn get_current_exe_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
}

/// Helper to extract executable paths of Antigravity IDE instances
/// Uses config path as primary, and falls back to cmd() arg scanning (works on macOS/Linux)
pub(crate) fn get_ide_exe_paths(system: &System) -> std::collections::HashSet<String> {
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
