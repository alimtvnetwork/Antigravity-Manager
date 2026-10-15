//! quota_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn dispatch_quota_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_quota_show(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_quota_show(args);
        return;
    }
    dispatch_quota_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_quota_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "show" | "status" | "quota" => cmd_quota_show(args),
        "refresh" => cmd_quota_refresh(args),
        "list" | "accounts" => crate::account_cmds::cmd_accounts_list(args),
        "add" => crate::account_cmds::cmd_accounts_add(args),
        "remove" | "rm" => crate::account_cmds::cmd_accounts_remove(args),
        "validate" | "check" => crate::account_cmds::cmd_accounts_validate(args),
        "balance" => crate::account_cmds::cmd_accounts_balance(args),
        _ => crate::common::handle_unknown_domain_command("quota", subcommand, args),
    }
}

pub(crate) fn cmd_quota_show(args: &[String]) {
    let is_csv = args.iter().any(|a| a.eq_ignore_ascii_case("--csv"))
        || args
            .windows(2)
            .any(|w| w[0].eq_ignore_ascii_case("--format") && w[1].eq_ignore_ascii_case("csv"));
    if is_csv {
        match telegram_inbound::generate_export_dataset("balance", "csv") {
            Ok((_name, _mime, csv)) => {
                print!("{}", csv);
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export balance CSV: {}", e);
                std::process::exit(1);
            }
        }
    }

    let ctx = CliContext::parse(args);
    if ctx.json_output {
        let cluster_balances =
            ssh_manager::get_cluster_instances_and_balances().unwrap_or_default();
        let accounts = account::list_accounts().unwrap_or_default();
        CliEnvelope::ok(
            "quota show",
            None,
            serde_json::json!({
                "instances_and_balances": cluster_balances,
                "accounts": accounts,
            }),
        )
        .print_and_exit();
    }
    if let Ok(items) = ssh_manager::get_cluster_instances_and_balances() {
        println!("\n[*] Multi-Machine Instance & Account Credit Balances:");
        println!(
            "    {:<14} {:<16} {:<14} {:<28} {:<8} {:<10} {:<10}",
            "MACHINE", "IP", "INSTANCE", "BOUND ACCOUNT", "TIER", "4H QUOTA", "WEEKLY"
        );
        println!("    --------------------------------------------------------------------------------------------------------");
        for item in &items {
            println!(
                "    {:<14} {:<16} {:<14} {:<28} {:<8} {:>7.1}%   {:>7.1}%",
                item.machine_alias,
                item.machine_ip,
                item.instance_name,
                item.bound_email,
                item.tier,
                item.quota_4h_pct,
                item.quota_weekly_pct
            );
        }
        println!();
    }
    crate::status_cmds::cmd_status(args);
}

pub(crate) fn cmd_quota_refresh(args: &[String]) {
    crate::account_cmds::cmd_accounts_refresh_tier(args);
}
