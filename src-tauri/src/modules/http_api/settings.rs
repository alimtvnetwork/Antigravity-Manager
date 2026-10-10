use crate::modules::{account, logger, proxy_db};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use super::*;

pub const DEFAULT_PORT: u16 = 19527;

// ============================================================================
// Settings
// ============================================================================

/// HTTP API Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpApiSettings {
    /// Whether to enable HTTP API service
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Listening port
    #[serde(default = "default_port")]
    pub port: u16,
}

pub(crate) fn default_enabled() -> bool {
    true
}

pub(crate) fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Default for HttpApiSettings {
    pub(crate) fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_PORT,
        }
    }
}

/// Load HTTP API settings
pub fn load_settings() -> Result<HttpApiSettings, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("http_api_settings.json");

    if !settings_path.exists() {
        return Ok(HttpApiSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

/// Save HTTP API settings
pub fn save_settings(settings: &HttpApiSettings) -> Result<(), String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("http_api_settings.json");

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))
}
