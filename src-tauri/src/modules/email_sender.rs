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

/// Send an email with automatic failover pool swapping across active accounts
pub fn dispatch_email_with_failover(
    subject: &str,
    html_body: &str,
    recipients: &[String],
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
        match send_via_account(&account, subject, html_body, recipients) {
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

    // RCPT TO
    for rcpt in recipients {
        send_smtp_cmd(&mut stream, &format!("RCPT TO:<{}>", rcpt), false)?;
        read_smtp_response(&mut stream)?;
    }

    // DATA
    send_smtp_cmd(&mut stream, "DATA", false)?;
    read_smtp_response(&mut stream)?;

    // Build MIME payload
    let mime = build_mime_message(account, subject, html_body, recipients);
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

/// Build full MIME email message with anti-spam formatting
fn build_mime_message(
    account: &EmailAccount,
    subject: &str,
    html_body: &str,
    recipients: &[String],
) -> String {
    let msg_id = format!("<{}@{}>", Uuid::new_v4(), account.smtp_host);
    let date = Utc::now().to_rfc2822();
    let encoded_subject = format!("=?UTF-8?B?{}?=", BASE64_STANDARD.encode(subject.as_bytes()));

    let content_type = if html_body.trim_start().starts_with('<') {
        "text/html; charset=UTF-8"
    } else {
        "text/plain; charset=UTF-8"
    };

    format!(
        "From: {} <{}>\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\nContent-Type: {}\r\nContent-Transfer-Encoding: 8bit\r\nX-Mailer: Antigravity-Manager-Mailer/4.62.0\r\n\r\n{}",
        account.alias,
        account.email,
        recipients.join(", "),
        encoded_subject,
        date,
        msg_id,
        content_type,
        html_body
    )
}

// ---------------------------------------------------------------------------
// HTML Email Templates
// ---------------------------------------------------------------------------

fn wrap_email_card(
    title: &str,
    content_html: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 20px; }}
    .card {{ max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05); }}
    .header {{ background: #0f172a; color: #ffffff; padding: 24px; text-align: left; }}
    .header h1 {{ margin: 0; font-size: 18px; font-weight: 600; }}
    .body {{ padding: 24px; color: #334155; line-height: 1.6; font-size: 14px; }}
    .badge {{ display: inline-block; padding: 4px 8px; border-radius: 6px; font-size: 12px; font-weight: 600; }}
    .badge-warn {{ background: #fef3c7; color: #92400e; }}
    .badge-info {{ background: #e0f2fe; color: #075985; }}
    .badge-success {{ background: #dcfce7; color: #166534; }}
    .footer {{ padding: 16px 24px; background: #f1f5f9; font-size: 12px; color: #64748b; border-top: 1px solid #e2e8f0; }}
    .cmd {{ background: #0f172a; color: #38bdf8; padding: 8px 12px; border-radius: 6px; font-family: monospace; font-size: 13px; margin: 12px 0; }}
  </style>
</head>
<body>
  <div class="card">
    <div class="header">
      <h1>Antigravity Manager · {}</h1>
    </div>
    <div class="body">
      {}
    </div>
    <div class="footer">
      Machine: <strong>{}</strong> &nbsp;|&nbsp; Local IP: <strong>{}</strong><br>
      Automated Mailbox Dispatcher · Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC
    </div>
  </div>
</body>
</html>"#,
        title, content_html, machine_name, machine_ip
    )
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
        "[AGM Alert] Low Quota Warning ({:.1}%) - {}",
        current_quota, email
    );
    let content = format!(
        r#"<p><span class="badge badge-warn">CREDIT THRESHOLD TRIGGER</span></p>
<p>The active profile <strong>{}</strong> has dropped to <strong>{:.1}%</strong> remaining credit, falling below the configured safety threshold of <strong>{}%</strong>.</p>
<p>If auto-profile switcher is enabled, Antigravity Manager will attempt to migrate active workspaces to the next highest credit account.</p>"#,
        email, current_quota, threshold
    );
    let html = wrap_email_card("Low Credit Alert", &content, machine_name, machine_ip);
    (subject, html)
}

/// Render HTML email for automated workspace switch
pub fn render_workspace_switch_email(
    from_instance: &str,
    to_instance: &str,
    reason: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Notice] Workspace Auto-Switched: {}", to_instance);
    let content = format!(
        r#"<p><span class="badge badge-info">WORKSPACE SWITCHED</span></p>
<p>Antigravity Manager transitioned active tasks from <strong>{}</strong> to <strong>{}</strong>.</p>
<p><strong>Reason:</strong> {}</p>
<p>Running prompts and repository states were safely snapshotted to <code>repo_prompts.db</code>.</p>"#,
        from_instance, to_instance, reason
    );
    let html = wrap_email_card(
        "Profile Migration Notice",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, html)
}

/// Render HTML email for idle running projects alert
pub fn render_idle_projects_email(
    projects: &[String],
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = "[AGM Prompt Request] Running Projects Idle - Ready for Instructions".to_string();
    let mut proj_list = String::new();
    for p in projects {
        proj_list.push_str(&format!("<li><strong>{}</strong></li>", p));
    }

    let content = format!(
        r#"<p><span class="badge badge-success">IDLE WORKSPACE SENSOR</span></p>
<p>There are no active prompts currently running in the following active projects:</p>
<ul>{}</ul>
<p>Would you like to send something to these projects? Reply to this email with the format:</p>
<div class="cmd">Subject: Project: &lt;project-name&gt;<br><br>&lt;Your prompt instruction here...&gt;</div>
<p>Antigravity Manager will read your reply and inject the prompt automatically.</p>"#,
        proj_list
    );
    let html = wrap_email_card(
        "Idle Projects Notification",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, html)
}

/// Render HTML email for remote CLI execution results
pub fn render_exec_result_email(
    cmd: &str,
    exit_code: i32,
    output: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Execution Report] exit: {} - {}", exit_code, cmd);
    let status_badge = if exit_code == 0 {
        r#"<span class="badge badge-success">SUCCESS</span>"#
    } else {
        r#"<span class="badge badge-warn">FAILED</span>"#
    };

    let content = format!(
        r#"<p>{} <strong>Command:</strong> <code>{}</code></p>
<p><strong>Exit Code:</strong> {}</p>
<pre style="background: #0f172a; color: #f8fafc; padding: 14px; border-radius: 8px; overflow-x: auto; font-size: 12px;">{}</pre>"#,
        status_badge, cmd, exit_code, output
    );
    let html = wrap_email_card(
        "Command Execution Report",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, html)
}

/// Render HTML email for help cheat sheet
pub fn render_help_email(machine_name: &str, machine_ip: &str) -> (String, String) {
    let subject = "[AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet".to_string();
    let content = format!(
        r#"<p>You can remotely command this Antigravity Manager instance by sending emails matching these subjects:</p>
<table style="width: 100%; border-collapse: collapse; font-size: 13px; margin: 16px 0;">
  <thead>
    <tr style="background: #f1f5f9; text-align: left;">
      <th style="padding: 8px; border: 1px solid #cbd5e1;">Command Subject</th>
      <th style="padding: 8px; border: 1px solid #cbd5e1;">Body Content</th>
      <th style="padding: 8px; border: 1px solid #cbd5e1;">Action Triggered</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>Project: &lt;name&gt;</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Prompt instructions</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Injects prompt into running project workspace</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>exec: {}</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">gitmap status</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Runs command on target IP machine & emails back result</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>instance: new</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Profile name or empty</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Launches isolated Antigravity IDE instance</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>rotate: accounts</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">(Empty)</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Rotates to next highest quota account profile</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>prompt: &lt;name&gt;</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">(Empty or keyword)</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Finds saved prompt by name/keyword and executes it on active workspace</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>status</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">(Empty)</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Reports running projects, active prompt queue, and node telemetry</td>
    </tr>
    <tr>
      <td style="padding: 8px; border: 1px solid #cbd5e1;"><code>help</code></td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">(Empty)</td>
      <td style="padding: 8px; border: 1px solid #cbd5e1;">Returns this command cheat sheet</td>
    </tr>
  </tbody>
</table>"#,
        machine_ip
    );
    let html = wrap_email_card(
        "Remote Instructions Cheat Sheet",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, html)
}

/// Render HTML email for self-test mailbox verification
pub fn render_self_test_email(
    email: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Test] Mailbox Connection Verified - {}", email);
    let content = format!(
        r#"<p><span class="badge badge-success">CONNECTION VERIFIED ✓</span></p>
<p>This is an automated self-test verification email from <strong>Antigravity Manager</strong>.</p>
<p>Your mailbox account <code>{}</code> successfully authenticated via SMTP, passed credentials check, and delivered this verification message to itself.</p>
<p>Remote commands, quota notifications, and failover routing are active for this account.</p>"#,
        email
    );
    let html = wrap_email_card(
        "Mailbox Self-Test Verification",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, html)
}

/// Render HTML email for test ping command verification
pub fn render_test_ping_email(
    project_name: &str,
    machine_name: &str,
    machine_ip: &str,
    timestamp: i64,
) -> (String, String) {
    let subject = format!("[AGM Ping] Test Command Ping: {}", project_name);
    let content = format!(
        r#"<p><span class="badge badge-info">COMMAND TEST PING</span></p>
<p>Test ping dispatched for <strong>{}</strong> (epoch: <code>{}</code>).</p>
<p>Reply to verify remote command handling:</p>
<div class="cmd">Subject: Project: {}<br><br>echo 'Ping verified!'</div>
<p>Adaptive fast polling (5-10s) active.</p>"#,
        project_name, timestamp, project_name
    );
    let html = wrap_email_card("Command Test Ping", &content, machine_name, machine_ip);
    (subject, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let (subj, html) =
            render_quota_drop_email("test@example.com", 12.5, 15, "my-pc", "192.168.1.50");
        assert!(subj.contains("12.5%"));
        assert!(html.contains("192.168.1.50"));
    }
}
