//! telegram_blocks_a — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn telegram_route_help(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
        println!("AGM Telegram Remote & Notification Subsystem:");
        println!("  agm telegram chat <token> [chat_id]     Connect bot & chat, auto-discover chat ID & send welcome ping");
        println!("  agm telegram setup <token> [chat_id]    Setup Telegram credentials, register bot commands & enable");
        println!("  agm telegram connect <token> [chat_id]  Auto-detect chat ID, save token & send welcome ping");
        println!("  agm telegram set <token> [chat_id]      Save bot credentials (auto-detects chat_id if omitted)");
        println!("  agm telegram detect-chat-id [token]     Auto-discover your numeric Chat ID from getUpdates");
        println!("  agm telegram ls [--json]                Show configured bot token, chat ID, and status");
        println!("  agm telegram nodes                      List all cluster VM nodes & connectivity status");
        println!(
            "  agm telegram projects                   List discovered workspaces and project IDs"
        );
        println!("  agm telegram prompts [node]             List active running prompts (optionally scoped to node)");
        println!("  agm telegram prompt <node> <proj> <txt> Inject prompt to workspace or cluster VM node");
        println!("  agm telegram ping                       Send rich telemetry ping card to Telegram chat");
        println!("  agm telegram observe, status            Send live workspaces, quota & prompt queue report");
        println!("  agm telegram gitmap [args...]           Run GitMap command (e.g. pe) & forward output to chat");
        println!("  agm telegram api                        Query local API proxy status & forward to chat");
        println!("  agm telegram backup, backpack [ls]      Backup running prompts to split SQLite DB & notify");
        println!("  agm telegram restore                    Restore backed-up prompts & notify Telegram chat");
        println!("  agm telegram email [status|ping|help]   Check or dispatch email & forward receipt to Telegram");
        println!("  agm telegram send, notify \"<message>\"   Send custom notification message to Telegram chat");
        println!(
            "  agm telegram poll, watch [--once]       Poll & execute inbound Telegram commands"
        );
        println!("  agm telegram cmds, commands             List supported inbound Telegram slash commands");
        println!("  agm telegram export [--file <path>]     Export telegram config wrapped in standard JSON envelope");
        println!("  agm telegram import <path>              Import telegram config from standard JSON envelope file");
        return true;
    }
    false
}

pub(crate) fn telegram_route_export(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "export" {
        match telegram_inbound::export_config_json(&t_cfg) {
            Ok(json_str) => {
                let file_arg = args
                    .iter()
                    .position(|a| a == "--file" || a == "-o")
                    .and_then(|idx| args.get(idx + 1));
                if let Some(target_file) = file_arg {
                    if let Err(e) = fs::write(target_file, &json_str) {
                        eprintln!("[ERROR] Failed to write to {}: {}", target_file, e);
                    } else {
                        println!(
                            "✅ Successfully exported Telegram configuration to {}",
                            target_file
                        );
                    }
                } else {
                    println!("{}", json_str);
                }
            }
            Err(e) => eprintln!("[ERROR] Failed to export Telegram configuration: {}", e),
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_import(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "import" || first_lower == "load-json" {
        let target_path = args.get(1);
        if let Some(path_str) = target_path {
            let resolved_path = json_envelope::resolve_relative_json_path(path_str);
            match fs::read_to_string(&resolved_path) {
                Ok(raw_json) => {
                    match json_envelope::extract_payload::<telegram_inbound::TelegramConfig>(
                        &raw_json,
                    ) {
                        Ok((imported_cfg, attrs)) => {
                            match telegram_inbound::save_config(&imported_cfg) {
                                Ok(_) => {
                                    println!(
                                            "✅ Successfully imported Telegram configuration from '{}' (Envelope v{}).",
                                            resolved_path.display(),
                                            attrs.version
                                        );
                                }
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to save Telegram config: {}", e)
                                }
                            }
                        }
                        Err(e) => eprintln!("[ERROR] Failed to parse Telegram config: {}", e),
                    }
                }
                Err(e) => eprintln!(
                    "[ERROR] Failed to read file '{}': {}",
                    resolved_path.display(),
                    e
                ),
            }
        } else {
            eprintln!("Usage: agm telegram import <file_path>");
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_set(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "set"
        || first_lower == "connect"
        || first_lower == "chat"
        || first_lower == "setup"
    {
        let help_requested = args
            .get(1)
            .map(|s| s.eq_ignore_ascii_case("help") || s == "--help" || s == "-h")
            .unwrap_or(args.len() <= 1);

        if help_requested {
            println!("\n=== Telegram Bot & Chat Setup Guide ===");
            println!("Usage:");
            println!("  agm telegram chat <BOT_TOKEN> [CHAT_ID]");
            println!("  agm telegram setup <BOT_TOKEN> [CHAT_ID]");
            println!("\nArguments:");
            println!(
                "  <BOT_TOKEN>     The API token from @BotFather (e.g. 123456789:ABCdefGHI...)"
            );
            println!(
                "  [CHAT_ID]       Optional numeric ID of your chat or group. If omitted, AGM"
            );
            println!(
                "                  automatically queries getUpdates to discover your Chat ID!"
            );
            println!("\nWhere do I get the Chat ID?");
            println!("  Option 1 (Automatic Discovery - Recommended):");
            println!("    1. Open your bot in Telegram and send '/start' or '/ping'");
            println!("    2. Run: agm telegram chat <BOT_TOKEN>");
            println!(
                "    3. AGM will discover your Chat ID from the incoming message automatically!"
            );
            println!("  Option 2 (Direct Query via Bot):");
            println!("    1. In Telegram search for '@userinfobot' or '@RawDataBot'");
            println!("    2. Send '/start' -> it will immediately show your numeric 'Id' (e.g. 987654321)");
            println!("    3. Run: agm telegram chat <BOT_TOKEN> 987654321");
            println!("  Option 3 (Group or Channel):");
            println!("    1. Add your bot to the group or channel and post any message");
            println!("    2. Run: agm telegram chat <BOT_TOKEN>");
            println!(
                "    3. AGM will discover the group's negative Chat ID (e.g. -100123456789)\n"
            );
            return true;
        }

        // Extract token and optional chat_id (handling --auto-detect flag if present)
        let mut remaining_args: Vec<String> = args[1..].to_vec();
        remaining_args.retain(|a| a != "--auto-detect" && a != "-a");

        let token = match remaining_args.first() {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => {
                eprintln!(
                    "[ERROR] Missing bot token. Run 'agm telegram chat --help' for instructions."
                );
                return true;
            }
        };

        let chat_id_opt: Option<i64> = if let Some(cid_str) = remaining_args.get(1) {
            match cid_str.parse::<i64>() {
                Ok(id) => Some(id),
                Err(_) => {
                    eprintln!(
                        "[ERROR] chat_id must be a numeric integer. Found: '{}'",
                        cid_str
                    );
                    return true;
                }
            }
        } else {
            println!("[*] Auto-detecting Telegram Chat ID via getUpdates...");
            match rt.block_on(telegram_inbound::detect_telegram_chat_id(&token)) {
                Ok(det) => {
                    println!(
                        "[+] Discovered Chat ID: {} ({}) on bot @{}",
                        det.chat_id, det.chat_label, det.bot_username
                    );
                    Some(det.chat_id)
                }
                Err(e) => {
                    eprintln!("[WARN] Could not auto-detect Chat ID yet: {}", e);
                    println!("💡 Hint: Open Telegram, search for your bot, send '/start', then run this command again!");
                    t_cfg.allowed_chat_id
                }
            }
        };

        t_cfg.bot_token = token.clone();
        if let Some(cid) = chat_id_opt {
            t_cfg.allowed_chat_id = Some(cid);
        }
        t_cfg.is_enabled = true;
        if let Err(e) = telegram_inbound::save_config(&t_cfg) {
            eprintln!("[ERROR] Failed to save Telegram settings: {}", e);
            std::process::exit(1);
        }
        let _ = rt.block_on(telegram_inbound::register_telegram_bot_commands(&token));
        println!(
            "[SUCCESS] Telegram credentials saved and enabled! (Chat ID: {:?})",
            t_cfg.allowed_chat_id
        );

        if let Some(cid) = t_cfg.allowed_chat_id {
            let welcome = telegram_inbound::format_ping_report();
            if rt
                .block_on(telegram_inbound::send_telegram_message(
                    &token, cid, &welcome,
                ))
                .is_ok()
            {
                println!(
                    "[SUCCESS] Delivered welcome telemetry ping to Telegram chat {}!",
                    cid
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_detect_chat_id(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "detect-chat-id" || first_lower == "chat-id" {
        let token = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| t_cfg.bot_token.clone());
        if token.trim().is_empty() {
            eprintln!("[ERROR] No bot token provided or configured. Usage: agm telegram detect-chat-id <bot_token>");
            return true;
        }
        match rt.block_on(telegram_inbound::detect_telegram_chat_id(&token)) {
            Ok(det) => {
                println!("=== Discovered Telegram Chat ===");
                println!("  Bot Username: @{}", det.bot_username);
                println!("  Chat ID:      {}", det.chat_id);
                println!("  Chat Label:   {}", det.chat_label);
                if t_cfg.allowed_chat_id.is_none() {
                    t_cfg.bot_token = token;
                    t_cfg.allowed_chat_id = Some(det.chat_id);
                    t_cfg.is_enabled = true;
                    let _ = telegram_inbound::save_config(&t_cfg);
                    println!(
                        "[SUCCESS] Automatically saved Chat ID {} to telegram_config.json!",
                        det.chat_id
                    );
                }
            }
            Err(e) => eprintln!("[ERROR] {}", e),
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_ls(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "ls" || first_lower == "list" {
        let is_json = args.iter().any(|a| a == "--json");
        if is_json {
            let mut redacted = t_cfg.clone();
            if redacted.bot_token.len() > 8 {
                redacted.bot_token = format!(
                    "{}...{}",
                    &t_cfg.bot_token[..4],
                    &t_cfg.bot_token[t_cfg.bot_token.len() - 4..]
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&redacted).unwrap_or_else(|_| "{}".to_string())
            );
            return true;
        }
        let masked_token = if t_cfg.bot_token.len() > 8 {
            format!(
                "{}...{}",
                &t_cfg.bot_token[..4],
                &t_cfg.bot_token[t_cfg.bot_token.len() - 4..]
            )
        } else if !t_cfg.bot_token.is_empty() {
            "***".to_string()
        } else {
            "(not set)".to_string()
        };
        println!("\n=== Telegram Notification Settings ===");
        println!("  Enabled:              {}", t_cfg.is_enabled);
        println!("  Bot Token:            {}", masked_token);
        println!(
            "  Allowed Chat ID:      {}",
            t_cfg
                .allowed_chat_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "(auto-bind on first message)".to_string())
        );
        println!("  Poll Interval (secs): {}", t_cfg.poll_interval_secs);
        println!("  Notify on Update:     {}", t_cfg.notify_on_system_update);
        println!();
        return true;
    }
    false
}

pub(crate) fn telegram_route_ping(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "ping" {
        let Some(chat_id) = t_cfg.allowed_chat_id else {
            eprintln!(
                "[ERROR] Telegram chat ID is not configured. Run 'agm telegram connect <token>'"
            );
            return true;
        };
        if t_cfg.bot_token.is_empty() {
            eprintln!("[ERROR] Telegram bot token is not configured.");
            return true;
        }
        println!(
            "[*] Sending telemetry ping to Telegram chat '{}'...",
            chat_id
        );
        let msg = telegram_inbound::format_ping_report();
        match rt.block_on(telegram_inbound::send_telegram_message(
            &t_cfg.bot_token,
            chat_id,
            &msg,
        )) {
            Ok(_) => println!("[SUCCESS] Telegram telemetry ping delivered!"),
            Err(e) => eprintln!("[ERROR] Failed to send Telegram message: {}", e),
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_observe(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "observe" || first_lower == "status" {
        let report = telegram_inbound::format_observe_report();
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                match rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                )) {
                    Ok(_) => println!(
                        "\n[SUCCESS] Observation report delivered to Telegram chat {}!",
                        chat_id
                    ),
                    Err(e) => eprintln!("\n[ERROR] Failed to send observation report: {}", e),
                }
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_queues(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "queues" || first_lower == "queue" {
        let report = rt.block_on(telegram_inbound::format_prompt_queues_report());
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                ));
                println!(
                    "\n[SUCCESS] Prompt queues report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_projects(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "projects" || first_lower == "workspaces" {
        let report = telegram_inbound::format_projects_list();
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                ));
                println!(
                    "\n[SUCCESS] Projects catalog delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_prompts_tpl(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "prompts" || first_lower == "templates" {
        let target_sub = args.get(1).map(|s| s.as_str()).unwrap_or("");
        let report = if target_sub.is_empty() || target_sub == "ls" || target_sub == "list" {
            telegram_inbound::format_prompts_templates_report()
        } else if target_sub == "all" || target_sub == "db" {
            telegram_inbound::format_prompts_list()
        } else {
            rt.block_on(telegram_inbound::format_node_scoped_prompts(target_sub))
        };
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                ));
                println!(
                    "\n[SUCCESS] Prompts report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}
