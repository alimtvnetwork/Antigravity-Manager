//! Idle Mode Notification Hub
//! Handles automated notifications across Telegram and Email channels
//! when Antigravity IDE instances complete active prompt tasks and enter idle state.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use super::state::escape_telegram_html;
use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::telegram_inbound;

/// 600-second (10-minute) deduplication cooldown tracker per instance identifier
static LAST_IDLE_ALERT_PER_INSTANCE: Lazy<Mutex<HashMap<String, i64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Clear all idle alert cooldowns (primarily used for unit testing)
pub fn clear_idle_alert_cooldowns() {
    if let Ok(mut guard) = LAST_IDLE_ALERT_PER_INSTANCE.lock() {
        guard.clear();
    }
}

/// Dispatches an idle mode notification alert across Telegram and Email channels.
///
/// Dispatches rich notification to Telegram using `telegram_inbound::send_telegram_message_with_markup`
/// when Telegram bot is configured and enabled, and dispatches Email notification using
/// `email_sender::render_idle_projects_email` and `email_sender::dispatch_email_with_failover`.
///
/// Implements a 600-second cooldown per `instance_id` to prevent repeated alerts during continuous idle.
pub async fn dispatch_idle_mode_alert(
    instance_id: &str,
    instance_name: &str,
    idle_duration_secs: u64,
) -> Result<String, String> {
    let norm_instance_id = if instance_id.trim().is_empty() {
        "default".to_string()
    } else {
        instance_id.trim().to_lowercase()
    };

    let now = chrono::Utc::now().timestamp();

    // 1. Cooldown check: 600-second cooldown per instance_id
    {
        let mut guard = LAST_IDLE_ALERT_PER_INSTANCE
            .lock()
            .map_err(|e| format!("cooldown lock poisoned: {}", e))?;
        if let Some(&last_ts) = guard.get(&norm_instance_id) {
            let elapsed = now - last_ts;
            if elapsed < 600 {
                return Err(format!(
                    "cooldown active: instance '{}' alerted {}s ago (cooldown 600s)",
                    norm_instance_id, elapsed
                ));
            }
        }
        guard.insert(norm_instance_id.clone(), now);
    }

    // 2. Dispatch Telegram alert
    let tg_res = dispatch_telegram_idle_alert(instance_id, instance_name, idle_duration_secs).await;

    // 3. Dispatch Email alert
    let email_res = dispatch_email_idle_alert(instance_id, instance_name);

    // 4. If neither channel succeeded, rollback the cooldown entry so future attempts aren't blocked
    if tg_res.is_err() && email_res.is_err() {
        if let Ok(mut guard) = LAST_IDLE_ALERT_PER_INSTANCE.lock() {
            guard.remove(&norm_instance_id);
        }
    }

    // 5. Evaluate combined results
    match (tg_res, email_res) {
        (Ok(tg_msg), Ok(email_msg)) => {
            let summary = format!(
                "Idle alert dispatched: tg [{}], email [{}]",
                tg_msg, email_msg
            );
            crate::modules::logger::log_info(&format!("[Notify] {}", summary));
            Ok(summary)
        }
        (Ok(tg_msg), Err(email_err)) => {
            let summary = format!(
                "Idle alert dispatched: tg [{}] (email skipped/failed: {})",
                tg_msg, email_err
            );
            crate::modules::logger::log_info(&format!("[Notify] {}", summary));
            Ok(summary)
        }
        (Err(tg_err), Ok(email_msg)) => {
            let summary = format!(
                "Idle alert dispatched: email [{}] (telegram skipped/failed: {})",
                email_msg, tg_err
            );
            crate::modules::logger::log_info(&format!("[Notify] {}", summary));
            Ok(summary)
        }
        (Err(tg_err), Err(email_err)) => {
            let err_summary = format!(
                "Failed to dispatch idle alert across channels: telegram: {}, email: {}",
                tg_err, email_err
            );
            crate::modules::logger::log_warn(&format!("[Notify] {}", err_summary));
            Err(err_summary)
        }
    }
}

/// Helper to render and dispatch Telegram idle notification
pub async fn dispatch_telegram_idle_alert(
    instance_id: &str,
    instance_name: &str,
    idle_duration_secs: u64,
) -> Result<String, String> {
    let config = match telegram_inbound::load_config() {
        Ok(c) => c,
        Err(e) => return Err(format!("telegram config unreadable: {}", e)),
    };

    if !config.is_enabled {
        return Err("telegram bot is disabled".to_string());
    }
    if config.bot_token.trim().is_empty() {
        return Err("telegram bot token is empty".to_string());
    }

    let Some(chat_id) = config.allowed_chat_id else {
        return Err("telegram chat id is not set".to_string());
    };

    let pkg_ver = env!("CARGO_PKG_VERSION");
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let ip_display = if m_ip.trim().is_empty() || m_ip == "Unknown IP" {
        "127.0.0.1"
    } else {
        m_ip.as_str()
    };

    let display_instance = if !instance_name.trim().is_empty() {
        instance_name.trim()
    } else if !instance_id.trim().is_empty() {
        instance_id.trim()
    } else {
        "Default"
    };

    let text = format!(
        "💤 <b>Antigravity Manager · IDE Idle Notification</b>\n\
         🖥️ <b>Machine:</b> <b>{}</b> [<code>{}</code>]\n\
         🏷️ <b>Instance:</b> <b>{}</b>\n\
         ━━━━━━━━━━━━━━━━━━━━━━━━\n\
         ✨ <b>Status:</b> All prompt tasks completed. IDE is sitting idle and ready for new instructions.\n\
         ⏳ <b>Idle Time:</b> {}s\n\
         💡 <b>Action:</b> Send prompts via Telegram (<code>/send &lt;prompt&gt;</code>) or Antigravity Manager to start working!\n\
         ━━━━━━━━━━━━━━━━━━━━━━━━\n\
         <i>Antigravity Manager v{}</i>",
        escape_telegram_html(&m_name),
        escape_telegram_html(ip_display),
        escape_telegram_html(display_instance),
        idle_duration_secs,
        escape_telegram_html(pkg_ver),
    );

    let reply_markup = Some(serde_json::json!({
        "inline_keyboard": [
            [
                {
                    "text": "💬 /send Instructions",
                    "switch_inline_query_current_chat": "/send "
                },
                {
                    "text": "📋 Copy /send",
                    "copy_text": {
                        "text": "/send "
                    }
                }
            ]
        ]
    }));

    match telegram_inbound::send_telegram_message_with_markup(
        &config.bot_token,
        chat_id,
        &text,
        reply_markup,
        false,
    )
    .await
    {
        Ok(()) => Ok(format!("sent to chat {}", chat_id)),
        Err(e) => Err(format!("telegram error: {}", e)),
    }
}

/// Helper to render and dispatch Email idle notification
pub fn dispatch_email_idle_alert(instance_id: &str, instance_name: &str) -> Result<String, String> {
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(e) => return Err(format!("email settings unreadable: {}", e)),
    };

    if !settings.is_enabled {
        return Err("email notifications are disabled".to_string());
    }
    if !settings.notify_on_idle_workspace {
        return Err("idle-workspace email is disabled".to_string());
    }

    let recipients = match email_vault_db::list_notify_recipients() {
        Ok(r) => r,
        Err(e) => return Err(format!("email recipients unreadable: {}", e)),
    };

    let active_recipients: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();

    if active_recipients.is_empty() {
        return Err("no active email recipients".to_string());
    }

    let live_projects = crate::modules::repo_db::get_live_project_execution_info();
    let mut idle_projects: Vec<_> = live_projects.into_iter().filter(|p| p.is_idle).collect();

    let norm_id = instance_id.trim().to_lowercase();
    if !norm_id.is_empty() && norm_id != "all" && norm_id != "default" {
        let matched: Vec<_> = idle_projects
            .iter()
            .filter(|p| p.project_id.to_lowercase().contains(&norm_id))
            .cloned()
            .collect();
        if !matched.is_empty() {
            idle_projects = matched;
        }
    }

    let now = chrono::Utc::now().timestamp();
    if idle_projects.is_empty() {
        let display_name = if !instance_name.trim().is_empty() {
            instance_name.trim()
        } else if !instance_id.trim().is_empty() {
            instance_id.trim()
        } else {
            "Default"
        };
        idle_projects.push(crate::modules::repo_db::ProjectExecutionInfo {
            project_id: format!("{}__{}", display_name, instance_id),
            repo_name: display_name.to_string(),
            repo_path: "-".to_string(),
            is_running: false,
            is_idle: true,
            status: "IDLE / READY".to_string(),
            active_prompt: None,
            last_detected_at: now,
        });
    }

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let (subj, html) = email_sender::render_idle_projects_email(&idle_projects, &m_name, &m_ip);

    match email_sender::dispatch_email_with_failover(&subj, &html, &active_recipients) {
        Ok(send_res) => Ok(format!(
            "sent to {} recipients ({:?})",
            active_recipients.len(),
            send_res
        )),
        Err(e) => Err(format!("email send error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_alert_formatting_components() {
        let pkg_ver = env!("CARGO_PKG_VERSION");
        let m_name = "test-machine";
        let ip_display = "192.168.1.100";
        let display_instance = "Dev-Workspace";
        let idle_duration_secs = 240;

        let text = format!(
            "💤 <b>Antigravity Manager · IDE Idle Notification</b>\n\
             🖥️ <b>Machine:</b> <b>{}</b> [<code>{}</code>]\n\
             🏷️ <b>Instance:</b> <b>{}</b>\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━\n\
             ✨ <b>Status:</b> All prompt tasks completed. IDE is sitting idle and ready for new instructions.\n\
             ⏳ <b>Idle Time:</b> {}s\n\
             💡 <b>Action:</b> Send prompts via Telegram (<code>/send &lt;prompt&gt;</code>) or Antigravity Manager to start working!\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━\n\
             <i>Antigravity Manager v{}</i>",
            escape_telegram_html(m_name),
            escape_telegram_html(ip_display),
            escape_telegram_html(display_instance),
            idle_duration_secs,
            escape_telegram_html(pkg_ver),
        );

        assert!(text.contains("💤 <b>Antigravity Manager · IDE Idle Notification</b>"));
        assert!(text.contains("test-machine"));
        assert!(text.contains("192.168.1.100"));
        assert!(text.contains("Dev-Workspace"));
        assert!(text.contains("240s"));
        assert!(text.contains(&format!("<i>Antigravity Manager v{}</i>", pkg_ver)));
    }

    #[tokio::test]
    async fn test_idle_alert_cooldown() {
        clear_idle_alert_cooldowns();
        let test_inst = "unit-test-instance-1";

        // Manually insert a recent cooldown timestamp
        let now = chrono::Utc::now().timestamp();
        {
            let mut guard = LAST_IDLE_ALERT_PER_INSTANCE.lock().unwrap();
            guard.insert(test_inst.to_string(), now - 100); // 100s ago (< 600s)
        }

        let res = dispatch_idle_mode_alert(test_inst, "Unit Test", 300).await;
        assert!(res.is_err(), "Must be on cooldown");
        let err_str = res.unwrap_err();
        assert!(
            err_str.contains("cooldown active"),
            "Error must mention cooldown active: {}",
            err_str
        );

        // After clearing, it should not be on cooldown
        clear_idle_alert_cooldowns();
        {
            let guard = LAST_IDLE_ALERT_PER_INSTANCE.lock().unwrap();
            assert!(!guard.contains_key(test_inst));
        }
    }

    #[test]
    fn test_idle_alert_inline_keyboard() {
        let reply_markup = serde_json::json!({
            "inline_keyboard": [
                [
                    {
                        "text": "💬 /send Instructions",
                        "switch_inline_query_current_chat": "/send "
                    },
                    {
                        "text": "📋 Copy /send",
                        "copy_text": {
                            "text": "/send "
                        }
                    }
                ]
            ]
        });

        let rows = reply_markup["inline_keyboard"]
            .as_array()
            .expect("must be array");
        assert_eq!(rows.len(), 1);
        let buttons = rows[0].as_array().expect("must be array");
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0]["switch_inline_query_current_chat"], "/send ");
        assert_eq!(buttons[1]["copy_text"]["text"], "/send ");
    }
}
