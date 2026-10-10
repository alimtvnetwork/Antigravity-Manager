//! email_arms_a — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::process::{Command, Stdio};

pub(crate) fn email_arm_help(args: &[String], is_json: bool, sub: &str) {
    println!("================================================================================");
    println!("  AGM EMAIL TELEMETRY, VAULT & REMOTE COMMAND CONTROL");
    println!("================================================================================");
    println!("CLI Subcommands:");
    println!(
        "  agm email [status] [--json]               Show email settings & dispatch status email"
    );
    println!("  agm email help                            Show guide & dispatch help email to recipients");
    println!("  agm email ls [--json]                     List configured mailboxes & recipients");
    println!(
        "  agm email add <email> <password> [opts]   Add SMTP/IMAP account (sends JSON self-email)"
    );
    println!("  agm email add <email> --recipient         Add notification recipient (sends JSON self-email)");
    println!("  agm email rm <seq|id|email>               Remove email account or recipient");
    println!(
        "  agm email mv <seq|id|email> --default     Promote mailbox account to default sender"
    );
    println!("  agm email export [-f <path>]              Export email config & encrypted secrets to JSON");
    println!("  agm email import [-f <path>]              Import email config bundle from JSON");
    println!();
    println!("Inbound Email Remote Command Subject Syntax:");
    println!("  <VM_NAME_OR_*> | <INSTANCE_SEQ_OR_REPO> | <ACTION>");
    println!("  Examples:");
    println!("    VM1 | 1 | help");
    println!("    VM1 | 1 | ff");
    println!("    VM1 | 1 | agm status");
    println!("    *   | gitmap | prompt Run full test suite");
    println!();

    if sub == "help" {
        let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
        let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
        let mut target_recipients: Vec<String> = recipients
            .iter()
            .filter(|r| r.is_active)
            .map(|r| r.email.clone())
            .collect();
        if target_recipients.is_empty() {
            if let Some(def_acc) = accounts
                .iter()
                .find(|a| a.is_default && a.is_active)
                .or_else(|| accounts.iter().find(|a| a.is_active))
            {
                target_recipients.push(def_acc.email.clone());
            }
        }
        if !target_recipients.is_empty() {
            let m_name = email_watcher::detect_machine_name();
            let m_ip = email_watcher::detect_local_ip();
            let (subject, html) = email_sender::render_help_email(&m_name, &m_ip);
            match email_sender::dispatch_email_with_failover(&subject, &html, &target_recipients) {
                Ok(res) => {
                    println!(
                                "[SUCCESS] Dispatched help instructions email via '{}' to {} recipient(s): {}",
                                res.used_account_email,
                                target_recipients.len(),
                                target_recipients.join(", ")
                            );
                }
                Err(e) => {
                    eprintln!("[WARN] Failed to dispatch help email: {}", e);
                }
            }
        }
    }
}

pub(crate) fn email_arm_status(args: &[String], is_json: bool) {
    let settings = email_vault_db::get_notification_settings().unwrap_or_default();
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    let default_acc = accounts
        .iter()
        .find(|a| a.is_default)
        .or_else(|| accounts.first());

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let active_acc = account::get_current_account().ok().flatten();
    let (immediate_quota, weekly_quota, tier) = match active_acc.as_ref() {
        Some(acc) => crate::common::extract_immediate_and_weekly_credits(acc),
        None => (0.0, 0.0, "NONE".to_string()),
    };
    let node_alias = supabase_sync::load_config()
        .map(|c| c.node_alias)
        .unwrap_or_else(|_| m_name.clone());
    let app_cfg = config::load_app_config().unwrap_or_default();
    let threshold_percent = app_cfg.auto_profile_switcher.low_quota_threshold_percent;
    let target_model = app_cfg.auto_profile_switcher.target_model.clone();

    let instances_list = instance::list_instances().unwrap_or_default();
    let running_instances = instances_list.iter().filter(|i| i.is_running).count();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = all_prompts
        .iter()
        .filter(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        .count();
    let has_images = all_prompts.iter().any(|p| {
        (p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
            && p.image_payload.is_some()
    });
    let prompts_resent = running_prompts > 0;

    let mut excluded = auto_switcher::get_active_in_use_account_ids();
    if let Some(ref acc) = active_acc {
        if !excluded.contains(&acc.id) {
            excluded.push(acc.id.clone());
        }
        if !excluded.contains(&acc.email) {
            excluded.push(acc.email.clone());
        }
    }
    let predicted_candidate = auto_switcher::select_next_best_profile(
        "default",
        &target_model,
        threshold_percent,
        &excluded,
    )
    .ok()
    .flatten();
    let predicted_next_account = predicted_candidate
        .as_ref()
        .map(|c| c.email.clone())
        .filter(|em| {
            active_acc
                .as_ref()
                .map(|a| !em.trim().eq_ignore_ascii_case(a.email.trim()))
                .unwrap_or(true)
        });
    let predicted_next_quota = predicted_candidate.as_ref().map(|c| c.quota_percent);

    let mut target_recipients: Vec<String> = recipients
        .iter()
        .filter(|r| r.is_active)
        .map(|r| r.email.clone())
        .collect();
    if target_recipients.is_empty() {
        if let Some(def_acc) = accounts
            .iter()
            .find(|a| a.is_default && a.is_active)
            .or_else(|| accounts.iter().find(|a| a.is_active))
        {
            target_recipients.push(def_acc.email.clone());
        }
    }

    let status_json = serde_json::json!({
        "event": "node_and_credits_status",
        "enabled": settings.is_enabled,
        "version": crate::common::VERSION,
        "machine_name": m_name,
        "node_alias": node_alias,
        "vm_alias": node_alias,
        "local_machine_name": m_name,
        "local_machine_ip": m_ip,
        "local_ip": m_ip,
        "polling_interval_minutes": settings.polling_interval_minutes,
        "inbox_check_interval_minutes": settings.inbox_check_interval_minutes,
        "default_sender": default_acc.map(|a| &a.email),
        "accounts_count": accounts.len(),
        "recipients_count": recipients.len(),
        "email_accounts_count": accounts.len(),
        "email_recipients_count": recipients.len(),
        "active_account": active_acc.as_ref().map(|a| &a.email),
        "previous_account": serde_json::Value::Null,
        "predicted_next_account": predicted_next_account,
        "predicted_next_quota_percent": predicted_next_quota,
        "selected_account": serde_json::Value::Null,
        "tier": tier,
        "credit_before_switch": immediate_quota,
        "immediate_quota_percent": immediate_quota,
        "weekly_quota_percent": weekly_quota,
        "threshold_percent": threshold_percent,
        "threshold_activated": threshold_percent,
        "instances_total": instances_list.len(),
        "instances_running": running_instances,
        "prompts_running": running_prompts,
        "prompts_resent": prompts_resent,
        "has_images": has_images,
    });

    if is_json {
        if !target_recipients.is_empty() {
            let subject = format!(
                "[AGM v{} | {} | {}] [JSON] Node & Credits Status",
                crate::common::VERSION,
                node_alias,
                m_ip
            );
            let json_body = serde_json::to_string_pretty(&status_json).unwrap_or_default();
            let _ = email_sender::dispatch_email_with_failover(
                &subject,
                &json_body,
                &target_recipients,
            );
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&status_json).unwrap_or_default()
        );
        return;
    }

    println!("[*] AGM Email Telemetry & Notification Status:");
    println!("    Enabled:               {}", settings.is_enabled);
    println!("    Node Identity:         {} ({})", node_alias, m_ip);
    println!(
        "    Default Sender:        {}",
        default_acc
            .map(|a| a.email.as_str())
            .unwrap_or("(None configured)")
    );
    println!("    Configured Mailboxes:  {}", accounts.len());
    println!("    Notifier Recipients:   {}", recipients.len());
    println!(
        "    Inbox Poll Interval:   {} min",
        settings.inbox_check_interval_minutes
    );
    println!(
        "    Active Account:        {} [{}] (Immediate: {:.1}%, Weekly: {:.1}%)",
        active_acc
            .as_ref()
            .map(|a| a.email.as_str())
            .unwrap_or("(None)"),
        tier,
        immediate_quota,
        weekly_quota
    );
    println!(
        "    Threshold Target:      {:.1}% ({})",
        threshold_percent, target_model
    );
    if let Some(ref pred) = predicted_next_account {
        println!(
            "    Predicted Next:        {} ({:.1}% quota)",
            pred,
            predicted_next_quota.unwrap_or(100.0)
        );
    }
    println!(
        "    Running Instances:     {} / {} | Running Prompts: {}",
        running_instances,
        instances_list.len(),
        running_prompts
    );
    println!(
        "    Prompts Resent:        {}",
        if prompts_resent {
            "Yes (Auto-Resumed via .antigravity_resume_task.json)"
        } else {
            "No"
        }
    );
    println!(
        "    Attached Images:       {}",
        if has_images {
            "Yes (Base64 payload preserved)"
        } else {
            "None"
        }
    );

    if !target_recipients.is_empty() {
        let subject = format!(
            "[AGM v{} | {} | {}] Node & Credits Status",
            crate::common::VERSION,
            node_alias,
            m_ip
        );
        let html = email_sender::render_node_credits_status_table_html(
            crate::common::VERSION,
            &node_alias,
            &m_ip,
            active_acc.as_ref().map(|a| a.email.as_str()),
            &tier,
            predicted_next_account.as_deref(),
            immediate_quota,
            weekly_quota,
            threshold_percent,
            running_instances,
            instances_list.len(),
            running_prompts,
            prompts_resent,
            has_images,
            accounts.len(),
            recipients.len(),
        );
        match email_sender::dispatch_email_with_failover(&subject, &html, &target_recipients) {
            Ok(res) => {
                println!(
                    "[SUCCESS] Dispatched status email via '{}' to {} recipient(s): {}",
                    res.used_account_email,
                    target_recipients.len(),
                    target_recipients.join(", ")
                );
            }
            Err(e) => {
                eprintln!("[WARN] Failed to dispatch status email: {}", e);
            }
        }
    }
}
