use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(skip)]
    pub oauth_client_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

impl UserInfo {
    /// Get best display name
    pub fn get_display_name(&self) -> Option<String> {
        // Prefer name
        if let Some(name) = &self.name {
            if !name.trim().is_empty() {
                return Some(name.clone());
            }
        }

        // If name is empty, combine given_name and family_name
        match (&self.given_name, &self.family_name) {
            (Some(given), Some(family)) => Some(format!("{} {}", given, family)),
            (Some(given), None) => Some(given.clone()),
            (None, Some(family)) => Some(family.clone()),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OAuthClientConfig {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) is_builtin: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct OAuthClientRegistry {
    pub(crate) clients: Vec<OAuthClientConfig>,
    pub(crate) active_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClientDescriptor {
    pub key: String,
    pub label: String,
    pub client_id: String,
    pub is_active: bool,
    pub is_builtin: bool,
}
