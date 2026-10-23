use crate::modules::user_token_db::{self, TokenIpBinding, UserToken};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub username: String,
    pub expires_type: String,
    pub description: Option<String>,
    pub max_ips: i32,
    pub curfew_start: Option<String>,
    pub curfew_end: Option<String>,
    pub custom_expires_at: Option<i64>, // Custom expiration timestamp (seconds)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTokenRequest {
    pub username: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub max_ips: Option<i32>,
    pub curfew_start: Option<Option<String>>,
    pub curfew_end: Option<Option<String>>,
}

// Command implementation

/// List all tokens
#[tauri::command]
pub async fn list_user_tokens() -> Result<Vec<UserToken>, String> {
    user_token_db::list_tokens()
}

/// Create new token
#[tauri::command]
pub async fn create_user_token(request: CreateTokenRequest) -> Result<UserToken, String> {
    user_token_db::create_token(
        request.username,
        request.expires_type,
        request.description,
        request.max_ips,
        request.curfew_start,
        request.curfew_end,
        request.custom_expires_at,
    )
}

/// update token
#[tauri::command]
pub async fn update_user_token(id: String, request: UpdateTokenRequest) -> Result<(), String> {
    user_token_db::update_token(
        &id,
        request.username,
        request.description,
        request.enabled,
        request.max_ips,
        request.curfew_start,
        request.curfew_end,
    )
}

/// delete token
#[tauri::command]
pub async fn delete_user_token(id: String) -> Result<(), String> {
    user_token_db::delete_token(&id)
}

/// Renew token
#[tauri::command]
pub async fn renew_user_token(id: String, expires_type: String) -> Result<(), String> {
    user_token_db::renew_token(&id, &expires_type)
}

/// Get token IP binding
#[tauri::command]
pub async fn get_token_ip_bindings(token_id: String) -> Result<Vec<TokenIpBinding>, String> {
    user_token_db::get_token_ips(&token_id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTokenStats {
    pub total_tokens: usize,
    pub active_tokens: usize,
    pub total_users: usize,
    pub today_requests: i64,
}

/// Get simple statistics
#[tauri::command]
pub async fn get_user_token_summary() -> Result<UserTokenStats, String> {
    let tokens = user_token_db::list_tokens()?;
    let active_tokens = tokens.iter().filter(|t| t.enabled).count();

    // Count unique users
    let mut users = std::collections::HashSet::new();
    for t in &tokens {
        users.insert(t.username.clone());
    }

    // Here we simply return some data. The number of requests is best aggregated and queried from the database.
    // It is only used as a demonstration at this time, and the number of requests is not accurately counted today.

    Ok(UserTokenStats {
        total_tokens: tokens.len(),
        active_tokens,
        total_users: users.len(),
        today_requests: 0, // TODO: Implement daily stats query
    })
}
