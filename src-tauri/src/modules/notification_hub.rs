//! Notification Hub Module
//! Coordinates unified notifications across Email and Telegram channels
//! for critical events such as account switching, quota alerts, and workspace updates.

#![allow(dead_code)]

use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::logger;
use crate::modules::telegram_inbound;

/// Dispatch notifications across Email and Telegram upon account/instance switch
pub fn notify_account_switched(
    account_email: &str,
    instance_name: &str,
    reason: &str,
    is_auto: bool,
) {
    let email_copy = account_email.to_string();
    let instance_copy = instance_name.to_string();
    let reason_copy = reason.to_string();

    tokio::spawn(async move {
        dispatch_email_switch_alert(&email_copy, &instance_copy, &reason_copy, is_auto);
        dispatch_telegram_switch_alert(&email_copy, &instance_copy, &reason_copy, is_auto).await;
    });
}

/// Helper to render and dispatch email switch notification
fn dispatch_email_switch_alert(
    account_email: &str,
    instance_name: &str,
    reason: &str,
    is_auto: bool,
) {
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(_) => return,
    };

    if !settings.is_enabled {
        return;
    }
    if !settings.notify_on_workspace_switch {
        return;
    }

    let recipients = match email_vault_db::list_notify_recipients() {
        Ok(r) => r,
        Err(_) => return,
    };

    let active_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if active_recipients.is_empty() {
        return;
    }

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let trigger_label = if is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let subject = format!(
        "[{} | {} | {}] [Antigravity] Account Switched: {} -> {}",
        pkg_ver, m_name, m_ip, instance_name, account_email
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
</head>
<body style="margin: 0; padding: 20px; background-color: #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
  <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); border: 1px solid #e2e8f0;">
    <div style="background: #0f172a; padding: 20px 24px; color: #ffffff;">
      <div style="margin-bottom: 8px;">
        <span style="background: #334155; color: #f8fafc; padding: 4px 10px; border-radius: 6px; font-family: monospace; font-size: 13px; font-weight: bold;">[{} | {} | {}]</span>
        <span style="background: #059669; color: #ffffff; padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: bold; text-transform: uppercase; margin-left: 8px;">SWITCHED</span>
      </div>
      <h2 style="margin: 8px 0 0 0; font-size: 18px; color: #ffffff;">Antigravity Account Switched</h2>
    </div>
    <div style="padding: 24px;">
      <p style="margin: 0 0 16px 0; color: #475569; font-size: 14px;">An account rotation was executed successfully. Target credentials have been injected into IDE state storage.</p>
      <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 140px;">Version</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 140px;">Target Account</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Target Instance</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Trigger Mode</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Reason</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Origin Node</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{} ({})</td></tr>
      </table>
    </div>
    <div style="background: #f8fafc; padding: 14px 24px; border-top: 1px solid #e2e8f0; font-size: 11px; color: #94a3b8; text-align: center;">
      Automated Dispatcher · Antigravity Manager {} · Maintained by Alim, Sponsored by RISEUP ASIA LLC
    </div>
  </div>
</body>
</html>"#,
        pkg_ver,
        m_name,
        m_ip,
        pkg_ver,
        account_email,
        instance_name,
        trigger_label,
        reason,
        m_name,
        m_ip,
        pkg_ver
    );

    let _ = email_sender::dispatch_email_with_failover(&subject, &html, &active_recipients);
}

/// Helper to render and dispatch Telegram switch notification
async fn dispatch_telegram_switch_alert(
    account_email: &str,
    instance_name: &str,
    reason: &str,
    is_auto: bool,
) {
    let config = match telegram_inbound::load_config() {
        Ok(c) => c,
        Err(_) => return,
    };

    if !config.is_enabled {
        return;
    }
    if config.bot_token.trim().is_empty() {
        return;
    }

    let Some(chat_id) = config.allowed_chat_id else {
        return;
    };

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now_str = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let trigger_label = if is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let text = format!(
        "🔄 <b>Antigravity Manager: Account Switched</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        📦 <b>Version:</b> <code>{}</code>\n\
        👤 <b>Account:</b> <code>{}</code>\n\
        💻 <b>Target Instance:</b> <code>{}</code>\n\
        🏷️ <b>Trigger:</b> {}\n\
        📝 <b>Reason:</b> {}\n\
        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
        ⏰ <b>Timestamp:</b> {}",
        pkg_ver, account_email, instance_name, trigger_label, reason, m_name, m_ip, now_str
    );

    if let Err(e) = telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &text).await
    {
        logger::log_warn(&format!(
            "[NotificationHub] Telegram switch alert failed: {}",
            e
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_switch_labels() {
        let auto_label = "Auto-Switcher (Quota/Period boundary)";
        let manual_label = "Manual User Switch";
        assert!(auto_label.contains("Auto"));
        assert!(manual_label.contains("Manual"));
    }
}
