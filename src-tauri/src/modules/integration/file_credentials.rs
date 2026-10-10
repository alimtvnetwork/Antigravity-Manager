use crate::models::Account;
use std::fs;

use super::*;

/// 辅助方法：同步写入指定目录下的本地文件凭据 (<base_home>/.gemini/oauth_creds.json, google_accounts.json, jetski-standalone-oauth-token)
pub fn write_to_file_credentials_at(
    base_home: &std::path::Path,
    account: &crate::models::Account,
) -> Result<(), String> {
    let gemini_dir = if base_home.ends_with(".gemini") {
        base_home.to_path_buf()
    } else {
        base_home.join(".gemini")
    };

    if !gemini_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&gemini_dir) {
            crate::modules::logger::log_warn(&format!(
                "[Desktop] Failed to create .gemini directory at {:?}: {}",
                gemini_dir, e
            ));
            return Err(format!("Failed to create .gemini directory: {}", e));
        }
    }

    // Also copy installation_id if it exists from global .gemini/antigravity/installation_id
    if let Some(global_home) = dirs::home_dir() {
        let global_inst_id = global_home
            .join(".gemini")
            .join("antigravity")
            .join("installation_id");
        if global_inst_id.exists() {
            let target_antigravity = gemini_dir.join("antigravity");
            // Justification: installation_id mirror directory is best-effort; the credential files below are the authoritative output
            crate::error::record_ignored(
                std::fs::create_dir_all(&target_antigravity),
                "create antigravity directory for installation_id mirror",
            );
            let target_inst_id = target_antigravity.join("installation_id");
            if !target_inst_id.exists() {
                // Justification: installation_id copy is a nicety; a missing copy does not affect credentials
                crate::error::record_ignored(
                    std::fs::copy(&global_inst_id, &target_inst_id),
                    "mirror installation_id",
                );
            }
        }
    }

    let expiry_ms = if account.token.expiry_timestamp > 10_000_000_000 {
        account.token.expiry_timestamp
    } else {
        account.token.expiry_timestamp * 1000
    };

    let expiry_datetime = chrono::DateTime::from_timestamp(account.token.expiry_timestamp, 0)
        .unwrap_or_else(chrono::Utc::now);
    let expiry_rfc3339 = expiry_datetime.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    #[derive(serde::Serialize)]
    struct OAuthCredsFile {
        access_token: String,
        refresh_token: String,
        token_type: String,
        expiry_date: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        id_token: Option<String>,
        scope: String,
    }

    let creds = OAuthCredsFile {
        access_token: account.token.access_token.clone(),
        refresh_token: account.token.refresh_token.clone(),
        token_type: "Bearer".to_string(),
        expiry_date: expiry_ms,
        id_token: account.token.id_token.clone(),
        scope: "https://www.googleapis.com/auth/userinfo.email openid https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/userinfo.profile".to_string(),
    };

    let creds_path = gemini_dir.join("oauth_creds.json");
    let json_str = serde_json::to_string_pretty(&creds)
        .map_err(|e| format!("Failed to serialize oauth_creds JSON: {}", e))?;

    if let Err(e) = std::fs::write(&creds_path, json_str) {
        crate::modules::logger::log_warn(&format!(
            "[Desktop] Failed to write oauth_creds.json at {:?}: {}",
            creds_path, e
        ));
        return Err(format!("Failed to write oauth_creds.json: {}", e));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&creds_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set 0600 permissions on oauth_creds.json: {e}"))?;
    }

    #[derive(serde::Serialize)]
    struct GoogleAccountsFile {
        active: String,
        old: Vec<String>,
    }

    let accounts_info = GoogleAccountsFile {
        active: account.email.clone(),
        old: vec![],
    };

    let accounts_path = gemini_dir.join("google_accounts.json");
    if let Ok(accounts_json_str) = serde_json::to_string_pretty(&accounts_info) {
        // Justification: google_accounts.json is a mirror; oauth_creds.json is the authoritative credential file
        crate::error::record_ignored(
            std::fs::write(&accounts_path, accounts_json_str),
            "write google_accounts.json mirror",
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Justification: permission hardening on a best-effort mirror file
            crate::error::record_ignored(
                std::fs::set_permissions(&accounts_path, std::fs::Permissions::from_mode(0o600)),
                "set 0600 permissions on google_accounts.json",
            );
        }
    }

    // Sync jetski-standalone-oauth-token in ~/.gemini for Go language_server worker fallback
    let mut token_obj = serde_json::json!({
        "access_token": account.token.access_token,
        "token_type": "Bearer",
        "refresh_token": account.token.refresh_token,
        "expiry": expiry_rfc3339,
    });
    if let Some(ref id_tok) = account.token.id_token {
        token_obj["id_token"] = serde_json::Value::String(id_tok.clone());
    }
    let jetski_payload = serde_json::json!({
        "token": token_obj,
        "auth_method": "consumer"
    });
    let jetski_path = gemini_dir.join("jetski-standalone-oauth-token");
    if let Ok(jetski_json) = serde_json::to_string(&jetski_payload) {
        // Justification: jetski token file is a fallback for the Go language_server worker; best-effort
        crate::error::record_ignored(
            std::fs::write(&jetski_path, jetski_json),
            "write jetski standalone oauth token",
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Justification: permission hardening on a best-effort mirror file
            crate::error::record_ignored(
                std::fs::set_permissions(&jetski_path, std::fs::Permissions::from_mode(0o600)),
                "set 0600 permissions on jetski token file",
            );
        }
    }

    // Also mirror to .gemini subdirectories for language_server, agy CLI, and background workers
    for sub in &["antigravity", "antigravity-ide", "antigravity-cli", "cache"] {
        let target_sub = gemini_dir.join(sub);
        if std::fs::create_dir_all(&target_sub).is_ok() {
            // Justification: credential mirror to IDE subdirectories is best-effort; the primary .gemini files are authoritative
            crate::error::record_ignored(
                std::fs::copy(&creds_path, target_sub.join("oauth_creds.json")),
                "mirror oauth_creds.json to IDE subdir",
            );
            // Justification: credential mirror to IDE subdirectories is best-effort; the primary .gemini files are authoritative
            crate::error::record_ignored(
                std::fs::copy(&accounts_path, target_sub.join("google_accounts.json")),
                "mirror google_accounts.json to IDE subdir",
            );
            // Justification: credential mirror to IDE subdirectories is best-effort; the primary .gemini files are authoritative
            crate::error::record_ignored(
                std::fs::copy(
                    &jetski_path,
                    target_sub.join("jetski-standalone-oauth-token"),
                ),
                "mirror jetski token to IDE subdir",
            );
        }
    }

    if !base_home.ends_with(".gemini") {
        // Justification: credential mirror to the base home is best-effort; the primary .gemini files are authoritative
        crate::error::record_ignored(
            std::fs::copy(&creds_path, base_home.join("oauth_creds.json")),
            "mirror oauth_creds.json to base home",
        );
        // Justification: credential mirror to the base home is best-effort; the primary .gemini files are authoritative
        crate::error::record_ignored(
            std::fs::copy(&accounts_path, base_home.join("google_accounts.json")),
            "mirror google_accounts.json to base home",
        );
        // Justification: credential mirror to the base home is best-effort; the primary .gemini files are authoritative
        crate::error::record_ignored(
            std::fs::copy(
                &jetski_path,
                base_home.join("jetski-standalone-oauth-token"),
            ),
            "mirror jetski token to base home",
        );
    }

    for marker_name in &[
        "antigravity-keyring-unavailable",
        "antigravity-ide-keyring-unavailable",
        "antigravity-cli-keyring-unavailable",
    ] {
        // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
        crate::error::record_ignored(
            std::fs::write(gemini_dir.join(marker_name), b"1\n"),
            "write keyring-unavailable marker",
        );
        // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
        crate::error::record_ignored(
            std::fs::write(gemini_dir.join("antigravity").join(marker_name), b"1\n"),
            "write keyring-unavailable marker (antigravity subdir)",
        );
        // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
        crate::error::record_ignored(
            std::fs::write(gemini_dir.join("antigravity-ide").join(marker_name), b"1\n"),
            "write keyring-unavailable marker (antigravity-ide subdir)",
        );
        // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
        crate::error::record_ignored(
            std::fs::write(gemini_dir.join("antigravity-cli").join(marker_name), b"1\n"),
            "write keyring-unavailable marker (antigravity-cli subdir)",
        );
        // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
        crate::error::record_ignored(
            std::fs::write(gemini_dir.join("cache").join(marker_name), b"1\n"),
            "write keyring-unavailable marker (cache subdir)",
        );
        if !base_home.ends_with(".gemini") {
            // Justification: keyring-unavailable marker is informational; the credential sync already succeeded
            crate::error::record_ignored(
                std::fs::write(base_home.join(marker_name), b"1\n"),
                "write keyring-unavailable marker (base home)",
            );
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Desktop] Successfully synced file-based credentials to {:?} for: {}",
        gemini_dir, account.email
    ));

    Ok(())
}

/// 辅助方法：同步写入本地文件凭据 (~/.gemini/oauth_creds.json, ~/.gemini/google_accounts.json, 以及 ~/.gemini/jetski-standalone-oauth-token)
/// 用于在 SSH 会话、容器环境或无系统 Keyring / D-Bus 的场景下保障 CLI/工具/Worker 的凭据兼容性
pub fn write_to_file_credentials(account: &crate::models::Account) -> Result<(), String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Err("Failed to resolve user home directory".to_string()),
    };
    write_to_file_credentials_at(&home, account)
}

/// 辅助方法：从本地文件凭据 (~/.gemini/oauth_creds.json) 读取 Token 作为跨平台回退
pub(crate) fn read_from_file_credentials(
) -> Result<crate::modules::migration::ImportedOAuthState, String> {
    let home =
        dirs::home_dir().ok_or_else(|| "Failed to resolve user home directory".to_string())?;
    let creds_path = home.join(".gemini").join("oauth_creds.json");
    if !creds_path.exists() {
        return Err("No ~/.gemini/oauth_creds.json found".to_string());
    }
    let content = fs::read_to_string(&creds_path)
        .map_err(|e| format!("Failed to read oauth_creds.json: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse oauth_creds.json: {}", e))?;
    let refresh_token = json
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Refresh token not found in oauth_creds.json".to_string())?
        .to_string();
    Ok(crate::modules::migration::ImportedOAuthState {
        refresh_token,
        is_gcp_tos: true,
        project_id: None,
    })
}
