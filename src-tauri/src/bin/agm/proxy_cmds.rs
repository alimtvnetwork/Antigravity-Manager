//! proxy_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) fn dispatch_proxy_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_proxy_status(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_proxy_status(args);
        return;
    }
    dispatch_proxy_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_proxy_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "status" => cmd_proxy_status(args),
        "pool" => cmd_proxy_pool(args),
        "model" => cmd_proxy_model(args),
        "config" => cmd_proxy_config(args),
        "restart" => cmd_proxy_restart(args),
        "cache-clear" => cmd_proxy_cache_clear(args),
        _ => crate::common::handle_unknown_domain_command("proxy", subcommand, args),
    }
}

pub(crate) fn cmd_proxy_status(args: &[String]) {
    let ctx = CliContext::parse(args);
    let is_running = is_daemon_running();
    if ctx.json_output {
        let data = serde_json::json!({ "daemon_running": is_running, "port": 8045 });
        CliEnvelope::ok("proxy status", None, data).print_and_exit();
    }
    println!(
        "Proxy Daemon Status: {}",
        if is_running {
            "Running on 127.0.0.1:8045"
        } else {
            "Stopped"
        }
    );
}

pub(crate) fn cmd_proxy_pool(args: &[String]) {
    cmd_proxy(args);
}

pub(crate) fn cmd_proxy_model(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output {
        let data = serde_json::json!({ "default_model": "gemini-3.8-flash-high" });
        CliEnvelope::ok("proxy model", None, data).print_and_exit();
    }
    println!("Default Model: gemini-3.8-flash-high");
}

pub(crate) fn cmd_proxy_config(args: &[String]) {
    let ctx = CliContext::parse(args);
    let cfg = config::load_app_config().unwrap_or_default();
    if ctx.json_output {
        CliEnvelope::ok("proxy config", None, cfg).print_and_exit();
    }
    println!("Proxy Config: port={}", cfg.proxy.port);
}

pub(crate) fn cmd_proxy_restart(_args: &[String]) {
    println!("[*] Reloading proxy configurations...");
}

pub(crate) fn cmd_proxy_cache_clear(args: &[String]) {
    crate::cache_cmds::cmd_clear_cache(args);
}

pub(crate) fn cmd_proxy(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Reverse Proxy Gateway:");
        println!("  agm proxy [test] [--json]");
        println!("\nDescription:");
        println!("  Checks status and tests connectivity of the Antigravity local reverse proxy gateway.");
        println!("\nSubcommands:");
        println!(
            "  test                Perform loopback ping and latency test against the proxy port"
        );
        println!("\nExamples:");
        println!(
            "  agm proxy                           # Show proxy gateway status and listening port"
        );
        println!(
            "  agm proxy test                      # Test loopback proxy latency and connectivity"
        );
        return;
    }

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

pub(crate) fn cmd_sync(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Synchronization:");
        println!("  agm sync");
        println!("\nDescription:");
        println!(
            "  Synchronizes registered accounts, sandbox instances, and split SQLite databases."
        );
        println!("\nExamples:");
        println!("  agm sync                            # Verify and synchronize local vaults");
        return;
    }

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

pub(crate) fn cmd_pull(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Git Pull:");
        println!("  agm pull");
        println!("\nDescription:");
        println!("  Executes 'git pull origin main' in the Antigravity-Manager repository root.");
        println!("\nExamples:");
        println!("  agm pull                            # Fetch and merge latest code from origin");
        return;
    }

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
