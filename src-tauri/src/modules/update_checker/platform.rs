use crate::modules::logger;
#[cfg(target_os = "windows")]
use crate::utils::command::CommandExtWrapper;

use super::*;

/// Detect if the app was installed via Homebrew Cask (macOS only)
pub fn is_homebrew_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        let caskroom_paths = [
            "/opt/homebrew/Caskroom/antigravity-tools",
            "/usr/local/Caskroom/antigravity-tools",
        ];

        for path in &caskroom_paths {
            if std::path::Path::new(path).exists() {
                logger::log_info(&format!("Detected Homebrew Cask installation at: {}", path));
                return true;
            }
        }
    }

    false
}

/// Detect if the app is currently running as an AppImage (Linux only).
///
/// The AppImage runtime always sets the `APPIMAGE` environment variable to the
/// absolute path of the source `.AppImage` file before mounting and executing the
/// bundled application. This is the canonical way to detect an AppImage execution
/// context without inspecting the filesystem.
///
/// This is used to gate Tauri's native auto-updater on Linux: Tauri's updater plugin
/// only supports AppImage bundles on Linux. Attempting to use it on RPM/DEB-installed
/// binaries results in an `ENOEXEC` error because the downloaded artifact is an
/// AppImage that cannot be executed without FUSE support (or proper permissions).
pub fn is_appimage_running() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("APPIMAGE").is_ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Execute `brew upgrade --cask antigravity-tools` with timeout (macOS only)
#[cfg(not(target_os = "macos"))]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    Err("brew_not_supported".to_string())
}

#[cfg(target_os = "macos")]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    logger::log_info("Starting Homebrew Cask upgrade for antigravity-tools...");

    // Find brew binary
    let brew_path = if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
        "/opt/homebrew/bin/brew"
    } else if std::path::Path::new("/usr/local/bin/brew").exists() {
        "/usr/local/bin/brew"
    } else {
        return Err("brew_not_found".to_string());
    };

    // 3 min timeout to prevent hanging
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        tokio::process::Command::new(brew_path)
            .args(["upgrade", "--cask", "antigravity-tools"])
            .output(),
    )
    .await;

    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            logger::log_error(&format!("Failed to execute brew upgrade: {}", e));
            return Err("brew_exec_failed".to_string());
        }
        Err(_) => {
            logger::log_error("Homebrew upgrade timed out after 3 minutes");
            return Err("brew_timeout".to_string());
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        logger::log_info(&format!("Homebrew upgrade succeeded: {}", stdout));
        Ok(stdout)
    } else {
        logger::log_error(&format!(
            "brew upgrade failed - stdout: {} stderr: {}",
            stdout, stderr
        ));
        // Return structured error key for frontend i18n
        if stderr.contains("already installed") || stdout.contains("already installed") {
            Err("brew_already_latest".to_string())
        } else {
            Err("brew_upgrade_failed".to_string())
        }
    }
}

/// Check for updates by executing our official installer shell script
pub async fn check_update_via_script() -> Result<UpdateInfo, String> {
    logger::log_info("Executing installer shell script to check for updates...");

    #[cfg(target_os = "windows")]
    {
        let mut ps_cmd = tokio::process::Command::new("powershell");
        ps_cmd.creation_flags_windows().args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
        ]);

        if std::path::Path::new("install.ps1").exists() {
            ps_cmd.args(["-File", ".\\install.ps1", "-CheckUpdate"]);
        } else {
            ps_cmd.args([
                "-Command",
                "& { $script = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($script)) -CheckUpdate }",
            ]);
        }

        let output = ps_cmd
            .output()
            .await
            .map_err(|e| format!("Failed to run powershell update check: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !stdout.is_empty() {
            if let Some(start_idx) = stdout.find('{') {
                if let Some(end_idx) = stdout.rfind('}') {
                    let json_str = &stdout[start_idx..=end_idx];
                    if let Ok(info) = serde_json::from_str::<UpdateInfo>(json_str) {
                        return Ok(info);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let script_cmd = if std::path::Path::new("install.sh").exists() {
            "./install.sh --check-update 2>/dev/null".to_string()
        } else {
            "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --check-update 2>/dev/null".to_string()
        };

        let output = tokio::process::Command::new("bash")
            .args(["-c", &script_cmd])
            .output()
            .await
            .map_err(|e| format!("Failed to run bash update check: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !stdout.is_empty() {
            if let Some(start_idx) = stdout.find('{') {
                if let Some(end_idx) = stdout.rfind('}') {
                    let json_str = &stdout[start_idx..=end_idx];
                    if let Ok(info) = serde_json::from_str::<UpdateInfo>(json_str) {
                        return Ok(info);
                    }
                }
            }
        }
    }

    logger::log_warn(
        "Script update check did not yield parseable result, falling back to check_for_updates",
    );
    check_for_updates().await
}

/// Run official installer to update the tool to the latest version
pub async fn run_installer_update() -> Result<String, String> {
    logger::log_info("Starting 3-stage delegated CLI updater for application update...");

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate current executable path: {}", e))?;
    let current_dir = current_exe
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let my_pid = std::process::id();
    let my_dir = current_dir.to_string_lossy().to_string();

    match crate::modules::delegate_updater::prepare_isolated_update_cli(my_pid) {
        Ok(temp_agm) => {
            logger::log_info(&format!(
                "Prepared isolated Update CLI copy at: {:?}",
                temp_agm
            ));

            match crate::modules::delegate_updater::spawn_delegated_update_cli(
                &temp_agm,
                my_pid,
                &current_dir,
                &current_exe,
                true,
                None,
            ) {
                Ok(child_pid) => {
                    logger::log_info(&format!(
                        "Launched delegated Update CLI process (PID: {}). Scheduling UI exit in 600ms...",
                        child_pid
                    ));
                    crate::modules::notification_hub::notify_system_updated(
                        CURRENT_VERSION,
                        "latest (delegated update CLI launched)",
                        Some("Delegated Update CLI (`agm-update-cli`) launched in isolated temp folder. It will run `agm update` and then `agm open-ui` to reopen the UI automatically."),
                    );

                    // Schedule unconditional clean exit after returning IPC response so tray_enabled prevent_exit() cannot hold file locks
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(650));
                        std::process::exit(0);
                    });

                    return Ok(
                        "Delegated Update CLI started (`agm-update-cli` -> `agm update` -> `agm open-ui`). Closing UI to apply update and reopen automatically."
                            .to_string(),
                    );
                }
                Err(e) => {
                    logger::log_error(&format!(
                        "Failed to spawn delegated Update CLI ({}), falling back to direct installer",
                        e
                    ));
                }
            }
        }
        Err(e) => {
            logger::log_warn(&format!(
                "Failed to prepare isolated Update CLI ({}), falling back to direct installer",
                e
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Fallback: On Windows, launch the installer in a visible, detached PowerShell process
        let dir_arg = format!("-InstallDir \"{}\"", my_dir);
        let ps_body = if current_dir.join("install.ps1").exists() {
            format!("Write-Host 'Updating Antigravity Tools...' -ForegroundColor Cyan; .\\install.ps1 -Update -NoLaunch {}", dir_arg)
        } else {
            format!("Write-Host 'Updating Antigravity Tools from official release...' -ForegroundColor Cyan; $s = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($s)) -Update -NoLaunch {}", dir_arg)
        };

        let mut spawn_cmd = std::process::Command::new("cmd.exe");
        spawn_cmd.args([
            "/c",
            "start",
            "Antigravity Tools Updater",
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_body,
        ]);

        match spawn_cmd.spawn() {
            Ok(_) => {
                logger::log_info("Successfully launched visible detached installer process.");
                crate::modules::notification_hub::notify_system_updated(
                    CURRENT_VERSION,
                    "latest (installer launched)",
                    Some("Official Windows installer update launched. Package download and binary replacement in progress."),
                );
                Ok(
                    "Official installer launched successfully! Check the update console window."
                        .to_string(),
                )
            }
            Err(e) => {
                logger::log_error(&format!("Failed to spawn updater process: {}", e));
                Err(format!("Failed to spawn updater: {}", e))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = if std::path::Path::new("install.sh").exists() {
            "bash ./install.sh --update".to_string()
        } else {
            "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --update".to_string()
        };

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tokio::process::Command::new("bash")
                .args(["-c", &cmd])
                .output(),
        )
        .await;

        let output = match result {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => return Err(format!("Failed to execute installer: {}", e)),
            Err(_) => return Err("Installer timed out after 5 minutes".to_string()),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            logger::log_info(&format!("Installer update succeeded: {}", stdout));
            crate::modules::notification_hub::notify_system_updated(
                CURRENT_VERSION,
                "latest",
                Some("Installer update script executed successfully."),
            );
            Ok(stdout)
        } else {
            let error_detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else if !stdout.trim().is_empty() {
                let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
                lines
                    .iter()
                    .rev()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                format!("Process exited with status code {:?}", output.status.code())
            };

            logger::log_error(&format!(
                "Installer update failed - stdout: {} stderr: {}",
                stdout, stderr
            ));
            Err(format!("Installer update failed: {}", error_detail))
        }
    }
}
