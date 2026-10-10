//! email_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::email_vault_db::NotifyRecipientInput;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;

pub(crate) fn cmd_broadcast_email(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Broadcast Email Management:");
            println!("  agm broadcast-email ls [--json]");
            println!("  agm broadcast-email add <email> [alias]");
            println!("  agm broadcast-email edit <id|email> <new_email>");
            println!("  agm broadcast-email rm <id|email>");
            println!("  agm broadcast-email send-to-all <subj> <body>");
            println!("  agm broadcast-email send <email> <subj> <body>");
            println!("  agm broadcast-email send-help, sh");
            println!("  agm broadcast-email test");
            println!("\nDescription:");
            println!("  Manages authorized email broadcast notification lists and dispatches status alerts.");
            println!("\nAliases: agm broadcast-email");
            println!("\nSubcommands:");
            println!("  ls                  List configured broadcast recipients");
            println!("  add <email> [alias] Add a new recipient to the notification list");
            println!("  edit <id> <email>   Update recipient address");
            println!("  rm <id|email>       Remove recipient");
            println!("  send-to-all <s > <b> Dispatch message to all active recipients");
            println!("  send <email> <s > <b> Dispatch message to single recipient");
            println!("  send-help, sh       Dispatch help cheat sheet to all recipients");
            println!("  test                Send test connectivity ping email");
            println!("\nExamples:");
            println!("  agm broadcast-email ls              # List notification recipients");
            println!(
                "  agm broadcast-email test            # Verify email delivery with test ping"
            );
            println!("  agm broadcast-email send-help       # Email cheat sheet to all recipients");
            return;
        }
        if first_lower == "ls" || first_lower == "list" {
            let is_json = args.iter().any(|a| a == "--json");
            match email_vault_db::list_notify_recipients() {
                Ok(recipients) => {
                    if is_json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&recipients)
                                .unwrap_or_else(|_| "[]".to_string())
                        );
                        return;
                    }
                    println!(
                        "\n=== Broadcast Email Recipients ({} configured) ===",
                        recipients.len()
                    );
                    println!(
                        "{:<5} {:<24} {:<32} {:<10} GROUP",
                        "SEQ", "ID", "EMAIL", "ACTIVE"
                    );
                    println!("{}", "-".repeat(85));
                    for (idx, r) in recipients.iter().enumerate() {
                        let sid: String = r.id.chars().take(22).collect();
                        let active_lbl = if r.is_active { "Yes" } else { "No" };
                        let grp = &r.group_name;
                        println!(
                            "#{:<4} {:<24} {:<32} {:<10} {}",
                            idx + 1,
                            sid,
                            r.email,
                            active_lbl,
                            grp
                        );
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to list recipients: {}", e);
                    std::process::exit(1);
                }
            }
        }
        if first_lower == "add" {
            if let Some(email) = args.get(1) {
                let alias = args.get(2).map(|s| s.as_str());
                let input = NotifyRecipientInput {
                    email: email.to_string(),
                    group_name: alias
                        .map(|s| s.to_string())
                        .or(Some("broadcast".to_string())),
                    is_active: Some(true),
                };
                match email_vault_db::add_notify_recipient(input) {
                    Ok(r) => println!("[SUCCESS] Added recipient '{}' ({})", r.email, r.id),
                    Err(e) => eprintln!("[ERROR] Failed to add recipient: {}", e),
                }
                return;
            } else {
                eprintln!("Usage: agm broadcast-email add <email> [alias]");
                return;
            }
        }
        if first_lower == "edit" {
            if let (Some(target), Some(new_email)) = (args.get(1), args.get(2)) {
                if let Ok(all) = email_vault_db::list_notify_recipients() {
                    if let Some(existing) = all
                        .into_iter()
                        .find(|r| r.email.eq_ignore_ascii_case(target) || r.id == *target)
                    {
                        let _ = email_vault_db::delete_notify_recipient(&existing.id);
                        let input = NotifyRecipientInput {
                            email: new_email.to_string(),
                            group_name: Some(existing.group_name),
                            is_active: Some(existing.is_active),
                        };
                        match email_vault_db::add_notify_recipient(input) {
                            Ok(r) => {
                                println!("[SUCCESS] Updated recipient to '{}' ({})", r.email, r.id)
                            }
                            Err(e) => eprintln!("[ERROR] Failed to update recipient: {}", e),
                        }
                    } else {
                        eprintln!("[ERROR] Recipient '{}' not found.", target);
                    }
                }
                return;
            } else {
                eprintln!("Usage: agm broadcast-email edit <id|email> <new_email>");
                return;
            }
        }
        if first_lower == "rm" || first_lower == "remove" {
            if let Some(target) = args.get(1) {
                let id = if let Ok(all) = email_vault_db::list_notify_recipients() {
                    all.into_iter()
                        .find(|r| r.email.eq_ignore_ascii_case(target) || r.id == *target)
                        .map(|r| r.id)
                        .unwrap_or_else(|| target.to_string())
                } else {
                    target.to_string()
                };
                match email_vault_db::delete_notify_recipient(&id) {
                    Ok(_) => println!("[SUCCESS] Removed recipient '{}'", target),
                    Err(e) => eprintln!("[ERROR] Failed to remove recipient: {}", e),
                }
                return;
            } else {
                eprintln!("Usage: agm broadcast-email rm <id|email>");
                return;
            }
        }
        if first_lower == "send-to-all" {
            if let (Some(subj), Some(body)) = (args.get(1), args.get(2)) {
                let active_recipients: Vec<String> = email_vault_db::list_notify_recipients()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|r| r.is_active)
                    .map(|r| r.email)
                    .collect();
                if active_recipients.is_empty() {
                    eprintln!("[WARN] No active broadcast recipients configured.");
                    return;
                }
                match email_sender::dispatch_email_with_failover(subj, body, &active_recipients) {
                    Ok(res) => println!(
                        "[SUCCESS] Dispatched email to {} recipient(s) via '{}'",
                        active_recipients.len(),
                        res.used_account_email
                    ),
                    Err(e) => eprintln!("[ERROR] Failed to dispatch email: {}", e),
                }
                return;
            } else {
                eprintln!("Usage: agm broadcast-email send-to-all <subject> <body>");
                return;
            }
        }
        if first_lower == "send" {
            if let Some(sub) = args.get(1) {
                if sub.eq_ignore_ascii_case("help") || sub.eq_ignore_ascii_case("-h") {
                    println!("Usage: agm broadcast-email send <email> <subject> <body>");
                    println!("       agm broadcast-email send help (or send-help / sh) - send help cheat sheet to all");
                    return;
                }
            }
            if let (Some(email), Some(subj), Some(body)) = (args.get(1), args.get(2), args.get(3)) {
                match email_sender::dispatch_email_with_failover(
                    subj,
                    body,
                    std::slice::from_ref(email),
                ) {
                    Ok(res) => println!(
                        "[SUCCESS] Dispatched email to '{}' via '{}'",
                        email, res.used_account_email
                    ),
                    Err(e) => eprintln!("[ERROR] Failed to dispatch email: {}", e),
                }
                return;
            } else {
                eprintln!("Usage: agm broadcast-email send <email> <subject> <body>");
                return;
            }
        }
        if first_lower == "send help" || first_lower == "send-help" || first_lower == "sh" {
            let m_name = email_watcher::detect_machine_name();
            let m_ip = email_watcher::detect_local_ip();
            let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);
            let active_recipients: Vec<String> = email_vault_db::list_notify_recipients()
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.is_active)
                .map(|r| r.email)
                .collect();
            if active_recipients.is_empty() {
                eprintln!("[WARN] No active broadcast recipients configured.");
                return;
            }
            match email_sender::dispatch_email_with_failover(&subj, &body, &active_recipients) {
                Ok(res) => println!(
                    "[SUCCESS] Dispatched cheat sheet email to {} recipient(s) via '{}'",
                    active_recipients.len(),
                    res.used_account_email
                ),
                Err(e) => eprintln!("[ERROR] Failed to dispatch email: {}", e),
            }
            return;
        }
        if first_lower == "test" {
            let active_recipients: Vec<String> = email_vault_db::list_notify_recipients()
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.is_active)
                .map(|r| r.email)
                .collect();
            let target = active_recipients
                .first()
                .cloned()
                .unwrap_or_else(|| "dev@riseup-asia.com".to_string());
            let m_name = email_watcher::detect_machine_name();
            let m_ip = email_watcher::detect_local_ip();
            let now = Utc::now().timestamp();
            let (subj, body) =
                email_sender::render_test_ping_email("Broadcast-Test", &m_name, &m_ip, now);
            match email_sender::dispatch_email_with_failover(
                &subj,
                &body,
                std::slice::from_ref(&target),
            ) {
                Ok(res) => println!(
                    "[SUCCESS] Broadcast test ping delivered to '{}' via '{}'",
                    target, res.used_account_email
                ),
                Err(e) => eprintln!("[ERROR] Broadcast test ping failed: {}", e),
            }
            return;
        }
    }

    println!("AGM Broadcast Email CLI. Run 'agm broadcast-email help' for commands.");
}
