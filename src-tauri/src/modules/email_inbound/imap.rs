use crate::modules::email_sender::{self, EmailStream};
use crate::modules::email_vault_db::{self, EmailAccount, EmailInboundAuditLog};
use base64::prelude::*;
use chrono::Utc;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use uuid::Uuid;

use std::io::{Read, Write};

use super::*;

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

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(read_imap_greeting(&mut stream), "read_imap_greeting");

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
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        read_imap_tagged_response(&mut stream, "A02"),
        "read_imap_tagged_response",
    );

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

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        send_imap_cmd(&mut stream, "A99", "LOGOUT", false),
        "send_imap_cmd",
    );
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        read_imap_tagged_response(&mut stream, "A99"),
        "read_imap_tagged_response",
    );
    Ok(messages)
}

pub(crate) fn send_imap_cmd(
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

pub(crate) fn read_imap_greeting(stream: &mut EmailStream) -> Result<String, String> {
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

pub(crate) fn read_imap_tagged_response(
    stream: &mut EmailStream,
    tag: &str,
) -> Result<String, String> {
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

pub(crate) fn parse_raw_fetch_response(resp: &str) -> Option<RawEmailMessage> {
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
        let is_json_switch = (msg.subject.contains("[JSON]")
            && msg.subject.contains("Account Switched"))
            || msg.subject.contains("IN-USE");
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

        let vm_name = val
            .get("vm_name")
            .or_else(|| val.get("node_name"))
            .or_else(|| val.get("hostname"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let local_ip = val
            .get("local_ip")
            .or_else(|| val.get("node_ip"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let timestamp = val
            .get("timestamp")
            .or_else(|| val.get("claimed_at_unix"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let lease_expires = val
            .get("lease_expires_at_unix")
            .or_else(|| val.get("lease_expires_at"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let new_email = val
            .get("claimed_account")
            .or_else(|| val.get("new_email"))
            .or_else(|| val.get("selected_email"))
            .or_else(|| val.get("target_account"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let is_same_machine = vm_name == my_vm && local_ip == my_ip;
        let is_recent =
            (timestamp > 0 && timestamp >= cutoff) || (lease_expires > 0 && lease_expires > now);

        if !is_same_machine && is_recent && !new_email.is_empty() {
            let email_clean = new_email.trim().to_string();
            if !excluded.contains(&email_clean) {
                excluded.push(email_clean);
            }
        }
    }

    excluded
}
