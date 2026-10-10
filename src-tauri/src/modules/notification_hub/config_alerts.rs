use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;

use super::*;

/// Dispatch self-notification email when an email account or recipient is added/updated
pub fn notify_email_config_added(title: &str, details: serde_json::Value) {
    let title_str = title.to_string();
    tauri::async_runtime::spawn(async move {
        dispatch_email_config_added_alert(&title_str, details);
    });
}

pub(crate) fn dispatch_email_config_added_alert(title: &str, details: serde_json::Value) {
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
        if let Some(new_email) = details.get("email").and_then(|v| v.as_str()) {
            if !new_email.trim().is_empty() {
                target_recipients.push(new_email.trim().to_string());
            }
        }
    }

    if target_recipients.is_empty() {
        return;
    }

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let subject = format!(
        "[AGM {} | {} | {}] [JSON] Email Config Added: {}",
        pkg_ver, m_name, m_ip, title
    );

    let json_pretty = serde_json::to_string_pretty(&details).unwrap_or_default();
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_email_with_failover(&subject, &json_pretty, &target_recipients),
        "dispatch_email_with_failover",
    );
}
