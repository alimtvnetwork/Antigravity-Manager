//! Inbound Email Poller and Remote Execution Bridge
//! Reads last 5 unread messages via IMAP, parses instruction subjects/bodies,
//! and dispatches prompts, CLI execution, or instance management with full HTML reply receipts.

#![allow(dead_code)]

use crate::modules::email_sender::{self, EmailStream};
use crate::modules::email_vault_db::{self, EmailAccount, EmailInboundAuditLog};
use crate::utils::command::CommandExtWrapper;
use chrono::Utc;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
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
    NamedPromptExecution {
        prompt_query: String,
    },
    CliExecution {
        target_ip: String,
        command: String,
    },
    InstanceCreate {
        profile_name: String,
    },
    AccountRotate,
    FastForward {
        target_node: String,
    },
    MultiNodeSnapshotQuery,
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

/// Clean subject by removing reply/forward prefixes
fn strip_email_prefixes(subject: &str) -> String {
    let mut clean = subject.trim();
    loop {
        let lower = clean.to_lowercase();
        if lower.starts_with("re:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("fwd:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fw:") {
            clean = clean[3..].trim();
        } else {
            break;
        }
    }
    clean.to_string()
}

/// Escape raw console output for clean HTML embedding without spam scoring
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Parse subject and body into strongly typed InboundAction
pub fn parse_email_command(subject: &str, body: &str) -> InboundAction {
    let clean_subj = strip_email_prefixes(subject);
    let lower_subj = clean_subj.to_lowercase();

    // Check Subject First
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
        let profile = body
            .trim()
            .lines()
            .next()
            .unwrap_or("email-spawned")
            .trim()
            .to_string();
        return InboundAction::InstanceCreate {
            profile_name: if profile.is_empty() {
                "email-spawned".to_string()
            } else {
                profile
            },
        };
    }

    if lower_subj.starts_with("rotate: accounts") || lower_subj.starts_with("account: rotate") {
        return InboundAction::AccountRotate;
    }

    if lower_subj == "ff"
        || lower_subj.starts_with("ff:")
        || lower_subj.starts_with("fast-forward")
        || lower_subj.starts_with("fastforward")
    {
        let prefix_len = if lower_subj.starts_with("ff:") {
            "ff:".len()
        } else if lower_subj.starts_with("fast-forward:") {
            "fast-forward:".len()
        } else {
            0
        };
        let target_node = if prefix_len > 0 {
            clean_subj[prefix_len..].trim().to_string()
        } else {
            "*".to_string()
        };
        return InboundAction::FastForward { target_node };
    }

    if lower_subj.contains("how many machines")
        || lower_subj.contains("nodes running")
        || lower_subj.contains("cluster")
    {
        return InboundAction::MultiNodeSnapshotQuery;
    }

    if lower_subj == "status" || lower_subj.starts_with("query") {
        return InboundAction::StatusQuery;
    }

    // Check Named Prompt Execution
    let is_named = lower_subj.starts_with("named-prompt:");
    let is_run = lower_subj.starts_with("run-prompt:");
    let is_prompt = lower_subj.starts_with("prompt:");
    if is_named || is_run || is_prompt {
        let prefix_len = if is_named {
            "named-prompt:".len()
        } else if is_run {
            "run-prompt:".len()
        } else {
            "prompt:".len()
        };
        let query = clean_subj[prefix_len..].trim().to_string();
        let body_first = body.trim().lines().next().unwrap_or("").trim().to_string();
        let effective_query = if query.is_empty() { body_first } else { query };
        return InboundAction::NamedPromptExecution {
            prompt_query: effective_query,
        };
    }

    if lower_subj == "help" {
        return InboundAction::HelpRequest;
    }

    // Fallback: Check first line of body if user replied to an idle notification
    let first_line = body.lines().next().unwrap_or("").trim();
    let lower_first_line = first_line.to_lowercase();
    if lower_first_line.starts_with("project:") || lower_first_line.starts_with("project-prompt:") {
        let prefix_len = if lower_first_line.starts_with("project:") {
            "project:".len()
        } else {
            "project-prompt:".len()
        };
        let project_name = first_line[prefix_len..].trim().to_string();
        let prompt_body = body
            .lines()
            .skip(1)
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string();
        return InboundAction::PromptInjection {
            project_name,
            prompt: prompt_body,
        };
    }

    let is_body_named = lower_first_line.starts_with("named-prompt:");
    let is_body_run = lower_first_line.starts_with("run-prompt:");
    let is_body_prompt = lower_first_line.starts_with("prompt:");
    if is_body_named || is_body_run || is_body_prompt {
        let prefix_len = if is_body_named {
            "named-prompt:".len()
        } else if is_body_run {
            "run-prompt:".len()
        } else {
            "prompt:".len()
        };
        let query = first_line[prefix_len..].trim().to_string();
        return InboundAction::NamedPromptExecution {
            prompt_query: query,
        };
    }

    if lower_first_line.starts_with("exec:") || lower_first_line.starts_with("command:") {
        let prefix_len = if lower_first_line.starts_with("exec:") {
            "exec:".len()
        } else {
            "command:".len()
        };
        let target_ip = first_line[prefix_len..].trim().to_string();
        let cmd_body = body
            .lines()
            .skip(1)
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string();
        return InboundAction::CliExecution {
            target_ip,
            command: cmd_body,
        };
    }

    InboundAction::Ignored {
        reason: format!(
            "Subject '{}' does not match any recognized command pattern",
            subject
        ),
    }
}

/// Process a parsed inbound email action and dispatch bidirectional HTML reply
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
        InboundAction::PromptInjection {
            project_name,
            prompt,
        } => {
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
                result_summary = format!(
                    "Prompt injected into project '{}' (id: {})",
                    proj.repo_name, p_id
                );

                // Bidirectional confirmation email receipt
                let escaped_prompt = html_escape(&prompt);
                let reply_content = format!(
                    r#"<p><span class="badge badge-success">PROMPT INJECTED</span></p>
<p>Your prompt was successfully injected into workspace <strong>{}</strong>.</p>
<div class="cmd">{}</div>
<p>Antigravity is currently executing this instruction.</p>"#,
                    proj.repo_name, escaped_prompt
                );
                let html = wrap_card(
                    "Prompt Injection Receipt",
                    &reply_content,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    &format!("[AGM Receipt] Prompt Injected: {}", proj.repo_name),
                    &html,
                    &[msg.from.clone()],
                );
            } else {
                status = "rejected".to_string();
                let available_names: Vec<String> =
                    projects.iter().map(|p| p.repo_name.clone()).collect();
                result_summary = format!(
                    "Project '{}' not found among active running projects",
                    project_name
                );

                let reply_content = format!(
                    r#"<p><span class="badge badge-warn">PROJECT NOT FOUND</span></p>
<p>Could not find active project matching <strong>{}</strong>.</p>
<p>Available running projects on this machine:</p>
<ul>{}</ul>
<p>Please reply with <code>Project: &lt;exact-name&gt;</code> to try again.</p>"#,
                    project_name,
                    available_names
                        .iter()
                        .map(|n| format!("<li>{}</li>", n))
                        .collect::<Vec<_>>()
                        .join("")
                );
                let html = wrap_card(
                    "Prompt Injection Failed",
                    &reply_content,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    &format!("[AGM Alert] Project Not Found: {}", project_name),
                    &html,
                    &[msg.from.clone()],
                );
            }
        }
        InboundAction::CliExecution { target_ip, command } => {
            action_str = "cli_exec".to_string();
            // Check IP or Node Name match
            let ip_matches = target_ip.is_empty()
                || target_ip == "any"
                || target_ip == "localhost"
                || target_ip == local_machine_ip
                || target_ip.eq_ignore_ascii_case(local_machine_name);

            if !ip_matches {
                status = "rejected".to_string();
                result_summary = format!(
                    "Command target mismatch: target '{}' != local IP '{}' / node '{}'",
                    target_ip, local_machine_ip, local_machine_name
                );
                let reply_content = format!(
                    r#"<p><span class="badge badge-warn">IP MISMATCH</span></p>
<p>Instruction targeted IP <code>{}</code>, but this machine's local IP is <code>{}</code>.</p>
<p>Command was skipped to prevent execution on wrong node.</p>"#,
                    target_ip, local_machine_ip
                );
                let html = wrap_card(
                    "Execution Skipped",
                    &reply_content,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    "[AGM Alert] Command IP Mismatch",
                    &html,
                    &[msg.from.clone()],
                );
            } else {
                let trimmed_cmd = command.trim();
                let output = execute_safe_cli_command(trimmed_cmd);
                let (code, text) = match output {
                    Ok(res) => (0, res),
                    Err(err) => (1, err),
                };

                result_summary = format!("Executed '{}' (exit: {})", trimmed_cmd, code);
                let escaped_output = html_escape(&text);
                let (_, html) = email_sender::render_exec_result_email(
                    trimmed_cmd,
                    code,
                    &escaped_output,
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
            let create_res = crate::modules::instance::create_instance(profile_name.clone());
            result_summary = match create_res {
                Ok(_) => format!("Spawned instance profile '{}'", profile_name),
                Err(e) => format!("Failed to spawn instance: {}", e),
            };

            let reply_content = format!(
                r#"<p><span class="badge badge-success">INSTANCE CREATED</span></p>
<p>Successfully provisioned isolated IDE instance profile <strong>{}</strong> on Node <strong>{}</strong>.</p>
<p>You can launch it anytime via Antigravity Manager.</p>"#,
                profile_name, local_machine_name
            );
            let html = wrap_card(
                "Instance Created",
                &reply_content,
                local_machine_name,
                local_machine_ip,
            );
            let _ = email_sender::dispatch_email_with_failover(
                &format!("[AGM Response] Instance Created: {}", profile_name),
                &html,
                &[msg.from.clone()],
            );
        }
        InboundAction::FastForward { target_node } => {
            action_str = "fast_forward".to_string();
            let is_match = target_node == "*"
                || target_node.eq_ignore_ascii_case("all")
                || target_node.eq_ignore_ascii_case(local_machine_name)
                || target_node == local_machine_ip;

            if is_match {
                let _ = crate::modules::auto_switcher::trigger_manual_rotation();
                let current_account =
                    crate::modules::account::get_current_account().unwrap_or(None);
                let current_email = current_account
                    .map(|a| a.email)
                    .unwrap_or_else(|| "Default".to_string());
                result_summary =
                    format!("Fast Forward executed. Active account: {}", current_email);
                let reply_content = format!(
                    r#"<p><span class="badge badge-success">FAST FORWARD (DOUBLE PLAY)</span></p>
<p>Remote workspace rotation successfully executed on Node <strong>{}</strong> ({}).</p>
<p>Switched to freshest workspace/account: <strong>{}</strong>.</p>"#,
                    local_machine_name, local_machine_ip, current_email
                );
                let html = wrap_card(
                    "Fast Forward Result",
                    &reply_content,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    &format!("[AGM Response] Fast Forward (FF): {}", local_machine_name),
                    &html,
                    &[msg.from.clone()],
                );
            } else {
                status = "skipped".to_string();
                result_summary = format!(
                    "Fast Forward target mismatch: {} != {}",
                    target_node, local_machine_name
                );
            }
        }
        InboundAction::MultiNodeSnapshotQuery => {
            action_str = "cluster_snapshot".to_string();
            let uptime = crate::modules::supabase_sync::get_uptime_seconds();
            let uptime_min = uptime / 60;
            result_summary = format!("Multi-Node Snapshot generated by {}", local_machine_name);
            let reply_content = format!(
                r#"<p><span class="badge badge-info">CLUSTER SNAPSHOT</span></p>
<table style="width:100%; border-collapse:collapse; margin-top:12px; font-family:monospace; font-size:13px;">
  <thead>
    <tr style="background:#1e293b; color:#f8fafc; text-align:left;">
      <th style="padding:8px; border:1px solid #334155;">Node Alias</th>
      <th style="padding:8px; border:1px solid #334155;">IP Address</th>
      <th style="padding:8px; border:1px solid #334155;">Status</th>
      <th style="padding:8px; border:1px solid #334155;">Uptime</th>
    </tr>
  </thead>
  <tbody>
    <tr style="background:#0f172a; color:#e2e8f0;">
      <td style="padding:8px; border:1px solid #334155;"><strong>{}</strong></td>
      <td style="padding:8px; border:1px solid #334155;">{}</td>
      <td style="padding:8px; border:1px solid #334155; color:#10b981;">ONLINE</td>
      <td style="padding:8px; border:1px solid #334155;">{}m</td>
    </tr>
  </tbody>
</table>
<p style="margin-top:12px; font-size:12px; color:#94a3b8;">To send commands: reply with <code>Exec: &lt;Node-Alias&gt;</code> or fast-forward workspace with <code>FF: &lt;Node-Alias&gt;</code>.</p>"#,
                local_machine_name, local_machine_ip, uptime_min
            );
            let html = wrap_card(
                "Cluster Node Snapshot",
                &reply_content,
                local_machine_name,
                local_machine_ip,
            );
            let _ = email_sender::dispatch_email_with_failover(
                "[AGM Response] Cluster Snapshot: Online Nodes",
                &html,
                &[msg.from.clone()],
            );
        }
        InboundAction::AccountRotate => {
            action_str = "rotate".to_string();
            let _rotate_res = crate::modules::auto_switcher::check_and_rotate_if_needed();
            let current_account = crate::modules::account::get_current_account().unwrap_or(None);
            let current_email = current_account
                .map(|a| a.email)
                .unwrap_or_else(|| "Unknown".to_string());

            result_summary = format!(
                "Triggered account rotation. Current profile: {}",
                current_email
            );
            let reply_content = format!(
                r#"<p><span class="badge badge-info">PROFILE ROTATED</span></p>
<p>Account rotation was triggered remotely via mailbox command.</p>
<p>Active Profile: <strong>{}</strong></p>
<p>Node: <strong>{}</strong> ({})</p>"#,
                current_email, local_machine_name, local_machine_ip
            );
            let html = wrap_card(
                "Account Rotation Report",
                &reply_content,
                local_machine_name,
                local_machine_ip,
            );
            let _ = email_sender::dispatch_email_with_failover(
                "[AGM Response] Account Rotation Completed",
                &html,
                &[msg.from.clone()],
            );
        }
        InboundAction::NamedPromptExecution { prompt_query } => {
            action_str = "named_prompt_exec".to_string();
            let all_prompts = crate::modules::repo_db::list_all_prompts().unwrap_or_default();
            let lower_query = prompt_query.to_lowercase();

            let matched_prompt = all_prompts.iter().find(|p| {
                let id_match = p.id.to_lowercase().contains(&lower_query);
                let content_match = p.prompt_content.to_lowercase().contains(&lower_query);
                id_match || content_match
            });

            if let Some(found_prompt) = matched_prompt {
                let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
                let target_proj = projects
                    .iter()
                    .find(|p| p.id == found_prompt.project_id)
                    .or_else(|| projects.first());

                if let Some(proj) = target_proj {
                    let p_id = Uuid::new_v4().to_string();
                    if let Ok(conn) = crate::modules::repo_db::connect_db() {
                        let _ = conn.execute(
                            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
                             VALUES (?, ?, ?, ?, ?, 'running', ?, ?)",
                            rusqlite::params![&p_id, &proj.id, &proj.instance_id, &proj.repo_path, &found_prompt.prompt_content, now, now],
                        );
                    }
                    result_summary = format!(
                        "Named prompt '{}' found and dispatched into project '{}' (id: {})",
                        prompt_query, proj.repo_name, p_id
                    );

                    let escaped_prompt = html_escape(&found_prompt.prompt_content);
                    let reply_content = format!(
                        r#"<p><span class="badge badge-success">NAMED PROMPT EXECUTED</span></p>
<p>Found saved prompt matching <strong>{}</strong> and injected into workspace <strong>{}</strong>.</p>
<div class="cmd">{}</div>
<p>Antigravity is currently executing this instruction.</p>"#,
                        html_escape(&prompt_query),
                        proj.repo_name,
                        escaped_prompt
                    );
                    let html = wrap_card(
                        "Named Prompt Execution Receipt",
                        &reply_content,
                        local_machine_name,
                        local_machine_ip,
                    );
                    let _ = email_sender::dispatch_email_with_failover(
                        &format!("[AGM Receipt] Named Prompt Executed: {}", prompt_query),
                        &html,
                        &[msg.from.clone()],
                    );
                } else {
                    status = "rejected".to_string();
                    result_summary =
                        format!("Prompt found but no active running projects available");
                    let reply_content = format!(
                        r#"<p><span class="badge badge-warn">NO RUNNING WORKSPACES</span></p>
<p>Found saved prompt matching <strong>{}</strong>, but no active workspaces are currently running to execute it.</p>"#,
                        html_escape(&prompt_query)
                    );
                    let html = wrap_card(
                        "Named Prompt Execution Failed",
                        &reply_content,
                        local_machine_name,
                        local_machine_ip,
                    );
                    let _ = email_sender::dispatch_email_with_failover(
                        "[AGM Alert] No Active Workspaces",
                        &html,
                        &[msg.from.clone()],
                    );
                }
            } else {
                status = "rejected".to_string();
                result_summary = format!("No saved prompt found matching '{}'", prompt_query);
                let available_snippets: Vec<String> = all_prompts
                    .iter()
                    .take(5)
                    .map(|p| {
                        let short_id = &p.id[..8.min(p.id.len())];
                        let short_content: String = p.prompt_content.chars().take(40).collect();
                        format!(
                            "<li><code>{}</code>: {}...</li>",
                            short_id,
                            html_escape(&short_content)
                        )
                    })
                    .collect();

                let reply_content = format!(
                    r#"<p><span class="badge badge-warn">PROMPT NOT FOUND</span></p>
<p>Could not find any saved prompt matching <strong>{}</strong>.</p>
<p>Available saved prompts in queue:</p>
<ul>{}</ul>
<p>Reply with <code>prompt: &lt;keyword-or-id&gt;</code> or send a new prompt with <code>Project: &lt;name&gt;</code>.</p>"#,
                    html_escape(&prompt_query),
                    if available_snippets.is_empty() {
                        "<li>No saved prompts found</li>".to_string()
                    } else {
                        available_snippets.join("")
                    }
                );
                let html = wrap_card(
                    "Prompt Not Found",
                    &reply_content,
                    local_machine_name,
                    local_machine_ip,
                );
                let _ = email_sender::dispatch_email_with_failover(
                    &format!("[AGM Alert] Prompt Not Found: {}", prompt_query),
                    &html,
                    &[msg.from.clone()],
                );
            }
        }
        InboundAction::StatusQuery => {
            action_str = "status_query".to_string();
            let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
            let prompts = crate::modules::repo_db::list_backed_up_prompts().unwrap_or_default();
            let proj_names: Vec<String> = projects.into_iter().map(|p| p.repo_name).collect();

            let reply_content = format!(
                r#"<p><span class="badge badge-info">NODE STATUS</span></p>
<p>Node: <strong>{}</strong> &nbsp;|&nbsp; Local IP: <strong>{}</strong></p>
<p><strong>Running Projects ({}):</strong></p>
<ul>{}</ul>
<p><strong>Active Prompts in Queue:</strong> {}</p>"#,
                local_machine_name,
                local_machine_ip,
                proj_names.len(),
                proj_names
                    .iter()
                    .map(|n| format!("<li>{}</li>", n))
                    .collect::<Vec<_>>()
                    .join(""),
                prompts.len()
            );
            let html = wrap_card(
                "Node Status Report",
                &reply_content,
                local_machine_name,
                local_machine_ip,
            );
            let _ = email_sender::dispatch_email_with_failover(
                "[AGM Status Report] Running Projects & Queue",
                &html,
                &[msg.from.clone()],
            );
            result_summary = format!("Reported {} projects to '{}'", proj_names.len(), msg.from);
        }
        InboundAction::HelpRequest => {
            action_str = "help".to_string();
            let (subj, html) =
                email_sender::render_help_email(local_machine_name, local_machine_ip);
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

/// Helper to wrap simple response cards
fn wrap_card(title: &str, content_html: &str, machine_name: &str, machine_ip: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; }}
    .card {{ max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; overflow: hidden; }}
    .header {{ background: #0f172a; color: #ffffff; padding: 20px; }}
    .header h1 {{ margin: 0; font-size: 17px; }}
    .body {{ padding: 20px; color: #334155; line-height: 1.6; font-size: 14px; }}
    .badge {{ display: inline-block; padding: 3px 8px; border-radius: 6px; font-size: 11px; font-weight: 600; }}
    .badge-warn {{ background: #fef3c7; color: #92400e; }}
    .badge-info {{ background: #e0f2fe; color: #075985; }}
    .badge-success {{ background: #dcfce7; color: #166534; }}
    .footer {{ padding: 14px 20px; background: #f1f5f9; font-size: 12px; color: #64748b; border-top: 1px solid #e2e8f0; }}
    .cmd {{ background: #0f172a; color: #38bdf8; padding: 8px 12px; border-radius: 6px; font-family: monospace; font-size: 13px; margin: 12px 0; }}
  </style>
</head>
<body>
  <div class="card">
    <div class="header"><h1>Antigravity Manager · {}</h1></div>
    <div class="body">{}</div>
    <div class="footer">Node: <strong>{}</strong> | Local IP: <strong>{}</strong><br>Automated Mailbox Control</div>
  </div>
</body>
</html>"#,
        title, content_html, machine_name, machine_ip
    )
}

/// Execute approved CLI / GitMap command safely
fn execute_safe_cli_command(cmd_str: &str) -> Result<String, String> {
    if cmd_str.is_empty() {
        return Err("Empty command instruction".to_string());
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
            cmd_str,
        ]);
        cmd.output()
            .map_err(|e| format!("Failed to run command on Windows: {}", e))?
    };

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
        Err(format!(
            "Error (exit code {:?}):\n{}\n{}",
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

pub fn is_implicit_tls_imap(port: u16, encryption_type: &str) -> bool {
    if port == 993 {
        return true;
    }
    if port == 995 {
        return true;
    }
    let enc = encryption_type.trim();
    if enc.eq_ignore_ascii_case("SSL") {
        return true;
    }
    if enc.eq_ignore_ascii_case("IMAPS") {
        return true;
    }
    false
}

pub fn is_starttls_imap(port: u16, encryption_type: &str) -> bool {
    if is_implicit_tls_imap(port, encryption_type) {
        return false;
    }
    if port == 143 {
        return true;
    }
    let enc = encryption_type.trim();
    if enc.eq_ignore_ascii_case("STARTTLS") {
        return true;
    }
    if enc.eq_ignore_ascii_case("TLS") {
        return true;
    }
    false
}

/// Poll unread messages via IMAP connection
pub fn poll_unread_messages(
    account: &EmailAccount,
    max_messages: usize,
) -> Result<Vec<RawEmailMessage>, String> {
    let password = email_vault_db::get_account_secret(&account.id).unwrap_or_default();
    let addr = format!("{}:{}", account.imap_host, account.imap_port);

    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| {
            format!(
                "Failed to resolve IMAP server '{}:{}': {}",
                account.imap_host, account.imap_port, e
            )
        })?
        .next()
        .ok_or_else(|| {
            format!(
                "No socket address resolved for '{}:{}'",
                account.imap_host, account.imap_port
            )
        })?;

    let tcp_stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10)).map_err(|e| {
            format!(
                "IMAP connection to '{}:{}' failed: {}",
                account.imap_host, account.imap_port, e
            )
        })?;

    tcp_stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    let is_implicit = is_implicit_tls_imap(account.imap_port, &account.encryption_type);
    let mut stream = if is_implicit {
        EmailStream::Plain(tcp_stream).upgrade_to_tls(&account.imap_host)?
    } else {
        EmailStream::Plain(tcp_stream)
    };

    // Read greeting
    let _ = read_imap_response(&mut stream);

    let is_starttls = is_starttls_imap(account.imap_port, &account.encryption_type);
    if is_starttls {
        send_imap_cmd(&mut stream, "A00", "STARTTLS", false)?;
        let starttls_resp = read_imap_response(&mut stream)?;
        if starttls_resp.contains("OK") {
            stream = stream.upgrade_to_tls(&account.imap_host)?;
        }
    }

    // Login
    send_imap_cmd(
        &mut stream,
        "A01",
        &format!("LOGIN \"{}\" \"{}\"", account.email, password),
        true,
    )?;
    let login_resp = read_imap_response(&mut stream)?;
    if !login_resp.contains("OK") {
        return Err(format!("IMAP login failed: {}", login_resp));
    }

    // Select Inbox
    send_imap_cmd(&mut stream, "A02", "SELECT INBOX", false)?;
    let _ = read_imap_response(&mut stream);

    // Search Unseen
    send_imap_cmd(&mut stream, "A03", "SEARCH UNSEEN", false)?;
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
        send_imap_cmd(
            &mut stream,
            &tag,
            &format!(
                "FETCH {} (BODY[HEADER.FIELDS (SUBJECT FROM MESSAGE-ID)] BODY[TEXT])",
                id
            ),
            false,
        )?;
        let fetch_resp = read_imap_response(&mut stream)?;
        if let Some(msg) = parse_raw_fetch_response(&fetch_resp) {
            messages.push(msg);
        }
    }

    let _ = send_imap_cmd(&mut stream, "A99", "LOGOUT", false);
    Ok(messages)
}

fn send_imap_cmd(
    stream: &mut EmailStream,
    tag: &str,
    cmd: &str,
    is_sensitive: bool,
) -> Result<(), String> {
    let line = format!("{} {}\r\n", tag, cmd);
    stream.write_all(line.as_bytes()).map_err(|e| {
        if is_sensitive {
            format!("Failed to send IMAP command '{} <REDACTED>': {}", tag, e)
        } else {
            format!("Failed to send IMAP command '{} {}': {}", tag, cmd, e)
        }
    })
}

fn read_imap_response(stream: &mut EmailStream) -> Result<String, String> {
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

    let body = resp
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();

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
    fn test_parse_project_command_with_re_prefix() {
        let action = parse_email_command("Re: Project: my-awesome-app", "Please fix bug #12");
        match action {
            InboundAction::PromptInjection {
                project_name,
                prompt,
            } => {
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

    #[test]
    fn test_parse_named_prompt_command() {
        let action = parse_email_command("prompt: refactor auth", "");
        match action {
            InboundAction::NamedPromptExecution { prompt_query } => {
                assert_eq!(prompt_query, "refactor auth");
            }
            _ => panic!("Expected NamedPromptExecution"),
        }
    }
}
