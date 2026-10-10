//! account_mgmt — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn cmd_accounts(args: &[String]) {
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("switch") || s.eq_ignore_ascii_case("use"))
        .unwrap_or(false)
    {
        let forward_args: Vec<String> = args
            .iter()
            .filter(|a| !a.eq_ignore_ascii_case("switch") && !a.eq_ignore_ascii_case("use"))
            .cloned()
            .collect();
        cmd_switch(&forward_args);
        return;
    }

    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("export"))
        .unwrap_or(false)
    {
        crate::account_cmds::cmd_accounts_export(args);
        return;
    }

    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("import") || s.eq_ignore_ascii_case("load-json"))
        .unwrap_or(false)
    {
        crate::account_cmds::cmd_accounts_import(args);
        return;
    }

    if non_flag_args
        .first()
        .map(|s| {
            s.eq_ignore_ascii_case("refresh-tier")
                || s.eq_ignore_ascii_case("refresh_tier")
                || s.eq_ignore_ascii_case("refreshtier")
        })
        .unwrap_or(false)
    {
        crate::account_cmds::cmd_accounts_refresh_tier(args);
        return;
    }

    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Accounts & Quota CLI:");
        println!("  agm accounts [ls] [--active] [--json]");
        println!("  agm accounts refresh-tier [--all] [--json]");
        println!("  agm accounts export [--file <path>]");
        println!("  agm accounts import <path>");
        println!("  agm account switch <email|prefix|id|#seq> [--instance <id|alias>]");
        println!("  agm account switch <instance> <email>");
        println!("\nDescription:");
        println!("  Lists all authenticated Google Gemini profiles in the credential vault,");
        println!("  their bound email, tier, 4-hour window quota, weekly quota, and active state.");
        println!(
            "  Allows direct switching of accounts across default or multi-instance profiles."
        );
        println!("\nAliases: agm accounts, agm account, agm acc");
        println!("\nSubcommands & Actions:");
        println!("  ls, list            Display table of all configured accounts (default)");
        println!("  refresh-tier        Fetch and persist subscription tiers (PRO / ULTRA / FREE)");
        println!("  export [--file]     Export accounts wrapped in standard JSON envelope");
        println!("  import <path>       Import accounts from standard JSON envelope file");
        println!("  switch, use <query> Switch active account (or instance account) directly");
        println!("\nOptions:");
        println!("    --all             (refresh-tier) Force-refresh all accounts, including known tiers");
        println!("    --active          Show only the currently active account profile");
        println!("    --json, -j        Output account list in structured JSON format");
        println!("\nExamples:");
        println!("  agm accounts                                # Display table of all configured accounts");
        println!(
            "  agm accounts refresh-tier                   # Refresh missing subscription tiers"
        );
        println!(
            "  agm accounts refresh-tier --all             # Force-refresh all subscription tiers"
        );
        println!("  agm accounts refresh-tier --json            # Output tier refresh statistics as JSON");
        println!(
            "  agm accounts export --file accounts.json    # Export accounts envelope to file"
        );
        println!(
            "  agm accounts import accounts.json           # Import accounts envelope from file"
        );
        println!("  agm accounts --active                       # Show currently selected active account");
        println!("  agm accounts --json                         # Export accounts and quota matrix as JSON");
        println!("  agm account switch dev.user@gmail.com       # Switch active profile to dev.user@gmail.com");
        println!("  agm account switch #2 dev.user@gmail.com    # Switch instance #2 to dev.user@gmail.com");
        return;
    }

    let show_only_active = args.iter().any(|a| a == "--active");
    let is_json = args.iter().any(|a| a == "--json");

    let index = match account::load_account_index() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("[ERROR] Failed to load accounts index: {}", e);
            std::process::exit(1);
        }
    };

    let active_id = index.current_account_id.as_deref().unwrap_or("");

    let mut account_rows = Vec::new();
    for (i, summary) in index.accounts.iter().enumerate() {
        let is_current = summary.id == active_id;
        if show_only_active && !is_current {
            continue;
        }

        let full_acc = account::load_account(&summary.id).ok();
        let tier = full_acc
            .as_ref()
            .and_then(|a| a.quota.as_ref())
            .and_then(|q| q.subscription_tier.clone())
            .unwrap_or_else(|| "FREE".to_string());

        let status = if is_current { "ACTIVE" } else { "STANDBY" };

        let quota_str = if let Some(ref acc) = full_acc {
            if let Some(ref q) = acc.quota {
                if let Some(first_m) = q.models.first() {
                    format!("{}%", first_m.percentage)
                } else if let Some(ref groups) = q.quota_groups {
                    if let Some(first_b) = groups.first().and_then(|g| g.buckets.first()) {
                        format!("{:.0}%", first_b.remaining_fraction * 100.0)
                    } else {
                        "-".to_string()
                    }
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };

        let updated_str = if summary.last_used > 0 {
            chrono::DateTime::from_timestamp(summary.last_used, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };

        account_rows.push((
            i + 1,
            summary.email.clone(),
            tier,
            status,
            quota_str,
            updated_str,
            summary.id.clone(),
        ));
    }

    if is_json {
        let json_items: Vec<_> = account_rows
            .iter()
            .map(|(idx, email, tier, status, quota, updated, id)| {
                serde_json::json!({
                    "index": idx,
                    "id": id,
                    "email": email,
                    "tier": tier,
                    "status": status,
                    "quota": quota,
                    "last_used": updated,
                })
            })
            .collect();
        let envelope = json_envelope::JsonEnvelope::new("agm/accounts-export", json_items);
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_default()
        );
        return;
    }

    println!("\nRegistered Accounts ({} total):", account_rows.len());
    println!(
        "{:<5} {:<32} {:<10} {:<10} {:<14} LAST USED",
        "INDEX", "EMAIL", "TIER", "STATUS", "QUOTA"
    );
    println!("{}", "-".repeat(85));

    for (idx, email, tier, status, quota, updated, _) in account_rows {
        println!(
            "{:<5} {:<32} {:<10} {:<10} {:<14} {}",
            idx, email, tier, status, quota, updated
        );
    }
    println!();
}

pub(crate) fn cmd_switch(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Account & Profile Switch CLI:");
        println!("  agm switch <account> [--instance <id|alias>] [--json]");
        println!("  agm switch <instance> <account> [--json]");
        println!("  agm switch account <account> [--instance <id>]");
        println!("  agm switch <email|prefix|id|#seq>");
        println!("\nDescription:");
        println!("  Directly switches authenticated Google Gemini account credentials for an");
        println!(
            "  Antigravity IDE profile (or the active/default profile) without GUI intervention."
        );
        println!("  Automatically injects tokens, updates profile configurations, and preserves running prompts.");
        println!("\nAliases: agm switch, agm switch-account, agm switch account, agm account-switch, agm swtich, agm swtich-account");
        println!("\nArguments & Options:");
        println!("  <account>                   Account email, email prefix, account ID, or account number (#1, #2)");
        println!("  <instance>                  Instance name, ID, sequence number (#1, #2), or 'default' / 'active'");
        println!("  --instance, -i <id|alias>   Target instance profile to switch (defaults to active instance)");
        println!("  --json, -j                  Output switch outcome in structured JSON format");
        println!("\nExamples:");
        println!("  agm switch dev.user@gmail.com                        # Switch active profile to dev.user@gmail.com");
        println!("  agm switch account dev.user@gmail.com                # Switch account using 'switch account' syntax");
        println!("  agm switch dev.user                                  # Switch by email prefix");
        println!("  agm switch #2 dev.user@gmail.com                     # Switch instance #2 to dev.user@gmail.com");
        println!("  agm switch dev.user@gmail.com --instance #2          # Same: specify target instance with flag");
        println!("  agm switch dev.user@gmail.com -i Worker-1            # Target instance by name with -i flag");
        println!("  agm switch Worker-1 dev.user@gmail.com               # Switch instance named 'Worker-1'");
        println!("  agm switch acc_01j7x8a                               # Switch using exact internal account ID");
        println!("  agm switch #2                                        # Switch active profile to account #2 in list");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let raw_non_flag: Vec<&String> = args
        .iter()
        .filter(|a| {
            !a.starts_with('-')
                && !a.eq_ignore_ascii_case("switch")
                && !a.eq_ignore_ascii_case("swtich")
                && !a.eq_ignore_ascii_case("use")
        })
        .collect();

    // Strip semantic filler keywords like "account", "to", "for" when followed by actual values
    let mut non_flag_args: Vec<&String> = Vec::new();
    for (idx, arg) in raw_non_flag.iter().enumerate() {
        let lower = arg.to_lowercase();
        if (lower == "account" || lower == "acc" || lower == "to" || lower == "for")
            && (idx == 0 || idx + 1 < raw_non_flag.len())
            && raw_non_flag.len() > 1
        {
            continue;
        }
        non_flag_args.push(arg);
    }

    let index = match account::load_account_index() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("[ERROR] Failed to load accounts index: {}", e);
            std::process::exit(1);
        }
    };

    if non_flag_args.is_empty() {
        let curr = account::get_current_account().ok().flatten();
        let active_inst =
            instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
        println!("AGM Profile Switch:");
        println!("  Current active instance: {}", active_inst);
        println!(
            "  Current active account:  {}",
            curr.as_ref().map(|a| a.email.as_str()).unwrap_or("(none)")
        );
        println!("\nUsage: agm switch <email|id|#seq> [--instance <id>]");
        println!("       agm switch <instance> <account>");
        println!("       agm switch account <email>");
        println!("Run 'agm switch --help' for full guide or 'agm accounts' to list accounts.");
        std::process::exit(1);
    }

    let explicit_inst_opt = args
        .iter()
        .position(|a| a == "--instance" || a == "-i")
        .and_then(|pos| args.get(pos + 1).map(|s| s.as_str()));

    // Determine target instance specifier and account query
    let (target_inst_spec, acc_query) = if non_flag_args.len() >= 2 {
        // Check if first arg resolves to an instance
        let first_is_instance = instance::resolve_instance_id(non_flag_args[0]).is_ok();
        let second_is_instance = instance::resolve_instance_id(non_flag_args[1]).is_ok();

        if first_is_instance && !second_is_instance {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        } else if second_is_instance && !first_is_instance {
            (Some(non_flag_args[1].as_str()), non_flag_args[0].as_str())
        } else if explicit_inst_opt.is_some() {
            (explicit_inst_opt, non_flag_args[0].as_str())
        } else {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        }
    } else {
        (explicit_inst_opt, non_flag_args[0].as_str())
    };

    let query_lower = acc_query.trim().to_lowercase();

    // Check if query is an account index like #1, #2, 1, 2
    let index_match =
        if query_lower.starts_with('#') || query_lower.chars().all(|c| c.is_ascii_digit()) {
            let num_str = query_lower.trim_start_matches('#');
            if let Ok(num) = num_str.parse::<usize>() {
                if num >= 1 && num <= index.accounts.len() {
                    Some(&index.accounts[num - 1])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

    let target_account = if let Some(acc) = index_match {
        acc
    } else {
        let matches: Vec<_> = index
            .accounts
            .iter()
            .filter(|a| {
                let email_l = a.email.to_lowercase();
                let id_l = a.id.to_lowercase();
                email_l == query_lower
                    || id_l == query_lower
                    || email_l.starts_with(&query_lower)
                    || email_l.contains(&query_lower)
                    || id_l.contains(&query_lower)
            })
            .collect();

        if matches.is_empty() {
            eprintln!("[ERROR] No account found matching '{}'.", acc_query);
            eprintln!("Run 'agm accounts' to view registered accounts.");
            std::process::exit(1);
        }

        if matches.len() == 1 {
            matches[0]
        } else if let Some(exact) = matches
            .iter()
            .find(|a| a.email.to_lowercase() == query_lower)
        {
            *exact
        } else {
            eprintln!("[ERROR] Query '{}' matched multiple accounts:", acc_query);
            for m in matches {
                eprintln!("  - {} (ID: {})", m.email, m.id);
            }
            eprintln!("Please specify a more precise email or account ID.");
            std::process::exit(1);
        }
    };

    // Resolve target instance details
    let resolved_inst_id = match target_inst_spec {
        Some(spec) => match instance::resolve_instance_id(spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[ERROR] Could not resolve instance '{}': {}", spec, e);
                std::process::exit(1);
            }
        },
        None => instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string()),
    };

    let all_insts = instance::list_instances().unwrap_or_default();
    let target_inst_info = all_insts.iter().find(|i| i.config.id == resolved_inst_id);
    let target_inst_name = target_inst_info
        .map(|i| i.config.name.clone())
        .unwrap_or_else(|| resolved_inst_id.clone());
    let prev_bound = if resolved_inst_id == "default" || resolved_inst_id == "__default__" {
        account::get_current_account()
            .ok()
            .flatten()
            .map(|a| a.email)
            .unwrap_or_else(|| "(none)".to_string())
    } else {
        target_inst_info
            .and_then(|i| i.config.bound_email.clone())
            .unwrap_or_else(|| "(none)".to_string())
    };

    if !is_json {
        println!(
            "[*] Switching instance '{}' ({}) to account '{}' (ID: {})...",
            target_inst_name, resolved_inst_id, target_account.email, target_account.id
        );
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = rt.block_on(instance::switch_account_to_instance(
        &target_account.id,
        Some(&resolved_inst_id),
    )) {
        eprintln!("[ERROR] Failed to switch account: {}", e);
        std::process::exit(1);
    }

    if is_json {
        let result = serde_json::json!({
            "success": true,
            "instance_id": resolved_inst_id,
            "instance_name": target_inst_name,
            "account_id": target_account.id,
            "email": target_account.email,
            "previous_email": prev_bound,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else {
        println!(
            "[SUCCESS] Switched account for instance '{}' ({}):",
            target_inst_name, resolved_inst_id
        );
        println!("          Previous: {}", prev_bound);
        println!(
            "          Active:   {} (ID: {})",
            target_account.email, target_account.id
        );
    }
}
