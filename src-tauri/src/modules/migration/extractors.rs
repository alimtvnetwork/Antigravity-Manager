use crate::models::{Account, TokenData};
use crate::modules::{account, db};
use crate::utils::protobuf;
use base64::{engine::general_purpose, Engine as _};
use std::path::PathBuf;

use super::*;

/// Import current logged-in accounts from all local sources (Keyring, candidate DBs, V1 CLI)
pub async fn import_from_db(target_ide: Option<&str>) -> Result<Account, String> {
    let accounts = import_all_local_accounts(target_ide).await?;
    accounts
        .into_iter()
        .next()
        .ok_or_else(|| "No accounts found".to_string())
}

/// Get current Refresh Token from database (common logic)
pub fn extract_refresh_token_from_file(db_path: &PathBuf) -> Result<String, String> {
    extract_oauth_state_from_file(db_path).map(|state| state.refresh_token)
}

pub(crate) fn extract_enterprise_project_id_from_conn(
    conn: &rusqlite::Connection,
) -> Result<Option<String>, String> {
    let entry_b64: Option<String> = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["antigravityUnifiedStateSync.enterprisePreferences"],
            |row| row.get(0),
        )
        .ok();

    let Some(entry_b64) = entry_b64 else {
        return Ok(None);
    };

    let (sentinel_key, payload) = protobuf::decode_unified_state_entry(&entry_b64)?;
    if sentinel_key != "enterpriseGcpProjectId" {
        return Ok(None);
    }

    let Some(project_bytes) = protobuf::find_field(&payload, 3)? else {
        return Ok(None);
    };

    let project_id = String::from_utf8(project_bytes)
        .map_err(|_| "enterpriseGcpProjectId is not UTF-8 encoded".to_string())?;
    if project_id.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(project_id))
    }
}

pub(crate) fn extract_oauth_state_from_file(
    db_path: &PathBuf,
) -> Result<ImportedOAuthState, String> {
    use base64::{engine::general_purpose, Engine as _};

    if !db_path.exists() {
        return Err(format!("Database file not found: {:?}", db_path));
    }

    // Connect to database
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // 1. 尝试新版格式 (>= 1.16.5)
    // 键: antigravityUnifiedStateSync.oauthToken
    // 结构: Outer(F1) -> Inner(F2) -> Inner2(F1) -> Base64 -> OAuthInfo
    let new_format_data: Option<String> = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["antigravityUnifiedStateSync.oauthToken"],
            |row| row.get(0),
        )
        .ok();

    if let Some(outer_b64) = new_format_data {
        crate::modules::logger::log_info(
            "Detected new format database (antigravityUnifiedStateSync.oauthToken)",
        );
        let (sentinel_key, oauth_info_blob) = protobuf::decode_unified_state_entry(&outer_b64)?;
        if sentinel_key != "oauthTokenInfoSentinelKey" {
            return Err(format!("Unexpected OAuth sentinel key: {}", sentinel_key));
        }

        // 解析 OAuthInfo (Field 3) -> Refresh Token
        let refresh_bytes = protobuf::find_field(&oauth_info_blob, 3)
            .map_err(|e| format!("Parsing OAuthInfo Field 3 failed: {}", e))?
            .ok_or("Refresh Token not found in OAuthInfo (Field 3)")?;

        let refresh_token = String::from_utf8(refresh_bytes)
            .map_err(|_| "Refresh Token is not UTF-8 encoded".to_string())?;
        let is_gcp_tos = protobuf::find_varint_field(&oauth_info_blob, 6)?.unwrap_or(1) != 0;
        let project_id = extract_enterprise_project_id_from_conn(&conn)?;

        return Ok(ImportedOAuthState {
            refresh_token,
            is_gcp_tos,
            project_id,
        });
    }

    // 2. 尝试旧版格式 (< 1.16.5)
    crate::modules::logger::log_info(
        "Falling back to old format database (jetskiStateSync.agentManagerInitState)",
    );
    let current_data: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["jetskiStateSync.agentManagerInitState"],
            |row| row.get(0),
        )
        .map_err(|_| "Login state data not found in either format".to_string())?;

    // Base64 decode
    let blob = general_purpose::STANDARD
        .decode(&current_data)
        .map_err(|e| format!("Base64 decoding failed: {}", e))?;

    // 1. Find oauthTokenInfo (Field 6)
    let oauth_data = protobuf::find_field(&blob, 6)
        .map_err(|e| format!("Protobuf parsing failed: {}", e))?
        .ok_or("OAuth data not found (Field 6)")?;

    // 2. Extract refresh_token (Field 3)
    let refresh_bytes = protobuf::find_field(&oauth_data, 3)
        .map_err(|e| format!("OAuth data parsing failed: {}", e))?
        .ok_or("Refresh Token not included in data (Field 3)")?;

    let refresh_token = String::from_utf8(refresh_bytes)
        .map_err(|_| "Refresh Token is not UTF-8 encoded".to_string())?;

    Ok(ImportedOAuthState {
        refresh_token,
        is_gcp_tos: true,
        project_id: extract_enterprise_project_id_from_conn(&conn)?,
    })
}

/// Get current Refresh Token from System Keyring or candidate databases
pub fn get_refresh_token_from_db(target_ide: Option<&str>) -> Result<String, String> {
    use crate::modules::integration;

    if let Ok(oauth_state) = integration::read_from_system_keyring() {
        return Ok(oauth_state.refresh_token);
    }

    let candidate_paths = db::get_all_candidate_db_paths(target_ide);
    for db_path in candidate_paths {
        if db_path.exists() {
            if let Ok(token) = extract_refresh_token_from_file(&db_path) {
                return Ok(token);
            }
        }
    }

    Err("Login state data not found in keyring or any database format".to_string())
}
