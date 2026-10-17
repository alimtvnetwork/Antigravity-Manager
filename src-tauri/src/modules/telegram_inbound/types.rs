use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::*;

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_card_palette() -> String {
    "riseup".to_string()
}

pub(crate) fn default_node_name_refresh_hours() -> u64 {
    24
}

pub(crate) fn default_prompt_summary_words() -> usize {
    100
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
    /// Card palette id: riseup, obsidian, gold, indigo.
    #[serde(default = "default_card_palette")]
    pub card_palette: String,
    /// Hours before a generated Node-* display name is resolved through GitMap again.
    #[serde(default = "default_node_name_refresh_hours")]
    pub node_name_refresh_hours: u64,
    /// Word limit for previous project prompt summaries (e.g. 100 or 200).
    #[serde(default = "default_prompt_summary_words")]
    pub prompt_summary_words: usize,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            allowed_chat_id: None,
            is_enabled: false,
            poll_interval_secs: 5,
            notify_on_system_update: true,
            card_palette: default_card_palette(),
            node_name_refresh_hours: default_node_name_refresh_hours(),
            prompt_summary_words: default_prompt_summary_words(),
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

pub(crate) static TELEGRAM_RUNNING: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

pub(crate) static LAST_TELEGRAM_STATUS: Lazy<Arc<RwLock<TelegramWatcherStatus>>> =
    Lazy::new(|| {
        Arc::new(RwLock::new(TelegramWatcherStatus {
            is_running: false,
            last_poll_at: 0,
            last_update_id: 0,
            last_message_received: None,
            error_message: None,
        }))
    });
