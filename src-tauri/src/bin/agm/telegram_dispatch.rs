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
        if first_lower == "theme" || first_lower == "palette" {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
            match telegram_inbound::set_card_palette(name) {
                Ok(msg) => println!("{}", msg),
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }
        if first_lower == "config" {
            let mut summary_words = None;
            let mut iter = args[1..].iter().peekable();
            while let Some(arg) = iter.next() {
                if arg == "--summary-words" || arg == "-w" || arg == "--words" {
                    if let Some(val) = iter.next() {
                        match val.parse::<usize>() {
                            Ok(w) if w > 0 => summary_words = Some(w),
                            _ => {
                                eprintln!("[ERROR] --summary-words requires a positive integer (e.g. 100 or 200)");
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("[ERROR] Missing argument for --summary-words");
                        std::process::exit(1);
                    }
                }
            }

            if let Some(words) = summary_words {
                t_cfg.prompt_summary_words = words;
                if let Err(e) = telegram_inbound::save_config(&t_cfg) {
                    eprintln!("[ERROR] Failed to save Telegram config: {}", e);
                    std::process::exit(1);
                }
                println!("[+] Telegram prompt summary word count set to: {}", words);
                return;
            }

            // Display current configuration
            println!("=== Telegram Configuration ===");
            println!("  Enabled: {}", t_cfg.is_enabled);
            println!(
                "  Bot Token: {}",
                if t_cfg.bot_token.is_empty() {
                    "(not set)"
                } else {
                    "(configured)"
                }
            );
            println!(
                "  Chat ID: {}",
                t_cfg
                    .allowed_chat_id
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "(none)".to_string())
            );
            println!("  Prompt Summary Words: {}", t_cfg.prompt_summary_words);
            println!("  Card Palette: {}", t_cfg.card_palette);
            println!("  Poll Interval: {}s", t_cfg.poll_interval_secs);
            println!("\nUsage: agm telegram config --summary-words <100|200>");
            return;
        }
        if crate::telegram_blocks_a::telegram_route_help(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_export(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_import(args, &first_lower, &t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_set(args, &first_lower, &mut t_cfg, &rt) {
            return;
        }
        if crate::telegram_blocks_a::telegram_route_detect_chat_id(
            args,
            &first_lower,
            &mut t_cfg,
            &rt,
        ) {
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
