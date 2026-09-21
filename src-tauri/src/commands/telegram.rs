//! Tauri IPC Commands for Telegram Inbound Remote Watcher

#![allow(dead_code)]

use crate::error::AppResult;
use crate::modules::telegram_inbound::{self, TelegramConfig, TelegramWatcherStatus};

#[tauri::command]
pub async fn get_telegram_config() -> AppResult<TelegramConfig> {
    telegram_inbound::load_config()
}

#[tauri::command]
pub async fn save_telegram_config(config: TelegramConfig) -> AppResult<()> {
    let is_enabled = config.is_enabled;
    telegram_inbound::save_config(&config)?;

    if is_enabled {
        telegram_inbound::start_telegram_daemon();
    } else {
        telegram_inbound::stop_telegram_daemon();
    }
    Ok(())
}

#[tauri::command]
pub async fn test_telegram_bot(bot_token: String) -> AppResult<String> {
    telegram_inbound::test_telegram_connection(&bot_token).await
}

#[tauri::command]
pub async fn get_telegram_status() -> AppResult<TelegramWatcherStatus> {
    Ok(telegram_inbound::get_telegram_status().await)
}

#[tauri::command]
pub async fn send_telegram_test_message(bot_token: String, chat_id: i64) -> AppResult<()> {
    let msg = "🚀 <b>Antigravity Manager:</b> Telegram bot connection test successful!";
    telegram_inbound::send_telegram_message(&bot_token, chat_id, msg).await
}
