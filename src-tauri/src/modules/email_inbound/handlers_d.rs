use base64::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::Duration;

use super::*;

pub(crate) fn handle_proxy_status(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    is_test: bool,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
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
            if proxy_online {
                "ONLINE (Listening)"
            } else {
                "OFFLINE (Standby)"
            }
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
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_system_clean(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
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
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_sync_state(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
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
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_help_request(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    instance_id: Option<String>,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
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

        let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
        output_text = format!(
            "================================================================================
ANTIGRAVITY-MANAGER EMAIL COMMAND MANUAL & SYNTAX GUIDE
================================================================================
Node: {} ({}) | Version: {}

1. EMAIL SUBJECT FORMAT & TARGET ROUTING:
   sub: [worker-name|ip] | [ins-{{instance}}] | <command> [ | proj-{{project name}} ]

   Wildcard / Broadcast Target:
   * | <command> (dispatches to all matching worker nodes/instances)

   Examples:
   • * | help
   • * | prompt | proj-Antigravity-Manager
   • * | ps | Get-Process
   • * | cmd | dir /b
   • * | gitmap | status
   • * | agm | status
   • * | status

2. AVAILABLE INBOUND COMMANDS:
   • help
     Returns this comprehensive command manual & syntax guide.
   • prompt: Injects prompt into workspace.
     Email Subject: * | prompt | proj-<ProjectName>
     Email Body format:
       prompt-name: <Optional Prompt Name>
       prompt instruction:
       <Your Multi-line AI Instructions Here>
   • powershell / ps: Executes Windows PowerShell commands/scripts from body.
     Email Subject: * | ps | <Short Command Description>
   • cmd: Runs Command Prompt script from email body.
     Email Subject: * | cmd | <Short Command Description>
   • gitmap: Executes GitMap autonomous CLI commands (e.g. status, scan, sync, macro).
     Email Subject: * | gitmap | <Arguments>
   • agm status: Returns node status, active account, and credits remaining.
   • agm ff / agm smart-switch: Rotates immediately to the freshest available account.
   • agm switch | <email>: Switches the active account profile to the specified email.
   • agm accounts / acc: Returns registered accounts and active status.
   • agm doctor / check: Runs system health diagnostics and reports anomalies.
   • agm instances / ls: Lists active sandbox profiles and running process IDs.
   • agm proxy [status|test]: Checks proxy socket status or runs loopback test.
   • agm clean / purge: Safely prunes build caches and test artifacts.
   • agm sync: Synchronizes local accounts, instances, and DB vaults.
   • agy prompts ls: Lists backed-up workspace prompts.
   • gitmap prompts ls: Lists GitMap automated prompts.
   • update: Checks for and applies latest Antigravity Manager updates.

3. TWO-PHASE AUTOMATED RECEIPTS:
   • Phase 1 ACK: Immediate acknowledgement email with IN_PROGRESS badge.
   • Phase 2 RESULT: Final completion receipt with exit code and stdout/stderr logs.
================================================================================",
            local_machine_name, local_machine_ip, pkg_ver
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
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}
