use base64::prelude::*;
use chrono::Utc;
use std::process::Command;

use super::*;
use crate::modules::email_sender;

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
            "<tr><td style=\"padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 170px; background: #f8fafc;\">Target / Query</td><td style=\"padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-size: 15px;\">{}</td></tr>",
            target_or_query
        )
    } else {
        String::new()
    };

    let output_block = if !output.trim().is_empty() {
        format!(
            r#"<div style="margin-top: 24px;">
  <div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin-bottom: 10px; letter-spacing: 0.08em;">Execution Output &amp; Diagnostics</div>
  <pre style="background: #0f172a; color: #38bdf8; padding: 18px; border-radius: 10px; font-family: 'Ubuntu Mono', 'Consolas', monospace; font-size: 15px; line-height: 1.6; white-space: pre-wrap; word-break: break-word; margin: 0; max-height: 600px; overflow-y: auto; border: 1px solid #1e293b;">{}</pre>
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
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Ubuntu:ital,wght@0,400;0,500;0,700;1,400&family=Ubuntu+Mono:wght@400;700&display=swap" rel="stylesheet">
<style>
  body, table, td, p, div, span {{ font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }}
  code, pre {{ font-family: 'Ubuntu Mono', 'Consolas', 'Courier New', monospace; }}
  a {{ color: #ffffff !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #ffffff !important; }}
  a:hover {{ color: #e0f2fe !important; }}
  td a, p a {{ background: #2563eb; color: #ffffff !important; padding: 2px 8px; border-radius: 6px; text-decoration: none; display: inline-block; font-size: 14px; font-weight: bold; }}
  td a:hover, p a:hover {{ background: #1d4ed8; color: #ffffff !important; }}
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
  <div style="max-width: 680px; margin: 0 auto; background: #ffffff; border-radius: 14px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 4px solid #38bdf8; padding: 24px 28px; color: #ffffff;">
      <div style="margin-bottom: 12px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #38bdf8; padding: 5px 12px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 13px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[{} | {} | {}]</span>
        <span style="background: {}; color: #ffffff; padding: 5px 12px; border-radius: 9999px; font-size: 12px; font-weight: 700; text-transform: uppercase; margin-left: 8px; letter-spacing: 0.06em;">{}</span>
      </div>
      <h2 style="margin: 6px 0 0 0; font-size: 24px; color: #ffffff; font-weight: 700; line-height: 1.3;">{}</h2>
    </div>
    <div style="padding: 28px;">
      <p style="margin: 0 0 18px 0; color: #475569; font-size: 15px; line-height: 1.6;">{}</p>
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 22px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 170px; background: #f8fafc;">Version</td><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 170px; background: #f8fafc;">Command</td><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;"><code>{}</code></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">Origin Node</td><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 600;">{} ({})</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">Target Instance</td><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td></tr>
        {}
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">Timestamp</td><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td></tr>
      </table>
      {}
    </div>
    <div style="background: #f8fafc; padding: 16px 28px; border-top: 1px solid #e2e8f0; font-size: 13px; color: #64748b; text-align: center;">
      Automated Remote Dispatcher &middot; Antigravity Manager {} &middot; Maintained by Alim, Sponsored by RISEUP ASIA LLC
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
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_reply_with_failover(&subject, &body, &[clean_sender], in_reply_to),
        "dispatch_reply_with_failover",
    );
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
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_reply_with_failover(&subject, &body, &[clean_sender], in_reply_to),
        "dispatch_reply_with_failover",
    );
}
