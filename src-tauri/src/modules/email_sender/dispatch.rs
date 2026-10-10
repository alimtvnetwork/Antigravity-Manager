use crate::modules::email_vault_db::{self, EmailAccount};
use base64::prelude::*;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::*;

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
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(send_smtp_cmd(&mut stream, "QUIT", false), "send_smtp_cmd");
    Ok(())
}
