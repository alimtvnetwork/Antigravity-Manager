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

/// Execution status for inbound prompts injected into AGM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptExecutionStatusType {
    ExecutingNow,
    Queued,
}

impl PromptExecutionStatusType {
    #[inline]
    pub fn from_spawned(is_spawned: bool) -> Self {
        if is_spawned {
            Self::ExecutingNow
        } else {
            Self::Queued
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExecutingNow => "executing_now",
            Self::Queued => "queued",
        }
    }

    #[inline]
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::ExecutingNow => "Executing now",
            Self::Queued => "Queued",
        }
    }

    #[inline]
    pub fn is_executing(&self) -> bool {
        matches!(self, Self::ExecutingNow)
    }

    #[inline]
    pub fn is_queued(&self) -> bool {
        matches!(self, Self::Queued)
    }
}

impl std::fmt::Display for PromptExecutionStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_label())
    }
}

/// Remote cluster dispatch resolution status for cross-node commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteDispatchStatusType {
    GitMapClusterSsh,
    SupabaseQueued,
    LocalRecorded,
}

impl RemoteDispatchStatusType {
    #[inline]
    pub fn resolve(is_dispatched_gitmap: bool, is_enqueued_supabase: bool) -> Self {
        if is_dispatched_gitmap {
            Self::GitMapClusterSsh
        } else if is_enqueued_supabase {
            Self::SupabaseQueued
        } else {
            Self::LocalRecorded
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitMapClusterSsh => "gitmap_cluster_ssh",
            Self::SupabaseQueued => "supabase_queued",
            Self::LocalRecorded => "local_recorded",
        }
    }

    #[inline]
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::GitMapClusterSsh => "Sent via GitMap Cluster SSH",
            Self::SupabaseQueued => "Queued on the secondary database — the node will pick it up",
            Self::LocalRecorded => "Queued (the node recorded your instruction)",
        }
    }
}

impl std::fmt::Display for RemoteDispatchStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_execution_status_type_methods() {
        let exec = PromptExecutionStatusType::from_spawned(true);
        assert_eq!(exec, PromptExecutionStatusType::ExecutingNow);
        assert_eq!(exec.as_str(), "executing_now");
        assert_eq!(exec.display_label(), "Executing now");
        assert_eq!(exec.to_string(), "Executing now");
        assert!(exec.is_executing());
        assert!(!exec.is_queued());

        let queued = PromptExecutionStatusType::from_spawned(false);
        assert_eq!(queued, PromptExecutionStatusType::Queued);
        assert_eq!(queued.as_str(), "queued");
        assert_eq!(queued.display_label(), "Queued");
        assert_eq!(queued.to_string(), "Queued");
        assert!(!queued.is_executing());
        assert!(queued.is_queued());
    }

    #[test]
    fn test_remote_dispatch_status_type_resolution() {
        let ssh = RemoteDispatchStatusType::resolve(true, false);
        assert_eq!(ssh, RemoteDispatchStatusType::GitMapClusterSsh);
        assert_eq!(ssh.as_str(), "gitmap_cluster_ssh");
        assert_eq!(ssh.display_label(), "Sent via GitMap Cluster SSH");
        assert_eq!(ssh.to_string(), "Sent via GitMap Cluster SSH");

        let supa = RemoteDispatchStatusType::resolve(false, true);
        assert_eq!(supa, RemoteDispatchStatusType::SupabaseQueued);
        assert_eq!(supa.as_str(), "supabase_queued");
        assert_eq!(
            supa.display_label(),
            "Queued on the secondary database — the node will pick it up"
        );
        assert_eq!(
            supa.to_string(),
            "Queued on the secondary database — the node will pick it up"
        );

        let local = RemoteDispatchStatusType::resolve(false, false);
        assert_eq!(local, RemoteDispatchStatusType::LocalRecorded);
        assert_eq!(local.as_str(), "local_recorded");
        assert_eq!(
            local.display_label(),
            "Queued (the node recorded your instruction)"
        );
        assert_eq!(
            local.to_string(),
            "Queued (the node recorded your instruction)"
        );
    }
}
