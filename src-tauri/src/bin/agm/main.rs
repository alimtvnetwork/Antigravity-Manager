//! AGM - Antigravity-Manager Native Terminal CLI
//! Autonomous terminal companion for Antigravity-Manager:
//! Status monitoring, multi-instance management, fast-forward switching,
//! accounts listing, direct account switching, doctor health checks,
//! prompt inspection, proxy status/test, sync, git pull, clean/purge, logs,
//! PATH self-installation, GitHub auto-updates, and SSH remote machine management.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
mod account_cmds;
mod account_mgmt;
mod cache_cmds;
mod common;
mod doctor_cmds;
mod email_arms_a;
mod email_arms_b;
mod email_cmds;
mod email_dispatch;
mod fleet_cmds;
mod green_cmds;
mod help_tables;
mod help_text;
mod install_cmds;
mod instance_cmds;
mod instance_dispatch;
mod instance_fastforward;
mod instance_import_export;
mod instance_routes_a;
mod instance_routes_b;
mod instance_settings;
mod misc_cmds;
mod project_cmds;
mod prompt_cmds;
mod prompt_dispatch;
mod prompt_goals;
mod prompt_import_export;
mod prompt_mgmt;
mod prompt_resend;
mod proxy_cmds;
mod quota_cmds;
mod remediate_cmds;
mod running_backup_cmds;
mod running_prompts_cmds;
mod scheduler_cmds;
mod security_cmds;
mod ssh_cmds;
mod ssh_handlers;
mod status_cmds;
mod supabase_cmds;
mod supabase_config_cmds;
mod supabase_format;
mod switch_check_cmds;
mod switch_credit_cmds;
mod system_cmds;
mod telegram_blocks_a;
mod telegram_blocks_b;
mod telegram_blocks_c;
mod telegram_dispatch;
mod test_cmds;
mod test_flow;
mod test_flow_steps;
mod test_flow_steps2;
mod update_cmds;
mod update_helpers;

use std::env;

fn main() {
    antigravity_tools_lib::modules::logger::init_logger();
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        crate::help_text::print_banner();
        crate::help_text::print_help();
        crate::help_text::print_commands_table();
        crate::help_tables::print_live_projects_table();
        crate::help_tables::print_prompts_reference_table();
        crate::help_tables::print_recent_prompts_table();
        return;
    }

    let subcommand = args[1]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    let cmd_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    match subcommand.as_str() {
        "status" => crate::status_cmds::cmd_status(&cmd_args),
        "credits" | "credit" | "status/credits" | "quota" => {
            crate::quota_cmds::dispatch_quota_domain_from_args(&cmd_args);
        }
        "instances" | "instance" | "intrance" | "intrances" | "profile" | "profiles" => {
            crate::instance_cmds::dispatch_instance_domain_from_args(&cmd_args);
        }
        "ls" => {
            if cmd_args
                .first()
                .map(|s| s == "prompts" || s == "prompt")
                .unwrap_or(false)
            {
                crate::prompt_cmds::dispatch_prompt_domain("list", &cmd_args[1..]);
            } else {
                crate::instance_cmds::dispatch_instance_domain("list", &cmd_args);
            }
        }
        "duplicate" | "instance-duplicate" => {
            crate::instance_cmds::dispatch_instance_domain("copy", &cmd_args);
        }
        "clone" | "instance-clone" => {
            crate::instance_cmds::dispatch_instance_domain("copy", &cmd_args);
        }
        "count" | "instance-count" | "instances-count" => {
            let mut forward_args = vec!["count".to_string()];
            forward_args.extend(cmd_args);
            crate::instance_dispatch::cmd_instances(&forward_args);
        }
        "copy-projects" | "copy_projects" => {
            let mut forward_args = vec!["copy-projects".to_string()];
            forward_args.extend(cmd_args);
            crate::instance_dispatch::cmd_instances(&forward_args);
        }
        "copy-settings" | "copy_settings" | "sync-settings" | "sync_settings" => {
            let mut forward_args = vec!["sync-settings".to_string()];
            forward_args.extend(cmd_args);
            crate::instance_dispatch::cmd_instances(&forward_args);
        }
        "theme" | "profile-theme" => crate::instance_settings::cmd_theme(&cmd_args),
        "settings" | "instance-settings" => {
            let mut forward_args = vec!["settings".to_string()];
            forward_args.extend(cmd_args);
            crate::instance_dispatch::cmd_instances(&forward_args);
        }
        "create" | "create-instance" | "create_instance" | "instance-create"
        | "intrance-create" | "intrance_create" => {
            crate::instance_cmds::dispatch_instance_domain("create", &cmd_args);
        }
        "launch" | "start" | "instance-launch" | "instance-start" => {
            crate::instance_cmds::dispatch_instance_domain("start", &cmd_args);
        }
        "stop" | "kill" | "instance-stop" | "instance-kill" => {
            crate::instance_cmds::dispatch_instance_domain("stop", &cmd_args);
        }
        "restart" | "instance-restart" => {
            crate::instance_cmds::dispatch_instance_domain("restart", &cmd_args);
        }
        "rm" | "delete" | "instance-delete" | "instance-rm" => {
            crate::instance_cmds::dispatch_instance_domain("delete", &cmd_args);
        }
        "instances-all" => crate::instance_import_export::cmd_instances_all(&cmd_args),
        "observe" | "inspect" | "watch" => {
            crate::prompt_goals::cmd_observe(&cmd_args);
        }
        "test-instance-flow"
        | "test-instance-switching"
        | "instance-flow"
        | "test-instance"
        | "tif" => {
            crate::test_flow::cmd_test_instance_flow(&cmd_args);
        }
        "doctor" | "check" => crate::system_cmds::dispatch_system_domain("doctor", &cmd_args),
        "accounts" | "account" | "acc" => {
            crate::quota_cmds::dispatch_quota_domain("list", &cmd_args)
        }
        "refresh-tier" | "refresh_tier" => {
            crate::quota_cmds::dispatch_quota_domain("refresh", &cmd_args)
        }
        "history" | "audit" => crate::misc_cmds::cmd_history(&cmd_args),
        "switch" | "switch-account" | "switch_account" | "account-switch" | "swtich"
        | "swtich-account" | "swtich_account" | "account-swtich" => {
            crate::instance_cmds::dispatch_instance_domain("switch", &cmd_args);
        }
        "switch-if-low-credit" | "swlc" | "sfc" | "switch-if-no-credit" => {
            crate::switch_credit_cmds::cmd_switch_if_low_credit(&cmd_args);
        }
        "is-low-credit-for-switch" | "is-low-credit" | "ilc" => {
            crate::switch_check_cmds::cmd_is_low_credit_for_switch(&cmd_args);
        }
        "which-prompts-running" | "wpr" => {
            crate::prompt_cmds::dispatch_prompt_domain("running", &cmd_args)
        }
        "prompts" => crate::prompt_cmds::dispatch_prompt_domain_from_args(&cmd_args),
        "prompt" => {
            if !cmd_args.is_empty() && crate::common::is_prompt_subcommand(&cmd_args[0]) {
                crate::prompt_cmds::dispatch_prompt_domain_from_args(&cmd_args);
            } else {
                crate::prompt_dispatch::cmd_prompt_dispatch(&cmd_args);
            }
        }
        "rerun" => crate::prompt_goals::cmd_rerun(&cmd_args),
        "prompts-export" | "pe" => crate::prompt_cmds::dispatch_prompt_domain("export", &cmd_args),
        "prompts-import" | "pi" => crate::prompt_import_export::cmd_prompts_import(&cmd_args),
        "resend-running-commands" | "rrc" | "resend-running" | "resend" => {
            crate::prompt_resend::cmd_resend_running_commands(&cmd_args);
        }
        "backup-running-prompts" | "brp" | "backup-prompts" | "backup" | "backpack" => {
            crate::prompt_cmds::dispatch_prompt_domain("backup", &cmd_args);
        }
        "queue-scheduler" | "queue_scheduler" | "scheduler" | "qs" => {
            crate::scheduler_cmds::cmd_queue_scheduler(&cmd_args);
        }
        "restore-running-prompts" | "rrp" | "restore-prompts" | "restore" => {
            crate::prompt_cmds::dispatch_prompt_domain("restore", &cmd_args);
        }
        "running-prompts" => {
            crate::prompt_cmds::dispatch_prompt_domain("running", &cmd_args);
        }
        "running-projects" => {
            crate::green_cmds::cmd_running_projects(&cmd_args);
        }
        "finish-prompts-until-green" | "fpug" => {
            crate::green_cmds::cmd_finish_prompts_until_green(&cmd_args);
        }
        "shutdown-until-green" | "sug" => {
            crate::green_cmds::cmd_shutdown_until_green(&cmd_args);
        }
        "broadcast-email" => {
            crate::email_cmds::cmd_broadcast_email(&cmd_args);
        }
        "telegram" => {
            crate::telegram_dispatch::cmd_telegram(&cmd_args);
        }
        "supabase" | "supa" => {
            crate::supabase_cmds::cmd_supabase(&cmd_args);
        }
        "which-format" | "which_format" | "format" | "inspect-format" | "scan-format" => {
            crate::supabase_format::cmd_which_format(&cmd_args);
        }
        "nodes" => {
            crate::fleet_cmds::dispatch_fleet_domain("nodes", &cmd_args);
        }
        "projects" | "workspaces" => {
            crate::telegram_dispatch::cmd_telegram(&["projects".to_string()]);
        }
        "tree" => {
            crate::prompt_cmds::dispatch_prompt_domain("tree", &cmd_args);
        }
        "query" | "search" | "find" => {
            crate::prompt_mgmt::cmd_prompts_query(&cmd_args);
        }
        "active" | "running" => {
            if cmd_args.is_empty() {
                crate::prompt_cmds::dispatch_prompt_domain("tree", &[]);
            } else {
                crate::misc_cmds::cmd_agy(&cmd_args);
            }
        }
        "queues" | "queue" => {
            crate::prompt_cmds::dispatch_prompt_domain("queue", &cmd_args);
        }
        "agy" => {
            crate::misc_cmds::cmd_agy(&cmd_args);
        }
        "proxy" => crate::proxy_cmds::dispatch_proxy_domain_from_args(&cmd_args),
        "fleet" => crate::fleet_cmds::dispatch_fleet_domain_from_args(&cmd_args),
        "security" => crate::security_cmds::dispatch_security_domain_from_args(&cmd_args),
        "system" => crate::system_cmds::dispatch_system_domain_from_args(&cmd_args),
        "db-stats" | "db_stats" => {
            crate::system_cmds::dispatch_system_domain("db-stats", &cmd_args)
        }
        "vacuum" => crate::system_cmds::dispatch_system_domain("vacuum", &cmd_args),
        "ip-list" => crate::security_cmds::dispatch_security_domain("ip-list", &cmd_args),
        "ip-block" => crate::security_cmds::dispatch_security_domain("ip-block", &cmd_args),
        "ip-unblock" => crate::security_cmds::dispatch_security_domain("ip-unblock", &cmd_args),
        "sync" => crate::proxy_cmds::cmd_sync(&cmd_args),
        "pull" => crate::proxy_cmds::cmd_pull(&cmd_args),
        "clean" | "purge" => crate::misc_cmds::cmd_clean(&cmd_args),
        "prune" | "pr" | "clear-cache" | "cache-clear" => {
            crate::cache_cmds::cmd_clear_cache(&cmd_args)
        }
        "clear-terminal" | "clean-terminal" => crate::system_cmds::cmd_clear_terminal(&cmd_args),
        "gitignore" | "gitignore-agm" | "gitignore-agy" | "ignore" => {
            crate::misc_cmds::cmd_gitignore(&cmd_args)
        }
        "failed-commands"
        | "failed-command"
        | "fc"
        | "unknown-commands"
        | "unknown-command"
        | "failed-to-detect"
        | "failed-to-detect-commands"
        | "failed-commands-count"
        | "fcc" => {
            crate::misc_cmds::cmd_failed_commands(&cmd_args);
        }
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
                crate::cache_cmds::cmd_clear_cache(&rest);
            } else if cmd_args
                .first()
                .map(|s| s.eq_ignore_ascii_case("terminal"))
                .unwrap_or(false)
            {
                crate::system_cmds::cmd_clear_terminal(&cmd_args);
            } else {
                crate::cache_cmds::cmd_clear_cache(&cmd_args);
            }
        }
        "recreate-project" => crate::project_cmds::cmd_recreate_project(&cmd_args),
        "recreate" => crate::project_cmds::cmd_recreate(&cmd_args),
        "email" => crate::email_dispatch::cmd_email(&cmd_args),
        "logs" | "log" => crate::cache_cmds::cmd_logs(&cmd_args),
        "ff" | "smart-switch" | "fast-forward" => {
            crate::instance_fastforward::cmd_fast_forward(&cmd_args)
        }
        "test-switcher" | "test-auto-switch" | "auto-switch" | "auto-swtich" | "auto"
        | "autoswitch" | "auto_switch" | "switcher" => {
            crate::scheduler_cmds::cmd_auto_switch(&cmd_args);
        }
        "test-email" | "email-test" | "check-email" => crate::test_cmds::cmd_test_email(&cmd_args),
        "test-training" | "training" | "train" => crate::test_cmds::cmd_test_training(&cmd_args),
        "install" => crate::install_cmds::cmd_install(&cmd_args),
        "update" | "update-all" | "ua" => crate::update_cmds::cmd_update(&cmd_args),
        "delegate-update" | "update-ui" | "ui-update-runner" => {
            crate::update_cmds::cmd_delegate_update(&cmd_args);
        }
        "open-ui" | "ui" | "launch-ui" | "start-ui" => {
            antigravity_tools_lib::modules::delegate_updater::open_ui(&cmd_args);
        }
        "ssh" | "sj" | "se" => {
            let mut f = if subcommand == "se" {
                vec!["exec".to_string()]
            } else {
                Vec::new()
            };
            f.extend(cmd_args);
            crate::ssh_cmds::cmd_ssh(&f);
        }
        "version" | "--version" | "-v" => {
            let git_hash = antigravity_tools_lib::modules::git_info::get_git_hash();
            let git_branch = antigravity_tools_lib::modules::git_info::get_git_branch();
            let last_release = antigravity_tools_lib::modules::git_info::get_last_release();
            println!(
                "agm v{} (commit: {}, branch: {}, last release: {})",
                crate::common::VERSION,
                git_hash,
                git_branch,
                last_release
            );
        }
        "help" | "--help" | "-h" => {
            if cmd_args.iter().any(|a| a == "--json" || a == "-j") {
                crate::help_text::print_help_json();
            } else {
                crate::help_text::print_banner();
                crate::help_text::print_help();
                crate::help_text::print_commands_table();
                crate::help_tables::print_live_projects_table();
                crate::help_tables::print_prompts_reference_table();
                crate::help_tables::print_recent_prompts_table();
            }
        }
        _ => {
            crate::common::handle_unknown_command(&args[1], &args);
        }
    }
}
