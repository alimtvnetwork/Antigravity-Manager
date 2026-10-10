use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::logger;
use crate::modules::telegram_inbound;

use super::*;

/// Dispatch rich update notifications across Email and Telegram when the system is updated
pub fn notify_system_updated(previous_version: &str, current_version: &str, details: Option<&str>) {
    let prev = previous_version.to_string();
    let curr = current_version.to_string();
    let det = details.map(|s| s.to_string());

    tauri::async_runtime::spawn(async move {
        dispatch_email_update_alert(&prev, &curr, det.as_deref());
        dispatch_telegram_update_alert(&prev, &curr, det.as_deref()).await;
    });
}

/// Helper to render and dispatch email system update notification
pub(crate) fn dispatch_email_update_alert(
    previous_version: &str,
    current_version: &str,
    details: Option<&str>,
) {
    let update_settings =
        crate::modules::update_checker::load_update_settings().unwrap_or_default();
    if !update_settings.notify_on_update || !update_settings.notify_via_email {
        return;
    }

    let email_settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(_) => return,
    };

    if !email_settings.is_enabled || !email_settings.notify_on_system_update {
        return;
    }

    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    let mut target_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if target_recipients.is_empty() {
        if let Ok(Some(default_acc)) = email_vault_db::get_default_account() {
            target_recipients.push(default_acc.email);
        }
    }

    if target_recipients.is_empty() {
        return;
    }

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let (subject, body) = email_sender::render_system_update_email(
        previous_version,
        current_version,
        details,
        &m_name,
        &m_ip,
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_email_with_failover(&subject, &body, &target_recipients),
        "dispatch_email_with_failover",
    );
}

/// Helper to render and dispatch Telegram system update notification
async fn dispatch_telegram_update_alert(
    previous_version: &str,
    current_version: &str,
    details: Option<&str>,
) {
    let update_settings =
        crate::modules::update_checker::load_update_settings().unwrap_or_default();
    if !update_settings.notify_on_update || !update_settings.notify_via_telegram {
        return;
    }

    let config = match telegram_inbound::load_config() {
        Ok(c) => c,
        Err(_) => return,
    };

    if !config.is_enabled || !config.notify_on_system_update {
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

    let status_notes = details.unwrap_or("System update successfully applied.");

    let text = format!(
        "🚀 <b>Antigravity Manager: System Updated!</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        📦 <b>New Version:</b> <code>v{}</code>\n\
        ⏮️ <b>Previous Version:</b> <code>v{}</code>\n\
        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
        ⏰ <b>Timestamp:</b> {}\n\
        📝 <b>Status:</b> {}\n\
        🔗 <a href=\"https://github.com/alimtvnetwork/Antigravity-Manager/releases\">Release Notes &amp; Changelog</a>",
        current_version,
        previous_version,
        m_name,
        m_ip,
        now_str,
        status_notes
    );

    if let Err(e) = telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &text).await
    {
        logger::log_warn(&format!(
            "[NotificationHub] Telegram update alert failed: {}",
            e
        ));
    }
}

/// Startup hook: inspect version changes and notify across configured channels if an update occurred
pub fn check_and_notify_system_updated() {
    let current_ver = env!("CARGO_PKG_VERSION");
    let mut update_settings = match crate::modules::update_checker::load_update_settings() {
        Ok(s) => s,
        Err(_) => return,
    };

    let prev_ver = update_settings.last_known_version.trim().to_string();

    if prev_ver.is_empty() {
        // Initial run with version tracking enabled -> record current version without sending spurious alert
        update_settings.last_known_version = current_ver.to_string();
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::update_checker::save_update_settings(&update_settings),
            "save_update_settings",
        );
        return;
    }

    if prev_ver != current_ver {
        // Version changed! An update occurred!
        logger::log_info(&format!(
            "[NotificationHub] System update detected on startup: v{} -> v{}",
            prev_ver, current_ver
        ));

        update_settings.last_known_version = current_ver.to_string();
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::update_checker::save_update_settings(&update_settings),
            "save_update_settings",
        );

        notify_system_updated(
            &prev_ver,
            current_ver,
            Some("System update installed and launched successfully."),
        );
    }
}
