//! running_backup_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

/// Helper to parse instance identifier and strip conversational noise tokens.
///
/// Strips noise tokens: `"running"`, `"running-prompts"`, `"rp"`, `"prompts"`, `"active"`.
/// Supports `-i <id>`, `--instance <id>`, `--instance=<id>`, and positional instance argument
/// (e.g. `#1`, `default`, `worker-1`, `a-6650`) resolved via `instance::resolve_instance_id`.
/// Returns `(target_instance, remaining_args)`.
pub(crate) fn parse_instance_and_noise_args(
    args: &[String],
    is_restore: bool,
) -> (Option<String>, Vec<String>) {
    let noise_tokens = ["running", "running-prompts", "rp", "prompts", "active"];
    let subcommands = ["help", "--help", "-h", "ls", "list", "clean"];

    let mut target_instance: Option<String> = None;
    let mut remaining_args = Vec::new();
    let mut i = 0;

    // Detect if the first argument is "running", strip it, and if the next argument does not start with '-', bind it as target instance
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("running") {
            i = 1;
            if i < args.len() {
                let next = &args[i];
                if !next.starts_with('-')
                    && !subcommands.iter().any(|s| next.eq_ignore_ascii_case(s))
                {
                    let resolved =
                        instance::resolve_instance_id(next).unwrap_or_else(|_| next.clone());
                    target_instance = Some(resolved);
                    i = 2;
                }
            }
        }
    }

    while i < args.len() {
        let arg = &args[i];
        let arg_lower = arg.to_lowercase();

        // 1. Explicit instance flag: -i <id>, --instance <id>, -instance <id>
        if (arg == "-i" || arg == "--instance" || arg == "-instance") && i + 1 < args.len() {
            let spec = &args[i + 1];
            let resolved = instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone());
            target_instance = Some(resolved);
            i += 2;
            continue;
        }

        // 2. Explicit instance flag: --instance=<id>, -i=<id>
        if arg.starts_with("--instance=") || arg.starts_with("-i=") {
            if let Some(spec) = arg.split('=').nth(1) {
                let resolved =
                    instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.to_string());
                target_instance = Some(resolved);
            }
            i += 1;
            continue;
        }

        // 3. Flags with values to preserve: -f <path>, --file <path>, -file <path>
        if (arg == "-f" || arg == "--file" || arg == "-file") && i + 1 < args.len() {
            remaining_args.push(arg.clone());
            remaining_args.push(args[i + 1].clone());
            i += 2;
            continue;
        }

        // 4. Strip conversational noise tokens
        if noise_tokens.iter().any(|t| arg_lower == *t) {
            i += 1;
            continue;
        }

        // 5. Preserve known subcommands and general flags
        if subcommands.iter().any(|s| arg_lower == *s) || arg.starts_with('-') {
            remaining_args.push(arg.clone());
            i += 1;
            continue;
        }

        // 6. Positional instance argument (e.g. #1, default, worker-1, a-6650)
        if target_instance.is_none() {
            let resolved = instance::resolve_instance_id(arg).unwrap_or_else(|_| arg.clone());
            target_instance = Some(resolved);
            i += 1;
            continue;
        }

        // Anything else goes into remaining_args
        remaining_args.push(arg.clone());
        i += 1;
    }

    if !is_restore && target_instance.is_none() {
        target_instance =
            Some(instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string()));
    }

    (target_instance, remaining_args)
}

pub(crate) fn cmd_backup_running_prompts(args: &[String]) {
    let (target_instance_opt, remaining_args) = parse_instance_and_noise_args(args, false);
    let target_instance = target_instance_opt.unwrap_or_else(|| {
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
    });

    if let Some(first) = remaining_args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Backup Running Prompts (Split SQLite):");
            println!("  agm backup [-file/-f <path.db>] [--json] [-i/--instance <id>] [instance]");
            println!("  agm backup running [instance]              Create split SQLite snapshot of active/queued prompts");
            println!("  agm backup ls [--json]                     List all backup batches and prompt counts");
            println!("  agm backup clean [--force]                 Clean expired (1-day) restored backups, or force clean all");
            println!("\nAliases: agm backup, agm backpack, agm backup-running-prompts, agm brp");
            println!("\nExamples:");
            println!(
                "  agm backup                       # Snapshot running prompts for active instance"
            );
            println!(
                "  agm backup #1                    # Snapshot running prompts for instance #1"
            );
            println!("  agm backup running default       # Snapshot running prompts for default instance");
            println!("  agm backup ls                    # List captured backup batches");
            println!("  agm backup clean --force         # Purge all stored prompt backups");
            return;
        }
        if first_lower == "ls" || first_lower == "list" {
            let is_json = remaining_args.iter().any(|a| a == "--json" || a == "-j");
            let mut custom_file: Option<&str> = None;
            let mut i = 1;
            while i < remaining_args.len() {
                if (remaining_args[i] == "-f"
                    || remaining_args[i] == "--file"
                    || remaining_args[i] == "-file")
                    && i + 1 < remaining_args.len()
                {
                    custom_file = Some(&remaining_args[i + 1]);
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
            let is_force = remaining_args.iter().any(|a| a == "--force" || a == "-f");
            let count = if is_force {
                backup_prompts_db::force_clean_all(None).unwrap_or(0)
            } else {
                backup_prompts_db::auto_cleanup_expired(None, 86400).unwrap_or(0)
            };
            println!("[INFO] Cleaned up {} prompt backup record(s).", count);
            return;
        }
    }

    let is_json = remaining_args.iter().any(|a| a == "--json" || a == "-j");
    let mut custom_file: Option<&str> = None;
    let mut i = 0;
    while i < remaining_args.len() {
        if (remaining_args[i] == "-f"
            || remaining_args[i] == "--file"
            || remaining_args[i] == "-file")
            && i + 1 < remaining_args.len()
        {
            custom_file = Some(&remaining_args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }

    if !is_json {
        println!(
            "[*] Step 1/3: Scanning running prompts for instance '{}'...",
            target_instance
        );
        println!("[*] Step 2/3: Snapshotting active/queued prompts with multimodal images into Split SQLite DB...");
    }

    antigravity_tools_lib::error::record_ignored(
        repo_db::backup_running_prompts(&target_instance),
        "backup running prompts in repo_db prior to split SQLite snapshot",
    ); // Justification: best effort backup in repo_db; split SQLite snapshot is primary

    match backup_prompts_db::backup_active_running_prompts(Some(&target_instance), custom_file) {
        Ok((batch, records)) => {
            let mut image_count: usize = 0;
            let image_details: Vec<serde_json::Value> = records
                .iter()
                .map(|r| {
                    let uris = r
                        .images_payload
                        .as_ref()
                        .map(|p| repo_db::detection::parse_image_uris(p))
                        .unwrap_or_default();
                    let count = if uris.is_empty() && r.has_images {
                        1
                    } else {
                        uris.len()
                    };
                    image_count += count;
                    serde_json::json!({
                        "prompt_id": r.prompt_id,
                        "project_name": r.project_name,
                        "has_images": r.has_images,
                        "image_count": count,
                        "image_uris": uris,
                    })
                })
                .collect();

            if is_json {
                let payload = serde_json::json!({
                    "batch_id": batch.id,
                    "file_path": batch.file_path,
                    "record_count": records.len(),
                    "prompts_count": records.len(),
                    "image_count": image_count,
                    "image_details": image_details,
                    "target_instance": target_instance,
                    "created_at": batch.created_at,
                    "records": records,
                });
                let envelope =
                    json_envelope::JsonEnvelope::new("agm/backup-running-prompts", payload);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }

            println!(
                "[*] Step 3/3: Successfully secured {} prompt(s) with {} image(s) in batch '{}'",
                records.len(),
                image_count,
                batch.id
            );
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
    let (explicit_instance, remaining_args) = parse_instance_and_noise_args(args, true);

    if let Some(first) = remaining_args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Restore Running Prompts (Split SQLite):");
            println!("  agm restore [--keep/-k] [--json] [-file/-f <path>] [-i/--instance <id>] [instance]");
            println!("  agm restore running [instance]             FIFO restore backed-up prompts");
            println!("  Restores unrestored prompts into active execution queue and re-injects to workspaces.");
            println!(
                "\nAliases: agm restore, agm restore-running-prompts, agm rrp, agm resend-running"
            );
            println!("\nOptions:");
            println!("    --keep, -k      Preserve backup records as unrestored without starting 1-day retention timer");
            println!("    --json          Output pure JSON restored records payload");
            println!("    -f, --file      Target custom SQLite database file");
            println!("    -i, --instance  Restore only backups recorded for this instance id");
            println!("                    (default: restore all unrestored backups across every instance)");
            println!("\nExamples:");
            println!(
                "  agm restore                      # Restore & re-inject all backed-up prompts"
            );
            println!("  agm restore #1                   # Restore prompts for instance #1");
            println!("  agm restore running default      # Restore prompts for default instance");
            println!("  agm restore --keep               # Restore prompts without clearing backup state");
            println!("  agm restore --json               # Restore and print JSON payload");
            println!("  agm restore -i inst-1            # Restore only instance 'inst-1' backups");
            return;
        }
    }

    let is_json = remaining_args.iter().any(|a| a == "--json" || a == "-j");
    let keep_backup = remaining_args.iter().any(|a| a == "--keep" || a == "-k");
    let mut custom_file: Option<&str> = None;
    let mut i = 0;
    while i < remaining_args.len() {
        if (remaining_args[i] == "-f"
            || remaining_args[i] == "--file"
            || remaining_args[i] == "-file")
            && i + 1 < remaining_args.len()
        {
            custom_file = Some(&remaining_args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }

    // Restore first so freshly re-injected prompts are dispatched in this same run.
    // restore_running_prompts fans out the resend itself (scoped when explicit_instance was given,
    // across all instances otherwise).
    let restore_scope = explicit_instance.as_deref();
    let scope_label = restore_scope.unwrap_or("all instances");

    if !is_json {
        println!(
            "[*] Step 1/3: Fetching unrestored prompts from backup database (scope: {})...",
            scope_label
        );
    }

    match backup_prompts_db::restore_running_prompts(restore_scope, keep_backup, custom_file) {
        Ok(records) => {
            let restored_prompt_ids: Vec<String> =
                records.iter().map(|r| r.prompt_id.clone()).collect();
            if is_json {
                let payload = serde_json::json!({
                    "restored_count": records.len(),
                    "restored_prompt_ids": restored_prompt_ids,
                    "target_instance": restore_scope,
                    "instance_scope": restore_scope,
                    "keep_backup": keep_backup,
                    "is_restored": true,
                    "records": records,
                });
                let envelope =
                    json_envelope::JsonEnvelope::new("agm/restore-running-prompts", payload);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }

            println!(
                "[*] Step 2/3: Restoring {} prompt(s) to SQLite FIFO queue in strict FIFO order...",
                records.len()
            );
            println!("[*] Step 3/3: Re-seeding workspace resume tasks and dispatching in-flight triggers...");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instance_and_noise_args() {
        // Noise tokens stripping
        let args = vec![
            "running".to_string(),
            "#1".to_string(),
            "--json".to_string(),
        ];
        let (inst, rem) = parse_instance_and_noise_args(&args, false);
        assert!(inst.is_some());
        assert_eq!(rem, vec!["--json"]);

        // Restore unscoped when no instance provided
        let restore_args = vec!["running-prompts".to_string(), "--keep".to_string()];
        let (inst_res, rem_res) = parse_instance_and_noise_args(&restore_args, true);
        assert!(inst_res.is_none());
        assert_eq!(rem_res, vec!["--keep"]);

        // Explicit -i flag
        let explicit_args = vec![
            "-i".to_string(),
            "default".to_string(),
            "-f".to_string(),
            "test.db".to_string(),
        ];
        let (inst_exp, rem_exp) = parse_instance_and_noise_args(&explicit_args, true);
        assert_eq!(inst_exp, Some("default".to_string()));
        assert_eq!(rem_exp, vec!["-f", "test.db"]);

        // Positional instance with noise token 'rp'
        let rp_args = vec!["rp".to_string(), "worker-1".to_string()];
        let (inst_rp, rem_rp) = parse_instance_and_noise_args(&rp_args, false);
        assert_eq!(inst_rp, Some("worker-1".to_string()));
        assert!(rem_rp.is_empty());
    }
}
