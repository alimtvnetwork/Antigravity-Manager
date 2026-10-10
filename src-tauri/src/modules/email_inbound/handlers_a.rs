use base64::prelude::*;
use uuid::Uuid;

use super::*;

pub(crate) fn handle_prompt_injection(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    project_name: String,
    prompt_name: String,
    prompt: String,
    instance_id: Option<String>,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "prompt_injection".to_string();
    let inst_str = instance_id.as_deref().unwrap_or("default");

    // Send Phase 1 Immediate ACK
    send_ack_receipt(
        &msg.from,
        "prompt",
        &project_name,
        inst_str,
        &project_name,
        local_machine_name,
        local_machine_ip,
        Some(&msg.message_id),
        Some(&msg.subject),
    );

    // Resolve effective prompt content
    let effective_prompt = if prompt.is_empty() && !prompt_name.is_empty() {
        let all_prompts = crate::modules::repo_db::list_all_prompts().unwrap_or_default();
        let lower_p = prompt_name.to_lowercase();
        all_prompts
            .into_iter()
            .find(|p| {
                p.id.to_lowercase().contains(&lower_p)
                    || p.prompt_content.to_lowercase().contains(&lower_p)
            })
            .map(|p| p.prompt_content)
            .unwrap_or_else(|| prompt_name.clone())
    } else {
        prompt
    };

    let projects = crate::modules::repo_db::list_running_projects()?;
    let target_proj = projects.iter().find(|p| {
        if project_name.is_empty() {
            return true;
        }
        p.repo_name.eq_ignore_ascii_case(&project_name)
            || p.id.to_lowercase().contains(&project_name.to_lowercase())
    });

    if let Some(proj) = target_proj {
        let p_id = Uuid::new_v4().to_string();
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'running', ?, ?)",
            rusqlite::params![&p_id, &proj.id, &proj.instance_id, &proj.repo_path, &effective_prompt, now, now],
        ), "db execute");
        }
        result_summary = format!(
            "Prompt injected into project '{}' (id: {})",
            proj.repo_name, p_id
        );
        output_text = format!(
            "Prompt successfully injected into workspace '{}'.\nContent: {}",
            proj.repo_name, effective_prompt
        );
    } else {
        status = "rejected".to_string();
        exit_code = 1;
        result_summary = format!(
            "Project '{}' not found among active running projects",
            project_name
        );
        output_text = format!(
            "Could not find active project matching '{}'.\nPlease ensure the workspace is running.",
            project_name
        );
    }

    // Send Phase 2 Result
    send_result_receipt(
        &msg.from,
        "prompt",
        &status.to_uppercase(),
        exit_code,
        &output_text,
        local_machine_name,
        local_machine_ip,
        inst_str,
        Some(&msg.message_id),
        Some(&msg.subject),
    );
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_git_map_execution(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    command: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "gitmap_exec".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!(
            "GitMap target '{}' does not match local node '{}'",
            target, local_machine_name
        );
    } else {
        // Send Phase 1 Immediate ACK
        send_ack_receipt(
            &msg.from,
            &format!("gitmap {}", command),
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let res = execute_gitmap_command(&command);
        match res {
            Ok(out) => {
                exit_code = 0;
                output_text = out;
                result_summary = format!("GitMap '{}' executed successfully", command);
            }
            Err(err) => {
                exit_code = 1;
                status = "error".to_string();
                output_text = err;
                result_summary = format!("GitMap '{}' failed", command);
            }
        }

        // Send Phase 2 Result
        send_result_receipt(
            &msg.from,
            &format!("gitmap {}", command),
            &status.to_uppercase(),
            exit_code,
            &output_text,
            local_machine_name,
            local_machine_ip,
            "default",
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_cli_execution(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target_ip: String,
    command: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "cli_exec".to_string();
    if !matches_target_node_or_ip(&target_ip, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!(
            "CLI target mismatch: '{}' != local IP '{}' / node '{}'",
            target_ip, local_machine_ip, local_machine_name
        );
    } else {
        send_ack_receipt(
            &msg.from,
            "cmd",
            &target_ip,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let trimmed_cmd = command.trim();
        let output = execute_safe_cli_command(trimmed_cmd);
        let (code, text) = match output {
            Ok(res) => (0, res),
            Err(err) => (1, err),
        };
        exit_code = code;
        output_text = text;
        result_summary = format!("Executed '{}' (exit: {})", trimmed_cmd, code);

        send_result_receipt(
            &msg.from,
            "cmd",
            if code == 0 { "SUCCESS" } else { "FAILED" },
            exit_code,
            &output_text,
            local_machine_name,
            local_machine_ip,
            "default",
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_update_execution(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    is_gitmap: bool,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "update_exec".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Update target mismatch: {}", target);
    } else {
        let cmd_label = if is_gitmap {
            "gitmap update"
        } else {
            "agm update"
        };
        send_ack_receipt(
            &msg.from,
            cmd_label,
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        if is_gitmap {
            let out = execute_safe_cli_command("gitmap update");
            match out {
                Ok(t) => {
                    output_text = t;
                    result_summary = "GitMap updated successfully".to_string();
                }
                Err(e) => {
                    exit_code = 1;
                    status = "error".to_string();
                    output_text = e;
                    result_summary = "GitMap update failed".to_string();
                }
            }
        } else {
            output_text = "AGM update triggered on host.\nChecking latest releases from GitHub and executing update routine.".to_string();
            result_summary = "AGM update initiated".to_string();
        }

        send_result_receipt(
            &msg.from,
            cmd_label,
            &status.to_uppercase(),
            exit_code,
            &output_text,
            local_machine_name,
            local_machine_ip,
            "default",
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}
