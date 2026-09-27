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
    let config: TelegramConfig = serde_json::from_str(&data)
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
        .timeout(Duration::from_secs(15))
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
            ws_section.push_str(&format!(
                "• <b>{}</b> [{}] (<code>{}</code>)\n",
                clean_for_telegram_html(&p.repo_name, 40),
                badge,
                clean_for_telegram_html(&p.status, 20)
            ));
        }
    }

    match repo_db::list_all_prompts() {
        Ok(recent_prompts) if !recent_prompts.is_empty() => {
            for p in recent_prompts.iter().take(5) {
                prompts_section.push_str(&format!(
                    "• [<code>{}</code>] <b>{}</b>: <code>{}</code>\n",
                    clean_for_telegram_html(&p.status, 16),
                    clean_for_telegram_html(&p.project_id, 28),
                    clean_for_telegram_html(&p.prompt_content, 48)
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
        • <code>/ping</code> — Check node connectivity, IP, Git build &amp; uptime\n\
        • <code>/status</code> or <code>/observe</code> — Live workspaces, active account quota &amp; prompts\n\
        • <code>/snapshot</code> — Multi-node cluster status snapshot\n\n\
        🧭 <b>GitMap &amp; AGM CLI Execution:</b>\n\
        • <code>/gitmap pe</code> — Check CI/CD pipeline execution status\n\
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
    } else if sub == "restore" {
        match backup_prompts_db::restore_running_prompts(false, None) {
            Ok(records) => format!(
                "♻️ <b>Prompt Restoration Complete:</b>\n\
                Restored and re-queued <b>{}</b> prompt(s) from split SQLite backup.",
                records.len()
            ),
            Err(e) => format!(
                "⚠️ <b>Prompt Restore Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        match backup_prompts_db::backup_active_running_prompts(None) {
            Ok((batch, records)) => format!(
                "🎒 <b>Running Prompts Backed Up Successfully!</b>\n\n\
                • <b>Batch ID:</b> <code>{}</code>\n\
                • <b>Captured Prompts:</b> <b>{}</b>\n\
                • <b>Vault Path:</b> <code>{}</code>\n\n\
                💡 Send <code>/restore</code> anytime to resume backed-up prompts.",
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

/// Format cluster nodes snapshot for Telegram response
pub async fn format_cluster_snapshot() -> String {
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
        • <code>/ping</code> or <code>/observe</code> (Live node &amp; prompt telemetry)\n\
        • <code>/gitmap pe</code> or <code>/agm status</code> (CLI commands)\n\
        • <code>/backup</code> or <code>/restore</code> (Split SQLite prompt vault)\n\
        • <code>/email ping</code> or <code>/ff</code> (Email &amp; fast-forward)\n\
        • <code>CMD:&lt;node-alias&gt;:&lt;command&gt;</code> (Execute PowerShell/Bash)",
        online_count, nodes_text
    )
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
        "gitmap" | "gm" => Some(execute_gitmap_subcommand(rest)),
        "agm" => Some(execute_agm_subcommand(rest)),
        "api" | "proxy" => Some(execute_api_status_command().await),
        "backup" | "backpack" => Some(execute_backup_command(rest)),
        "restore" => Some(execute_backup_command("restore")),
        "email" | "mail" => Some(execute_email_command(rest)),
        "snapshot" | "cluster" => Some(format_cluster_snapshot().await),
        "ff" | "rotate" => {
            let _ = auto_switcher::check_and_rotate_if_needed();
            Some("⏩ <b>Fast-Forward Triggered:</b> Checked live quota and rotated workspace profile if needed.".to_string())
        }
        _ => {
            if lower_full.contains("how many machines") {
                return Some(format_cluster_snapshot().await);
            }
            if lower_full.starts_with("ff:") {
                let _ = auto_switcher::check_and_rotate_if_needed();
                return Some(
                    "⏩ <b>Fast-Forward Triggered:</b> Workspace profile rotation evaluated."
                        .to_string(),
                );
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
