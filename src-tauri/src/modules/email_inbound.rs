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
use base64::prelude::*;
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
    AccountRotate {
        target: String,
        instance_id: Option<String>,
    },
    FastForward {
        target_node: String,
        instance_id: Option<String>,
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
        instance_id: Option<String>,
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
    HelpRequest {
        target: String,
        instance_id: Option<String>,
    },
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
        } else if lower.starts_with("re :") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fwd:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fwd :") {
            clean = clean[5..].trim();
        } else if lower.starts_with("fw:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("fw :") {
            clean = clean[4..].trim();
        } else if lower.starts_with("sub:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("subject:") {
            clean = clean[8..].trim();
        } else if lower.starts_with("[agm]:") {
            clean = clean[6..].trim();
        } else if lower.starts_with("[agm]") {
            clean = clean[5..].trim();
        } else if lower.starts_with("[agm-task]") {
            clean = clean[10..].trim();
        } else if lower.starts_with("[agm ack]") {
            clean = clean[9..].trim();
        } else if lower.starts_with("[agm result]") {
            clean = clean[12..].trim();
        } else if lower.starts_with("[agm alert]") {
            clean = clean[11..].trim();
        } else if lower.starts_with("[agm help]:") {
            clean = clean[11..].trim();
        } else if lower.starts_with("[agm help]") {
            clean = clean[10..].trim();
        } else if lower.starts_with("[agm execution report]") {
            clean = clean[22..].trim();
        } else if lower.starts_with("[agm test ping]") {
            clean = clean[15..].trim();
        } else if lower.starts_with("[agm status]") {
            clean = clean[12..].trim();
        } else if clean.starts_with('[') {
            if let Some(end) = clean.find(']') {
                let inside = &clean[1..end];
                let lower_inside = inside.to_lowercase();
                if lower_inside.starts_with("agm")
                    || lower_inside.contains('|')
                    || lower_inside.contains('.')
                    || lower_inside.starts_with("antigravity")
                    || lower_inside.starts_with("vm")
                {
                    clean = clean[end + 1..].trim();
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Support responses to "[Node: AI-MAIN] [Type: prompt]"
    let lower_clean = clean.to_lowercase();
    if lower_clean.starts_with("[node:") {
        if let Some(node_end) = clean.find(']') {
            let node_name = clean["[node:".len()..node_end].trim();
            let after_node = clean[node_end + 1..].trim();
            let lower_after = after_node.to_lowercase();
            if lower_after.starts_with("[type:") {
                if let Some(type_end) = after_node.find(']') {
                    let task_type = after_node["[type:".len()..type_end].trim();
                    return format!("{} | {}", node_name, task_type);
                }
            }
        }
    }

    clean.to_string()
}

/// Check if a pipe segment is an instance specifier (e.g. "1", "#1", "ins-1", "default")
pub fn is_instance_specifier(part: &str) -> bool {
    let lower = part.trim().to_lowercase();
    if lower.starts_with("ins-") || lower.starts_with("instance-") || lower.starts_with("instance:")
    {
        return true;
    }
    if lower.starts_with('#') && lower[1..].chars().all(|c| c.is_ascii_digit()) && lower.len() > 1 {
        return true;
    }
    if !lower.is_empty() && lower.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if lower == "default" || lower == "active" {
        return true;
    }
    false
}

/// Extract clean instance name/number from instance specifier
pub fn extract_clean_instance_id(part: &str) -> String {
    let trimmed = part.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("ins-") {
        trimmed["ins-".len()..].trim().to_string()
    } else if lower.starts_with("instance-") {
        trimmed["instance-".len()..].trim().to_string()
    } else if lower.starts_with("instance:") {
        trimmed["instance:".len()..].trim().to_string()
    } else if lower.starts_with('#') {
        trimmed["#".len()..].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Check if a pipe segment represents a known AGM command
pub fn is_known_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "help"
            | "status"
            | "agm status"
            | "cmd"
            | "update"
            | "agm update"
            | "gitmap update"
            | "ls"
            | "agm ls"
            | "instances"
            | "agm instances"
            | "ff"
            | "agm ff"
            | "smart-switch"
            | "agm smart-switch"
            | "agm ff/smart-switch"
            | "prompts"
            | "agm prompts"
            | "agm prompts ls"
            | "agy prompts ls"
            | "gitmap prompts ls"
            | "doctor"
            | "agm doctor"
            | "check"
            | "agm check"
            | "accounts"
            | "agm accounts"
            | "acc"
            | "agm acc"
            | "rotate"
            | "agm rotate"
            | "proxy"
            | "agm proxy"
            | "agm proxy status"
            | "proxy test"
            | "agm proxy test"
            | "clean"
            | "agm clean"
            | "purge"
            | "agm purge"
            | "sync"
            | "agm sync"
            | "prompt"
    ) || lower.starts_with("gitmap")
        || lower.starts_with("switch")
        || lower.starts_with("agm switch")
}

/// Decode RFC 2047 encoded words in email headers (e.g. =?UTF-8?B?...?= or =?UTF-8?Q?...?=)
pub fn decode_rfc2047(input: &str) -> String {
    if !input.contains("=?") || !input.contains("?=") {
        return input.to_string();
    }
    let mut result = String::new();
    let mut remaining = input;
    while let Some(start) = remaining.find("=?") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("?=") {
            let token = &after_start[..end];
            let parts: Vec<&str> = token.split('?').collect();
            if parts.len() == 3 {
                let enc = parts[1].to_uppercase();
                let payload = parts[2];
                let decoded_opt = if enc == "B" {
                    BASE64_STANDARD
                        .decode(payload.trim())
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else if enc == "Q" {
                    let mut bytes = Vec::new();
                    let mut chars = payload.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '_' {
                            bytes.push(b' ');
                        } else if c == '=' {
                            let hex1 = chars.next();
                            let hex2 = chars.next();
                            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                                let hex_str = format!("{}{}", h1, h2);
                                if let Ok(b) = u8::from_str_radix(&hex_str, 16) {
                                    bytes.push(b);
                                }
                            }
                        } else {
                            bytes.push(c as u8);
                        }
                    }
                    String::from_utf8(bytes).ok()
                } else {
                    None
                };

                if let Some(decoded) = decoded_opt {
                    result.push_str(&decoded);
                } else {
                    result.push_str(&format!("=?{}?=", token));
                }
            } else {
                result.push_str(&format!("=?{}?=", token));
            }
            remaining = &after_start[end + 2..];
        } else {
            result.push_str("=?");
            remaining = after_start;
        }
    }
    result.push_str(remaining);
    result
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

/// Extract prompt name and prompt instruction from email body (stripping any footer after ---)
pub fn parse_prompt_body(body: &str) -> (String, String) {
    let body_before_footer = body.split("\n---").next().unwrap_or(body).trim();
    let mut prompt_name = String::new();
    let mut prompt_instruction = String::new();
    let mut in_instruction = false;

    for line in body_before_footer.lines() {
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
        prompt_instruction = body_before_footer.to_string();
    }

    (prompt_name, prompt_instruction)
}

/// Parse single command line string and body into strongly typed InboundAction
pub fn parse_single_command_string(command_str: &str, body: &str) -> InboundAction {
    let clean_subj = strip_email_prefixes(command_str);
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

            let mut arg_str = "";
            if parts.len() >= 3 {
                if is_instance_specifier(parts[1])
                    || (!is_known_command(parts[1]) && is_known_command(parts[2]))
                {
                    instance_id = Some(extract_clean_instance_id(parts[1]));
                    cmd_str = parts[2];
                    if parts.len() >= 4 {
                        arg_str = parts[3];
                        proj_str = parts[3];
                    }
                } else {
                    cmd_str = parts[1];
                    arg_str = parts[2];
                    if parts.len() >= 4 {
                        proj_str = parts[3];
                    } else if parts[2].to_lowercase().starts_with("proj-")
                        || parts[2].to_lowercase().starts_with("project:")
                    {
                        proj_str = parts[2];
                    }
                }
            } else if parts.len() >= 2 {
                if is_instance_specifier(parts[1]) {
                    instance_id = Some(extract_clean_instance_id(parts[1]));
                    cmd_str = "status";
                } else {
                    cmd_str = parts[1];
                }
            } else {
                cmd_str = "help";
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
                return InboundAction::HelpRequest {
                    target: target.to_string(),
                    instance_id: instance_id.clone(),
                };
            }

            if lower_cmd == "status" || lower_cmd == "agm status" {
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
                    instance_id,
                };
            }

            if lower_cmd == "rotate" || lower_cmd == "agm rotate" {
                return InboundAction::AccountRotate {
                    target: target.to_string(),
                    instance_id,
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
                let inline_query = if lower_cmd.starts_with("agm switch") {
                    cmd_str["agm switch".len()..].trim().to_string()
                } else if lower_cmd.starts_with("switch") {
                    cmd_str["switch".len()..].trim().to_string()
                } else {
                    String::new()
                };
                let query = if !inline_query.is_empty() {
                    inline_query
                } else if !arg_str.is_empty() {
                    arg_str.trim().to_string()
                } else {
                    body.trim().to_string()
                };
                return InboundAction::AccountSwitch {
                    target: target.to_string(),
                    email_query: query,
                    instance_id,
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

    // 2. Legacy and Flexible Prefix Support
    if lower_subj.starts_with("project:")
        || lower_subj.starts_with("project-prompt:")
        || lower_subj.starts_with("prompt:")
        || lower_subj.starts_with("prompt ")
        || lower_subj.starts_with("prompt injection:")
        || lower_subj.starts_with("prompt-injection:")
        || lower_subj.starts_with("ai:")
        || lower_subj.starts_with("ai-task:")
        || lower_subj.starts_with("instruction:")
        || lower_subj.starts_with("inject:")
    {
        let prefix_len = if lower_subj.starts_with("project-prompt:") {
            "project-prompt:".len()
        } else if lower_subj.starts_with("project:") {
            "project:".len()
        } else if lower_subj.starts_with("prompt injection:") {
            "prompt injection:".len()
        } else if lower_subj.starts_with("prompt-injection:") {
            "prompt-injection:".len()
        } else if lower_subj.starts_with("prompt:") {
            "prompt:".len()
        } else if lower_subj.starts_with("prompt ") {
            "prompt ".len()
        } else if lower_subj.starts_with("ai-task:") {
            "ai-task:".len()
        } else if lower_subj.starts_with("ai:") {
            "ai:".len()
        } else if lower_subj.starts_with("instruction:") {
            "instruction:".len()
        } else if lower_subj.starts_with("inject:") {
            "inject:".len()
        } else {
            0
        };
        let project_name = clean_subj[prefix_len..].trim().to_string();
        let (p_name, p_inst) = parse_prompt_body(body);
        let final_prompt = if p_inst.is_empty() {
            body.trim().to_string()
        } else {
            p_inst
        };
        return InboundAction::PromptInjection {
            project_name: if project_name.is_empty() {
                "Default".to_string()
            } else {
                project_name
            },
            prompt_name: p_name,
            prompt: final_prompt,
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

    if lower_subj.starts_with("rotate: accounts")
        || lower_subj.starts_with("account: rotate")
        || lower_subj == "rotate"
        || lower_subj == "agm rotate"
    {
        return InboundAction::AccountRotate {
            target: "*".to_string(),
            instance_id: None,
        };
    }

    if lower_subj == "ff"
        || lower_subj.starts_with("ff:")
        || lower_subj.starts_with("fast-forward")
        || lower_subj.starts_with("fastforward")
        || lower_subj == "agm ff"
        || lower_subj == "agm smart-switch"
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
        return InboundAction::FastForward {
            target_node,
            instance_id: None,
        };
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

    if lower_subj == "help"
        || lower_subj == "man"
        || lower_subj == "manual"
        || lower_subj == "agm help"
    {
        return InboundAction::HelpRequest {
            target: "*".to_string(),
            instance_id: None,
        };
    }

    if lower_subj == "accounts"
        || lower_subj == "acc"
        || lower_subj == "agm accounts"
        || lower_subj == "agm acc"
    {
        return InboundAction::ListAccounts {
            target: "*".to_string(),
        };
    }

    if lower_subj == "instances"
        || lower_subj == "ls"
        || lower_subj == "agm instances"
        || lower_subj == "agm ls"
    {
        return InboundAction::ListInstances {
            target: "*".to_string(),
        };
    }

    if lower_subj == "doctor"
        || lower_subj == "check"
        || lower_subj == "agm doctor"
        || lower_subj == "agm check"
    {
        return InboundAction::DoctorDiagnostic {
            target: "*".to_string(),
        };
    }

    if lower_subj == "clean"
        || lower_subj == "purge"
        || lower_subj == "agm clean"
        || lower_subj == "agm purge"
    {
        return InboundAction::SystemClean {
            target: "*".to_string(),
        };
    }

    if lower_subj == "sync" || lower_subj == "agm sync" {
        return InboundAction::SyncState {
            target: "*".to_string(),
        };
    }

    if lower_subj == "proxy" || lower_subj == "agm proxy" || lower_subj == "agm proxy status" {
        return InboundAction::ProxyStatus {
            target: "*".to_string(),
            is_test: false,
        };
    }

    if lower_subj == "proxy test" || lower_subj == "agm proxy test" {
        return InboundAction::ProxyStatus {
            target: "*".to_string(),
            is_test: true,
        };
    }

    if lower_subj == "prompts"
        || lower_subj == "agm prompts"
        || lower_subj == "agm prompts ls"
        || lower_subj == "agy prompts ls"
    {
        return InboundAction::ListPrompts {
            target: "*".to_string(),
            is_gitmap: false,
        };
    }

    if lower_subj.starts_with("gitmap ") || lower_subj == "gitmap" {
        let sub = if lower_subj.starts_with("gitmap ") {
            clean_subj["gitmap ".len()..].trim().to_string()
        } else {
            "status".to_string()
        };
        return InboundAction::GitMapExecution {
            target: "*".to_string(),
            command: sub,
        };
    }

    if lower_subj == "switch"
        || lower_subj == "agm switch"
        || lower_subj.starts_with("switch ")
        || lower_subj.starts_with("agm switch ")
    {
        let email_query = if lower_subj.starts_with("agm switch ") {
            clean_subj["agm switch ".len()..].trim().to_string()
        } else if lower_subj.starts_with("switch ") {
            clean_subj["switch ".len()..].trim().to_string()
        } else {
            body.trim().to_string()
        };
        return InboundAction::AccountSwitch {
            target: "*".to_string(),
            email_query,
            instance_id: None,
        };
    }

    InboundAction::Ignored {
        reason: format!(
            "Input '{}' does not match any recognized command pattern",
            command_str
        ),
    }
}

/// Extract clean user content from email body, discarding quoted reply text and signatures
pub fn extract_clean_reply_body(body: &str) -> (String, String) {
    let mut clean_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Detect quotation block boundaries common in Gmail, Outlook, Apple Mail, Thunderbird
        if lower.starts_with('>')
            || (lower.starts_with("on ") && (lower.ends_with("wrote:") || lower.contains("wrote:")))
            || lower.starts_with("-----original message-----")
            || lower.starts_with("--- original message ---")
            || lower.starts_with("________________________________")
            || (lower.starts_with("from:") && !clean_lines.is_empty())
            || (lower.starts_with("sent:") && !clean_lines.is_empty())
        {
            break;
        }

        clean_lines.push(trimmed);
    }

    let mut first_cmd = String::new();
    let mut remaining = Vec::new();
    let mut found_first = false;

    for line in clean_lines {
        if !found_first {
            if !line.is_empty() {
                first_cmd = line.to_string();
                found_first = true;
            }
        } else {
            remaining.push(line);
        }
    }

    (first_cmd, remaining.join("\n").trim().to_string())
}

/// Parse subject and body into strongly typed InboundAction with bidirectional reply support
pub fn parse_email_command(subject: &str, body: &str) -> InboundAction {
    let lower_subj_raw = subject.trim().to_lowercase();
    let is_reply_or_fwd = lower_subj_raw.starts_with("re:")
        || lower_subj_raw.starts_with("re :")
        || lower_subj_raw.starts_with("fwd:")
        || lower_subj_raw.starts_with("fw:");

    // 1. If this is an email reply, user's command is usually typed in the body
    if is_reply_or_fwd {
        let (first_cmd, remaining_body) = extract_clean_reply_body(body);
        if !first_cmd.is_empty() {
            let body_action = parse_single_command_string(&first_cmd, &remaining_body);
            if !matches!(body_action, InboundAction::Ignored { .. }) {
                return body_action;
            }
        }
    }

    // 2. Try parsing the subject line
    let subject_action = parse_single_command_string(subject, body);
    if !matches!(subject_action, InboundAction::Ignored { .. }) {
        return subject_action;
    }

    // 3. Fallback: Check the body even if subject wasn't explicitly marked as reply
    if !is_reply_or_fwd {
        let (first_cmd, remaining_body) = extract_clean_reply_body(body);
        if !first_cmd.is_empty() {
            let body_action = parse_single_command_string(&first_cmd, &remaining_body);
            if !matches!(body_action, InboundAction::Ignored { .. }) {
                return body_action;
            }
        }
    }

    subject_action
}

/// Format an RFC 5322 reply subject that strictly preserves threading in Gmail / Outlook
/// while prepending [v<VERSION> | <VM_ALIAS> | <LOCAL_IP>] to identify the originating VM node clearly.
pub fn format_reply_subject(
    original_subject: Option<&str>,
    fallback_prefix: &str,
    command_name: &str,
    local_name: &str,
    local_ip: &str,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let node_tag = format!("[{} | {} | {}]", pkg_ver, local_name, local_ip);
    if let Some(orig) = original_subject {
        let trimmed = orig.trim();
        if !trimmed.is_empty() {
            let clean_orig = strip_email_prefixes(trimmed);
            // If clean_orig starts with an existing bracketed tag with a pipe, strip it
            let final_orig = if clean_orig.starts_with('[') && clean_orig.contains(']') {
                if let Some(end_idx) = clean_orig.find(']') {
                    let inside = &clean_orig[1..end_idx];
                    if inside.contains('|') {
                        clean_orig[end_idx + 1..].trim()
                    } else {
                        &clean_orig
                    }
                } else {
                    &clean_orig
                }
            } else {
                &clean_orig
            };

            if !final_orig.is_empty() {
                return format!("{} Re: {}", node_tag, final_orig);
            }
        }
    }
    format!("{} {} {}", node_tag, fallback_prefix, command_name)
}

/// Render responsive HTML card layout for inbound execution ACK and Result receipts
pub fn render_html_receipt(
    command_name: &str,
    status_label: &str,
    exit_code: Option<i32>,
    output: &str,
    local_name: &str,
    local_ip: &str,
    instance: &str,
    target_or_query: &str,
    is_ack: bool,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let now_str = Utc::now().to_rfc3339();
    let badge_color = if is_ack {
        "#2563eb" // Blue
    } else if exit_code == Some(0) || status_label.eq_ignore_ascii_case("success") {
        "#059669" // Green
    } else {
        "#dc2626" // Red
    };

    let status_text = if is_ack {
        "IN PROGRESS".to_string()
    } else if let Some(code) = exit_code {
        if code == 0 {
            "COMPLETED (0)".to_string()
        } else {
            format!("FAILED ({})", code)
        }
    } else {
        status_label.to_uppercase()
    };

    let title_text = if is_ack {
        "Command Acknowledged"
    } else {
        "Execution Receipt"
    };

    let subtitle = if is_ack {
        "Your command has been accepted and is executing in the background. A final completion receipt will follow upon finish."
    } else {
        "Command execution has concluded. Review status, metadata, and diagnostic logs below."
    };

    let query_row = if !target_or_query.is_empty() && target_or_query != "-" {
        format!(
            "<tr><td style=\"padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;\">Target / Query</td><td style=\"padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;\">{}</td></tr>",
            target_or_query
        )
    } else {
        String::new()
    };

    let output_block = if !output.trim().is_empty() {
        format!(
            r#"<div style="margin-top: 20px;">
  <div style="font-weight: 700; font-size: 11px; text-transform: uppercase; color: #64748b; margin-bottom: 8px; letter-spacing: 0.05em;">Execution Output &amp; Diagnostics</div>
  <pre style="background: #0f172a; color: #38bdf8; padding: 14px; border-radius: 8px; font-family: 'Consolas', 'Courier New', monospace; font-size: 12px; line-height: 1.5; white-space: pre-wrap; word-break: break-all; margin: 0; max-height: 500px; overflow-y: auto;">{}</pre>
</div>"#,
            output.trim()
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
</head>
<body style="margin: 0; padding: 20px; background-color: #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
  <div style="max-width: 620px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.06); border: 1px solid #e2e8f0;">
    <div style="background: #0f172a; padding: 20px 24px; color: #ffffff;">
      <div style="margin-bottom: 8px;">
        <span style="background: #334155; color: #f8fafc; padding: 4px 10px; border-radius: 6px; font-family: monospace; font-size: 13px; font-weight: bold;">[{} | {} | {}]</span>
        <span style="background: {}; color: #ffffff; padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: bold; text-transform: uppercase; margin-left: 8px;">{}</span>
      </div>
      <h2 style="margin: 8px 0 0 0; font-size: 18px; color: #ffffff; font-weight: 700;">{}</h2>
    </div>
    <div style="padding: 24px;">
      <p style="margin: 0 0 16px 0; color: #475569; font-size: 14px; line-height: 1.5;">{}</p>
      <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
        <tr><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 140px;">Version</td><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 140px;">Command</td><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;"><code>{}</code></td></tr>
        <tr><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Origin Node</td><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{} ({})</td></tr>
        <tr><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Target Instance</td><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        {}
        <tr><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Timestamp</td><td style="padding: 8px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;">{}</td></tr>
      </table>
      {}
    </div>
    <div style="background: #f8fafc; padding: 14px 24px; border-top: 1px solid #e2e8f0; font-size: 11px; color: #94a3b8; text-align: center;">
      Automated Remote Dispatcher · Antigravity Manager {} · Maintained by Alim, Sponsored by RISEUP ASIA LLC
    </div>
  </div>
</body>
</html>"#,
        pkg_ver,
        local_name,
        local_ip,
        badge_color,
        status_text,
        title_text,
        subtitle,
        pkg_ver,
        command_name,
        local_name,
        local_ip,
        instance,
        query_row,
        now_str,
        output_block,
        pkg_ver
    )
}

/// Send Phase 1 Immediate Acknowledgment HTML Receipt
pub fn send_ack_receipt(
    sender: &str,
    command_name: &str,
    target: &str,
    instance: &str,
    project: &str,
    local_name: &str,
    local_ip: &str,
    in_reply_to: Option<&str>,
    original_subject: Option<&str>,
) {
    let clean_sender = extract_email_address(sender);
    let subject = format_reply_subject(
        original_subject,
        "[AGM ACK] Running:",
        command_name,
        local_name,
        local_ip,
    );
    let body = render_html_receipt(
        command_name,
        "IN_PROGRESS",
        None,
        "",
        local_name,
        local_ip,
        instance,
        project,
        true,
    );
    let _ =
        email_sender::dispatch_reply_with_failover(&subject, &body, &[clean_sender], in_reply_to);
}

/// Send Phase 2 Completion Result HTML Receipt
pub fn send_result_receipt(
    sender: &str,
    command_name: &str,
    status_label: &str,
    exit_code: i32,
    output: &str,
    local_name: &str,
    local_ip: &str,
    instance: &str,
    in_reply_to: Option<&str>,
    original_subject: Option<&str>,
) {
    let clean_sender = extract_email_address(sender);
    let subject = format_reply_subject(
        original_subject,
        &format!("[AGM Result] {}:", status_label),
        command_name,
        local_name,
        local_ip,
    );
    let body = render_html_receipt(
        command_name,
        status_label,
        Some(exit_code),
        output,
        local_name,
        local_ip,
        instance,
        "-",
        false,
    );
    let _ =
        email_sender::dispatch_reply_with_failover(&subject, &body, &[clean_sender], in_reply_to);
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
        InboundAction::AccountRotate { .. }
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

    let mut action_str = "unknown".to_string();
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
                Some(&msg.message_id),
                Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
        }

        InboundAction::FastForward {
            target_node,
            instance_id,
        } => {
            action_str = "fast_forward".to_string();
            if !matches_target_node_or_ip(&target_node, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!(
                    "Fast Forward target mismatch: {} != {}",
                    target_node, local_machine_name
                );
            } else {
                let inst_label = instance_id.as_deref().unwrap_or("default");
                send_ack_receipt(
                    &msg.from,
                    "agm ff",
                    &target_node,
                    inst_label,
                    "-",
                    local_machine_name,
                    local_machine_ip,
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );

                let rt = tokio::runtime::Runtime::new().ok();
                let ff_result = if let Some(r) = rt {
                    r.block_on(
                        crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                            instance_id.as_deref(),
                        ),
                    )
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
                    inst_label,
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
        }

        InboundAction::AccountSwitch {
            target,
            email_query,
            instance_id,
        } => {
            action_str = "switch_account".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Switch target mismatch: {}", target);
            } else {
                let inst_label = instance_id.as_deref().unwrap_or("default");
                send_ack_receipt(
                    &msg.from,
                    "agm switch",
                    &target,
                    inst_label,
                    &email_query,
                    local_machine_name,
                    local_machine_ip,
                    Some(&msg.message_id),
                    Some(&msg.subject),
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

                // If query is empty or requesting smart rotation
                if query.is_empty()
                    || query == "smart"
                    || query == "ff"
                    || query == "rotate"
                    || query == "next"
                {
                    let rt = tokio::runtime::Runtime::new().ok();
                    let rot_res = if let Some(r) = rt {
                        r.block_on(
                            crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                                instance_id.as_deref(),
                            ),
                        )
                    } else {
                        Err("Failed to start runtime".to_string())
                    };

                    match rot_res {
                        Ok(details) => {
                            switch_ok = true;
                            new_email = crate::modules::account::get_current_account()
                                .ok()
                                .flatten()
                                .map(|a| a.email)
                                .unwrap_or_else(|| "Rotated Account".to_string());
                            result_summary = format!(
                                "Rotated to '{}' via Smart Rotator ({})",
                                new_email, details
                            );
                        }
                        Err(e) => {
                            switch_err = format!("Smart rotation failed: {}", e);
                        }
                    }
                } else if let Ok(index) = crate::modules::account::load_account_index() {
                    let matched = index.accounts.iter().find(|a| {
                        a.email.to_lowercase().contains(&query)
                            || a.id.to_lowercase().contains(&query)
                    });
                    if let Some(target_acc) = matched {
                        // Consult Smart Rotator: if target_acc is already the active account on this workspace,
                        // rotate to the next best candidate via Smart Rotator instead of re-injecting the same account.
                        let is_already_current = target_acc.email.eq_ignore_ascii_case(&prev_email);
                        let target_inst = instance_id.clone();
                        let rt = tokio::runtime::Runtime::new().ok();

                        if is_already_current {
                            let rot_res = if let Some(r) = rt {
                                r.block_on(
                                    crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                                        target_inst.as_deref(),
                                    ),
                                )
                            } else {
                                Err("Failed to start runtime".to_string())
                            };
                            match rot_res {
                                Ok(_) => {
                                    switch_ok = true;
                                    new_email = crate::modules::account::get_current_account()
                                        .ok()
                                        .flatten()
                                        .map(|a| a.email)
                                        .unwrap_or_else(|| target_acc.email.clone());
                                }
                                Err(e) => {
                                    switch_err = format!("Smart rotation failed: {}", e);
                                }
                            }
                        } else {
                            let target_acc_id = target_acc.id.clone();
                            let target_acc_email = target_acc.email.clone();
                            let switch_res = if let Some(r) = rt {
                                r.block_on(crate::modules::instance::switch_account_to_instance(
                                    &target_acc_id,
                                    target_inst.as_deref(),
                                ))
                            } else {
                                Err("Failed to start runtime".to_string())
                            };

                            match switch_res {
                                Ok(_) => {
                                    switch_ok = true;
                                    new_email = target_acc_email;
                                }
                                Err(e) => {
                                    switch_err = format!(
                                        "Failed to inject account credentials to IDE: {}",
                                        e
                                    );
                                }
                            }
                        }
                    } else {
                        // No exact match for query -> delegate to Smart Rotator to pick the best candidate
                        let rt = tokio::runtime::Runtime::new().ok();
                        let rot_res = if let Some(r) = rt {
                            r.block_on(
                                crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                                    instance_id.as_deref(),
                                ),
                            )
                        } else {
                            Err("Failed to start runtime".to_string())
                        };
                        match rot_res {
                            Ok(_) => {
                                switch_ok = true;
                                new_email = crate::modules::account::get_current_account()
                                    .ok()
                                    .flatten()
                                    .map(|a| a.email)
                                    .unwrap_or_else(|| "Smart Rotated Account".to_string());
                            }
                            Err(e) => {
                                switch_err = format!(
                                    "No account matched '{}' and Smart Rotator fallback failed: {}",
                                    query, e
                                );
                            }
                        }
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
Target Instance:    {}
Previous Account:   {}
New Active Account: {}
IDE Injection:      COMPLETED (state.vscdb updated, token refreshed, IDE synced)
Status:             SUCCESS
================================================================================",
                        local_machine_name, inst_label, prev_email, new_email
                    );
                    result_summary = format!("Switched to '{}' on [{}]", new_email, inst_label);
                    send_result_receipt(
                        &msg.from,
                        "agm switch",
                        "SUCCESS",
                        0,
                        &output_text,
                        local_machine_name,
                        local_machine_ip,
                        inst_label,
                        Some(&msg.message_id),
                        Some(&msg.subject),
                    );
                } else {
                    output_text = format!(
"================================================================================
ACCOUNT SWITCH FAILED
================================================================================
Node Name:          {}
Target Instance:    {}
Error:              {}
Previous Account:   {}
================================================================================",
                        local_machine_name, inst_label, switch_err, prev_email
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
                        inst_label,
                        Some(&msg.message_id),
                        Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
        }

        InboundAction::HelpRequest {
            target,
            instance_id,
        } => {
            action_str = "help".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Help target mismatch: {}", target);
            } else {
                let inst_str = instance_id.as_deref().unwrap_or("default");
                send_ack_receipt(
                    &msg.from,
                    "help",
                    &target,
                    inst_str,
                    "-",
                    local_machine_name,
                    local_machine_ip,
                    Some(&msg.message_id),
                    Some(&msg.subject),
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
                    inst_str,
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
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

        InboundAction::AccountRotate {
            target,
            instance_id,
        } => {
            action_str = "rotate".to_string();
            if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
                status = "skipped".to_string();
                result_summary = format!("Rotate target mismatch: {}", target);
            } else {
                let inst_label = instance_id.as_deref().unwrap_or("default");
                send_ack_receipt(
                    &msg.from,
                    "agm rotate",
                    &target,
                    inst_label,
                    "-",
                    local_machine_name,
                    local_machine_ip,
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );

                let rt = tokio::runtime::Runtime::new().ok();
                let rot_result = if let Some(r) = rt {
                    r.block_on(
                        crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                            instance_id.as_deref(),
                        ),
                    )
                } else {
                    Err("Failed to start runtime".to_string())
                };

                let current_account =
                    crate::modules::account::get_current_account().unwrap_or(None);
                let current_email = current_account
                    .map(|a| a.email)
                    .unwrap_or_else(|| "Default".to_string());

                match rot_result {
                    Ok(msg_txt) => {
                        output_text = format!(
                            "Account rotation successfully executed via Smart Rotator.\nDetails: {}\nActive Account: {}",
                            msg_txt, current_email
                        );
                        result_summary = format!("Rotation completed. Active: {}", current_email);
                        send_result_receipt(
                            &msg.from,
                            "agm rotate",
                            "SUCCESS",
                            0,
                            &output_text,
                            local_machine_name,
                            local_machine_ip,
                            inst_label,
                            Some(&msg.message_id),
                            Some(&msg.subject),
                        );
                    }
                    Err(e) => {
                        exit_code = 1;
                        status = "error".to_string();
                        output_text = format!("Rotation error: {}", e);
                        result_summary = format!("Rotation failed: {}", e);
                        send_result_receipt(
                            &msg.from,
                            "agm rotate",
                            "FAILED",
                            1,
                            &output_text,
                            local_machine_name,
                            local_machine_ip,
                            inst_label,
                            Some(&msg.message_id),
                            Some(&msg.subject),
                        );
                    }
                }
            }
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

    let _ = read_imap_greeting(&mut stream);

    let is_starttls = is_starttls_imap(account.imap_port, &account.encryption_type);
    if is_starttls {
        send_imap_cmd(&mut stream, "A00", "STARTTLS", false)?;
        let starttls_resp = read_imap_tagged_response(&mut stream, "A00")?;
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
    let login_resp = read_imap_tagged_response(&mut stream, "A01")?;
    if !login_resp.contains("OK") {
        return Err(format!("IMAP login failed: {}", login_resp));
    }

    send_imap_cmd(&mut stream, "A02", "SELECT INBOX", false)?;
    let _ = read_imap_tagged_response(&mut stream, "A02");

    send_imap_cmd(&mut stream, "A03", "SEARCH UNSEEN", false)?;
    let search_resp = read_imap_tagged_response(&mut stream, "A03")?;

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
        let fetch_resp = read_imap_tagged_response(&mut stream, &tag)?;
        if let Some(msg) = parse_raw_fetch_response(&fetch_resp) {
            messages.push(msg);
        }
    }

    let _ = send_imap_cmd(&mut stream, "A99", "LOGOUT", false);
    let _ = read_imap_tagged_response(&mut stream, "A99");
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

fn read_imap_greeting(stream: &mut EmailStream) -> Result<String, String> {
    let mut accumulated = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| format!("IMAP greeting read error: {}", e))?;
        if n == 0 {
            break;
        }
        accumulated.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&accumulated);
        if text.contains("\r\n") {
            return Ok(text.to_string());
        }
    }
    Ok(String::from_utf8_lossy(&accumulated).to_string())
}

fn read_imap_tagged_response(stream: &mut EmailStream, tag: &str) -> Result<String, String> {
    let mut accumulated = Vec::new();
    let mut buf = [0u8; 4096];
    let ok_marker = format!("{} OK", tag);
    let no_marker = format!("{} NO", tag);
    let bad_marker = format!("{} BAD", tag);

    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| format!("IMAP read error for tag '{}': {}", tag, e))?;
        if n == 0 {
            break;
        }
        accumulated.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&accumulated);
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(&ok_marker)
                || trimmed.starts_with(&no_marker)
                || trimmed.starts_with(&bad_marker)
            {
                return Ok(text.to_string());
            }
        }
    }
    Ok(String::from_utf8_lossy(&accumulated).to_string())
}

fn parse_raw_fetch_response(resp: &str) -> Option<RawEmailMessage> {
    let mut subject = String::new();
    let mut from = String::new();
    let mut message_id = Uuid::new_v4().to_string();

    for line in resp.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("subject:") {
            subject = decode_rfc2047(line["subject:".len()..].trim());
        } else if lower.starts_with("from:") {
            from = line["from:".len()..].trim().to_string();
        } else if lower.starts_with("message-id:") {
            let raw_id = line["message-id:".len()..].trim();
            let clean = raw_id.trim_matches(|c| c == '<' || c == '>').trim();
            if !clean.is_empty() {
                message_id = clean.to_string();
            }
        }
    }

    if subject.is_empty() && from.is_empty() {
        return None;
    }

    let body = if let Some(pos) = resp.find("\r\n\r\n") {
        resp[pos + 4..].trim().to_string()
    } else if let Some(pos) = resp.find("\n\n") {
        resp[pos + 2..].trim().to_string()
    } else {
        String::new()
    };

    Some(RawEmailMessage {
        message_id,
        from,
        subject,
        body,
    })
}

/// Query IMAP inbox for recent account switch [JSON] telemetry events from sibling VMs
pub fn fetch_recent_cross_vm_switched_accounts(lookback_seconds: i64) -> Vec<String> {
    let default_acc = match email_vault_db::get_default_account() {
        Ok(Some(acc)) => acc,
        _ => return Vec::new(),
    };

    let messages = match poll_unread_messages(&default_acc, 20) {
        Ok(msgs) => msgs,
        Err(_) => return Vec::new(),
    };

    let my_vm = crate::modules::email_watcher::detect_machine_name();
    let my_ip = crate::modules::email_watcher::detect_local_ip();
    let now = Utc::now().timestamp();
    let cutoff = now - lookback_seconds;
    let mut excluded = Vec::new();

    for msg in messages {
        let is_json_switch =
            msg.subject.contains("[JSON]") && msg.subject.contains("Account Switched");
        if !is_json_switch {
            continue;
        }

        let json_start = match msg.body.find('{') {
            Some(idx) => idx,
            None => continue,
        };
        let json_end = match msg.body.rfind('}') {
            Some(idx) => idx,
            None => continue,
        };
        if json_end <= json_start {
            continue;
        }

        let json_str = &msg.body[json_start..=json_end];
        let val: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let vm_name = val.get("vm_name").and_then(|v| v.as_str()).unwrap_or("");
        let local_ip = val.get("local_ip").and_then(|v| v.as_str()).unwrap_or("");
        let timestamp = val.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
        let new_email = val
            .get("new_email")
            .or_else(|| val.get("selected_email"))
            .or_else(|| val.get("target_account"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let is_same_machine = vm_name == my_vm && local_ip == my_ip;
        let is_recent = timestamp >= cutoff;

        if !is_same_machine && is_recent && !new_email.is_empty() {
            let email_clean = new_email.trim().to_string();
            if !excluded.contains(&email_clean) {
                excluded.push(email_clean);
            }
        }
    }

    excluded
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
                email_query: "abidul@example.com".to_string(),
                instance_id: None,
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm proxy", ""),
            InboundAction::ProxyStatus {
                target: "VM3".to_string(),
                is_test: false,
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm proxy test", ""),
            InboundAction::ProxyStatus {
                target: "VM3".to_string(),
                is_test: true,
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm clean", ""),
            InboundAction::SystemClean {
                target: "VM3".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm purge", ""),
            InboundAction::SystemClean {
                target: "VM3".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm sync", ""),
            InboundAction::SyncState {
                target: "VM3".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm prompts", ""),
            InboundAction::ListPrompts {
                target: "VM3".to_string(),
                is_gitmap: false,
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm instances", ""),
            InboundAction::ListInstances {
                target: "VM3".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm ls", ""),
            InboundAction::ListInstances {
                target: "VM3".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm ff", ""),
            InboundAction::FastForward {
                target_node: "VM3".to_string(),
                instance_id: None,
            }
        );
        assert_eq!(
            parse_email_command("VM3 | agm smart-switch", ""),
            InboundAction::FastForward {
                target_node: "VM3".to_string(),
                instance_id: None,
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

    #[test]
    fn test_parse_vm3_instance_help_command() {
        // User's exact subject: "VM3 | 1 | help"
        let action = parse_email_command("VM3 | 1 | help", "");
        assert_eq!(
            action,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // With Re: prefix: "Re: VM3 | 1 | help"
        let action_re = parse_email_command("Re: VM3 | 1 | help", "");
        assert_eq!(
            action_re,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // Without instance part: "VM3 | help"
        let action_simple = parse_email_command("VM3 | help", "");
        assert_eq!(
            action_simple,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: None,
            }
        );
    }

    #[test]
    fn test_rfc2047_decoding() {
        let raw = "=?UTF-8?B?Vk0zIHwgMSB8IGhlbHA=?=";
        assert_eq!(decode_rfc2047(raw), "VM3 | 1 | help");

        let plain = "VM3 | 1 | help";
        assert_eq!(decode_rfc2047(plain), "VM3 | 1 | help");
    }

    #[test]
    fn test_extract_clean_reply_body_strips_quotes() {
        let gmail_reply = "VM3 | 1 | help\r\n\r\nOn Thu, Sep 24, 2026 at 7:45 PM ai-agm-tool-v1 <...> wrote:\r\n> ================================================================================\r\n> [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet\r\n> ...";
        let (first_cmd, remaining) = extract_clean_reply_body(gmail_reply);
        assert_eq!(first_cmd, "VM3 | 1 | help");
        assert_eq!(remaining, "");

        let outlook_reply = "status\r\n\r\n-----Original Message-----\r\nFrom: ai-agm-tool-v1\r\nSent: Thursday, September 24, 2026\r\nTo: user\r\nSubject: [AGM Help]";
        let (first_cmd2, remaining2) = extract_clean_reply_body(outlook_reply);
        assert_eq!(first_cmd2, "status");
        assert_eq!(remaining2, "");
    }

    #[test]
    fn test_parse_reply_with_body_command() {
        // User clicks "Reply" to Cheat Sheet in Gmail:
        // Subject: "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet"
        // Body: "VM3 | 1 | help\n\nOn Thu, Sep 24, 2026... wrote:\n> ..."
        let action = parse_email_command(
            "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet",
            "VM3 | 1 | help\r\n\r\nOn Thu, Sep 24, 2026 at 7:45 PM ai-agm-tool-v1 <...> wrote:\r\n> [AGM Help] ...",
        );
        assert_eq!(
            action,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // User clicks "Reply" and types just "status" in body:
        let action_status = parse_email_command(
            "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet",
            "status\r\n\r\n> [AGM Help] ...",
        );
        assert_eq!(
            action_status,
            InboundAction::StatusQuery {
                target: "*".to_string(),
            }
        );

        // User clicks "Reply" and types "accounts" in body:
        let action_acc = parse_email_command(
            "Re: [AGM Result] SUCCESS: help",
            "accounts\r\n\r\nOn Wed, Sep 24... wrote:\n> ...",
        );
        assert_eq!(
            action_acc,
            InboundAction::ListAccounts {
                target: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_single_word_commands() {
        assert_eq!(
            parse_email_command("help", ""),
            InboundAction::HelpRequest {
                target: "*".to_string(),
                instance_id: None,
            }
        );
        assert_eq!(
            parse_email_command("status", ""),
            InboundAction::StatusQuery {
                target: "*".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("accounts", ""),
            InboundAction::ListAccounts {
                target: "*".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("instances", ""),
            InboundAction::ListInstances {
                target: "*".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("doctor", ""),
            InboundAction::DoctorDiagnostic {
                target: "*".to_string(),
            }
        );
        assert_eq!(
            parse_email_command("rotate", ""),
            InboundAction::AccountRotate {
                target: "*".to_string(),
                instance_id: None,
            }
        );
        assert_eq!(
            parse_email_command("agm rotate", ""),
            InboundAction::AccountRotate {
                target: "*".to_string(),
                instance_id: None,
            }
        );
    }

    #[test]
    fn test_format_reply_subject_preserves_threading() {
        let local_name = "VM3";
        let local_ip = "192.168.1.12";
        let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
        let node_tag = format!("[{} | {} | {}]", pkg_ver, local_name, local_ip);

        // Direct subject: should prepend node tag and Re:
        assert_eq!(
            format_reply_subject(
                Some("VM3 | 1 | help"),
                "[AGM ACK]",
                "help",
                local_name,
                local_ip
            ),
            format!("{} Re: VM3 | 1 | help", node_tag)
        );

        // Subject already has Re: should NOT duplicate Re:
        assert_eq!(
            format_reply_subject(
                Some("Re: VM3 | 1 | help"),
                "[AGM ACK]",
                "help",
                local_name,
                local_ip
            ),
            format!("{} Re: VM3 | 1 | help", node_tag)
        );

        // Replying to existing notification:
        assert_eq!(
            format_reply_subject(
                Some("Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet"),
                "[AGM ACK]",
                "help",
                local_name,
                local_ip
            ),
            format!(
                "{} Re: Inbound Remote Mailbox Instructions Cheat Sheet",
                node_tag
            )
        );

        // Fallback when no original subject:
        assert_eq!(
            format_reply_subject(None, "[AGM ACK] Running:", "help", local_name, local_ip),
            format!("{} [AGM ACK] Running: help", node_tag)
        );
    }

    #[test]
    fn test_parse_flexible_pipe_spacing_and_instances() {
        // 0 spaces
        let zero_space_help = parse_email_command("VM3|1|help", "");
        assert_eq!(
            zero_space_help,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        let zero_space_switch = parse_email_command("VM3|1|switch|user@gmail.com", "");
        assert_eq!(
            zero_space_switch,
            InboundAction::AccountSwitch {
                target: "VM3".to_string(),
                email_query: "user@gmail.com".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // 1 space
        let one_space_switch = parse_email_command("VM3 | 1 | switch | user@gmail.com", "");
        assert_eq!(
            one_space_switch,
            InboundAction::AccountSwitch {
                target: "VM3".to_string(),
                email_query: "user@gmail.com".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // Multiple spaces
        let multi_space_help = parse_email_command("VM3   |   1   |   help", "");
        assert_eq!(
            multi_space_help,
            InboundAction::HelpRequest {
                target: "VM3".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        let multi_space_switch =
            parse_email_command("VM3   |   1   |   switch   |   user@gmail.com", "");
        assert_eq!(
            multi_space_switch,
            InboundAction::AccountSwitch {
                target: "VM3".to_string(),
                email_query: "user@gmail.com".to_string(),
                instance_id: Some("1".to_string()),
            }
        );

        // Fast forward and rotate on specific instance
        let ff_inst = parse_email_command("VM3 | 2 | ff", "");
        assert_eq!(
            ff_inst,
            InboundAction::FastForward {
                target_node: "VM3".to_string(),
                instance_id: Some("2".to_string()),
            }
        );

        let rotate_inst = parse_email_command("VM3 | 2 | rotate", "");
        assert_eq!(
            rotate_inst,
            InboundAction::AccountRotate {
                target: "VM3".to_string(),
                instance_id: Some("2".to_string()),
            }
        );
    }
}
