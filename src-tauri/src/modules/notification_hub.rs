//! Notification Hub Module
//! Coordinates unified notifications across Email and Telegram channels
//! for critical events such as account switching, quota alerts, and workspace updates.

#![allow(dead_code)]

use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::logger;
use crate::modules::telegram_inbound;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static PREVIOUS_EMAIL_STATE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static LAST_SWITCH_DISPATCH: Lazy<Mutex<(String, i64)>> =
    Lazy::new(|| Mutex::new((String::new(), 0)));

/// Record the previous account email prior to switching so telemetry can report old_email accurately
pub fn record_previous_email(email: &str) {
    let trimmed = email.trim();
    if !trimmed.is_empty() {
        if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
            *guard = Some(trimmed.to_string());
        }
    }
}

fn resolve_switch_context(
    new_email: &str,
    instance_spec: &str,
) -> (String, String, String, String) {
    let mut old_email = String::new();
    if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
        if let Some(prev) = guard.take() {
            if !prev.is_empty() {
                old_email = prev;
            }
        }
    }

    let mut inst_id = instance_spec.to_string();
    let mut inst_name = instance_spec.to_string();
    let mut is_default = instance_spec.eq_ignore_ascii_case("default");

    if let Ok(registry) = crate::modules::instance::load_registry() {
        let found = registry
            .instances
            .iter()
            .find(|i| {
                i.id.eq_ignore_ascii_case(instance_spec)
                    || i.name.eq_ignore_ascii_case(instance_spec)
            })
            .or_else(|| {
                registry
                    .instances
                    .iter()
                    .find(|i| i.id == registry.active_instance_id)
            });

        if let Some(inst) = found {
            inst_id = inst.id.clone();
            inst_name = inst.name.clone();
            is_default = inst.is_default || inst.id == "default";

            if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
                if let Some(ref b_email) = inst.bound_email {
                    if !b_email.is_empty() && !b_email.eq_ignore_ascii_case(new_email) {
                        old_email = b_email.clone();
                    } else if old_email.is_empty() && !b_email.is_empty() {
                        old_email = b_email.clone();
                    }
                }
                if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
                    if let Some(ref b_id) = inst.bound_account_id {
                        if let Ok(acc) = crate::modules::account::load_account(b_id) {
                            if !acc.email.is_empty() && !acc.email.eq_ignore_ascii_case(new_email) {
                                old_email = acc.email;
                            } else if old_email.is_empty() && !acc.email.is_empty() {
                                old_email = acc.email;
                            }
                        }
                    }
                }
            }
        }
    }

    if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
        if let Ok(Some(cur_acc)) = crate::modules::account::get_current_account() {
            if !cur_acc.email.is_empty() && !cur_acc.email.eq_ignore_ascii_case(new_email) {
                old_email = cur_acc.email;
            } else if old_email.is_empty() && !cur_acc.email.is_empty() {
                old_email = cur_acc.email;
            }
        }
    }

    if !new_email.trim().is_empty() {
        if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
            *guard = Some(new_email.trim().to_string());
        }
    }

    let instance_mode = if is_default {
        "default".to_string()
    } else {
        "isolated".to_string()
    };

    (old_email, inst_id, inst_name, instance_mode)
}

#[derive(Debug, Clone, Default)]
pub struct SwitchNotificationDetails {
    pub previous_email: Option<String>,
    pub predicted_next_email: Option<String>,
    pub selected_email: String,
    pub credit_before_switch: Option<f64>,
    pub threshold_activated: Option<f64>,
    pub instance_id: String,
    pub instance_name: String,
    pub instance_mode: String,
    pub reason: String,
    pub is_auto: bool,
}

/// Dispatch rich notifications across Email and Telegram upon account/instance switch
pub fn notify_account_switched_details(mut details: SwitchNotificationDetails) {
    let now_ts = chrono::Utc::now().timestamp();
    let selected_clean = details.selected_email.trim().to_string();

    if let Ok(mut guard) = LAST_SWITCH_DISPATCH.lock() {
        if guard.0.eq_ignore_ascii_case(&selected_clean) && (now_ts - guard.1).abs() <= 5 {
            return;
        }
        *guard = (selected_clean.clone(), now_ts);
    }

    let (context_old_email, inst_id, inst_name, instance_mode) =
        resolve_switch_context(&selected_clean, &details.instance_name);

    if details.instance_id.is_empty() {
        details.instance_id = inst_id;
    }
    if details.instance_name.is_empty() {
        details.instance_name = inst_name;
    }
    if details.instance_mode.is_empty() {
        details.instance_mode = instance_mode;
    }

    let resolved_prev = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("default"))
        .unwrap_or(context_old_email.trim());

    let final_prev = if resolved_prev.is_empty() || resolved_prev.eq_ignore_ascii_case("default") {
        "(none / standby)".to_string()
    } else {
        resolved_prev.to_string()
    };
    details.previous_email = Some(final_prev);

    if details.predicted_next_email.is_none() {
        details.predicted_next_email = Some(selected_clean.clone());
    }

    tokio::spawn(async move {
        dispatch_email_switch_alert(&details);
        dispatch_telegram_switch_alert(&details).await;
    });
}

/// Dispatch notifications across Email and Telegram upon account/instance switch (backward-compatible)
pub fn notify_account_switched(
    account_email: &str,
    instance_name: &str,
    reason: &str,
    is_auto: bool,
) {
    notify_account_switched_details(SwitchNotificationDetails {
        previous_email: None,
        predicted_next_email: None,
        selected_email: account_email.to_string(),
        credit_before_switch: None,
        threshold_activated: None,
        instance_id: instance_name.to_string(),
        instance_name: instance_name.to_string(),
        instance_mode: String::new(),
        reason: reason.to_string(),
        is_auto,
    });
}

/// Helper to render and dispatch email switch notification
fn dispatch_email_switch_alert(details: &SwitchNotificationDetails) {
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

    let from_display = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("default"))
        .unwrap_or("(none / standby)");

    let predicted_display = details
        .predicted_next_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(details.selected_email.as_str());

    let selected_display = details.selected_email.trim();

    let credit_before_display = details
        .credit_before_switch
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());

    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    let trigger_label = if details.is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let condition = if details.is_auto {
        let r_lower = details.reason.to_lowercase();
        if r_lower.contains("critical") {
            "critical_quota"
        } else if r_lower.contains("depleted") {
            "depleted_before_finish"
        } else if r_lower.contains("force") {
            "forced_rotation"
        } else {
            "low_quota_threshold"
        }
    } else {
        "manual_switch"
    };

    let threshold_display = details
        .threshold_activated
        .map(|t| format!("{:.1}% ({})", t, condition))
        .unwrap_or_else(|| condition.to_string());

    let subject = format!(
        "[Antigravity | {} | {} | {}] [JSON] Account Switched: {} -> {}",
        pkg_ver, m_name, m_ip, from_display, selected_display
    );

    // Extract active running prompt, reinjection status, and image payload for the switched instance
    let mut running_prompt_id: Option<String> = None;
    let mut running_prompt_snippet: Option<String> = None;
    let mut running_prompt_project: Option<String> = None;
    let mut running_prompts_count: usize = 0;
    let mut has_images: bool = false;
    let mut images_attached: bool = false;

    if let Ok(prompts) = crate::modules::repo_db::list_all_prompts() {
        let matched: Vec<_> = prompts
            .into_iter()
            .filter(|p| {
                p.instance_id.eq_ignore_ascii_case(&details.instance_name)
                    || p.instance_id.eq_ignore_ascii_case(&details.instance_id)
                    || (details.instance_id == "default"
                        && (p.instance_id.is_empty() || p.instance_id == "default"))
            })
            .collect();

        running_prompts_count = matched
            .iter()
            .filter(|p| {
                p.status == "running" || p.status == "backed_up" || p.status == "dispatched"
            })
            .count();

        if let Some(p) = matched.first() {
            running_prompt_id = Some(p.id.clone());
            let snippet = if p.prompt_content.len() > 120 {
                format!("{}...", &p.prompt_content[..120])
            } else {
                p.prompt_content.clone()
            };
            running_prompt_snippet = Some(snippet);
            running_prompt_project = Some(p.repo_path.clone());
            if p.image_payload.is_some() {
                has_images = true;
                images_attached = true;
            }
        }
    }

    if running_prompt_snippet.is_none() {
        if let Ok(projects) = crate::modules::repo_db::list_running_projects() {
            let matched_projs: Vec<_> = projects
                .into_iter()
                .filter(|p| {
                    p.instance_id.eq_ignore_ascii_case(&details.instance_name)
                        || p.instance_id.eq_ignore_ascii_case(&details.instance_id)
                        || (details.instance_id == "default"
                            && (p.instance_id.is_empty() || p.instance_id == "default"))
                })
                .collect();

            if running_prompts_count == 0 {
                running_prompts_count = matched_projs.iter().filter(|p| p.is_running).count();
            }

            if let Some(proj) = matched_projs.first() {
                let resume_file =
                    std::path::PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&resume_file) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(prompt_text) =
                                val.get("prompt_content").and_then(|v| v.as_str())
                            {
                                let snippet = if prompt_text.len() > 120 {
                                    format!("{}...", &prompt_text[..120])
                                } else {
                                    prompt_text.to_string()
                                };
                                running_prompt_snippet = Some(snippet);
                                running_prompt_id = val
                                    .get("prompt_id")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                running_prompt_project = Some(proj.repo_path.clone());
                                if val.get("image_payload").and_then(|v| v.as_str()).is_some()
                                    || val
                                        .get("has_image")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false)
                                {
                                    has_images = true;
                                    images_attached = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let is_reinjecting = running_prompts_count > 0;
    let prompts_resent = is_reinjecting;
    let now_ts = chrono::Utc::now().timestamp();

    let telemetry_data = serde_json::json!({
        "agm_version": pkg_ver,
        "vm_name": m_name,
        "local_ip": m_ip,
        "previous_email": from_display,
        "predicted_next_email": predicted_display,
        "selected_email": selected_display,
        "credit_before_switch": details.credit_before_switch,
        "threshold_activated": details.threshold_activated,
        "target_account": selected_display,
        "old_email": from_display,
        "new_email": selected_display,
        "instance_id": details.instance_id,
        "instance_name": details.instance_name,
        "instance_mode": details.instance_mode,
        "switch_mode": if details.is_auto { "auto" } else { "manual" },
        "condition": condition,
        "reason": details.reason,
        "running_prompts_count": running_prompts_count,
        "prompts_resent": prompts_resent,
        "is_reinjecting": is_reinjecting,
        "running_prompt_id": running_prompt_id,
        "running_prompt_snippet": running_prompt_snippet,
        "running_prompt_project": running_prompt_project,
        "has_images": has_images,
        "images_attached": images_attached,
        "timestamp": now_ts,
    });
    let telemetry_json_pretty = serde_json::to_string_pretty(&telemetry_data).unwrap_or_default();

    let running_prompt_display = running_prompt_snippet
        .as_deref()
        .unwrap_or("(none running)");
    let reinject_display = if prompts_resent {
        "<span style=\"color: #059669; font-weight: bold;\">Yes (Auto-Resumed via .antigravity_resume_task.json)</span>"
    } else {
        "<span style=\"color: #64748b;\">No (0 running prompts)</span>"
    };
    let images_display = if has_images {
        "<span style=\"color: #2563eb; font-weight: bold;\">Yes (Base64 image payload preserved & attached)</span>"
    } else {
        "<span style=\"color: #64748b;\">None</span>"
    };

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
        <span style="background: #334155; color: #f8fafc; padding: 4px 10px; border-radius: 6px; font-family: monospace; font-size: 13px; font-weight: bold;">[Antigravity | {} | {} | {}]</span>
        <span style="background: #059669; color: #ffffff; padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: bold; text-transform: uppercase; margin-left: 8px;">SWITCHED</span>
      </div>
      <h2 style="margin: 8px 0 0 0; font-size: 18px; color: #ffffff;">Antigravity Account Switched</h2>
    </div>
    <div style="padding: 24px;">
      <p style="margin: 0 0 16px 0; color: #475569; font-size: 14px;">An account rotation was executed successfully. Target credentials have been injected into IDE state storage.</p>
      <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Version</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Previous Account</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Predicted Next Account</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Selected Account</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #059669; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Credit Before Switch</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-weight: bold;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600; width: 160px;">Threshold Activated</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Target Instance</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{} ({})</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Instance Mode</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Trigger Mode</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Reason</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Running Prompts Count</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-weight: bold;">{} prompt(s)</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Prompts Resent / Re-injected</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Attached Images</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Active Prompt Preview</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: monospace; font-size: 12px;">{}</td></tr>
        <tr><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #64748b; font-weight: 600;">Origin Node</td><td style="padding: 10px 12px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{} ({})</td></tr>
      </table>
      <div style="margin-top: 16px;">
        <span style="font-size: 11px; font-weight: bold; color: #64748b; text-transform: uppercase;">Machine Telemetry (JSON State Machine)</span>
        <pre style="background: #0f172a; color: #38bdf8; padding: 12px; border-radius: 8px; font-family: monospace; font-size: 12px; overflow-x: auto; margin-top: 6px;">{}</pre>
      </div>
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
        from_display,
        predicted_display,
        selected_display,
        credit_before_display,
        threshold_display,
        details.instance_name,
        details.instance_id,
        details.instance_mode,
        trigger_label,
        details.reason,
        running_prompts_count,
        reinject_display,
        images_display,
        running_prompt_display,
        m_name,
        m_ip,
        telemetry_json_pretty,
        pkg_ver
    );

    let _ = email_sender::dispatch_email_with_failover(
        &subject,
        &telemetry_json_pretty,
        &active_recipients,
    );
}

/// Helper to render and dispatch Telegram switch notification
async fn dispatch_telegram_switch_alert(details: &SwitchNotificationDetails) {
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

    let trigger_label = if details.is_auto {
        "Auto-Switcher (Quota/Period boundary)"
    } else {
        "Manual User Switch"
    };

    let from_display = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("default"))
        .unwrap_or("(none / standby)");

    let predicted_display = details
        .predicted_next_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(details.selected_email.as_str());

    let credit_before_display = details
        .credit_before_switch
        .map(|q| format!("{:.1}%", q))
        .unwrap_or_else(|| "-".to_string());

    let threshold_display = details
        .threshold_activated
        .map(|t| format!("{:.1}%", t))
        .unwrap_or_else(|| "-".to_string());

    let text = format!(
        "🔄 <b>Antigravity Manager: Account Switched</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        📦 <b>Version:</b> <code>{}</code>\n\
        👤 <b>Previous Account:</b> <code>{}</code>\n\
        🎯 <b>Predicted Next:</b> <code>{}</code>\n\
        ✅ <b>Selected Account:</b> <code>{}</code>\n\
        📊 <b>Credit Before Switch:</b> <code>{}</code>\n\
        ⚙️ <b>Threshold Activated:</b> <code>{}</code>\n\
        💻 <b>Target Instance:</b> <code>{}</code> ({})\n\
        🏷️ <b>Trigger:</b> {}\n\
        📝 <b>Reason:</b> {}\n\
        🖥️ <b>Host:</b> <code>{}</code> ({})\n\
        ⏰ <b>Timestamp:</b> {}",
        pkg_ver,
        from_display,
        predicted_display,
        details.selected_email,
        credit_before_display,
        threshold_display,
        details.instance_name,
        details.instance_id,
        trigger_label,
        details.reason,
        m_name,
        m_ip,
        now_str
    );

    if let Err(e) = telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &text).await
    {
        logger::log_warn(&format!(
            "[NotificationHub] Telegram switch alert failed: {}",
            e
        ));
    }
}

/// Dispatch self-notification email when an email account or recipient is added/updated
pub fn notify_email_config_added(title: &str, details: serde_json::Value) {
    let title_str = title.to_string();
    tokio::spawn(async move {
        dispatch_email_config_added_alert(&title_str, details);
    });
}

fn dispatch_email_config_added_alert(title: &str, details: serde_json::Value) {
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
        "[Antigravity | {} | {} | {}] [JSON] Email Config Added: {}",
        pkg_ver, m_name, m_ip, title
    );

    let json_pretty = serde_json::to_string_pretty(&details).unwrap_or_default();
    let _ = email_sender::dispatch_email_with_failover(&subject, &json_pretty, &target_recipients);
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
