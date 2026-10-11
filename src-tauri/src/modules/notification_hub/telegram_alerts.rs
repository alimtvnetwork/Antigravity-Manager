use crate::modules::email_watcher;
use crate::modules::telegram_inbound;

use super::*;

/// Helper to render and dispatch Telegram switch notification
pub(crate) async fn dispatch_telegram_switch_alert(
    details: &SwitchNotificationDetails,
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

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now_str = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let trigger_label = if details.is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let selected_clean = details.selected_email.trim();
    let from_display = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("default")
                && !s.eq_ignore_ascii_case(selected_clean)
        })
        .unwrap_or("(none / standby)");

    let prev_4h_str = details
        .previous_quota_4h
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());
    let prev_weekly_str = details
        .previous_quota_weekly
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());

    let target_4h_str = details
        .target_quota_4h
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());
    let target_weekly_str = details
        .target_quota_weekly
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());

    let threshold_display = details
        .threshold_activated
        .map(|t| format!("{:.1}%", t))
        .unwrap_or_else(|| "-".to_string());

    let raw_projects = if !details.backed_up_projects.is_empty() {
        deduplicate_names(&details.backed_up_projects)
    } else {
        match crate::modules::repo_db::list_running_projects() {
            Ok(projs) if !projs.is_empty() => {
                let names = deduplicate_names(projs.into_iter().map(|p| p.repo_name));
                if !names.is_empty() {
                    names
                } else {
                    vec!["Antigravity-Manager".to_string()]
                }
            }
            _ => vec!["Antigravity-Manager".to_string()],
        }
    };
    let projects_display = raw_projects.join(", ");

    let backup_stats = match (
        details.backed_up_prompts_count,
        details.restored_prompts_count,
    ) {
        (Some(b), Some(r)) => format!("{} captured | {} re-injected", b, r),
        (Some(b), None) => format!("{} captured", b),
        (None, Some(r)) => format!("{} re-injected", r),
        (None, None) => "preserved (SQLite split db)".to_string(),
    };

    let (prompt_preview_text, prompt_words_count) =
        if let Ok(all_prompts) = crate::modules::repo_db::list_all_prompts() {
            if let Some(p) = all_prompts.into_iter().find(|p| {
                p.status == "running" || p.status == "backed_up" || p.status == "dispatched"
            }) {
                extract_words_preview(&p.prompt_content, 200)
            } else {
                (String::new(), 0)
            }
        } else {
            (String::new(), 0)
        };

    let prompt_section = if !prompt_preview_text.is_empty() {
        format!(
            "\n💬 <b>Active Prompt:</b> ({} words)\n<code>{}</code>\n",
            prompt_words_count,
            crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500)
        )
    } else {
        String::new()
    };

    let text = format!(
        "🔄 <b>Antigravity Manager: Account Switched</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        📦 <b>Version:</b> <code>{}</code>\n\
        👤 <b>Previous Account:</b> <code>{}</code>\n\
        📉 <b>Previous Balances:</b> <code>4h: {} | Weekly: {}</code>\n\
        ✅ <b>Selected Account:</b> <code>{}</code>\n\
        📈 <b>Target Balances:</b> <code>4h: {} | Weekly: {}</code>\n\
        ⚙️ <b>Threshold:</b> <code>{}</code>\n\
        💻 <b>Target Instance:</b> <code>{}</code> ({})\n\
        📁 <b>Projects:</b> <code>{}</code>\n\
        💾 <b>Prompt Backup:</b> <code>{}</code>\n\
        🏷️ <b>Trigger:</b> {}\n\
        📝 <b>Reason:</b> {}\n\
        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
        ⏰ <b>Timestamp:</b> {}{}",
        escape_telegram_html(&pkg_ver),
        escape_telegram_html(&from_display),
        escape_telegram_html(&prev_4h_str),
        escape_telegram_html(&prev_weekly_str),
        escape_telegram_html(&details.selected_email),
        escape_telegram_html(&target_4h_str),
        escape_telegram_html(&target_weekly_str),
        escape_telegram_html(&threshold_display),
        escape_telegram_html(&details.instance_name),
        escape_telegram_html(&details.instance_id),
        escape_telegram_html(&projects_display),
        escape_telegram_html(&backup_stats),
        escape_telegram_html(&trigger_label),
        escape_telegram_html(&details.reason),
        escape_telegram_html(&m_name),
        escape_telegram_html(&m_ip),
        escape_telegram_html(&now_str),
        prompt_section
    );

    match telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &text).await {
        Ok(()) => Ok(format!("sent to chat {}", chat_id)),
        Err(e) => Err(redact_secrets(&e.to_string())),
    }
}
