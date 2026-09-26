//! Outbound Email Dispatcher Module
//! Implements SMTP delivery with failover mailbox swapping,
//! anti-spam HTML templates, and machine telemetry headers.

#![allow(dead_code)]

use crate::modules::email_vault_db::{self, EmailAccount};
use base64::prelude::*;
use boring2::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode};
use chrono::Utc;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use uuid::Uuid;

/// Stream wrapper supporting both plaintext TCP and TLS streams
pub enum EmailStream {
    Plain(TcpStream),
    Tls(SslStream<TcpStream>),
}

impl Read for EmailStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            EmailStream::Plain(s) => s.read(buf),
            EmailStream::Tls(s) => s.read(buf),
        }
    }
}

impl Write for EmailStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            EmailStream::Plain(s) => s.write(buf),
            EmailStream::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            EmailStream::Plain(s) => s.flush(),
            EmailStream::Tls(s) => s.flush(),
        }
    }
}

impl EmailStream {
    pub fn upgrade_to_tls(self, host: &str) -> Result<Self, String> {
        match self {
            EmailStream::Plain(tcp) => {
                let mut builder = SslConnector::builder(SslMethod::tls())
                    .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
                builder.set_verify(SslVerifyMode::NONE);
                let connector = builder.build();
                let tls = connector
                    .configure()
                    .map_err(|e| format!("Failed to configure TLS: {}", e))?
                    .verify_hostname(false)
                    .connect(host, tcp)
                    .map_err(|e| format!("TLS handshake failed with '{}': {}", host, e))?;
                Ok(EmailStream::Tls(tls))
            }
            EmailStream::Tls(_) => Ok(self),
        }
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        match self {
            EmailStream::Plain(s) => s.set_read_timeout(timeout).map_err(|e| e.to_string()),
            EmailStream::Tls(s) => s
                .get_ref()
                .set_read_timeout(timeout)
                .map_err(|e| e.to_string()),
        }
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        match self {
            EmailStream::Plain(s) => s.set_write_timeout(timeout).map_err(|e| e.to_string()),
            EmailStream::Tls(s) => s
                .get_ref()
                .set_write_timeout(timeout)
                .map_err(|e| e.to_string()),
        }
    }
}

pub fn is_implicit_tls_smtp(port: u16, encryption_type: &str) -> bool {
    if port == 465 {
        return true;
    }
    let enc = encryption_type.trim();
    if enc.eq_ignore_ascii_case("SSL") {
        return true;
    }
    if enc.eq_ignore_ascii_case("SMTPS") {
        return true;
    }
    false
}

pub fn is_starttls_smtp(port: u16, encryption_type: &str) -> bool {
    if is_implicit_tls_smtp(port, encryption_type) {
        return false;
    }
    if port == 587 {
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

/// Result of an email delivery attempt
#[derive(Debug, Clone)]
pub struct SendResult {
    pub is_success: bool,
    pub used_account_id: String,
    pub used_account_email: String,
    pub attempts_count: usize,
    pub message: String,
}

/// Clean recipient string into RFC 5321 pure email address
pub fn clean_recipient_email(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find('<') {
        if let Some(end) = trimmed[start + 1..].find('>') {
            let inner = &trimmed[start + 1..start + 1 + end];
            return inner.trim().to_string();
        }
    }
    trimmed
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'')
        .trim()
        .to_string()
}

/// Send an email with automatic failover pool swapping across active accounts
pub fn dispatch_email_with_failover(
    subject: &str,
    html_body: &str,
    recipients: &[String],
) -> Result<SendResult, String> {
    dispatch_reply_with_failover(subject, html_body, recipients, None)
}

/// Send an email with threading headers (In-Reply-To, References) and failover
pub fn dispatch_reply_with_failover(
    subject: &str,
    html_body: &str,
    recipients: &[String],
    in_reply_to: Option<&str>,
) -> Result<SendResult, String> {
    if recipients.is_empty() {
        return Err("No recipients specified for delivery".to_string());
    }

    let accounts = email_vault_db::list_email_accounts()?;
    let active_accounts: Vec<EmailAccount> = accounts.into_iter().filter(|a| a.is_active).collect();

    if active_accounts.is_empty() {
        return Err("No active email accounts configured in vault".to_string());
    }

    // Prioritize default account first, then fallback to others
    let mut prioritized = active_accounts.clone();
    prioritized.sort_by_key(|a| if a.is_default { 0 } else { 1 });

    let mut last_error = String::new();
    let mut attempts = 0;

    for account in prioritized {
        attempts += 1;
        match send_via_account_credentials_with_reply(
            &account,
            &email_vault_db::get_account_secret(&account.id).unwrap_or_default(),
            subject,
            html_body,
            recipients,
            in_reply_to,
        ) {
            Ok(_) => {
                crate::modules::logger::log_info(&format!(
                    "[EmailDispatcher] Successfully sent email '{}' via account '{}'",
                    subject, account.email
                ));
                return Ok(SendResult {
                    is_success: true,
                    used_account_id: account.id,
                    used_account_email: account.email,
                    attempts_count: attempts,
                    message: "Email dispatched successfully".to_string(),
                });
            }
            Err(err) => {
                crate::modules::logger::log_warn(&format!(
                    "[EmailDispatcher] Failed delivery via '{}': {}. Attempting failover...",
                    account.email, err
                ));
                last_error = err;
            }
        }
    }

    Err(format!(
        "All {} active email accounts failed. Last error: {}",
        attempts, last_error
    ))
}

/// Send email through a specific email account via SMTP
pub fn send_via_account(
    account: &EmailAccount,
    subject: &str,
    html_body: &str,
    recipients: &[String],
) -> Result<(), String> {
    let password = email_vault_db::get_account_secret(&account.id).unwrap_or_default();
    send_via_account_credentials(account, &password, subject, html_body, recipients)
}

/// Send email through an account with explicitly passed credentials
pub fn send_via_account_credentials(
    account: &EmailAccount,
    password: &str,
    subject: &str,
    html_body: &str,
    recipients: &[String],
) -> Result<(), String> {
    send_via_account_credentials_with_reply(account, password, subject, html_body, recipients, None)
}

/// Send email through an account with explicitly passed credentials and optional threading headers
pub fn send_via_account_credentials_with_reply(
    account: &EmailAccount,
    password: &str,
    subject: &str,
    html_body: &str,
    recipients: &[String],
    in_reply_to: Option<&str>,
) -> Result<(), String> {
    let addr = format!("{}:{}", account.smtp_host, account.smtp_port);

    // Resolve hostname or IP
    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| {
            format!(
                "Failed to resolve SMTP server '{}:{}': {}",
                account.smtp_host, account.smtp_port, e
            )
        })?
        .next()
        .ok_or_else(|| {
            format!(
                "No socket address resolved for '{}:{}'",
                account.smtp_host, account.smtp_port
            )
        })?;

    // Establish TCP connection with timeout
    let tcp_stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10)).map_err(|e| {
            format!(
                "TCP connection to SMTP server '{}:{}' failed: {}",
                account.smtp_host, account.smtp_port, e
            )
        })?;

    tcp_stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    tcp_stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    let is_implicit = is_implicit_tls_smtp(account.smtp_port, &account.encryption_type);
    let mut stream = if is_implicit {
        EmailStream::Plain(tcp_stream).upgrade_to_tls(&account.smtp_host)?
    } else {
        EmailStream::Plain(tcp_stream)
    };

    // Read greeting
    read_smtp_response(&mut stream)?;

    // Send EHLO
    send_smtp_cmd(&mut stream, "EHLO localhost", false)?;
    read_smtp_response(&mut stream)?;

    let is_starttls = is_starttls_smtp(account.smtp_port, &account.encryption_type);
    if is_starttls {
        send_smtp_cmd(&mut stream, "STARTTLS", false)?;
        read_smtp_response(&mut stream)?;
        stream = stream.upgrade_to_tls(&account.smtp_host)?;
        send_smtp_cmd(&mut stream, "EHLO localhost", false)?;
        read_smtp_response(&mut stream)?;
    }

    // Handle authentication if password exists
    if !password.is_empty() {
        send_smtp_cmd(&mut stream, "AUTH LOGIN", false)?;
        read_smtp_response(&mut stream)?;

        let b64_user = BASE64_STANDARD.encode(&account.email);
        send_smtp_cmd(&mut stream, &b64_user, false)?;
        read_smtp_response(&mut stream)?;

        let b64_pass = BASE64_STANDARD.encode(&password);
        send_smtp_cmd(&mut stream, &b64_pass, true)?;
        read_smtp_response(&mut stream)?;
    }

    // MAIL FROM
    send_smtp_cmd(
        &mut stream,
        &format!("MAIL FROM:<{}>", account.email),
        false,
    )?;
    read_smtp_response(&mut stream)?;

    // RCPT TO (RFC 5321 pure address without display name or nested brackets)
    for rcpt in recipients {
        let clean_rcpt = clean_recipient_email(rcpt);
        if clean_rcpt.is_empty() {
            continue;
        }
        send_smtp_cmd(&mut stream, &format!("RCPT TO:<{}>", clean_rcpt), false)?;
        read_smtp_response(&mut stream)?;
    }

    // DATA
    send_smtp_cmd(&mut stream, "DATA", false)?;
    read_smtp_response(&mut stream)?;

    // Build MIME payload
    let mime = build_mime_message(account, subject, html_body, recipients, in_reply_to);
    stream
        .write_all(mime.as_bytes())
        .map_err(|e| format!("Failed to write MIME data: {}", e))?;
    stream
        .write_all(b"\r\n.\r\n")
        .map_err(|e| format!("Failed to write end of DATA: {}", e))?;
    read_smtp_response(&mut stream)?;

    // QUIT
    let _ = send_smtp_cmd(&mut stream, "QUIT", false);
    Ok(())
}

/// Send single SMTP command line with optional credential redaction on error
fn send_smtp_cmd(stream: &mut EmailStream, cmd: &str, is_sensitive: bool) -> Result<(), String> {
    let line = format!("{}\r\n", cmd);
    stream.write_all(line.as_bytes()).map_err(|e| {
        if is_sensitive {
            format!("Failed to send SMTP command '<REDACTED>': {}", e)
        } else {
            format!("Failed to send SMTP command '{}': {}", cmd, e)
        }
    })
}

/// Read SMTP response and verify status code (< 400 is success)
fn read_smtp_response(stream: &mut EmailStream) -> Result<String, String> {
    let mut total_resp = String::new();
    let mut buf = [0u8; 1024];

    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| format!("Failed to read SMTP response: {}", e))?;
        if n == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buf[0..n]);
        total_resp.push_str(&chunk);

        let mut is_done = false;
        for line in total_resp.lines() {
            let trimmed = line.trim_start();
            if trimmed.len() == 3 {
                let is_digits = trimmed.chars().all(|c| c.is_ascii_digit());
                if is_digits {
                    is_done = true;
                }
            }
            if trimmed.len() >= 4 {
                let code_part = &trimmed[0..3];
                let sep = &trimmed[3..4];
                let is_digits = code_part.chars().all(|c| c.is_ascii_digit());
                if is_digits {
                    if sep == " " {
                        is_done = true;
                    }
                }
            }
        }
        if is_done {
            break;
        }
    }

    if total_resp.is_empty() {
        return Err("Empty response from SMTP server".to_string());
    }

    let mut last_code = 200u16;
    for line in total_resp.lines() {
        let trimmed = line.trim_start();
        if trimmed.len() >= 3 {
            if let Ok(code) = trimmed[0..3].parse::<u16>() {
                last_code = code;
            }
        }
    }
    if last_code >= 400 {
        return Err(format!("SMTP server error: {}", total_resp.trim()));
    }
    Ok(total_resp)
}

/// Detect if content contains HTML markup
fn is_html_content(content: &str) -> bool {
    let lower = content.trim().to_lowercase();
    lower.contains("<html")
        || lower.contains("<!doctype")
        || lower.contains("<div")
        || lower.contains("<table")
        || lower.contains("<body")
        || lower.contains("<span style=")
}

/// Strip HTML tags for clean RFC 2046 plaintext fallback
fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    let lines: Vec<&str> = out
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    lines.join("\r\n")
}

/// Encode string as RFC 2045 76-character CRLF-wrapped base64 payload
fn encode_mime_base64_body(input: &str) -> String {
    let b64 = BASE64_STANDARD.encode(input.as_bytes());
    let mut wrapped = String::with_capacity(b64.len() + (b64.len() / 76) * 2 + 4);
    for (idx, ch) in b64.chars().enumerate() {
        if idx > 0 && idx % 76 == 0 {
            wrapped.push_str("\r\n");
        }
        wrapped.push(ch);
    }
    wrapped
}

/// Escape basic HTML entities for safe inclusion in <pre> or table cells
fn escape_html_entities(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format an RFC 5322 subject with standard telemetry prefix: `[Antigravity | v<VERSION> | <VM_ALIAS> | <LOCAL_IP>]`
/// Replaces any obsolete or unversioned prefix like `[VM | IP]`, `[vX | VM | IP]`, or duplicate `[Antigravity]`,
/// preserves `Re:` prefix, and handles arbitrary whitespace around pipes.
pub fn format_subject_with_telemetry(
    subject: &str,
    pkg_ver: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    let prefix = format!(
        "[Antigravity | {} | {} | {}]",
        pkg_ver, machine_name, machine_ip
    );
    let mut trimmed = subject.trim();

    // Strip any leading telemetry tag like [Antigravity | ...], [v4.75.0 | ...], or [VM | IP]
    if trimmed.starts_with('[') {
        if let Some(end_idx) = trimmed.find(']') {
            let inside = &trimmed[1..end_idx];
            if inside.contains('|') {
                trimmed = trimmed[end_idx + 1..].trim();
            }
        }
    }

    let is_reply = trimmed.to_lowercase().starts_with("re:");
    if is_reply {
        trimmed = trimmed[3..].trim();
        // Check if reply had a secondary telemetry tag after Re:
        if trimmed.starts_with('[') {
            if let Some(end_idx) = trimmed.find(']') {
                let inside = &trimmed[1..end_idx];
                if inside.contains('|') {
                    trimmed = trimmed[end_idx + 1..].trim();
                }
            }
        }
    }

    // Strip leading [Antigravity] if present (case-insensitive)
    while trimmed.to_lowercase().starts_with("[antigravity]") {
        trimmed = trimmed["[antigravity]".len()..].trim();
    }

    if is_reply {
        if trimmed.is_empty() {
            format!("{} Re:", prefix)
        } else {
            format!("{} Re: {}", prefix, trimmed)
        }
    } else if trimmed.is_empty() {
        prefix
    } else {
        format!("{} {}", prefix, trimmed)
    }
}

/// Helper to extract clean pretty-printed JSON from raw text or strip HTML wrappers
pub fn extract_clean_json_body(raw: &str) -> String {
    let trimmed = raw.trim();
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            let candidate = &trimmed[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                return serde_json::to_string_pretty(&val).unwrap_or_else(|_| candidate.to_string());
            }
        }
    }
    if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.rfind(']')) {
        if end > start {
            let candidate = &trimmed[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                return serde_json::to_string_pretty(&val).unwrap_or_else(|_| candidate.to_string());
            }
        }
    }
    strip_html_tags(trimmed)
}

/// Helper to convert key-value lines into rich table rows with Ubuntu styling and better coloring
fn format_content_as_table_rows(content: &str) -> (bool, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut table_rows = String::new();
    let mut non_kv_lines = Vec::new();
    let mut kv_count = 0;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if line is Key: Value
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let val = trimmed[colon_pos + 1..].trim();
            if !key.is_empty() && !val.is_empty() && key.len() <= 45 && !key.contains("://") && !key.starts_with("http") {
                kv_count += 1;
                let val_badge = if val.eq_ignore_ascii_case("yes") || val.starts_with("Yes") || val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("success") || val.eq_ignore_ascii_case("pass") {
                    format!(r#"<span style="background: #ecfdf5; color: #059669; padding: 4px 10px; border-radius: 6px; font-weight: 700; border: 1px solid #a7f3d0; font-size: 14px;">{}</span>"#, escape_html_entities(val))
                } else if val.eq_ignore_ascii_case("no") || val.starts_with("No") || val.eq_ignore_ascii_case("none") {
                    format!(r#"<span style="color: #64748b; font-weight: 500;">{}</span>"#, escape_html_entities(val))
                } else if val.contains('%') {
                    format!(r#"<span style="color: #0284c7; font-weight: 700; font-family: 'Ubuntu Mono', monospace;">{}</span>"#, escape_html_entities(val))
                } else if val.contains('@') {
                    format!(r#"<span style="color: #0f172a; font-weight: 700; font-family: 'Ubuntu Mono', monospace;">{}</span>"#, escape_html_entities(val))
                } else {
                    format!(r#"<span style="color: #0f172a; font-weight: 600;">{}</span>"#, escape_html_entities(val))
                };

                table_rows.push_str(&format!(
                    r#"<tr>
  <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; font-size: 15px; width: 190px; background: #f8fafc;">{}</td>
  <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-size: 15px; line-height: 1.5;">{}</td>
</tr>"#,
                    escape_html_entities(key),
                    val_badge
                ));
                continue;
            }
        }

        non_kv_lines.push(escape_html_entities(trimmed));
    }

    if kv_count >= 2 {
        let mut out = format!(
            r#"<table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 20px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
  {}
</table>"#,
            table_rows
        );
        if !non_kv_lines.is_empty() {
            out.push_str(&format!(
                r#"<div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin: 16px 0 8px 0; letter-spacing: 0.08em;">Details</div>
<div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px 18px; color: #334155; font-size: 15px; line-height: 1.6;">{}</div>"#,
                non_kv_lines.join("<br>")
            ));
        }
        (true, out)
    } else {
        (false, String::new())
    }
}

/// Build a rich, responsive HTML card for any notification or command output
pub fn wrap_html_email_card(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    let (is_table_format, rendered_table) = format_content_as_table_rows(content);
    let details_section = if is_table_format {
        rendered_table
    } else {
        let escaped_content = escape_html_entities(content.trim());
        format!(
            r#"<div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin-bottom: 10px; letter-spacing: 0.08em;">Message Details</div>
<pre style="background: #0f172a; color: #e2e8f0; padding: 18px; border-radius: 10px; font-family: 'Ubuntu Mono', 'Consolas', monospace; font-size: 14px; line-height: 1.6; white-space: pre-wrap; word-break: break-word; margin: 0; border: 1px solid #1e293b;">{}</pre>"#,
            escaped_content
        )
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
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
  <div style="max-width: 680px; margin: 0 auto; background: #ffffff; border-radius: 14px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 4px solid #38bdf8; padding: 24px 28px; color: #ffffff;">
      <div style="margin-bottom: 12px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #38bdf8; padding: 5px 12px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 13px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[{} | {} | {}]</span>
        <span style="background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%); color: #ffffff; padding: 5px 12px; border-radius: 9999px; font-size: 11px; font-weight: 700; text-transform: uppercase; margin-left: 8px; letter-spacing: 0.06em;">AGM TELEMETRY</span>
      </div>
      <h2 style="margin: 6px 0 0 0; font-size: 24px; color: #ffffff; font-weight: 700; line-height: 1.3;">{}</h2>
    </div>
    <div style="padding: 28px;">
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 22px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 170px; background: #f8fafc;">Application Version</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">VM / Node Alias</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">Local IPv4</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; color: #475569; font-weight: 600; background: #f8fafc;">Dispatched At</td>
          <td style="padding: 12px 18px; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td>
        </tr>
      </table>
      {}
    </div>
    <div style="background: #f8fafc; padding: 16px 28px; border-top: 1px solid #e2e8f0; font-size: 13px; color: #64748b; text-align: center;">
      Automated Remote Dispatcher &middot; Antigravity Manager {} &middot; Node {} ({})
    </div>
  </div>
</body>
</html>"#,
        pkg_ver,
        machine_name,
        machine_ip,
        escape_html_entities(title),
        pkg_ver,
        machine_name,
        machine_ip,
        now_str,
        details_section,
        pkg_ver,
        machine_name,
        machine_ip
    )
}

/// Render rich table-based HTML email for Node & Credits Status Report
pub fn render_node_credits_status_table_html(
    version: &str,
    node_alias: &str,
    local_ip: &str,
    active_account: Option<&str>,
    tier: &str,
    predicted_next: Option<&str>,
    immediate_credits: f64,
    weekly_credits: f64,
    threshold_percent: f64,
    running_instances: usize,
    total_instances: usize,
    running_prompts: usize,
    prompts_resent: bool,
    has_images: bool,
    accounts_count: usize,
    recipients_count: usize,
) -> String {
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let imm_color = if immediate_credits >= 50.0 {
        "#059669"
    } else if immediate_credits >= 20.0 {
        "#d97706"
    } else {
        "#dc2626"
    };
    let weekly_color = if weekly_credits >= 50.0 {
        "#0284c7"
    } else if weekly_credits >= 20.0 {
        "#d97706"
    } else {
        "#dc2626"
    };

    let active_acc_display = active_account.unwrap_or("(None / Standby)");
    let pred_next_display = predicted_next.unwrap_or("None / Standby");

    let resent_badge = if prompts_resent {
        r#"<span style="background: #ecfdf5; color: #059669; padding: 4px 10px; border-radius: 6px; font-weight: 700; border: 1px solid #a7f3d0; font-size: 14px;">Yes (Auto-Resumed via resume file)</span>"#
    } else {
        r#"<span style="color: #64748b; font-size: 14px;">No</span>"#
    };

    let images_badge = if has_images {
        r#"<span style="background: #eff6ff; color: #2563eb; padding: 4px 10px; border-radius: 6px; font-weight: 700; border: 1px solid #bfdbfe; font-size: 14px;">Yes (Base64 payload preserved)</span>"#
    } else {
        r#"<span style="color: #64748b; font-size: 14px;">None</span>"#
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
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
  <div style="max-width: 680px; margin: 0 auto; background: #ffffff; border-radius: 14px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 4px solid #38bdf8; padding: 24px 28px; color: #ffffff;">
      <div style="margin-bottom: 12px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #38bdf8; padding: 5px 12px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 13px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[{} | {} | {}]</span>
        <span style="background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%); color: #ffffff; padding: 5px 12px; border-radius: 9999px; font-size: 11px; font-weight: 700; text-transform: uppercase; margin-left: 8px; letter-spacing: 0.06em;">AGM STATUS</span>
      </div>
      <h2 style="margin: 6px 0 0 0; font-size: 24px; color: #ffffff; font-weight: 700; line-height: 1.3;">Node &amp; Credits Status</h2>
    </div>
    <div style="padding: 28px;">
      <div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin-bottom: 10px; letter-spacing: 0.08em;">Node Telemetry</div>
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 24px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Application Version</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;">v{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">VM / Node Alias</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; background: #f8fafc;">Local IPv4</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; color: #475569; font-weight: 600; background: #f8fafc;">Dispatched At</td>
          <td style="padding: 12px 18px; color: #0f172a; font-family: 'Ubuntu Mono', monospace;">{}</td>
        </tr>
      </table>

      <div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin-bottom: 10px; letter-spacing: 0.08em;">Credits &amp; Rotation State</div>
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 24px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; width: 190px; background: #f8fafc;">Active Account</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">
            <span style="font-weight: 700; font-family: 'Ubuntu Mono', monospace;">{}</span>
            <span style="background: #e0f2fe; color: #0369a1; padding: 2px 8px; border-radius: 6px; font-size: 12px; font-weight: 700; margin-left: 8px;">{}</span>
          </td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Immediate Credits</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: {}; font-weight: 700; font-family: 'Ubuntu Mono', monospace; font-size: 16px;">
            {:.1}% remaining
          </td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Weekly Credits</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: {}; font-weight: 700; font-family: 'Ubuntu Mono', monospace; font-size: 16px;">
            {:.1}% remaining
          </td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Threshold Target</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-weight: 600;">
            {:.1}% (auto-switch trigger)
          </td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; color: #334155; font-weight: 600; background: #f8fafc;">Predicted Next</td>
          <td style="padding: 12px 18px; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 600;">
            {}
          </td>
        </tr>
      </table>

      <div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin-bottom: 10px; letter-spacing: 0.08em;">Instances &amp; Workload</div>
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 8px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; width: 190px; background: #f8fafc;">Running Instances</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-weight: 700;">
            <span style="background: #f1f5f9; padding: 3px 10px; border-radius: 6px;">{} / {} running</span>
          </td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Active Prompts</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-weight: 600;">{} queued/running</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Prompts Resent</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; background: #f8fafc;">Attached Images</td>
          <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9;">{}</td>
        </tr>
        <tr>
          <td style="padding: 12px 18px; color: #334155; font-weight: 600; background: #f8fafc;">Mailbox Infrastructure</td>
          <td style="padding: 12px 18px; color: #0f172a; font-size: 14px;">{} account(s) configured &middot; {} recipient(s)</td>
        </tr>
      </table>
    </div>
    <div style="background: #f8fafc; padding: 16px 28px; border-top: 1px solid #e2e8f0; font-size: 13px; color: #64748b; text-align: center;">
      Automated Remote Dispatcher &middot; Antigravity Manager v{} &middot; Node {} ({})
    </div>
  </div>
</body>
</html>"#,
        version,
        node_alias,
        local_ip,
        version,
        node_alias,
        local_ip,
        now_str,
        active_acc_display,
        tier,
        imm_color,
        immediate_credits,
        weekly_color,
        weekly_credits,
        threshold_percent,
        pred_next_display,
        running_instances,
        total_instances,
        running_prompts,
        resent_badge,
        images_badge,
        accounts_count,
        recipients_count,
        version,
        node_alias,
        local_ip
    )
}

/// Build full RFC 5322 / RFC 2046 MIME email message payload
/// Guarantees:
/// 1. Subject always contains `[v<VERSION> | <VM_ALIAS> | <LOCAL_IP>]`
/// 2. Body is always rendered as a rich HTML card (`multipart/alternative` with Base64 transfer encoding)
fn build_mime_message(
    account: &EmailAccount,
    subject: &str,
    body: &str,
    recipients: &[String],
    in_reply_to: Option<&str>,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = crate::modules::email_watcher::detect_machine_name();
    let m_ip = crate::modules::email_watcher::detect_local_ip();

    let normalized_subject = format_subject_with_telemetry(subject, &pkg_ver, &m_name, &m_ip);

    let msg_id = format!("<{}@{}>", Uuid::new_v4(), account.smtp_host);
    let date = Utc::now().to_rfc2822();
    let is_pure_ascii = normalized_subject
        .chars()
        .all(|c| c.is_ascii() && c != '\r' && c != '\n');
    let encoded_subject = if is_pure_ascii {
        normalized_subject.clone()
    } else {
        format!(
            "=?UTF-8?B?{}?=",
            BASE64_STANDARD.encode(normalized_subject.as_bytes())
        )
    };
    let clean_rcpts: Vec<String> = recipients
        .iter()
        .map(|r| clean_recipient_email(r))
        .filter(|s| !s.is_empty())
        .collect();
    let to_header = if clean_rcpts.is_empty() {
        recipients.join(", ")
    } else {
        clean_rcpts.join(", ")
    };

    let is_json_email = normalized_subject.contains("[JSON]") || normalized_subject.contains("[json]");

    if is_json_email {
        let clean_json = extract_clean_json_body(body);
        let b64_plain = encode_mime_base64_body(&clean_json);
        let mut headers = format!(
            "From: {} <{}>\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\nX-Origin-Node: {}\r\nX-Origin-IP: {}\r\nX-Origin-Version: {}\r\nX-Mailer: Antigravity-Manager-Mailer/{}\r\n",
            account.alias,
            account.email,
            to_header,
            encoded_subject,
            date,
            msg_id,
            m_name,
            m_ip,
            pkg_ver,
            pkg_ver
        );

        if let Some(reply_id) = in_reply_to {
            let clean_reply_id = reply_id.trim();
            if !clean_reply_id.is_empty() {
                let formatted_id = if clean_reply_id.starts_with('<') && clean_reply_id.ends_with('>') {
                    clean_reply_id.to_string()
                } else {
                    format!("<{}>", clean_reply_id)
                };
                headers.push_str(&format!(
                    "In-Reply-To: {}\r\nReferences: {}\r\n",
                    formatted_id, formatted_id
                ));
            }
        }

        let mut payload = headers;
        payload.push_str("\r\n");
        payload.push_str(&b64_plain);
        payload.push_str("\r\n");
        return payload;
    }

    let html_body = if body.trim().to_lowercase().starts_with("<!doctype")
        || body.trim().to_lowercase().starts_with("<html")
    {
        body.to_string()
    } else if is_html_content(body) {
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
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
{}
</body>
</html>"#,
            body.trim()
        )
    } else {
        wrap_html_email_card(&normalized_subject, body, &m_name, &m_ip)
    };

    let boundary = format!("===============AGM_{}==", Uuid::new_v4().simple());
    let plain_fallback = strip_html_tags(&html_body);
    let b64_plain = encode_mime_base64_body(&plain_fallback);
    let b64_html = encode_mime_base64_body(&html_body);

    let mut headers = format!(
        "From: {} <{}>\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"{}\"\r\nX-Origin-Node: {}\r\nX-Origin-IP: {}\r\nX-Origin-Version: {}\r\nX-Mailer: Antigravity-Manager-Mailer/{}\r\n",
        account.alias,
        account.email,
        to_header,
        encoded_subject,
        date,
        msg_id,
        boundary,
        m_name,
        m_ip,
        pkg_ver,
        pkg_ver
    );

    if let Some(reply_id) = in_reply_to {
        let clean_reply_id = reply_id.trim();
        if !clean_reply_id.is_empty() {
            let formatted_id = if clean_reply_id.starts_with('<') && clean_reply_id.ends_with('>') {
                clean_reply_id.to_string()
            } else {
                format!("<{}>", clean_reply_id)
            };
            headers.push_str(&format!(
                "In-Reply-To: {}\r\nReferences: {}\r\n",
                formatted_id, formatted_id
            ));
        }
    }

    let mut payload = headers;
    payload.push_str("\r\n");
    // Part 1: text/plain fallback (Base64 encoded for strict RFC 2045 MTA compliance)
    payload.push_str(&format!(
        "--{}\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n\r\n",
        boundary, b64_plain
    ));
    // Part 2: text/html rich layout (Base64 encoded so Gmail/Outlook never strip or flatten HTML)
    payload.push_str(&format!(
        "--{}\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n\r\n",
        boundary, b64_html
    ));
    // End boundary
    payload.push_str(&format!("--{}--\r\n", boundary));
    payload
}

// ---------------------------------------------------------------------------
// Rich HTML Email Templates (With Node Alias & Local IPv4 Telemetry)
// ---------------------------------------------------------------------------

fn wrap_plaintext_email(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    wrap_html_email_card(title, content, machine_name, machine_ip)
}

/// Render HTML email for low quota alerts
pub fn render_quota_drop_email(
    email: &str,
    current_quota: f64,
    threshold: u32,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[Antigravity | {} | {} | {}] Low Quota Warning ({:.1}%) - {}",
        pkg_ver, machine_name, machine_ip, current_quota, email
    );
    let content = format!(
        "[!] CREDIT THRESHOLD TRIGGER\r\n\r\n\
         The active profile '{}' has dropped to {:.1}% remaining credit,\r\n\
         falling below the configured safety threshold of {}%.\r\n\r\n\
         If auto-profile switcher is enabled, Antigravity Manager will attempt\r\n\
         to migrate active workspaces to the next highest credit account.",
        email, current_quota, threshold
    );
    let body = wrap_plaintext_email("Low Credit Alert", &content, machine_name, machine_ip);
    (subject, body)
}

/// Render HTML email for automated workspace switch
pub fn render_workspace_switch_email(
    from_instance: &str,
    to_instance: &str,
    reason: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[Antigravity | {} | {} | {}] [Notice] Workspace Auto-Switched: {}",
        pkg_ver, machine_name, machine_ip, to_instance
    );
    let content = format!(
        "[*] WORKSPACE SWITCHED\r\n\r\n\
         Antigravity Manager transitioned active tasks from '{}' to '{}'.\r\n\r\n\
         Reason: {}\r\n\r\n\
         Running prompts and repository states were safely snapshotted to repo_prompts.db.",
        from_instance, to_instance, reason
    );
    let body = wrap_plaintext_email(
        "Profile Migration Notice",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for idle running projects alert
pub fn render_idle_projects_email(
    projects: &[String],
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[Antigravity | {} | {} | {}] [Prompt Request] Running Projects Idle - Ready for Instructions",
        pkg_ver, machine_name, machine_ip
    );
    let mut proj_list = String::new();
    for p in projects {
        proj_list.push_str(&format!("  - {}\r\n", p));
    }

    let content = format!(
        "[*] IDLE WORKSPACE SENSOR\r\n\r\n\
         There are no active prompts currently running in the following active projects:\r\n\r\n\
         {}\r\n\
         Would you like to send something to these projects? Reply to this email with:\r\n\r\n\
         Subject: sub: {} | proj-<project-name>\r\n\r\n\
         <Your prompt instruction here...>\r\n\r\n\
         Antigravity Manager will read your reply and inject the prompt automatically.",
        proj_list.trim_end(),
        machine_name
    );
    let body = wrap_plaintext_email(
        "Idle Projects Notification",
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
        "[Antigravity | {} | {} | {}] [Execution Report] exit: {} - {}",
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
        "[Antigravity | {} | {} | {}] Inbound Remote Mailbox Instructions Cheat Sheet",
        pkg_ver, machine_name, machine_ip
    );
    let content = format!(
        "You can remotely command this Antigravity Manager instance by sending emails\r\n\
         matching the following pipe-delimited syntax in the subject (spaces around '|' are optional):\r\n\r\n\
         Format: [node|ip] | [instance] | <command>\r\n\
         Examples: {0}|1|help   OR   {0} | 1 | help   OR   {0}   |   1   |   switch\r\n\r\n\
         Supported Commands & Subject Examples:\r\n\r\n\
         1. Inject Prompt to Workspace:\r\n\
            Subject: {0} | 1 | prompt | proj-my-project\r\n\
            Body:    <Your prompt instruction here...>\r\n\r\n\
         2. Execute Command on Target Machine:\r\n\
            Subject: {0} | 1 | gitmap status\r\n\
            Body:    (Optional command arguments)\r\n\r\n\
         3. Rotate to Next Highest Quota Account (Smart Rotator):\r\n\
            Subject: {0} | 1 | rotate\r\n\r\n\
         4. Switch Account on Instance:\r\n\
            Subject: {0} | 1 | switch | user@gmail.com\r\n\r\n\
         5. Query Status Telemetry & Health:\r\n\
            Subject: {0} | 1 | status\r\n\r\n\
         6. Help & Cheat Sheet:\r\n\
            Subject: {0} | 1 | help",
        machine_name
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
        "[Antigravity | {} | {} | {}] Mailbox Connection Verified - {}",
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
        "[Antigravity | {} | {} | {}] Test Command Ping: {}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
        let (subj, text) =
            render_quota_drop_email("test@example.com", 12.5, 15, "my-pc", "192.168.1.50");
        assert!(subj.contains("12.5%"));
        assert!(subj.starts_with(&format!(
            "[Antigravity | {} | my-pc | 192.168.1.50]",
            pkg_ver
        )));
        assert!(!subj.contains("] [Antigravity]"));
        assert!(text.contains("192.168.1.50"));
        assert!(text.contains("<html"));
        assert!(text.contains("<div"));
        assert!(text.contains("Ubuntu"));
    }

    #[test]
    fn test_format_subject_with_telemetry() {
        let ver = "v4.71.4";
        let node = "VM3";
        let ip = "192.168.1.12";

        // Plain subject without duplicate [Antigravity]
        let s1 = format_subject_with_telemetry("[Antigravity] Account Switched", ver, node, ip);
        assert_eq!(
            s1,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Account Switched"
        );

        // Old tag upgrade
        let s2 = format_subject_with_telemetry("[VM3 | 192.168.1.12] Alert", ver, node, ip);
        assert_eq!(
            s2,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Alert"
        );

        // Reply subject with old tag
        let s3 = format_subject_with_telemetry("Re: [VM3 | 192.168.1.12] Result", ver, node, ip);
        assert_eq!(
            s3,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Re: Result"
        );

        // Reply subject without tag
        let s4 = format_subject_with_telemetry("Re: help", ver, node, ip);
        assert_eq!(
            s4,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Re: help"
        );

        // Already tagged cleanly
        let s5 = format_subject_with_telemetry(
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Status",
            ver,
            node,
            ip,
        );
        assert_eq!(
            s5,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Status"
        );

        // User request sample upgrade: strips redundant [Antigravity] before [JSON]
        let s6 = format_subject_with_telemetry(
            "[Antigravity | v4.75.0 | VM3 | 192.168.1.12] [Antigravity] [JSON] Node & Credits Status",
            ver,
            node,
            ip,
        );
        assert_eq!(
            s6,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] [JSON] Node & Credits Status"
        );

        // Other subject with redundant tag
        let s7 = format_subject_with_telemetry(
            "[v4.75.0 | VM3 | 192.168.1.12] [Antigravity] Write the subject",
            ver,
            node,
            ip,
        );
        assert_eq!(
            s7,
            "[Antigravity | v4.71.4 | VM3 | 192.168.1.12] Write the subject"
        );
    }

    #[test]
    fn test_build_mime_message_json_plain_no_html() {
        let account = EmailAccount {
            id: "acc-1".to_string(),
            alias: "Test Node".to_string(),
            email: "test@example.com".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
            encryption_type: "TLS".to_string(),
            is_default: true,
            is_active: true,
            created_at: 0,
            updated_at: 0,
        };
        let json_body = r#"{"event":"node_and_credits_status","immediate_credits":85.5}"#;
        let mime = build_mime_message(
            &account,
            "[JSON] Node & Credits Status",
            json_body,
            &["user@example.com".to_string()],
            None,
        );

        assert!(mime.contains("Content-Type: text/plain; charset=UTF-8"));
        assert!(!mime.contains("Content-Type: text/html"));
        assert!(!mime.contains("multipart/alternative"));
        assert!(mime.contains("Subject: [Antigravity | "));
        assert!(mime.contains("[JSON] Node & Credits Status"));
    }

    #[test]
    fn test_build_mime_message_html_multipart_ubuntu() {
        let account = EmailAccount {
            id: "acc-1".to_string(),
            alias: "Test Node".to_string(),
            email: "test@example.com".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
            encryption_type: "TLS".to_string(),
            is_default: true,
            is_active: true,
            created_at: 0,
            updated_at: 0,
        };
        let text_body = "Active Account: default@example.com\nImmediate Credits: 95.0%\nWeekly Credits: 98.0%";
        let mime = build_mime_message(
            &account,
            "Node & Credits Status",
            text_body,
            &["user@example.com".to_string()],
            None,
        );

        assert!(mime.contains("Content-Type: multipart/alternative; boundary="));
        assert!(mime.contains("Content-Type: text/plain; charset=UTF-8"));
        assert!(mime.contains("Content-Type: text/html; charset=UTF-8"));
        assert!(mime.contains("Content-Transfer-Encoding: base64"));
        assert!(!mime.contains("[JSON]"));
    }
}
