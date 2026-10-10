use super::*;

#[tauri::command]
pub async fn test_execute_cli_command(command: String) -> AppResult<CliExecResult> {
    let cmd_str = command.trim();
    if cmd_str.is_empty() {
        return Err(AppError::Config("Command cannot be empty".to_string()));
    }

    let lower_cmd = cmd_str.to_lowercase();
    let is_ai_prompt = lower_cmd.starts_with("project:")
        || lower_cmd.starts_with("prompt:")
        || lower_cmd.starts_with("prompt ")
        || lower_cmd.starts_with("prompt-name:")
        || lower_cmd.starts_with("prompt instruction:")
        || lower_cmd.starts_with("prompt injection:")
        || lower_cmd.starts_with("prompt-injection:")
        || lower_cmd.starts_with("ai:")
        || lower_cmd.starts_with("instruction:")
        || lower_cmd.starts_with("inject:");

    if is_ai_prompt {
        let lines: Vec<&str> = cmd_str.lines().collect();
        let mut project = "Default";
        let mut prompt_lines: Vec<&str> = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let l_trim = line.trim();
            let l_lower = l_trim.to_lowercase();
            if idx == 0 {
                if let Some(pos) = l_trim.find(':') {
                    let candidate = l_trim[pos + 1..].trim();
                    if !candidate.is_empty() {
                        project = candidate;
                    }
                }
            } else if l_lower.starts_with("project:") {
                let candidate = l_trim["project:".len()..].trim();
                if !candidate.is_empty() {
                    project = candidate;
                }
            } else {
                prompt_lines.push(*line);
            }
        }

        let prompt_text = prompt_lines.join("\n").trim().to_string();
        let prompt_display = if prompt_text.is_empty() {
            cmd_str.to_string()
        } else {
            prompt_text
        };

        let machine_name = email_watcher::detect_machine_name();
        let machine_ip = email_watcher::detect_local_ip();

        return Ok(CliExecResult {
            exit_code: 0,
            stdout: format!(
                "[AI Prompt Simulation]\nNode: {}\nTarget Project: {}\nInstructions:\n{}\n\nStatus: Prompt validated and ready for remote execution / agent injection.",
                machine_name, project, prompt_display
            ),
            stderr: String::new(),
            success: true,
            machine_name,
            machine_ip,
        });
    }

    // Strip shell prefixes if present
    let mut clean_cmd = cmd_str;
    if lower_cmd.starts_with("powershell:") {
        clean_cmd = clean_cmd["powershell:".len()..].trim();
    } else if lower_cmd.starts_with("ps:") {
        clean_cmd = clean_cmd["ps:".len()..].trim();
    } else if lower_cmd.starts_with("ps ") {
        clean_cmd = clean_cmd[3..].trim();
    } else if lower_cmd.starts_with("pwsh:") {
        clean_cmd = clean_cmd["pwsh:".len()..].trim();
    } else if lower_cmd.starts_with("bash:") {
        clean_cmd = clean_cmd["bash:".len()..].trim();
    } else if lower_cmd.starts_with("sh:") {
        clean_cmd = clean_cmd["sh:".len()..].trim();
    } else if lower_cmd.starts_with("cmd:") {
        clean_cmd = clean_cmd["cmd:".len()..].trim();
    }

    #[cfg(target_os = "windows")]
    let output = {
        let mut cmd = std::process::Command::new("powershell.exe");
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
        cmd.output().map_err(|e| {
            AppError::Process(format!("Failed to execute PowerShell on Windows: {}", e))
        })?
    };

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .args(["-c", clean_cmd])
        .output()
        .map_err(|e| AppError::Process(format!("Failed to execute command on Unix: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();
    let machine_name = email_watcher::detect_machine_name();
    let machine_ip = email_watcher::detect_local_ip();

    Ok(CliExecResult {
        exit_code,
        stdout,
        stderr,
        success,
        machine_name,
        machine_ip,
    })
}
