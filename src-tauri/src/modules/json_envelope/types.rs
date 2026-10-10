use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::*;

pub(crate) fn default_version() -> String {
    "2.0".to_string()
}

/// Structured work directory configuration using root variables.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkDirectoryConfig {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_path: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_applied: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_enforced: bool,
}

pub(crate) fn deserialize_work_directory<'de, D>(
    deserializer: D,
) -> Result<Option<WorkDirectoryConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Value> = Option::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(WorkDirectoryConfig {
            path: s,
            default_path: None,
            is_applied: true,
            is_enforced: false,
        })),
        Some(Value::Object(_)) => serde_json::from_value::<WorkDirectoryConfig>(val.unwrap())
            .map(Some)
            .map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

/// Metadata attributes header required for all standard AGM JSON envelopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JsonAttributes {
    #[serde(rename = "type")]
    pub data_type: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "agmVersion",
        alias = "gitmapVersion",
        alias = "gitmap_version"
    )]
    pub agm_version: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "importCommand"
    )]
    pub import_command: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "exportCommand"
    )]
    pub export_command: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "helpCommand"
    )]
    pub help_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "workDirectory",
        deserialize_with = "deserialize_work_directory"
    )]
    pub work_directory: Option<WorkDirectoryConfig>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "defaultWorkDirectory"
    )]
    pub default_work_directory: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "std::ops::Not::not",
        alias = "isWorkDirectoryApplied"
    )]
    pub is_work_directory_applied: bool,
    #[serde(
        default,
        skip_serializing_if = "std::ops::Not::not",
        alias = "isWorkDirectoryEnforced"
    )]
    pub is_work_directory_enforced: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "nodeId")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "nodeAlias")]
    pub node_alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Resolves a relative JSON filename against CWD and common relative subdirectories.
pub fn resolve_relative_json_path(input: &str) -> PathBuf {
    let trimmed = input.trim();
    let direct = PathBuf::from(trimmed);
    if direct.exists() {
        return direct;
    }
    let base_name = direct
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(trimmed);
    let search_dirs = [
        ".",
        "vault",
        "instances",
        "vault/instances",
        "02-antigravity-manager/vault",
        "02-antigravity-and-event-manager/vault",
        "02-antigravity-and-event-manager/vault/instances",
        "01-gitmap",
    ];
    for dir in &search_dirs {
        let candidate = Path::new(dir).join(base_name);
        if candidate.exists() {
            return candidate;
        }
    }
    direct
}

pub fn default_commands_for_type(
    dtype: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let lower = dtype.to_lowercase();
    if lower.contains("supabase-endpoints") || lower.contains("supabase_endpoints") {
        (
            Some("agm supabase load-json supabase_config.json -y".to_string()),
            Some("agm supabase export --file supabase_config.json".to_string()),
            Some("agm which-format supabase_config.json".to_string()),
            Some("Supabase multi-node endpoints configuration for AGM fleet. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("supabase-credentials") || lower.contains("supabase_credentials") {
        (
            Some("agm supabase load-json supabase_config.json -y".to_string()),
            Some("agm supabase export --file supabase_config.json".to_string()),
            Some("agm which-format supabase_config.json".to_string()),
            Some("Supabase fleet REST authentication credentials and service tokens. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("accounts") {
        (
            Some("agm accounts import accounts.json".to_string()),
            Some("agm accounts export --file accounts.json".to_string()),
            Some("agm which-format accounts.json".to_string()),
            Some("Google Gemini accounts and quota matrix. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("instances") {
        (
            Some("agm instances import instances.json".to_string()),
            Some("agm instances export --file instances.json".to_string()),
            Some("agm which-format instances.json".to_string()),
            Some("Sandbox instance profiles and directory bindings for AGM fleet. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("email") {
        (
            Some("agm email import email-credentials.json".to_string()),
            Some("agm email export --file email-credentials.json".to_string()),
            Some("agm which-format email-credentials.json".to_string()),
            Some("Inbound IMAP & outbound SMTP fleet email credentials configuration. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("telegram") {
        (
            Some("agm telegram import telegram_config.json".to_string()),
            Some("agm telegram export --file telegram_config.json".to_string()),
            Some("agm which-format telegram_config.json".to_string()),
            Some("Telegram bot configuration for remote AGM commands and alerts. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("config") {
        (
            Some("agm config restore gui_config.json -y".to_string()),
            Some("agm config export --file gui_config.json".to_string()),
            Some("agm which-format gui_config.json".to_string()),
            Some("Antigravity Manager system configuration backup. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("ssh") || lower.contains("nodes") {
        (
            Some("agm ssh nodes import-json gitmap-ssh-nodes.json -y".to_string()),
            Some("agm ssh nodes export-json gitmap-ssh-nodes.json".to_string()),
            Some("agm which-format gitmap-ssh-nodes.json".to_string()),
            Some("SSH cluster fleet nodes configuration. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else {
        (None, None, Some("agm which-format".to_string()), None)
    }
}

impl JsonAttributes {
    pub fn new(data_type: impl Into<String>) -> Self {
        let dtype = data_type.into();
        let cur_time = chrono::Utc::now().to_rfc3339();

        let work_cfg = WorkDirectoryConfig {
            path: "${workDir}".to_string(),
            default_path: Some("D:\\work".to_string()),
            is_applied: true,
            is_enforced: false,
        };

        let (import_cmd, export_cmd, help_cmd, notes) = default_commands_for_type(&dtype);

        Self {
            data_type: dtype,
            version: default_version(),
            source: Some("agm-cli".to_string()),
            how: export_cmd.clone(),
            agm_version: Some("4.103.0".to_string()),
            import_command: import_cmd,
            export_command: export_cmd,
            help_command: help_cmd,
            notes,
            timestamp: Some(cur_time.clone()),
            work_directory: Some(work_cfg),
            default_work_directory: Some("D:\\work".to_string()),
            is_work_directory_applied: true,
            is_work_directory_enforced: false,
            created_at: Some(cur_time),
            node_id: None,
            node_alias: None,
            encoding: Some("utf-8".to_string()),
            description: None,
        }
    }
}
