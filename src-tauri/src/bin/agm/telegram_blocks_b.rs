//! telegram_blocks_b — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::time::Duration;

pub(crate) fn telegram_route_prompt_inject(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "prompt" || first_lower == "inject" {
        let prompt_args = args[1..].join(" ");
        let report = rt.block_on(telegram_inbound::execute_prompt_injection(&prompt_args));
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                ));
                println!(
                    "\n[SUCCESS] Prompt injection receipt delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_node(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "nodes" || first_lower == "node" {
        let sub = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            String::new()
        };
        let reply = if sub.is_empty() || sub == "ls" || sub == "list" || sub == "status" {
            rt.block_on(telegram_inbound::format_cluster_nodes_report())
        } else {
            rt.block_on(telegram_inbound::format_node_scoped_prompts(&sub))
        };
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] Cluster nodes report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_prompt_queue(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "prompts" || first_lower == "prompt_queue" {
        let sub = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            String::new()
        };
        let reply = if sub.is_empty() || sub == "ls" || sub == "list" {
            telegram_inbound::format_prompts_list()
        } else {
            rt.block_on(telegram_inbound::execute_prompt_injection(&sub))
        };
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
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

pub(crate) fn telegram_route_workspaces(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "projects" || first_lower == "workspaces" || first_lower == "workspace" {
        let reply = telegram_inbound::format_projects_list();
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
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

pub(crate) fn telegram_route_inject2(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "prompt" || first_lower == "inject" {
        if args.len() < 2 {
            eprintln!("Usage: agm telegram prompt [node] <project> \"<prompt text>\"");
            return true;
        }
        let sub_args = args[1..].join(" ");
        let reply = rt.block_on(telegram_inbound::execute_prompt_injection(&sub_args));
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] Prompt injection delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_gitmap(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "gitmap" || first_lower == "gm" {
        let sub_args = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            "pe".to_string()
        };
        let reply = telegram_inbound::execute_gitmap_subcommand(&sub_args);
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] GitMap output delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_api(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "api" || first_lower == "proxy" {
        let reply = rt.block_on(telegram_inbound::execute_api_status_command());
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] API status delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_backup(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "backup" || first_lower == "backpack" || first_lower == "restore" {
        let sub_args = if first_lower == "restore" {
            "restore".to_string()
        } else if args.len() > 1 {
            args[1..].join(" ")
        } else {
            String::new()
        };
        let reply = telegram_inbound::execute_backup_command(&sub_args);
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] Backup/Restore report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_email(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "email" || first_lower == "mail" {
        let sub_args = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            "status".to_string()
        };
        let reply = telegram_inbound::execute_email_command(&sub_args);
        println!("{}", reply);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &reply,
                ));
                println!(
                    "\n[SUCCESS] Email command report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_send(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "send" || first_lower == "notify" {
        let Some(chat_id) = t_cfg.allowed_chat_id else {
            eprintln!("[ERROR] Telegram chat ID is not configured.");
            return true;
        };
        if args.len() < 2 {
            eprintln!("Usage: agm telegram send \"<message>\"");
            return true;
        }
        let raw_msg = args[1..].join(" ");
        let formatted = format!(
            "🔔 <b>AGM Notification</b>\n\n{}",
            telegram_inbound::clean_for_telegram_html(&raw_msg, 3500)
        );
        match rt.block_on(telegram_inbound::send_telegram_message(
            &t_cfg.bot_token,
            chat_id,
            &formatted,
        )) {
            Ok(_) => println!(
                "[SUCCESS] Notification delivered to Telegram chat {}!",
                chat_id
            ),
            Err(e) => eprintln!("[ERROR] Failed to send notification: {}", e),
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_poll(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "poll" || first_lower == "watch" {
        let is_once = args.iter().any(|a| a == "--once" || a == "-1");
        println!("[*] Polling inbound Telegram updates...");
        loop {
            match rt.block_on(telegram_inbound::poll_telegram_updates_once()) {
                Ok(items) => {
                    for (cid, cmd_in, _reply) in &items {
                        println!("[+] Processed command '{}' from chat {}", cmd_in, cid);
                    }
                    if is_once {
                        println!("[*] Poll complete ({} command(s) processed).", items.len());
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Poll failed: {}", e);
                    if is_once {
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(t_cfg.poll_interval_secs.max(3)));
        }
        return true;
    }
    false
}

pub(crate) fn telegram_route_cmds(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "cmds" || first_lower == "commands" {
        println!("\n=== Supported Inbound Telegram Commands ===");
        println!("  /ping                 Verify node connectivity, IP, Git version & uptime");
        println!(
            "  /status, /observe     Inspect live workspaces, active account quota & prompt queues"
        );
        println!(
            "  /gitmap <args>        Execute GitMap CLI command (e.g. /gitmap pe, /gitmap version)"
        );
        println!("  /agm <args>           Execute AGM CLI command (e.g. /agm status, /agm accounts, /agm wpr)");
        println!("  /api                  Check local API proxy (port 8045) & account bindings");
        println!("  /backup, /backpack    Backup running prompts into split SQLite DB (/backup ls to list)");
        println!("  /restore              Restore backed-up prompts to resume execution");
        println!("  /email [status|ping]  Query email vault status or dispatch test/help email");
        println!(
            "  /ff                   Fast-forward switch to freshest highest-quota standby account"
        );
        println!("  /snapshot             Multi-node cluster status snapshot");
        println!("  /help                 Show full interactive remote command manual\n");
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let help_html = telegram_inbound::format_help_manual();
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &help_html,
                ));
            }
        }
        return true;
    }
    false
}
