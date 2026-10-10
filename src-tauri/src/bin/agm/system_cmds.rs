//! system_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

pub(crate) fn dispatch_system_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_system_doctor(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_system_doctor(args);
        return;
    }
    dispatch_system_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_system_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "version" => cmd_system_version(args),
        "env" => cmd_system_env(args),
        "doctor" => cmd_system_doctor(args),
        "db-stats" => cmd_system_db_stats(args),
        "vacuum" => cmd_system_vacuum(args),
        _ => crate::common::handle_unknown_domain_command("system", subcommand, args),
    }
}

pub(crate) fn cmd_system_version(args: &[String]) {
    let ctx = CliContext::parse(args);
    let git_hash = antigravity_tools_lib::modules::git_info::get_git_hash();
    let git_branch = antigravity_tools_lib::modules::git_info::get_git_branch();
    let last_release = antigravity_tools_lib::modules::git_info::get_last_release();
    if ctx.json_output {
        let data = serde_json::json!({
            "version": crate::common::VERSION,
            "commit": git_hash,
            "branch": git_branch,
            "last_release": last_release
        });
        CliEnvelope::ok("system version", None, data).print_and_exit();
    }
    println!(
        "agm v{} (commit: {}, branch: {}, last release: {})",
        crate::common::VERSION,
        git_hash,
        git_branch,
        last_release
    );
}

pub(crate) fn cmd_system_env(args: &[String]) {
    let ctx = CliContext::parse(args);
    let (node, ip) = email_sender::get_local_node_identity();
    let data = serde_json::json!({
        "node_alias": node,
        "local_ip": ip,
        "os": env::consts::OS,
        "arch": env::consts::ARCH,
        "daemon_running": is_daemon_running(),
    });
    if ctx.json_output {
        CliEnvelope::ok("system env", None, data).print_and_exit();
    }
    println!(
        "System Environment:\n  OS: {}\n  Arch: {}\n  Node: {}\n  IP: {}",
        env::consts::OS,
        env::consts::ARCH,
        node,
        ip
    );
}

pub(crate) fn cmd_system_doctor(args: &[String]) {
    crate::doctor_cmds::cmd_doctor(args);
}

pub(crate) fn cmd_system_db_stats(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output && is_daemon_running() {
        if let Ok(resp) = forward_to_local_rest::<serde_json::Value>(
            reqwest::Method::GET,
            "/system/db-stats",
            None,
        ) {
            if let Some(data) = resp.get("data") {
                CliEnvelope::ok("system db-stats", None, data.clone()).print_and_exit();
            }
        }
    }
    let data_dir = account::get_data_dir().unwrap_or_default();
    let repo_db_p = data_dir.join("repo_prompts.db");
    let security_db_p = data_dir.join("security.db");
    let accounts_db_p = data_dir.join("account.db");
    let repo_size = fs::metadata(&repo_db_p).map(|m| m.len()).unwrap_or(0);
    let security_size = fs::metadata(&security_db_p).map(|m| m.len()).unwrap_or(0);
    let accounts_size = fs::metadata(&accounts_db_p).map(|m| m.len()).unwrap_or(0);
    let data = serde_json::json!({
        "repo_prompts_bytes": repo_size,
        "security_db_bytes": security_size,
        "accounts_db_bytes": accounts_size,
        "total_bytes": repo_size + security_size + accounts_size
    });
    if ctx.json_output {
        CliEnvelope::ok("system db-stats", None, data).print_and_exit();
    }
    println!(
        "Database Stats:\n  repo_prompts.db: {} bytes\n  security.db: {} bytes\n  account.db: {} bytes",
        repo_size, security_size, accounts_size
    );
}

pub(crate) fn cmd_system_vacuum(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output && is_daemon_running() {
        if let Ok(resp) = forward_to_local_rest::<serde_json::Value>(
            reqwest::Method::POST,
            "/system/vacuum",
            None,
        ) {
            if let Some(data) = resp.get("data") {
                CliEnvelope::ok("system vacuum", None, data.clone()).print_and_exit();
            }
        }
    }
    let mut count = 0;
    if let Ok(conn) = repo_db::connect_db() {
        if conn.execute("VACUUM", []).is_ok() {
            count += 1;
        }
    }
    let data = serde_json::json!({ "vacuumed_databases": count });
    if ctx.json_output {
        CliEnvelope::ok("system vacuum", None, data).print_and_exit();
    }
    println!("[SUCCESS] Vacuumed {} database(s).", count);
}

pub(crate) fn cmd_clear_terminal(_args: &[String]) {
    print!("\x1B[2J\x1B[1;1H\x1B[3J");
    let _ = std::io::Write::flush(&mut std::io::stdout());

    println!("✔ Terminal screen cleared and session refreshed.");
    println!("\n  💡 Optimization & Next Step Suggestions:");
    println!("    • Inspect JSON Formats:     agm which-format .");
    println!("    • Inspect Failed Commands:  agm failed-commands");
    println!("    • SSH Fleet Status:         agm ssh nodes");
    println!("    • Sync Cluster SSH Keys:    agm ssh deploy-keys");
    println!("    • Clean Artifacts & Caches: agm clean\n");
}
