use super::config_paths::{get_config_paths, parse_jsonc};
use super::dtos::{PluginAccount, PluginAccountsFile};
use super::lock::{
    acquire_opencode_config_lock, atomically_write_config, ANTIGRAVITY_PROVIDER_ID,
    MAX_PROVIDER_ID_LEN,
};
use super::models::ModelInput;
use super::sync_apply::{apply_openai_compatible_provider_sync, apply_sync_to_config};
use super::sync_helpers::create_backup;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub fn sync_opencode_config(
    proxy_url: &str,
    api_key: &str,
    sync_accounts: bool,
    models_to_sync: Option<Vec<ModelInput>>,
) -> Result<(), String> {
    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _ag_config_path, ag_accounts_path)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    let mut config = match fs::read_to_string(&config_path) {
        Ok(content) => parse_jsonc(&content)
            .filter(Value::is_object)
            .ok_or_else(|| {
                "OpenCode config must be a valid JSON/JSONC object; file left unchanged".to_string()
            })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => return Err(format!("Failed to read OpenCode config: {}", error)),
    };

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    create_backup(&config_path)?;

    config = apply_sync_to_config(config, proxy_url, api_key, models_to_sync.as_deref());

    atomically_write_config(&config_path, &config)?;

    if sync_accounts {
        sync_accounts_file(&ag_accounts_path)?;
    }

    Ok(())
}

/// Provider ids become JSON keys in opencode.json and are accepted over the admin
/// HTTP API, so restrict them to a safe charset.
pub(crate) fn validate_provider_id(provider_id: &str) -> Result<(), String> {
    if provider_id.is_empty() {
        return Err("OpenCode provider id is required".to_string());
    }
    if provider_id.len() > MAX_PROVIDER_ID_LEN {
        return Err(format!(
            "Invalid OpenCode provider id: must be at most {} characters",
            MAX_PROVIDER_ID_LEN
        ));
    }
    if !provider_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "Invalid OpenCode provider id '{}': only letters, digits, '-' and '_' are allowed",
            provider_id
        ));
    }
    Ok(())
}

/// True when the error came from input validation rather than I/O, so callers can
/// answer 4xx instead of 5xx. Matches on explicit markers, never on generic
/// OS strings such as "Invalid argument".
pub fn is_provider_validation_error(message: &str) -> bool {
    const MARKERS: [&str; 6] = [
        "provider id is required",
        "Invalid OpenCode provider id",
        "is reserved",
        "already belongs to another API key",
        "OpenCode API key and base URL are required",
        "is not managed by Antigravity-Manager",
    ];
    MARKERS.iter().any(|marker| message.contains(marker))
}

pub fn sync_opencode_openai_provider(
    provider_id: &str,
    provider_name: &str,
    proxy_url: &str,
    api_key: &str,
    models_to_sync: Option<Vec<ModelInput>>,
) -> Result<(), String> {
    let provider_id = provider_id.trim();
    validate_provider_id(provider_id)?;

    if provider_id.eq_ignore_ascii_case(ANTIGRAVITY_PROVIDER_ID) {
        return Err(format!(
            "Provider id '{}' is reserved for Antigravity-Manager internal configuration",
            ANTIGRAVITY_PROVIDER_ID
        ));
    }

    let Some((config_path, _, _)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    sync_openai_provider_to_path(
        &config_path,
        provider_id,
        provider_name,
        proxy_url,
        api_key,
        models_to_sync.as_deref(),
    )
}

pub(crate) fn sync_openai_provider_to_path(
    config_path: &PathBuf,
    provider_id: &str,
    provider_name: &str,
    proxy_url: &str,
    api_key: &str,
    models_to_sync: Option<&[ModelInput]>,
) -> Result<(), String> {
    if api_key.trim().is_empty() || proxy_url.trim().is_empty() {
        return Err("OpenCode API key and base URL are required".to_string());
    }
    let _lock = acquire_opencode_config_lock();

    // A read/parse failure must never turn a user's existing config into {}.
    let mut config = match fs::read_to_string(config_path) {
        Ok(content) => parse_jsonc(&content)
            .filter(Value::is_object)
            .ok_or_else(|| {
                "OpenCode config must be a valid JSON/JSONC object; file left unchanged".to_string()
            })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => return Err(format!("Failed to read OpenCode config: {}", error)),
    };

    if provider_id.starts_with("apikey-fun-") {
        if let Some(existing) = config.get("provider").and_then(|p| p.get(provider_id)) {
            let existing_key = existing
                .get("options")
                .and_then(|options| options.get("apiKey"))
                .and_then(Value::as_str);
            if existing_key.map(str::trim) != Some(api_key.trim()) {
                return Err(format!(
                    "OpenCode provider '{}' already belongs to another API key",
                    provider_id
                ));
            }
        }
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    create_backup(config_path)?;

    config = apply_openai_compatible_provider_sync(
        config,
        provider_id,
        provider_name,
        proxy_url,
        api_key,
        models_to_sync,
    );

    atomically_write_config(config_path, &config)?;

    Ok(())
}

/// Precondition: the caller must already hold `OPENCODE_CONFIG_MUTEX`
/// (see `acquire_opencode_config_lock`). This function does not lock itself.
fn sync_accounts_file(accounts_path: &PathBuf) -> Result<(), String> {
    create_backup(accounts_path)?;

    // Read existing file for state preservation
    let existing_content = if accounts_path.exists() {
        fs::read_to_string(accounts_path).ok()
    } else {
        None
    };

    // Parse existing accounts for state preservation (match by refresh_token first, then email)
    let mut existing_accounts_by_refresh_token: HashMap<String, PluginAccount> = HashMap::new();
    let mut existing_accounts_by_email: HashMap<String, PluginAccount> = HashMap::new();
    let mut existing_active_index: i32 = 0;
    let mut existing_active_index_by_family: HashMap<String, i32> = HashMap::new();

    if let Some(ref content) = existing_content {
        if let Ok(existing_json) = serde_json::from_str::<Value>(content) {
            // Parse existing accounts
            if let Some(existing_accounts) =
                existing_json.get("accounts").and_then(|a| a.as_array())
            {
                for acc in existing_accounts {
                    if let Ok(plugin_acc) = serde_json::from_value::<PluginAccount>(acc.clone()) {
                        // Index by refresh_token (primary key for matching)
                        existing_accounts_by_refresh_token
                            .insert(plugin_acc.refresh_token.clone(), plugin_acc.clone());
                        // Index by email (fallback)
                        if let Some(email) = &plugin_acc.email {
                            existing_accounts_by_email.insert(email.clone(), plugin_acc);
                        }
                    }
                }
            }
            // Parse existing active indices
            if let Some(idx) = existing_json.get("activeIndex").and_then(|v| v.as_i64()) {
                existing_active_index = idx as i32;
            }
            if let Some(family_indices) = existing_json
                .get("activeIndexByFamily")
                .and_then(|v| v.as_object())
            {
                for (key, val) in family_indices {
                    if let Some(idx) = val.as_i64() {
                        existing_active_index_by_family.insert(key.clone(), idx as i32);
                    }
                }
            }
        }
    }

    let app_accounts = crate::modules::account::list_accounts()
        .map_err(|e| format!("Failed to list accounts: {}", e))?;

    let mut new_accounts: Vec<PluginAccount> = Vec::new();

    for acc in app_accounts {
        // Skip disabled accounts (preserve existing logic)
        if acc.disabled || acc.proxy_disabled {
            continue;
        }

        let refresh_token = acc.token.refresh_token.clone();
        let project_id = acc.token.project_id.clone();

        // Try to find existing account state (match by refresh_token first, then email fallback)
        let existing = existing_accounts_by_refresh_token
            .get(&refresh_token)
            .cloned()
            .or_else(|| existing_accounts_by_email.get(&acc.email).cloned());

        let plugin_account = if let Some(existing) = existing {
            // Preserve existing state
            PluginAccount {
                email: Some(acc.email),
                refresh_token,
                project_id,
                added_at: existing.added_at,
                last_used: existing.last_used.max(acc.last_used),
                rate_limit_reset_times: existing.rate_limit_reset_times,
                managed_project_id: existing.managed_project_id,
                enabled: existing.enabled,
                last_switch_reason: existing.last_switch_reason,
                cooling_down_until: existing.cooling_down_until,
                cooldown_reason: existing.cooldown_reason,
                fingerprint: existing.fingerprint,
                cached_quota: existing.cached_quota,
                cached_quota_updated_at: existing.cached_quota_updated_at,
                fingerprint_history: existing.fingerprint_history,
            }
        } else {
            // New account - use defaults
            let now = chrono::Utc::now().timestamp_millis();
            PluginAccount {
                email: Some(acc.email),
                refresh_token,
                project_id,
                added_at: now,
                last_used: acc.last_used,
                rate_limit_reset_times: None,
                managed_project_id: None,
                enabled: None,
                last_switch_reason: None,
                cooling_down_until: None,
                cooldown_reason: None,
                fingerprint: None,
                cached_quota: None,
                cached_quota_updated_at: None,
                fingerprint_history: None,
            }
        };

        new_accounts.push(plugin_account);
    }

    // Clamp activeIndex to valid range
    let account_count = new_accounts.len() as i32;
    let clamped_active_index = if account_count > 0 {
        existing_active_index.clamp(0, account_count - 1)
    } else {
        0
    };

    // Clamp activeIndexByFamily values
    let mut clamped_active_index_by_family = HashMap::new();
    for (family, idx) in existing_active_index_by_family {
        let clamped_idx = if account_count > 0 {
            idx.clamp(0, account_count - 1)
        } else {
            0
        };
        clamped_active_index_by_family.insert(family, clamped_idx);
    }

    // Ensure family indices always exist for plugin v3 behavior.
    if !clamped_active_index_by_family.contains_key("claude") {
        clamped_active_index_by_family.insert("claude".to_string(), clamped_active_index);
    }
    if !clamped_active_index_by_family.contains_key("gemini") {
        clamped_active_index_by_family.insert("gemini".to_string(), clamped_active_index);
    }

    // Build schema v3 output
    let new_data = PluginAccountsFile {
        version: 3,
        accounts: new_accounts,
        active_index: clamped_active_index,
        active_index_by_family: clamped_active_index_by_family,
    };

    let value = serde_json::to_value(&new_data)
        .map_err(|e| format!("Failed to serialize accounts: {}", e))?;
    atomically_write_config(accounts_path, &value)?;

    Ok(())
}
