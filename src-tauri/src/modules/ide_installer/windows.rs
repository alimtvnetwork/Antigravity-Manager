//! Windows installer dispatch engine (Winget & PowerShell ladder).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use super::types::{IdeTarget, InstallOptions, InstallResult};
use super::verify::find_ide_executable;
use crate::error::AppError;
use std::process::Command;

/// Returns the primary installation command string for the target IDE on Windows.
pub fn get_install_command(target: IdeTarget) -> (String, Vec<String>) {
    match target {
        IdeTarget::Antigravity => (
            "powershell.exe".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                "& { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $installer = Join-Path $env:TEMP 'AntigravitySetup.exe'; Invoke-WebRequest -Uri 'https://dl.google.com/antigravity/win/latest/AntigravitySetup.exe' -OutFile $installer; Start-Process -FilePath $installer -ArgumentList '/passive','/norestart' -Wait }".to_string(),
            ],
        ),
        IdeTarget::Cursor => (
            "winget.exe".to_string(),
            vec![
                "install".to_string(),
                "Anysphere.Cursor".to_string(),
                "--silent".to_string(),
                "--accept-package-agreements".to_string(),
                "--accept-source-agreements".to_string(),
            ],
        ),
        IdeTarget::VsCode => (
            "winget.exe".to_string(),
            vec![
                "install".to_string(),
                "Microsoft.VisualStudioCode".to_string(),
                "--silent".to_string(),
                "--accept-package-agreements".to_string(),
                "--accept-source-agreements".to_string(),
            ],
        ),
    }
}

/// Executes installation of an IDE target on Windows.
pub fn execute_windows_install(
    target: IdeTarget,
    options: InstallOptions,
) -> Result<InstallResult, AppError> {
    let (program, args) = get_install_command(target);
    let full_cmd = format!("{} {}", program, args.join(" "));

    if options.dry_run {
        return Ok(InstallResult {
            target,
            success: true,
            installed_path: find_ide_executable(target).map(|p| p.to_string_lossy().to_string()),
            command_executed: full_cmd,
            output: "[DRY-RUN] Command planned but not executed.".to_string(),
        });
    }

    let mut cmd = Command::new(&program);
    cmd.args(&args);

    let output = cmd.output().map_err(|e| {
        AppError::Process(format!(
            "Failed to spawn installer command '{full_cmd}': {e}"
        ))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined_output = format!("{}\n{}", stdout, stderr).trim().to_string();

    let post_path = find_ide_executable(target);
    let success = output.status.success() || post_path.is_some();

    Ok(InstallResult {
        target,
        success,
        installed_path: post_path.map(|p| p.to_string_lossy().to_string()),
        command_executed: full_cmd,
        output: combined_output,
    })
}
