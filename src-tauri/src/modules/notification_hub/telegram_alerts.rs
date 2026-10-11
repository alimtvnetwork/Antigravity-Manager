use std::path::Path;
use std::time::Duration;

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

    // Extract active running prompt and any associated images for this instance
    let (prompt_preview_text, prompt_words_count, full_prompt_content, detected_images) =
        if let Ok(all_prompts) = crate::modules::repo_db::list_all_prompts() {
            let matched_prompt = all_prompts.into_iter().find(|p| {
                (p.status == "running" || p.status == "backed_up" || p.status == "dispatched")
                    && (p.instance_id.eq_ignore_ascii_case(&details.instance_name)
                        || p.instance_id.eq_ignore_ascii_case(&details.instance_id)
                        || (details.instance_id == "default"
                            && (p.instance_id.is_empty() || p.instance_id == "default")))
            });

            if let Some(p) = matched_prompt {
                let (_, images) =
                    crate::modules::repo_db::extract_image_payload_or_path(&p.prompt_content);
                let (preview, words) = extract_words_preview(&p.prompt_content, 200);
                (preview, words, p.prompt_content, images)
            } else {
                (String::new(), 0, String::new(), Vec::new())
            }
        } else {
            (String::new(), 0, String::new(), Vec::new())
        };

    // Resolve or upload detected images to free CDN
    let mut resolved_image_urls: Vec<String> = Vec::new();
    for img_ref in &detected_images {
        let trimmed = img_ref.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            resolved_image_urls.push(trimmed.to_string());
        } else {
            let path = Path::new(trimmed);
            if path.exists() {
                // Attempt cached lookup or upload with short timeout
                if let Ok(url_res) = tokio::time::timeout(
                    Duration::from_secs(3),
                    telegram_inbound::upload_local_image_file(path),
                )
                .await
                {
                    if let Ok(public_url) = url_res {
                        resolved_image_urls.push(public_url);
                    }
                }
            }
        }
    }

    let attached_images_section = if !resolved_image_urls.is_empty() {
        let mut links = Vec::new();
        for (idx, url) in resolved_image_urls.iter().enumerate() {
            links.push(format!("<a href=\"{}\">🖼️ Image #{}</a>", url, idx + 1));
        }
        format!(
            "\n🖼️ <b>Attached Media ({}):</b> {}\n",
            resolved_image_urls.len(),
            links.join(" &bull; ")
        )
    } else {
        String::new()
    };

    let telemetry_banner = format!(
        "🖥️ <b>Machine:</b> <code>{}</code> (<code>{}</code>) | 🏷️ <b>Instance:</b> <code>{}</code> | 👤 <b>Bound:</b> <code>{}</code>",
        escape_telegram_html(&m_name),
        escape_telegram_html(&m_ip),
        escape_telegram_html(&details.instance_name),
        escape_telegram_html(&details.selected_email)
    );

    let prompt_section = if !prompt_preview_text.is_empty() {
        format!(
            "\n<blockquote expandable><b>💬 Active Prompt ({} words):</b>\n<code>{}</code></blockquote>\n",
            prompt_words_count,
            crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500)
        )
    } else {
        String::new()
    };

    let text = format!(
        "🔄 <b>Antigravity Manager: Account Rotation</b> ⚡\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
        {}\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
        <b>📊 QUOTA DELTA &amp; ROTATION STATUS</b>\n\
        <blockquote>\
        • <b>Previous Account:</b> <code>{}</code>\n\
        &nbsp;&nbsp;└ <i>Balances:</i> <code>4h: {} | Weekly: {}</code>\n\
        • <b>Selected Account:</b> <code>{}</code>\n\
        &nbsp;&nbsp;└ <i>Balances:</i> <code>4h: {} | Weekly: {}</code>\n\
        • <b>Threshold Guard:</b> <code>{}</code> (Trigger: <i>{}</i>)\
        </blockquote>\n\n\
        <b>📁 WORKSPACE &amp; SYSTEM CONTEXT</b>\n\
        <blockquote>\
        • <b>Projects:</b> <code>{}</code>\n\
        • <b>Prompt Vault:</b> <code>{}</code>\n\
        • <b>Event Reason:</b> <i>{}</i>\n\
        • <b>Version &amp; Timestamp:</b> <code>{}</code> | <code>{}</code>\
        </blockquote>{}{}",
        telemetry_banner,
        escape_telegram_html(&from_display),
        escape_telegram_html(&prev_4h_str),
        escape_telegram_html(&prev_weekly_str),
        escape_telegram_html(&details.selected_email),
        escape_telegram_html(&target_4h_str),
        escape_telegram_html(&target_weekly_str),
        escape_telegram_html(&threshold_display),
        escape_telegram_html(&trigger_label),
        escape_telegram_html(&projects_display),
        escape_telegram_html(&backup_stats),
        escape_telegram_html(&details.reason),
        escape_telegram_html(&pkg_ver),
        escape_telegram_html(&now_str),
        attached_images_section,
        prompt_section
    );

    let reply_markup = if !full_prompt_content.trim().is_empty() || !resolved_image_urls.is_empty()
    {
        Some(telegram_inbound::build_copy_prompt_reply_markup(
            &full_prompt_content,
            &resolved_image_urls,
            None,
        ))
    } else {
        None
    };

    match telegram_inbound::send_telegram_message_with_markup(
        &config.bot_token,
        chat_id,
        &text,
        reply_markup,
    )
    .await
    {
        Ok(()) => Ok(format!("sent to chat {}", chat_id)),
        Err(e) => Err(redact_secrets(&e.to_string())),
    }
}
