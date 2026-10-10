use crate::modules::{account, auto_switcher, config, instance, repo_db};
use chrono::Utc;

use super::*;

// -----------------------------------------------------------------------------
// Supabase Subcommand Handler
// -----------------------------------------------------------------------------

pub(crate) fn handle_supabase_subcommand(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_supabase_cli_help();
        std::process::exit(0);
    }

    let sub = args
        .first()
        .map(|s| {
            s.trim_start_matches('/')
                .trim_start_matches('-')
                .to_lowercase()
        })
        .unwrap_or_else(|| "status".to_string());
    let sub_args = if args.len() > 1 { &args[1..] } else { &[] };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Tokio runtime error: {}", e);
            std::process::exit(1);
        }
    };

    match sub.as_str() {
        "status" | "info" | "ls" => {
            let mut cfg = match crate::modules::supabase_sync::load_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[ERROR] Failed to load Supabase config: {}", e);
                    std::process::exit(1);
                }
            };

            // Auto-discover endpoints from repo-secrets if available
            let seeded = crate::modules::supabase_sync::auto_seed_from_repo_secrets(&mut cfg);
            if seeded {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    crate::modules::supabase_sync::save_config(&cfg),
                    "save_config",
                );
            }

            let node_id = crate::modules::supabase_sync::get_local_node_id();
            let local_ip = crate::modules::supabase_sync::get_local_ip();
            let uptime = crate::modules::supabase_sync::get_uptime_seconds();
            let repo_secrets_files: Vec<_> =
                crate::modules::supabase_sync::candidate_repo_secrets_paths()
                    .into_iter()
                    .filter(|p| p.exists())
                    .collect();

            println!(
                "================================================================================"
            );
            println!("  Antigravity-Manager Supabase Fleet & Synchronization Status");
            println!(
                "================================================================================"
            );
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
            println!("  Endpoints Count: {}", cfg.endpoints.len());
            println!(
                "  Repo Secrets:    {} file(s) detected",
                repo_secrets_files.len()
            );
            for rf in &repo_secrets_files {
                println!("                   • {}", rf.display());
            }
            println!();

            println!("  --- Configured Endpoints ({}) ---", cfg.endpoints.len());
            if cfg.endpoints.is_empty() {
                println!("  (No endpoints configured. Use 'agm supabase set ...' or 'agm supabase load-json <file>')");
            } else {
                println!(
                    "  {:<20} {:<10} {:<8} {:<8} {:<12} {:<30}",
                    "ID", "ROLE", "ENABLED", "PRIORITY", "SOURCE", "URL"
                );
                println!("  {}", "-".repeat(95));
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
                    println!(
                        "  {:<20} {:<10} {:<8} {:<8} {:<12} {:<30}",
                        ep.id,
                        ep.role,
                        if ep.is_enabled { "yes" } else { "no" },
                        ep.priority,
                        source_str,
                        if ep.url.len() > 28 {
                            format!("{}...", &ep.url[..26])
                        } else {
                            ep.url.clone()
                        }
                    );
                }
            }
            println!();

            println!("  --- Active Remote Workspace Leases ---");
            match rt.block_on(crate::modules::workspace_lease_manager::list_active_leases()) {
                Ok(leases) => {
                    if leases.is_empty() {
                        println!("  (No active remote leases held across fleet)");
                    } else {
                        let now = chrono::Utc::now().timestamp();
                        println!(
                            "  {:<30} {:<16} {:<16} {:<16} {:<10}",
                            "ACCOUNT EMAIL / ID",
                            "NODE ALIAS",
                            "IP ADDRESS",
                            "PROFILE",
                            "EXPIRES IN"
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
            println!(
                "================================================================================"
            );
            std::process::exit(0);
        }

        "sync" => {
            println!("[CLI] Triggering Supabase manual synchronization...");
            match rt.block_on(crate::modules::supabase_sync::trigger_manual_sync()) {
                Ok(_) => {
                    println!("[SUCCESS] Supabase synchronization completed successfully.");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Supabase synchronization failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "list-leases" | "leases" | "in-use" => {
            println!("[CLI] Fetching active workspace account leases from Supabase Root DB...");
            match rt.block_on(crate::modules::workspace_lease_manager::list_active_leases()) {
                Ok(leases) => {
                    if leases.is_empty() {
                        println!("No active accounts currently leased across the cluster.");
                    } else {
                        let now = chrono::Utc::now().timestamp();
                        println!(
                            "\n{:<32} {:<24} {:<18} {:<16} {:<16} {:<10}",
                            "ACCOUNT EMAIL",
                            "ACCOUNT ID",
                            "NODE ALIAS",
                            "IP ADDRESS",
                            "INSTANCE",
                            "EXPIRES IN"
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
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to query workspace leases: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "test" => {
            let cfg = match crate::modules::supabase_sync::load_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[ERROR] Failed to load Supabase config: {}", e);
                    std::process::exit(1);
                }
            };
            let target_id = sub_args.first().map(|s| s.as_str());
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
                std::process::exit(0);
            }

            println!("[CLI] Testing {} Supabase endpoint(s)...", endpoints.len());
            for ep in endpoints {
                print!("  Connecting to [{}] {} ({}) ... ", ep.id, ep.name, ep.role);
                let client = match crate::modules::supabase_client::SupabaseClient::new(ep) {
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
                        } else {
                            println!("FAIL (HTTP {})", res.status_code.unwrap_or(0));
                        }
                        println!("    -> {}", res.message);
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
            }
            std::process::exit(0);
        }

        _ => {
            eprintln!(
                "[ERROR] Unknown subcommand: 'supabase {}'. Run 'antigravity-manager supabase --help' for guide.",
                sub
            );
            std::process::exit(1);
        }
    }
}

pub(crate) fn print_supabase_cli_help() {
    println!("================================================================================");
    println!("         Antigravity-Manager: Supabase Fleet Synchronization CLI                ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager supabase <subcommand> [options]");
    println!("  agm supabase <subcommand> [options]");
    println!();
    println!("Subcommands:");
    println!(
        "  status, ls                       Display Supabase node status, endpoints count & leases"
    );
    println!("  sync                             Trigger immediate node & instance profile sync");
    println!("  list-leases, leases              Show active cross-machine account leases");
    println!("  test [endpoint_id]               Test connection to configured Supabase endpoints");
    println!("================================================================================");
}
