use crate::modules::email_inbound;
use crate::modules::email_sender;
use crate::modules::email_vault_db;
use chrono::Utc;

use super::*;

/// Run a single cycle of inbox checking
pub(crate) async fn poll_inbox_cycle(m_name: &str, m_ip: &str) {
    let accounts = match email_vault_db::list_email_accounts() {
        Ok(a) => a,
        Err(_) => return,
    };

    let active_account = accounts.into_iter().find(|a| a.is_default && a.is_active);
    let account = match active_account {
        Some(a) => a,
        None => return,
    };

    // Spawn blocking for socket IMAP calls
    let acc_clone = account.clone();
    let unread =
        tokio::task::spawn_blocking(move || email_inbound::poll_unread_messages(&acc_clone, 5))
            .await
            .unwrap_or_else(|_| Ok(Vec::new()));

    if let Ok(messages) = unread {
        for msg in messages {
            if let Ok(seen) = email_vault_db::is_message_already_processed(&msg.message_id) {
                if seen {
                    continue;
                }
            }

            let action = email_inbound::parse_email_command(&msg.subject, &msg.body);
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                email_inbound::execute_inbound_action(&msg, action, m_ip, m_name),
                "execute_inbound_action",
            );
        }
    }
}

/// Check quota drop sensor
pub(crate) async fn check_quota_drop_sensor(
    settings: &email_vault_db::EmailNotificationSettings,
    m_name: &str,
    m_ip: &str,
    last_alert: &mut i64,
) {
    // Only check if Antigravity IDE, isolated instances, active projects, or proxy services are currently running
    if !crate::modules::auto_switcher::is_antigravity_or_instance_running(None) {
        return;
    }

    let accounts = match crate::modules::account::list_accounts() {
        Ok(accs) => accs,
        Err(_) => return,
    };

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

    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let threshold = if settings.quota_drop_threshold_percent > 0 {
        settings.quota_drop_threshold_percent as f64
    } else {
        app_cfg.auto_profile_switcher.low_quota_threshold_percent
    };

    if threshold <= 0.0 {
        return;
    }

    for acc in accounts {
        // Enforce: ONLY send low credit alert if we are currently using this account!
        if !crate::modules::auto_switcher::is_account_in_use(&acc) {
            continue;
        }

        if let Some(quota) = acc.quota {
            let mut lowest_model_opt: Option<(String, f64)> = None;

            for m in quota.models {
                let name_lower = m.name.to_lowercase();
                let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
                if is_banned {
                    continue;
                }
                let pct = m.percentage as f64;
                if pct > 0.0 {
                    match lowest_model_opt {
                        Some((_, cur_min)) if pct < cur_min => {
                            lowest_model_opt = Some((m.name.clone(), pct));
                        }
                        None => {
                            lowest_model_opt = Some((m.name.clone(), pct));
                        }
                        _ => {}
                    }
                }
            }

            if let Some((_model_name, pct)) = lowest_model_opt {
                let acc_key = acc.email.trim().to_lowercase();
                if pct < threshold {
                    // Check deduplication: only alert if not alerted yet or if dropped by >= 1.0% further
                    let should_alert = {
                        let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
                        match alerts.get(&acc_key) {
                            Some(&last_pct) => {
                                if (last_pct - pct) >= 1.0 {
                                    alerts.insert(acc_key.clone(), pct);
                                    true
                                } else {
                                    false
                                }
                            }
                            None => {
                                alerts.insert(acc_key.clone(), pct);
                                true
                            }
                        }
                    };

                    if should_alert {
                        let (subj, html) = email_sender::render_quota_drop_email(
                            &acc.email,
                            pct,
                            threshold as u32,
                            m_name,
                            m_ip,
                        );
                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                        crate::error::record_ignored(
                            email_sender::dispatch_email_with_failover(
                                &subj,
                                &html,
                                &active_recipients,
                            ),
                            "dispatch_email_with_failover",
                        );
                        *last_alert = Utc::now().timestamp();
                        let mut st = LAST_STATUS.lock().await;
                        st.last_alert_sent = Some(*last_alert);
                        return;
                    }
                } else {
                    // Quota is healthy/reset above threshold, clear from alert cache
                    if let Ok(mut alerts) = LAST_QUOTA_ALERTED_PERCENT.lock() {
                        alerts.remove(&acc_key);
                    }
                }
            }
        }
    }
}

/// Check idle running projects sensor
pub(crate) async fn check_idle_projects_sensor(_m_name: &str, _m_ip: &str, last_alert: &mut i64) {
    // 1. If ANY prompt or conversation is actively running in Antigravity or repo_db, we are NOT idle!
    if crate::modules::repo_db::is_any_prompt_actively_running() {
        return;
    }

    // 2. Query live execution states
    let project_infos = crate::modules::repo_db::get_live_project_execution_info();
    if project_infos.is_empty() {
        return;
    }

    // 3. If any project is actively running, suppress idle alert completely
    let has_any_running = project_infos.iter().any(|p| p.is_running);
    if has_any_running {
        return;
    }

    // 4. Collect genuinely idle projects
    let idle_projects: Vec<_> = project_infos.into_iter().filter(|p| p.is_idle).collect();
    if idle_projects.is_empty() {
        return;
    }

    // 5. Resolve instance id and name for notification hub
    let (inst_id, inst_name) = {
        let mut resolved_id = "default".to_string();
        let mut resolved_name = "Default".to_string();
        if let Ok(running_projs) = crate::modules::repo_db::list_running_projects() {
            if let Some(first_p) = idle_projects.first() {
                if let Some(rp) = running_projs
                    .iter()
                    .find(|rp| rp.id == first_p.project_id || rp.repo_path == first_p.repo_path)
                {
                    resolved_id = rp.instance_id.clone();
                } else if let Some((_, inst)) = first_p.project_id.split_once("__") {
                    resolved_id = inst.to_string();
                }
            }
        }
        if let Ok(reg) = crate::modules::instance::load_registry() {
            if let Some(inst) = reg
                .instances
                .iter()
                .find(|i| i.id.eq_ignore_ascii_case(&resolved_id))
            {
                resolved_name = inst.name.clone();
            } else {
                resolved_name = resolved_id.clone();
            }
        }
        (resolved_id, resolved_name)
    };

    let now = Utc::now().timestamp();
    let idle_duration_secs = idle_projects
        .iter()
        .map(|p| now.saturating_sub(p.last_detected_at) as u64)
        .max()
        .unwrap_or(180)
        .max(180);

    // 6. Dispatch multi-channel idle alert via notification hub (Telegram & Email)
    let alert_res = crate::modules::notification_hub::dispatch_idle_mode_alert(
        &inst_id,
        &inst_name,
        idle_duration_secs,
    )
    .await;

    // Justification: best-effort idle alert notification for Telegram and Email; failure logged and does not affect sensor loop
    crate::error::record_ignored(
        alert_res.map(|_| ()),
        "check_idle_projects_sensor: dispatch_idle_mode_alert",
    );
    *last_alert = now;
}

/// Send workspace switched notification immediately
pub fn notify_workspace_switched(from_instance: &str, to_instance: &str, reason: &str) {
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(_) => return,
    };

    let is_enabled = settings.is_enabled;
    if !is_enabled {
        return;
    }

    let is_switch_notify_enabled = settings.notify_on_workspace_switch;
    if !is_switch_notify_enabled {
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

    let m_name = detect_machine_name();
    let m_ip = detect_local_ip();
    let (subj, html) = email_sender::render_workspace_switch_email(
        from_instance,
        to_instance,
        reason,
        &m_name,
        &m_ip,
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_sender::dispatch_email_with_failover(&subj, &html, &active_recipients),
        "dispatch_email_with_failover",
    );
}
