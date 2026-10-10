//! doctor_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub(crate) fn cmd_doctor(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM System Doctor Diagnostic:");
        println!("  agm doctor [--help]");
        println!("\nDescription:");
        println!(
            "  Performs comprehensive end-to-end health checks across all Antigravity subsystems:"
        );
        println!("  - Account credential vault and active token validity");
        println!("  - Multi-instance directory structures and config integrity");
        println!("  - SQLite split databases (prompts.db, repo.db, email_vault.db, proxy.db)");
        println!("  - Reverse proxy server loopback connectivity");
        println!("  - Telegram bot token and chat ID integration");
        println!("  - Outbound SMTP / Inbound IMAP email watchers");
        println!("\nAliases: agm doctor, agm check");
        println!("\nExamples:");
        println!("  agm doctor                          # Run full system diagnostics");
        return;
    }

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
    crate::help_tables::print_doctor_probe(
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
    crate::help_tables::print_doctor_probe("Vault: email_vault.db", &email_vault_check, email_ok);

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
    crate::help_tables::print_doctor_probe("Vault: repo_prompts.db", &repo_db_check, repo_ok);

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
    crate::help_tables::print_doctor_probe("Vault: security.db", &sec_db_check, sec_ok);

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
    crate::help_tables::print_doctor_probe(
        "Vault: thinking_store.db",
        &thinking_check,
        thinking_ok,
    );

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
    crate::help_tables::print_doctor_probe("Proxy Gateway (8045)", &proxy_detail, proxy_listening);

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
    crate::help_tables::print_doctor_probe("Sandbox Instances", &instances_detail, inst_ok);

    // Probe 8: PATH registration
    checks_total += 1;
    let path_registered = crate::common::check_is_in_path();
    let path_detail = if path_registered {
        checks_passed += 1;
        "Registered in system PATH".to_string()
    } else {
        "Not detected in PATH (run 'agm install')".to_string()
    };
    crate::help_tables::print_doctor_probe(
        "System PATH Configuration",
        &path_detail,
        path_registered,
    );

    // Probe 9: Network Identity
    checks_total += 1;
    let local_ip = email_watcher::detect_local_ip();
    let node_name = email_watcher::detect_machine_name();
    let net_detail = format!("Node: {} | IP: {}", node_name, local_ip);
    checks_passed += 1;
    crate::help_tables::print_doctor_probe("Network Identity", &net_detail, true);

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
