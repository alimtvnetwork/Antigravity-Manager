//! running_backup_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn cmd_backup_running_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Backup Running Prompts (Split SQLite):");
            println!("  agm backup [-file/-f <path.db>] [--json]   Create split SQLite snapshot of active/queued prompts");
            println!("  agm backup ls [--json]                     List all backup batches and prompt counts");
            println!("  agm backup clean [--force]                 Clean expired (1-day) restored backups, or force clean all");
            println!("\nAliases: agm backup, agm backpack, agm backup-running-prompts, agm brp");
            println!("\nExamples:");
            println!(
                "  agm backup                       # Snapshot all running prompts before rotation"
            );
            println!("  agm backup ls                    # List captured backup batches");
            println!("  agm backup clean --force         # Purge all stored prompt backups");
            return;
        }
        if first_lower == "ls" || first_lower == "list" {
            let is_json = args.iter().any(|a| a == "--json");
            let mut custom_file: Option<&str> = None;
            let mut i = 1;
            while i < args.len() {
                if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file")
                    && i + 1 < args.len()
                {
                    custom_file = Some(&args[i + 1]);
                    i += 2;
                    continue;
                }
                i += 1;
            }
            match backup_prompts_db::list_backup_batches(custom_file) {
                Ok(batches) => {
                    let storage = backup_prompts_db::get_storage_info(custom_file).ok();
                    if is_json {
                        let payload = serde_json::json!({
                            "storage": storage,
                            "batches": batches,
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&payload)
                                .unwrap_or_else(|_| "{}".to_string())
                        );
                        return;
                    }
                    println!("\n=== Backup Running Prompts Batches ===");
                    if let Some(st) = storage {
                        println!(
                            "  Database:   {}",
                            st["database_path"].as_str().unwrap_or("-")
                        );
                        println!("  Size:       {}", st["size_kb"].as_str().unwrap_or("-"));
                        println!(
                            "  Total:      {} prompt(s) across {} batch(es)",
                            st["total_prompt_records"], st["total_batches"]
                        );
                        println!(
                            "  Restored:   {} (Unrestored: {})",
                            st["restored_records"], st["unrestored_records"]
                        );
                        println!("  Retention:  1 day (restored records auto-cleaned after 24h)\n");
                    }
                    if batches.is_empty() {
                        println!("No prompt backup batches recorded yet.");
                    } else {
                        println!(
                            "{:<5} {:<24} {:<12} {:<22} CREATED AT",
                            "#", "BATCH ID", "PROMPTS", "RESTORED"
                        );
                        println!("{}", "-".repeat(80));
                        for (idx, b) in batches.iter().enumerate() {
                            let created_str = chrono::DateTime::from_timestamp(b.created_at, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "-".to_string());
                            let restored_label = if b.is_fully_restored { "Yes" } else { "No" };
                            println!(
                                "#{:<4} {:<24} {:<12} {:<22} {}",
                                idx + 1,
                                b.id,
                                b.prompts_count,
                                restored_label,
                                created_str
                            );
                        }
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to list backup batches: {}", e);
                    std::process::exit(1);
                }
            }
        }
        if first_lower == "clean" {
            let is_force = args.iter().any(|a| a == "--force" || a == "-f");
            let count = if is_force {
                backup_prompts_db::force_clean_all(None).unwrap_or(0)
            } else {
                backup_prompts_db::auto_cleanup_expired(None, 86400).unwrap_or(0)
            };
            println!("[INFO] Cleaned up {} prompt backup record(s).", count);
            return;
        }
    }

    let is_json = args.iter().any(|a| a == "--json");
    let mut custom_file: Option<&str> = None;
    let mut target_instance =
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        } else if (args[i] == "-i" || args[i] == "--instance" || args[i] == "-instance")
            && i + 1 < args.len()
        {
            let spec = &args[i + 1];
            target_instance = instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone());
            i += 2;
            continue;
        } else if args[i].starts_with("--instance=") || args[i].starts_with("-i=") {
            if let Some(spec) = args[i].split('=').nth(1) {
                target_instance =
                    instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.to_string());
            }
        }
        i += 1;
    }

    let _ = repo_db::backup_running_prompts(&target_instance);
    match backup_prompts_db::backup_active_running_prompts(Some(&target_instance), custom_file) {
        Ok((batch, records)) => {
            if is_json {
                let payload = serde_json::json!({
                    "batch_id": batch.id,
                    "file_path": batch.file_path,
                    "prompts_count": records.len(),
                    "created_at": batch.created_at,
                    "records": records,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM BACKUP RUNNING PROMPTS (Split SQLite)");
            println!(
                "================================================================================"
            );
            println!(
                "[Successfully secured {} prompt(s) in split SQLite DB: {}]\n",
                records.len(),
                batch.file_path
            );
            if records.is_empty() {
                println!("No active or running prompts found to back up.");
                return;
            }
            println!(
                "{:<5} {:<10} {:<24} {:<10} {:<16} PROMPT SNIPPET",
                "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
            );
            println!("{}", "-".repeat(110));
            for (idx, r) in records.iter().enumerate() {
                let short_id: String = r.prompt_id.chars().take(8).collect();
                let (snippet, _) = crate::common::truncate_words(&r.prompt_text, 15);
                let img_label = if r.has_images { "Yes" } else { "None" };
                println!(
                    "#{:<4} {:<10} {:<24} {:<10} {:<16} {}",
                    idx + 1,
                    short_id,
                    r.project_name,
                    r.status,
                    img_label,
                    snippet
                );
            }
            println!("\n[SUCCESS] Backup batch '{}' recorded.", batch.id);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to backup running prompts: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_restore_running_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Restore Running Prompts (Split SQLite):");
            println!("  agm restore [--keep/-k] [--json] [-file/-f <path>]");
            println!("  Restores unrestored prompts into active execution queue and re-injects to workspaces.");
            println!(
                "\nAliases: agm restore, agm restore-running-prompts, agm rrp, agm resend-running"
            );
            println!("\nOptions:");
            println!("    --keep, -k      Preserve backup records as unrestored without starting 1-day retention timer");
            println!("    --json          Output pure JSON restored records payload");
            println!("    -f, --file      Target custom SQLite database file");
            println!("\nExamples:");
            println!(
                "  agm restore                      # Restore & re-inject all backed-up prompts"
            );
            println!("  agm restore --keep               # Restore prompts without clearing backup state");
            println!("  agm restore --json               # Restore and print JSON payload");
            return;
        }
    }

    let is_json = args.iter().any(|a| a == "--json");
    let keep_backup = args.iter().any(|a| a == "--keep" || a == "-k");
    let mut custom_file: Option<&str> = None;
    let mut target_instance =
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        } else if (args[i] == "-i" || args[i] == "--instance" || args[i] == "-instance")
            && i + 1 < args.len()
        {
            let spec = &args[i + 1];
            target_instance = instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone());
            i += 2;
            continue;
        } else if args[i].starts_with("--instance=") || args[i].starts_with("-i=") {
            if let Some(spec) = args[i].split('=').nth(1) {
                target_instance =
                    instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.to_string());
            }
        }
        i += 1;
    }

    let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);
    let _ = repo_db::dispatch_running_prompts(&target_instance);
    match backup_prompts_db::restore_running_prompts(
        Some(&target_instance),
        keep_backup,
        custom_file,
    ) {
        Ok(records) => {
            if is_json {
                let payload = serde_json::json!({
                    "restored_count": records.len(),
                    "keep_backup": keep_backup,
                    "records": records,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM RESTORE RUNNING PROMPTS");
            println!(
                "================================================================================"
            );
            if records.is_empty() {
                println!("No unrestored prompts found in backup database.");
                return;
            }
            println!(
                "[Successfully restored and re-enqueued {} prompt(s)]\n",
                records.len()
            );
            println!(
                "{:<5} {:<10} {:<24} {:<10} {:<16} PROMPT SNIPPET",
                "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
            );
            println!("{}", "-".repeat(110));
            for (idx, r) in records.iter().enumerate() {
                let short_id: String = r.prompt_id.chars().take(8).collect();
                let (snippet, _) = crate::common::truncate_words(&r.prompt_text, 15);
                let img_label = if r.has_images { "Yes" } else { "None" };
                let status_label = if r.status == "queued" {
                    "queued"
                } else {
                    "running"
                };
                println!(
                    "#{:<4} {:<10} {:<24} {:<10} {:<16} {}",
                    idx + 1,
                    short_id,
                    r.project_name,
                    status_label,
                    img_label,
                    snippet
                );
            }
            println!();
            if keep_backup {
                println!("[INFO] Backup records preserved as unrestored (--keep specified).");
            } else {
                println!("[INFO] Marked records as restored. Will be automatically cleaned up after 1 day.");
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to restore running prompts: {}", e);
            std::process::exit(1);
        }
    }
}
