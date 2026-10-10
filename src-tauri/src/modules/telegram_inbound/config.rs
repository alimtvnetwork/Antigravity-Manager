use crate::error::AppError;
use crate::modules::account;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::*;

/// Path to telegram_config.json
pub fn get_config_path() -> Result<PathBuf, AppError> {
    let data_dir = account::get_data_dir()
        .map_err(|e| AppError::Config(format!("Failed to get data dir: {}", e)))?;
    Ok(data_dir.join("telegram_config.json"))
}

/// Load configuration from disk
pub fn load_config() -> Result<TelegramConfig, AppError> {
    let path = get_config_path()?;
    if !path.exists() {
        let def = TelegramConfig::default();
        save_config(&def)?;
        return Ok(def);
    }
    let data = fs::read_to_string(&path).map_err(AppError::Io)?;
    let clean_data = data.trim_start_matches('\u{FEFF}');
    let config: TelegramConfig =
        match crate::modules::json_envelope::extract_payload::<TelegramConfig>(clean_data) {
            Ok((cfg, _)) => cfg,
            Err(_) => serde_json::from_str(clean_data)
                .map_err(|e| AppError::Config(format!("Failed to parse Telegram config: {}", e)))?,
        };
    Ok(config)
}

/// Export Telegram configuration as standard portable JSON envelope
pub fn export_config_json(config: &TelegramConfig) -> Result<String, AppError> {
    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/telegram-config", config.clone());
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| AppError::Config(format!("Failed to serialize Telegram config: {}", e)))
}

/// Save configuration to disk
pub fn save_config(config: &TelegramConfig) -> Result<(), AppError> {
    let path = get_config_path()?;
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("Failed to serialize Telegram config: {}", e)))?;
    fs::write(&path, data).map_err(AppError::Io)?;
    Ok(())
}

/// Test connection to Telegram Bot API by calling getMe and registering slash commands
pub async fn test_telegram_connection(bot_token: &str) -> Result<String, AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err(AppError::Config("Bot token cannot be empty".to_string()));
    }
    let url = format!("https://api.telegram.org/bot{}/getMe", clean_token);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let status = resp.status().as_u16();
    if status != 200 {
        return Err(AppError::Network(
            format!("Telegram Bot API test failed with status: {}", status),
            Some(status),
        ));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let username = body["result"]["username"]
        .as_str()
        .unwrap_or("UnknownBot")
        .to_string();

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        register_telegram_bot_commands(clean_token).await,
        "register_telegram_bot_commands",
    );

    Ok(username)
}

/// Register bot slash-commands menu in Telegram via setMyCommands
pub async fn register_telegram_bot_commands(bot_token: &str) -> Result<(), AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Ok(());
    }
    let url = format!("https://api.telegram.org/bot{}/setMyCommands", clean_token);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let payload = json!({
        "commands": [
            { "command": "tree", "description": "Project → Conversation → 200-word Prompt tree with AGM Seq IDs" },
            { "command": "active", "description": "List active running prompts & AGM tree view" },
            { "command": "nodes", "description": "List all cluster VM nodes & status" },
            { "command": "queues", "description": "Inspect workspace prompt queues" },
            { "command": "projects", "description": "List registered workspaces & prompt syntax" },
            { "command": "prompts", "description": "List reusable prompt templates" },
            { "command": "prompt", "description": "Inject prompt by Seq ID (C001/P001), instance, or node" },
            { "command": "ssh", "description": "List SSH nodes or run command on remote machine" },
            { "command": "update", "description": "Update AGM or GitMap CLI (e.g. /update agm, /update gitmap)" },
            { "command": "ping", "description": "Verify node connectivity, IP, Git version & uptime" },
            { "command": "status", "description": "Full node, account quota & proxy status" },
            { "command": "observe", "description": "Inspect live workspaces & running prompt queues" },
            { "command": "expand", "description": "Display full prompt instructions by ID (/expand <id>)" },
            { "command": "gitmap", "description": "Run GitMap CLI command (e.g. /gitmap agy active)" },
            { "command": "agm", "description": "Run AGM CLI command (e.g. /agm tree, /agm update)" },
            { "command": "api", "description": "Query local API proxy status & account bindings" },
            { "command": "backup", "description": "Backup running prompts to split SQLite DB" },
            { "command": "restore", "description": "Restore backed-up prompts to resume execution" },
            { "command": "prune", "description": "Safely prune older conversations (e.g. /prune 5, /prune 10)" },
            { "command": "query", "description": "Query cached SQLite prompts with ≥200-word preview" },
            { "command": "email", "description": "Check email status or send test/help email" },
            { "command": "ff", "description": "Fast-forward switch to highest-quota account" },
            { "command": "snapshot", "description": "View multi-node cluster status snapshot" },
            { "command": "help", "description": "Show full Telegram remote command reference" }
        ]
    });

    // Justification: best-effort send; failure logged without changing control flow
    crate::error::record_ignored(
        client.post(&url).json(&payload).send().await,
        "send via post",
    );
    Ok(())
}
