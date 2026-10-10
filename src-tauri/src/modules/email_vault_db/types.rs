use base64::prelude::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: String,
    pub alias: String,
    pub email: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub imap_host: String,
    pub imap_port: u16,
    pub encryption_type: String, // "TLS", "STARTTLS", "SSL", "NONE"
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Input payload for creating or updating an email account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccountInput {
    pub id: Option<String>,
    pub alias: String,
    pub email: String,
    pub password: Option<String>,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub imap_host: String,
    pub imap_port: u16,
    pub encryption_type: String,
    pub is_default: bool,
    pub is_active: bool,
}

/// Represents a secure credential record in the dedicated split passwords database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCredential {
    pub account_id: String,
    pub auth_type: String,
    pub encrypted_secret: String,
    pub rsa_public_fingerprint: String,
    pub ssh_rsa_public_key: String,
    pub salt: String,
    pub updated_at: i64,
}

/// Represents a notification recipient (individual or group)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecipient {
    pub id: String,
    pub email: String,
    pub group_name: String,
    pub is_active: bool,
    pub created_at: i64,
}

/// Input payload for creating a notification recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecipientInput {
    pub email: String,
    pub group_name: Option<String>,
    pub is_active: Option<bool>,
}

pub(crate) fn default_baseline_polling() -> u32 {
    4
}

pub(crate) fn default_active_awaiting() -> u32 {
    10
}

pub(crate) fn default_true() -> bool {
    true
}

/// Email watcher & notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotificationSettings {
    pub id: String,
    pub is_enabled: bool,
    pub polling_interval_minutes: u32,
    pub inbox_check_interval_minutes: u32,
    #[serde(default = "default_baseline_polling")]
    pub baseline_polling_interval_minutes: u32,
    #[serde(default = "default_active_awaiting")]
    pub active_awaiting_interval_seconds: u32,
    pub notify_on_quota_drop: bool,
    pub quota_drop_threshold_percent: u32,
    pub notify_on_workspace_switch: bool,
    pub notify_on_idle_workspace: bool,
    #[serde(default = "default_true")]
    pub notify_on_system_update: bool,
    pub allow_remote_prompt_execution: bool,
    pub allow_remote_cli_execution: bool,
    pub allow_remote_instance_rotation: bool,
    pub local_machine_name: String,
    pub local_machine_ip: String,
    pub updated_at: i64,
}

impl Default for EmailNotificationSettings {
    fn default() -> Self {
        Self {
            id: "global".to_string(),
            is_enabled: false,
            polling_interval_minutes: 3,
            inbox_check_interval_minutes: 1,
            baseline_polling_interval_minutes: 5,
            active_awaiting_interval_seconds: 10,
            notify_on_quota_drop: true,
            quota_drop_threshold_percent: 25,
            notify_on_workspace_switch: true,
            notify_on_idle_workspace: true,
            notify_on_system_update: true,
            allow_remote_prompt_execution: true,
            allow_remote_cli_execution: true,
            allow_remote_instance_rotation: true,
            local_machine_name: String::new(),
            local_machine_ip: String::new(),
            updated_at: Utc::now().timestamp(),
        }
    }
}

/// Represents an inbound audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInboundAuditLog {
    pub id: String,
    pub message_id: String,
    pub sender_email: String,
    pub subject: String,
    pub action_type: String,
    pub action_payload: String,
    pub execution_status: String,
    pub execution_result: String,
    pub received_at: i64,
}
