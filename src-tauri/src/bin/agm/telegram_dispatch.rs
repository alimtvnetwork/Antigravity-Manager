//! telegram_dispatch — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn cmd_telegram(args: &[String]) {
    let mut t_cfg = match telegram_inbound::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load Telegram config: {}", e);
            std::process::exit(1);
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Tokio runtime error: {}", e);
            return;
        }
    };

    if let Some(first) = args.first() {
        let first_lower = first.trim_start_matches('/').to_lowercase();
        if crate::telegram_blocks_a::telegram_route_help(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_export(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_import(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_set(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_detect_chat_id(args, &first_lower, &t_cfg, &rt)
        {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_ls(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_ping(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_observe(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_c::telegram_route_nodes(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_c::telegram_route_active(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_queues(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_projects(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_prompts_tpl(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_prompt_inject(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_node(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_prompt_queue(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_workspaces(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_inject2(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_gitmap(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_api(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_backup(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_email(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_send(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_poll(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_b::telegram_route_cmds(args, &first_lower, &t_cfg, &rt) {
            return;
        }
    }

    println!("AGM Telegram Subsystem. Run 'agm telegram help' for available commands.");
}
