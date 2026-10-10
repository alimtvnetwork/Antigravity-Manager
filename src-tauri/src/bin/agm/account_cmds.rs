//! account_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn cmd_accounts_list(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output {
        let accounts = account::list_accounts().unwrap_or_default();
        CliEnvelope::ok("accounts list", None, accounts).print_and_exit();
    }
    crate::account_mgmt::cmd_accounts(args);
}

pub(crate) fn cmd_accounts_add(_args: &[String]) {
    println!("[*] Open browser or AGM UI to connect account OAuth.");
}

pub(crate) fn cmd_accounts_remove(args: &[String]) {
    let ctx = CliContext::parse(args);
    if let Some(target) = ctx.positional_args.first() {
        let _ = account::delete_account(target);
        if ctx.json_output {
            CliEnvelope::ok(
                "accounts remove",
                None,
                serde_json::json!({ "removed": target }),
            )
            .print_and_exit();
        }
        println!("[SUCCESS] Removed account '{}'.", target);
    }
}

pub(crate) fn cmd_accounts_validate(args: &[String]) {
    crate::doctor_cmds::cmd_doctor(args);
}

pub(crate) fn cmd_accounts_balance(args: &[String]) {
    crate::status_cmds::cmd_status(args);
}

pub(crate) fn cmd_accounts_refresh_tier(args: &[String]) {
    let refresh_all = args.iter().any(|a| a == "--all");
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Accounts Refresh Tier CLI:");
        println!("  agm accounts refresh-tier [--all] [--json]");
        println!("\nDescription:");
        println!("  Fetches and updates subscription tiers (FREE / PRO / ULTRA) from Google");
        println!("  for registered accounts in the credential vault.");
        println!("  By default, only refreshes accounts with missing or unknown tiers.");
        println!("\nOptions:");
        println!("  --all       Force-refresh all accounts, including those with known tiers");
        println!("  --json, -j  Output results wrapped in standard JSON envelope");
        println!("\nExamples:");
        println!("  agm accounts refresh-tier");
        println!("  agm accounts refresh-tier --all");
        println!("  agm accounts refresh-tier --json");
        return;
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Failed to start async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let result = rt.block_on(account::refresh_missing_tiers_with_options(
        refresh_all,
        None,
    ));

    match result {
        Ok(stats) => {
            if is_json {
                let envelope =
                    json_envelope::JsonEnvelope::new("agm/accounts-refresh-tier", &stats);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).unwrap_or_default()
                );
            } else {
                println!("\nAccount subscription tier refresh completed:");
                println!("  Targeted accounts: {}", stats.total);
                println!("  Updated:           {}", stats.updated);
                println!("  Still unknown:     {}", stats.still_unknown);
                println!("  Failed:            {}", stats.failed);
                if !stats.details.is_empty() {
                    println!("\nDetails:");
                    for detail in &stats.details {
                        println!("  - {}", detail);
                    }
                }
            }
            if stats.failed > 0 && stats.updated == 0 && stats.total > 0 {
                std::process::exit(1);
            }
        }
        Err(e) => {
            if is_json {
                let err_obj = serde_json::json!({
                    "error": e,
                    "success": false,
                });
                let envelope =
                    json_envelope::JsonEnvelope::new("agm/accounts-refresh-tier", err_obj);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).unwrap_or_default()
                );
            } else {
                eprintln!("[ERROR] Tier refresh failed: {}", e);
            }
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_accounts_export(args: &[String]) {
    match account::export_accounts_envelope() {
        Ok(json_str) => {
            let file_arg = args
                .iter()
                .position(|a| a == "--file" || a == "-o")
                .and_then(|idx| args.get(idx + 1));
            if let Some(target_file) = file_arg {
                if let Err(e) = fs::write(target_file, &json_str) {
                    eprintln!("[ERROR] Failed to write accounts to {}: {}", target_file, e);
                } else {
                    println!(
                        "✅ Successfully exported accounts envelope to {}",
                        target_file
                    );
                }
            } else {
                println!("{}", json_str);
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to export accounts envelope: {}", e),
    }
}

pub(crate) fn cmd_accounts_import(args: &[String]) {
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    let target_file = if non_flag_args.len() > 1 {
        Some(non_flag_args[1].as_str())
    } else {
        None
    };

    let path_str = match target_file {
        Some(p) => p,
        None => {
            eprintln!("Usage: agm accounts import <file_path>");
            return;
        }
    };

    let resolved_path = json_envelope::resolve_relative_json_path(path_str);
    let raw_json = match fs::read_to_string(&resolved_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[ERROR] Failed to read file '{}': {}",
                resolved_path.display(),
                e
            );
            return;
        }
    };

    match json_envelope::extract_payload::<account::AccountIndex>(&raw_json) {
        Ok((imported_index, attrs)) => {
            let count = imported_index.accounts.len();
            match account::save_account_index(&imported_index) {
                Ok(_) => {
                    println!(
                        "✅ Successfully imported {} accounts from '{}' (Envelope v{}).",
                        count,
                        resolved_path.display(),
                        attrs.version
                    );
                }
                Err(e) => eprintln!("[ERROR] Failed to save accounts index: {}", e),
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to parse accounts envelope: {}", e),
    }
}
