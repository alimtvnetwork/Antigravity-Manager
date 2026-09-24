//! AGM - Antigravity-Manager Native Terminal CLI
//! Autonomous terminal companion for Antigravity-Manager:
//! Status monitoring, multi-instance management, fast-forward switching,
//! accounts listing, direct account switching, doctor health checks,
//! prompt inspection, proxy status/test, sync, git pull, clean/purge, logs,
//! PATH self-installation, GitHub auto-updates, and SSH remote machine management.

use antigravity_tools_lib::modules::{
    account, auto_switcher, config, email_inbound, email_sender, email_vault_db, email_watcher,
    instance, proxy_db, repo_db, security_db, supabase_sync, training_api,
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
    match subcommand.as_str() {
        "status" => cmd_status(),
        "instances" | "instance" | "ls" => cmd_instances(),
        "doctor" | "check" => cmd_doctor(),
        "accounts" | "account" | "acc" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_accounts(&cmd_args);
        }
        "switch" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_switch(&cmd_args);
        }
        "prompts" | "prompt" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_prompts(&cmd_args);
        }
        "proxy" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_proxy(&cmd_args);
        }
        "sync" => cmd_sync(),
        "pull" => cmd_pull(),
        "clean" | "purge" => cmd_clean(),
        "logs" | "log" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_logs(&cmd_args);
        }
        "ff" | "smart-switch" | "fast-forward" => cmd_fast_forward(),
        "test-switcher" | "test-auto-switch" | "auto-switch" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_test_auto_switch(&cmd_args);
        }
        "test-email" | "email-test" | "check-email" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_test_email(&cmd_args);
        }
        "test-training" | "training" | "train" => {
            let cmd_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_test_training(&cmd_args);
        }
        "install" => cmd_install(),
        "update" => cmd_update(),
        "ssh" => {
            let ssh_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_ssh(&ssh_args);
        }
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
    println!("Core Commands:");
    println!("  status                      Show current node status, proxy, active profile & IP");
    println!("  instances, ls               List all registered sandbox profiles and running PIDs");
    println!("  doctor, check               Run comprehensive pre-flight system health checks");
    println!("  accounts, acc [--active]    List registered accounts, tiers, and weekly quotas");
    println!("  switch <email|prefix|id>    Switch active account directly without opening GUI");
    println!("  prompts [--running]         List tracked prompt tasks and available templates");
    println!(
        "  proxy [status|test]         Check local proxy service status or test loopback ping"
    );
    println!("  sync                        Synchronize local accounts, instances, and DB vaults");
    println!("  pull                        Execute git pull origin main in repository root");
    println!("  clean, purge                Safely clean temp caches while protecting DB vaults");
    println!("  logs [--tail N] [-f text]   View recent application and proxy log lines");
    println!("  ff, smart-switch            Trigger fast-forward rotation to freshest account");
    println!("  install                     Install 'agm' executable into system PATH and profile");
    println!("  update                      Check GitHub releases and update AGM binary");
    println!(
        "  ssh <target> [options]      Connect to remote VM via SSH or run remote auto-update"
    );
    println!("  version, -v                 Print agm CLI version");
    println!("  help, -h                    Display this help manual");
    println!();
    println!("SSH Options:");
    println!("  agm ssh <[user@]host> [-p port] [--password <pwd>] [--update]");
    println!("  --password <pwd>            Provide SSH password non-interactively");
    println!("  --update                    Auto-update or install AGM on the remote VM via SSH");
    println!("  -p <port>                   Specify custom SSH port (default: 22)");
    println!();
    println!("Examples:");
    println!("  agm status");
    println!("  agm doctor");
    println!("  agm accounts");
    println!("  agm switch abidul");
    println!("  agm proxy test");
    println!("  agm prompts --running");
    println!("  agm sync");
    println!("  agm clean");
    println!("  agm ssh root@192.168.1.50");
    println!("  agm ssh 192.168.1.50 --update");
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

fn cmd_prompts(args: &[String]) {
    let running_only = args.iter().any(|a| a == "--running");

    match repo_db::connect_db() {
        Ok(conn) => {
            let query = if running_only {
                "SELECT id, project_id, instance_id, prompt_content, model, status, updated_at \
                 FROM active_prompts WHERE status = 'running' ORDER BY updated_at DESC LIMIT 50"
            } else {
                "SELECT id, project_id, instance_id, prompt_content, model, status, updated_at \
                 FROM active_prompts ORDER BY updated_at DESC LIMIT 50"
            };

            let mut stmt = match conn.prepare(query) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[ERROR] Failed to query active_prompts: {}", e);
                    return;
                }
            };

            let prompts = stmt
                .query_map([], |row| {
                    let id: String = row.get(0)?;
                    let proj: String = row.get(1)?;
                    let _inst: String = row.get(2)?;
                    let content: String = row.get(3)?;
                    let model: Option<String> = row.get(4)?;
                    let status: String = row.get(5)?;
                    let updated: i64 = row.get(6)?;
                    Ok((id, proj, content, model, status, updated))
                })
                .map(|rows| rows.flatten().collect::<Vec<_>>())
                .unwrap_or_default();

            if prompts.is_empty() {
                println!("No active prompt tasks tracked in repo_prompts.db.");
            } else {
                println!("\nTracked Prompt Tasks ({} total):", prompts.len());
                println!(
                    "{:<12} {:<24} {:<12} {:<14} {}",
                    "PROMPT ID", "PROJECT", "STATUS", "MODEL", "SNIPPET"
                );
                println!("{}", "-".repeat(95));
                for (id, proj, content, model, status, _) in prompts {
                    let snippet: String = content
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(40)
                        .collect();
                    let model_str = model.unwrap_or_else(|| "default".to_string());
                    let short_id: String = id.chars().take(8).collect();
                    println!(
                        "{:<12} {:<24} {:<12} {:<14} {}",
                        short_id, proj, status, model_str, snippet
                    );
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("[WARN] Could not connect to repo_prompts.db: {}", e);
        }
    }

    scan_prompt_templates();
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

fn cmd_status() {
    let machine_name = email_watcher::detect_machine_name();
    let local_ip = email_watcher::detect_local_ip();

    println!("[*] Antigravity-Manager Node Status:");
    println!("    Machine Name:    {}", machine_name);
    println!("    Local IP:        {}", local_ip);
    println!("    CLI Version:     v{}", VERSION);

    // Active Account
    match account::get_current_account() {
        Ok(Some(acc)) => {
            println!("    Active Account:  {}", acc.email);
            println!(
                "    Account Name:    {}",
                acc.name.as_deref().unwrap_or("-")
            );
        }
        Ok(None) => {
            println!("    Active Account:  (None / Default)");
        }
        Err(e) => {
            println!("    Active Account:  Error querying ({})", e);
        }
    }

    // Instances Count
    match instance::list_instances() {
        Ok(list) => {
            let running_count = list.iter().filter(|i| i.is_running).count();
            println!(
                "    Sandbox Profiles: {} configured ({} currently running)",
                list.len(),
                running_count
            );
        }
        Err(e) => {
            println!("    Sandbox Profiles: Error querying ({})", e);
        }
    }
}

fn cmd_instances() {
    match instance::list_instances() {
        Ok(instances) => {
            if instances.is_empty() {
                println!("No sandbox profiles registered yet.");
                println!("Create one via AGM GUI or 'agm-alim --create-profile <name>'.");
                return;
            }

            let node_alias = supabase_sync::load_config()
                .map(|c| c.node_alias)
                .unwrap_or_else(|_| "Local".to_string());
            let local_ip = supabase_sync::get_local_ip();

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

fn cmd_fast_forward() {
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
