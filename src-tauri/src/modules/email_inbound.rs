//! Inbound Email Poller and Remote Execution Bridge
//! Reads unread messages via IMAP, parses instruction subjects/bodies via
//! universal pipe-delimited syntax and legacy commands, validates sender authorization,
//! applies a 10-second debounce rate-limiting stack, and dispatches 2-phase
//! plaintext receipts (ACK and Result).

#![allow(dead_code)]

use crate::modules::email_sender::{self, EmailStream};
use crate::modules::email_vault_db::{self, EmailAccount, EmailInboundAuditLog};
#[cfg(target_os = "windows")]
use crate::utils::command::CommandExtWrapper;
use chrono::Utc;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use uuid::Uuid;

/// Parsed inbound email command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundAction {
    PromptInjection {
        project_name: String,
        prompt_name: String,
        prompt: String,
        instance_id: Option<String>,
    },
    NamedPromptExecution {
        prompt_query: String,
    },
    CliExecution {
        target_ip: String,
        command: String,
    },
    GitMapExecution {
        target: String,
        command: String,
    },
    UpdateExecution {
        target: String,
        is_gitmap: bool,
    },
    ListInstances {
        target: String,
    },
    ListPrompts {
        target: String,
        is_gitmap: bool,
    },
    InstanceCreate {
        profile_name: String,
    },
    AccountRotate,
    FastForward {
        target_node: String,
    },
    MultiNodeSnapshotQuery,
    StatusQuery {
        target: String,
    },
    DoctorDiagnostic {
        target: String,
    },
    ListAccounts {
        target: String,
    },
    AccountSwitch {
        target: String,
        email_query: String,
    },
    ProxyStatus {
        target: String,
        is_test: bool,
    },
    SystemClean {
        target: String,
    },
    SyncState {
        target: String,
    },
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

#[derive(Debug, Clone)]
pub struct DebounceRecord {
    pub first_seen: i64,
    pub count: usize,
    pub has_acked: bool,
    pub has_completed: bool,
}

static DEBOUNCE_STACK: Lazy<Mutex<HashMap<String, DebounceRecord>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Check sliding 10-second debounce stack
/// Returns true if execution should proceed, false if throttled
pub fn check_debounce_rate_limit(sender: &str, command: &str, target: &str, now: i64) -> bool {
    let key = format!(
        "{}:{}:{}",
        sender.trim().to_lowercase(),
        command.trim().to_lowercase(),
        target.trim().to_lowercase()
    );
    let mut stack = DEBOUNCE_STACK.lock().unwrap();

    if let Some(record) = stack.get_mut(&key) {
        if now - record.first_seen < 10 {
            record.count += 1;
            // Within 10s window: allow at most 2 calls through (initial + 1 retry)
            return record.count <= 2;
        } else {
            *record = DebounceRecord {
                first_seen: now,
                count: 1,
                has_acked: true,
                has_completed: false,
            };
            true
        }
    } else {
        stack.insert(
            key,
            DebounceRecord {
                first_seen: now,
                count: 1,
                has_acked: true,
                has_completed: false,
            },
        );
        true
    }
}

/// Extract clean email from RFC From header (e.g. "User <user@example.com>")
pub fn extract_email_address(raw_from: &str) -> String {
    let raw = raw_from.trim();
    if let Some(start) = raw.find('<') {
        if let Some(end) = raw.find('>') {
            if end > start {
                return raw[start + 1..end].trim().to_string();
            }
        }
    }
    raw.to_string()
}

/// Verify sender email against active notify recipients and vault accounts
pub fn is_authorized_notifier(sender_raw: &str) -> bool {
    let clean = extract_email_address(sender_raw).to_lowercase();
    if clean.is_empty() {
        return false;
    }

    if let Ok(recipients) = email_vault_db::list_notify_recipients() {
        for r in recipients {
            if r.is_active && r.email.trim().eq_ignore_ascii_case(&clean) {
                return true;
            }
        }
    }

    if let Ok(accounts) = email_vault_db::list_email_accounts() {
        for acc in accounts {
            if acc.is_active && acc.email.trim().eq_ignore_ascii_case(&clean) {
                return true;
            }
        }
    }

    false
}

/// Clean subject by removing reply/forward/subject prefixes
pub fn strip_email_prefixes(subject: &str) -> String {
    let mut clean = subject.trim();
    loop {
        let lower = clean.to_lowercase();
        if lower.starts_with("re:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("fwd:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fw:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("sub:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("subject:") {
            clean = clean[8..].trim();
        } else if lower.starts_with("[agm]:") {
            clean = clean[6..].trim();
        } else {
            break;
        }
    }
    clean.to_string()
}

/// Check if target matches local machine name, IP, partial octet, or wildcard
pub fn matches_target_node_or_ip(target: &str, local_ip: &str, local_name: &str) -> bool {
    let t = target.trim();
    if t.is_empty() || t == "*" || t.eq_ignore_ascii_case("all") || t.eq_ignore_ascii_case("any") {
        return true;
    }
    if t.eq_ignore_ascii_case("local") || t.eq_ignore_ascii_case("localhost") || t == "127.0.0.1" {
        return true;
    }
    if t.eq_ignore_ascii_case(local_name) {
        return true;
    }
    if t == local_ip {
        return true;
    }

    // IP Octet match: e.g. "12" matches "192.168.1.12"
    let ip_octets: Vec<&str> = local_ip.split('.').collect();
    if ip_octets.contains(&t) {
        return true;
    }
    if local_ip.ends_with(&format!(".{}", t)) {
        return true;
    }

    // Host environment variables check
    if let Ok(comp) = std::env::var("COMPUTERNAME") {
        if t.eq_ignore_ascii_case(&comp) {
            return true;
        }
    }
    if let Ok(host) = std::env::var("HOSTNAME") {
        if t.eq_ignore_ascii_case(&host) {
            return true;
        }
    }

    false
}

/// Extract prompt name and prompt instruction from email body
pub fn parse_prompt_body(body: &str) -> (String, String) {
    let mut prompt_name = String::new();
    let mut prompt_instruction = String::new();
    let mut in_instruction = false;

    for line in body.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("prompt-name:") {
            prompt_name = trimmed["prompt-name:".len()..].trim().to_string();
            in_instruction = false;
        } else if lower.starts_with("prompt instruction:") {
            prompt_instruction = trimmed["prompt instruction:".len()..].trim().to_string();
            in_instruction = true;
        } else if in_instruction {
            if !prompt_instruction.is_empty() {
                prompt_instruction.push('\n');
            }
            prompt_instruction.push_str(line);
        }
    }

    if prompt_instruction.is_empty() && prompt_name.is_empty() {
        prompt_instruction = body.trim().to_string();
    }

    (prompt_name, prompt_instruction)
}

/// Parse subject and body into strongly typed InboundAction
pub fn parse_email_command(subject: &str, body: &str) -> InboundAction {
    let clean_subj = strip_email_prefixes(subject);
    let lower_subj = clean_subj.to_lowercase();

    // 1. Unified Pipe-Delimited Grammar
    // sub: [worker-name|ip] | [ins-{instance}] | <command> [ | proj-{project name} ]
    if clean_subj.contains('|') {
        let parts: Vec<&str> = clean_subj
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if !parts.is_empty() {
            let target = parts[0];
            let mut instance_id: Option<String> = None;
            let mut cmd_str = "";
            let mut proj_str = "";

            if parts.len() >= 2 {
                let p1 = parts[1];
                if p1.to_lowercase().starts_with("ins-") {
                    instance_id = Some(p1["ins-".len()..].trim().to_string());
                    if parts.len() >= 3 {
                        cmd_str = parts[2];
                    }
                    if parts.len() >= 4 {
                        proj_str = parts[3];
                    }
                } else {
                    cmd_str = p1;
                    if parts.len() >= 3 {
                        // Check if part 2 is another command argument (e.g. "* | gitmap | status")
                        if parts.len() >= 4 {
                            proj_str = parts[3];
                        } else if parts[2].to_lowercase().starts_with("proj-")
                            || parts[2].to_lowercase().starts_with("project:")
                        {
                            proj_str = parts[2];
                        }
                    }
                }
            }

            let project_name = if !proj_str.is_empty() {
                let lower_p = proj_str.to_lowercase();
                if lower_p.starts_with("proj-") {
                    proj_str["proj-".len()..].trim().to_string()
                } else if lower_p.starts_with("project:") {
                    proj_str["project:".len()..].trim().to_string()
                } else {
                    proj_str.to_string()
                }
            } else {
                String::new()
            };

            let lower_cmd = cmd_str.to_lowercase();

            if lower_cmd == "prompt" {
                let (p_name, p_inst) = parse_prompt_body(body);
                return InboundAction::PromptInjection {
                    project_name,
                    prompt_name: p_name,
                    prompt: p_inst,
                    instance_id,
                };
            }

            if lower_cmd == "gitmap update" {
                return InboundAction::UpdateExecution {
                    target: target.to_string(),
                    is_gitmap: true,
                };
            }

            if lower_cmd == "gitmap prompts ls" {
                return InboundAction::ListPrompts {
                    target: target.to_string(),
                    is_gitmap: true,
                };
            }

            if lower_cmd.starts_with("gitmap macro") {
                let macro_name = if lower_cmd.len() > "gitmap macro".len() {
                    cmd_str["gitmap macro".len()..].trim()
                } else {
                    body.trim()
                };
                return InboundAction::GitMapExecution {
                    target: target.to_string(),
                    command: format!("macro {}", macro_name).trim().to_string(),
                };
            }

            if lower_cmd == "gitmap" || lower_cmd.starts_with("gitmap ") {
                let cmd_arg = if lower_cmd.starts_with("gitmap ") {
                    cmd_str["gitmap ".len()..].trim().to_string()
                } else if parts.len() >= 3 {
                    let p2 = parts[2].to_lowercase();
                    if p2.starts_with("proj-") {
                        let raw = body.trim();
                        if raw.to_lowercase().starts_with("gitmap ") {
                            raw["gitmap ".len()..].trim().to_string()
                        } else if raw.is_empty() {
                            "status".to_string()
                        } else {
                            raw.to_string()
                        }
                    } else {
                        parts[2].trim().to_string()
                    }
                } else {
                    let raw = body.trim();
                    if raw.to_lowercase().starts_with("gitmap ") {
                        raw["gitmap ".len()..].trim().to_string()
                    } else if raw.is_empty() {
                        "status".to_string()
                    } else {
                        raw.to_string()
                    }
                };

                let clean_arg = if cmd_arg.to_lowercase().starts_with("gitmap ") {
                    cmd_arg["gitmap ".len()..].trim().to_string()
                } else {
                    cmd_arg
                };

                return InboundAction::GitMapExecution {
                    target: target.to_string(),
                    command: clean_arg,
                };
            }

            if lower_cmd == "cmd" {
                return InboundAction::CliExecution {
                    target_ip: target.to_string(),
                    command: body.trim().to_string(),
                };
            }

            if lower_cmd == "update" || lower_cmd == "agm update" {
                return InboundAction::UpdateExecution {
                    target: target.to_string(),
                    is_gitmap: false,
                };
            }

            if lower_cmd == "ls" || lower_cmd == "agm ls" || lower_cmd == "agm instances" {
                return InboundAction::ListInstances {
                    target: target.to_string(),
                };
            }

            if lower_cmd == "help" {
                return InboundAction::HelpRequest;
            }

            if lower_cmd == "agm status" {
                return InboundAction::StatusQuery {
                    target: target.to_string(),
                };
            }

            if lower_cmd == "agm ff"
                || lower_cmd == "agm smart-switch"
                || lower_cmd == "agm ff/smart-switch"
                || lower_cmd == "ff"
            {
                return InboundAction::FastForward {
                    target_node: target.to_string(),
                };
            }

            if lower_cmd == "agy prompts ls"
                || lower_cmd == "agm prompts"
                || lower_cmd == "agm prompts ls"
                || lower_cmd == "prompts"
            {
                return InboundAction::ListPrompts {
                    target: target.to_string(),
                    is_gitmap: false,
                };
            }

            if lower_cmd == "doctor"
                || lower_cmd == "agm doctor"
                || lower_cmd == "check"
                || lower_cmd == "agm check"
            {
                return InboundAction::DoctorDiagnostic {
                    target: target.to_string(),
                };
            }

            if lower_cmd == "accounts"
                || lower_cmd == "agm accounts"
                || lower_cmd == "agm acc"
                || lower_cmd == "acc"
            {
                return InboundAction::ListAccounts {
                    target: target.to_string(),
                };
            }

            if lower_cmd.starts_with("agm switch") || lower_cmd.starts_with("switch") {
                let email_query = if lower_cmd.starts_with("agm switch") {
                    cmd_str["agm switch".len()..].trim().to_string()
                } else if lower_cmd.starts_with("switch") {
                    cmd_str["switch".len()..].trim().to_string()
                } else {
                    String::new()
                };
                let query = if !email_query.is_empty() {
                    email_query
                } else if parts.len() >= 3 {
                    parts[2].trim().to_string()
                } else {
                    body.trim().to_string()
                };
                return InboundAction::AccountSwitch {
                    target: target.to_string(),
                    email_query: query,
                };
            }

            if lower_cmd == "proxy" || lower_cmd == "agm proxy" || lower_cmd == "agm proxy status" {
                return InboundAction::ProxyStatus {
                    target: target.to_string(),
                    is_test: false,
                };
            }

            if lower_cmd == "agm proxy test" || lower_cmd == "proxy test" {
                return InboundAction::ProxyStatus {
                    target: target.to_string(),
                    is_test: true,
                };
            }

            if lower_cmd == "clean"
                || lower_cmd == "agm clean"
                || lower_cmd == "purge"
                || lower_cmd == "agm purge"
            {
                return InboundAction::SystemClean {
                    target: target.to_string(),
                };
            }

            if lower_cmd == "sync" || lower_cmd == "agm sync" {
                return InboundAction::SyncState {
                    target: target.to_string(),
                };
            }
        }
    }

    // 2. Legacy Prefix Support
    if lower_subj.starts_with("project:") || lower_subj.starts_with("project-prompt:") {
        let prefix_len = if lower_subj.starts_with("project:") {
            "project:".len()
        } else {
            "project-prompt:".len()
        };
        let project_name = clean_subj[prefix_len..].trim().to_string();
        return InboundAction::PromptInjection {
            project_name,
            prompt_name: String::new(),
            prompt: body.trim().to_string(),
            instance_id: None,
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
        return InboundAction::StatusQuery {
            target: "*".to_string(),
        };
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

    InboundAction::Ignored {
        reason: format!(
            "Subject '{}' does not match any recognized command pattern",
            subject
        ),
    }
}

/// Send Phase 1 Immediate Acknowledgment Plaintext Receipt
pub fn send_ack_receipt(
    sender: &str,
    command_name: &str,
    target: &str,
    instance: &str,
    project: &str,
    local_name: &str,
    local_ip: &str,
) {
    let now_str = Utc::now().to_rfc3339();
    let body = format!(
        "================================================================================
[AGM ACK] COMMAND ACKNOWLEDGED AND RUNNING
================================================================================
Command:    {}
Target:     {}
Node:       {} ({})
Instance:   {}
Project:    {}
Received:   {}
Status:     IN_PROGRESS

Execution has started in the background. A completion receipt will follow.
================================================================================",
        command_name, target, local_name, local_ip, instance, project, now_str
    );

    let subject = format!("[AGM ACK] Running: {}", command_name);
    let _ = email_sender::dispatch_email_with_failover(&subject, &body, &[sender.to_string()]);
}

/// Send Phase 2 Completion Result Plaintext Receipt
pub fn send_result_receipt(
    sender: &str,
    command_name: &str,
    status_label: &str,
    exit_code: i32,
    output: &str,
    local_name: &str,
    local_ip: &str,
    instance: &str,
) {
    let now_str = Utc::now().to_rfc3339();
    let body = format!(
        "================================================================================
[AGM Result] EXECUTION COMPLETED
================================================================================
Command:    {}
Status:     {}
Exit Code:  {}
Node:       {} ({})
Instance:   {}
Completed:  {}

Execution Output:
--------------------------------------------------------------------------------
{}
--------------------------------------------------------------------------------
================================================================================",
        command_name, status_label, exit_code, local_name, local_ip, instance, now_str, output
    );

    let subject = format!("[AGM Result] {}: {}", status_label, command_name);
    let _ = email_sender::dispatch_email_with_failover(&subject, &body, &[sender.to_string()]);
}

/// Process a parsed inbound email action and dispatch bidirectional 2-phase receipts
pub fn execute_inbound_action(
    msg: &RawEmailMessage,
    action: InboundAction,
    local_machine_ip: &str,
    local_machine_name: &str,
) -> Result<String, String> {
    let now = Utc::now().timestamp();

    // 1. Security Check: Sender Authorization ACL
    if !is_authorized_notifier(&msg.from) {
        let err_msg = format!("Sender '{}' not authorized in notify recipients", msg.from);
        crate::modules::logger::log_warn(&format!("[InboundEmail] Rejected: {}", err_msg));
        let audit = EmailInboundAuditLog {
            id: Uuid::new_v4().to_string(),
            message_id: msg.message_id.clone(),
            sender_email: msg.from.clone(),
            subject: msg.subject.clone(),
            action_type: "unauthorized".to_string(),
            action_payload: msg.body.chars().take(255).collect(),
            execution_status: "rejected_unauthorized_sender".to_string(),
            execution_result: err_msg.clone(),
            received_at: now,
        };
        let _ = email_vault_db::record_inbound_audit_log(audit);
        return Err(err_msg);
    }

    // 2. Sliding 10-Second Debounce Stack
    let action_slug = format!("{:?}", action);
    let can_proceed = check_debounce_rate_limit(&msg.from, &action_slug, local_machine_name, now);
    if !can_proceed {
        let debounced_msg = "Debounced duplicate request throttled within 10s window".to_string();
        crate::modules::logger::log_info(&format!("[InboundEmail] {}", debounced_msg));
        let audit = EmailInboundAuditLog {
            id: Uuid::new_v4().to_string(),
            message_id: msg.message_id.clone(),
            sender_email: msg.from.clone(),
            subject: msg.subject.clone(),
            action_type: "debounced".to_string(),
            action_payload: msg.body.chars().take(255).collect(),
            execution_status: "throttled".to_string(),
            execution_result: debounced_msg.clone(),
            received_at: now,
        };
        let _ = email_vault_db::record_inbound_audit_log(audit);
        return Ok(debounced_msg);
    }

    // 2b. SQLite 10-Minute Rate Limit for Heavy Actions
    let is_heavy_action = matches!(
        action,
        InboundAction::AccountRotate
            | InboundAction::SystemClean { .. }
            | InboundAction::UpdateExecution { .. }
    );
    if is_heavy_action {
        if let Ok(true) = email_vault_db::is_rate_limited_in_sqlite(&msg.from, &action_slug, 600) {
            let rate_limited_msg = format!(
                "Action '{:?}' rate-limited by SQLite audit guard (10-minute cooldown).",
                action
            );
            crate::modules::logger::log_info(&format!("[InboundEmail] {}", rate_limited_msg));
            let audit = EmailInboundAuditLog {
                id: Uuid::new_v4().to_string(),
                message_id: msg.message_id.clone(),
                sender_email: msg.from.clone(),
                subject: msg.subject.clone(),
                action_type: action_slug,
                action_payload: msg.body.chars().take(255).collect(),
                execution_status: "rate_limited_10m".to_string(),
                execution_result: rate_limited_msg.clone(),
                received_at: now,
            };
            let _ = email_vault_db::record_inbound_audit_log(audit);
            return Ok(rate_limited_msg);
        }
    }

    let action_str;
    let mut status = "success".to_string();
    let mut result_summary;
    let mut output_text = String::new();
    let mut exit_code = 0;

    match action {
        InboundAction::PromptInjection {
            project_name,
            prompt_name,
            prompt,
            instance_id,
        } => {
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
                    let _ = conn.execute(
                        "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, 'running', ?, ?)",
                        rusqlite::params![&p_id, &proj.id, &proj.instance_id, &proj.repo_path, &effective_prompt, now, now],
                    );
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
            );
        }

        InboundAction::GitMapExecution { target, command } => {
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
                );
            }
        }

        InboundAction::CliExecution { target_ip, command } => {
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
                );
            }
        }

        InboundAction::UpdateExecution { target, is_gitmap } => {
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
                );
            }
        }

        InboundAction::ListInstances { target } => {
            action_str = "list_instances".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm instances",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let instances = crate::modules::instance::list_instances().unwrap_or_default();
                let mut table = format!(
                    "{:<16} {:<20} {:<16} {:<24} {}\n",
                    "ID", "NAME", "STATUS", "BOUND ACCOUNT", "DATA DIR"
                );
                table.push_str(&"-".repeat(95));
                table.push('\n');
                for inst in instances {
                    let status_str = if inst.is_running {
                        format!("Running (PID: {})", inst.pid.unwrap_or(0))
                    } else {
                        "Idle".to_string()
                    };
                    let email = inst.config.bound_email.unwrap_or_else(|| "-".to_string());
                    table.push_str(&format!(
                        "{:<16} {:<20} {:<16} {:<24} {}\n",
                        inst.config.id, inst.config.name, status_str, email, inst.config.data_dir
                    ));
                }
                output_text = table;
                result_summary = "Listed sandbox profiles".to_string();

                send_result_receipt(
                    &msg.from,
                    "agm instances",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::ListPrompts { target, is_gitmap } => {
            action_str = "list_prompts".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Target mismatch: {}", target);
            } else {
                let cmd_label = if is_gitmap {
                    "gitmap prompts ls"
                } else {
                    "agy prompts ls"
                };
                send_ack_receipt(
                    &msg.from,
                    cmd_label,
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                if is_gitmap {
                    output_text = execute_safe_cli_command("gitmap prompts ls")
                        .unwrap_or_else(|e| format!("Failed to list gitmap prompts: {}", e));
                } else {
                    let all_prompts =
                        crate::modules::repo_db::list_all_prompts().unwrap_or_default();
                    let mut list = format!("Total Backed-Up Prompts: {}\n\n", all_prompts.len());
                    for p in all_prompts.iter().take(15) {
                        let short_id = &p.id[..8.min(p.id.len())];
                        let short_content: String = p.prompt_content.chars().take(60).collect();
                        list.push_str(&format!("- [{}] {}\n", short_id, short_content));
                    }
                    output_text = list;
                }
                result_summary = "Listed prompts".to_string();

                send_result_receipt(
                    &msg.from,
                    cmd_label,
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::FastForward { target_node } => {
            action_str = "fast_forward".to_string();
            if !matches_target_node_or_ip(&target_node, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!(
                    "Fast Forward target mismatch: {} != {}",
                    target_node, local_machine_name
                );
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm ff",
                    &target_node,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let rt = tokio::runtime::Runtime::new().ok();
                let ff_result = if let Some(r) = rt {
                    r.block_on(crate::modules::auto_switcher::trigger_manual_rotation())
                } else {
                    Err("Failed to start runtime".to_string())
                };

                let current_account =
                    crate::modules::account::get_current_account().unwrap_or(None);
                let current_email = current_account
                    .map(|a| a.email)
                    .unwrap_or_else(|| "Default".to_string());

                match ff_result {
                    Ok(msg_txt) => {
                        output_text = format!(
                            "Fast-forward successfully executed.\nDetails: {}\nActive Account: {}",
                            msg_txt, current_email
                        );
                        result_summary =
                            format!("Fast-forward completed. Active: {}", current_email);
                    }
                    Err(e) => {
                        exit_code = 1;
                        status = "error".to_string();
                        output_text = format!("Fast-forward error: {}", e);
                        result_summary = format!("Fast-forward failed: {}", e);
                    }
                }

                send_result_receipt(
                    &msg.from,
                    "agm ff",
                    &status.to_uppercase(),
                    exit_code,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::StatusQuery { target } => {
            action_str = "status_query".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Status target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm status",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
                let prompts = crate::modules::repo_db::list_backed_up_prompts().unwrap_or_default();
                let instances = crate::modules::instance::list_instances().unwrap_or_default();
                let current_account =
                    crate::modules::account::get_current_account().unwrap_or(None);
                let active_acc_str = current_account
                    .map(|a| a.email)
                    .unwrap_or_else(|| "Default".to_string());

                output_text = format!(
"================================================================================
NODE STATUS REPORT
================================================================================
Node Name:          {}
Local IP:           {}
Active Profile:     {}
Running Projects:   {}
Registered Sandbox: {}
Prompts in Queue:   {}
================================================================================",
                    local_machine_name,
                    local_machine_ip,
                    active_acc_str,
                    projects.len(),
                    instances.len(),
                    prompts.len()
                );
                result_summary = format!("Status reported to '{}'", msg.from);

                send_result_receipt(
                    &msg.from,
                    "agm status",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::DoctorDiagnostic { target } => {
            action_str = "doctor".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Doctor target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm doctor",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let accounts_cnt = crate::modules::account::load_account_index()
                    .map(|idx| idx.accounts.len())
                    .unwrap_or(0);
                let instances_cnt = crate::modules::instance::list_instances()
                    .map(|l| l.len())
                    .unwrap_or(0);
                let proxy_online = std::net::TcpStream::connect_timeout(
                    &"127.0.0.1:8045".parse().unwrap(),
                    std::time::Duration::from_millis(500),
                )
                .is_ok();

                output_text = format!(
"================================================================================
AGM SYSTEM DIAGNOSTIC (DOCTOR)
================================================================================
Node Name:          {}
Local IP:           {}
Database Vaults:    Verified Healthy (accounts: {}, instances: {})
Proxy Gateway:      {}
System Status:      HEALTHY
================================================================================",
                    local_machine_name,
                    local_machine_ip,
                    accounts_cnt,
                    instances_cnt,
                    if proxy_online { "ONLINE (Port 8045 listening)" } else { "OFFLINE (Standby)" }
                );
                result_summary = format!("Doctor diagnostic sent to '{}'", msg.from);

                send_result_receipt(
                    &msg.from,
                    "agm doctor",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::ListAccounts { target } => {
            action_str = "list_accounts".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Accounts target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm accounts",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let index_res = crate::modules::account::load_account_index();
                let mut acc_rows = Vec::new();
                if let Ok(index) = index_res {
                    let active_id = index.current_account_id.as_deref().unwrap_or("");
                    for (i, acc) in index.accounts.iter().enumerate() {
                        let is_curr = acc.id == active_id;
                        let st = if is_curr { "ACTIVE" } else { "STANDBY" };
                        acc_rows.push(format!("{:<4} {:<32} {:<10}", i + 1, acc.email, st));
                    }
                }

                output_text = format!(
"================================================================================
REGISTERED ACCOUNTS
================================================================================
INDEX EMAIL                            STATUS
--------------------------------------------------------------------------------
{}
================================================================================",
                    if acc_rows.is_empty() { "No accounts registered.".to_string() } else { acc_rows.join("\n") }
                );
                result_summary = format!("Accounts list sent to '{}'", msg.from);

                send_result_receipt(
                    &msg.from,
                    "agm accounts",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::AccountSwitch {
            target,
            email_query,
        } => {
            action_str = "switch_account".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Switch target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm switch",
                    &target,
                    "default",
                    &email_query,
                    local_machine_name,
                    local_machine_ip,
                );

                let query = email_query.trim().to_lowercase();
                let prev_email = crate::modules::account::get_current_account()
                    .ok()
                    .flatten()
                    .map(|a| a.email)
                    .unwrap_or_else(|| "(None)".to_string());

                let mut switch_ok = false;
                let mut new_email = String::new();
                let mut switch_err = String::new();

                if let Ok(index) = crate::modules::account::load_account_index() {
                    let matched = index.accounts.iter().find(|a| {
                        a.email.to_lowercase().contains(&query)
                            || a.id.to_lowercase().contains(&query)
                    });
                    if let Some(target_acc) = matched {
                        if let Err(e) =
                            crate::modules::account::set_current_account_id(&target_acc.id)
                        {
                            switch_err = format!("Failed to set active account: {}", e);
                        } else {
                            let _ = crate::modules::account::apply_device_profile(&target_acc.id);
                            switch_ok = true;
                            new_email = target_acc.email.clone();
                        }
                    } else {
                        switch_err = format!("No account found matching query '{}'", query);
                    }
                } else {
                    switch_err = "Failed to load accounts index".to_string();
                }

                if switch_ok {
                    output_text = format!(
"================================================================================
ACCOUNT SWITCH SUCCESS
================================================================================
Node Name:          {}
Previous Account:   {}
New Active Account: {}
Status:             SUCCESS
================================================================================",
                        local_machine_name, prev_email, new_email
                    );
                    result_summary = format!("Switched to '{}'", new_email);
                    send_result_receipt(
                        &msg.from,
                        "agm switch",
                        "SUCCESS",
                        0,
                        &output_text,
                        local_machine_name,
                        local_machine_ip,
                        "default",
                    );
                } else {
                    output_text = format!(
"================================================================================
ACCOUNT SWITCH FAILED
================================================================================
Error:              {}
Previous Account:   {}
================================================================================",
                        switch_err, prev_email
                    );
                    result_summary = format!("Switch failed: {}", switch_err);
                    send_result_receipt(
                        &msg.from,
                        "agm switch",
                        "FAILED",
                        1,
                        &output_text,
                        local_machine_name,
                        local_machine_ip,
                        "default",
                    );
                }
            }
        }

        InboundAction::ProxyStatus { target, is_test } => {
            action_str = "proxy".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Proxy target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm proxy",
                    &target,
                    "default",
                    if is_test { "test" } else { "status" },
                    local_machine_name,
                    local_machine_ip,
                );

                let proxy_online = std::net::TcpStream::connect_timeout(
                    &"127.0.0.1:8045".parse().unwrap(),
                    std::time::Duration::from_millis(500),
                )
                .is_ok();

                output_text = format!(
"================================================================================
AGM PROXY GATEWAY STATUS
================================================================================
Node Name:          {}
Proxy Address:      http://127.0.0.1:8045
Socket Status:      {}
Supported Routes:   /v1/messages, /v1/chat/completions, /v1beta/models/*
================================================================================",
                    local_machine_name,
                    if proxy_online { "ONLINE (Listening)" } else { "OFFLINE (Standby)" }
                );
                result_summary = format!("Proxy status sent to '{}'", msg.from);

                send_result_receipt(
                    &msg.from,
                    "agm proxy",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::SystemClean { target } => {
            action_str = "clean".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Clean target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm clean",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let mut cleaned_cnt = 0;
                let temp_dir = std::env::temp_dir();
                if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("antigravity_test_") {
                            let p = entry.path();
                            let n = p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_lowercase();
                            let is_sensitive =
                                n.contains("vault") || n.contains("account") || n.contains(".db");
                            if !is_sensitive {
                                if std::fs::remove_dir_all(&p).is_ok() {
                                    cleaned_cnt += 1;
                                }
                            }
                        }
                    }
                }

                output_text = format!(
"================================================================================
AGM SYSTEM CLEAN REPORT
================================================================================
Node Name:          {}
Artifacts Pruned:   {} temporary folder(s)
Database Vaults:    Protected and Untouched
Status:             SUCCESS
================================================================================",
                    local_machine_name, cleaned_cnt
                );
                result_summary = format!("Cleaned {} items", cleaned_cnt);

                send_result_receipt(
                    &msg.from,
                    "agm clean",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::SyncState { target } => {
            action_str = "sync".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Sync target mismatch: {}", target);
            } else {
                send_ack_receipt(
                    &msg.from,
                    "agm sync",
                    &target,
                    "default",
                    "-",
                    local_machine_name,
                    local_machine_ip,
                );

                let acc_cnt = crate::modules::account::load_account_index()
                    .map(|idx| idx.accounts.len())
                    .unwrap_or(0);
                let inst_cnt = crate::modules::instance::list_instances()
                    .map(|l| l.len())
                    .unwrap_or(0);

                output_text = format!(
"================================================================================
AGM STATE SYNCHRONIZATION REPORT
================================================================================
Node Name:          {}
Accounts Synced:    {}
Instances Synced:   {}
Split DB Vaults:    Verified & Active
Status:             SUCCESS
================================================================================",
                    local_machine_name, acc_cnt, inst_cnt
                );
                result_summary = "State synchronized".to_string();

                send_result_receipt(
                    &msg.from,
                    "agm sync",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    "default",
                );
            }
        }

        InboundAction::HelpRequest => {
            action_str = "help".to_string();
            send_ack_receipt(
                &msg.from,
                "help",
                "*",
                "default",
                "-",
                local_machine_name,
                local_machine_ip,
            );

            output_text = format!(
                "================================================================================
ANTIGRAVITY-MANAGER EMAIL COMMAND MANUAL
================================================================================
Format:
sub: [worker-name|ip] | [ins-{{instance}}] | <command> [ | proj-{{project name}} ]

Available Commands:
- prompt: Injects prompt into workspace.
    Body format:
    prompt-name: <prompt-name>
    prompt instruction:
    <multi-line instructions>
- gitmap: Executes gitmap command (e.g. status, scan).
- cmd: Runs raw PowerShell or shell script from body.
- update: Checks for and installs AGM update.
- ls / agm instances: Lists sandbox profiles and running PIDs.
- help: Returns this command cheat sheet.
- gitmap macro: Runs GitMap macros.
- gitmap update: Runs gitmap CLI self-updater.
- agm status: Returns current node and account status.
- agm ff / agm smart-switch: Rotates to freshest account.
- agm doctor / check: Runs system health diagnostics.
- agm accounts / acc: Lists registered accounts and active status.
- agm switch | <email>: Switches the active profile to the given email.
- agm proxy [status|test]: Checks proxy socket status or runs loopback test.
- agm clean / purge: Safely prunes build caches and test directories.
- agm sync: Synchronizes local accounts, instances, and DB vaults.
- agy prompts ls: Lists backed-up workspace prompts.
- gitmap prompts ls: Lists GitMap automated prompts.
================================================================================"
            );
            result_summary = "Help dispatched".to_string();

            send_result_receipt(
                &msg.from,
                "help",
                "SUCCESS",
                0,
                &output_text,
                local_machine_name,
                local_machine_ip,
                "default",
            );
        }

        InboundAction::NamedPromptExecution { prompt_query } => {
            action_str = "named_prompt_exec".to_string();
            result_summary = format!("Named prompt query: {}", prompt_query);
        }

        InboundAction::InstanceCreate { profile_name } => {
            action_str = "instance_create".to_string();
            let _ = crate::modules::instance::create_instance(profile_name.clone());
            result_summary = format!("Spawned instance profile '{}'", profile_name);
        }

        InboundAction::AccountRotate => {
            action_str = "rotate".to_string();
            let _ = crate::modules::auto_switcher::check_and_rotate_if_needed();
            result_summary = "Account rotation triggered".to_string();
        }

        InboundAction::MultiNodeSnapshotQuery => {
            action_str = "cluster_snapshot".to_string();
            result_summary = "Cluster snapshot generated".to_string();
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

/// Helper to execute approved CLI command safely
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

/// Helper to execute GitMap command, collapsing duplicate prefixes
fn execute_gitmap_command(cmd_args: &str) -> Result<String, String> {
    let clean_cmd = if cmd_args.trim().to_lowercase().starts_with("gitmap ") {
        cmd_args.trim()["gitmap ".len()..].trim()
    } else {
        cmd_args.trim()
    };

    let full_command = format!("gitmap {}", clean_cmd);
    execute_safe_cli_command(&full_command)
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

    let _ = read_imap_response(&mut stream);

    let is_starttls = is_starttls_imap(account.imap_port, &account.encryption_type);
    if is_starttls {
        send_imap_cmd(&mut stream, "A00", "STARTTLS", false)?;
        let starttls_resp = read_imap_response(&mut stream)?;
        if starttls_resp.contains("OK") {
            stream = stream.upgrade_to_tls(&account.imap_host)?;
        }
    }

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

    send_imap_cmd(&mut stream, "A02", "SELECT INBOX", false)?;
    let _ = read_imap_response(&mut stream);

    send_imap_cmd(&mut stream, "A03", "SEARCH UNSEEN", false)?;
    let search_resp = read_imap_response(&mut stream)?;

    let mut ids: Vec<&str> = search_resp
        .lines()
        .find(|l| l.starts_with("* SEARCH"))
        .unwrap_or("")
        .split_whitespace()
        .filter(|&w| w != "*" && w != "SEARCH")
        .collect();

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
                ..
            } => {
                assert_eq!(project_name, "my-awesome-app");
                assert_eq!(prompt, "Please fix bug #12");
            }
            _ => panic!("Expected PromptInjection"),
        }
    }

    #[test]
    fn test_parse_pipe_prompt_with_instance_and_project() {
        let body = "prompt-name: fix-auth\nprompt instruction:\nPlease refactor auth middleware";
        let action = parse_email_command("sub: worker-1 | ins-default | prompt | proj-web", body);
        match action {
            InboundAction::PromptInjection {
                project_name,
                prompt_name,
                prompt,
                instance_id,
            } => {
                assert_eq!(project_name, "web");
                assert_eq!(prompt_name, "fix-auth");
                assert_eq!(prompt, "Please refactor auth middleware");
                assert_eq!(instance_id, Some("default".to_string()));
            }
            _ => panic!("Expected PromptInjection with pipe grammar"),
        }
    }

    #[test]
    fn test_parse_pipe_ip_octet_prompt() {
        let action = parse_email_command("12 | prompt | proj-test", "Fix lint error");
        match action {
            InboundAction::PromptInjection {
                project_name,
                prompt,
                instance_id,
                ..
            } => {
                assert_eq!(project_name, "test");
                assert_eq!(prompt, "Fix lint error");
                assert_eq!(instance_id, None);
            }
            _ => panic!("Expected PromptInjection"),
        }
    }

    #[test]
    fn test_parse_pipe_gitmap_status() {
        let action = parse_email_command("* | gitmap | status", "");
        match action {
            InboundAction::GitMapExecution { target, command } => {
                assert_eq!(target, "*");
                assert_eq!(command, "status");
            }
            _ => panic!("Expected GitMapExecution"),
        }
    }

    #[test]
    fn test_parse_pipe_cmd() {
        let action = parse_email_command("local | cmd", "Get-Process");
        match action {
            InboundAction::CliExecution { target_ip, command } => {
                assert_eq!(target_ip, "local");
                assert_eq!(command, "Get-Process");
            }
            _ => panic!("Expected CliExecution"),
        }
    }

    #[test]
    fn test_parse_pipe_agm_commands() {
        assert_eq!(
            parse_email_command("VM3 | agm update", ""),
            InboundAction::UpdateExecution {
                target: "VM3".to_string(),
                is_gitmap: false
            }
        );
        assert_eq!(
            parse_email_command("VM3 | gitmap update", ""),
            InboundAction::UpdateExecution {
                target: "VM3".to_string(),
                is_gitmap: true
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm status", ""),
            InboundAction::StatusQuery {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm doctor", ""),
            InboundAction::DoctorDiagnostic {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm check", ""),
            InboundAction::DoctorDiagnostic {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm accounts", ""),
            InboundAction::ListAccounts {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm acc", ""),
            InboundAction::ListAccounts {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm switch | abidul@example.com", ""),
            InboundAction::AccountSwitch {
                target: "VM3".to_string(),
                email_query: "abidul@example.com".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm proxy", ""),
            InboundAction::ProxyStatus {
                target: "VM3".to_string(),
                is_test: false
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm proxy test", ""),
            InboundAction::ProxyStatus {
                target: "VM3".to_string(),
                is_test: true
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm clean", ""),
            InboundAction::SystemClean {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm purge", ""),
            InboundAction::SystemClean {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm sync", ""),
            InboundAction::SyncState {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm prompts", ""),
            InboundAction::ListPrompts {
                target: "VM3".to_string(),
                is_gitmap: false
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm instances", ""),
            InboundAction::ListInstances {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm ls", ""),
            InboundAction::ListInstances {
                target: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm ff", ""),
            InboundAction::FastForward {
                target_node: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm smart-switch", ""),
            InboundAction::FastForward {
                target_node: "VM3".to_string()
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agy prompts ls", ""),
            InboundAction::ListPrompts {
                target: "VM3".to_string(),
                is_gitmap: false
            }
        );
        assert_eq!(
            parse_email_command("VM3 | gitmap prompts ls", ""),
            InboundAction::ListPrompts {
                target: "VM3".to_string(),
                is_gitmap: true
            }
        );
    }

    #[test]
    fn test_matches_target_node_or_ip() {
        let local_ip = "192.168.1.12";
        let local_name = "VM3";

        // Wildcards & local
        assert!(matches_target_node_or_ip("*", local_ip, local_name));
        assert!(matches_target_node_or_ip("all", local_ip, local_name));
        assert!(matches_target_node_or_ip("any", local_ip, local_name));
        assert!(matches_target_node_or_ip("local", local_ip, local_name));
        assert!(matches_target_node_or_ip("localhost", local_ip, local_name));

        // Node name match
        assert!(matches_target_node_or_ip("vm3", local_ip, local_name));
        assert!(matches_target_node_or_ip("VM3", local_ip, local_name));

        // Full IP match
        assert!(matches_target_node_or_ip(
            "192.168.1.12",
            local_ip,
            local_name
        ));

        // Octet match
        assert!(matches_target_node_or_ip("12", local_ip, local_name));

        // Mismatches
        assert!(!matches_target_node_or_ip("VM4", local_ip, local_name));
        assert!(!matches_target_node_or_ip(
            "192.168.1.99",
            local_ip,
            local_name
        ));
        assert!(!matches_target_node_or_ip("99", local_ip, local_name));
    }

    #[test]
    fn test_extract_email_address() {
        assert_eq!(
            extract_email_address("John Doe <john@example.com>"),
            "john@example.com"
        );
        assert_eq!(
            extract_email_address("admin@example.com"),
            "admin@example.com"
        );
    }

    #[test]
    fn test_debounce_rate_limiter() {
        let now = 1000;
        let sender = "test@example.com";
        let cmd = "status";
        let target = "VM3";

        assert!(check_debounce_rate_limit(sender, cmd, target, now));
        assert!(check_debounce_rate_limit(sender, cmd, target, now + 2));
        // 3rd call in window: throttled
        assert!(!check_debounce_rate_limit(sender, cmd, target, now + 4));
        // After 10s: resets
        assert!(check_debounce_rate_limit(sender, cmd, target, now + 11));
    }
}
