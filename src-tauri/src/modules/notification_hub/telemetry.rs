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

    let is_emergency = verified_running == 0 && backed_up_count > 0;

    // Telegram telemetry
    if let Ok(config) = telegram_inbound::load_config() {
        if config.is_enabled && !config.bot_token.trim().is_empty() {
            if let Some(chat_id) = config.allowed_chat_id {
                let tg_text = if is_emergency {
                    format!(
                        "🚨 <b>Antigravity Manager: EMERGENCY ALERT</b>\n\
                        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                        ⚠️ <b>Prompts Failed to Resume After Switch!</b>\n\
                        📦 <b>Version:</b> <code>{}</code>\n\
                        💻 <b>Instance:</b> <code>{}</code>\n\
                        💾 <b>Backed Up Prompts:</b> <code>{}</code>\n\
                        🔄 <b>Restored Records:</b> <code>{}</code>\n\
                        🛑 <b>Verified Running:</b> <code>0 (FAILED TO DETECT RUNNING PROMPT)</code>\n\
                        📁 <b>Projects:</b> <code>{}</code>\n\
                        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
                        ⏰ <b>Timestamp:</b> {}\n\
                        ⚡ <b>Action:</b> Run <code>agm prompts status</code> or <code>agm prompts restore</code>.",
                        escape_telegram_html(&pkg_ver),
                        escape_telegram_html(instance_id),
                        backed_up_count,
                        restored_count,
                        escape_telegram_html(&projects_display),
                        escape_telegram_html(&m_name),
                        escape_telegram_html(&m_ip),
                        escape_telegram_html(&now_str)
                    )
                } else {
                    format!(
                        "✅ <b>Antigravity Manager: Post-Switch Liveness Confirmed</b>\n\
                        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                        📦 <b>Version:</b> <code>{}</code>\n\
                        💻 <b>Instance:</b> <code>{}</code>\n\
                        🟢 <b>Verified Running Prompts:</b> <code>{}</code> (of {} backed up)\n\
                        📁 <b>Active Projects:</b> <code>{}</code>\n\
                        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
                        ⏰ <b>Timestamp:</b> {}",
                        escape_telegram_html(&pkg_ver),
                        escape_telegram_html(instance_id),
                        verified_running,
                        backed_up_count,
                        escape_telegram_html(&projects_display),
                        escape_telegram_html(&m_name),
                        escape_telegram_html(&m_ip),
                        escape_telegram_html(&now_str)
                    )
                };

                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &tg_text)
                        .await,
                    "send_telegram_message",
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
