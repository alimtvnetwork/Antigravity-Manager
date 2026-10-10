//! email_arms_b — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::email_vault_db::NotifyRecipientInput;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn email_arm_ls(args: &[String], is_json: bool) {
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();

    if is_json {
        let out = serde_json::json!({
            "accounts": accounts,
            "recipients": recipients,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return;
    }

    println!("\nConfigured Email Accounts ({} total):", accounts.len());
    println!(
        "{:<5} {:<20} {:<30} {:<22} {:<22} {:<8} ACTIVE",
        "SEQ", "ID", "EMAIL", "SMTP", "IMAP", "DEFAULT"
    );
    println!("{}", "-".repeat(115));
    for (idx, a) in accounts.iter().enumerate() {
        let short_id: String = a.id.chars().take(18).collect();
        let smtp = format!("{}:{}", a.smtp_host, a.smtp_port);
        let imap = format!("{}:{}", a.imap_host, a.imap_port);
        println!(
            "#{:<4} {:<20} {:<30} {:<22} {:<22} {:<8} {}",
            idx + 1,
            short_id,
            a.email,
            smtp,
            imap,
            a.is_default,
            a.is_active
        );
    }

    println!("\nNotification Recipients ({} total):", recipients.len());
    println!(
        "{:<5} {:<20} {:<32} {:<14} ACTIVE",
        "SEQ", "ID", "EMAIL", "GROUP"
    );
    println!("{}", "-".repeat(85));
    for (idx, r) in recipients.iter().enumerate() {
        let short_id: String = r.id.chars().take(18).collect();
        println!(
            "#{:<4} {:<20} {:<32} {:<14} {}",
            idx + 1,
            short_id,
            r.email,
            r.group_name,
            r.is_active
        );
    }
    println!();
}

pub(crate) fn email_arm_add(args: &[String], is_json: bool) {
    let rest = &args[1..];
    if rest.is_empty() {
        eprintln!("Usage: agm email add <email> [password] [--smtp-host H] [--imap-host H] [--default] [--recipient]");
        std::process::exit(1);
    }

    let is_recipient = rest.iter().any(|a| a == "--recipient" || a == "-r");
    let is_default = rest.iter().any(|a| a == "--default" || a == "-d");
    let mut email_addr = String::new();
    let mut password: Option<String> = None;
    let mut smtp_host: Option<String> = None;
    let mut smtp_port: u16 = 587;
    let mut imap_host: Option<String> = None;
    let mut imap_port: u16 = 993;
    let mut alias: Option<String> = None;

    let mut i = 0;
    while i < rest.len() {
        let arg = &rest[i];
        if arg == "--smtp-host" && i + 1 < rest.len() {
            smtp_host = Some(rest[i + 1].clone());
            i += 2;
            continue;
        } else if arg == "--smtp-port" && i + 1 < rest.len() {
            smtp_port = rest[i + 1].parse().unwrap_or(587);
            i += 2;
            continue;
        } else if arg == "--imap-host" && i + 1 < rest.len() {
            imap_host = Some(rest[i + 1].clone());
            i += 2;
            continue;
        } else if arg == "--imap-port" && i + 1 < rest.len() {
            imap_port = rest[i + 1].parse().unwrap_or(993);
            i += 2;
            continue;
        } else if arg == "--alias" && i + 1 < rest.len() {
            alias = Some(rest[i + 1].clone());
            i += 2;
            continue;
        } else if arg.starts_with('-') {
            i += 1;
            continue;
        } else if email_addr.is_empty() {
            email_addr = arg.clone();
        } else if password.is_none() {
            password = Some(arg.clone());
        }
        i += 1;
    }

    if email_addr.is_empty() {
        eprintln!("[ERROR] Email address is required.");
        std::process::exit(1);
    }

    if is_recipient || password.is_none() {
        let input = email_vault_db::NotifyRecipientInput {
            email: email_addr.clone(),
            group_name: Some("default".to_string()),
            is_active: Some(true),
        };
        match email_vault_db::add_notify_recipient(input) {
            Ok(rec) => {
                notification_hub::notify_email_config_added(
                    "Notification Recipient Added (CLI)",
                    serde_json::json!({
                        "event": "notify_recipient_added",
                        "id": rec.id,
                        "email": rec.email,
                        "group_name": rec.group_name,
                        "is_active": rec.is_active,
                    }),
                );
                println!(
                            "[SUCCESS] Added notification recipient '{}' (ID: {}) and queued JSON self-email.",
                            rec.email, rec.id
                        );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to add recipient: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let domain = email_addr.split('@').nth(1).unwrap_or("gmail.com");
    let eff_smtp = smtp_host.unwrap_or_else(|| format!("smtp.{}", domain));
    let eff_imap = imap_host.unwrap_or_else(|| format!("imap.{}", domain));
    let eff_alias = alias.unwrap_or_else(|| email_addr.clone());
    let existing = email_vault_db::list_email_accounts().unwrap_or_default();
    let eff_default = is_default || existing.is_empty();

    let input = email_vault_db::EmailAccountInput {
        id: None,
        alias: eff_alias,
        email: email_addr,
        password,
        smtp_host: eff_smtp,
        smtp_port,
        imap_host: eff_imap,
        imap_port,
        encryption_type: "TLS".to_string(),
        is_default: eff_default,
        is_active: true,
    };

    match email_vault_db::upsert_email_account(input) {
        Ok(acc) => {
            notification_hub::notify_email_config_added(
                "Email Account Added (CLI)",
                serde_json::json!({
                    "event": "email_account_added",
                    "account_id": acc.id,
                    "alias": acc.alias,
                    "email": acc.email,
                    "smtp_host": acc.smtp_host,
                    "smtp_port": acc.smtp_port,
                    "imap_host": acc.imap_host,
                    "imap_port": acc.imap_port,
                    "is_default": acc.is_default,
                    "is_active": acc.is_active,
                }),
            );
            println!(
                        "[SUCCESS] Added email account '{}' (ID: {}, default: {}) and dispatched JSON self-email.",
                        acc.email, acc.id, acc.is_default
                    );
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to add email account: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn email_arm_rm(args: &[String], is_json: bool) {
    let target = match args.get(1) {
        Some(t) => t.trim(),
        None => {
            eprintln!("Usage: agm email rm <seq|id|email>");
            std::process::exit(1);
        }
    };

    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let clean_seq = target.trim_start_matches('#');
    let matched_acc = if let Ok(seq) = clean_seq.parse::<usize>() {
        if seq >= 1 && seq <= accounts.len() {
            Some(accounts[seq - 1].clone())
        } else {
            None
        }
    } else {
        accounts
            .iter()
            .find(|a| {
                a.id.eq_ignore_ascii_case(target)
                    || a.email.eq_ignore_ascii_case(target)
                    || a.alias.eq_ignore_ascii_case(target)
            })
            .cloned()
    };

    if let Some(acc) = matched_acc {
        match email_vault_db::delete_email_account(&acc.id) {
            Ok(_) => {
                println!(
                    "[SUCCESS] Removed email account '{}' (ID: {}).",
                    acc.email, acc.id
                );
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to remove email account: {}", e);
                std::process::exit(1);
            }
        }
    }

    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    if let Some(rec) = recipients
        .iter()
        .find(|r| r.id.eq_ignore_ascii_case(target) || r.email.eq_ignore_ascii_case(target))
    {
        match email_vault_db::delete_notify_recipient(&rec.id) {
            Ok(_) => {
                println!(
                    "[SUCCESS] Removed notification recipient '{}' (ID: {}).",
                    rec.email, rec.id
                );
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to remove recipient: {}", e);
                std::process::exit(1);
            }
        }
    }

    eprintln!(
        "[ERROR] No email account or recipient matched '{}'.",
        target
    );
    std::process::exit(1);
}

pub(crate) fn email_arm_mv(args: &[String], is_json: bool) {
    let target = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.trim())
        .unwrap_or("");
    if target.is_empty() {
        eprintln!("Usage: agm email mv <seq|id|email> --default");
        std::process::exit(1);
    }

    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let clean_seq = target.trim_start_matches('#');
    let matched = if let Ok(seq) = clean_seq.parse::<usize>() {
        if seq >= 1 && seq <= accounts.len() {
            Some(accounts[seq - 1].clone())
        } else {
            None
        }
    } else {
        accounts
            .iter()
            .find(|a| {
                a.id.eq_ignore_ascii_case(target)
                    || a.email.eq_ignore_ascii_case(target)
                    || a.alias.eq_ignore_ascii_case(target)
            })
            .cloned()
    };

    let Some(acc) = matched else {
        eprintln!("[ERROR] Email account '{}' not found.", target);
        std::process::exit(1);
    };

    match email_vault_db::set_default_email_account(&acc.id) {
        Ok(_) => println!(
            "[SUCCESS] Set '{}' (ID: {}) as the default email account.",
            acc.email, acc.id
        ),
        Err(e) => {
            eprintln!("[ERROR] Failed to set default email account: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn email_arm_export(args: &[String], is_json: bool) {
    let mut out_file: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file") && i + 1 < args.len() {
            out_file = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if !args[i].starts_with('-') && out_file.is_none() {
            out_file = Some(args[i].clone());
        }
        i += 1;
    }

    let json = match email_io::export_to_json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[ERROR] Failed to export email config: {}", e);
            std::process::exit(1);
        }
    };

    let target_path = out_file.unwrap_or_else(|| "agm-email-config.json".to_string());
    if let Err(e) = fs::write(&target_path, &json) {
        eprintln!("[ERROR] Failed to write {}: {}", target_path, e);
        std::process::exit(1);
    }
    println!(
        "[SUCCESS] Exported email configuration bundle to '{}'.",
        target_path
    );
}

pub(crate) fn email_arm_import(args: &[String], is_json: bool) {
    let target_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| "agm-email-config.json".to_string());
    let payload = match fs::read_to_string(&target_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to read '{}': {}", target_path, e);
            std::process::exit(1);
        }
    };
    match email_io::import_from_json(&payload) {
        Ok(sum) => println!(
            "[SUCCESS] Imported {} account(s) and {} recipient(s) from '{}'.",
            sum.accounts_imported, sum.recipients_imported, target_path
        ),
        Err(e) => {
            eprintln!("[ERROR] Failed to import email config: {}", e);
            std::process::exit(1);
        }
    }
}
