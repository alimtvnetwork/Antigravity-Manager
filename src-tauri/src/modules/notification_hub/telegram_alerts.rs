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
    let selected_masked = crate::modules::logger::mask_email(&details.selected_email);
    let from_display = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("default")
                && !s.eq_ignore_ascii_case(selected_clean)
        })
        .map(|s| crate::modules::logger::mask_email(s))
        .unwrap_or_else(|| "(none / standby)".to_string());

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

    let meaningful_projects: Vec<String> = {
        let filtered: Vec<String> = raw_projects
            .iter()
            .filter(|p| {
                !p.eq_ignore_ascii_case("unnamed-project")
                    && !p.eq_ignore_ascii_case("project")
                    && !p.trim().is_empty()
            })
            .cloned()
            .collect();
        if filtered.is_empty() {
            raw_projects
        } else {
            filtered
        }
    };
    let total_projects = meaningful_projects.len();
    let projects_formatted = if total_projects == 0 {
        "<b>Antigravity-Manager</b>".to_string()
    } else if total_projects <= 3 {
        meaningful_projects
            .iter()
            .map(|p| format!("<b>{}</b>", escape_telegram_html(p)))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        let top3 = meaningful_projects[..3]
            .iter()
            .map(|p| format!("<b>{}</b>", escape_telegram_html(p)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} <i>(+{} more)</i>", top3, total_projects - 3)
    };

    let vault_summary = match (
        details.backed_up_prompts_count,
        details.restored_prompts_count,
    ) {
        (Some(b), Some(r)) if b > 0 || r > 0 => {
            format!("<b>{} captured</b> • <b>{} restored</b>", b, r)
        }
        (Some(b), _) if b > 0 => format!("<b>{} captured</b>", b),
        _ => "<b>0 active prompts</b> (SQLite split-db)".to_string(),
    };

    let trigger_summary = if !details.reason.trim().is_empty() {
        details.reason.trim()
    } else if details.is_auto {
        "Auto-Switcher (Quota boundary)"
    } else {
        "Manual rotation triggered by user"
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
            links.join(" • ")
        )
    } else {
        String::new()
    };

    let node_line = if !m_ip.is_empty() && m_ip != "127.0.0.1" && m_ip != "Unknown IP" {
        format!(
            "🖥️ <b>Machine:</b> <b>{}</b> [<code>{}</code>] • 🏷️ <b>Instance:</b> <b>{}</b>",
            escape_telegram_html(&m_name),
            escape_telegram_html(&m_ip),
            escape_telegram_html(&details.instance_name),
        )
    } else {
        format!(
            "🖥️ <b>Machine:</b> <b>{}</b> • 🏷️ <b>Instance:</b> <b>{}</b>",
            escape_telegram_html(&m_name),
            escape_telegram_html(&details.instance_name),
        )
    };

    let palette = telegram_inbound::active_card_palette();
    let prompt_section = if !prompt_preview_text.is_empty() {
        telegram_inbound::prompt_note(
            palette,
            &format!("Active Prompt ({} words)", prompt_words_count),
            &crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500),
        )
    } else {
        String::new()
    };

    let quota = telegram_inbound::section(
        palette,
        "Quota &amp; rotation",
        &[
            telegram_inbound::kv_block(
                palette,
                "Previous account",
                &format!(
                    "<b>{}</b> (4h: <b>{}</b> · Week: <b>{}</b>)",
                    escape_telegram_html(&from_display),
                    escape_telegram_html(&prev_4h_str),
                    escape_telegram_html(&prev_weekly_str)
                ),
            ),
            telegram_inbound::kv_block(
                palette,
                "Selected account",
                &format!(
                    "<b>{}</b> (4h: <b>{}</b> · Week: <b>{}</b>)",
                    escape_telegram_html(&selected_masked),
                    escape_telegram_html(&target_4h_str),
                    escape_telegram_html(&target_weekly_str)
                ),
            ),
            telegram_inbound::kv_block(
                palette,
                "Rotation trigger",
                &format!(
                    "<b>{}</b> (Guard: <b>{}</b>)",
                    escape_telegram_html(trigger_summary),
                    escape_telegram_html(&threshold_display)
                ),
            ),
        ],
    );

    let workspace = telegram_inbound::section(
        palette,
        "Workspace",
        &[
            telegram_inbound::kv_block(
                palette,
                &format!("Active projects ({})", total_projects),
                &projects_formatted,
            ),
            telegram_inbound::kv_block(palette, "Prompt vault", &vault_summary),
        ],
    );

    let text = format!(
        "{} <b>Antigravity Manager · Account Rotation</b>\n\
        {}\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        {}\n\
        {}\
        {}\
        {}\n\
        <i>{} Antigravity Manager {} · {}</i>",
        palette.mark,
        node_line,
        quota,
        workspace,
        attached_images_section,
        prompt_section,
        palette.mark,
        escape_telegram_html(&pkg_ver),
        palette.label
    );

    let reply_markup = Some(telegram_inbound::build_switch_alert_reply_markup(
        &details.instance_name,
        &details.selected_email,
        &target_4h_str,
        &target_weekly_str,
        &full_prompt_content,
        &resolved_image_urls,
    ));

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
        Err(e) => Err(redact_secrets(&e.to_string())),
    }
}
