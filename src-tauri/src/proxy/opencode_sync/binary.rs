use super::config_paths::extract_version;
#[cfg(target_os = "windows")]
use super::lock::CREATE_NO_WINDOW;
use std::env;
use std::fs;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn resolve_opencode_path() -> Option<PathBuf> {
    // First, try to find in PATH
    if let Some(path) = find_in_path("opencode") {
        tracing::debug!("Found opencode in PATH: {:?}", path);
        return Some(path);
    }

    // Try fallback locations based on OS
    #[cfg(target_os = "windows")]
    {
        resolve_opencode_path_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        resolve_opencode_path_unix()
    }
}

#[cfg(target_os = "windows")]
fn resolve_opencode_path_windows() -> Option<PathBuf> {
    // Check npm global location
    if let Ok(app_data) = env::var("APPDATA") {
        let npm_opencode_cmd = PathBuf::from(&app_data).join("npm").join("opencode.cmd");
        if npm_opencode_cmd.exists() {
            tracing::debug!("Found opencode.cmd in APPDATA\\npm: {:?}", npm_opencode_cmd);
            return Some(npm_opencode_cmd);
        }
        let npm_opencode_exe = PathBuf::from(&app_data).join("npm").join("opencode.exe");
        if npm_opencode_exe.exists() {
            tracing::debug!("Found opencode.exe in APPDATA\\npm: {:?}", npm_opencode_exe);
            return Some(npm_opencode_exe);
        }
    }

    // Check pnpm location
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let pnpm_opencode_cmd = PathBuf::from(&local_app_data)
            .join("pnpm")
            .join("opencode.cmd");
        if pnpm_opencode_cmd.exists() {
            tracing::debug!(
                "Found opencode.cmd in LOCALAPPDATA\\pnpm: {:?}",
                pnpm_opencode_cmd
            );
            return Some(pnpm_opencode_cmd);
        }
        let pnpm_opencode_exe = PathBuf::from(&local_app_data)
            .join("pnpm")
            .join("opencode.exe");
        if pnpm_opencode_exe.exists() {
            tracing::debug!(
                "Found opencode.exe in LOCALAPPDATA\\pnpm: {:?}",
                pnpm_opencode_exe
            );
            return Some(pnpm_opencode_exe);
        }
    }

    // Check Yarn location
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let yarn_opencode = PathBuf::from(&local_app_data)
            .join("Yarn")
            .join("bin")
            .join("opencode.cmd");
        if yarn_opencode.exists() {
            tracing::debug!("Found opencode.cmd in Yarn bin: {:?}", yarn_opencode);
            return Some(yarn_opencode);
        }
    }

    // Scan NVM_HOME
    if let Ok(nvm_home) = env::var("NVM_HOME") {
        if let Some(path) = scan_nvm_directory(&nvm_home) {
            return Some(path);
        }
    }

    // Try common NVM locations
    if let Some(home) = dirs::home_dir() {
        let nvm_default = home.join(".nvm");
        if let Some(path) = scan_nvm_directory(&nvm_default) {
            return Some(path);
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn resolve_opencode_path_unix() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // Common user bin locations
    let user_bins = [
        home.join(".local").join("bin").join("opencode"),
        home.join(".npm-global").join("bin").join("opencode"),
        home.join(".volta").join("bin").join("opencode"),
        home.join("bin").join("opencode"),
    ];

    for path in &user_bins {
        if path.exists() {
            tracing::debug!("Found opencode in user bin: {:?}", path);
            return Some(path.clone());
        }
    }

    // System-wide locations
    let system_bins = [
        PathBuf::from("/opt/homebrew/bin/opencode"),
        PathBuf::from("/usr/local/bin/opencode"),
        PathBuf::from("/usr/bin/opencode"),
    ];

    for path in &system_bins {
        if path.exists() {
            tracing::debug!("Found opencode in system bin: {:?}", path);
            return Some(path.clone());
        }
    }

    // Scan nvm directories
    let nvm_dirs = [home.join(".nvm").join("versions").join("node")];

    for nvm_dir in &nvm_dirs {
        if let Some(path) = scan_node_versions(nvm_dir) {
            return Some(path);
        }
    }

    // Scan fnm directories
    let fnm_dirs = [
        home.join(".fnm").join("node-versions"),
        home.join("Library")
            .join("Application Support")
            .join("fnm")
            .join("node-versions"),
    ];

    for fnm_dir in &fnm_dirs {
        if let Some(path) = scan_fnm_versions(fnm_dir) {
            return Some(path);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn scan_nvm_directory(nvm_path: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let nvm_path = nvm_path.as_ref();
    if !nvm_path.exists() {
        return None;
    }

    let entries = fs::read_dir(nvm_path).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let opencode_cmd = path.join("opencode.cmd");
            if opencode_cmd.exists() {
                tracing::debug!("Found opencode.cmd in NVM: {:?}", opencode_cmd);
                return Some(opencode_cmd);
            }
            let opencode_exe = path.join("opencode.exe");
            if opencode_exe.exists() {
                tracing::debug!("Found opencode.exe in NVM: {:?}", opencode_exe);
                return Some(opencode_exe);
            }
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn scan_node_versions(versions_dir: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let versions_dir = versions_dir.as_ref();
    if !versions_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(versions_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let opencode = path.join("bin").join("opencode");
            if opencode.exists() {
                tracing::debug!("Found opencode in nvm: {:?}", opencode);
                return Some(opencode);
            }
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn scan_fnm_versions(versions_dir: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let versions_dir = versions_dir.as_ref();
    if !versions_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(versions_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let opencode = path.join("installation").join("bin").join("opencode");
            if opencode.exists() {
                tracing::debug!("Found opencode in fnm: {:?}", opencode);
                return Some(opencode);
            }
        }
    }

    None
}

fn find_in_path(executable: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let extensions = ["exe", "cmd", "bat"];
        if let Ok(path_var) = env::var("PATH") {
            for dir in path_var.split(';') {
                for ext in &extensions {
                    let full_path = PathBuf::from(dir).join(format!("{}.{}", executable, ext));
                    if full_path.exists() {
                        return Some(full_path);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(path_var) = env::var("PATH") {
            for dir in path_var.split(':') {
                let full_path = PathBuf::from(dir).join(executable);
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn run_opencode_version(opencode_path: &PathBuf) -> Option<String> {
    let path_str = opencode_path.to_string_lossy();

    // Check if it's a .cmd or .bat file that needs cmd.exe
    let is_cmd = path_str.ends_with(".cmd") || path_str.ends_with(".bat");

    let output = if is_cmd {
        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/C")
            .arg(opencode_path)
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW);
        cmd.output()
    } else {
        let mut cmd = Command::new(opencode_path);
        cmd.arg("--version").creation_flags(CREATE_NO_WINDOW);
        cmd.output()
    };

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Some tools output version to stderr
            let raw = if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };

            tracing::debug!("opencode --version output: {}", raw.trim());
            Some(extract_version(&raw))
        }
        Ok(output) => {
            tracing::debug!("opencode --version failed with status: {:?}", output.status);
            None
        }
        Err(e) => {
            tracing::debug!("Failed to run opencode --version: {}", e);
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_opencode_version(opencode_path: &PathBuf) -> Option<String> {
    let output = Command::new(opencode_path).arg("--version").output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Some tools output version to stderr
            let raw = if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };

            tracing::debug!("opencode --version output: {}", raw.trim());
            Some(extract_version(&raw))
        }
        Ok(output) => {
            tracing::debug!("opencode --version failed with status: {:?}", output.status);
            None
        }
        Err(e) => {
            tracing::debug!("Failed to run opencode --version: {}", e);
            None
        }
    }
}

pub fn check_opencode_installed() -> (bool, Option<String>) {
    tracing::debug!("Checking opencode installation...");

    let opencode_path = match resolve_opencode_path() {
        Some(path) => {
            tracing::debug!("Resolved opencode path: {:?}", path);
            path
        }
        None => {
            tracing::debug!("Could not resolve opencode path");
            return (false, None);
        }
    };

    match run_opencode_version(&opencode_path) {
        Some(version) => {
            tracing::debug!("opencode version detected: {}", version);
            (true, Some(version))
        }
        None => {
            tracing::debug!("Failed to get opencode version");
            (false, None)
        }
    }
}
