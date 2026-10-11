use crate::utils::command::CommandExtWrapper;
use base64::prelude::*;
use std::process::Command;

use super::*;

/// Helper to execute approved CLI command safely
pub(crate) fn execute_safe_cli_command(cmd_str: &str) -> Result<String, String> {
    let mut clean_cmd = cmd_str.trim();
    if clean_cmd.is_empty() {
        return Err("Empty command instruction".to_string());
    }

    let lower = clean_cmd.to_lowercase();
    if lower.starts_with("powershell:") {
        clean_cmd = clean_cmd["powershell:".len()..].trim();
    } else if lower.starts_with("ps:") {
        clean_cmd = clean_cmd["ps:".len()..].trim();
    } else if lower.starts_with("ps ") {
        clean_cmd = clean_cmd[3..].trim();
    } else if lower.starts_with("pwsh:") {
        clean_cmd = clean_cmd["pwsh:".len()..].trim();
    } else if lower.starts_with("bash:") {
        clean_cmd = clean_cmd["bash:".len()..].trim();
    } else if lower.starts_with("sh:") {
        clean_cmd = clean_cmd["sh:".len()..].trim();
    } else if lower.starts_with("cmd:") {
        clean_cmd = clean_cmd["cmd:".len()..].trim();
    }

    #[cfg(target_os = "windows")]
    let output = {
        let mut cmd = Command::new("powershell.exe");
        cmd.creation_flags_windows().args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            clean_cmd,
        ]);
        cmd.output()
            .map_err(|e| format!("Failed to run command on Windows: {}", e))?
    };

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", clean_cmd])
        .output()
        .map_err(|e| format!("Failed to run command on Unix: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "Error (exit code {:?}):\n{}\n{}",
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

/// Helper to execute GitMap command, collapsing duplicate prefixes
pub(crate) fn execute_gitmap_command(cmd_args: &str) -> Result<String, String> {
    let clean_cmd = if cmd_args.trim().to_lowercase().starts_with("gitmap ") {
        cmd_args.trim()["gitmap ".len()..].trim()
    } else {
        cmd_args.trim()
    };

    let full_command = format!("gitmap {}", clean_cmd);
    execute_safe_cli_command(&full_command)
}
