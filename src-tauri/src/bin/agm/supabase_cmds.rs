//! supabase_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;

pub(crate) fn cmd_supabase(args: &[String]) {
    let sub = args
        .first()
        .map(|s| s.trim_start_matches('/').to_lowercase())
        .unwrap_or_else(|| "help".to_string());
    let sub_args = if args.len() > 1 { &args[1..] } else { &[] };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Tokio runtime error: {}", e);
            return;
        }
    };

    match sub.as_str() {
        "help" | "--help" | "-h" => crate::help_tables::print_supabase_help(),
        "status" | "info" | "ls" => cmd_supabase_status(&rt),
        "list-leases" | "leases" | "in-use" => cmd_supabase_list_leases(&rt),
        "test" => cmd_supabase_test(&rt, sub_args),
        "set-endpoint" | "set" | "add" => {
            crate::supabase_config_cmds::cmd_supabase_set_endpoint(sub_args)
        }
        "set-config" | "config" => crate::supabase_config_cmds::cmd_supabase_set_config(sub_args),
        "load-json" | "import" => crate::supabase_config_cmds::cmd_supabase_load_json(sub_args),
        "export" => match supabase_sync::export_config_json(
            &supabase_sync::load_config().unwrap_or_default(),
        ) {
            Ok(json_str) => {
                let file_arg = sub_args
                    .iter()
                    .position(|a| a == "--file" || a == "-o")
                    .and_then(|idx| sub_args.get(idx + 1));
                if let Some(target_file) = file_arg {
                    if let Err(e) = fs::write(target_file, &json_str) {
                        eprintln!("[ERROR] Failed to write to {}: {}", target_file, e);
                    } else {
                        println!(
                            "✅ Successfully exported Supabase configuration to {}",
                            target_file
                        );
                    }
                } else {
                    println!("{}", json_str);
                }
            }
            Err(e) => eprintln!("[ERROR] Failed to export Supabase configuration: {}", e),
        },
        "schema" => crate::supabase_config_cmds::cmd_supabase_schema(sub_args),
        "sync" => crate::supabase_config_cmds::cmd_supabase_sync(&rt),
        "confirm" => crate::supabase_config_cmds::cmd_supabase_confirm(&rt, sub_args),
        "enable" => {
            let mut cfg = supabase_sync::load_config().unwrap_or_default();
            cfg.is_sync_enabled = true;
            let _ = supabase_sync::save_config(&cfg);
            println!("✅ Supabase synchronization ENABLED.");
        }
        "disable" => {
            let mut cfg = supabase_sync::load_config().unwrap_or_default();
            cfg.is_sync_enabled = false;
            let _ = supabase_sync::save_config(&cfg);
            println!("⏸️ Supabase synchronization DISABLED.");
        }
        "set-alias" => {
            if let Some(alias) = sub_args.first() {
                let mut cfg = supabase_sync::load_config().unwrap_or_default();
                cfg.node_alias = alias.to_string();
                let _ = supabase_sync::save_config(&cfg);
                println!("✅ Node alias updated to: {}", alias);
            } else {
                eprintln!("[ERROR] Usage: agm supabase set-alias <new_alias>");
            }
        }
        "load-secrets" | "auto-load" | "discover" => {
            match supabase_sync::auto_discover_supabase_credentials() {
                Ok(cfg) => {
                    if !cfg.endpoints.is_empty() {
                        println!(
                            "✅ Successfully discovered and loaded {} Supabase endpoint(s) from repo-secrets.",
                            cfg.endpoints.len()
                        );
                        for ep in &cfg.endpoints {
                            println!("   • [{}] {} -> {}", ep.id, ep.name, ep.url);
                        }
                    } else {
                        eprintln!(
                            "[WARN] No Supabase credentials found in candidate repo-secrets paths."
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[ERROR] Failed to auto-discover Supabase credentials: {}",
                        e
                    );
                }
            }
        }
        "set-prune" => {
            if sub_args.len() >= 2 {
                if let (Ok(root_mb), Ok(sec_mb)) =
                    (sub_args[0].parse::<u64>(), sub_args[1].parse::<u64>())
                {
                    let mut cfg = supabase_sync::load_config().unwrap_or_default();
                    cfg.auto_prune_root_mb = root_mb;
                    cfg.auto_prune_secondary_mb = sec_mb;
                    let _ = supabase_sync::save_config(&cfg);
                    println!(
                        "✅ Supabase prune thresholds updated: Root={}MB, Secondary={}MB",
                        root_mb, sec_mb
                    );
                } else {
                    eprintln!("[ERROR] Invalid numeric values. Usage: agm supabase set-prune <root_mb> <secondary_mb>");
                }
            } else {
                let cfg = supabase_sync::load_config().unwrap_or_default();
                println!(
                    "Current Supabase prune thresholds: Root={}MB, Secondary={}MB",
                    cfg.auto_prune_root_mb, cfg.auto_prune_secondary_mb
                );
            }
        }
        "set-heartbeat" => {
            if let Some(sec_str) = sub_args.first() {
                if let Ok(secs) = sec_str.parse::<u64>() {
                    let mut cfg = supabase_sync::load_config().unwrap_or_default();
                    cfg.heartbeat_interval_secs = secs;
                    let _ = supabase_sync::save_config(&cfg);
                    println!("✅ Supabase heartbeat interval updated to {}s", secs);
                } else {
                    eprintln!("[ERROR] Invalid numeric value. Usage: agm supabase set-heartbeat <seconds>");
                }
            } else {
                let cfg = supabase_sync::load_config().unwrap_or_default();
                println!(
                    "Current Supabase heartbeat interval: {}s",
                    cfg.heartbeat_interval_secs
                );
            }
        }
        _ => {
            eprintln!(
                "[ERROR] Unknown command: 'agm supabase {}'. Run 'agm supabase help' for guide.",
                sub
            );
        }
    }
}

pub(crate) fn cmd_supabase_status(rt: &tokio::runtime::Runtime) {
    let mut cfg = match supabase_sync::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load Supabase config: {}", e);
            return;
        }
    };

    // Auto-discover endpoints from repo-secrets if available
    let seeded = supabase_sync::auto_seed_from_repo_secrets(&mut cfg);
    if seeded {
        let _ = supabase_sync::save_config(&cfg);
    }

    let node_id = supabase_sync::get_local_node_id();
    let local_ip = supabase_sync::get_local_ip();
    let uptime = supabase_sync::get_uptime_seconds();
    let cfg_path = supabase_sync::get_config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let repo_secrets_files: Vec<_> = supabase_sync::candidate_repo_secrets_paths()
        .into_iter()
        .filter(|p| p.exists())
        .collect();

    let auto_discovered_eps: Vec<_> = cfg
        .endpoints
        .iter()
        .filter(|ep| {
            ep.notes
                .as_deref()
                .map_or(false, |n| n.to_lowercase().contains("repo-secrets"))
                || ep
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains("repo-secrets"))
        })
        .collect();

    println!("================================================================================");
    println!("  AGM Supabase Node & Fleet Status");
    println!("================================================================================");
    println!("  Node ID:         {}", node_id);
    println!("  Node Alias:      {}", cfg.node_alias);
    println!("  Local IPv4:      {}", local_ip);
    println!("  Uptime:          {}s ({}m)", uptime, uptime / 60);
    println!(
        "  Sync Active:     {}",
        if cfg.is_sync_enabled {
            "YES (Enabled)"
        } else {
            "NO (Disabled)"
        }
    );
    println!("  Heartbeat:       every {}s", cfg.heartbeat_interval_secs);
    println!("  Config File:     {}", cfg_path);
    println!(
        "  Repo Secrets:    {} source file(s) found ({} auto-discovered endpoint(s))",
        repo_secrets_files.len(),
        auto_discovered_eps.len()
    );
    if !repo_secrets_files.is_empty() {
        for rf in &repo_secrets_files {
            println!("                   • {}", rf.display());
        }
    }
    println!();

    println!("  --- Configured Endpoints ({}) ---", cfg.endpoints.len());
    if cfg.endpoints.is_empty() {
        println!("  (No endpoints configured. Use 'agm supabase set ...' or 'agm supabase load-json <file>')");
    } else {
        println!(
            "  {:<20} {:<10} {:<8} {:<8} {:<14} {:<24} {:<30}",
            "ID", "ROLE", "ENABLED", "PRIORITY", "SOURCE", "TAGS/NOTES", "URL"
        );
        println!("  {}", "-".repeat(120));
        for ep in &cfg.endpoints {
            let is_repo_secret = ep
                .notes
                .as_deref()
                .map_or(false, |n| n.to_lowercase().contains("repo-secrets"))
                || ep
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains("repo-secrets"));
            let source_str = if is_repo_secret {
                "repo-secrets"
            } else {
                "manual"
            };
            let notes_str =
                ep.notes
                    .as_deref()
                    .unwrap_or(if ep.tags.is_empty() { "-" } else { "" });
            let tags_str = if !ep.tags.is_empty() {
                format!("[{}] {}", ep.tags.join(", "), notes_str)
            } else {
                notes_str.to_string()
            };
            println!(
                "  {:<20} {:<10} {:<8} {:<8} {:<14} {:<24} {:<30}",
                ep.id,
                ep.role,
                if ep.is_enabled { "yes" } else { "no" },
                ep.priority,
                source_str,
                if tags_str.len() > 22 {
                    format!("{}...", &tags_str[..20])
                } else {
                    tags_str
                },
                if ep.url.len() > 28 {
                    format!("{}...", &ep.url[..26])
                } else {
                    ep.url.clone()
                }
            );
        }
    }
    println!();

    println!("  --- Active Remote Workspace Leases (In-Use Accounts Across Machines) ---");
    match rt.block_on(workspace_lease_manager::list_active_leases()) {
        Ok(leases) => {
            if leases.is_empty() {
                println!("  (No active remote leases held across fleet)");
            } else {
                let now = Utc::now().timestamp();
                println!(
                    "  {:<30} {:<16} {:<16} {:<16} {:<10}",
                    "ACCOUNT EMAIL / ID", "NODE ALIAS", "IP ADDRESS", "PROFILE", "EXPIRES IN"
                );
                println!("  {}", "-".repeat(95));
                for l in &leases {
                    let display_acc = if !l.account_email.is_empty() {
                        l.account_email.clone()
                    } else {
                        l.account_id.clone()
                    };
                    let exp = if l.expires_at > now {
                        format!("{}s", l.expires_at - now)
                    } else {
                        "expired".to_string()
                    };
                    let display_ip = if !l.ip_address.is_empty() {
                        l.ip_address.clone()
                    } else {
                        "-".to_string()
                    };
                    println!(
                        "  {:<30} {:<16} {:<16} {:<16} {:<10}",
                        if display_acc.len() > 28 {
                            format!("{}...", &display_acc[..26])
                        } else {
                            display_acc
                        },
                        l.node_alias,
                        display_ip,
                        l.profile_name,
                        exp
                    );
                }
                println!("\n  Total active cluster leases: {}", leases.len());
            }
        }
        Err(e) => {
            println!("  (Could not fetch remote leases: {})", e);
        }
    }
    println!("================================================================================");
}

pub(crate) fn cmd_supabase_list_leases(rt: &tokio::runtime::Runtime) {
    println!("Fetching active workspace account leases from Supabase Root DB...");
    match rt.block_on(workspace_lease_manager::list_active_leases()) {
        Ok(leases) => {
            if leases.is_empty() {
                println!("No active accounts currently leased across the cluster.");
                return;
            }
            let now = Utc::now().timestamp();
            println!(
                "\n{:<32} {:<24} {:<18} {:<16} {:<16} {:<10}",
                "ACCOUNT EMAIL", "ACCOUNT ID", "NODE ALIAS", "IP ADDRESS", "INSTANCE", "EXPIRES IN"
            );
            println!("{}", "-".repeat(120));
            for l in &leases {
                let exp = if l.expires_at > now {
                    format!("{}s", l.expires_at - now)
                } else {
                    "expired".to_string()
                };
                let display_ip = if !l.ip_address.is_empty() {
                    l.ip_address.clone()
                } else {
                    "-".to_string()
                };
                println!(
                    "{:<32} {:<24} {:<18} {:<16} {:<16} {:<10}",
                    if l.account_email.len() > 30 {
                        format!("{}...", &l.account_email[..28])
                    } else {
                        l.account_email.clone()
                    },
                    if l.account_id.len() > 22 {
                        format!("{}...", &l.account_id[..20])
                    } else {
                        l.account_id.clone()
                    },
                    l.node_alias,
                    display_ip,
                    l.profile_name,
                    exp
                );
            }
            println!(
                "\nTotal active accounts in use across machines: {}",
                leases.len()
            );
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to query workspace leases: {}", e);
        }
    }
}

pub(crate) fn cmd_supabase_test(rt: &tokio::runtime::Runtime, args: &[String]) {
    let cfg = match supabase_sync::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load Supabase config: {}", e);
            return;
        }
    };

    let target_id = args.first().map(|s| s.as_str());
    let endpoints: Vec<_> = cfg
        .endpoints
        .iter()
        .filter(|ep| {
            if let Some(id) = target_id {
                ep.id == id || ep.name.to_lowercase().contains(&id.to_lowercase())
            } else {
                ep.is_enabled
            }
        })
        .collect();

    if endpoints.is_empty() {
        println!("No matching endpoints found to test.");
        return;
    }

    println!("Testing {} Supabase endpoint(s)...", endpoints.len());
    for ep in endpoints {
        print!("  Connecting to [{}] {} ({}) ... ", ep.id, ep.name, ep.role);
        let client = match supabase_client::SupabaseClient::new(ep) {
            Ok(c) => c,
            Err(e) => {
                println!("FAILED (Client init error: {})", e);
                continue;
            }
        };
        match rt.block_on(client.test_connection()) {
            Ok(res) => {
                if res.is_success {
                    println!("PASS (HTTP {})", res.status_code.unwrap_or(200));
                    println!("    -> {}", res.message);
                } else {
                    println!("FAIL (HTTP {})", res.status_code.unwrap_or(0));
                    println!("    -> {}", res.message);
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    }
}
