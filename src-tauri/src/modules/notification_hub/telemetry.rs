use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::telegram_inbound;

use super::*;

/// Dispatch post-switch prompt restoration status and emergency alert if prompts failed to resume
pub fn notify_post_switch_prompt_status(
    instance_id: &str,
    backed_up_count: usize,
    restored_count: usize,
    verified_running: usize,
    project_names: Vec<String>,
) {
    let inst_id = instance_id.to_string();
    tauri::async_runtime::spawn(async move {
        dispatch_post_switch_prompt_telemetry(
            &inst_id,
            backed_up_count,
            restored_count,
            verified_running,
            &project_names,
        )
        .await;
    });
}

async fn dispatch_post_switch_prompt_telemetry(
    instance_id: &str,
    backed_up_count: usize,
    restored_count: usize,
    verified_running: usize,
    project_names: &[String],
) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now_str = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let deduped = deduplicate_names(project_names);
    let projects_display = if !deduped.is_empty() {
        deduped.join(", ")
    } else {
        "Antigravity-Manager".to_string()
    };
    let meaningful_projects: Vec<String> = {
        let filtered: Vec<String> = deduped
            .iter()
            .filter(|p| {
                !p.eq_ignore_ascii_case("unnamed-project")
                    && !p.eq_ignore_ascii_case("project")
                    && !p.trim().is_empty()
            })
            .cloned()
            .collect();
        if filtered.is_empty() {
            deduped
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

    let is_emergency = verified_running == 0 && backed_up_count > 0;

    // Telegram telemetry
    if let Ok(config) = telegram_inbound::load_config() {
        if config.is_enabled && !config.bot_token.trim().is_empty() {
            if let Some(chat_id) = config.allowed_chat_id {
                let node_line = if !m_ip.is_empty() && m_ip != "127.0.0.1" && m_ip != "Unknown IP" {
                    format!(
                        "🖥️ <b>Machine:</b> <b>{}</b> [<code>{}</code>] • 🏷️ <b>Instance:</b> <b>{}</b>",
                        escape_telegram_html(&m_name),
                        escape_telegram_html(&m_ip),
                        escape_telegram_html(instance_id)
                    )
                } else {
                    format!(
                        "🖥️ <b>Machine:</b> <b>{}</b> • 🏷️ <b>Instance:</b> <b>{}</b>",
                        escape_telegram_html(&m_name),
                        escape_telegram_html(instance_id)
                    )
                };

                // Look up active running or backed up prompt preview for this instance
                let (prompt_preview_text, prompt_words_count, full_prompt_content, detected_images) =
                    if let Ok(all_prompts) = crate::modules::repo_db::list_all_prompts() {
                        let matched_prompt = all_prompts.into_iter().find(|p| {
                            (p.status == "running"
                                || p.status == "backed_up"
                                || p.status == "dispatched")
                                && (p.instance_id.eq_ignore_ascii_case(instance_id)
                                    || (instance_id == "default"
                                        && (p.instance_id.is_empty()
                                            || p.instance_id == "default")))
                        });

                        if let Some(p) = matched_prompt {
                            let (_, images) =
                                crate::modules::repo_db::extract_image_payload_or_path(
                                    &p.prompt_content,
                                );
                            let (preview, words) = extract_words_preview(&p.prompt_content, 200);
                            (preview, words, p.prompt_content, images)
                        } else {
                            (String::new(), 0, String::new(), Vec::new())
                        }
                    } else {
                        (String::new(), 0, String::new(), Vec::new())
                    };

                let attached_images_section = if !detected_images.is_empty() {
                    let mut links = Vec::new();
                    for (idx, url) in detected_images.iter().enumerate() {
                        let clean_url =
                            crate::modules::telegram_inbound::clean_for_telegram_html(url, 200);
                        links.push(format!(
                            "<a href=\"{}\">🖼️ Image #{}</a>",
                            clean_url,
                            idx + 1
                        ));
                    }
                    format!(
                        "\n🖼️ <b>Attached Media ({}):</b> {}\n",
                        detected_images.len(),
                        links.join(" • ")
                    )
                } else {
                    String::new()
                };

                let palette = telegram_inbound::active_card_palette();
                let prompt_section = if !prompt_preview_text.is_empty() {
                    telegram_inbound::prompt_note(
                        palette,
                        &format!("Active Prompt ({} words)", prompt_words_count),
                        &crate::modules::telegram_inbound::clean_for_telegram_html(
                            &prompt_preview_text,
                            1500,
                        ),
                    )
                } else {
                    String::new()
                };

                let tg_text = if is_emergency {
                    let body = telegram_inbound::section(
                        palette,
                        "Prompts failed to resume",
                        &[
                            telegram_inbound::kv_block(
                                palette,
                                "Backed up prompts",
                                &format!("<b>{}</b>", backed_up_count),
                            ),
                            telegram_inbound::kv_block(
                                palette,
                                "Restored records",
                                &format!("<b>{}</b>", restored_count),
                            ),
                            telegram_inbound::kv_block(
                                palette,
                                "Verified running",
                                "<b>0 — none detected</b>",
                            ),
                            telegram_inbound::kv_block(
                                palette,
                                &format!("Active projects ({})", total_projects),
                                &projects_formatted,
                            ),
                        ],
                    );
                    format!(
                        "{} <b>Antigravity Manager · Emergency</b>\n\
                        {}\n\
                        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                        {}\
                        {}\
                        {}\n\
                        <i>{} Antigravity Manager {} · {}</i>",
                        palette.mark,
                        node_line,
                        body,
                        attached_images_section,
                        prompt_section,
                        palette.mark,
                        escape_telegram_html(&pkg_ver),
                        palette.label
                    )
                } else {
                    let body = telegram_inbound::section(
                        palette,
                        "Execution status",
                        &[
                            telegram_inbound::kv_block(
                                palette,
                                "Verified running",
                                &format!(
                                    "<b>{} of {}</b> backed up",
                                    verified_running, backed_up_count
                                ),
                            ),
                            telegram_inbound::kv_block(
                                palette,
                                &format!("Active projects ({})", total_projects),
                                &projects_formatted,
                            ),
                        ],
                    );
                    format!(
                        "{} <b>Antigravity Manager · Post-switch liveness</b>\n\
                        {}\n\
                        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                        {}\
                        {}\
                        {}\n\
                        <i>{} Antigravity Manager {} · {}</i>",
                        palette.mark,
                        node_line,
                        body,
                        attached_images_section,
                        prompt_section,
                        palette.mark,
                        escape_telegram_html(&pkg_ver),
                        palette.label
                    )
                };

                let reply_markup = Some(telegram_inbound::build_liveness_alert_reply_markup(
                    instance_id,
                    verified_running,
                    backed_up_count,
                    &full_prompt_content,
                    &detected_images,
                ));

                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    telegram_inbound::send_telegram_message_with_markup(
                        &config.bot_token,
                        chat_id,
                        &tg_text,
                        reply_markup,
                        false,
                    )
                    .await,
                    "send_telegram_message_with_markup",
                );
            }
        }
    }

    // Email telemetry
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

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = if is_emergency {
        format!(
            "[AGM {} | {} | {}] [EMERGENCY] Prompts Failed to Resume on {} ({})",
            pkg_ver, m_name, m_ip, m_name, m_ip
        )
    } else {
        format!(
            "[AGM {} | {} | {}] [STATUS] Post-Switch Liveness Verified: {} Prompts Running on {}",
            pkg_ver, m_name, m_ip, verified_running, m_name
        )
    };

    let status_color = if is_emergency { "#dc2626" } else { "#059669" };
    let status_badge = if is_emergency {
        "EMERGENCY: STALLED"
    } else {
        "LIVENESS VERIFIED"
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body {{ font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; font-size: 15px; color: #0f172a; margin: 0; padding: 24px; background: #f1f5f9; }}
  code {{ font-family: 'Ubuntu Mono', Consolas, monospace; background: #0f172a; color: #38bdf8; padding: 3px 6px; border-radius: 4px; }}
</style>
</head>
<body>
  <div style="max-width: 600px; margin: 0 auto; background: #ffffff; border-radius: 12px; overflow: hidden; border: 1px solid #cbd5e1; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
    <div style="background: #0f172a; padding: 20px 24px; color: #ffffff; border-top: 4px solid {};">
      <span style="background: {}; color: #ffffff; padding: 4px 10px; border-radius: 9999px; font-size: 11px; font-weight: 700; text-transform: uppercase;">{}</span>
      <h3 style="margin: 10px 0 0 0; color: #ffffff;">Post-Switch Prompt Execution Telemetry</h3>
    </div>
    <div style="padding: 24px;">
      <table style="width: 100%; border-collapse: collapse;">
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600; width: 180px;">Host Machine:</td><td style="padding: 8px 0;"><b>{}</b> ({})</td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Instance:</td><td style="padding: 8px 0;"><code>{}</code></td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Backed Up Prompts:</td><td style="padding: 8px 0;">{}</td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Restored Prompts:</td><td style="padding: 8px 0;">{}</td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Verified Running:</td><td style="padding: 8px 0; font-weight: bold; color: {};">{} prompt(s)</td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Active Projects:</td><td style="padding: 8px 0;">{}</td></tr>
        <tr><td style="padding: 8px 0; color: #64748b; font-weight: 600;">Timestamp:</td><td style="padding: 8px 0;">{}</td></tr>
      </table>
      {}
    </div>
  </div>
</body>
</html>"#,
        status_color,
        status_color,
        status_badge,
        m_name,
        m_ip,
        instance_id,
        backed_up_count,
        restored_count,
        status_color,
        verified_running,
        projects_display,
        now_str,
        if is_emergency {
            "<p style=\"margin-top: 18px; padding: 12px; background: #fee2e2; border-left: 4px solid #dc2626; color: #991b1b; border-radius: 4px;\"><b>Emergency Warning:</b> Backed up prompts failed to verify active in IDE queue. Please inspect with <code>agm prompts status</code> or re-run <code>agm prompts restore</code>.</p>"
        } else {
            "<p style=\"margin-top: 18px; padding: 12px; background: #dcfce7; border-left: 4px solid #16a34a; color: #166534; border-radius: 4px;\">Prompts were re-injected and verified actively running post-rotation.</p>"
        }
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_email_with_failover(&subject, &html, &target_recipients),
        "dispatch_email_with_failover",
    );
}
