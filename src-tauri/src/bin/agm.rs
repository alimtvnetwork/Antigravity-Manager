//! AGM - Antigravity-Manager Native Terminal CLI
//! Autonomous terminal companion for Antigravity-Manager:
//! Status monitoring, multi-instance management, fast-forward switching,
//! accounts listing, direct account switching, doctor health checks,
//! prompt inspection, proxy status/test, sync, git pull, clean/purge, logs,
//! PATH self-installation, GitHub auto-updates, and SSH remote machine management.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, config, email_inbound, email_io, email_sender,
    email_vault_db, email_watcher, instance, notification_hub, proxy_db, repo_db, security_db,
    supabase_sync, training_api,
};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        print_banner();
        print_help();
        return;
    }

    let subcommand = args[1].to_lowercase();
    let cmd_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    match subcommand.as_str() {
        "status" | "credits" | "credit" | "status/credits" => cmd_status(&cmd_args),
        "instances" | "instance" | "ls" => cmd_instances(&cmd_args),
        "instances-all" => cmd_instances_all(&cmd_args),
        "doctor" | "check" => cmd_doctor(),
        "accounts" | "account" | "acc" => cmd_accounts(&cmd_args),
        "switch" => cmd_switch(&cmd_args),
        "switch-if-low-credit" | "swlc" | "sfc" | "switch-if-no-credit" => {
            cmd_switch_if_low_credit(&cmd_args);
        }
        "which-prompts-running" | "wpr" => cmd_which_prompts_running(&cmd_args),
        "prompts" => cmd_prompts(&cmd_args),
        "prompt" => cmd_prompt_dispatch(&cmd_args),
        "rerun" => cmd_rerun(&cmd_args),
        "prompts-export" | "pe" => cmd_prompts_export(&cmd_args),
        "prompts-import" | "pi" => cmd_prompts_import(&cmd_args),
        "proxy" => cmd_proxy(&cmd_args),
        "sync" => cmd_sync(),
        "pull" => cmd_pull(),
        "clean" | "purge" => cmd_clean(),
        "clear-cache" | "cache-clear" => cmd_clear_cache(&cmd_args),
        "clear" => {
            if cmd_args
                .first()
                .map(|s| s.eq_ignore_ascii_case("cache"))
                .unwrap_or(false)
            {
                let rest = if cmd_args.len() > 1 {
                    cmd_args[1..].to_vec()
                } else {
                    Vec::new()
                };
                cmd_clear_cache(&rest);
            } else {
                cmd_clear_cache(&cmd_args);
            }
        }
        "recreate-project" => cmd_recreate_project(&cmd_args),
        "recreate" => cmd_recreate(&cmd_args),
        "email" => cmd_email(&cmd_args),
        "logs" | "log" => cmd_logs(&cmd_args),
        "ff" | "smart-switch" | "fast-forward" => cmd_fast_forward(&cmd_args),
        "test-switcher" | "test-auto-switch" | "auto-switch" => cmd_test_auto_switch(&cmd_args),
        "test-email" | "email-test" | "check-email" => cmd_test_email(&cmd_args),
        "test-training" | "training" | "train" => cmd_test_training(&cmd_args),
        "install" => cmd_install(),
        "update" => cmd_update(),
        "ssh" => cmd_ssh(&cmd_args),
        "version" | "--version" | "-v" => {
            println!("agm v{}", VERSION);
        }
        "help" | "--help" | "-h" => {
            print_banner();
            print_help();
        }
        _ => {
            eprintln!("Unknown command: '{}'", args[1]);
            eprintln!("Run 'agm help' for available commands.");
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("================================================================================");
    println!("             AGM - Antigravity-Manager Native Terminal CLI                      ");
    println!(
        "             Version: v{}                                                        ",
        VERSION
    );
    println!("================================================================================");
}

fn print_help() {
    println!("Usage:");
    println!("  agm <command> [arguments] [options]");
    println!();
    println!("Core Status & Account Rotation Commands:");
    println!(
        "  status, credits [--json]              Show node status, immediate & weekly credits"
    );
    println!(
        "  ff, smart-switch                      Trigger fast-forward rotation to freshest account"
    );
    println!(
        "  switch-if-low-credit, swlc, sfc [pct] Check live quota and rotate if below threshold"
    );
    println!("  accounts, acc [--active] [--json]     List registered accounts, tiers, and quotas");
    println!("  switch <email|prefix|id>              Switch active account directly without GUI");
    println!();
    println!("Prompt Inspection, Export/Import & Rerun Commands:");
    println!("  which-prompts-running, wpr [--json]   List running projects, conv IDs, and prompt queues");
    println!("  prompts ls [N] [--json] [--words W]   Show N running prompts in ASC stack order");
    println!("  prompts-export, pe [N] [-f <path>]    Export prompts with Base64 images to JSON");
    println!("  prompts-import, pi [-f <path>]        Import and rerun prompts from JSON file(s)");
    println!("  prompt \"<text>\" [--prefix C] [--suffix C] Dispatch prompt with git pull & 01-prompts templates");
    println!(
        "  rerun [prompts [N]] [-prefix <cat>]   Git pull and rerun last N prompts with template"
    );
    println!();
    println!("Instance & Workspace Management Commands:");
    println!("  instances [ls] [--json]               List all sandbox profiles and running PIDs");
    println!(
        "  instances <seq|id|alias> [switch] ff  Fast-forward rotate account for specific instance"
    );
    println!(
        "  instances-all ff                      Fast-forward rotate accounts across ALL instances"
    );
    println!("  instances create \"<name>\" [--data-only] Create a new isolated sandbox instance profile");
    println!("  instances rm <seq|id|alias>           Remove a specific sandbox instance profile");
    println!("  instances rm-all                      Remove all non-default instances (preserves default)");
    println!("  recreate-project [path]               Purge workspace cache & conversations and reopen in agy");
    println!("  recreate [project-or-path...]         Purge & recreate one or more projects in fresh session");
    println!("  clear-cache, cache-clear [-k N]       Prune old conversations (keep N=10) & clean caches");
    println!();
    println!("Email Telemetry & Vault Commands:");
    println!("  email [status] [--json]               Show email notification & IMAP/SMTP status");
    println!("  email help                            Show detailed email command & subject syntax guide");
    println!(
        "  email ls [--json]                     List configured email accounts and recipients"
    );
    println!("  email add <email> [pwd] [options]     Add sender account or recipient (sends JSON self-email)");
    println!("  email rm <seq|id|email>               Remove email account or recipient");
    println!("  email mv <seq|id|email> --default     Promote an email account to default sender");
    println!("  email export [-f <path>]              Export email configuration bundle to JSON");
    println!();
    println!("System & Maintenance Commands:");
    println!(
        "  doctor, check                         Run comprehensive pre-flight system health checks"
    );
    println!(
        "  proxy [status|test]                   Check local proxy service status or test loopback"
    );
    println!("  sync                                  Synchronize local accounts, instances, and DB vaults");
    println!(
        "  pull                                  Execute git pull origin main in repository root"
    );
    println!("  clean, purge                          Safely clean temp caches while protecting DB vaults");
    println!("  logs [--tail N] [-f text]             View recent application and proxy log lines");
    println!("  install                               Install 'agm' executable into system PATH");
    println!("  update                                Check GitHub releases and update AGM binary");
    println!(
        "  ssh <target> [options]                Connect to remote VM via SSH or run auto-update"
    );
    println!("  version, -v                           Print agm CLI version");
    println!("  help, -h                              Display this help manual");
    println!();
}

fn print_doctor_probe(name: &str, detail: &str, is_pass: bool) {
    if is_pass {
        println!("    [\x1b[32mPASS\x1b[0m] {:<30} {}", name, detail);
    } else {
        println!("    [\x1b[31mFAIL\x1b[0m] {:<30} {}", name, detail);
    }
}

fn cmd_doctor() {
    println!("================================================================================");
    println!("             AGM System Health Diagnostic (Doctor)                              ");
    println!("================================================================================");

    let mut checks_passed = 0;
    let mut checks_total = 0;

    // Probe 1: accounts.json vault
    checks_total += 1;
    let accounts_check = match account::load_account_index() {
        Ok(idx) => {
            checks_passed += 1;
            format!("Present ({} account(s) registered)", idx.accounts.len())
        }
        Err(e) => format!("Error reading index: {}", e),
    };
    print_doctor_probe(
        "Vault: accounts.json",
        &accounts_check,
        accounts_check.starts_with("Present"),
    );

    // Probe 2: email_vault.db
    checks_total += 1;
    let email_vault_check = match email_vault_db::get_email_vault_db_path() {
        Ok(p) => {
            if p.exists() {
                checks_passed += 1;
                format!(
                    "Healthy ({})",
                    p.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                "Not created yet (Standby)".to_string()
            }
        }
        Err(e) => format!("Error resolving path: {}", e),
    };
    let email_ok =
        email_vault_check.starts_with("Healthy") || email_vault_check.contains("Standby");
    print_doctor_probe("Vault: email_vault.db", &email_vault_check, email_ok);

    // Probe 3: repo_prompts.db
    checks_total += 1;
    let repo_db_check = match repo_db::get_repo_db_path() {
        Ok(p) => {
            if p.exists() {
                checks_passed += 1;
                format!(
                    "Healthy ({})",
                    p.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                "Not created yet (Standby)".to_string()
            }
        }
        Err(e) => format!("Error resolving path: {}", e),
    };
    let repo_ok = repo_db_check.starts_with("Healthy") || repo_db_check.contains("Standby");
    print_doctor_probe("Vault: repo_prompts.db", &repo_db_check, repo_ok);

    // Probe 4: security.db
    checks_total += 1;
    let sec_db_check = match security_db::get_security_db_path() {
        Ok(p) => {
            if p.exists() {
                checks_passed += 1;
                format!(
                    "Healthy ({})",
                    p.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                "Not created yet (Standby)".to_string()
            }
        }
        Err(e) => format!("Error resolving path: {}", e),
    };
    let sec_ok = sec_db_check.starts_with("Healthy") || sec_db_check.contains("Standby");
    print_doctor_probe("Vault: security.db", &sec_db_check, sec_ok);

    // Probe 5: thinking_store.db
    checks_total += 1;
    let thinking_check = match proxy_db::get_thinking_db_path() {
        Ok(p) => {
            if p.exists() {
                checks_passed += 1;
                format!(
                    "Healthy ({})",
                    p.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                "Not created yet (Standby)".to_string()
            }
        }
        Err(e) => format!("Error resolving path: {}", e),
    };
    let thinking_ok = thinking_check.starts_with("Healthy") || thinking_check.contains("Standby");
    print_doctor_probe("Vault: thinking_store.db", &thinking_check, thinking_ok);

    // Probe 6: Proxy Gateway (Port 8045)
    checks_total += 1;
    let addr: SocketAddr = "127.0.0.1:8045".parse().unwrap();
    let proxy_listening = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();
    let proxy_detail = if proxy_listening {
        checks_passed += 1;
        "Active (Port 8045 listening)".to_string()
    } else {
        "Offline (Port 8045 standby)".to_string()
    };
    print_doctor_probe("Proxy Gateway (8045)", &proxy_detail, proxy_listening);

    // Probe 7: Sandbox Profiles / Processes
    checks_total += 1;
    let instances_detail = match instance::list_instances() {
        Ok(list) => {
            let running = list.iter().filter(|i| i.is_running).count();
            checks_passed += 1;
            format!("{} configured, {} running", list.len(), running)
        }
        Err(e) => format!("Error querying instances: {}", e),
    };
    let inst_ok = !instances_detail.starts_with("Error");
    print_doctor_probe("Sandbox Instances", &instances_detail, inst_ok);

    // Probe 8: PATH registration
    checks_total += 1;
    let path_registered = check_is_in_path();
    let path_detail = if path_registered {
        checks_passed += 1;
        "Registered in system PATH".to_string()
    } else {
        "Not detected in PATH (run 'agm install')".to_string()
    };
    print_doctor_probe("System PATH Configuration", &path_detail, path_registered);

    // Probe 9: Network Identity
    checks_total += 1;
    let local_ip = email_watcher::detect_local_ip();
    let node_name = email_watcher::detect_machine_name();
    let net_detail = format!("Node: {} | IP: {}", node_name, local_ip);
    checks_passed += 1;
    print_doctor_probe("Network Identity", &net_detail, true);

    println!("{}", "-".repeat(80));
    let status_str = if checks_passed >= checks_total {
        "\x1b[32mHEALTHY\x1b[0m - All systems nominal"
    } else if checks_passed >= checks_total - 2 {
        "\x1b[33mDEGRADED\x1b[0m - Operational with minor standby services"
    } else {
        "\x1b[31mUNHEALTHY\x1b[0m - Critical probes failed"
    };
    println!("Overall Diagnostic Verdict: {}\n", status_str);
}

fn check_is_in_path() -> bool {
    let path_var = env::var("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    let current_exe_name = if cfg!(windows) { "agm.exe" } else { "agm" };

    for dir in path_var.split(separator) {
        let p = Path::new(dir).join(current_exe_name);
        if p.exists() {
            return true;
        }
    }
    false
}

fn cmd_accounts(args: &[String]) {
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
        if show_only_active {
            if !is_current {
                continue;
            }
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
        println!(
            "{}",
            serde_json::to_string_pretty(&json_items).unwrap_or_default()
        );
        return;
    }

    println!("\nRegistered Accounts ({} total):", account_rows.len());
    println!(
        "{:<5} {:<32} {:<10} {:<10} {:<14} {}",
        "INDEX", "EMAIL", "TIER", "STATUS", "QUOTA", "LAST USED"
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

fn cmd_switch(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: agm switch <email|prefix|id>");
        eprintln!("Example: agm switch abidul.rasia@gmail.com");
        std::process::exit(1);
    }

    let query = args[0].trim().to_lowercase();
    let index = match account::load_account_index() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("[ERROR] Failed to load accounts: {}", e);
            std::process::exit(1);
        }
    };

    let matches: Vec<_> = index
        .accounts
        .iter()
        .filter(|a| {
            let email_l = a.email.to_lowercase();
            let id_l = a.id.to_lowercase();
            email_l.contains(&query) || id_l.contains(&query)
        })
        .collect();

    if matches.is_empty() {
        eprintln!("[ERROR] No account found matching '{}'.", query);
        eprintln!("Run 'agm accounts' to list all registered accounts.");
        std::process::exit(1);
    }

    let target = if matches.len() == 1 {
        matches[0]
    } else {
        if let Some(exact) = matches.iter().find(|a| a.email.to_lowercase() == query) {
            *exact
        } else {
            eprintln!("[ERROR] Query '{}' matched multiple accounts:", query);
            for m in matches {
                eprintln!("  - {} (ID: {})", m.email, m.id);
            }
            eprintln!("Please specify a more precise email or ID.");
            std::process::exit(1);
        }
    };

    let prev_email = match account::get_current_account() {
        Ok(Some(curr)) => curr.email,
        _ => "(None / Standby)".to_string(),
    };

    if let Err(e) = account::set_current_account_id(&target.id) {
        eprintln!("[ERROR] Failed to switch active account: {}", e);
        std::process::exit(1);
    }

    let _ = account::apply_device_profile(&target.id);

    println!("[SUCCESS] Active account switched:");
    println!("          Previous: {}", prev_email);
    println!("          Active:   {} (ID: {})", target.email, target.id);
}

fn derive_current_repo_slug() -> String {
    env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .map(|s| {
            s.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .trim_matches('-')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "workspace".to_string())
}

fn truncate_words(text: &str, max_words: usize) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let total = words.len();
    if total <= max_words {
        (words.join(" "), total)
    } else {
        (format!("{} ...", words[..max_words].join(" ")), total)
    }
}

fn cmd_which_prompts_running(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");

    // Refresh live projects across registered instances
    if let Ok(reg) = instance::load_registry() {
        for inst in &reg.instances {
            let _ = repo_db::detect_running_projects(&inst.id);
        }
    }

    let projects = repo_db::list_running_projects().unwrap_or_default();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let conversations = agy_cleaner::scan_conversations(100);

    let mut rows = Vec::new();
    let mut seq = 0usize;

    for proj in &projects {
        let proj_prompts: Vec<&repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                (p.project_id == proj.id || p.repo_path.eq_ignore_ascii_case(&proj.repo_path))
                    && (p.status == "running"
                        || p.status == "backed_up"
                        || p.status == "dispatched")
            })
            .collect();

        if !proj.is_running && proj_prompts.is_empty() {
            continue;
        }

        seq += 1;
        let repo_norm = proj.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || (!proj.repo_name.is_empty()
                    && uris_norm.contains(&proj.repo_name.to_lowercase()))
        });

        let conv_id = matched_conv
            .map(|c| c.conversation_id.clone())
            .or_else(|| proj_prompts.first().and_then(|p| p.session_id.clone()))
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv
            .and_then(|c| {
                if c.title.trim().is_empty() {
                    None
                } else {
                    Some(c.title.clone())
                }
            })
            .unwrap_or_else(|| proj.repo_name.clone());

        let queue_count = if proj_prompts.is_empty() && proj.is_running {
            1usize
        } else {
            proj_prompts.len()
        };

        rows.push(serde_json::json!({
            "seq": seq,
            "project": proj.repo_name,
            "id": proj.id,
            "instance_id": proj.instance_id,
            "repo_path": proj.repo_path,
            "conv_id": conv_id,
            "conv_name": conv_name,
            "prompts_count": queue_count,
            "is_running": proj.is_running,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    if rows.is_empty() {
        println!("No projects currently have running or queued prompts.");
        return;
    }

    println!(
        "\nProjects with Running / Queued Prompts ({} active):",
        rows.len()
    );
    println!(
        "{:<5} {:<22} {:<24} {:<16} {:<26} {}",
        "SEQ", "PROJECT", "ID", "CONV ID", "CONV NAME", "PROMPTS (QUEUE)"
    );
    println!("{}", "-".repeat(110));

    for item in &rows {
        let s = item["seq"].as_u64().unwrap_or(0);
        let proj = item["project"].as_str().unwrap_or("-");
        let id: String = item["id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(22)
            .collect();
        let cid: String = item["conv_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(14)
            .collect();
        let cname: String = item["conv_name"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(24)
            .collect();
        let qcount = item["prompts_count"].as_u64().unwrap_or(0);
        println!(
            "#{:<4} {:<22} {:<24} {:<16} {:<26} {}",
            s, proj, id, cid, cname, qcount
        );
    }
    println!();
}

fn cmd_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("export") || first.eq_ignore_ascii_case("pe") {
            cmd_prompts_export(&args[1..]);
            return;
        }
        if first.eq_ignore_ascii_case("import") || first.eq_ignore_ascii_case("pi") {
            cmd_prompts_import(&args[1..]);
            return;
        }
    }

    let is_ls = args
        .first()
        .map(|a| a.eq_ignore_ascii_case("ls") || a.eq_ignore_ascii_case("list"))
        .unwrap_or(false);
    let is_json = args.iter().any(|a| a == "--json");
    let running_only = args.iter().any(|a| a == "--running") || is_ls;

    let mut limit_n: usize = 10;
    let mut max_words: usize = 100;

    let mut i = if is_ls { 1 } else { 0 };
    while i < args.len() {
        let arg = &args[i];
        if arg == "--words" || arg == "-w" {
            if i + 1 < args.len() {
                max_words = args[i + 1].parse().unwrap_or(100);
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            }
        }
        i += 1;
    }

    let cwd_opt = env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_lowercase().replace('\\', "/"));

    let mut all_prompts = repo_db::list_all_prompts().unwrap_or_default();

    // Filter to current repo if we are inside a repo folder that has tracked prompts
    if let Some(ref cwd) = cwd_opt {
        let repo_matches: Vec<repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd.starts_with(&rp) || rp.starts_with(cwd))
            })
            .cloned()
            .collect();
        if !repo_matches.is_empty() {
            all_prompts = repo_matches;
        }
    }

    if running_only {
        let running_filtered: Vec<repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                p.status == "running" || p.status == "dispatched" || p.status == "backed_up"
            })
            .cloned()
            .collect();
        if !running_filtered.is_empty() {
            all_prompts = running_filtered;
        }
    }

    // Take top N most recent and reverse into ASC stack order (oldest -> newest in the N window)
    all_prompts.truncate(limit_n);
    all_prompts.reverse();

    let mut stack_items = Vec::new();
    for (idx, p) in all_prompts.iter().enumerate() {
        let (snippet, word_count) = truncate_words(&p.prompt_content, max_words);
        stack_items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "model": p.model.clone().unwrap_or_else(|| "default".to_string()),
            "word_count": word_count,
            "words_limit": max_words,
            "prompt": snippet,
            "has_image": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stack_items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!(
        "\n[Table Mode: Displaying {} prompt(s) in ASC stack order (truncated to {} words; pass --json for raw JSON)]",
        stack_items.len(),
        max_words
    );

    if stack_items.is_empty() {
        println!("No active prompt tasks tracked in repo_prompts.db.");
    } else {
        println!(
            "{:<5} {:<10} {:<22} {:<12} {:<10} {}",
            "SEQ", "ID", "PROJECT", "STATUS", "WORDS", "PROMPT SNIPPET (ASC STACK)"
        );
        println!("{}", "-".repeat(110));
        for item in &stack_items {
            let seq = item["seq"].as_u64().unwrap_or(0);
            let short_id: String = item["id"].as_str().unwrap_or("-").chars().take(8).collect();
            let proj: String = item["project_id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(20)
                .collect();
            let status = item["status"].as_str().unwrap_or("-");
            let wc = item["word_count"].as_u64().unwrap_or(0);
            let prompt_txt = item["prompt"].as_str().unwrap_or("");
            println!(
                "#{:<4} {:<10} {:<22} {:<12} {:<10} {}",
                seq, short_id, proj, status, wc, prompt_txt
            );
        }
        println!();
    }

    if !is_ls {
        scan_prompt_templates();
    }
}

fn cmd_prompts_export(args: &[String]) {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    let mut limit_n: usize = 50;
    let mut target_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" || arg == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                target_file = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            } else if target_file.is_none() {
                target_file = Some(arg.clone());
            }
        }
        i += 1;
    }

    let slug = derive_current_repo_slug();
    let default_filename = format!("agm-{}-prompts.json", slug);
    let out_path = match target_file {
        Some(ref f) if !f.trim().is_empty() => PathBuf::from(f.trim()),
        _ => PathBuf::from(&default_filename),
    };

    let mut prompts = repo_db::list_all_prompts().unwrap_or_default();
    if let Ok(cwd) = env::current_dir() {
        let cwd_norm = cwd.to_string_lossy().to_lowercase().replace('\\', "/");
        let matched: Vec<repo_db::ActivePrompt> = prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd_norm.starts_with(&rp) || rp.starts_with(&cwd_norm))
            })
            .cloned()
            .collect();
        if !matched.is_empty() {
            prompts = matched;
        }
    }

    prompts.truncate(limit_n);
    prompts.reverse(); // ASC stack order

    let exported_items: Vec<serde_json::Value> = prompts
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let b64_image = p.image_payload.as_ref().map(|img| {
                if img.starts_with("data:image/") || img.len() > 128 {
                    img.clone()
                } else if Path::new(img).exists() {
                    fs::read(img)
                        .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
                        .unwrap_or_else(|_| STANDARD.encode(img.as_bytes()))
                } else {
                    STANDARD.encode(img.as_bytes())
                }
            });
            serde_json::json!({
                "seq": idx + 1,
                "id": p.id,
                "project_id": p.project_id,
                "instance_id": p.instance_id,
                "repo_path": p.repo_path,
                "prompt_content": p.prompt_content,
                "model": p.model,
                "session_id": p.session_id,
                "status": p.status,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "image_base64": b64_image,
            })
        })
        .collect();

    let bundle = serde_json::json!({
        "schema": "agm-prompts-export-v1",
        "repo_slug": slug,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "count": exported_items.len(),
        "prompts": exported_items,
    });

    let json_str = serde_json::to_string_pretty(&bundle).unwrap_or_else(|_| "{}".to_string());
    if let Err(e) = fs::write(&out_path, &json_str) {
        eprintln!(
            "[ERROR] Failed to write prompts export to {:?}: {}",
            out_path, e
        );
        std::process::exit(1);
    }

    println!(
        "[SUCCESS] Exported {} prompt(s) (with Base64 image encoding) to {:?}",
        exported_items.len(),
        out_path
    );
}

fn cmd_prompts_import(args: &[String]) {
    let mut explicit_file: Option<String> = None;
    let auto_yes = args.iter().any(|a| a == "--yes" || a == "-y");

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" || arg == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                explicit_file = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') && explicit_file.is_none() {
            explicit_file = Some(arg.clone());
        }
        i += 1;
    }

    let slug = derive_current_repo_slug();
    let default_filename = format!("agm-{}-prompts.json", slug);

    let mut files_to_import: Vec<PathBuf> = Vec::new();
    if let Some(ref f) = explicit_file {
        files_to_import.push(PathBuf::from(f));
    } else {
        let default_path = PathBuf::from(&default_filename);
        if default_path.exists() {
            files_to_import.push(default_path.clone());
        }

        // Scan current directory for other prompt JSON files
        if let Ok(entries) = fs::read_dir(".") {
            let mut other_jsons = Vec::new();
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json") {
                    let fname = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if fname != default_filename
                        && (fname.contains("prompt") || fname.starts_with("agm-"))
                    {
                        other_jsons.push(p);
                    }
                }
            }

            if !other_jsons.is_empty() {
                if auto_yes {
                    files_to_import.extend(other_jsons);
                } else {
                    println!(
                        "[*] Found {} additional prompt JSON file(s) in current folder:",
                        other_jsons.len()
                    );
                    for oj in &other_jsons {
                        println!("    - {}", oj.display());
                    }
                    print!("Do you want to import and rerun these additional JSON files as well? [y/N]: ");
                    let _ = io::stdout().flush();
                    let mut answer = String::new();
                    if io::stdin().read_line(&mut answer).is_ok() {
                        let trimmed = answer.trim().to_lowercase();
                        if trimmed == "y" || trimmed == "yes" {
                            files_to_import.extend(other_jsons);
                        }
                    }
                }
            }
        }
    }

    if files_to_import.is_empty() {
        eprintln!(
            "[ERROR] No prompt JSON file found to import (expected '{}' or specify -f <path>).",
            default_filename
        );
        std::process::exit(1);
    }

    let conn = match repo_db::connect_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to connect to repo_prompts.db: {}", e);
            std::process::exit(1);
        }
    };

    let cwd_str = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().timestamp();
    let mut total_imported = 0usize;

    for file_path in &files_to_import {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[WARN] Failed to read {:?}: {}", file_path, e);
                continue;
            }
        };
        let parsed: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[WARN] Invalid JSON in {:?}: {}", file_path, e);
                continue;
            }
        };

        let arr = parsed
            .get("prompts")
            .and_then(|v| v.as_array())
            .or_else(|| parsed.as_array());

        let Some(items) = arr else {
            continue;
        };

        for item in items {
            let prompt_content = item
                .get("prompt_content")
                .or_else(|| item.get("prompt"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if prompt_content.trim().is_empty() {
                continue;
            }

            let id = uuid::Uuid::new_v4().to_string();
            let proj_id = item
                .get("project_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&slug)
                .to_string();
            let inst_id = item
                .get("instance_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
            let repo_path = item
                .get("repo_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(&cwd_str)
                .to_string();
            let model = item
                .get("model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some("gemini-3.8-flash-high".to_string()));
            let img = item
                .get("image_base64")
                .or_else(|| item.get("image_payload"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let _ = conn.execute(
                "INSERT INTO active_prompts \
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'dispatched', ?8, ?8, ?9)",
                rusqlite::params![
                    &id,
                    &proj_id,
                    &inst_id,
                    &repo_path,
                    &prompt_content,
                    &model,
                    &proj_id,
                    now,
                    &img,
                ],
            );

            // Also write .antigravity_resume_task.json in target repo to trigger immediate rerun
            let task_file = PathBuf::from(&repo_path).join(".antigravity_resume_task.json");
            let task_payload = serde_json::json!({
                "prompt_id": id,
                "project_id": proj_id,
                "instance_id": inst_id,
                "repo_path": repo_path,
                "prompt_content": prompt_content,
                "model": model,
                "image_payload": img,
                "auto_boot": true,
                "imported_at": now,
            });
            if let Ok(js) = serde_json::to_string_pretty(&task_payload) {
                let _ = fs::write(&task_file, js);
            }

            total_imported += 1;
        }
    }

    println!(
        "[SUCCESS] Imported and queued {} prompt(s) for rerun across {} file(s).",
        total_imported,
        files_to_import.len()
    );
}

fn resolve_prompt_template(category_or_name: &str) -> Option<String> {
    let query = category_or_name.trim().to_lowercase();
    if query.is_empty() {
        return None;
    }

    let mut search_roots = vec![PathBuf::from("01-prompts")];
    if let Ok(cwd) = env::current_dir() {
        search_roots.push(cwd.join("01-prompts"));
        if let Some(parent) = cwd.parent() {
            search_roots.push(parent.join("coding-guidelines").join("01-prompts"));
            search_roots.push(parent.join("Antigravity-Manager").join("01-prompts"));
        }
    }

    for root in search_roots {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                if fname.contains(&query) {
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            return Some(content.trim().to_string());
                        }
                    } else if path.is_dir() {
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            let mut md_files: Vec<PathBuf> = sub_entries
                                .flatten()
                                .map(|e| e.path())
                                .filter(|p| p.is_file())
                                .collect();
                            md_files.sort();
                            if let Some(first_file) = md_files.first() {
                                if let Ok(content) = fs::read_to_string(first_file) {
                                    return Some(content.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn wrap_prompt_with_templates(
    raw_prompt: &str,
    prefix_cat: Option<&str>,
    suffix_cat: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(pref) = prefix_cat {
        if let Some(tpl) = resolve_prompt_template(pref) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Prefix: {}]", pref));
        }
    }
    if !raw_prompt.trim().is_empty() {
        parts.push(raw_prompt.trim().to_string());
    }
    if let Some(suff) = suffix_cat {
        if let Some(tpl) = resolve_prompt_template(suff) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Suffix: {}]", suff));
        }
    }
    parts.join("\n\n")
}

fn cmd_prompt_dispatch(args: &[String]) {
    if args.is_empty() {
        cmd_prompts(args);
        return;
    }

    // If first arg is "ls" or "--running", delegate to cmd_prompts
    if let Some(first) = args.first() {
        if first == "ls" || first == "list" || first == "--running" {
            cmd_prompts(args);
            return;
        }
    }

    let mut prefix_cat: Option<String> = None;
    let mut suffix_cat: Option<String> = None;
    let mut text_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--prefix" || arg == "-prefix" {
            if i + 1 < args.len() {
                prefix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "--suffix" || arg == "-suffix" {
            if i + 1 < args.len() {
                suffix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else {
            text_parts.push(arg.clone());
        }
        i += 1;
    }

    // Pull latest changes before dispatching prompt
    if Path::new(".git").exists() {
        println!("[*] Synchronizing repository via git pull before prompt dispatch...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let raw_text = text_parts.join(" ");
    let final_prompt =
        wrap_prompt_with_templates(&raw_text, prefix_cat.as_deref(), suffix_cat.as_deref());

    if final_prompt.trim().is_empty() {
        eprintln!("[ERROR] Prompt content cannot be empty.");
        std::process::exit(1);
    }

    let slug = derive_current_repo_slug();
    let cwd_str = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().timestamp();
    let prompt_id = uuid::Uuid::new_v4().to_string();
    let inst_id = instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());

    if let Ok(conn) = repo_db::connect_db() {
        let _ = conn.execute(
            "INSERT INTO active_prompts \
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'gemini-3.8-flash-high', ?2, 'dispatched', ?6, ?6)",
            rusqlite::params![&prompt_id, &slug, &inst_id, &cwd_str, &final_prompt, now],
        );
    }

    let task_file = PathBuf::from(&cwd_str).join(".antigravity_resume_task.json");
    let payload = serde_json::json!({
        "prompt_id": prompt_id,
        "project_id": slug,
        "instance_id": inst_id,
        "repo_path": cwd_str,
        "prompt_content": final_prompt,
        "prefix_template": prefix_cat,
        "suffix_template": suffix_cat,
        "dispatched_at": now,
    });
    if let Ok(js) = serde_json::to_string_pretty(&payload) {
        let _ = fs::write(&task_file, js);
    }

    println!(
        "[SUCCESS] Dispatched prompt ({} chars) to workspace '{}' [Instance: {}].",
        final_prompt.len(),
        slug,
        inst_id
    );
}

fn cmd_rerun(args: &[String]) {
    let mut count_n: usize = 1;
    let mut prefix_cat: Option<String> = None;
    let mut suffix_cat: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.eq_ignore_ascii_case("prompts") || arg.eq_ignore_ascii_case("prompt") {
            i += 1;
            continue;
        } else if arg == "-prefix" || arg == "--prefix" {
            if i + 1 < args.len() {
                prefix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "-suffix" || arg == "--suffix" {
            if i + 1 < args.len() {
                suffix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if let Ok(n) = arg.parse::<usize>() {
            count_n = n.max(1);
        }
        i += 1;
    }

    if Path::new(".git").exists() {
        println!("[*] Running git pull before rerunning prompt(s)...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let mut prompts = repo_db::list_all_prompts().unwrap_or_default();
    if let Ok(cwd) = env::current_dir() {
        let cwd_norm = cwd.to_string_lossy().to_lowercase().replace('\\', "/");
        let repo_matched: Vec<repo_db::ActivePrompt> = prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd_norm.starts_with(&rp) || rp.starts_with(&cwd_norm))
            })
            .cloned()
            .collect();
        if !repo_matched.is_empty() {
            prompts = repo_matched;
        }
    }

    if prompts.is_empty() {
        eprintln!("[ERROR] No historical prompts found in repo_prompts.db to rerun.");
        std::process::exit(1);
    }

    prompts.truncate(count_n);
    prompts.reverse(); // Rerun in chronological ASC order

    let now = chrono::Utc::now().timestamp();
    let conn_opt = repo_db::connect_db().ok();

    for (idx, p) in prompts.iter().enumerate() {
        let wrapped = wrap_prompt_with_templates(
            &p.prompt_content,
            prefix_cat.as_deref(),
            suffix_cat.as_deref(),
        );
        if let Some(ref conn) = conn_opt {
            let _ = conn.execute(
                "UPDATE active_prompts SET prompt_content = ?1, status = 'dispatched', updated_at = ?2 WHERE id = ?3",
                rusqlite::params![&wrapped, now, &p.id],
            );
        }

        let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "prompt_content": wrapped,
            "model": p.model,
            "image_payload": p.image_payload,
            "rerun_seq": idx + 1,
            "resumed_at": now,
        });
        if let Ok(js) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, js);
        }

        let (preview, _) = truncate_words(&wrapped, 20);
        println!(
            "  [Rerun #{}] Project '{}' -> {}",
            idx + 1,
            p.project_id,
            preview
        );
    }

    println!(
        "[SUCCESS] Queued and dispatched {} prompt(s) for rerun.",
        prompts.len()
    );
}

fn scan_prompt_templates() {
    let prompts_dir = Path::new("01-prompts");
    if prompts_dir.exists() {
        if let Ok(entries) = fs::read_dir(prompts_dir) {
            let mut categories = Vec::new();
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    categories.push(entry.file_name().to_string_lossy().to_string());
                }
            }
            if !categories.is_empty() {
                println!(
                    "Available Prompt Categories in 01-prompts/ ({} found):",
                    categories.len()
                );
                for cat in categories.iter().take(10) {
                    println!("  - 01-prompts/{}", cat);
                }
                if categories.len() > 10 {
                    println!("  ... and {} more categories.", categories.len() - 10);
                }
                println!();
            }
        }
    }
}

fn cmd_proxy(args: &[String]) {
    let is_test = args.iter().any(|a| a == "test");

    if is_test {
        println!("[*] Testing proxy loopback connectivity...");
        let start = std::time::Instant::now();
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[ERROR] Failed to create HTTP client: {}", e);
                return;
            }
        };

        match client.get("http://127.0.0.1:8045/accounts/current").send() {
            Ok(resp) => {
                let elapsed = start.elapsed().as_millis();
                println!(
                    "[OK] Proxy responded with HTTP {} in {}ms",
                    resp.status(),
                    elapsed
                );
            }
            Err(e) => {
                let elapsed = start.elapsed().as_millis();
                eprintln!(
                    "[FAIL] Proxy loopback test failed after {}ms: {}",
                    elapsed, e
                );
                eprintln!(
                    "       Ensure Antigravity-Manager GUI is running or proxy daemon is active."
                );
            }
        }
        return;
    }

    let addr: SocketAddr = "127.0.0.1:8045".parse().unwrap();
    let is_listening = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();

    println!("[*] Antigravity-Manager Proxy Gateway Status:");
    println!("    Proxy Address:   http://127.0.0.1:8045");
    println!(
        "    Socket Status:   {}",
        if is_listening {
            "ONLINE (Listening)"
        } else {
            "OFFLINE (Standby)"
        }
    );
    println!("    Supported Routes:");
    println!("      - Claude Messages:     POST /v1/messages");
    println!("      - OpenAI Completions:  POST /v1/chat/completions");
    println!("      - Gemini Models:       POST /v1beta/models/*");
    println!("      - Active Account:      GET  /accounts/current");
    println!("      - Token Analytics:     GET  /tokens");
    println!();
    println!("Run 'agm proxy test' to verify loopback latency.");
}

fn cmd_sync() {
    println!("[*] Synchronizing Antigravity-Manager state & split vaults...");

    // Validate accounts
    match account::load_account_index() {
        Ok(idx) => {
            println!(
                "    [✓] Accounts index validated ({} account(s))",
                idx.accounts.len()
            );
        }
        Err(e) => {
            eprintln!("    [✗] Accounts index check failed: {}", e);
        }
    }

    // Refresh instances
    match instance::list_instances() {
        Ok(list) => {
            println!(
                "    [✓] Sandbox instances synchronized ({} profile(s))",
                list.len()
            );
        }
        Err(e) => {
            eprintln!("    [✗] Instance query failed: {}", e);
        }
    }

    // Touch databases
    if repo_db::connect_db().is_ok() {
        println!("    [✓] repo_prompts.db schema verified");
    }
    if email_vault_db::connect_vault_db().is_ok() {
        println!("    [✓] email_vault.db schema verified");
    }

    println!("[SUCCESS] AGM state synchronization complete.");
}

fn cmd_pull() {
    println!("[*] Executing git pull in Antigravity-Manager repository...");

    let res = Command::new("git")
        .args(["pull", "origin", "main"])
        .output();

    match res {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.is_empty() {
                for line in stdout.lines() {
                    println!("    [git] {}", line);
                }
            }
            if !stderr.is_empty() {
                for line in stderr.lines() {
                    eprintln!("    [git] {}", line);
                }
            }
            if out.status.success() {
                println!("[SUCCESS] Git pull completed successfully.");
            } else {
                eprintln!("[ERROR] Git pull exited with code {:?}", out.status.code());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to execute git: {}", e);
        }
    }
}

fn is_safe_to_delete(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if name.contains("vault")
        || name.contains("account")
        || name.contains(".db")
        || name.contains("config")
    {
        return false;
    }
    true
}

fn cmd_clean() {
    println!("[*] Performing safe AGM storage and build cache hygiene...");

    let mut removed_dirs = 0;
    let mut reclaimed_bytes: u64 = 0;

    // 1. Clean temporary test directories in OS temp
    let temp_dir = env::temp_dir();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("antigravity_test_") {
                let p = entry.path();
                // Safety invariant: NEVER delete vault files
                if is_safe_to_delete(&p) {
                    if let Ok(meta) = fs::metadata(&p) {
                        reclaimed_bytes += meta.len();
                    }
                    if fs::remove_dir_all(&p).is_ok() {
                        removed_dirs += 1;
                    }
                }
            }
        }
    }

    println!(
        "    [✓] Temporary test artifacts removed: {} folder(s)",
        removed_dirs
    );
    println!("    [✓] Safety invariant verified: all database vaults strictly protected.");
    println!(
        "[SUCCESS] Cleanup finished. Space reclaimed: {} KB.",
        reclaimed_bytes / 1024
    );
}

fn cmd_logs(args: &[String]) {
    let mut tail = 25;
    let mut filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--tail" || args[i] == "-n" {
            if i + 1 < args.len() {
                tail = args[i + 1].parse().unwrap_or(25);
                i += 2;
                continue;
            }
        } else if args[i] == "--filter" || args[i] == "-f" {
            if i + 1 < args.len() {
                filter = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    let data_dir = match account::get_data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ERROR] Could not resolve data dir: {}", e);
            return;
        }
    };

    let log_file = data_dir.join("logs").join("antigravity.log");
    let fallback_log = PathBuf::from("antigravity.log");

    let target_log = if log_file.exists() {
        log_file
    } else if fallback_log.exists() {
        fallback_log
    } else {
        println!("No log file found at {:?}.", log_file);
        return;
    };

    println!("[*] Reading logs from {:?} (tail: {})...", target_log, tail);
    let file = match fs::File::open(&target_log) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[ERROR] Failed to open log file: {}", e);
            return;
        }
    };

    let reader = io::BufReader::new(file);
    let mut matched_lines = Vec::new();

    for line in reader.lines().flatten() {
        if let Some(ref kw) = filter {
            if !line.to_lowercase().contains(&kw.to_lowercase()) {
                continue;
            }
        }
        matched_lines.push(line);
    }

    let start_idx = if matched_lines.len() > tail {
        matched_lines.len() - tail
    } else {
        0
    };

    for line in &matched_lines[start_idx..] {
        println!("{}", line);
    }
}

fn extract_immediate_and_weekly_credits(
    acc: &antigravity_tools_lib::models::Account,
) -> (f64, f64, String) {
    let tier = acc
        .quota
        .as_ref()
        .and_then(|q| q.subscription_tier.clone())
        .unwrap_or_else(|| "FREE".to_string());

    let app_cfg = config::load_app_config().unwrap_or_default();
    let target_model = &app_cfg.auto_profile_switcher.target_model;

    let immediate_pct = auto_switcher::calculate_account_quota(acc, target_model).unwrap_or(100.0);

    let mut weekly_pct = immediate_pct;
    if let Some(ref q) = acc.quota {
        if let Some(ref groups) = q.quota_groups {
            let mut weekly_vals = Vec::new();
            for g in groups {
                for b in &g.buckets {
                    let w = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    if w.contains("week") || bid.contains("week") {
                        weekly_vals.push((b.remaining_fraction * 100.0).round());
                    }
                }
            }
            if !weekly_vals.is_empty() {
                weekly_pct = weekly_vals.into_iter().fold(100.0, f64::min);
            }
        }
    }

    (immediate_pct, weekly_pct, tier)
}

fn cmd_status(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");
    let machine_name = email_watcher::detect_machine_name();
    let local_ip = email_watcher::detect_local_ip();
    let active_acc = account::get_current_account().ok().flatten();

    let (immediate_quota, weekly_quota, tier) = match active_acc.as_ref() {
        Some(acc) => extract_immediate_and_weekly_credits(acc),
        None => (0.0, 0.0, "NONE".to_string()),
    };

    let instances_list = instance::list_instances().unwrap_or_default();
    let running_instances = instances_list.iter().filter(|i| i.is_running).count();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = all_prompts
        .iter()
        .filter(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        .count();

    if is_json {
        let out = serde_json::json!({
            "version": VERSION,
            "machine_name": machine_name,
            "local_ip": local_ip,
            "active_account": active_acc.as_ref().map(|a| a.email.clone()),
            "active_account_id": active_acc.as_ref().map(|a| a.id.clone()),
            "tier": tier,
            "immediate_quota_percent": immediate_quota,
            "weekly_quota_percent": weekly_quota,
            "instances_total": instances_list.len(),
            "instances_running": running_instances,
            "prompts_running": running_prompts,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    println!("[*] Antigravity-Manager Node & Credits Status:");
    println!("    Machine Name:      {}", machine_name);
    println!("    Local IP:          {}", local_ip);
    println!("    CLI Version:       v{}", VERSION);

    if let Some(acc) = active_acc {
        println!("    Active Account:    {} [{}]", acc.email, tier);
        println!(
            "    Account Name:      {}",
            acc.name.as_deref().unwrap_or("-")
        );
        println!("    Immediate Credits: {:.1}% remaining", immediate_quota);
        println!("    Weekly Credits:    {:.1}% remaining", weekly_quota);
    } else {
        println!("    Active Account:    (None / Default)");
    }

    println!(
        "    Sandbox Profiles:  {} configured ({} currently running)",
        instances_list.len(),
        running_instances
    );
    println!("    Queued/Running Prompts: {}", running_prompts);
}

fn cmd_switch_if_low_credit(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let mut custom_threshold: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--threshold" || arg == "-t" {
            if i + 1 < args.len() {
                custom_threshold = args[i + 1].parse::<f64>().ok();
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(val) = arg.parse::<f64>() {
                custom_threshold = Some(val);
            }
        }
        i += 1;
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let status_before = auto_switcher::get_status();
    let res = rt.block_on(auto_switcher::check_and_rotate_for_threshold(
        custom_threshold,
        force,
    ));
    let status_after = auto_switcher::get_status();

    match res {
        Ok(Some(reason)) => {
            if is_json {
                let out = serde_json::json!({
                    "rotated": true,
                    "reason": reason,
                    "previous_account": status_before.active_account_email,
                    "active_account": status_after.active_account_email,
                    "quota_percent": status_after.current_quota_percent,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                println!("[SUCCESS] Low-credit rotation triggered!");
                println!("          Reason: {}", reason);
                println!(
                    "          Active Account: {}",
                    status_after
                        .active_account_email
                        .as_deref()
                        .unwrap_or("(None)")
                );
            }
        }
        Ok(None) => {
            if is_json {
                let out = serde_json::json!({
                    "rotated": false,
                    "reason": "Quota is healthy (above threshold) or no alternative candidate needed",
                    "active_account": status_after.active_account_email,
                    "quota_percent": status_after.current_quota_percent,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                println!(
                    "[OK] Credits are sufficient ({:.1}% remaining on {}). No switch needed.",
                    status_after.current_quota_percent.unwrap_or(100.0),
                    status_after
                        .active_account_email
                        .as_deref()
                        .unwrap_or("current profile")
                );
            }
        }
        Err(e) => {
            if is_json {
                let out = serde_json::json!({
                    "rotated": false,
                    "error": e,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                eprintln!("[ERROR] switch-if-low-credit failed: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_clear_cache(args: &[String]) {
    let mut keep_count: usize = 10;
    let is_json = args.iter().any(|a| a == "--json");

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--keep" || arg == "-k" {
            if i + 1 < args.len() {
                keep_count = args[i + 1].parse::<usize>().unwrap_or(10);
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                keep_count = n;
            }
        }
        i += 1;
    }

    if !is_json {
        println!(
            "[*] Pruning old conversations (keeping {} most recent) and cleaning application caches...",
            keep_count
        );
    }

    match agy_cleaner::prune_and_clean(keep_count) {
        Ok(res) => {
            if is_json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
            } else {
                println!(
                    "    [✓] Conversations preserved: {} | pruned: {}",
                    res.preserved_count, res.pruned_count
                );
                println!(
                    "    [✓] Total disk space freed: {:.2} MB (Staged at: {})",
                    res.total_freed_bytes as f64 / 1024.0 / 1024.0,
                    res.staging_dir
                );
                cmd_clean();
            }
        }
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "error": e }))
                        .unwrap_or_default()
                );
            } else {
                eprintln!("[WARN] Conversation prune warning: {}", e);
                cmd_clean();
            }
        }
    }
}

fn cmd_instances(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();

    // Subcommand: agm instances all ff
    if non_flag_args.len() >= 2
        && non_flag_args[0].eq_ignore_ascii_case("all")
        && (non_flag_args[1].eq_ignore_ascii_case("ff")
            || non_flag_args[1].eq_ignore_ascii_case("switch"))
    {
        cmd_instances_all(args);
        return;
    }

    // Subcommand: agm instances rm-all
    if args
        .first()
        .map(|s| s.eq_ignore_ascii_case("rm-all") || s.eq_ignore_ascii_case("remove-all"))
        .unwrap_or(false)
    {
        let instances = instance::list_instances().unwrap_or_default();
        let mut removed = 0usize;
        for inst in instances {
            if inst.config.is_default || inst.config.id == "default" {
                continue;
            }
            if instance::delete_instance(&inst.config.id).is_ok() {
                println!(
                    "  [✓] Removed instance '{}' ({})",
                    inst.config.name, inst.config.id
                );
                removed += 1;
            }
        }
        println!(
            "[SUCCESS] Removed {} non-default instance(s). Default profile preserved.",
            removed
        );
        return;
    }

    // Subcommand: agm instances create "name" [--data-only | --do]
    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("create") || s.eq_ignore_ascii_case("add"))
        .unwrap_or(false)
    {
        let is_data_only = args
            .iter()
            .any(|a| a == "--data-only" || a == "-data-only" || a == "--do" || a == "-do");
        let name = non_flag_args
            .get(1)
            .map(|s| (*s).clone())
            .unwrap_or_else(|| format!("Instance-{}", chrono::Utc::now().timestamp() % 1000));

        match instance::create_instance(name) {
            Ok(cfg) => {
                if !is_data_only {
                    let _ = instance::clone_instance_executable(&cfg.id);
                }
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
                } else {
                    println!(
                        "[SUCCESS] Created instance #{}: '{}' (ID: {}, data_only: {}, dir: {})",
                        cfg.seq_num.unwrap_or(1),
                        cfg.name,
                        cfg.id,
                        is_data_only,
                        cfg.data_dir
                    );
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to create instance: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Subcommand: agm instances rm <target> OR agm instances <target> rm
    if non_flag_args.len() >= 2
        && (non_flag_args[0].eq_ignore_ascii_case("rm")
            || non_flag_args[0].eq_ignore_ascii_case("remove")
            || non_flag_args[0].eq_ignore_ascii_case("delete")
            || non_flag_args[1].eq_ignore_ascii_case("rm")
            || non_flag_args[1].eq_ignore_ascii_case("remove"))
    {
        let target_spec = if non_flag_args[0].eq_ignore_ascii_case("rm")
            || non_flag_args[0].eq_ignore_ascii_case("remove")
            || non_flag_args[0].eq_ignore_ascii_case("delete")
        {
            non_flag_args[1]
        } else {
            non_flag_args[0]
        };

        let resolved_id = match instance::resolve_instance_id(target_spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Could not resolve instance '{}': {}",
                    target_spec, e
                );
                std::process::exit(1);
            }
        };

        match instance::delete_instance(&resolved_id) {
            Ok(_) => println!("[SUCCESS] Deleted instance '{}'.", resolved_id),
            Err(e) => {
                eprintln!("[ERROR] Failed to delete instance '{}': {}", resolved_id, e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Subcommand: agm instances <seq|id|alias> [switch] ff
    if non_flag_args.len() >= 2 {
        let last_arg = non_flag_args.last().unwrap().to_lowercase();
        let second_arg = non_flag_args[1].to_lowercase();
        if last_arg == "ff"
            || last_arg == "fast-forward"
            || second_arg == "ff"
            || second_arg == "switch"
        {
            let target_spec = non_flag_args[0];
            let resolved_id = match instance::resolve_instance_id(target_spec) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[ERROR] Could not resolve instance '{}': {}",
                        target_spec, e
                    );
                    std::process::exit(1);
                }
            };
            println!(
                "[*] Fast-forward rotating account for instance '{}' (resolved from '{}')...",
                resolved_id, target_spec
            );
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
                &resolved_id,
            ))) {
                Ok(msg) => println!("[SUCCESS] {}", msg),
                Err(e) => {
                    eprintln!("[ERROR] Instance fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    // Default: agm instances [ls] [--json]
    match instance::list_instances() {
        Ok(instances) => {
            let node_alias = supabase_sync::load_config()
                .map(|c| c.node_alias)
                .unwrap_or_else(|_| email_watcher::detect_machine_name());
            let local_ip = supabase_sync::get_local_ip();

            if is_json {
                let items: Vec<serde_json::Value> = instances
                    .iter()
                    .enumerate()
                    .map(|(idx, inst)| {
                        let seq = inst.config.seq_num.unwrap_or((idx + 1) as u32);
                        let eff_email = inst.config.bound_email.clone().or_else(|| {
                            if inst.config.is_default || inst.config.id == "default" {
                                account::get_current_account()
                                    .ok()
                                    .flatten()
                                    .map(|a| a.email)
                            } else {
                                None
                            }
                        });
                        serde_json::json!({
                            "seq": seq,
                            "id": inst.config.id,
                            "name": inst.config.name,
                            "is_default": inst.config.is_default,
                            "is_running": inst.is_running,
                            "pid": inst.pid,
                            "bound_account_id": inst.config.bound_account_id,
                            "bound_email": eff_email,
                            "node": node_alias,
                            "local_ip": local_ip,
                            "data_dir": inst.config.data_dir,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
                );
                return;
            }

            if instances.is_empty() {
                println!("No sandbox profiles registered yet.");
                return;
            }

            println!(
                "\nRegistered Sandbox Profiles ({} total) [Node: {} | IP: {}]:",
                instances.len(),
                node_alias,
                local_ip
            );
            println!(
                "{:<5} {:<16} {:<20} {:<18} {:<24} {:<20} {}",
                "#", "ID", "NAME", "STATUS", "BOUND ACCOUNT", "NODE / IP", "DATA DIR"
            );
            println!("{}", "-".repeat(115));

            for (idx, inst) in instances.iter().enumerate() {
                let seq_str = match inst.config.seq_num {
                    Some(s) => format!("#{}", s),
                    None => format!("#{}", idx + 1),
                };
                let status_str = if inst.is_running {
                    format!("Running (PID: {})", inst.pid.unwrap_or(0))
                } else {
                    "Idle".to_string()
                };
                let email = inst
                    .config
                    .bound_email
                    .clone()
                    .or_else(|| {
                        if inst.config.is_default || inst.config.id == "default" {
                            account::get_current_account()
                                .ok()
                                .flatten()
                                .map(|a| a.email)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());
                let node_info = format!("{}/{}", node_alias, local_ip);
                println!(
                    "{:<5} {:<16} {:<20} {:<18} {:<24} {:<20} {}",
                    seq_str,
                    inst.config.id,
                    inst.config.name,
                    status_str,
                    email,
                    node_info,
                    inst.config.data_dir
                );
            }
            println!();
        }
        Err(e) => {
            eprintln!("Error querying instances: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_instances_all(_args: &[String]) {
    let instances = match instance::list_instances() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[ERROR] Failed to query instances: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "[*] Triggering fast-forward rotation across all {} registered instance(s)...",
        instances.len()
    );
    let rt = tokio::runtime::Runtime::new().expect("Failed to initialize async runtime");

    for inst in instances {
        print!(
            "  -> Instance '{}' ({}): ",
            inst.config.name, inst.config.id
        );
        let _ = io::stdout().flush();
        match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
            &inst.config.id,
        ))) {
            Ok(msg) => println!("[OK] {}", msg),
            Err(e) => println!("[SKIPPED/WARN] {}", e),
        }
    }
}

fn cmd_fast_forward(args: &[String]) {
    if let Some(target) = args.first() {
        if !target.starts_with('-') {
            let resolved =
                instance::resolve_instance_id(target).unwrap_or_else(|_| target.to_string());
            println!(
                "[*] Triggering fast-forward account rotation for instance '{}'...",
                resolved
            );
            let rt = tokio::runtime::Runtime::new().expect("Failed to initialize async runtime");
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
                &resolved,
            ))) {
                Ok(result) => println!("[OK] {}", result),
                Err(e) => {
                    eprintln!("[ERROR] Fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    println!("[*] Triggering fast-forward account rotation...");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    match rt.block_on(auto_switcher::trigger_manual_rotation()) {
        Ok(result) => {
            println!("[OK] Fast-forward completed: {}", result);
            if let Ok(Some(current)) = account::get_current_account() {
                println!("     New Active Profile: {}", current.email);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Fast-forward failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_email(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");
    let sub = args
        .first()
        .filter(|s| !s.starts_with('-'))
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "status".to_string());

    match sub.as_str() {
        "help" | "-h" | "--help" => {
            println!(
                "================================================================================"
            );
            println!("  AGM EMAIL TELEMETRY, VAULT & REMOTE COMMAND CONTROL");
            println!(
                "================================================================================"
            );
            println!("CLI Subcommands:");
            println!(
                "  agm email [status] [--json]               Show email settings & active sender"
            );
            println!("  agm email ls [--json]                     List configured mailboxes & recipients");
            println!("  agm email add <email> <password> [opts]   Add SMTP/IMAP account (sends JSON self-email)");
            println!("  agm email add <email> --recipient         Add notification recipient (sends JSON self-email)");
            println!(
                "  agm email rm <seq|id|email>               Remove email account or recipient"
            );
            println!("  agm email mv <seq|id|email> --default     Promote mailbox account to default sender");
            println!("  agm email export [-f <path>]              Export email config & encrypted secrets to JSON");
            println!(
                "  agm email import [-f <path>]              Import email config bundle from JSON"
            );
            println!();
            println!("Inbound Email Remote Command Subject Syntax:");
            println!("  <VM_NAME_OR_*> | <INSTANCE_SEQ_OR_REPO> | <ACTION>");
            println!("  Examples:");
            println!("    VM1 | 1 | help");
            println!("    VM1 | 1 | ff");
            println!("    VM1 | 1 | agm status");
            println!("    *   | gitmap | prompt Run full test suite");
            println!();
        }
        "status" => {
            let settings = email_vault_db::get_notification_settings().unwrap_or_default();
            let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
            let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
            let default_acc = accounts
                .iter()
                .find(|a| a.is_default)
                .or_else(|| accounts.first());

            if is_json {
                let out = serde_json::json!({
                    "enabled": settings.is_enabled,
                    "local_machine_name": email_watcher::detect_machine_name(),
                    "local_machine_ip": email_watcher::detect_local_ip(),
                    "polling_interval_minutes": settings.polling_interval_minutes,
                    "inbox_check_interval_minutes": settings.inbox_check_interval_minutes,
                    "default_sender": default_acc.map(|a| &a.email),
                    "accounts_count": accounts.len(),
                    "recipients_count": recipients.len(),
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                return;
            }

            println!("[*] AGM Email Telemetry & Notification Status:");
            println!("    Enabled:               {}", settings.is_enabled);
            println!(
                "    Node Identity:         {} ({})",
                email_watcher::detect_machine_name(),
                email_watcher::detect_local_ip()
            );
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
        }
        "ls" | "list" => {
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
                "{:<5} {:<20} {:<30} {:<22} {:<22} {:<8} {}",
                "SEQ", "ID", "EMAIL", "SMTP", "IMAP", "DEFAULT", "ACTIVE"
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
                "{:<5} {:<20} {:<32} {:<14} {}",
                "SEQ", "ID", "EMAIL", "GROUP", "ACTIVE"
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
        "add" => {
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
        "rm" | "remove" | "delete" => {
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
        "mv" | "default" | "set-default" => {
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
        "export" => {
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
        "import" => {
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
        _ => {
            eprintln!("Unknown email subcommand: '{}'. Run 'agm email help'.", sub);
            std::process::exit(1);
        }
    }
}

fn recreate_single_workspace(target_spec: &str) {
    // Resolve target_spec to a concrete folder path
    let resolved_path: PathBuf = {
        let direct = PathBuf::from(target_spec);
        if direct.exists() {
            direct.canonicalize().unwrap_or(direct)
        } else {
            // Check running_projects in repo_db by name or id
            let projects = repo_db::list_running_projects().unwrap_or_default();
            if let Some(found) = projects.into_iter().find(|p| {
                p.repo_name.eq_ignore_ascii_case(target_spec)
                    || p.id.eq_ignore_ascii_case(target_spec)
                    || p.repo_path
                        .to_lowercase()
                        .contains(&target_spec.to_lowercase())
            }) {
                PathBuf::from(found.repo_path)
            } else if let Ok(cwd) = env::current_dir() {
                if let Some(parent) = cwd.parent() {
                    let sibling = parent.join(target_spec);
                    if sibling.exists() {
                        sibling
                    } else {
                        direct
                    }
                } else {
                    direct
                }
            } else {
                direct
            }
        }
    };

    let path_str = resolved_path.to_string_lossy().to_string();
    let clean_str = path_str.trim_start_matches(r"\\?\").to_string();
    let repo_name = resolved_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| target_spec.to_string());

    println!(
        "[*] Recreating project workspace '{}' ({})...",
        repo_name, clean_str
    );

    // 1. Remove .antigravity_resume_task.json if present
    let resume_file = resolved_path.join(".antigravity_resume_task.json");
    if resume_file.exists() {
        let _ = fs::remove_file(&resume_file);
    }

    // 2. Clean matching workspaceStorage folders across instances
    if let Ok(reg) = instance::load_registry() {
        let norm_target = clean_str.to_lowercase().replace('\\', "/");
        for inst in &reg.instances {
            let ws_root = PathBuf::from(&inst.data_dir)
                .join("User")
                .join("workspaceStorage");
            if ws_root.is_dir() {
                if let Ok(entries) = fs::read_dir(&ws_root) {
                    for entry in entries.flatten() {
                        let ws_json = entry.path().join("workspace.json");
                        if ws_json.is_file() {
                            if let Ok(content) = fs::read_to_string(&ws_json) {
                                let content_norm = content
                                    .to_lowercase()
                                    .replace("%3a", ":")
                                    .replace('\\', "/");
                                if content_norm.contains(&norm_target) {
                                    let _ = fs::remove_dir_all(entry.path());
                                    println!(
                                        "    [✓] Purged cached workspaceStorage: {:?}",
                                        entry.file_name()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Remove stale active_prompts and running_projects rows in repo_db
    if let Ok(conn) = repo_db::connect_db() {
        let _ = conn.execute(
            "DELETE FROM active_prompts WHERE LOWER(repo_path) = LOWER(?1) OR LOWER(project_id) LIKE LOWER(?2)",
            rusqlite::params![&clean_str, format!("%{}%", repo_name)],
        );
        let _ = conn.execute(
            "DELETE FROM running_projects WHERE LOWER(repo_path) = LOWER(?1) OR LOWER(repo_name) = LOWER(?2)",
            rusqlite::params![&clean_str, &repo_name],
        );
        println!("    [✓] Cleared tracked prompt/session state in repo_prompts.db");
    }

    // 4. Re-open project in Antigravity IDE (agy)
    if resolved_path.exists() {
        let launched = Command::new("agy")
            .arg(&clean_str)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if launched {
            println!(
                "    [✓] Spawned fresh Antigravity IDE session for '{}'",
                clean_str
            );
        } else if let Ok(exe) =
            antigravity_tools_lib::modules::process::detect_antigravity_with_diagnostics(None)
        {
            let _ = Command::new(exe)
                .arg("--new-window")
                .arg(&clean_str)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            println!(
                "    [✓] Launched Antigravity binary directly for '{}'",
                clean_str
            );
        }
    }

    println!("[SUCCESS] Project '{}' recreated cleanly.", repo_name);
}

fn cmd_recreate_project(args: &[String]) {
    let target = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
    recreate_single_workspace(&target);
}

fn cmd_recreate(args: &[String]) {
    let targets: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();
    if targets.is_empty() {
        cmd_recreate_project(args);
        return;
    }
    for t in targets {
        recreate_single_workspace(&t);
    }
}

fn cmd_test_auto_switch(args: &[String]) {
    println!("[*] Testing Auto-Switcher on this machine...");
    let threshold: f64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(90.0);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let mut app_cfg = match config::load_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let orig_enabled = app_cfg.auto_profile_switcher.is_enabled;
    let orig_low = app_cfg.auto_profile_switcher.low_quota_threshold_percent;
    let orig_crit = app_cfg.auto_profile_switcher.critical_threshold_percent;

    println!(
        "    Target Model: {}",
        app_cfg.auto_profile_switcher.target_model
    );
    println!("    Test Low-Quota Threshold: {:.1}%", threshold);

    // Apply test threshold and enable switcher
    app_cfg.auto_profile_switcher.is_enabled = true;
    app_cfg.auto_profile_switcher.low_quota_threshold_percent = threshold;
    app_cfg.auto_profile_switcher.critical_threshold_percent = threshold.min(15.0);
    let _ = config::save_app_config(&app_cfg);

    let status_before = auto_switcher::get_status();
    println!(
        "    Monitored Instance: {}",
        status_before.active_instance_id
    );
    println!(
        "    Current Bound Account: {}",
        status_before
            .active_account_email
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "    Current Quota: {:.1}%",
        status_before.current_quota_percent.unwrap_or(100.0)
    );

    let interval = auto_switcher::calculate_next_interval_seconds(
        status_before.current_quota_percent,
        &app_cfg.auto_profile_switcher,
    );
    println!("    Calculated Polling Interval: {}s", interval);

    println!("[*] Triggering check_and_rotate_if_needed()...");
    let rotate_res = rt.block_on(auto_switcher::check_and_rotate_if_needed());

    // Restore original config
    app_cfg.auto_profile_switcher.is_enabled = orig_enabled;
    app_cfg.auto_profile_switcher.low_quota_threshold_percent = orig_low;
    app_cfg.auto_profile_switcher.critical_threshold_percent = orig_crit;
    let _ = config::save_app_config(&app_cfg);

    match rotate_res {
        Ok(Some(reason)) => {
            println!("[SUCCESS] Auto-switcher rotated successfully!");
            println!("          Reason: {}", reason);
            if let Ok(Some(current)) = account::get_current_account() {
                println!("          New Active Account: {}", current.email);
            }
        }
        Ok(None) => {
            println!(
                "[INFO] Check cycle complete: No rotation needed (quota was above {:.1}% or candidate optimal).",
                threshold
            );
        }
        Err(e) => {
            eprintln!("[ERROR] Auto-switcher check failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_test_email(args: &[String]) {
    println!("============================================================");
    println!("  AGM INBOUND EMAIL & SMTP DIAGNOSTIC SUITE");
    println!("============================================================");

    // 1. Settings
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ERROR] Failed to load email settings: {}", e);
            return;
        }
    };
    println!("[*] Notification Settings:");
    println!("    Enabled:                    {}", settings.is_enabled);
    println!(
        "    Local Node Name:            {}",
        settings.local_machine_name
    );
    println!(
        "    Polling Interval (min):     {}",
        settings.polling_interval_minutes
    );
    println!(
        "    Inbox Check Interval (min): {}",
        settings.inbox_check_interval_minutes
    );

    // 2. Recipients
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    println!(
        "[*] Authorized Notifier Recipients ({} found):",
        recipients.len()
    );
    for r in &recipients {
        println!("    - {} (active: {})", r.email, r.is_active);
    }

    // 3. Accounts
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    println!(
        "[*] Configured Mailbox Accounts ({} found):",
        accounts.len()
    );
    for a in &accounts {
        println!(
            "    - [{}] {} | IMAP: {}:{} | SMTP: {}:{} | default: {} | active: {}",
            a.id,
            a.email,
            a.imap_host,
            a.imap_port,
            a.smtp_host,
            a.smtp_port,
            a.is_default,
            a.is_active
        );
    }

    let default_acc = match accounts.into_iter().find(|a| a.is_default && a.is_active) {
        Some(a) => a,
        None => {
            eprintln!("[ERROR] No active default email account found in vault!");
            return;
        }
    };

    // 4. Test IMAP poll
    println!(
        "[*] Connecting to IMAP server '{}:{}' for '{}'...",
        default_acc.imap_host, default_acc.imap_port, default_acc.email
    );
    match email_inbound::poll_unread_messages(&default_acc, 5) {
        Ok(msgs) => {
            println!("[SUCCESS] IMAP connection and authentication succeeded!");
            println!("          Found {} unread message(s):", msgs.len());
            for (i, m) in msgs.iter().enumerate() {
                println!("          [{}] From:    {}", i + 1, m.from);
                println!("              Subject: {}", m.subject);
                println!("              Msg-ID:  {}", m.message_id);
                let parsed = email_inbound::parse_email_command(&m.subject, &m.body);
                println!("              Parsed:  {:?}", parsed);
                let is_auth = email_inbound::is_authorized_notifier(&m.from);
                println!("              Authorized Sender: {}", is_auth);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] IMAP poll failed: {}", e);
        }
    }

    // 5. Test Subject Parser with various inputs
    println!("[*] Testing Subject Command Parser:");
    let test_subjects = vec![
        "VM3 | 1 | help",
        "VM3 | help",
        "VM3 | default | help",
        "VM3 | #1 | help",
        "VM3 | ins-1 | help",
        "VM3 | 1 | cmd",
        "VM3 | 1 | agm status",
        "* | gitmap | status",
    ];
    for subj in test_subjects {
        let action = email_inbound::parse_email_command(subj, "");
        println!("    '{}' => {:?}", subj, action);
    }

    // 6. Optional SMTP test if requested: agm test-email send [recipient]
    if args.first().map(|s| s.as_str()) == Some("send") {
        let target_rcpt = args
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or("alim.karim@riseup-asia.com");
        println!("[*] Sending test SMTP dispatch to '{}'...", target_rcpt);
        let (subj, body) = email_sender::render_help_email(
            &settings.local_machine_name,
            &email_watcher::detect_local_ip(),
        );
        match email_sender::dispatch_email_with_failover(&subj, &body, &[target_rcpt.to_string()]) {
            Ok(res) => {
                println!(
                    "[SUCCESS] SMTP test email delivered successfully via '{}'!",
                    res.used_account_email
                );
            }
            Err(e) => {
                eprintln!("[ERROR] SMTP test email failed: {}", e);
            }
        }
    }

    // 7. Optional end-to-end command execution test: agm test-email execute [subject]
    if args.first().map(|s| s.as_str()) == Some("execute") {
        let test_subject = args.get(1).map(|s| s.as_str()).unwrap_or("VM3 | 1 | help");
        let test_from = args
            .get(2)
            .map(|s| s.as_str())
            .unwrap_or("Alim Ul Karim <alim.karim@riseup-asia.com>");
        let test_body = args.get(3).map(|s| s.as_str()).unwrap_or("");
        println!("[*] Executing live end-to-end simulated inbound message:");
        println!("    From:    {}", test_from);
        println!("    Subject: {}", test_subject);
        if !test_body.is_empty() {
            println!("    Body:    {}", test_body);
        }
        let mock_msg = email_inbound::RawEmailMessage {
            message_id: format!("<test-{}@agm>", uuid::Uuid::new_v4()),
            from: test_from.to_string(),
            subject: test_subject.to_string(),
            body: test_body.to_string(),
        };
        let action = email_inbound::parse_email_command(&mock_msg.subject, &mock_msg.body);
        println!("    Parsed Action: {:?}", action);
        let local_ip = email_watcher::detect_local_ip();
        let local_name = email_watcher::detect_machine_name();
        match email_inbound::execute_inbound_action(&mock_msg, action, &local_ip, &local_name) {
            Ok(summary) => {
                println!("[SUCCESS] Inbound action executed and receipts dispatched!");
                println!("          Summary: {}", summary);
            }
            Err(e) => {
                eprintln!("[ERROR] Inbound execution failed: {}", e);
            }
        }
    }

    // 8. Optional process live unread messages: agm test-email poll
    if args.first().map(|s| s.as_str()) == Some("poll") {
        println!("[*] Polling and executing real unread messages from IMAP...");
        match email_inbound::poll_unread_messages(&default_acc, 5) {
            Ok(msgs) => {
                println!("    Found {} unread message(s)", msgs.len());
                let local_ip = email_watcher::detect_local_ip();
                let local_name = email_watcher::detect_machine_name();
                for (i, m) in msgs.iter().enumerate() {
                    println!("    [{}] Message-ID: {}", i + 1, m.message_id);
                    println!("        From: {}", m.from);
                    println!("        Subject: {}", m.subject);
                    let action = email_inbound::parse_email_command(&m.subject, &m.body);
                    println!("        Action: {:?}", action);
                    match email_inbound::execute_inbound_action(m, action, &local_ip, &local_name) {
                        Ok(res) => println!("        [SUCCESS] {}", res),
                        Err(e) => eprintln!("        [ERROR] {}", e),
                    }
                }
            }
            Err(e) => eprintln!("[ERROR] Polling failed: {}", e),
        }
    }
}

fn cmd_install() {
    println!("[*] Installing AGM CLI into system PATH...");

    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to locate current executable: {}", e);
            std::process::exit(1);
        }
    };

    #[cfg(target_os = "windows")]
    {
        let local_app_data = match env::var("LOCALAPPDATA") {
            Ok(v) => PathBuf::from(v),
            Err(_) => {
                let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
                PathBuf::from(user_profile).join("AppData").join("Local")
            }
        };

        let target_dir = local_app_data.join("agm-cli");
        if let Err(e) = fs::create_dir_all(&target_dir) {
            eprintln!("[ERROR] Failed to create directory {:?}: {}", target_dir, e);
            std::process::exit(1);
        }

        let target_exe = target_dir.join("agm.exe");
        if let Err(e) = fs::copy(&current_exe, &target_exe) {
            eprintln!("[ERROR] Failed to copy binary to {:?}: {}", target_exe, e);
            std::process::exit(1);
        }
        println!("  [OK] Binary copied to {:?}", target_exe);

        // Create agm.cmd helper wrapper
        let cmd_wrapper = target_dir.join("agm.cmd");
        let cmd_content = "@echo off\r\n\"%~dp0agm.exe\" %*\r\n";
        let _ = fs::write(&cmd_wrapper, cmd_content);

        // Add to User PATH via registry if missing
        let target_dir_str = target_dir.to_string_lossy().to_string();
        let path_script = format!(
            "$dir = '{}'; \
             $old = [Environment]::GetEnvironmentVariable('Path', 'User'); \
             if ($old -notlike \"*$dir*\") {{ \
                 [Environment]::SetEnvironmentVariable('Path', \"$old;$dir\", 'User'); \
                 Write-Host 'PATH updated'; \
             }} else {{ Write-Host 'Already in PATH'; }}",
            target_dir_str.replace('\'', "''")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &path_script])
            .output();

        // Register function in PowerShell profile
        register_powershell_profile_function(&target_exe);

        println!("[SUCCESS] AGM CLI installed successfully!");
        println!("          You can now run 'agm' from any Command Prompt or PowerShell window.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let target_dir = PathBuf::from(home).join(".local").join("bin");
        let _ = fs::create_dir_all(&target_dir);
        let target_exe = target_dir.join("agm");
        if let Err(e) = fs::copy(&current_exe, &target_exe) {
            eprintln!("[ERROR] Failed to copy binary to {:?}: {}", target_exe, e);
            std::process::exit(1);
        }
        println!("[SUCCESS] AGM CLI installed to {:?}", target_exe);
    }
}

#[cfg(target_os = "windows")]
fn register_powershell_profile_function(exe_path: &Path) {
    let script = format!(
        "$profilePath = $PROFILE; \
         if ($profilePath -and (Test-Path -Path $profilePath)) {{ \
             $content = Get-Content -LiteralPath $profilePath -Raw; \
             if ($content -notmatch 'function agm\\b') {{ \
                 $entry = \"`n# agm command wrapper`nfunction agm {{ & '{exe}' @args }}`n\"; \
                 Add-Content -LiteralPath $profilePath -Value $entry; \
             }} \
         }}",
        exe = exe_path.to_string_lossy().replace('\'', "''")
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

fn cmd_update() {
    println!("[*] Checking GitHub for AGM updates...");
    let client = match reqwest::blocking::Client::builder()
        .user_agent("AGM-CLI-Updater")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize HTTP client: {}", e);
            return;
        }
    };

    let url = "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";
    let resp = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to connect to GitHub releases API: {}", e);
            return;
        }
    };

    if !resp.status().is_success() {
        eprintln!("GitHub API returned HTTP status: {}", resp.status());
        return;
    }

    let json: serde_json::Value = match resp.json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse release response: {}", e);
            return;
        }
    };

    let tag_name = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');
    println!("    Current Version: v{}", VERSION);
    println!("    Latest Release:  v{}", tag_name);

    if tag_name == VERSION {
        println!("[OK] AGM is already at the latest release (v{}).", VERSION);
        return;
    }

    println!(
        "[*] A new version is available: v{} -> v{}",
        VERSION, tag_name
    );
    println!("[*] Triggering automatic update installation...");

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = "irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex";
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                ps_cmd,
            ])
            .status();
        match status {
            Ok(s) => {
                let code = s.code().unwrap_or(1);
                if code == 0 {
                    println!("[OK] Update completed successfully!");
                } else {
                    eprintln!("[ERROR] Update script exited with code {}", code);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to run update script: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let sh_cmd = "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash";
        let _ = Command::new("sh").args(["-c", sh_cmd]).status();
    }
}

fn cmd_ssh(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: agm ssh <[user@]host> [-p port] [--password <pwd>] [--update] [cmd...]");
        std::process::exit(1);
    }

    let mut target = String::new();
    let mut port = "22".to_string();
    let mut password: Option<String> = None;
    let mut is_update = false;
    let mut remote_cmd: Vec<String> = Vec::new();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "-p" || arg == "--port" {
            if idx + 1 < args.len() {
                port = args[idx + 1].clone();
                idx += 2;
                continue;
            }
        } else if arg == "--password" {
            if idx + 1 < args.len() {
                password = Some(args[idx + 1].clone());
                idx += 2;
                continue;
            }
        } else if arg == "--update" {
            is_update = true;
            idx += 1;
            continue;
        } else if target.is_empty() && !arg.starts_with('-') {
            target = arg.clone();
            idx += 1;
            continue;
        } else {
            remote_cmd.push(arg.clone());
            idx += 1;
        }
    }

    if target.is_empty() {
        eprintln!("[ERROR] Missing SSH host target.");
        std::process::exit(1);
    }

    println!(
        "[*] Connecting to SSH target '{}' (port {})...",
        target, port
    );

    let final_cmd = if is_update {
        println!("[*] Auto-update mode active: will execute AGM update on remote VM.");
        "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh 2>/dev/null | bash || powershell -Command \"irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex\"".to_string()
    } else if !remote_cmd.is_empty() {
        remote_cmd.join(" ")
    } else {
        String::new()
    };

    let effective_password = if password.is_some() {
        password
    } else {
        let test_res = Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=4",
                "-p",
                &port,
                &target,
                "exit",
            ])
            .output();

        let needs_password = match test_res {
            Ok(out) => out.status.code().unwrap_or(1) != 0,
            Err(_) => true,
        };

        if needs_password {
            print!("Enter SSH password for '{}': ", target);
            let _ = io::stdout().flush();
            let pwd = read_password_masked();
            Some(pwd)
        } else {
            None
        }
    };

    let mut ssh = Command::new("ssh");
    ssh.arg("-p").arg(&port);

    if let Some(ref pwd) = effective_password {
        #[cfg(target_os = "windows")]
        {
            env::set_var("SSH_PASSWORD", pwd);
        }
        #[cfg(not(target_os = "windows"))]
        {
            if Command::new("sshpass").arg("-V").output().is_ok() {
                let mut pass_cmd = Command::new("sshpass");
                pass_cmd
                    .arg("-p")
                    .arg(pwd)
                    .arg("ssh")
                    .arg("-p")
                    .arg(&port)
                    .arg(&target);
                if !final_cmd.is_empty() {
                    pass_cmd.arg(&final_cmd);
                }
                let _ = pass_cmd.status();
                return;
            }
        }
    }

    ssh.arg(&target);
    if !final_cmd.is_empty() {
        ssh.arg(&final_cmd);
    }

    ssh.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match ssh.status() {
        Ok(s) => {
            let code = s.code().unwrap_or(0);
            if code != 0 {
                eprintln!("[*] SSH session exited with code: {}", code);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to execute 'ssh': {}", e);
            eprintln!("Ensure OpenSSH client is installed and accessible in your system PATH.");
        }
    }
}

fn read_password_masked() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "$p = Read-Host -Prompt '' -AsSecureString; \
                 [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p))",
            ])
            .output();

        if let Ok(out) = output {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return res;
        }
    }

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

fn cmd_test_training(_args: &[String]) {
    println!("============================================================");
    println!("  AGM MACHINE TRAINING & TELEMETRY REST API TEST SUITE");
    println!("============================================================");

    // 1. Telemetry Gathering
    println!("[*] Gathering machine telemetry via training_api::gather_telemetry()...");
    match training_api::gather_telemetry() {
        Ok(t) => {
            println!("[SUCCESS] Telemetry retrieved:");
            println!("          Node Name:           {}", t.node_name);
            println!("          Local IP:            {}", t.local_ip);
            println!("          CLI/Lib Version:     v{}", t.version);
            println!("          Platform OS/Arch:    {}/{}", t.os, t.arch);
            println!("          API Enabled:         {}", t.training_api_enabled);
            println!(
                "          Active Prompts:      {} running ({} total)",
                t.active_prompts_running, t.active_prompts_total
            );
            if let Some(acc) = t.active_account {
                println!(
                    "          Active Account:      {} ({:.1}% quota, {})",
                    acc.email, acc.quota_percent, acc.tier
                );
            }
            println!("          Accounts Summary:    {}", t.accounts_summary);
            println!("          Instances Monitored: {}", t.instances.len());
        }
        Err(e) => {
            eprintln!("[ERROR] Telemetry gathering failed: {}", e);
            std::process::exit(1);
        }
    }

    // 2. Training Learning Feedback Ingestion
    println!("[*] Ingesting test learning signal into SQLite training_vault.db...");
    let req = training_api::LearnRequest {
        session_id: Some("agm-cli-test-session".to_string()),
        model: Some("gemini-flash".to_string()),
        prompt_type: Some("e2e-verification".to_string()),
        input_tokens: Some(256),
        output_tokens: Some(512),
        latency_ms: Some(180),
        success: Some(true),
        score: Some(0.99),
        feedback: Some("Live machine training test successfully ingested".to_string()),
        adjust_routing: Some(false),
    };
    match training_api::ingest_learning(req) {
        Ok(res) => {
            println!("[SUCCESS] Learning feedback ingested:");
            println!("          Log ID:    {}", res.log_id);
            println!("          Timestamp: {}", res.timestamp);
            println!("          Message:   {}", res.message);
        }
        Err(e) => {
            eprintln!("[ERROR] Learning feedback ingestion failed: {}", e);
            std::process::exit(1);
        }
    }

    // 3. Machine Remote Modification
    println!("[*] Testing machine remote modification (adjust_threshold action)...");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mod_req = training_api::MachineModifyRequest {
        action: "adjust_threshold".to_string(),
        account_email_or_id: None,
        instance_id: None,
        target_model: None,
        low_quota_threshold: Some(85.0),
        critical_quota_threshold: Some(12.0),
    };
    match rt.block_on(training_api::execute_machine_modify(mod_req)) {
        Ok(res) => {
            println!("[SUCCESS] Machine modification completed:");
            println!("          Action:  {}", res.action);
            println!("          Message: {}", res.message);
        }
        Err(e) => {
            eprintln!("[ERROR] Machine modification failed: {}", e);
            std::process::exit(1);
        }
    }

    // 4. Settings Toggle Test
    println!("[*] Verifying training_api_enabled settings toggle...");
    let orig = training_api::is_training_api_enabled();
    let _ = training_api::set_training_api_enabled(!orig);
    assert_eq!(training_api::is_training_api_enabled(), !orig);
    let _ = training_api::set_training_api_enabled(orig);
    println!(
        "[SUCCESS] Settings toggle successfully verified (state restored to {}).",
        orig
    );

    println!("============================================================");
    println!("[SUCCESS] All Training REST API engine tests passed!");
    println!("============================================================");
}
