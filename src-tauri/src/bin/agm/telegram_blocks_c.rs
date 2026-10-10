//! telegram_blocks_c — Telegram subcommand handlers, split from agm.rs.

pub(crate) fn telegram_route_nodes(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "nodes" || first_lower == "cluster" {
        let report = rt.block_on(telegram_inbound::format_cluster_nodes_report());
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
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

pub(crate) fn telegram_route_active(
    args: &[String],
    first_lower: &str,
    t_cfg: &telegram_inbound::TelegramConfig,
    rt: &tokio::runtime::Runtime,
) -> bool {
    if first_lower == "active" || first_lower == "running" {
        let report = rt.block_on(telegram_inbound::format_active_prompts_report());
        println!("{}", report);
        if let Some(chat_id) = t_cfg.allowed_chat_id {
            if !t_cfg.bot_token.is_empty() {
                let _ = rt.block_on(telegram_inbound::send_telegram_message(
                    &t_cfg.bot_token,
                    chat_id,
                    &report,
                ));
                println!(
                    "\n[SUCCESS] Active prompts report delivered to Telegram chat {}!",
                    chat_id
                );
            }
        }
        return true;
    }
    false
}
