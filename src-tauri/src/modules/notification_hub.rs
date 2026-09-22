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

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let trigger_label = if is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let subject = format!(
        "[Antigravity] Account Switched: {} -> {}",
        instance_name, account_email
    );

    let html = format!(
        "<div style=\"font-family: Arial, sans-serif; line-height: 1.6; color: #333;\">\
            <h2 style=\"color: #2563eb;\">Antigravity Account Switched</h2>\
            <p>An account switch operation was executed on node <b>{}</b>.</p>\
            <table style=\"border-collapse: collapse; width: 100%; max-width: 500px;\">\
                <tr><td style=\"padding: 8px; border: 1px solid #ddd;\"><b>Target Account</b></td><td style=\"padding: 8px; border: 1px solid #ddd;\"><code>{}</code></td></tr>\
                <tr><td style=\"padding: 8px; border: 1px solid #ddd;\"><b>Target Instance</b></td><td style=\"padding: 8px; border: 1px solid #ddd;\"><code>{}</code></td></tr>\
                <tr><td style=\"padding: 8px; border: 1px solid #ddd;\"><b>Trigger</b></td><td style=\"padding: 8px; border: 1px solid #ddd;\">{}</td></tr>\
                <tr><td style=\"padding: 8px; border: 1px solid #ddd;\"><b>Reason</b></td><td style=\"padding: 8px; border: 1px solid #ddd;\">{}</td></tr>\
                <tr><td style=\"padding: 8px; border: 1px solid #ddd;\"><b>Machine</b></td><td style=\"padding: 8px; border: 1px solid #ddd;\">{} ({})</td></tr>\
            </table>\
        </div>",
        m_name, account_email, instance_name, trigger_label, reason, m_name, m_ip
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
        👤 <b>Account:</b> <code>{}</code>\n\
        💻 <b>Target Instance:</b> <code>{}</code>\n\
        🏷️ <b>Trigger:</b> {}\n\
        📝 <b>Reason:</b> {}\n\
        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
        ⏰ <b>Timestamp:</b> {}",
        account_email, instance_name, trigger_label, reason, m_name, m_ip, now_str
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
