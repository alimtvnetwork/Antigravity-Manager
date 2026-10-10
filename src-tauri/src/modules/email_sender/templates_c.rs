use crate::modules::email_vault_db::{self, EmailAccount};
use crate::modules::*;
use base64::prelude::*;

use super::*;

/// Render HTML email for system updated notification
pub fn render_system_update_email(
    previous_version: &str,
    current_version: &str,
    details: Option<&str>,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] [System Update] Application Updated: v{} -> v{}",
        pkg_ver, machine_name, machine_ip, previous_version, current_version
    );
    let extra_notes = details.unwrap_or("System update successfully applied.");
    let content = format!(
        "[+] SYSTEM UPDATE COMPLETED\r\n\r\n\
         Antigravity Manager has been updated on this node:\r\n\r\n\
         Previous Version: v{}\r\n\
         Current Version:  v{}\r\n\
         Host Machine:     {} ({})\r\n\r\n\
         Status: {}\r\n\r\n\
         Documentation & Release Notes:\r\n\
         https://github.com/alimtvnetwork/Antigravity-Manager/releases",
        previous_version, current_version, machine_name, machine_ip, extra_notes
    );
    let body = wrap_plaintext_email(
        "System Update Notification",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for remote CLI execution results
pub fn render_exec_result_email(
    cmd: &str,
    exit_code: i32,
    output: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] [Execution Report] exit: {} - {}",
        pkg_ver, machine_name, machine_ip, exit_code, cmd
    );
    let status = if exit_code == 0 { "SUCCESS" } else { "FAILED" };

    let content = format!(
        "[{}] Command: {}\r\n\
         Exit Code: {}\r\n\r\n\
         -------------------------------- OUTPUT --------------------------------\r\n\
         {}\r\n\
         ------------------------------------------------------------------------",
        status, cmd, exit_code, output
    );
    let body = wrap_plaintext_email(
        "Command Execution Report",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for help cheat sheet
pub fn render_help_email(machine_name: &str, machine_ip: &str) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] Inbound Remote Mailbox Instructions Cheat Sheet",
        pkg_ver, machine_name, machine_ip
    );
    let content = format!(
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
   • agm tree [all]: Displays bracketed Project -> Conversation -> 200w Prompt Tree
     with dual Sequence IDs ([AGM:P001 | GM:#1], [AGM:C001 | GM:<cid>]).
   • agm prompt <C001|P001|GM:#1> [--instance <id>] [--node <node>] \"<Prompt>\":
     Injects prompt into exact conversation, project, instance, or remote machine.
   • agm instances assign <inst> <repo_paths...>:
     Binds multiple project workspaces to an isolated instance.
   • gitmap agy: Executes GitMap AGY commands (active, running-prompts ls|backup|restore,
     prompt -n <slug> -t <text>, prompt-project P001 -n <slug>, fpug, sug, rerun).
   • gitmap ssh: Executes GitMap SSH commands (nodes, exec \"<cmd>\" --node <n>, update agm).
   • gitmap: Executes GitMap autonomous CLI commands (e.g. status, scan, sync, macro).
     Email Subject: * | gitmap | <Arguments>
   • agm status: Returns node status, active account, and credits remaining.
   • agm ff / agm smart-switch: Rotates immediately to the freshest available account.
   • agm switch | <email> [--instance <id>]: Switches account profile (isolated per instance).
   • agm accounts / acc: Returns registered accounts and active status.
   • agm doctor / check: Runs system health diagnostics and reports anomalies.
   • agm instances / ls: Lists active sandbox profiles and running process IDs.
   • agm proxy [status|test]: Checks proxy socket status or runs loopback test.
   • agm clean / purge: Safely prunes build caches and test artifacts.
   • agm sync: Synchronizes local accounts, instances, and DB vaults.
   • agy prompts ls: Lists backed-up workspace prompts.
   • gitmap prompts ls: Lists GitMap automated prompts.
   • update: Checks for and applies latest Antigravity Manager / GitMap updates.

3. TWO-PHASE AUTOMATED RECEIPTS:
   • Phase 1 ACK: Immediate acknowledgement email with IN_PROGRESS badge.
   • Phase 2 RESULT: Final completion receipt with exit code and stdout/stderr logs.
================================================================================",
        machine_name, machine_ip, pkg_ver
    );
    let body = wrap_plaintext_email(
        "Remote Instructions Cheat Sheet",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for self-test mailbox verification
pub fn render_self_test_email(
    email: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] Mailbox Connection Verified - {}",
        pkg_ver, machine_name, machine_ip, email
    );
    let content = format!(
        "[PASS] CONNECTION VERIFIED\r\n\r\n\
         This is an automated self-test verification email from Antigravity Manager.\r\n\r\n\
         Your mailbox account '{}' successfully authenticated via SMTP, passed credentials\r\n\
         verification, and delivered this rich HTML verification message.\r\n\r\n\
         Remote commands, quota notifications, and Smart Rotator failover are active for this node.",
        email
    );
    let body = wrap_plaintext_email(
        "Mailbox Self-Test Verification",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for test ping command verification
pub fn render_test_ping_email(
    project_name: &str,
    machine_name: &str,
    machine_ip: &str,
    timestamp: i64,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] Test Command Ping: {}",
        pkg_ver, machine_name, machine_ip, project_name
    );
    let content = format!(
        "[*] COMMAND TEST PING\r\n\r\n\
         Test ping dispatched for '{}' (epoch: {}).\r\n\r\n\
         Reply to this message with a command to verify remote execution:\r\n\
         Subject: {} | 1 | help",
        project_name, timestamp, machine_name
    );
    let body = wrap_plaintext_email("Command Test Ping", &content, machine_name, machine_ip);
    (subject, body)
}
