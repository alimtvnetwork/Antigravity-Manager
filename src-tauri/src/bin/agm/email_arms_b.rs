//! email_arms_b — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::email_vault_db::NotifyRecipientInput;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
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
        eprintln!("Usage: agm email add <email> [password] [--smtp-host H] [--smtp-port P] [--imap-host H] [--imap-port P] [--encryption SSL|TLS] [--default] [--recipient]");
        std::process::exit(1);
    }

    let is_recipient = rest.iter().any(|a| a == "--recipient" || a == "-r");
    let is_default = rest.iter().any(|a| a == "--default" || a == "-d");
    let ask_password = rest.iter().any(|a| a == "--ask-password");
    let mut email_addr = String::new();
    let mut password: Option<String> = None;
    let mut smtp_host: Option<String> = None;
    let mut smtp_port: u16 = 587;
    let mut imap_host: Option<String> = None;
    let mut imap_port: u16 = 993;
    let mut encryption_opt: Option<String> = None;
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
        } else if (arg == "--encryption" || arg == "--security") && i + 1 < rest.len() {
            encryption_opt = Some(rest[i + 1].trim().to_uppercase());
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

    if !is_recipient && password.is_none() && (ask_password || smtp_host.is_some()) {
        use std::io::Write;
        eprint!("Enter password for {}: ", email_addr);
        let _ = std::io::stderr().flush();
        let mut buf = String::new();
        if std::io::stdin().read_line(&mut buf).is_ok() {
            let trimmed = buf.trim().to_string();
            if !trimmed.is_empty() {
                password = Some(trimmed);
            }
        }
    }

    if is_recipient || password.is_none() {
        let input = email_vault_db::NotifyRecipientInput {
            email: email_addr.clone(),
            group_name: Some("default".to_string()),
            is_active: Some(true),
        };
        match email_vault_db::add_notify_recipient_outcome(input) {
            Ok(outcome) => {
                let rec = outcome.recipient().clone();
                if outcome.is_added() {
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
                } else {
                    println!(
                        "[INFO] Notification recipient '{}' already existed (ID: {}); not added again.",
                        rec.email, rec.id
                    );
                }
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
    let eff_enc = encryption_opt.unwrap_or_else(|| {
        if smtp_port == 465 || imap_port == 993 {
            "SSL".to_string()
        } else {
            "TLS".to_string()
        }
    });

    let input = email_vault_db::EmailAccountInput {
        id: None,
        alias: eff_alias,
        email: email_addr,
        password,
        smtp_host: eff_smtp,
        smtp_port,
        imap_host: eff_imap,
        imap_port,
        encryption_type: eff_enc,
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

/// Mask an email for display: first character of the local part plus domain.
/// Never prints a full address (plan 88, subtask 003).
fn mask_email(normalized_email: &str) -> String {
    let mut parts = normalized_email.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    let first: String = local.chars().take(1).collect();
    if domain.is_empty() {
        format!("{}***", first)
    } else {
        format!("{}***@{}", first, domain)
    }
}

pub(crate) fn email_arm_dedupe(args: &[String], is_json: bool) {
    let is_apply = args.iter().any(|a| a == "--apply");

    let groups = match email_vault_db::find_duplicate_recipients() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("[ERROR] Failed to scan duplicate recipients: {}", e);
            std::process::exit(1);
        }
    };

    if !is_apply {
        if is_json {
            let masked: Vec<serde_json::Value> = groups
                .iter()
                .map(|g| {
                    serde_json::json!({
                        "masked_email": mask_email(&g.normalized_email),
                        "row_count": g.row_count,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "duplicate_groups": masked,
                    "group_count": groups.len(),
                    "applied": false,
                }))
                .unwrap_or_default()
            );
            return;
        }
        if groups.is_empty() {
            println!("[OK] No duplicate notification recipients found.");
            return;
        }
        println!(
            "Duplicate notification recipient groups ({}):",
            groups.len()
        );
        for g in &groups {
            println!("  {} — {} rows", mask_email(&g.normalized_email), g.row_count);
        }
        println!("Dry run only. Re-run with --apply to merge (a timestamped DB backup is taken first).");
        return;
    }

    if groups.is_empty() {
        println!("[OK] No duplicate notification recipients found; nothing to merge.");
        return;
    }

    // Validate before deleting: take the timestamped DB backup first and
    // abort if the copy fails.
    let db_path = match email_vault_db::get_email_vault_db_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to resolve email vault DB path: {}", e);
            std::process::exit(1);
        }
    };
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("email_vault.db.bak-{}", stamp);
    let backup_path = db_path.with_file_name(&backup_name);
    if let Err(e) = fs::copy(&db_path, &backup_path) {
        eprintln!(
            "[ERROR] Failed to back up email vault DB before dedupe: {}. Aborting; nothing was changed.",
            e
        );
        std::process::exit(1);
    }

    match email_vault_db::merge_duplicate_recipients() {
        Ok(deleted) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "duplicate_groups": groups.len(),
                        "rows_deleted": deleted,
                        "backup_file": backup_name,
                        "applied": true,
                    }))
                    .unwrap_or_default()
                );
            } else {
                println!(
                    "[SUCCESS] Merged {} duplicate group(s), deleted {} row(s). DB backup saved as '{}' next to the email vault DB.",
                    groups.len(),
                    deleted,
                    backup_name
                );
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to merge duplicate recipients: {}", e);
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
    let mut target_path: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut password_opt: Option<String> = None;
    let mut ask_password = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-f" || arg == "--file") && i + 1 < args.len() {
            target_path = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if (arg == "--json" || arg == "--payload") && i + 1 < args.len() {
            inline_json = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if (arg == "-p" || arg == "--password") && i + 1 < args.len() {
            password_opt = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if arg == "--ask-password" || arg == "--prompt-password" {
            ask_password = true;
            i += 1;
            continue;
        } else if !arg.starts_with('-') {
            if arg.trim_start().starts_with('{') && inline_json.is_none() {
                inline_json = Some(arg.clone());
            } else if target_path.is_none() {
                target_path = Some(arg.clone());
            }
        }
        i += 1;
    }

    let (source_label, payload) = if let Some(raw_json) = inline_json {
        ("<inline-json>".to_string(), raw_json)
    } else {
        let path_str = target_path.unwrap_or_else(|| "agm-email-config.json".to_string());
        let resolved = json_envelope::resolve_relative_json_path(&path_str);
        match fs::read_to_string(&resolved) {
            Ok(p) => (path_str, p),
            Err(e) => {
                eprintln!("[ERROR] Failed to read '{}': {}", path_str, e);
                std::process::exit(1);
            }
        }
    };

    let (detected_email, has_embedded_pwd) = email_io::extract_email_and_has_password(&payload);
    if password_opt.is_none() && (ask_password || !has_embedded_pwd) {
        use std::io::Write;
        let prompt_email = detected_email.as_deref().unwrap_or("email account");
        eprint!("Enter password for {}: ", prompt_email);
        let _ = std::io::stderr().flush();
        let mut buf = String::new();
        if std::io::stdin().read_line(&mut buf).is_ok() {
            let trimmed = buf.trim().to_string();
            if !trimmed.is_empty() {
                password_opt = Some(trimmed);
            }
        }
    }

    match email_io::import_from_json_with_password(&payload, password_opt.as_deref()) {
        Ok(sum) => println!(
            "[SUCCESS] Imported {} account(s) and {} recipient(s) from '{}' ({} recipient duplicate(s) skipped, {} failed).",
            sum.accounts_imported,
            sum.recipients_imported,
            source_label,
            sum.recipients_skipped_duplicates,
            sum.recipients_failed
        ),
        Err(e) => {
            eprintln!("[ERROR] Failed to import email config: {}", e);
            std::process::exit(1);
        }
    }
}
