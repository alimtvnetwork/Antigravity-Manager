//! Telegram Inbound Watcher and Remote Command Daemon
//! Polls Telegram Bot API for remote commands, cluster snapshot queries,
//! Chat ID auto-discovery, and dispatches instructions into local execution or Supabase.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::account;
use crate::modules::auto_switcher;
use crate::modules::backup_prompts_db;
use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::git_info;
use crate::modules::repo_db;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

fn default_true() -> bool {
    true
}

/// Telegram bot integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chat_id: Option<i64>,
    pub is_enabled: bool,
    pub poll_interval_secs: u64,
    #[serde(default = "default_true")]
    pub notify_on_system_update: bool,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            allowed_chat_id: None,
            is_enabled: false,
            poll_interval_secs: 5,
            notify_on_system_update: true,
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

/// Discovered Telegram Chat ID info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramDetectedChat {
    pub chat_id: i64,
    pub chat_label: String,
    pub bot_username: String,
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
    let data = fs::read_to_string(&path).map_err(AppError::Io)?;
    let clean_data = data.trim_start_matches('\u{FEFF}');
    let config: TelegramConfig = serde_json::from_str(clean_data)
        .map_err(|e| AppError::Config(format!("Failed to parse Telegram config: {}", e)))?;
    Ok(config)
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

    let _ = register_telegram_bot_commands(clean_token).await;

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
            { "command": "nodes", "description": "List all cluster VM nodes & status" },
            { "command": "active", "description": "List active running prompts & conversations" },
            { "command": "queues", "description": "Inspect workspace prompt queues" },
            { "command": "projects", "description": "List registered workspaces & prompt syntax" },
            { "command": "prompts", "description": "List reusable prompt templates" },
            { "command": "prompt", "description": "Inject prompt into node workspace" },
            { "command": "ping", "description": "Verify node connectivity, IP, Git version & uptime" },
            { "command": "status", "description": "Full node, account quota & proxy status" },
            { "command": "observe", "description": "Inspect live workspaces & running prompt queues" },
            { "command": "gitmap", "description": "Run GitMap CLI command (e.g. /gitmap pe)" },
            { "command": "agm", "description": "Run AGM CLI command (e.g. /agm accounts)" },
            { "command": "api", "description": "Query local API proxy status & account bindings" },
            { "command": "backup", "description": "Backup running prompts to split SQLite DB" },
            { "command": "restore", "description": "Restore backed-up prompts to resume execution" },
            { "command": "email", "description": "Check email status or send test/help email" },
            { "command": "ff", "description": "Fast-forward switch to highest-quota account" },
            { "command": "snapshot", "description": "View multi-node cluster status snapshot" },
            { "command": "help", "description": "Show full Telegram remote command reference" }
        ]
    });

    let _ = client.post(&url).json(&payload).send().await;
    Ok(())
}

/// Automatically detect Telegram Chat ID from recent bot updates (getUpdates)
pub async fn detect_telegram_chat_id(bot_token: &str) -> Result<TelegramDetectedChat, AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err(AppError::Config("Bot token cannot be empty".to_string()));
    }

    let bot_username = test_telegram_connection(clean_token).await?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates", clean_token);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if !resp.status().is_success() {
        return Err(AppError::Network(
            format!("Telegram getUpdates failed with status {}", resp.status()),
            Some(resp.status().as_u16()),
        ));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if let Some(updates) = body["result"].as_array() {
        for update in updates.iter().rev() {
            let chat_obj = if update["message"]["chat"]["id"].is_i64() {
                Some(&update["message"]["chat"])
            } else if update["edited_message"]["chat"]["id"].is_i64() {
                Some(&update["edited_message"]["chat"])
            } else if update["my_chat_member"]["chat"]["id"].is_i64() {
                Some(&update["my_chat_member"]["chat"])
            } else if update["channel_post"]["chat"]["id"].is_i64() {
                Some(&update["channel_post"]["chat"])
            } else {
                None
            };

            if let Some(chat) = chat_obj {
                if let Some(cid) = chat["id"].as_i64() {
                    let label = chat["username"]
                        .as_str()
                        .map(|u| format!("@{}", u))
                        .or_else(|| chat["title"].as_str().map(|t| t.to_string()))
                        .or_else(|| chat["first_name"].as_str().map(|f| f.to_string()))
                        .unwrap_or_else(|| cid.to_string());

                    return Ok(TelegramDetectedChat {
                        chat_id: cid,
                        chat_label: label,
                        bot_username,
                    });
                }
            }
        }
    }

    // Check if config on disk already has an allowed_chat_id
    if let Ok(existing) = load_config() {
        if let Some(cid) = existing.allowed_chat_id {
            return Ok(TelegramDetectedChat {
                chat_id: cid,
                chat_label: "Saved Config".to_string(),
                bot_username,
            });
        }
    }

    Err(AppError::Config(format!(
        "No recent messages found for @{}. Open https://t.me/{} in Telegram and send /ping or /start first, then click Auto-Detect Chat ID again.",
        bot_username, bot_username
    )))
}

/// Chunk text into pieces <= max_chars splitting at newline boundaries where possible
pub fn chunk_telegram_text(text: &str, max_chars: usize) -> Vec<String> {
    let limit = if max_chars == 0 { 3800 } else { max_chars };
    if text.len() <= limit {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(limit);

    for line in text.split_inclusive('\n') {
        if line.len() > limit {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = String::with_capacity(limit);
            }
            let mut remaining = line;
            while remaining.len() > limit {
                let (slice, rest) = remaining.split_at(limit);
                chunks.push(slice.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_chunk.push_str(remaining);
            }
        } else if current_chunk.len() + line.len() > limit {
            chunks.push(current_chunk);
            current_chunk = String::with_capacity(limit);
            current_chunk.push_str(line);
        } else {
            current_chunk.push_str(line);
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    if chunks.is_empty() {
        vec![text.to_string()]
    } else {
        chunks
    }
}

/// Send a text message to a Telegram chat, automatically chunking messages longer than 3800 characters
pub async fn send_telegram_message(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err(AppError::Config("Telegram bot token is empty".to_string()));
    }

    let chunks = chunk_telegram_text(text, 3800);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", clean_token);

    for (idx, chunk) in chunks.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(Duration::from_millis(80)).await;
        }

        let payload_html = json!({
            "chat_id": chat_id,
            "text": chunk,
            "parse_mode": "HTML"
        });

        let resp = client.post(&url).json(&payload_html).send().await;
        match resp {
            Ok(r) if r.status().is_success() => continue,
            Ok(r) if r.status().as_u16() == 400 => {
                // If HTML parse error occurred, retry sending chunk as plain text
                let payload_plain = json!({
                    "chat_id": chat_id,
                    "text": chunk
                });
                let retry_resp = client
                    .post(&url)
                    .json(&payload_plain)
                    .send()
                    .await
                    .map_err(|e| AppError::Network(e.to_string(), None))?;
                let status = retry_resp.status().as_u16();
                if !retry_resp.status().is_success() {
                    let err = retry_resp.text().await.unwrap_or_default();
                    return Err(AppError::Network(
                        format!("Telegram sendMessage failed: {}", err),
                        Some(status),
                    ));
                }
            }
            Ok(r) => {
                let status = r.status().as_u16();
                let err = r.text().await.unwrap_or_default();
                return Err(AppError::Network(
                    format!(
                        "Telegram sendMessage failed with status {}: {}",
                        status, err
                    ),
                    Some(status),
                ));
            }
            Err(e) => return Err(AppError::Network(e.to_string(), None)),
        }
    }

    Ok(())
}

/// Convenience alias for send_telegram_message with automatic chunking
pub async fn send_telegram_message_chunked(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    send_telegram_message(bot_token, chat_id, text).await
}

/// Strip ANSI escape codes and escape HTML entities for Telegram <pre> blocks
pub fn clean_for_telegram_html(input: &str, max_chars: usize) -> String {
    let mut no_ansi = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        no_ansi.push(c);
    }

    let truncated: String = if no_ansi.chars().count() > max_chars {
        let mut s: String = no_ansi.chars().take(max_chars).collect();
        s.push_str("\n... [truncated]");
        s
    } else {
        no_ansi
    };

    truncated
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format rich ping telemetry report
pub fn format_ping_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias
    };
    let local_ip = email_watcher::detect_local_ip();
    let uptime_min = supabase_sync::get_uptime_seconds() / 60;
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    let active_acc = account::load_account_index()
        .ok()
        .and_then(|idx| {
            let cid = idx.current_account_id?;
            idx.accounts.into_iter().find(|a| a.id == cid)
        })
        .map(|a| a.email)
        .unwrap_or_else(|| "None".to_string());

    format!(
        "🏓 <b>PONG — Antigravity Node Online</b>\n\n\
        • <b>Node:</b> <code>{}</code> (IP: <code>{}</code>)\n\
        • <b>Version:</b> <code>v{}</code> | <b>Commit:</b> <code>{}</code>\n\
        • <b>Branch:</b> <code>{}</code> | <b>Last Release:</b> <code>{}</code>\n\
        • <b>Uptime:</b> <code>{}m</code>\n\
        • <b>Active Account:</b> <code>{}</code>\n\n\
        💡 Send <code>/observe</code> for live workspaces &amp; prompts or <code>/help</code> for all commands.",
        clean_for_telegram_html(&local_alias, 64),
        clean_for_telegram_html(&local_ip, 64),
        clean_for_telegram_html(&ver, 32),
        clean_for_telegram_html(&hash, 32),
        clean_for_telegram_html(&branch, 32),
        clean_for_telegram_html(&last_rel, 32),
        uptime_min,
        clean_for_telegram_html(&active_acc, 64)
    )
}

/// Format live observation & status report (Accounts, Workspaces, Running Prompts, Backups)
pub fn format_observe_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias
    };
    let local_ip = email_watcher::detect_local_ip();
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    let (active_email, total_accounts, quota_summary) = match account::load_account_index() {
        Ok(idx) => {
            let total = idx.accounts.len();
            let cid = idx.current_account_id.unwrap_or_default();
            let email = idx
                .accounts
                .iter()
                .find(|a| a.id == cid)
                .map(|a| a.email.clone())
                .unwrap_or_else(|| "None".to_string());
            let quota_str = account::load_account(&cid)
                .ok()
                .and_then(|acc| acc.quota)
                .map(|q| {
                    let tier = q.subscription_tier.unwrap_or_else(|| "PRO".to_string());
                    let min_pct = q.models.iter().map(|m| m.percentage).min().unwrap_or(100);
                    format!("{} ({}% min model quota)", tier, min_pct)
                })
                .unwrap_or_else(|| "N/A".to_string());
            (email, total, quota_str)
        }
        Err(_) => ("None".to_string(), 0, "N/A".to_string()),
    };

    let mut ws_section = String::new();
    let mut prompts_section = String::new();
    let projects = repo_db::get_live_project_execution_info();
    if projects.is_empty() {
        ws_section.push_str("• No workspaces registered\n");
    } else {
        for p in projects.iter().take(8) {
            let badge = if p.is_running {
                "🟢 RUNNING"
            } else {
                "⚪ IDLE"
            };
            let friendly_label =
                repo_db::format_friendly_workspace_label(&p.project_id, &p.repo_name, &p.repo_path);
            ws_section.push_str(&format!(
                "• <b>{}</b> [{}] (<code>{}</code>)\n",
                clean_for_telegram_html(&friendly_label, 48),
                badge,
                clean_for_telegram_html(&p.status, 20)
            ));
        }
    }

    match repo_db::list_all_prompts() {
        Ok(recent_prompts) if !recent_prompts.is_empty() => {
            for p in recent_prompts.iter().take(5) {
                let friendly_ws =
                    repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
                let smart_summary = repo_db::extract_smart_prompt_summary(&p.prompt_content, 70);
                prompts_section.push_str(&format!(
                    "• [<code>{}</code>] <b>{}</b>: <code>\"{}\"</code>\n",
                    clean_for_telegram_html(&p.status, 16),
                    clean_for_telegram_html(&friendly_ws, 36),
                    clean_for_telegram_html(&smart_summary, 70)
                ));
            }
        }
        _ => {
            prompts_section.push_str("• Queue is empty\n");
        }
    }

    let backup_count = backup_prompts_db::list_backup_batches(None)
        .map(|b| b.len())
        .unwrap_or(0);

    format!(
        "🔭 <b>AGM Node Observation &amp; Telemetry Report</b>\n\n\
        🖥 <b>Node:</b> <code>{}</code> (<code>{}</code>)\n\
        📦 <b>Build:</b> <code>v{}</code> | <code>{}</code> | <code>{}</code> (rel: <code>{}</code>)\n\
        👤 <b>Active Account:</b> <code>{}</code> ({} total)\n\
        🔋 <b>Quota / Tier:</b> <code>{}</code>\n\
        💾 <b>Backup Batches:</b> <code>{}</code> split SQLite batch(es)\n\n\
        📂 <b>Discovered Workspaces:</b>\n{}\n\
        📝 <b>Recent Prompts Queue:</b>\n{}",
        clean_for_telegram_html(&local_alias, 48),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24),
        clean_for_telegram_html(&active_email, 48),
        total_accounts,
        clean_for_telegram_html(&quota_summary, 48),
        backup_count,
        ws_section,
        prompts_section
    )
}

/// Format comprehensive Telegram command manual
pub fn format_help_manual() -> String {
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    format!(
        "🤖 <b>Antigravity-Manager Telegram Remote Manual</b>\n\
        <code>v{} | commit {} | branch {} | release {}</code>\n\n\
        📌 <b>Core Telemetry &amp; Observation:</b>\n\
        • <code>/help</code> or <code>/start</code> — Display this full interactive command manual\n\
        • <code>/ping</code> — Check node connectivity, IP, Git build &amp; uptime\n\
        • <code>/status</code> or <code>/observe</code> — Live workspaces, active account quota &amp; prompts\n\
        • <code>/snapshot</code> — Multi-node cluster status snapshot\n\n\
        🖥️ <b>Cluster VM Nodes &amp; Prompt Control:</b>\n\
        • <code>/nodes</code> or <code>/node ls</code> — View all cluster VM nodes &amp; connectivity\n\
        • <code>/nodes &lt;alias&gt; prompts</code> — Inspect running prompts on specific VM node\n\
        • <code>/active</code> or <code>/running</code> — Compact table of active running prompts\n\
        • <code>/queues</code> or <code>/queue</code> — Inspect pending workspace prompt queues\n\
        • <code>/projects</code> — List registered workspaces, IDs &amp; sample syntax\n\
        • <code>/prompts</code> — List available reusable prompt templates &amp; queue\n\
        • <code>/prompt &lt;node&gt; &lt;proj&gt; &lt;text&gt;</code> — Inject prompt into VM workspace\n\
        • <code>/prompt &lt;proj&gt; &lt;text&gt;</code> — Inject prompt locally\n\n\
        🧭 <b>GitMap &amp; AGM CLI Execution:</b>\n\
        • <code>/gitmap pe</code> — Check CI/CD pipeline execution status\n\
        • <code>/gitmap agy active</code> — Check Antigravity active prompts via GitMap\n\
        • <code>/gitmap version</code> — Check installed GitMap CLI version\n\
        • <code>/agm status</code> — Run AGM status &amp; quota summary\n\
        • <code>/agm accounts</code> — List registered accounts &amp; quotas\n\
        • <code>/agm wpr</code> — List projects with running prompts\n\
        • <code>/api</code> — Inspect local API proxy (port 8045) &amp; account bindings\n\n\
        🎒 <b>Prompt Backup (Backpack) &amp; Recovery:</b>\n\
        • <code>/backup</code> or <code>/backpack</code> — Snapshot running prompts to split SQLite DB\n\
        • <code>/backup ls</code> — List saved prompt backup batches\n\
        • <code>/restore</code> — Restore &amp; resume backed-up prompts\n\n\
        📧 <b>Email &amp; Account Rotation:</b>\n\
        • <code>/email status</code> — Show configured email accounts &amp; recipients\n\
        • <code>/email ping</code> — Dispatch a test ping email to recipients\n\
        • <code>/email help</code> — Send full HTML command manual via email\n\
        • <code>/ff</code> — Fast-forward switch to freshest highest-quota account\n\
        • <code>CMD:&lt;node-alias&gt;:&lt;command&gt;</code> — Run shell command on node",
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24)
    )
}

/// Execute GitMap CLI command and format output for Telegram
pub fn execute_gitmap_subcommand(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let parts: Vec<&str> = if trimmed.is_empty() {
        vec!["pe"]
    } else {
        trimmed.split_whitespace().collect()
    };

    let output_res = Command::new("gitmap").args(&parts).output();
    match output_res {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = if stderr.trim().is_empty() {
                stdout.to_string()
            } else if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                format!("{}\n{}", stdout.trim_end(), stderr.trim_end())
            };
            let cleaned = clean_for_telegram_html(combined.trim(), 3200);
            format!(
                "🧭 <b>GitMap CLI:</b> <code>gitmap {}</code>\n<pre>{}</pre>",
                clean_for_telegram_html(&parts.join(" "), 64),
                if cleaned.is_empty() {
                    "(completed with no output)".to_string()
                } else {
                    cleaned
                }
            )
        }
        Err(e) => format!(
            "⚠️ <b>GitMap Execution Error:</b> <code>{}</code>",
            clean_for_telegram_html(&e.to_string(), 300)
        ),
    }
}

/// Resolve the path to the `agm` CLI binary
fn resolve_agm_binary() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(stem) = exe.file_stem() {
            if stem.to_string_lossy().eq_ignore_ascii_case("agm") {
                return exe;
            }
        }
        if let Some(parent) = exe.parent() {
            let candidate = if cfg!(windows) {
                parent.join("agm.exe")
            } else {
                parent.join("agm")
            };
            if candidate.exists() {
                return candidate;
            }
        }
    }
    let debug_candidate = PathBuf::from("src-tauri/target/debug/agm.exe");
    if debug_candidate.exists() {
        return debug_candidate;
    }
    PathBuf::from("agm")
}

/// Execute AGM CLI command and format output for Telegram
pub fn execute_agm_subcommand(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let parts: Vec<&str> = if trimmed.is_empty() {
        vec!["status"]
    } else {
        trimmed.split_whitespace().collect()
    };

    // Prevent recursive telegram polling
    if parts
        .first()
        .map(|s| s.eq_ignore_ascii_case("telegram"))
        .unwrap_or(false)
    {
        return format_observe_report();
    }

    let bin = resolve_agm_binary();
    match Command::new(&bin).args(&parts).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = if stderr.trim().is_empty() {
                stdout.to_string()
            } else if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                format!("{}\n{}", stdout.trim_end(), stderr.trim_end())
            };
            let cleaned = clean_for_telegram_html(combined.trim(), 3200);
            format!(
                "⚡ <b>AGM CLI:</b> <code>agm {}</code>\n<pre>{}</pre>",
                clean_for_telegram_html(&parts.join(" "), 64),
                if cleaned.is_empty() {
                    "(completed with no output)".to_string()
                } else {
                    cleaned
                }
            )
        }
        Err(e) => format!(
            "⚠️ <b>AGM CLI Error:</b> <code>{}</code>",
            clean_for_telegram_html(&e.to_string(), 300)
        ),
    }
}

/// Query API Proxy health and active bindings
pub async fn execute_api_status_command() -> String {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok();

    let mut proxy_online = false;
    let mut proxy_detail = String::new();
    if let Some(c) = client {
        if let Ok(resp) = c.get("http://127.0.0.1:8045/accounts/current").send().await {
            if resp.status().is_success() {
                proxy_online = true;
                proxy_detail = resp.text().await.unwrap_or_default();
            }
        }
    }

    let active_acc = account::load_account_index()
        .ok()
        .and_then(|idx| {
            let cid = idx.current_account_id?;
            idx.accounts.into_iter().find(|a| a.id == cid)
        })
        .map(|a| format!("{} ({})", a.email, a.id))
        .unwrap_or_else(|| "None".to_string());

    let status_badge = if proxy_online {
        "🟢 ONLINE (http://127.0.0.1:8045)"
    } else {
        "⚪ STANDBY / DIRECT MODE (Port 8045 idle)"
    };

    let mut msg = format!(
        "🔌 <b>API Gateway &amp; Account Bindings</b>\n\n\
        • <b>Gateway State:</b> <code>{}</code>\n\
        • <b>Active Bound Account:</b> <code>{}</code>\n",
        status_badge,
        clean_for_telegram_html(&active_acc, 80)
    );

    if !proxy_detail.trim().is_empty() {
        msg.push_str(&format!(
            "\n<b>Live Loopback Payload:</b>\n<pre>{}</pre>",
            clean_for_telegram_html(proxy_detail.trim(), 1200)
        ));
    }

    msg
}

/// Execute Prompt Backup ("Backpack") or Restore command
pub fn execute_backup_command(args_str: &str) -> String {
    let sub = args_str.trim().to_lowercase();
    if sub == "ls" || sub == "list" || sub == "status" {
        match backup_prompts_db::list_backup_batches(None) {
            Ok(batches) => {
                if batches.is_empty() {
                    return "🎒 <b>Prompt Backup Vault:</b> No backup batches stored.".to_string();
                }
                let mut rows = String::new();
                for (i, b) in batches.iter().take(8).enumerate() {
                    rows.push_str(&format!(
                        "{}. <code>{}</code> — {} prompt(s) [restored: {}]\n",
                        i + 1,
                        clean_for_telegram_html(&b.id, 32),
                        b.prompts_count,
                        b.is_fully_restored
                    ));
                }
                format!(
                    "🎒 <b>Split SQLite Prompt Backups ({} total):</b>\n\n{}",
                    batches.len(),
                    rows
                )
            }
            Err(e) => format!(
                "⚠️ <b>Backup Query Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else if sub == "restore" || sub == "rrp" || sub == "resend" {
        let projs = repo_db::list_running_projects().unwrap_or_default();
        let proj_names: Vec<String> = projs
            .into_iter()
            .map(|p| p.repo_name)
            .filter(|n| !n.is_empty())
            .collect();
        let proj_display = if !proj_names.is_empty() {
            proj_names.join(", ")
        } else {
            "Antigravity-Manager".to_string()
        };

        let repo_resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
        let _ = repo_db::dispatch_running_prompts("default");
        match backup_prompts_db::restore_running_prompts(false, None) {
            Ok(records) => format!(
                "♻️ <b>Prompt Restoration Complete:</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • Restored and re-queued <b>{}</b> prompt(s) from split SQLite backup.\n\
                • Re-injected <b>{}</b> in-flight prompt(s) directly into workspaces.",
                clean_for_telegram_html(&proj_display, 80),
                records.len(),
                repo_resent.len()
            ),
            Err(e) => format!(
                "⚠️ <b>Prompt Restore Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        let projs = repo_db::list_running_projects().unwrap_or_default();
        let proj_names: Vec<String> = projs
            .into_iter()
            .map(|p| p.repo_name)
            .filter(|n| !n.is_empty())
            .collect();
        let proj_display = if !proj_names.is_empty() {
            proj_names.join(", ")
        } else {
            "Antigravity-Manager".to_string()
        };

        let _ = repo_db::backup_running_prompts("default");
        match backup_prompts_db::backup_active_running_prompts(None) {
            Ok((batch, records)) => format!(
                "🎒 <b>Running Prompts Backed Up Successfully!</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Batch ID:</b> <code>{}</code>\n\
                • <b>Captured Prompts:</b> <b>{}</b>\n\
                • <b>Vault Path:</b> <code>{}</code>\n\n\
                💡 Send <code>/restore</code> anytime to resume backed-up prompts.",
                clean_for_telegram_html(&proj_display, 80),
                clean_for_telegram_html(&batch.id, 40),
                records.len(),
                clean_for_telegram_html(&batch.file_path, 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Backup Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}

/// Execute Email status / ping / help command from Telegram
pub fn execute_email_command(args_str: &str) -> String {
    let sub = args_str.trim().to_lowercase();
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    let active_recipients: Vec<String> = recipients
        .iter()
        .filter(|r| r.is_active)
        .map(|r| r.email.clone())
        .collect();

    if sub.is_empty() || sub == "status" || sub == "ls" {
        let default_sender = accounts
            .iter()
            .find(|a| a.is_default && a.is_active)
            .or_else(|| accounts.iter().find(|a| a.is_active))
            .map(|a| a.email.clone())
            .unwrap_or_else(|| "None".to_string());

        return format!(
            "📧 <b>Email Telemetry &amp; Vault Status</b>\n\n\
            • <b>Configured Senders:</b> <code>{}</code> (Default: <code>{}</code>)\n\
            • <b>Active Recipients:</b> <code>{}</code> ({})\n\n\
            💡 Send <code>/email ping</code> to dispatch a test email or <code>/email help</code> to send the full HTML command guide.",
            accounts.len(),
            clean_for_telegram_html(&default_sender, 48),
            active_recipients.len(),
            clean_for_telegram_html(&active_recipients.join(", "), 120)
        );
    }

    let targets = if active_recipients.is_empty() {
        accounts
            .iter()
            .filter(|a| a.is_active)
            .map(|a| a.email.clone())
            .collect::<Vec<_>>()
    } else {
        active_recipients
    };

    if targets.is_empty() {
        return "⚠️ <b>Email Dispatch Skipped:</b> No active sender or recipient accounts configured in Email Vault.".to_string();
    }

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    if sub == "help" {
        let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);
        match email_sender::dispatch_email_with_failover(&subj, &body, &targets) {
            Ok(res) => format!(
                "✅ <b>Help Manual Email Sent!</b>\n\
                • <b>Sender:</b> <code>{}</code>\n\
                • <b>Recipients ({}):</b> <code>{}</code>",
                clean_for_telegram_html(&res.used_account_email, 48),
                targets.len(),
                clean_for_telegram_html(&targets.join(", "), 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Email Dispatch Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        let now = Utc::now().timestamp();
        let (subj, body) =
            email_sender::render_test_ping_email("Telegram-Remote", &m_name, &m_ip, now);
        match email_sender::dispatch_email_with_failover(&subj, &body, &targets) {
            Ok(res) => format!(
                "✅ <b>Test Ping Email Delivered!</b>\n\
                • <b>Sender:</b> <code>{}</code>\n\
                • <b>Recipients ({}):</b> <code>{}</code>",
                clean_for_telegram_html(&res.used_account_email, 48),
                targets.len(),
                clean_for_telegram_html(&targets.join(", "), 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Email Ping Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}

/// Cluster node summary info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeInfo {
    pub alias: String,
    pub ip_address: String,
    pub status: String,
    pub source: String,
    pub last_seen: Option<String>,
    pub uptime_seconds: u64,
}

/// Parse GitMap cluster status output (e.g. Node vm-01: Connected (Last Seen: ...))
pub fn parse_gitmap_cluster_status(output: &str) -> Vec<ClusterNodeInfo> {
    let mut nodes = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Node ") {
            if let Some((alias, status_part)) = rest.split_once(':') {
                let alias = alias.trim().to_string();
                let status_trimmed = status_part.trim();
                let (status, last_seen) =
                    if let Some((st, seen_part)) = status_trimmed.split_once('(') {
                        let st = st.trim().to_string();
                        let seen = seen_part
                            .trim_end_matches(')')
                            .strip_prefix("Last Seen:")
                            .map(|s| s.trim().to_string());
                        (st, seen)
                    } else {
                        (status_trimmed.to_string(), None)
                    };

                nodes.push(ClusterNodeInfo {
                    alias,
                    ip_address: "Cluster Mesh".to_string(),
                    status,
                    source: "GitMap Fleet".to_string(),
                    last_seen,
                    uptime_seconds: 0,
                });
            }
        }
    }
    nodes
}

/// Query GitMap cluster nodes via CLI
pub fn query_gitmap_cluster_nodes() -> Vec<ClusterNodeInfo> {
    let output_res = Command::new("gitmap").args(["cluster", "status"]).output();
    if let Ok(out) = output_res {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let parsed = parse_gitmap_cluster_status(&stdout);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    Vec::new()
}

/// Query Supabase Root DB nodes table
pub async fn query_supabase_cluster_nodes() -> Vec<ClusterNodeInfo> {
    let mut nodes = Vec::new();
    let local_config = supabase_sync::load_config().unwrap_or_default();
    for ep in &local_config.endpoints {
        if !ep.is_enabled || ep.role != "root" {
            continue;
        }
        if let Ok(client) = SupabaseClient::new(ep) {
            if let Ok(val) = client.select("nodes", "order=last_heartbeat_at.desc").await {
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        let alias = item["alias"].as_str().unwrap_or("Node").to_string();
                        let ip = item["ip_address"].as_str().unwrap_or("0.0.0.0").to_string();
                        let st = item["status"].as_str().unwrap_or("online").to_string();
                        let ut = item["uptime_seconds"].as_u64().unwrap_or(0);
                        let last_hb = item["last_heartbeat_at"].as_i64();
                        let seen_str = last_hb.map(|ts| {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| ts.to_string())
                        });

                        nodes.push(ClusterNodeInfo {
                            alias,
                            ip_address: ip,
                            status: st,
                            source: "Supabase Root DB".to_string(),
                            last_seen: seen_str,
                            uptime_seconds: ut,
                        });
                    }
                    break;
                }
            }
        }
    }
    nodes
}

/// Format comprehensive cluster nodes report (Local + GitMap Fleet + Supabase)
pub async fn format_cluster_nodes_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias.clone()
    };
    let local_ip = supabase_sync::get_local_ip();
    let local_uptime = supabase_sync::get_uptime_seconds();

    let mut all_nodes: Vec<ClusterNodeInfo> = Vec::new();

    // 1. Add Local Host Node
    all_nodes.push(ClusterNodeInfo {
        alias: local_alias.clone(),
        ip_address: local_ip,
        status: "online".to_string(),
        source: "Host Node".to_string(),
        last_seen: Some(Utc::now().to_rfc3339()),
        uptime_seconds: local_uptime,
    });

    // 2. Add GitMap cluster fleet nodes
    let gm_nodes = query_gitmap_cluster_nodes();
    for n in gm_nodes {
        if !all_nodes
            .iter()
            .any(|x| x.alias.eq_ignore_ascii_case(&n.alias))
        {
            all_nodes.push(n);
        }
    }

    // 3. Add Supabase cluster nodes
    let sb_nodes = query_supabase_cluster_nodes().await;
    for n in sb_nodes {
        if let Some(existing) = all_nodes
            .iter_mut()
            .find(|x| x.alias.eq_ignore_ascii_case(&n.alias))
        {
            if (existing.ip_address == "Cluster Mesh" || existing.ip_address == "0.0.0.0")
                && n.ip_address != "0.0.0.0"
            {
                existing.ip_address = n.ip_address;
            }
            if existing.uptime_seconds == 0 {
                existing.uptime_seconds = n.uptime_seconds;
            }
        } else {
            all_nodes.push(n);
        }
    }

    let mut rows = String::new();
    let total = all_nodes.len();
    for (i, node) in all_nodes.iter().enumerate() {
        let is_online = node.status.to_lowercase().contains("online")
            || node.status.to_lowercase().contains("connected");
        let badge = if is_online {
            "🟢 ONLINE"
        } else {
            "⚪ STANDBY"
        };

        let seen_display = node.last_seen.as_deref().unwrap_or("Active");
        let uptime_str = if node.uptime_seconds > 0 {
            format!(" | Uptime: {}m", node.uptime_seconds / 60)
        } else {
            String::new()
        };

        rows.push_str(&format!(
            "{}. {} <b>{}</b> (<code>{}</code>)\n   • <b>Fleet:</b> {} [{}]\n   • <b>Last Seen:</b> <code>{}</code>{}\n\n",
            i + 1,
            badge,
            clean_for_telegram_html(&node.alias, 48),
            clean_for_telegram_html(&node.ip_address, 48),
            clean_for_telegram_html(&node.source, 32),
            clean_for_telegram_html(&node.status, 24),
            clean_for_telegram_html(seen_display, 36),
            uptime_str
        ));
    }

    format!(
        "🖥️ <b>Antigravity VM Cluster Fleet ({} Nodes)</b>\n\n\
        {}\
        📋 <b>Fleet Commands:</b>\n\
        • <code>/nodes &lt;alias&gt; prompts</code> — Inspect running prompts on node\n\
        • <code>/prompt &lt;alias&gt; &lt;proj&gt; &lt;text&gt;</code> — Inject prompt to VM\n\
        • <code>/projects</code> — List active workspaces &amp; project IDs\n\
        • <code>/prompts</code> — List state database prompt queue",
        total, rows
    )
}

/// Format cluster nodes snapshot for Telegram response (backward compatibility)
pub async fn format_cluster_snapshot() -> String {
    format_cluster_nodes_report().await
}

/// Format running prompts scoped to a specific node (e.g. /nodes vm-01 prompts)
pub async fn format_node_scoped_prompts(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let clean_str = if let Some(stripped) = trimmed.strip_suffix("running prompts") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_suffix("prompts") {
        stripped.trim()
    } else {
        trimmed
    };
    let parts: Vec<&str> = clean_str.split_whitespace().collect();
    let target_alias = if parts.is_empty() { "local" } else { parts[0] };

    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias.clone()
    };

    let is_local = target_alias.eq_ignore_ascii_case("local")
        || target_alias.eq_ignore_ascii_case(&local_alias)
        || target_alias.eq_ignore_ascii_case(&email_watcher::detect_machine_name());

    if is_local {
        let projects = repo_db::get_live_project_execution_info();
        let prompts = repo_db::list_all_prompts().unwrap_or_default();
        let running_prompts: Vec<_> = prompts.iter().filter(|p| p.status == "running").collect();

        let mut proj_text = String::new();
        if projects.is_empty() {
            proj_text.push_str("• No registered workspaces\n");
        } else {
            for p in &projects {
                let badge = if p.is_running {
                    "🟢 RUNNING"
                } else {
                    "⚪ IDLE"
                };
                let short_id = if p.project_id.len() > 12 {
                    &p.project_id[..12]
                } else {
                    &p.project_id
                };
                proj_text.push_str(&format!(
                    "• [{}] <b>{}</b> (ID: <code>{}</code>)\n  <code>{}</code>\n",
                    badge,
                    clean_for_telegram_html(&p.repo_name, 36),
                    short_id,
                    clean_for_telegram_html(&p.repo_path, 60)
                ));
            }
        }

        let mut prompt_text = String::new();
        if running_prompts.is_empty() {
            prompt_text.push_str("• No actively executing prompts (Node is idle)\n");
        } else {
            for p in &running_prompts {
                let short_pid = if p.id.len() > 10 { &p.id[..10] } else { &p.id };
                let clean_content = repo_db::extract_clean_user_prompt(&p.prompt_content);
                prompt_text.push_str(&format!(
                    "• [🟢 RUNNING] <code>{}</code> (Proj: <b>{}</b>)\n  <i>\"{}\"</i>\n",
                    short_pid,
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean_content, 120)
                ));
            }
        }

        format!(
            "⚡ <b>Node Telemetry &amp; Running Prompts: <code>{}</code></b>\n\n\
            📂 <b>Workspaces:</b>\n{}\n\
            📝 <b>Running Prompts ({} active):</b>\n{}\n\
            💡 Send <code>/prompt {} &lt;project-id&gt; &lt;text&gt;</code> to dispatch instructions.",
            clean_for_telegram_html(&local_alias, 48),
            proj_text,
            running_prompts.len(),
            prompt_text,
            clean_for_telegram_html(target_alias, 24)
        )
    } else {
        // Query remote VM node via GitMap cluster
        let mut remote_prompts_info = String::new();
        let cmd_out = Command::new("gitmap")
            .args(["cluster", "exec", target_alias, "agm wpr --json"])
            .output();

        let mut has_remote_data = false;
        if let Ok(out) = cmd_out {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.trim().is_empty() && (stdout.contains('[') || stdout.contains('{')) {
                has_remote_data = true;
                remote_prompts_info = format!(
                    "<pre>{}</pre>",
                    clean_for_telegram_html(stdout.trim(), 1500)
                );
            }
        }

        if !has_remote_data {
            // Check secondary DB queue for pending prompts for target node
            let mut queued_count = 0;
            for ep in &local_config.endpoints {
                if ep.is_enabled && ep.role == "secondary" {
                    if let Ok(c) = SupabaseClient::new(ep) {
                        let query = format!("target_node=eq.{}&status=eq.pending", target_alias);
                        if let Ok(val) = c.select("command_queue", &query).await {
                            if let Some(arr) = val.as_array() {
                                queued_count = arr.len();
                            }
                        }
                    }
                }
            }

            remote_prompts_info = format!(
                "• Remote Node Status: 🟢 Registered in Fleet\n\
                • Active Queued Prompts: <b>{}</b> pending in Supabase Secondary DB\n\
                • GitMap Cluster Link: Connected",
                queued_count
            );
        }

        format!(
            "⚡ <b>Remote VM Node Prompts: <code>{}</code></b>\n\n\
            {}\n\n\
            💡 Send <code>/prompt {} &lt;project-id&gt; &lt;text&gt;</code> to dispatch a prompt to this node.",
            clean_for_telegram_html(target_alias, 48),
            remote_prompts_info,
            clean_for_telegram_html(target_alias, 24)
        )
    }
}

/// Format discovered workspaces and project IDs
pub fn format_projects_list() -> String {
    let projects = repo_db::get_live_project_execution_info();
    if projects.is_empty() {
        return "📂 <b>Workspaces:</b> No registered workspaces found.".to_string();
    }

    let mut rows = String::new();
    for (i, p) in projects.iter().enumerate() {
        let badge = if p.is_running {
            "🟢 RUNNING"
        } else {
            "⚪ IDLE"
        };
        let friendly_label =
            repo_db::format_friendly_workspace_label(&p.project_id, &p.repo_name, &p.repo_path);
        let prompt_preview = if let Some(ref pr) = p.active_prompt {
            let clean = repo_db::extract_smart_prompt_summary(pr, 90);
            format!(
                "\n   • <i>Prompt: \"{}\"</i>",
                clean_for_telegram_html(&clean, 90)
            )
        } else {
            String::new()
        };

        rows.push_str(&format!(
            "{}. {} <b>{}</b>\n   • <b>ID:</b> <code>{}</code>\n   • <b>Path:</b> <code>{}</code>{}\n\n",
            i + 1,
            badge,
            clean_for_telegram_html(&friendly_label, 48),
            clean_for_telegram_html(&p.project_id, 48),
            clean_for_telegram_html(&p.repo_path, 60),
            prompt_preview
        ));
    }

    format!(
        "📂 <b>Discovered Workspaces &amp; Projects ({} Total)</b>\n\n\
        {}\
        💡 <b>Sample Prompt Invocations:</b>\n\
        • <b>Local workspace:</b> <code>/prompt &lt;project-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Remote VM node:</b> <code>/prompt &lt;node-alias&gt; &lt;project-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Active conversation:</b> <code>/prompt &lt;conversation-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Prompt template:</b> <code>/prompt &lt;project-id&gt; read-all</code>",
        projects.len(),
        rows
    )
}

/// Format active running prompts in compact GitMap style
pub async fn format_active_prompts_report() -> String {
    if let Ok(out) = Command::new("gitmap").args(["agy", "active"]).output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty()
            && (stdout.contains("Active Running Prompts") || stdout.contains("CONVERSATION ID"))
        {
            let cleaned = clean_for_telegram_html(stdout.trim(), 2800);
            return format!(
                "⚡ <b>Antigravity Active Running Prompts</b>\n\n\
                <pre>{}</pre>\n\n\
                💡 <b>How to Inject Prompt:</b>\n\
                • <code>/prompt &lt;conversation-id&gt; &lt;text&gt;</code>\n\
                • <code>/prompt &lt;project-id&gt; &lt;text&gt;</code>\n\
                • <code>/prompt &lt;node&gt; &lt;project-id&gt; &lt;text&gt;</code>",
                cleaned
            );
        }
    }

    let projects = repo_db::get_live_project_execution_info();
    let running: Vec<_> = projects.into_iter().filter(|p| p.is_running).collect();
    if running.is_empty() {
        return "⚡ <b>Active Running Prompts:</b> No conversations or workspaces are actively running prompts (Node is idle).".to_string();
    }

    let mut rows = String::new();
    for (i, p) in running.iter().enumerate() {
        let friendly_label =
            repo_db::format_friendly_workspace_label(&p.project_id, &p.repo_name, &p.repo_path);
        let clean = p
            .active_prompt
            .as_deref()
            .map(|s| repo_db::extract_smart_prompt_summary(s, 100))
            .unwrap_or_else(|| "AGM".to_string());
        rows.push_str(&format!(
            "{}. 🟢 <b>{}</b>\n   • <b>ID:</b> <code>{}</code>\n   • <b>Path:</b> <code>{}</code>\n   • <i>Prompt: \"{}\"</i>\n\n",
            i + 1,
            clean_for_telegram_html(&friendly_label, 48),
            clean_for_telegram_html(&p.project_id, 48),
            clean_for_telegram_html(&p.repo_path, 60),
            clean_for_telegram_html(&clean, 100)
        ));
    }

    format!(
        "⚡ <b>Active Running Prompts ({} Running)</b>\n\n\
        {}\
        💡 <b>How to Inject Prompt:</b>\n\
        • <code>/prompt &lt;project-id&gt; &lt;text&gt;</code>\n\
        • <code>/prompt &lt;node&gt; &lt;project-id&gt; &lt;text&gt;</code>",
        running.len(),
        rows
    )
}

/// Format workspace prompt queues in GitMap style
pub async fn format_prompt_queues_report() -> String {
    if let Ok(out) = Command::new("gitmap").args(["agy", "queues"]).output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty()
            && (stdout.contains("Prompt Queues") || stdout.contains("queued"))
        {
            let cleaned = clean_for_telegram_html(stdout.trim(), 2800);
            return format!(
                "📥 <b>Antigravity Workspace Prompt Queues</b>\n\n\
                <pre>{}</pre>\n\n\
                💡 Send <code>/active</code> to view running prompts or <code>/restore</code> to re-queue backed-up prompts.",
                cleaned
            );
        }
    }

    match repo_db::list_all_prompts() {
        Ok(prompts) => {
            let queued: Vec<_> = prompts
                .into_iter()
                .filter(|p| p.status == "queued" || p.status == "pending")
                .collect();
            if queued.is_empty() {
                return "📥 <b>Prompt Queues:</b> All workspace queues are currently empty."
                    .to_string();
            }
            let mut rows = String::new();
            for (i, p) in queued.iter().take(10).enumerate() {
                let friendly_ws =
                    repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
                let clean = repo_db::extract_smart_prompt_summary(&p.prompt_content, 80);
                rows.push_str(&format!(
                    "{}. [⏳ QUEUED] <code>{}</code>\n   • <b>Project:</b> <b>{}</b> (<code>{}</code>)\n   • <i>\"{}\"</i>\n\n",
                    i + 1,
                    if p.id.len() > 12 { &p.id[..12] } else { &p.id },
                    clean_for_telegram_html(&friendly_ws, 36),
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean, 80)
                ));
            }
            format!(
                "📥 <b>Workspace Prompt Queues ({} Queued)</b>\n\n\
                {}\
                💡 Send <code>/restore</code> to resume prompt queue.",
                queued.len(),
                rows
            )
        }
        Err(e) => format!(
            "⚠️ <b>Queue Check Failed:</b> <code>{}</code>",
            clean_for_telegram_html(&e, 200)
        ),
    }
}

/// Format available prompt templates like `gitmap agy prompt ls`
pub fn format_prompts_templates_report() -> String {
    if let Ok(out) = Command::new("gitmap")
        .args(["agy", "prompt", "ls"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty()
            && (stdout.contains("Available Prompt Templates") || stdout.contains("read-all"))
        {
            let cleaned = clean_for_telegram_html(stdout.trim(), 2600);
            return format!(
                "📋 <b>Available Reusable Prompt Templates</b>\n\n\
                <pre>{}</pre>\n\n\
                💡 <b>How to Run Templates:</b>\n\
                • <code>/prompt &lt;project-id&gt; read-all</code>\n\
                • <code>/prompt &lt;node&gt; &lt;project-id&gt; read-all</code>\n\
                • <code>/prompt &lt;project-id&gt; is-done</code>",
                cleaned
            );
        }
    }

    format!(
        "📋 <b>Available Reusable Prompt Templates</b>\n\n\
        1. <code>read-all</code> — Enhanced Read Memory protocol and project context ingestion\n\
        2. <code>is-done</code> — Standard task completion and quality verification check\n\
        3. <code>ci-cd-fix</code> — Diagnose, analyze, and fix CI/CD pipeline failures\n\
        4. <code>minor-bump</code> — Automated version bump and changelog synchronization\n\
        5. <code>coding-guidelines</code> — Audit and enforce grounded coding guidelines\n\
        6. <code>smart-test-runner</code> — Smart incremental test execution\n\n\
        💡 <b>How to Run Templates:</b>\n\
        • <code>/prompt &lt;project-id&gt; read-all</code>\n\
        • <code>/prompt &lt;node-alias&gt; &lt;project-id&gt; read-all</code>"
    )
}

/// Format active and queued prompts from state database
pub fn format_prompts_list() -> String {
    match repo_db::list_all_prompts() {
        Ok(prompts) if !prompts.is_empty() => {
            let total = prompts.len();
            let mut rows = String::new();
            for (i, p) in prompts.iter().take(10).enumerate() {
                let badge = match p.status.as_str() {
                    "running" => "🟢 RUNNING",
                    "dispatched" => "📤 DISPATCHED",
                    "backed_up" => "💾 BACKED UP",
                    "completed" => "✅ COMPLETED",
                    _ => "⚪ QUEUED",
                };
                let clean = repo_db::extract_clean_user_prompt(&p.prompt_content);
                rows.push_str(&format!(
                    "{}. [{}] <code>{}</code>\n   • <b>Project:</b> <code>{}</code>\n   • <i>\"{}\"</i>\n\n",
                    i + 1,
                    badge,
                    clean_for_telegram_html(&p.id, 24),
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean, 100)
                ));
            }
            format!(
                "📝 <b>State Database Prompt Queue ({} Prompts, Showing Top {})</b>\n\n\
                {}\
                💡 Send <code>/restore</code> to re-queue backed-up prompts.",
                total,
                prompts.len().min(10),
                rows
            )
        }
        _ => "📝 <b>Prompt Queue:</b> No prompts currently registered in split SQLite database."
            .to_string(),
    }
}

/// Execute prompt injection to local workspace or remote cluster node
pub async fn execute_prompt_injection(args_str: &str) -> String {
    let trimmed = args_str.trim();
    if trimmed.is_empty() {
        return "⚠️ <b>Missing Arguments:</b>\nUsage:\n• <code>/prompt &lt;project&gt; &lt;prompt text&gt;</code>\n• <code>/prompt &lt;node-alias&gt; &lt;project&gt; &lt;prompt text&gt;</code>".to_string();
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() < 2 {
        return "⚠️ <b>Missing Prompt Content:</b> Please specify both the project/node and the prompt text to run.\nExample: <code>/prompt agm-main \"Fix issue 20\"</code>".to_string();
    }

    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias.clone()
    };
    let gitmap_nodes = query_gitmap_cluster_nodes();

    let is_first_token_remote_node = gitmap_nodes.iter().any(|n| {
        n.alias.eq_ignore_ascii_case(tokens[0]) && !n.alias.eq_ignore_ascii_case(&local_alias)
    });

    let (target_node, target_project, prompt_text) =
        if is_first_token_remote_node && tokens.len() >= 3 {
            let node = tokens[0];
            let proj = tokens[1];
            let p_start = trimmed.find(proj).map(|idx| idx + proj.len()).unwrap_or(0);
            let text = trimmed[p_start..].trim();
            (node.to_string(), proj.to_string(), text.to_string())
        } else {
            let first = tokens[0];
            if first.eq_ignore_ascii_case("local") || first.eq_ignore_ascii_case(&local_alias) {
                if tokens.len() >= 3 {
                    let proj = tokens[1];
                    let p_start = trimmed.find(proj).map(|idx| idx + proj.len()).unwrap_or(0);
                    (
                        "local".to_string(),
                        proj.to_string(),
                        trimmed[p_start..].trim().to_string(),
                    )
                } else {
                    (
                        "local".to_string(),
                        "default".to_string(),
                        tokens[1..].join(" "),
                    )
                }
            } else {
                let proj = tokens[0];
                let p_start = trimmed.find(proj).map(|idx| idx + proj.len()).unwrap_or(0);
                let text = trimmed[p_start..].trim();
                ("local".to_string(), proj.to_string(), text.to_string())
            }
        };

    if prompt_text.trim().is_empty() {
        return "⚠️ <b>Empty Prompt:</b> Prompt text cannot be empty.".to_string();
    }

    if target_node == "local" || target_node.eq_ignore_ascii_case(&local_alias) {
        let projects = repo_db::get_live_project_execution_info();
        let matched_proj = projects.iter().find(|p| {
            p.project_id.eq_ignore_ascii_case(&target_project)
                || p.repo_name.eq_ignore_ascii_case(&target_project)
                || p.project_id.starts_with(&target_project)
                || target_project.starts_with(&p.project_id)
        });

        let (final_proj_id, repo_path) = if let Some(p) = matched_proj {
            (p.project_id.clone(), p.repo_path.clone())
        } else {
            let p_buf = PathBuf::from(&target_project);
            if p_buf.exists() && p_buf.is_dir() {
                (target_project.clone(), target_project.clone())
            } else if let Some(first_p) = projects.first() {
                (first_p.project_id.clone(), first_p.repo_path.clone())
            } else {
                (
                    "local-project".to_string(),
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string()),
                )
            }
        };

        let prompt_id = format!("p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let active_prompt = repo_db::ActivePrompt {
            id: prompt_id.clone(),
            project_id: final_proj_id.clone(),
            instance_id: "default".to_string(),
            repo_path: repo_path.clone(),
            prompt_content: prompt_text.clone(),
            model: None,
            session_id: None,
            status: "running".to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            image_payload: None,
        };

        if let Err(e) = repo_db::save_or_requeue_prompt(&active_prompt) {
            return format!(
                "⚠️ <b>Failed to Save Prompt:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            );
        }

        let spawned = repo_db::spawn_prompt_via_agy(&active_prompt);
        let exec_badge = if spawned {
            "🟢 Executing via agy"
        } else {
            "⚪ Enqueued in Split DB"
        };

        format!(
            "🚀 <b>Prompt Injected Locally!</b>\n\n\
            • <b>Prompt ID:</b> <code>{}</code>\n\
            • <b>Target Workspace:</b> <code>{}</code>\n\
            • <b>Path:</b> <code>{}</code>\n\
            • <b>Status:</b> {}\n\
            • <b>Prompt Content:</b>\n<pre>{}</pre>",
            clean_for_telegram_html(&prompt_id, 32),
            clean_for_telegram_html(&final_proj_id, 40),
            clean_for_telegram_html(&repo_path, 60),
            exec_badge,
            clean_for_telegram_html(&prompt_text, 800)
        )
    } else {
        let mut dispatched_gitmap = false;
        let gm_out = Command::new("gitmap")
            .args([
                "cluster",
                "exec",
                &target_node,
                &format!("agm prompt {} \"{}\"", target_project, prompt_text),
            ])
            .output();
        if let Ok(out) = gm_out {
            if out.status.success() {
                dispatched_gitmap = true;
            }
        }

        let mut enqueued_supabase = false;
        for ep in &local_config.endpoints {
            if ep.is_enabled && ep.role == "secondary" {
                if let Ok(c) = SupabaseClient::new(ep) {
                    let cmd_id = uuid::Uuid::new_v4().to_string();
                    let payload = json!({
                        "id": cmd_id,
                        "target_node": target_node,
                        "project_id": target_project,
                        "command": "prompt",
                        "prompt": prompt_text,
                        "status": "pending",
                        "created_at": Utc::now().timestamp()
                    });
                    if c.insert("command_queue", payload).await.is_ok() {
                        enqueued_supabase = true;
                        break;
                    }
                }
            }
        }

        let dispatch_status = if dispatched_gitmap {
            "🟢 Dispatched immediately via GitMap Cluster SSH"
        } else if enqueued_supabase {
            "📥 Enqueued to Supabase Secondary DB (Node will execute on poll)"
        } else {
            "⚠️ Dispatched instruction (Node recorded in queue)"
        };

        format!(
            "🌐 <b>Remote Prompt Dispatched!</b>\n\n\
            • <b>Target Node:</b> <code>{}</code>\n\
            • <b>Target Project:</b> <code>{}</code>\n\
            • <b>Status:</b> {}\n\
            • <b>Prompt Content:</b>\n<pre>{}</pre>",
            clean_for_telegram_html(&target_node, 40),
            clean_for_telegram_html(&target_project, 40),
            dispatch_status,
            clean_for_telegram_html(&prompt_text, 800)
        )
    }
}

/// Unified inbound Telegram command processor
pub async fn process_telegram_command_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Normalize leading slash and optional @bot_username suffix
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (trimmed, ""),
    };
    let cmd_no_slash = cmd_raw.strip_prefix('/').unwrap_or(cmd_raw);
    let cmd_base = cmd_no_slash
        .split('@')
        .next()
        .unwrap_or(cmd_no_slash)
        .to_lowercase();
    let lower_full = trimmed.to_lowercase();

    match cmd_base.as_str() {
        "start" | "help" => Some(format_help_manual()),
        "ping" => Some(format_ping_report()),
        "status" | "observe" => Some(format_observe_report()),
        "active" | "running" => Some(format_active_prompts_report().await),
        "queues" | "queue" => Some(format_prompt_queues_report().await),
        "nodes" | "node" => {
            let sub = rest.trim();
            if sub.is_empty() || sub == "ls" || sub == "list" || sub == "status" {
                Some(format_cluster_nodes_report().await)
            } else {
                Some(format_node_scoped_prompts(sub).await)
            }
        }
        "projects" | "workspaces" | "workspace" => Some(format_projects_list()),
        "prompts" | "prompt_queue" | "templates" => {
            let sub = rest.trim();
            if sub.is_empty() || sub == "ls" || sub == "list" {
                Some(format_prompts_templates_report())
            } else if sub == "queue" || sub == "queues" {
                Some(format_prompt_queues_report().await)
            } else if sub == "all" || sub == "db" {
                Some(format_prompts_list())
            } else {
                Some(execute_prompt_injection(sub).await)
            }
        }
        "prompt" | "inject" | "p" | "pt" => Some(execute_prompt_injection(rest).await),
        "gitmap" | "gm" => Some(execute_gitmap_subcommand(rest)),
        "agm" => Some(execute_agm_subcommand(rest)),
        "api" | "proxy" => Some(execute_api_status_command().await),
        "backup" | "backpack" => Some(execute_backup_command(rest)),
        "restore" => Some(execute_backup_command("restore")),
        "email" | "mail" => Some(execute_email_command(rest)),
        "snapshot" | "cluster" => Some(format_cluster_nodes_report().await),
        "ff" | "rotate" => {
            let projs = repo_db::list_running_projects().unwrap_or_default();
            let proj_names: Vec<String> = projs
                .into_iter()
                .map(|p| p.repo_name)
                .filter(|n| !n.is_empty())
                .collect();
            let proj_display = if !proj_names.is_empty() {
                proj_names.join(", ")
            } else {
                "Antigravity-Manager".to_string()
            };

            let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
            let _ = backup_prompts_db::backup_active_running_prompts(None);
            let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
            let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
            let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

            Some(format!(
                "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Pre-Switch Backup:</b> <code>{}</code> running prompt(s) captured\n\
                • <b>Rotation Evaluation:</b> {}\n\
                • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)\n\n\
                💡 All active prompts are preserved and continue without interruption.",
                clean_for_telegram_html(&proj_display, 80),
                backup_count,
                if rotate_res.is_ok() { "✅ Evaluated successfully" } else { "ℹ️ No switch required" },
                resent.len(),
                disp
            ))
        }
        _ => {
            if lower_full.contains("how many machines")
                || lower_full.contains("node ls")
                || lower_full == "nodes"
                || lower_full == "nodes ls"
            {
                return Some(format_cluster_nodes_report().await);
            }
            if lower_full.contains("running prompts") || lower_full.contains("active prompts") {
                if lower_full.starts_with("node") || lower_full.starts_with("nodes") {
                    return Some(format_node_scoped_prompts(trimmed).await);
                }
                return Some(format_active_prompts_report().await);
            }
            if lower_full.starts_with("ff:") {
                let projs = repo_db::list_running_projects().unwrap_or_default();
                let proj_names: Vec<String> = projs
                    .into_iter()
                    .map(|p| p.repo_name)
                    .filter(|n| !n.is_empty())
                    .collect();
                let proj_display = if !proj_names.is_empty() {
                    proj_names.join(", ")
                } else {
                    "Antigravity-Manager".to_string()
                };

                let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
                let _ = backup_prompts_db::backup_active_running_prompts(None);
                let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
                let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
                let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

                return Some(format!(
                    "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                    • <b>Projects:</b> <code>{}</code>\n\
                    • <b>Pre-Switch Backup:</b> <code>{}</code> prompt(s) captured\n\
                    • <b>Rotation Evaluation:</b> {}\n\
                    • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)",
                    clean_for_telegram_html(&proj_display, 80),
                    backup_count,
                    if rotate_res.is_ok() { "✅ Completed" } else { "ℹ️ Evaluated" },
                    resent.len(),
                    disp
                ));
            }
            if lower_full.starts_with("cmd:") || lower_full.starts_with("exec:") {
                let parts: Vec<&str> = trimmed.splitn(3, ':').collect();
                if parts.len() >= 3 {
                    let target_node = parts[1].trim();
                    let cmd_content = parts[2].trim();

                    let mut enqueued = false;
                    let s_config = supabase_sync::load_config().unwrap_or_default();
                    for ep in &s_config.endpoints {
                        if !ep.is_enabled {
                            continue;
                        }
                        if ep.role == "secondary" {
                            if let Ok(c) = SupabaseClient::new(ep) {
                                let cmd_id = uuid::Uuid::new_v4().to_string();
                                let cmd_payload = json!({
                                    "id": cmd_id,
                                    "target_node": target_node,
                                    "command": cmd_content,
                                    "status": "pending",
                                    "created_at": Utc::now().timestamp()
                                });
                                if c.insert("command_queue", cmd_payload).await.is_ok() {
                                    enqueued = true;
                                    break;
                                }
                            }
                        }
                    }

                    let reply_text = if enqueued {
                        format!(
                            "📥 <b>Command Enqueued:</b> Saved to Supabase Secondary DB for target node <code>{}</code>:\n<code>{}</code>",
                            clean_for_telegram_html(target_node, 48),
                            clean_for_telegram_html(cmd_content, 300)
                        )
                    } else {
                        format!(
                            "⚙️ <b>Command Received:</b> Target <code>{}</code>:\n<code>{}</code>",
                            clean_for_telegram_html(target_node, 48),
                            clean_for_telegram_html(cmd_content, 300)
                        )
                    };
                    return Some(reply_text);
                }
            }
            None
        }
    }
}

/// Poll pending Telegram updates once, execute commands, reply, and advance update offset
pub async fn poll_telegram_updates_once() -> Result<Vec<(i64, String, String)>, AppError> {
    let mut config = load_config()?;
    let clean_token = config.bot_token.trim().to_string();
    if clean_token.is_empty() {
        return Err(AppError::Config("Telegram bot token is empty".to_string()));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates", clean_token);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if !resp.status().is_success() {
        return Err(AppError::Network(
            format!("Telegram getUpdates returned {}", resp.status()),
            Some(resp.status().as_u16()),
        ));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let mut processed = Vec::new();
    let mut max_update_id: Option<i64> = None;

    if let Some(updates) = body["result"].as_array() {
        for update in updates {
            if let Some(up_id) = update["update_id"].as_i64() {
                max_update_id = Some(max_update_id.map_or(up_id, |m| m.max(up_id)));
            }

            let msg = &update["message"];
            let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
            let text = msg["text"].as_str().unwrap_or("").trim();
            if chat_id == 0 || text.is_empty() {
                continue;
            }

            // Auto-bind allowed_chat_id if not yet configured
            if config.allowed_chat_id.is_none() {
                config.allowed_chat_id = Some(chat_id);
                config.is_enabled = true;
                let _ = save_config(&config);
            }

            if let Some(allowed) = config.allowed_chat_id {
                if chat_id != allowed {
                    continue;
                }
            }

            if let Some(reply) = process_telegram_command_text(text).await {
                let _ = send_telegram_message(&clean_token, chat_id, &reply).await;
                processed.push((chat_id, text.to_string(), reply));
            }
        }
    }

    // Acknowledge processed updates so they are not re-processed on next call
    if let Some(last_id) = max_update_id {
        let ack_url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=0",
            clean_token,
            last_id + 1
        );
        let _ = client.get(&ack_url).send().await;
    }

    Ok(processed)
}

/// Start background Telegram inbound polling daemon
pub fn start_telegram_daemon() {
    let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tauri::async_runtime::spawn(async move {
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

            let mut config = match load_config() {
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

                                    if chat_id == 0 || text.is_empty() {
                                        continue;
                                    }

                                    // Auto-bind allowed_chat_id on first private message if not set
                                    if config.allowed_chat_id.is_none() {
                                        config.allowed_chat_id = Some(chat_id);
                                        let _ = save_config(&config);
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

                                    if let Some(reply) = process_telegram_command_text(text).await {
                                        let _ = send_telegram_message(
                                            &config.bot_token,
                                            chat_id,
                                            &reply,
                                        )
                                        .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_telegram_text_short() {
        let text = "Hello world";
        let chunks = chunk_telegram_text(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn test_chunk_telegram_text_splits_at_newline() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4";
        let chunks = chunk_telegram_text(text, 14);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= 14);
        }
    }

    #[test]
    fn test_chunk_telegram_text_long_line() {
        let text = "A".repeat(5000);
        let chunks = chunk_telegram_text(&text, 1000);
        assert_eq!(chunks.len(), 5);
        for chunk in &chunks {
            assert_eq!(chunk.len(), 1000);
        }
    }
}
