//! fleet_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn dispatch_fleet_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_fleet_nodes(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_fleet_nodes(args);
        return;
    }
    dispatch_fleet_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_fleet_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "nodes" => cmd_fleet_nodes(args),
        "ping" => cmd_fleet_ping(args),
        "sync" => cmd_fleet_sync(args),
        "broadcast" => cmd_fleet_broadcast(args),
        "health" => cmd_fleet_health(args),
        _ => crate::common::handle_unknown_domain_command("fleet", subcommand, args),
    }
}

pub(crate) fn cmd_fleet_nodes(args: &[String]) {
    let ctx = CliContext::parse(args);
    let (name, ip) = email_sender::get_local_node_identity();
    if ctx.json_output {
        let data = serde_json::json!([{ "node_alias": name, "local_ip": ip, "is_leader": true }]);
        CliEnvelope::ok("fleet nodes", None, data).print_and_exit();
    }
    println!("Fleet Nodes:\n  ● {} ({}) - Online", name, ip);
}

pub(crate) fn cmd_fleet_ping(args: &[String]) {
    let ctx = CliContext::parse(args);
    let (name, ip) = email_sender::get_local_node_identity();
    if ctx.json_output {
        CliEnvelope::ok(
            "fleet ping",
            None,
            serde_json::json!({ "node": name, "ip": ip, "status": "pong" }),
        )
        .print_and_exit();
    }
    println!("[PONG] Local node '{}' ({}) is healthy.", name, ip);
}

pub(crate) fn cmd_fleet_sync(args: &[String]) {
    crate::proxy_cmds::cmd_sync(args);
}

pub(crate) fn cmd_fleet_broadcast(args: &[String]) {
    crate::email_cmds::cmd_broadcast_email(args);
}

pub(crate) fn cmd_fleet_health(args: &[String]) {
    crate::doctor_cmds::cmd_doctor(args);
}
