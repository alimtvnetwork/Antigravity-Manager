use crate::models::{Account, TokenData};
use crate::modules::{account, db};
use crate::utils::protobuf;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::*;

#[derive(Debug, Clone)]
pub struct ImportedOAuthState {
    pub refresh_token: String,
    pub is_gcp_tos: bool,
    pub project_id: Option<String>,
}

/// Scan and import V1 data
pub async fn import_from_v1() -> Result<Vec<Account>, String> {
    use crate::modules::oauth;

    let home = dirs::home_dir().ok_or("Failed to get home directory")?;

    // V1 data directory (confirmed cross-platform consistency from utils.py)
    let v1_dir = home.join(".antigravity-agent");

    let mut imported_accounts = Vec::new();

    // Try multiple possible filenames
    let index_files = vec![
        "antigravity_accounts.json", // Directly use string literal
        "accounts.json",
    ];

    let mut found_index = false;

    for index_filename in index_files {
        let v1_accounts_path = v1_dir.join(index_filename);

        if !v1_accounts_path.exists() {
            continue;
        }

        found_index = true;
        crate::modules::logger::log_info(&format!("V1 data discovered: {:?}", v1_accounts_path));

        let content = match fs::read_to_string(&v1_accounts_path) {
            Ok(c) => c,
            Err(e) => {
                crate::modules::logger::log_warn(&format!("Failed to read index: {}", e));
                continue;
            }
        };

        let v1_index: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                crate::modules::logger::log_warn(&format!("Failed to parse index JSON: {}", e));
                continue;
            }
        };

        // Compatible with two formats: direct map, or contains "accounts" field
        let accounts_map = if let Some(map) = v1_index.as_object() {
            if let Some(accounts) = map.get("accounts").and_then(|v| v.as_object()) {
                accounts
            } else {
                map
            }
        } else {
            continue;
        };

        for (id, acc_info) in accounts_map {
            let email_placeholder = acc_info
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Skip non-account keys (e.g. "current_account_id")
            if !acc_info.is_object() {
                continue;
            }

            let backup_file_str = acc_info.get("backup_file").and_then(|v| v.as_str());
            let data_file_str = acc_info.get("data_file").and_then(|v| v.as_str());

            // Prefer backup_file, then data_file
            let target_file = backup_file_str.or(data_file_str);

            if target_file.is_none() {
                crate::modules::logger::log_warn(&format!(
                    "Account {} ({}) missing data file path",
                    id, email_placeholder
                ));
                continue;
            }

            let mut backup_path = PathBuf::from(target_file.unwrap());

            // If relative path, try joining with v1_dir
            if !backup_path.exists() {
                backup_path = v1_dir.join(backup_path.file_name().unwrap_or_default());
            }

            // Try joining data/ or backups/ subdirectories again
            if !backup_path.exists() {
                let file_name = backup_path.file_name().unwrap_or_default();
                let try_backups = v1_dir.join("backups").join(file_name);
                if try_backups.exists() {
                    backup_path = try_backups;
                } else {
                    let try_accounts = v1_dir.join("accounts").join(file_name);
                    if try_accounts.exists() {
                        backup_path = try_accounts;
                    }
                }
            }

            if !backup_path.exists() {
                crate::modules::logger::log_warn(&format!(
                    "Account {} ({}) backup file not found: {:?}",
                    id, email_placeholder, backup_path
                ));
                continue;
            }

            // Read backup file
            if let Ok(backup_content) = fs::read_to_string(&backup_path) {
                if let Ok(backup_json) = serde_json::from_str::<Value>(&backup_content) {
                    // Compatible with two formats:
                    // 1. V1 backup: jetskiStateSync.agentManagerInitState -> Protobuf
                    // 2. V2/Script data: JSON containing "token" field

                    let mut refresh_token_opt = None;

                    // Try format 2
                    if let Some(token_data) = backup_json.get("token") {
                        if let Some(rt) = token_data.get("refresh_token").and_then(|v| v.as_str()) {
                            refresh_token_opt = Some(rt.to_string());
                        }
                    }

                    // Try format 1
                    if refresh_token_opt.is_none() {
                        if let Some(state_b64) = backup_json
                            .get("jetskiStateSync.agentManagerInitState")
                            .and_then(|v| v.as_str())
                        {
                            // Parse Protobuf
                            if let Ok(blob) = general_purpose::STANDARD.decode(state_b64) {
                                if let Ok(Some(oauth_data)) = protobuf::find_field(&blob, 6) {
                                    if let Ok(Some(refresh_bytes)) =
                                        protobuf::find_field(&oauth_data, 3)
                                    {
                                        if let Ok(rt) = String::from_utf8(refresh_bytes) {
                                            refresh_token_opt = Some(rt);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(refresh_token) = refresh_token_opt {
                        crate::modules::logger::log_info(&format!(
                            "Importing account: {}",
                            email_placeholder
                        ));
                        let (email, access_token, expires_in, oauth_client_key) =
                            match oauth::refresh_access_token(&refresh_token, None).await {
                                Ok(token_resp) => {
                                    let oauth_client_key = token_resp.oauth_client_key.clone();
                                    match oauth::get_user_info(&token_resp.access_token, None).await
                                    {
                                        Ok(user_info) => (
                                            user_info.email,
                                            token_resp.access_token,
                                            token_resp.expires_in,
                                            oauth_client_key,
                                        ),
                                        Err(_) => (
                                            email_placeholder.clone(),
                                            token_resp.access_token,
                                            token_resp.expires_in,
                                            oauth_client_key,
                                        ),
                                    }
                                }
                                Err(e) => {
                                    crate::modules::logger::log_warn(&format!(
                                        "Token refresh failed (likely expired): {}",
                                        e
                                    ));
                                    (
                                        email_placeholder.clone(),
                                        "imported_access_token".to_string(),
                                        0,
                                        None,
                                    )
                                }
                            };
                        let token_data = TokenData::new(
                            access_token,
                            refresh_token,
                            expires_in,
                            Some(email.clone()),
                            None, // project_id will be fetched on demand
                            None, // session_id
                            true, // V1 tokens are Antigravity Google OAuth tokens
                            None, // V1 doesn't have id_token saved
                        )
                        .with_oauth_client_key(oauth_client_key);
                        // Name already fetched in get_user_info at line 153, but outside match scope, use None to be safe
                        match account::upsert_account(email.clone(), None, token_data) {
                            Ok(acc) => {
                                crate::modules::logger::log_info(&format!(
                                    "Import successful: {}",
                                    email
                                ));
                                imported_accounts.push(acc);
                            }
                            Err(e) => crate::modules::logger::log_error(&format!(
                                "Import save failed {}: {}",
                                email, e
                            )),
                        }
                    } else {
                        crate::modules::logger::log_warn(&format!(
                            "Account {} data file missing Refresh Token",
                            email_placeholder
                        ));
                    }
                }
            }
        }
    }

    if !found_index {
        return Err("V1 account data file not found".to_string());
    }

    Ok(imported_accounts)
}

/// Import account from custom database path
pub async fn import_from_custom_db_path(path_str: String) -> Result<Account, String> {
    use crate::modules::oauth;

    let path = PathBuf::from(path_str);
    if !path.exists() {
        return Err(format!("File does not exist: {:?}", path));
    }

    let oauth_state = extract_oauth_state_from_file(&path)?;
    let refresh_token = oauth_state.refresh_token.clone();

    // 3. Use Refresh Token to get latest Access Token and user info
    crate::modules::logger::log_info("Getting user info using Refresh Token...");
    let token_resp = oauth::refresh_access_token(&refresh_token, None).await?;
    let user_info = oauth::get_user_info(&token_resp.access_token, None).await?;

    let email = user_info.email;

    crate::modules::logger::log_info(&format!("Successfully retrieved account info: {}", email));

    let token_data = TokenData::new(
        token_resp.access_token,
        refresh_token,
        token_resp.expires_in,
        Some(email.clone()),
        oauth_state.project_id,
        None, // session_id will be generated in token_manager
        oauth_state.is_gcp_tos,
        token_resp.id_token,
    )
    .with_oauth_client_key(token_resp.oauth_client_key);
    // 4. Add or update account
    account::upsert_account(email.clone(), user_info.name, token_data)
}

/// Scan all local sources (Keyring/Keychain, IDE Databases, V1/CLI Directories) and import all unique accounts
pub async fn import_all_local_accounts(target_ide: Option<&str>) -> Result<Vec<Account>, String> {
    use crate::modules::{integration, oauth};

    let mut imported_accounts = Vec::new();
    let mut seen_refresh_tokens = std::collections::HashSet::new();

    // 1. Check System Keyring / Keychain
    if let Ok(oauth_state) = integration::read_from_system_keyring() {
        let refresh_token = oauth_state.refresh_token.clone();
        if !refresh_token.is_empty() && seen_refresh_tokens.insert(refresh_token.clone()) {
            crate::modules::logger::log_info("Discovered OAuth state in System Keyring/Keychain");
            match oauth::refresh_access_token(&refresh_token, None).await {
                Ok(token_resp) => {
                    let email = match oauth::get_user_info(&token_resp.access_token, None).await {
                        Ok(info) => info.email,
                        Err(_) => "Unknown".to_string(),
                    };
                    let token_data = TokenData::new(
                        token_resp.access_token,
                        refresh_token,
                        token_resp.expires_in,
                        Some(email.clone()),
                        oauth_state.project_id,
                        None,
                        oauth_state.is_gcp_tos,
                        token_resp.id_token,
                    )
                    .with_oauth_client_key(token_resp.oauth_client_key);

                    if let Ok(acc) = account::upsert_account(email, None, token_data) {
                        imported_accounts.push(acc);
                    }
                }
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "Candidate Keyring OAuth token refresh failed (likely expired/revoked): {}",
                        e
                    ));
                }
            }
        }
    }

    // 2. Scan all candidate database paths
    let candidate_paths = db::get_all_candidate_db_paths(target_ide);
    for db_path in candidate_paths {
        if db_path.exists() {
            if let Ok(oauth_state) = extract_oauth_state_from_file(&db_path) {
                let refresh_token = oauth_state.refresh_token.clone();
                if !refresh_token.is_empty() && seen_refresh_tokens.insert(refresh_token.clone()) {
                    crate::modules::logger::log_info(&format!(
                        "Discovered OAuth state in DB path: {:?}",
                        db_path
                    ));
                    match oauth::refresh_access_token(&refresh_token, None).await {
                        Ok(token_resp) => {
                            let email =
                                match oauth::get_user_info(&token_resp.access_token, None).await {
                                    Ok(info) => info.email,
                                    Err(_) => "Unknown".to_string(),
                                };
                            let token_data = TokenData::new(
                                token_resp.access_token,
                                refresh_token,
                                token_resp.expires_in,
                                Some(email.clone()),
                                oauth_state.project_id,
                                None,
                                oauth_state.is_gcp_tos,
                                token_resp.id_token,
                            )
                            .with_oauth_client_key(token_resp.oauth_client_key);

                            if let Ok(acc) = account::upsert_account(email, None, token_data) {
                                imported_accounts.push(acc);
                            }
                        }
                        Err(e) => {
                            crate::modules::logger::log_warn(&format!(
                                "Candidate DB ({:?}) OAuth token refresh failed (likely expired/revoked): {}",
                                db_path, e
                            ));
                        }
                    }
                }
            }
        }
    }

    // 3. Scan V1 / CLI agent directory (~/.antigravity-agent)
    if let Ok(v1_accounts) = import_from_v1().await {
        for acc in v1_accounts {
            if seen_refresh_tokens.insert(acc.token.refresh_token.clone()) {
                imported_accounts.push(acc);
            }
        }
    }

    if imported_accounts.is_empty() {
        return Err(
            "No login state data found across Keyring, IDE databases, or CLI directories"
                .to_string(),
        );
    }

    Ok(imported_accounts)
}
