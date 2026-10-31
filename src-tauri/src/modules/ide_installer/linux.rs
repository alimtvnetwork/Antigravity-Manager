//! Linux installer dispatch engine (Snap, APT, and AppImage ladder).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use super::types::{IdeTarget, InstallOptions, InstallResult};
use super::verify::find_ide_executable;
use crate::error::AppError;
use std::process::Command;

/// Returns the primary installation command string for the target IDE on Linux.
pub fn get_install_command(target: IdeTarget) -> (String, Vec<String>) {
    match target {
        IdeTarget::Antigravity => (
            "sh".to_string(),
            vec![
                "-c".to_string(),
                "mkdir -p ~/.local/bin && curl -fsSL -o ~/.local/bin/antigravity https://dl.google.com/antigravity/linux/latest/antigravity.AppImage && chmod +x ~/.local/bin/antigravity".to_string(),
            ],
        ),
        IdeTarget::Cursor => (
            "snap".to_string(),
            vec!["install".to_string(), "cursor".to_string(), "--classic".to_string()],
        ),
        IdeTarget::VsCode => (
            "snap".to_string(),
            vec!["install".to_string(), "code".to_string(), "--classic".to_string()],
        ),
    }
}

/// Executes installation of an IDE target on Linux.
pub fn execute_linux_install(
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
