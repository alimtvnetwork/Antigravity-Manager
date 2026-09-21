//! Telegram Inbound Watcher and Remote Command Daemon
//! Polls Telegram Bot API for remote commands, cluster snapshot queries,
//! and dispatches instructions into Supabase or local execution.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::account;
use crate::modules::auto_switcher;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Telegram bot integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chat_id: Option<i64>,
    pub is_enabled: bool,
    pub poll_interval_secs: u64,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            allowed_chat_id: None,
            is_enabled: false,
            poll_interval_secs: 5,
        }
    }
}

/// Status telemetry for Telegram background daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramWatcherStatus {
    pub is_running: bool,
    pub last_poll_at: i64,
    pub last_update_id: i64,
    pub last_message_received: Option<String>,
    pub error_message: Option<String>,
}

static TELEGRAM_RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static LAST_TELEGRAM_STATUS: Lazy<Arc<RwLock<TelegramWatcherStatus>>> = Lazy::new(|| {
    Arc::new(RwLock::new(TelegramWatcherStatus {
        is_running: false,
        last_poll_at: 0,
        last_update_id: 0,
        last_message_received: None,
        error_message: None,
    }))
});

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
    let data = fs::read_to_string(&path).map_err(|e| AppError::Io(e))?;
    let config: TelegramConfig = serde_json::from_str(&data)
        .map_err(|e| AppError::Config(format!("Failed to parse Telegram config: {}", e)))?;
    Ok(config)
}

/// Save configuration to disk
pub fn save_config(config: &TelegramConfig) -> Result<(), AppError> {
    let path = get_config_path()?;
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("Failed to serialize Telegram config: {}", e)))?;
    fs::write(&path, data).map_err(|e| AppError::Io(e))?;
    Ok(())
}

/// Test connection to Telegram Bot API by calling getMe
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

    Ok(username)
}

/// Send a text message to a Telegram chat
pub async fn send_telegram_message(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        bot_token.trim()
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let payload = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML"
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let status = resp.status().as_u16();
    if status < 400 {
        return Ok(());
    }

    let err_body = resp.text().await.unwrap_or_default();
    Err(AppError::Network(
        format!(
            "Telegram sendMessage failed with status {}: {}",
            status, err_body
        ),
        Some(status),
    ))
}

/// Format cluster nodes snapshot for Telegram response
pub async fn format_cluster_snapshot() -> String {
    let local_node_id = supabase_sync::get_local_node_id();
    let local_ip = supabase_sync::get_local_ip();
    let uptime_min = supabase_sync::get_uptime_seconds() / 60;
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = local_config.node_alias;

    let mut nodes_text = String::new();
    let mut online_count = 1;

    // Try reading cluster nodes from Supabase Root DB if configured
    let mut is_cluster_found = false;
    for ep in &local_config.endpoints {
        if !ep.is_enabled {
            continue;
        }
        if ep.role == "root" {
            if let Ok(client) = SupabaseClient::new(ep) {
                if let Ok(val) = client
                    .select("nodes", "status=eq.online&order=last_heartbeat_at.desc")
                    .await
                {
                    if let Some(arr) = val.as_array() {
                        online_count = arr.len();
                        for item in arr {
                            let alias = item["alias"].as_str().unwrap_or("Node");
                            let ip = item["ip_address"].as_str().unwrap_or("0.0.0.0");
                            let ut = item["uptime_seconds"].as_u64().unwrap_or(0) / 60;
                            nodes_text.push_str(&format!(
                                "• <b>{}</b> (IP: <code>{}</code>) | Uptime: {}m\n",
                                alias, ip, ut
                            ));
                        }
                        is_cluster_found = true;
                        break;
                    }
                }
            }
        }
    }

    if !is_cluster_found {
        nodes_text.push_str(&format!(
            "• <b>{}</b> (IP: <code>{}</code>) | Uptime: {}m [Local]\n",
            local_alias, local_ip, uptime_min
        ));
    }

    format!(
        "🌐 <b>Antigravity Cluster Snapshot</b>\n\n\
        Currently Online Machines: <b>{}</b>\n\n\
        {}\n\
        📋 <b>Remote Command Formats:</b>\n\
        • <code>CMD:&lt;node-alias&gt;:&lt;command&gt;</code> (Execute PowerShell/Bash)\n\
        • <code>FF</code> or <code>FF:&lt;node-alias&gt;</code> (Fast-Forward Workspace)\n\
        • <code>SNAPSHOT</code> (Refresh machine list)",
        online_count, nodes_text
    )
}

/// Start background Telegram inbound polling daemon
pub fn start_telegram_daemon() {
    let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tokio::spawn(async move {
        let mut last_update_id = 0i64;

        loop {
            let is_enabled_flag = {
                let config = load_config().unwrap_or_default();
                config.is_enabled && !config.bot_token.trim().is_empty()
            };

            if !is_enabled_flag {
                {
                    let mut st = LAST_TELEGRAM_STATUS.write().await;
                    st.is_running = false;
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            {
                let mut st = LAST_TELEGRAM_STATUS.write().await;
                st.is_running = true;
                st.last_poll_at = Utc::now().timestamp();
            }

            let config = match load_config() {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let poll_url = format!(
                "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=20",
                config.bot_token.trim(),
                last_update_id + 1
            );

            let http_client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
            {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let resp_result = http_client.get(&poll_url).send().await;
            match resp_result {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(body) = resp.json::<Value>().await {
                            if let Some(updates) = body["result"].as_array() {
                                for update in updates {
                                    if let Some(up_id) = update["update_id"].as_i64() {
                                        if up_id > last_update_id {
                                            last_update_id = up_id;
                                        }
                                    }

                                    let msg = &update["message"];
                                    let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
                                    let text = msg["text"].as_str().unwrap_or("").trim();

                                    if text.is_empty() {
                                        continue;
                                    }

                                    // Filter by allowed_chat_id if set
                                    if let Some(allowed) = config.allowed_chat_id {
                                        if chat_id != allowed {
                                            continue;
                                        }
                                    }

                                    {
                                        let mut st = LAST_TELEGRAM_STATUS.write().await;
                                        st.last_message_received = Some(text.to_string());
                                        st.last_update_id = last_update_id;
                                    }

                                    let lower_text = text.to_lowercase();

                                    // 1. Cluster Snapshot Query
                                    if lower_text.contains("how many machines")
                                        || lower_text == "/snapshot"
                                        || lower_text == "snapshot"
                                        || lower_text == "/cluster"
                                    {
                                        let snapshot_msg = format_cluster_snapshot().await;
                                        let _ = send_telegram_message(
                                            &config.bot_token,
                                            chat_id,
                                            &snapshot_msg,
                                        )
                                        .await;
                                        continue;
                                    }

                                    // 2. Fast Forward Workspace Command
                                    if lower_text == "ff"
                                        || lower_text.starts_with("ff:")
                                        || lower_text == "/ff"
                                    {
                                        let _ = auto_switcher::check_and_rotate_if_needed();
                                        let reply = "⏩ <b>Fast-Forward Triggered:</b> Workspace profile rotated to next highest credit account.";
                                        let _ = send_telegram_message(
                                            &config.bot_token,
                                            chat_id,
                                            reply,
                                        )
                                        .await;
                                        continue;
                                    }

                                    // 3. Command Execution Injection (CMD:<node>:<command>)
                                    if lower_text.starts_with("cmd:")
                                        || lower_text.starts_with("exec:")
                                    {
                                        let parts: Vec<&str> = text.splitn(3, ':').collect();
                                        if parts.len() >= 3 {
                                            let target_node = parts[1].trim();
                                            let cmd_content = parts[2].trim();

                                            // Enqueue to Supabase Secondary DB if available
                                            let mut enqueued = false;
                                            let s_config =
                                                supabase_sync::load_config().unwrap_or_default();
                                            for ep in &s_config.endpoints {
                                                if !ep.is_enabled {
                                                    continue;
                                                }
                                                if ep.role == "secondary" {
                                                    if let Ok(c) = SupabaseClient::new(ep) {
                                                        let cmd_id =
                                                            uuid::Uuid::new_v4().to_string();
                                                        let cmd_payload = json!({
                                                            "id": cmd_id,
                                                            "target_node": target_node,
                                                            "command": cmd_content,
                                                            "status": "pending",
                                                            "created_at": Utc::now().timestamp()
                                                        });
                                                        if c.insert("command_queue", cmd_payload)
                                                            .await
                                                            .is_ok()
                                                        {
                                                            enqueued = true;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }

                                            let reply_text = if enqueued {
                                                format!(
                                                    "📥 <b>Command Enqueued:</b> Saved to Supabase Secondary DB for target node <code>{}</code>:\n<code>{}</code>",
                                                    target_node, cmd_content
                                                )
                                            } else {
                                                format!(
                                                    "⚙️ <b>Command Received:</b> Target <code>{}</code>:\n<code>{}</code>",
                                                    target_node, cmd_content
                                                )
                                            };

                                            let _ = send_telegram_message(
                                                &config.bot_token,
                                                chat_id,
                                                &reply_text,
                                            )
                                            .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut st = LAST_TELEGRAM_STATUS.write().await;
                    st.error_message = Some(e.to_string());
                }
            }

            tokio::time::sleep(Duration::from_secs(config.poll_interval_secs.max(2))).await;
        }
    });
}

/// Stop background Telegram polling daemon
pub fn stop_telegram_daemon() {
    TELEGRAM_RUNNING.store(false, Ordering::SeqCst);
}

/// Get latest Telegram watcher status
pub async fn get_telegram_status() -> TelegramWatcherStatus {
    LAST_TELEGRAM_STATUS.read().await.clone()
}
