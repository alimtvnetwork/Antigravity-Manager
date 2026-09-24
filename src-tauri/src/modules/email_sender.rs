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

/// Build a rich, responsive HTML card for any notification or command output
pub fn wrap_html_email_card(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let escaped_content = escape_html_entities(content.trim());
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
</head>
<body style="margin: 0; padding: 20px; background-color: #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
  <div style="max-width: 640px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(15,23,42,0.08); border: 1px solid #e2e8f0;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%); padding: 20px 24px; color: #ffffff;">
      <div style="margin-bottom: 10px;">
        <span style="background: #334155; color: #38bdf8; padding: 4px 10px; border-radius: 6px; font-family: 'Consolas', monospace; font-size: 12px; font-weight: 700; border: 1px solid #475569;">NODE: {} | IP: {}</span>
        <span style="background: #2563eb; color: #ffffff; padding: 4px 10px; border-radius: 9999px; font-size: 11px; font-weight: 700; text-transform: uppercase; margin-left: 8px;">AGM TELEMETRY</span>
      </div>
      <h2 style="margin: 6px 0 0 0; font-size: 18px; color: #ffffff; font-weight: 700;">{}</h2>
    </div>
    <div style="padding: 24px;">
      <table style="width: 100%; border-collapse: collapse; font-size: 13px; margin-bottom: 16px; background: #f8fafc; border-radius: 8px; overflow: hidden; border: 1px solid #e2e8f0;">
        <tr>
          <td style="padding: 8px 12px; border-bottom: 1px solid #e2e8f0; color: #64748b; font-weight: 600; width: 130px;">VM / Node Alias</td>
          <td style="padding: 8px 12px; border-bottom: 1px solid #e2e8f0; color: #0f172a; font-family: monospace; font-weight: 700;">{}</td>
        </tr>
        <tr>
          <td style="padding: 8px 12px; border-bottom: 1px solid #e2e8f0; color: #64748b; font-weight: 600;">Local IPv4</td>
          <td style="padding: 8px 12px; border-bottom: 1px solid #e2e8f0; color: #0f172a; font-family: monospace;">{}</td>
        </tr>
        <tr>
          <td style="padding: 8px 12px; color: #64748b; font-weight: 600;">Dispatched At</td>
          <td style="padding: 8px 12px; color: #0f172a; font-family: monospace;">{}</td>
        </tr>
      </table>
      <div style="font-weight: 700; font-size: 11px; text-transform: uppercase; color: #64748b; margin-bottom: 8px; letter-spacing: 0.05em;">Message Details</div>
      <pre style="background: #0f172a; color: #e2e8f0; padding: 16px; border-radius: 8px; font-family: 'Consolas', 'Courier New', monospace; font-size: 12px; line-height: 1.55; white-space: pre-wrap; word-break: break-word; margin: 0; border: 1px solid #1e293b;">{}</pre>
    </div>
    <div style="background: #f8fafc; padding: 14px 24px; border-top: 1px solid #e2e8f0; font-size: 11px; color: #94a3b8; text-align: center;">
      Automated Remote Dispatcher &middot; Antigravity Manager &middot; Node {} ({})
    </div>
  </div>
</body>
</html>"#,
        machine_name,
        machine_ip,
        escape_html_entities(title),
        machine_name,
        machine_ip,
        now_str,
        escaped_content,
        machine_name,
        machine_ip
    )
}

/// Build full RFC 5322 / RFC 2046 MIME email message payload
/// Guarantees:
/// 1. Subject always contains `[<VM_ALIAS> | <LOCAL_IP>]`
/// 2. Body is always rendered as a rich HTML card (`multipart/alternative` with Base64 transfer encoding)
fn build_mime_message(
    account: &EmailAccount,
    subject: &str,
    body: &str,
    recipients: &[String],
    in_reply_to: Option<&str>,
) -> String {
    let m_name = crate::modules::email_watcher::detect_machine_name();
    let m_ip = crate::modules::email_watcher::detect_local_ip();
    let node_tag = format!("[{} | {}]", m_name, m_ip);

    let normalized_subject = if subject.contains(&m_ip)
        || (subject.contains('[') && subject.contains('|') && subject.contains(']'))
    {
        subject.trim().to_string()
    } else if subject.trim().to_lowercase().starts_with("re:") {
        let rest = subject.trim()[3..].trim();
        format!("Re: {} {}", node_tag, rest)
    } else {
        format!("{} {}", node_tag, subject.trim())
    };

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

    let html_body = if is_html_content(body) {
        body.to_string()
    } else {
        wrap_html_email_card(&normalized_subject, body, &m_name, &m_ip)
    };

    let boundary = format!("===============AGM_{}==", Uuid::new_v4().simple());
    let plain_fallback = strip_html_tags(&html_body);
    let b64_plain = encode_mime_base64_body(&plain_fallback);
    let b64_html = encode_mime_base64_body(&html_body);

    let mut headers = format!(
        "From: {} <{}>\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"{}\"\r\nX-Origin-Node: {}\r\nX-Origin-IP: {}\r\nX-Mailer: Antigravity-Manager-Mailer/4.71.2\r\n",
        account.alias,
        account.email,
        to_header,
        encoded_subject,
        date,
        msg_id,
        boundary,
        m_name,
        m_ip
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
    let subject = format!(
        "[{} | {}] [AGM Alert] Low Quota Warning ({:.1}%) - {}",
        machine_name, machine_ip, current_quota, email
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
    let subject = format!(
        "[{} | {}] [AGM Notice] Workspace Auto-Switched: {}",
        machine_name, machine_ip, to_instance
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
    let subject = format!(
        "[{} | {}] [AGM Prompt Request] Running Projects Idle - Ready for Instructions",
        machine_name, machine_ip
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
    let subject = format!(
        "[{} | {}] [AGM Execution Report] exit: {} - {}",
        machine_name, machine_ip, exit_code, cmd
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
    let subject = format!(
        "[{} | {}] [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet",
        machine_name, machine_ip
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
    let subject = format!(
        "[{} | {}] [AGM Test] Mailbox Connection Verified - {}",
        machine_name, machine_ip, email
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
    let subject = format!(
        "[{} | {}] [AGM Ping] Test Command Ping: {}",
        machine_name, machine_ip, project_name
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
        let (subj, text) =
            render_quota_drop_email("test@example.com", 12.5, 15, "my-pc", "192.168.1.50");
        assert!(subj.contains("12.5%"));
        assert!(subj.starts_with("[my-pc | 192.168.1.50]"));
        assert!(text.contains("192.168.1.50"));
        assert!(text.contains("<html"));
        assert!(text.contains("<div"));
    }

    #[test]
    fn test_build_mime_message_html_multipart() {
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
        let html_body = "<div style=\"color: red;\"><h1>Test Alert</h1><p>Running on VM3</p></div>";
        let mime = build_mime_message(
            &account,
            "Test Subject",
            html_body,
            &["user@example.com".to_string()],
            None,
        );

        assert!(mime.contains("Content-Type: multipart/alternative; boundary="));
        assert!(mime.contains("Content-Type: text/plain; charset=UTF-8"));
        assert!(mime.contains("Content-Type: text/html; charset=UTF-8"));
        assert!(mime.contains("Content-Transfer-Encoding: base64"));
    }
}
