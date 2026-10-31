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
        "paths" | "dirs" | "path" => cmd_system_paths(args),
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

pub(crate) fn cmd_system_paths(args: &[String]) {
    let ctx = CliContext::parse(args);
    let data_dir = account::get_data_dir().unwrap_or_default();
    let current_exe = env::current_exe().unwrap_or_default();
    let current_dir = env::current_dir().unwrap_or_default();
    let repo_db = data_dir.join("repo_prompts.db");
    let backup_db = data_dir.join("backup-prompts").join("backup-prompts.db");
    let error_db = data_dir.join("errors.db");
    let logs_dir = data_dir.join("logs");
    let instances_dir = data_dir.join("instances");
    let config_file = data_dir.join("gui_config.json");

    if ctx.json_output {
        let data = serde_json::json!({
            "data_dir": data_dir.to_string_lossy(),
            "current_exe": current_exe.to_string_lossy(),
            "working_dir": current_dir.to_string_lossy(),
            "repo_db": repo_db.to_string_lossy(),
            "backup_db": backup_db.to_string_lossy(),
            "error_db": error_db.to_string_lossy(),
            "logs_dir": logs_dir.to_string_lossy(),
            "instances_dir": instances_dir.to_string_lossy(),
            "config_file": config_file.to_string_lossy(),
        });
        CliEnvelope::ok("system paths", None, data).print_and_exit();
    }

    println!("AGM System & Storage Paths:");
    println!("  Data Directory:    {}", data_dir.display());
    println!("  Repo DB:           {}", repo_db.display());
    println!("  Backup DB:         {}", backup_db.display());
    println!("  Error DB:          {}", error_db.display());
    println!("  Logs Directory:    {}", logs_dir.display());
    println!("  Instances Dir:     {}", instances_dir.display());
    println!("  Config File:       {}", config_file.display());
    println!("  Binary Path:       {}", current_exe.display());
    println!("  Working Dir:       {}", current_dir.display());
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
