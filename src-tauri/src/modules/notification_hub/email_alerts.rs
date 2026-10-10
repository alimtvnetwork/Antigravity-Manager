use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;

use super::*;

/// Helper to render and dispatch email switch notification
pub(crate) fn dispatch_email_switch_alert(
    details: &SwitchNotificationDetails,
) -> Result<String, String> {
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(e) => return Err(format!("email settings unreadable: {}", e)),
    };

    if !settings.is_enabled {
        return Err("email notifications are disabled".to_string());
    }
    if !settings.notify_on_workspace_switch {
        return Err("workspace-switch email is disabled".to_string());
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

    let selected_display = details.selected_email.trim();

    let from_display = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("default")
                && !s.eq_ignore_ascii_case(selected_display)
        })
        .unwrap_or("(none / standby)");

    let predicted_display = details
        .predicted_next_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case(selected_display)
                && (from_display.eq_ignore_ascii_case("(none / standby)")
                    || !s.eq_ignore_ascii_case(from_display))
        })
        .unwrap_or("(none / pool exhausted)");

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
        "[AGM {} | {} | {}] [JSON] Account Switched: {} -> {}",
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

        let mut all_running_snippets: Vec<String> = Vec::new();
        for p in matched.iter().filter(|p| {
            p.status == "running"
                || p.status == "backed_up"
                || p.status == "dispatched"
                || p.status == "executing"
        }) {
            if running_prompt_id.is_none() {
                running_prompt_id = Some(p.id.clone());
            }
            if running_prompt_project.is_none() {
                running_prompt_project = Some(p.repo_path.clone());
            }
            if p.image_payload.is_some() {
                has_images = true;
                images_attached = true;
            }
            let (snippet, wc) = extract_words_preview(&p.prompt_content, 200);
            if !snippet.is_empty() {
                let p_id_short = if p.id.len() > 8 { &p.id[..8] } else { &p.id };
                all_running_snippets.push(format!(
                    "[Prompt #{}] ({}, {} words):\n{}",
                    p_id_short, p.repo_path, wc, snippet
                ));
            }
        }
        if !all_running_snippets.is_empty() {
            running_prompt_snippet = Some(all_running_snippets.join("\n\n---\n\n"));
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
                                let (snippet, _wc) = extract_words_preview(prompt_text, 200);
                                running_prompt_snippet = Some(snippet);
                                running_prompt_id = val
                                    .get("prompt_id")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                running_prompt_project = Some(if !proj.repo_name.is_empty() {
                                    proj.repo_name.clone()
                                } else {
                                    std::path::Path::new(&proj.repo_path)
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "Antigravity-Manager".to_string())
                                });
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

    let effective_projects = if !details.backed_up_projects.is_empty() {
        deduplicate_names(&details.backed_up_projects)
    } else {
        match crate::modules::repo_db::list_running_projects() {
            Ok(projs) => deduplicate_names(projs.into_iter().map(|p| p.repo_name)),
            Err(_) => Vec::new(),
        }
    };

    let (node_alias, _) = crate::modules::email_sender::get_local_node_identity();
    let quota_percent = details.target_quota_4h.unwrap_or(100.0);

    let telemetry_data = serde_json::json!({
        "agm_version": pkg_ver,
        "machine_name": m_name,
        "node_alias": node_alias,
        "local_ip": m_ip,
        "previous_email": from_display,
        "predicted_email": predicted_display,
        "selected_email": selected_display,
        "quota_percent": quota_percent,
        "credit_before_switch": details.credit_before_switch,
        "threshold_activated": details.threshold_activated,
        "instance_id": details.instance_id,
        "instance_name": details.instance_name,
        "switch_mode": if details.is_auto { "auto" } else { "manual" },
        "condition": condition,
        "reason": details.reason,
        "running_prompts_count": running_prompts_count,
        "prompts_resent": prompts_resent,
        "is_reinjecting": is_reinjecting,
        "running_prompt_id": running_prompt_id,
        "running_prompt_snippet": running_prompt_snippet,
        "running_prompt_project": running_prompt_project,
        "active_projects": effective_projects,
        "backed_up_prompts_count": details.backed_up_prompts_count,
        "restored_prompts_count": details.restored_prompts_count,
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

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Ubuntu:ital,wght@0,400;0,500;0,700;1,400&family=Ubuntu+Mono:wght@400;700&display=swap" rel="stylesheet">
<style>
  body, table, td, p, div, span {{ font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }}
  code, pre {{ font-family: 'Ubuntu Mono', 'Consolas', 'Courier New', monospace; }}
  a {{ color: #0284c7 !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #0369a1 !important; }}
  a:hover {{ color: #0284c7 !important; }}
  .btn-action, td a.btn-action, p a.btn-action {{ background: #0284c7; color: #ffffff !important; padding: 4px 10px; border-radius: 6px; text-decoration: none; display: inline-block; font-size: 14px; font-weight: bold; border: 1px solid #0369a1; }}
  .btn-action:hover, td a.btn-action:hover, p a.btn-action:hover {{ background: #0369a1; color: #ffffff !important; }}
  @media (prefers-color-scheme: dark) {{
    body {{ background-color: #0f172a !important; color: #f1f5f9 !important; }}
    td {{ color: #e2e8f0 !important; }}
    a {{ color: #38bdf8 !important; }}
    a:visited {{ color: #7dd3fc !important; }}
  }}
</style>
</head>
<body style="margin: 0; padding: 24px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 16px; color: #0f172a;">
  <div style="max-width: 680px; margin: 0 auto; background: #ffffff; border-radius: 14px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 4px solid #38bdf8; padding: 24px 28px; color: #ffffff;">
      <div style="margin-bottom: 12px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #38bdf8; padding: 5px 12px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 13px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[AGM {} | {} | {}]</span>
        <span style="background: #059669; color: #ffffff; padding: 5px 12px; border-radius: 9999px; font-size: 12px; font-weight: 700; text-transform: uppercase; margin-left: 8px; letter-spacing: 0.06em;">SWITCHED</span>
      </div>
      <h2 style="margin: 6px 0 0 0; font-size: 24px; color: #ffffff; font-weight: 700; line-height: 1.3;">Antigravity Account Switched</h2>
    </div>
    <div style="padding: 28px;">
      <p style="margin: 0 0 18px 0; color: #475569; font-size: 15px; line-height: 1.6;">An account rotation was executed successfully. Target credentials have been injected into IDE state storage. Value fields are formatted for quick selection and copying.</p>
      <table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 22px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Version</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #38bdf8; padding: 6px 10px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; font-weight: 700; display: inline-block; user-select: all; -webkit-user-select: all;"><code>{}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Previous Account</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #f8fafc; padding: 8px 12px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; border: 1px solid #334155; user-select: all; -webkit-user-select: all;"><code>{}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Previous Balances</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #cbd5e1; padding: 8px 12px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; border: 1px solid #334155; user-select: all; -webkit-user-select: all;"><code>4-Hour: {} &nbsp;|&nbsp; Weekly: {}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Selected Account (New)</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #34d399; padding: 8px 12px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; font-weight: bold; border: 1px solid #065f46; user-select: all; -webkit-user-select: all;"><code>{}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Target Balances</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #38bdf8; padding: 8px 12px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; border: 1px solid #0369a1; user-select: all; -webkit-user-select: all;"><code>4-Hour: {} &nbsp;|&nbsp; Weekly: {}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600; width: 190px; background: #f8fafc;">Threshold Activated</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #f8fafc; padding: 6px 10px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; user-select: all; -webkit-user-select: all;"><code>{}</code></div></td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Target Instance</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{} ({})</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Trigger Mode</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Reason</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Running Prompts Count</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-weight: bold;">{} prompt(s)</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Prompts Resent / Re-injected</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Attached Images</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Active Prompt Preview</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-size: 14px;">{}</td></tr>
        <tr><td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 600;">Origin Node</td><td style="padding: 10px 18px; border-bottom: 1px solid #f1f5f9;"><div style="background: #0f172a; color: #f8fafc; padding: 6px 10px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; user-select: all; -webkit-user-select: all;"><code>{} ({})</code></div></td></tr>
      </table>
      <div style="margin-top: 20px;">
        <span style="font-size: 13px; font-weight: bold; color: #475569; text-transform: uppercase; letter-spacing: 0.08em;">Active Running Prompts (≥ 200 Words Preview)</span>
        <pre style="background: #0f172a; color: #f8fafc; padding: 16px; border-radius: 10px; font-family: 'Ubuntu Mono', 'Consolas', monospace; font-size: 13px; line-height: 1.6; overflow-x: auto; margin-top: 8px; border: 1px solid #1e293b; white-space: pre-wrap; word-break: break-word; user-select: all; -webkit-user-select: all;">{}</pre>
      </div>
      <div style="margin-top: 20px;">
        <span style="font-size: 13px; font-weight: bold; color: #475569; text-transform: uppercase; letter-spacing: 0.08em;">Machine Telemetry (JSON State Machine)</span>
        <pre style="background: #0f172a; color: #38bdf8; padding: 16px; border-radius: 10px; font-family: 'Ubuntu Mono', 'Consolas', monospace; font-size: 14px; line-height: 1.6; overflow-x: auto; margin-top: 8px; border: 1px solid #1e293b; user-select: all; -webkit-user-select: all;">{}</pre>
      </div>
    </div>
    <div style="background: #f8fafc; padding: 16px 28px; border-top: 1px solid #e2e8f0; font-size: 13px; color: #64748b; text-align: center;">
      Automated Dispatcher &middot; Antigravity Manager {} &middot; Maintained by Alim, Sponsored by RISEUP ASIA LLC
    </div>
  </div>
</body>
</html>"#,
        pkg_ver,
        m_name,
        m_ip,
        pkg_ver,
        from_display,
        prev_4h_str,
        prev_weekly_str,
        selected_display,
        target_4h_str,
        target_weekly_str,
        threshold_display,
        details.instance_name,
        details.instance_id,
        trigger_label,
        details.reason,
        running_prompts_count,
        reinject_display,
        images_display,
        running_prompt_display,
        running_prompt_display,
        m_name,
        m_ip,
        telemetry_json_pretty,
        pkg_ver
    );

    let email_body = if subject.contains("[JSON]") || subject.contains("[json]") {
        &telemetry_json_pretty
    } else {
        &html
    };

    match email_sender::dispatch_email_with_failover(&subject, email_body, &active_recipients) {
        Ok(result) => Ok(format!(
            "sent '{}' via {}",
            subject, result.used_account_email
        )),
        Err(err) => Err(err),
    }
}

/// Dispatch machine-readable pure JSON self-broadcast email to the default account (zero HTML)
pub fn dispatch_self_json_in_use_broadcast(details: &SwitchNotificationDetails) {
    let default_acc = match email_vault_db::get_default_account() {
        Ok(Some(a)) => a,
        _ => return,
    };

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now = chrono::Utc::now();
    let now_ts = now.timestamp();
    let expires_ts = now_ts + 3600; // 1-hour lease window
    let now_iso = now.to_rfc3339();
    let expires_iso = (now + chrono::Duration::hours(1)).to_rfc3339();

    let target_email = details.selected_email.trim();
    let prev_email = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("default")
                && !s.eq_ignore_ascii_case(target_email)
        })
        .unwrap_or("(none / standby)");

    let subject = format!(
        "[AGM IN-USE | {} | {}] {} (1h lease)",
        m_name, m_ip, target_email
    );

    let payload = serde_json::json!({
        "event": "account_in_use_lease",
        "version": format!("v{}", env!("CARGO_PKG_VERSION")),
        "claimed_account": target_email,
        "previous_account": prev_email,
        "node_name": m_name,
        "node_ip": m_ip,
        "instance_id": details.instance_id,
        "claimed_at_utc": now_iso,
        "claimed_at_unix": now_ts,
        "lease_expires_at_utc": expires_iso,
        "lease_expires_at_unix": expires_ts,
        "lease_duration_seconds": 3600,
        "target_quota_4h": details.target_quota_4h,
        "target_quota_weekly": details.target_quota_weekly,
        "previous_quota_4h": details.previous_quota_4h,
        "previous_quota_weekly": details.previous_quota_weekly,
        "trigger_mode": if details.is_auto { "Auto-Switch" } else { "Manual Switch" },
        "reason": details.reason,
        "active_projects": deduplicate_names(&details.backed_up_projects),
        "prompts_backed_up": details.backed_up_prompts_count,
        "prompts_restored": details.restored_prompts_count,
    });

    let json_text = serde_json::to_string_pretty(&payload).unwrap_or_default();
    let recipients = vec![default_acc.email.clone()];
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_email_with_failover(&subject, &json_text, &recipients),
        "dispatch_email_with_failover",
    );
}
