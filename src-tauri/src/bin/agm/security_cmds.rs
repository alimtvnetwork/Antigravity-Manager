//! security_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn dispatch_security_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_security_ip_list(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_security_ip_list(args);
        return;
    }
    dispatch_security_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_security_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "ip-list" => cmd_security_ip_list(args),
        "ip-block" => cmd_security_ip_block(args),
        "ip-unblock" => cmd_security_ip_unblock(args),
        "audit" => cmd_security_audit(args),
        _ => crate::common::handle_unknown_domain_command("security", subcommand, args),
    }
}

pub(crate) fn cmd_security_ip_list(args: &[String]) {
    let ctx = CliContext::parse(args);
    let blacklist = security_db::get_blacklist().unwrap_or_default();
    if ctx.json_output {
        CliEnvelope::ok("security ip-list", None, blacklist).print_and_exit();
    }
    println!("Blacklisted IPs ({}):", blacklist.len());
    for item in blacklist {
        println!(
            "  ● {} ({})",
            item.ip_pattern,
            item.reason.unwrap_or_default()
        );
    }
}

pub(crate) fn cmd_security_ip_block(args: &[String]) {
    let ctx = CliContext::parse(args);
    if let Some(ip) = ctx.positional_args.first() {
        let _ = security_db::add_to_blacklist(ip, Some("Manual CLI block"), None, "agm_cli");
        if ctx.json_output {
            CliEnvelope::ok(
                "security ip-block",
                None,
                serde_json::json!({ "blocked": ip }),
            )
            .print_and_exit();
        }
        println!("[SUCCESS] Blocked IP '{}'.", ip);
    }
}

pub(crate) fn cmd_security_ip_unblock(args: &[String]) {
    let ctx = CliContext::parse(args);
    if let Some(ip) = ctx.positional_args.first() {
        let _ = security_db::remove_from_blacklist(ip);
        if ctx.json_output {
            CliEnvelope::ok(
                "security ip-unblock",
                None,
                serde_json::json!({ "unblocked": ip }),
            )
            .print_and_exit();
        }
        println!("[SUCCESS] Unblocked IP '{}'.", ip);
    }
}

pub(crate) fn cmd_security_audit(args: &[String]) {
    let ctx = CliContext::parse(args);
    let logs = security_db::get_ip_access_logs(50, 0, None, false).unwrap_or_default();
    if ctx.json_output {
        CliEnvelope::ok("security audit", None, logs).print_and_exit();
    }
    println!("Security Audit Logs (last {} events):", logs.len());
    for log in logs {
        println!(
            "  ● [{}] {} {} - {}",
            log.timestamp,
            log.method.as_deref().unwrap_or("-"),
            log.path.as_deref().unwrap_or("-"),
            log.status.unwrap_or(0)
        );
    }
}
