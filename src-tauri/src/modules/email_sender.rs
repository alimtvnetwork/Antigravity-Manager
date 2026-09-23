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

/// Build full MIME email message with strictly plaintext formatting (zero HTML)
fn build_mime_message(
    account: &EmailAccount,
    subject: &str,
    body: &str,
    recipients: &[String],
) -> String {
    let msg_id = format!("<{}@{}>", Uuid::new_v4(), account.smtp_host);
    let date = Utc::now().to_rfc2822();
    let encoded_subject = format!("=?UTF-8?B?{}?=", BASE64_STANDARD.encode(subject.as_bytes()));

    format!(
        "From: {} <{}>\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: 8bit\r\nX-Mailer: Antigravity-Manager-Mailer/4.65.0\r\n\r\n{}",
        account.alias,
        account.email,
        recipients.join(", "),
        encoded_subject,
        date,
        msg_id,
        body
    )
}

// ---------------------------------------------------------------------------
// Plaintext Email Templates (Strictly Plaintext, Zero HTML)
// ---------------------------------------------------------------------------

fn wrap_plaintext_email(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    format!(
        "================================================================================\r\n\
         ANTIGRAVITY MANAGER · {}\r\n\
         ================================================================================\r\n\r\n\
         {}\r\n\r\n\
         --------------------------------------------------------------------------------\r\n\
         Node: {} | IP: {}\r\n\
         Automated Dispatcher · Antigravity Manager Tools · Maintained by Alim, Sponsored by RISEUP ASIA LLC\r\n",
        title.to_uppercase(),
        content.trim(),
        machine_name,
        machine_ip
    )
}

/// Render plaintext email for low quota alerts
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

/// Render plaintext email for automated workspace switch
pub fn render_workspace_switch_email(
    from_instance: &str,
    to_instance: &str,
    reason: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Notice] Workspace Auto-Switched: {}", to_instance);
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

/// Render plaintext email for idle running projects alert
pub fn render_idle_projects_email(
    projects: &[String],
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = "[AGM Prompt Request] Running Projects Idle - Ready for Instructions".to_string();
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

/// Render plaintext email for remote CLI execution results
pub fn render_exec_result_email(
    cmd: &str,
    exit_code: i32,
    output: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Execution Report] exit: {} - {}", exit_code, cmd);
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

/// Render plaintext email for help cheat sheet
pub fn render_help_email(machine_name: &str, machine_ip: &str) -> (String, String) {
    let subject = "[AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet".to_string();
    let content = format!(
        "You can remotely command this Antigravity Manager instance by sending emails\r\n\
         matching the following pipe-delimited syntax in the subject:\r\n\r\n\
         Format: sub: [node|ip|ins-X] | proj-{{name}} | <command>\r\n\r\n\
         Supported Commands & Subject Examples:\r\n\r\n\
         1. Inject Prompt to Workspace:\r\n\
            Subject: sub: {} | proj-my-project | prompt\r\n\
            Body:    <Your prompt instruction here...>\r\n\r\n\
         2. Execute Command on Target Machine:\r\n\
            Subject: sub: {} | exec: gitmap status\r\n\
            Body:    (Optional command arguments)\r\n\r\n\
         3. Launch New Sandbox Instance:\r\n\
            Subject: sub: {} | instance: new\r\n\
            Body:    <profile-name>\r\n\r\n\
         4. Rotate to Next Highest Quota Account:\r\n\
            Subject: sub: {} | rotate: accounts\r\n\r\n\
         5. Query Status Telemetry & Health:\r\n\
            Subject: sub: {} | status\r\n\r\n\
         6. Help & Cheat Sheet:\r\n\
            Subject: sub: {} | help",
        machine_name, machine_ip, machine_name, machine_name, machine_name, machine_name
    );
    let body = wrap_plaintext_email(
        "Remote Instructions Cheat Sheet",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render plaintext email for self-test mailbox verification
pub fn render_self_test_email(
    email: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let subject = format!("[AGM Test] Mailbox Connection Verified - {}", email);
    let content = format!(
        "[PASS] CONNECTION VERIFIED\r\n\r\n\
         This is an automated self-test verification email from Antigravity Manager.\r\n\r\n\
         Your mailbox account '{}' successfully authenticated via SMTP, passed credentials\r\n\
         verification, and delivered this plaintext verification message.\r\n\r\n\
         Remote commands, quota notifications, and failover routing are active for this account.",
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

/// Render plaintext email for test ping command verification
pub fn render_test_ping_email(
    project_name: &str,
    machine_name: &str,
    machine_ip: &str,
    timestamp: i64,
) -> (String, String) {
    let subject = format!("[AGM Ping] Test Command Ping: {}", project_name);
    let content = format!(
        "[*] COMMAND TEST PING\r\n\r\n\
         Test ping dispatched for '{}' (epoch: {}).\r\n\r\n\
         Reply to this message with a prompt to verify remote execution:\r\n\
         Subject: sub: {} | proj-{}\r\n\r\n\
         echo 'Ping verified!'",
        project_name, timestamp, machine_name, project_name
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
        assert!(text.contains("192.168.1.50"));
        assert!(!text.contains("<html"));
        assert!(!text.contains("<div"));
    }
}
