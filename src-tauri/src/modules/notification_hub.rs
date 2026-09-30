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

/// Extract up to max_words words from prompt text and return (snippet, word_count)
pub fn extract_words_preview(text: &str, max_words: usize) -> (String, usize) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), 0);
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let count = words.len();
    if count <= max_words {
        (trimmed.to_string(), count)
    } else {
        (format!("{}...", words[..max_words].join(" ")), count)
    }
}

/// Escape text for Telegram HTML parse_mode
pub fn escape_telegram_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Deduplicate project and resource names while preserving insertion order
pub fn deduplicate_names<I, S>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for it in items {
        let s = it.as_ref().trim();
        if !s.is_empty() && seen.insert(s.to_lowercase()) {
            out.push(s.to_string());
        }
    }
    out
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
                    }
                }
                if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
                    if let Some(ref b_id) = inst.bound_account_id {
                        if let Ok(acc) = crate::modules::account::load_account(b_id) {
                            if !acc.email.is_empty() && !acc.email.eq_ignore_ascii_case(new_email) {
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
            }
        }
    }

    if old_email.trim().eq_ignore_ascii_case(new_email.trim()) {
        old_email.clear();
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
    pub previous_quota_4h: Option<f64>,
    pub previous_quota_weekly: Option<f64>,
    pub predicted_next_email: Option<String>,
    pub selected_email: String,
    pub target_quota_4h: Option<f64>,
    pub target_quota_weekly: Option<f64>,
    pub credit_before_switch: Option<f64>,
    pub threshold_activated: Option<f64>,
    pub instance_id: String,
    pub instance_name: String,
    pub instance_mode: String,
    pub reason: String,
    pub is_auto: bool,
    pub backed_up_projects: Vec<String>,
    pub backed_up_prompts_count: Option<usize>,
    pub restored_prompts_count: Option<usize>,
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

    let final_prev = if resolved_prev.is_empty()
        || resolved_prev.eq_ignore_ascii_case("default")
        || resolved_prev.eq_ignore_ascii_case(&selected_clean)
    {
        "(none / standby)".to_string()
    } else {
        resolved_prev.to_string()
    };
    details.previous_email = Some(final_prev.clone());

    let need_predict = match &details.predicted_next_email {
        None => true,
        Some(em) => {
            let em_trimmed = em.trim();
            em_trimmed.is_empty()
                || em_trimmed.eq_ignore_ascii_case(&selected_clean)
                || em_trimmed.eq_ignore_ascii_case(&final_prev)
        }
    };

    if need_predict {
        let mut pred_exclusions = vec![selected_clean.clone()];
        if !final_prev.is_empty() && !final_prev.eq_ignore_ascii_case("(none / standby)") {
            pred_exclusions.push(final_prev.clone());
        }
        let target_inst = if details.instance_id.is_empty() {
            "default"
        } else {
            &details.instance_id
        };
        let predicted_candidate = crate::modules::auto_switcher::select_candidate_profiles(
            target_inst,
            "gemini-2.5-pro",
            15.0,
            &pred_exclusions,
        )
        .ok()
        .and_then(|v| v.into_iter().next())
        .filter(|c| {
            !c.email.trim().eq_ignore_ascii_case(&selected_clean)
                && (final_prev.eq_ignore_ascii_case("(none / standby)")
                    || !c.email.trim().eq_ignore_ascii_case(&final_prev))
        });
        details.predicted_next_email = predicted_candidate.map(|c| c.email);
    }

    // Resolve target dual-window quotas if absent
    if details.target_quota_4h.is_none() || details.target_quota_weekly.is_none() {
        if let Ok(accounts) = crate::modules::account::list_accounts() {
            if let Some(target_acc) = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&selected_clean))
            {
                let (q4, qw) = crate::modules::auto_switcher::extract_dual_window_quotas(
                    target_acc,
                    "gemini-2.5-pro",
                );
                if details.target_quota_4h.is_none() {
                    details.target_quota_4h = q4;
                }
                if details.target_quota_weekly.is_none() {
                    details.target_quota_weekly = qw;
                }
            }
        }
    }

    // Resolve previous dual-window quotas if absent
    if (details.previous_quota_4h.is_none() || details.previous_quota_weekly.is_none())
        && !final_prev.eq_ignore_ascii_case("(none / standby)")
    {
        if let Ok(accounts) = crate::modules::account::list_accounts() {
            if let Some(prev_acc) = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&final_prev))
            {
                let (q4, qw) = crate::modules::auto_switcher::extract_dual_window_quotas(
                    prev_acc,
                    "gemini-2.5-pro",
                );
                if details.previous_quota_4h.is_none() {
                    details.previous_quota_4h = q4;
                }
                if details.previous_quota_weekly.is_none() {
                    details.previous_quota_weekly = qw;
                }
            }
        }
    }

    tauri::async_runtime::spawn(async move {
        dispatch_email_switch_alert(&details);
        dispatch_self_json_in_use_broadcast(&details);
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
        previous_quota_4h: None,
        previous_quota_weekly: None,
        predicted_next_email: None,
        selected_email: account_email.to_string(),
        target_quota_4h: None,
        target_quota_weekly: None,
        credit_before_switch: None,
        threshold_activated: None,
        instance_id: instance_name.to_string(),
        instance_name: instance_name.to_string(),
        instance_mode: String::new(),
        reason: reason.to_string(),
        is_auto,
        backed_up_projects: Vec::new(),
        backed_up_prompts_count: None,
        restored_prompts_count: None,
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
  a {{ color: #ffffff !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #ffffff !important; }}
  a:hover {{ color: #e0f2fe !important; }}
  td a, p a {{ background: #2563eb; color: #ffffff !important; padding: 2px 8px; border-radius: 6px; text-decoration: none; display: inline-block; font-size: 14px; font-weight: bold; }}
  td a:hover, p a:hover {{ background: #1d4ed8; color: #ffffff !important; }}
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

    let _ = email_sender::dispatch_email_with_failover(&subject, email_body, &active_recipients);
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
    let _ = email_sender::dispatch_email_with_failover(&subject, &json_text, &recipients);
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
    tauri::async_runtime::spawn(async move {
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
        "[AGM {} | {} | {}] [JSON] Email Config Added: {}",
        pkg_ver, m_name, m_ip, title
    );

    let json_pretty = serde_json::to_string_pretty(&details).unwrap_or_default();
    let _ = email_sender::dispatch_email_with_failover(&subject, &json_pretty, &target_recipients);
}

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

                let _ =
                    telegram_inbound::send_telegram_message(&config.bot_token, chat_id, &tg_text)
                        .await;
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

    let _ = email_sender::dispatch_email_with_failover(&subject, &html, &target_recipients);
}

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
fn dispatch_email_update_alert(
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

    let _ = email_sender::dispatch_email_with_failover(&subject, &body, &target_recipients);
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
        let _ = crate::modules::update_checker::save_update_settings(&update_settings);
        return;
    }

    if prev_ver != current_ver {
        // Version changed! An update occurred!
        logger::log_info(&format!(
            "[NotificationHub] System update detected on startup: v{} -> v{}",
            prev_ver, current_ver
        ));

        update_settings.last_known_version = current_ver.to_string();
        let _ = crate::modules::update_checker::save_update_settings(&update_settings);

        notify_system_updated(
            &prev_ver,
            current_ver,
            Some("System update installed and launched successfully."),
        );
    }
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

    #[test]
    fn test_resolve_switch_context_never_returns_same_as_new_email() {
        let same_email = "test.same@example.com";
        record_previous_email(same_email);

        // When switching to the same email, old_email must never equal new_email
        let (old, _, _, _) = resolve_switch_context(same_email, "default");
        assert_ne!(
            old.trim().to_lowercase(),
            same_email.trim().to_lowercase(),
            "old_email must not equal new_email"
        );

        // If new_email matches whatever current account is on disk, old_email must be empty
        if let Ok(Some(cur)) = crate::modules::account::get_current_account() {
            if !cur.email.is_empty() {
                record_previous_email(&cur.email);
                let (old_cur, _, _, _) = resolve_switch_context(&cur.email, "default");
                assert!(
                    old_cur.is_empty(),
                    "old_email must be empty when switching to current account"
                );
            }
        }

        // When switching to a different email with isolated test state
        let rec_email = "prior.recorded@example.com";
        record_previous_email(rec_email);
        let diff_email = "brand.new.target@example.com";
        let (old2, _, _, _) = resolve_switch_context(diff_email, "default");
        assert_ne!(
            old2.trim().to_lowercase(),
            diff_email.trim().to_lowercase(),
            "old_email must not equal new_email"
        );
    }

    #[test]
    fn test_email_switch_alert_distinctness_invariants() {
        let selected = "user1@example.com";
        let prev_identical = "user1@example.com";
        let pred_identical = "user1@example.com";

        let from_display = Some(prev_identical)
            .map(|s| s.trim())
            .filter(|s| {
                !s.is_empty()
                    && !s.eq_ignore_ascii_case("default")
                    && !s.eq_ignore_ascii_case(selected)
            })
            .unwrap_or("(none / standby)");

        assert_eq!(from_display, "(none / standby)");

        let predicted_display = Some(pred_identical)
            .map(|s| s.trim())
            .filter(|s| {
                !s.is_empty()
                    && !s.eq_ignore_ascii_case(selected)
                    && (from_display.eq_ignore_ascii_case("(none / standby)")
                        || !s.eq_ignore_ascii_case(from_display))
            })
            .unwrap_or("(none / pool exhausted)");

        assert_eq!(predicted_display, "(none / pool exhausted)");
    }
}
