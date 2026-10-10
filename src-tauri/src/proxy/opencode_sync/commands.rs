use super::binary::check_opencode_installed;
use super::catalog::normalize_opencode_base_url;
use super::config_paths::{get_config_paths, parse_jsonc};
use super::dtos::{CanonicalFamilyDto, OpencodeStatus};
use super::lock::{
    acquire_opencode_config_lock, atomically_write_config, ANTIGRAVITY_ACCOUNTS_FILE,
    ANTIGRAVITY_CONFIG_FILE, ANTIGRAVITY_PROVIDER_ID, APIKEY_FUN_PROVIDER_ID, BACKUP_SUFFIX,
    OLD_BACKUP_SUFFIX, OPENCODE_CONFIG_FILE, OPENCODE_CONFIG_FILE_JSONC,
};
use super::models::ModelInput;
use super::sync::{sync_opencode_config, sync_opencode_openai_provider, validate_provider_id};
use super::sync_apply::{apply_clear_to_config, restore_opencode_config};
use super::sync_helpers::{create_backup, get_sync_status, restore_backup_to_target};
use crate::proxy::common::variant_mapping::GEMINI_FAMILIES;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;

pub fn read_opencode_config_content(file_name: Option<String>) -> Result<String, String> {
    let _lock = acquire_opencode_config_lock();

    let Some((opencode_path, ag_config_path, ag_accounts_path)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    // Allowlist of permitted file names
    let allowed_files = [
        OPENCODE_CONFIG_FILE,
        OPENCODE_CONFIG_FILE_JSONC,
        ANTIGRAVITY_CONFIG_FILE,
        ANTIGRAVITY_ACCOUNTS_FILE,
    ];

    // Determine which file to read. Both opencode.json and opencode.jsonc map to the
    // active opencode config path (which is resolved by probing the directory), so a
    // caller asking for "opencode.json" still gets the user's actual config when it
    // happens to be opencode.jsonc.
    let target_path = match file_name.as_deref() {
        Some(name) if name == ANTIGRAVITY_CONFIG_FILE => ag_config_path,
        Some(name) if name == ANTIGRAVITY_ACCOUNTS_FILE => ag_accounts_path,
        Some(name) if name == OPENCODE_CONFIG_FILE || name == OPENCODE_CONFIG_FILE_JSONC => {
            opencode_path
        }
        Some(name) => {
            return Err(format!(
                "Invalid file name: {}. Allowed: {:?}",
                name, allowed_files
            ))
        }
        None => opencode_path, // Default to the active opencode config (json or jsonc)
    };

    if !target_path.exists() {
        return Err(format!("Config file does not exist: {:?}", target_path));
    }

    fs::read_to_string(&target_path).map_err(|e| format!("Failed to read config: {}", e))
}

#[tauri::command]
pub async fn get_opencode_sync_status(proxy_url: String) -> Result<OpencodeStatus, String> {
    tokio::task::spawn_blocking(move || {
        let (installed, version) = check_opencode_installed();
        let (is_synced, has_backup, current_base_url) = get_sync_status(&proxy_url);

        Ok(OpencodeStatus {
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: vec![
                OPENCODE_CONFIG_FILE.to_string(),
                OPENCODE_CONFIG_FILE_JSONC.to_string(),
                ANTIGRAVITY_CONFIG_FILE.to_string(),
                ANTIGRAVITY_ACCOUNTS_FILE.to_string(),
            ],
        })
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute check".to_string()))
}

#[tauri::command]
pub fn get_canonical_families() -> Vec<CanonicalFamilyDto> {
    GEMINI_FAMILIES
        .iter()
        .map(|family| {
            let mut normalized_match_ids = HashSet::new();
            let mut match_ids = Vec::new();

            for match_id in std::iter::once(family.canonical_id)
                .chain(family.aliases.iter().map(|(alias, _)| *alias))
                .chain(family.tiers.iter().map(|(_, spec)| spec.id))
            {
                if normalized_match_ids.insert(match_id.to_lowercase()) {
                    match_ids.push(match_id.to_string());
                }
            }

            CanonicalFamilyDto {
                canonical_id: family.canonical_id.to_string(),
                display_name: family.display_name.to_string(),
                match_ids,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn execute_opencode_sync(
    proxy_url: String,
    api_key: String,
    sync_accounts: Option<bool>,
    models: Option<Vec<ModelInput>>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        sync_opencode_config(&proxy_url, &api_key, sync_accounts.unwrap_or(false), models)
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute sync".to_string()))
}

#[tauri::command]
pub async fn execute_opencode_openai_sync(
    proxy_url: String,
    api_key: String,
    provider_id: Option<String>,
    provider_name: Option<String>,
    models: Option<Vec<ModelInput>>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        sync_opencode_openai_provider(
            provider_id
                .as_deref()
                .filter(|id| !id.trim().is_empty())
                .unwrap_or(APIKEY_FUN_PROVIDER_ID),
            provider_name
                .as_deref()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or("APIKEY.FUN"),
            &proxy_url,
            &api_key,
            models,
        )
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute sync".to_string()))
}

#[tauri::command]
pub async fn execute_opencode_restore() -> Result<(), String> {
    tokio::task::spawn_blocking(move || restore_opencode_config())
        .await
        .unwrap_or_else(|_| Err("Failed to execute restore".to_string()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOpencodeConfigRequest {
    pub file_name: Option<String>,
}

#[tauri::command]
pub async fn get_opencode_config_content(
    request: GetOpencodeConfigRequest,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || read_opencode_config_content(request.file_name))
        .await
        .unwrap_or_else(|_| Err("Failed to read config".to_string()))
}

/// List of Antigravity model IDs that may have been added to legacy providers
const ANTIGRAVITY_MODEL_IDS: &[&str] = &[
    "claude-sonnet-4-6",
    "claude-sonnet-4-6-thinking",
    "claude-sonnet-4-5",
    "claude-sonnet-4-5-thinking",
    "claude-opus-4-5-thinking",
    "gemini-3.1-pro-high",
    "gemini-3.1-pro-low",
    "gemini-3-pro-high",
    "gemini-3-pro-low",
    "gemini-3-flash",
    "gemini-3-pro-image",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-thinking",
    "gemini-2.5-pro",
];

/// Check if a base URL matches the proxy URL (supports both with and without /v1)
pub(crate) fn base_url_matches(config_url: &str, proxy_url: &str) -> bool {
    let normalized_config = normalize_opencode_base_url(config_url);
    let normalized_proxy = normalize_opencode_base_url(proxy_url);
    normalized_config == normalized_proxy
}

/// Clear OpenCode config by removing antigravity-manager provider and optionally cleaning up legacy entries
pub(crate) fn clear_opencode_config(
    proxy_url: Option<String>,
    clear_legacy: bool,
) -> Result<(), String> {
    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _, accounts_path)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    // Process opencode.json
    if config_path.exists() {
        // Create backup before modifying
        create_backup(&config_path)?;

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        // Tolerate JSONC (comments + trailing commas) when the user's config is opencode.jsonc.
        let config: Value = parse_jsonc(&content)
            .filter(Value::is_object)
            .ok_or_else(|| {
                "OpenCode config must be a valid JSON/JSONC object; file left unchanged".to_string()
            })?;
        let config = apply_clear_to_config(config, proxy_url.as_deref(), clear_legacy);

        atomically_write_config(&config_path, &config)?;
    }

    // Process antigravity-accounts.json
    let accounts_backup_new =
        accounts_path.with_file_name(format!("{}{}", ANTIGRAVITY_ACCOUNTS_FILE, BACKUP_SUFFIX));
    let accounts_backup_old = accounts_path.with_file_name(format!(
        "{}{}",
        ANTIGRAVITY_ACCOUNTS_FILE, OLD_BACKUP_SUFFIX
    ));

    if accounts_backup_new.exists() {
        // Restore from new backup
        restore_backup_to_target(&accounts_backup_new, &accounts_path, "accounts from backup")?;
    } else if accounts_backup_old.exists() {
        // Restore from old backup
        restore_backup_to_target(
            &accounts_backup_old,
            &accounts_path,
            "accounts from old backup",
        )?;
    } else if accounts_path.exists() {
        // No backup found, delete the file
        fs::remove_file(&accounts_path)
            .map_err(|e| format!("Failed to remove accounts file: {}", e))?;
    }

    Ok(())
}

/// Cleanup legacy provider entries (anthropic/google) that were configured by old versions
pub(crate) fn cleanup_legacy_provider(provider: &mut Value, proxy_url: &str) {
    if let Some(provider_obj) = provider.as_object_mut() {
        // Remove Antigravity model IDs from models list.
        let remove_models_key = if let Some(models) = provider_obj
            .get_mut("models")
            .and_then(|m| m.as_object_mut())
        {
            for model_id in ANTIGRAVITY_MODEL_IDS {
                models.remove(*model_id);
            }
            models.is_empty()
        } else {
            false
        };
        if remove_models_key {
            provider_obj.remove("models");
        }

        // Check and remove options.baseURL and options.apiKey if baseURL matches proxy.
        let remove_options_key = if let Some(options) = provider_obj
            .get_mut("options")
            .and_then(|o| o.as_object_mut())
        {
            let should_cleanup = options
                .get("baseURL")
                .and_then(|v| v.as_str())
                .map(|base_url| base_url_matches(base_url, proxy_url))
                .unwrap_or(false);

            if should_cleanup {
                options.remove("baseURL");
                options.remove("apiKey");
            }
            options.is_empty()
        } else {
            false
        };
        if remove_options_key {
            provider_obj.remove("options");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpencodeProviderSummary {
    pub id: String,
    pub name: Option<String>,
    pub npm: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub models: Vec<String>,
}

pub fn extract_providers_from_config(config: &Value) -> Vec<OpencodeProviderSummary> {
    let mut result = Vec::new();
    if let Some(providers) = config.get("provider").and_then(Value::as_object) {
        for (id, val) in providers {
            let Some(val_obj) = val.as_object() else {
                continue;
            };
            let name = val_obj
                .get("name")
                .and_then(Value::as_str)
                .map(String::from);
            let npm = val_obj.get("npm").and_then(Value::as_str).map(String::from);
            let options = val_obj.get("options");
            let base_url = options
                .and_then(|o| o.get("baseURL"))
                .and_then(Value::as_str)
                .map(String::from);
            let api_key = options
                .and_then(|o| o.get("apiKey"))
                .and_then(Value::as_str)
                .map(String::from);

            let mut models = Vec::new();
            if let Some(models_obj) = val_obj.get("models").and_then(Value::as_object) {
                models = models_obj.keys().cloned().collect();
                models.sort();
            }

            result.push(OpencodeProviderSummary {
                id: id.clone(),
                name,
                npm,
                base_url,
                api_key,
                models,
            });
        }
    }

    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

pub fn read_opencode_providers() -> Result<Vec<OpencodeProviderSummary>, String> {
    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _, _)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read OpenCode config: {}", e))?;
    let config: Value = parse_jsonc(&content)
        .filter(Value::is_object)
        .ok_or_else(|| "OpenCode config is not valid JSON/JSONC".to_string())?;

    Ok(extract_providers_from_config(&config))
}

pub fn apply_remove_provider(mut config: Value, provider_id: &str) -> (Value, bool) {
    let provider_id = provider_id.trim();
    let mut changed = false;
    if let Some(providers) = config.get_mut("provider").and_then(Value::as_object_mut) {
        if providers.remove(provider_id).is_some() {
            changed = true;
            if providers.is_empty() {
                if let Some(config_obj) = config.as_object_mut() {
                    config_obj.remove("provider");
                }
            }
        }
    }
    (config, changed)
}

pub fn remove_opencode_provider(provider_id: &str) -> Result<(), String> {
    let provider_id = provider_id.trim();
    validate_provider_id(provider_id)?;

    if provider_id == ANTIGRAVITY_PROVIDER_ID {
        return Err(format!(
            "Provider '{}' is reserved and cannot be removed; use clear instead",
            ANTIGRAVITY_PROVIDER_ID
        ));
    }

    // Only profiles managed by this feature may be removed.
    if provider_id != APIKEY_FUN_PROVIDER_ID
        && !provider_id.starts_with(&format!("{}-", APIKEY_FUN_PROVIDER_ID))
    {
        return Err(format!(
            "Provider '{}' is not managed by Antigravity-Manager and cannot be removed",
            provider_id
        ));
    }

    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _, _)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read OpenCode config: {}", e))?;
    let config: Value = parse_jsonc(&content)
        .filter(Value::is_object)
        .ok_or_else(|| "OpenCode config must be a valid JSON/JSONC object".to_string())?;

    let (updated_config, changed) = apply_remove_provider(config, provider_id);

    if changed {
        create_backup(&config_path)?;
        atomically_write_config(&config_path, &updated_config)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_opencode_providers() -> Result<Vec<OpencodeProviderSummary>, String> {
    tokio::task::spawn_blocking(read_opencode_providers)
        .await
        .unwrap_or_else(|_| Err("Failed to read OpenCode providers".to_string()))
}

#[tauri::command]
pub async fn execute_opencode_remove_provider(provider_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || remove_opencode_provider(&provider_id))
        .await
        .unwrap_or_else(|_| Err("Failed to execute remove provider".to_string()))
}

#[tauri::command]
pub async fn execute_opencode_clear(
    proxy_url: Option<String>,
    clear_legacy: Option<bool>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        clear_opencode_config(proxy_url, clear_legacy.unwrap_or(false))
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute clear".to_string()))
}
