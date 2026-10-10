use crate::modules::email_vault_db::{self, EmailAccount};
use crate::modules::*;
use base64::prelude::*;
use chrono::Utc;
use uuid::Uuid;

use super::*;

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
  a {{ color: #0284c7 !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #0369a1 !important; }}
  a:hover {{ color: #0284c7 !important; }}
  .btn-link, td a.btn, p a.btn {{ background: #0284c7; color: #ffffff !important; padding: 4px 10px; border-radius: 6px; text-decoration: none; display: inline-block; font-size: 14px; font-weight: bold; border: 1px solid #0369a1; }}
  .btn-link:hover, td a.btn:hover, p a.btn:hover {{ background: #0369a1; color: #ffffff !important; }}
  @media (prefers-color-scheme: dark) {{
    body {{ background-color: #0f172a !important; color: #f1f5f9 !important; }}
    td {{ color: #e2e8f0 !important; }}
    a {{ color: #f8fafc !important; }}
    a:visited {{ color: #e2e8f0 !important; }}
  }}
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
  <div style="max-width: 680px; margin: 0 auto; background: #ffffff; border-radius: 14px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 4px solid #38bdf8; padding: 24px 28px; color: #ffffff;">
      <div style="margin-bottom: 12px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #f8fafc; padding: 5px 12px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 13px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[{} | {} | {}]</span>
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
pub(crate) fn build_mime_message(
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

    let is_json_email =
        normalized_subject.contains("[JSON]") || normalized_subject.contains("[json]");

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
                let formatted_id =
                    if clean_reply_id.starts_with('<') && clean_reply_id.ends_with('>') {
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
  a {{ color: #ffffff !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #ffffff !important; }}
  a:hover {{ color: #e0f2fe !important; }}
  td a, p a {{ background: #2563eb; color: #ffffff !important; padding: 2px 8px; border-radius: 6px; text-decoration: none; display: inline-block; font-size: 14px; font-weight: bold; }}
  td a:hover, p a:hover {{ background: #1d4ed8; color: #ffffff !important; }}
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
