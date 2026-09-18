//! Inbound Email Poller and Remote Execution Bridge
//! Reads last 5 unread messages via IMAP, parses instruction subjects,
//! and dispatches prompts, CLI execution, or instance management.

#![allow(dead_code)]

use crate::modules::email_sender;
use crate::modules::email_vault_db::{self, EmailAccount, EmailInboundAuditLog};
use chrono::Utc;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;
use uuid::Uuid;

/// Parsed inbound email command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundAction {
    PromptInjection {
        project_name: String,
        prompt: String,
    },
    CliExecution {
        target_ip: String,
        command: String,
    },
    InstanceCreate {
        profile_name: String,
    },
    AccountRotate,
    StatusQuery,
    HelpRequest,
    Ignored {
        reason: String,
    },
}

/// Raw message header and body
#[derive(Debug, Clone)]
pub struct RawEmailMessage {
    pub message_id: String,
    pub from: String,
    pub subject: String,
    pub body: String,
}

/// Parse subject and body into strongly typed InboundAction
pub fn parse_email_command(subject: &str, body: &str) -> InboundAction {
    let clean_subj = subject.trim();
    let lower_subj = clean_subj.to_lowercase();

    if lower_subj.starts_with("project:") || lower_subj.starts_with("project-prompt:") {
        let prefix_len = if lower_subj.starts_with("project:") {
            "project:".len()
        } else {
            "project-prompt:".len()
        };
        let project_name = clean_subj[prefix_len..].trim().to_string();
        return InboundAction::PromptInjection {
            project_name,
            prompt: body.trim().to_string(),
        };
    }

    if lower_subj.starts_with("exec:") || lower_subj.starts_with("command:") {
        let prefix_len = if lower_subj.starts_with("exec:") {
            "exec:".len()
        } else {
            "command:".len()
        };
        let target_ip = clean_subj[prefix_len..].trim().to_string();
        return InboundAction::CliExecution {
            target_ip,
            command: body.trim().to_string(),
        };
    }

    if lower_subj.starts_with("instance: new") || lower_subj.starts_with("instance: create") {
        let profile = body.trim().lines().next().unwrap_or("email-spawned").trim().to_string();
        return InboundAction::InstanceCreate {
            profile_name: if profile.is_empty() { "email-spawned".to_string() } else { profile },
        };
    }

    if lower_subj.starts_with("rotate: accounts") || lower_subj.starts_with("account: rotate") {
        return InboundAction::AccountRotate;
    }

    if lower_subj == "status" || lower_subj.starts_with("query") {
        return InboundAction::StatusQuery;
    }

    if lower_subj == "help" {
        return InboundAction::HelpRequest;
    }

    InboundAction::Ignored {
        reason: format!("Subject '{}' does not match any recognized command pattern", subject),
    }
}

/// Process a parsed inbound email action
pub fn execute_inbound_action(
    msg: &RawEmailMessage,
    action: InboundAction,
    local_machine_ip: &str,
    local_machine_name: &str,
) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let action_str;
    let mut status = "success".to_string();
    let mut result_summary;

    match action {
        InboundAction::PromptInjection { project_name, prompt } => {
            action_str = "prompt_injection".to_string();
            // Match against running projects in repo_db
            let projects = crate::modules::repo_db::list_running_projects()?;
            let target_proj = projects.iter().find(|p| {
                p.repo_name.eq_ignore_ascii_case(&project_name)
                    || p.id.to_lowercase().contains(&project_name.to_lowercase())
            });

            if let Some(proj) = target_proj {
                let p_id = Uuid::new_v4().to_string();
                if let Ok(conn) = crate::modules::repo_db::connect_db() {
                    let _ = conn.execute(
                        "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, 'running', ?, ?)",
                        rusqlite::params![&p_id, &proj.id, &proj.instance_id, &proj.repo_path, &prompt, now, now],
                    );
                }
                result_summary = format!("Prompt injected into project '{}' (id: {})", proj.repo_name, p_id);
            } else {
                status = "rejected".to_string();
                result_summary = format!("Project '{}' not found among active running projects", project_name);
            }
        }
        InboundAction::CliExecution { target_ip, command } => {
            action_str = "cli_exec".to_string();
            // Check IP match
            let ip_matches = target_ip.is_empty()
                || target_ip == "any"
                || target_ip == "localhost"
                || target_ip == local_machine_ip;

            if !ip_matches {
                status = "rejected".to_string();
                result_summary = format!(
                    "Command IP mismatch: target '{}' != local '{}'",
                    target_ip, local_machine_ip
                );
            } else {
                let trimmed_cmd = command.trim();
                let output = execute_safe_cli_command(trimmed_cmd);
                let (code, text) = match output {
                    Ok(res) => (0, res),
                    Err(err) => (1, err),
                };

                result_summary = format!("Executed '{}' (exit: {})", trimmed_cmd, code);
                let (_, html) = email_sender::render_exec_result_email(
                    trimmed_cmd,
                    code,
                    &text,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    &format!("[AGM Response] Exec: {}", trimmed_cmd),
                    &html,
                    &[msg.from.clone()],
                );
            }
        }
        InboundAction::InstanceCreate { profile_name } => {
            action_str = "instance_create".to_string();
            let _ = crate::modules::instance::create_instance(&profile_name);
            result_summary = format!("Spawned instance profile '{}'", profile_name);
        }
        InboundAction::AccountRotate => {
            action_str = "rotate".to_string();
            result_summary = "Triggered account rotation".to_string();
        }
        InboundAction::StatusQuery => {
            action_str = "status_query".to_string();
            let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
            let proj_names: Vec<String> = projects.into_iter().map(|p| p.repo_name).collect();
            let (_, html) = email_sender::render_idle_projects_email(
                &proj_names,
                local_machine_name,
                local_machine_ip,
            );
            let _ = email_sender::dispatch_email_with_failover(
                "[AGM Status Report] Active Projects & Instances",
                &html,
                &[msg.from.clone()],
            );
            result_summary = format!("Reported {} projects to '{}'", proj_names.len(), msg.from);
        }
        InboundAction::HelpRequest => {
            action_str = "help".to_string();
            let (subj, html) = email_sender::render_help_email(local_machine_name, local_machine_ip);
            let _ = email_sender::dispatch_email_with_failover(&subj, &html, &[msg.from.clone()]);
            result_summary = format!("Dispatched help cheat sheet to '{}'", msg.from);
        }
        InboundAction::Ignored { reason } => {
            action_str = "ignored".to_string();
            status = "rejected".to_string();
            result_summary = reason;
        }
    }

    let audit = EmailInboundAuditLog {
        id: Uuid::new_v4().to_string(),
        message_id: msg.message_id.clone(),
        sender_email: msg.from.clone(),
        subject: msg.subject.clone(),
        action_type: action_str,
        action_payload: msg.body.chars().take(255).collect(),
        execution_status: status,
        execution_result: result_summary.clone(),
        received_at: now,
    };
    let _ = email_vault_db::record_inbound_audit_log(audit);

    Ok(result_summary)
}

/// Execute approved CLI / GitMap command safely
fn execute_safe_cli_command(cmd_str: &str) -> Result<String, String> {
    if cmd_str.is_empty() {
        return Err("Empty command instruction".to_string());
    }

    #[cfg(target_os = "windows")]
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", cmd_str])
        .output()
        .map_err(|e| format!("Failed to run command on Windows: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", cmd_str])
        .output()
        .map_err(|e| format!("Failed to run command on Unix: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Error (exit code {:?}):\n{}\n{}", output.status.code(), stdout, stderr))
    }
}

/// Poll unread messages via IMAP connection
pub fn poll_unread_messages(
    account: &EmailAccount,
    max_messages: usize,
) -> Result<Vec<RawEmailMessage>, String> {
    let password = email_vault_db::get_account_secret(&account.id).unwrap_or_default();
    let addr = format!("{}:{}", account.imap_host, account.imap_port);

    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid IMAP host/port '{}': {}", addr, e))?,
        Duration::from_secs(10),
    )
    .map_err(|e| format!("IMAP connection to '{}' failed: {}", addr, e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    // Read greeting
    let _ = read_imap_response(&mut stream);

    // Login
    send_imap_cmd(&mut stream, "A01", &format!("LOGIN \"{}\" \"{}\"", account.email, password))?;
    let login_resp = read_imap_response(&mut stream)?;
    if !login_resp.contains("OK") {
        return Err(format!("IMAP login failed: {}", login_resp));
    }

    // Select Inbox
    send_imap_cmd(&mut stream, "A02", "SELECT INBOX")?;
    let _ = read_imap_response(&mut stream);

    // Search Unseen
    send_imap_cmd(&mut stream, "A03", "SEARCH UNSEEN")?;
    let search_resp = read_imap_response(&mut stream)?;

    let mut ids: Vec<&str> = search_resp
        .lines()
        .find(|l| l.starts_with("* SEARCH"))
        .unwrap_or("")
        .split_whitespace()
        .filter(|&w| w != "*" && w != "SEARCH")
        .collect();

    // Cap to last N unread messages
    if ids.len() > max_messages {
        ids = ids[ids.len() - max_messages..].to_vec();
    }

    let mut messages = Vec::new();
    for (idx, id) in ids.iter().enumerate() {
        let tag = format!("A{:02}", idx + 10);
        send_imap_cmd(&mut stream, &tag, &format!("FETCH {} (BODY[HEADER.FIELDS (SUBJECT FROM MESSAGE-ID)] BODY[TEXT])", id))?;
        let fetch_resp = read_imap_response(&mut stream)?;
        if let Some(msg) = parse_raw_fetch_response(&fetch_resp) {
            messages.push(msg);
        }
    }

    let _ = send_imap_cmd(&mut stream, "A99", "LOGOUT");
    Ok(messages)
}

fn send_imap_cmd(stream: &mut TcpStream, tag: &str, cmd: &str) -> Result<(), String> {
    let line = format!("{} {}\r\n", tag, cmd);
    stream
        .write_all(line.as_bytes())
        .map_err(|e| format!("Failed to send IMAP command '{}': {}", cmd, e))
}

fn read_imap_response(stream: &mut TcpStream) -> Result<String, String> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    Ok(String::from_utf8_lossy(&buf[0..n]).to_string())
}

/// Parse IMAP fetch snippet into RawEmailMessage
fn parse_raw_fetch_response(resp: &str) -> Option<RawEmailMessage> {
    let mut subject = String::new();
    let mut from = String::new();
    let mut message_id = Uuid::new_v4().to_string();

    for line in resp.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("subject:") {
            subject = line["subject:".len()..].trim().to_string();
        } else if lower.starts_with("from:") {
            from = line["from:".len()..].trim().to_string();
        } else if lower.starts_with("message-id:") {
            message_id = line["message-id:".len()..].trim().to_string();
        }
    }

    let body = resp.split("\r\n\r\n").nth(1).unwrap_or("").trim().to_string();

    Some(RawEmailMessage {
        message_id,
        from,
        subject,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_command() {
        let action = parse_email_command("Project: my-awesome-app", "Please fix bug #12");
        match action {
            InboundAction::PromptInjection { project_name, prompt } => {
                assert_eq!(project_name, "my-awesome-app");
                assert_eq!(prompt, "Please fix bug #12");
            }
            _ => panic!("Expected PromptInjection"),
        }
    }

    #[test]
    fn test_parse_exec_command() {
        let action = parse_email_command("exec: 192.168.1.50", "gitmap status");
        match action {
            InboundAction::CliExecution { target_ip, command } => {
                assert_eq!(target_ip, "192.168.1.50");
                assert_eq!(command, "gitmap status");
            }
            _ => panic!("Expected CliExecution"),
        }
    }

    #[test]
    fn test_parse_help_command() {
        let action = parse_email_command("help", "");
        assert_eq!(action, InboundAction::HelpRequest);
    }
}
