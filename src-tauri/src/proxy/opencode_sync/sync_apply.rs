use super::catalog::{build_model_catalog, normalize_opencode_base_url, ModelDef};
use super::commands::cleanup_legacy_provider;
use super::config_paths::get_config_paths;
use super::lock::{
    acquire_opencode_config_lock, ANTIGRAVITY_ACCOUNTS_FILE, ANTIGRAVITY_PROVIDER_ID,
    BACKUP_SUFFIX, OLD_BACKUP_SUFFIX, OPENAI_COMPATIBLE_NPM, OPENCODE_CONFIG_FILE,
};
use super::models::{
    build_fallback_model_json, lookup_catalog_model, merge_catalog_models,
    migrate_gemini_alias_models, ModelInput,
};
use super::sync_helpers::{
    build_model_json, ensure_object, ensure_provider_object, ensure_provider_string_field,
    merge_provider_options, restore_backup_to_target,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn restore_opencode_config() -> Result<(), String> {
    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _, accounts_path)) = get_config_paths() else {
        return Err("Failed to get OpenCode config directory".to_string());
    };

    let mut restored = false;

    // Backups are named after the config file they protected. A user may have been
    // using opencode.json or opencode.jsonc, and the active path may now differ from
    // whichever backup exists. Look for backups under both file names, preferring the
    // active one, and restore the backup to its original (backup-name minus suffix)
    // target path so we don't resurrect a stale parallel file.
    let dir = config_path.parent();
    let active_file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(OPENCODE_CONFIG_FILE);
    let config_candidates: [(&str, &str); 4] = [
        (active_file_name, BACKUP_SUFFIX),
        (active_file_name, OLD_BACKUP_SUFFIX),
        (OPENCODE_CONFIG_FILE, BACKUP_SUFFIX),
        (OPENCODE_CONFIG_FILE, OLD_BACKUP_SUFFIX),
    ];
    for (file_name, suffix) in &config_candidates {
        let backup_path = config_path.with_file_name(format!("{}{}", file_name, suffix));
        if backup_path.exists() {
            let target = dir
                .map(|d| d.join(file_name))
                .unwrap_or_else(|| config_path.clone());
            restore_backup_to_target(&backup_path, &target, "config")?;
            restored = true;
            break;
        }
    }

    // Try new backup suffix first, fall back to old suffix for backward compatibility
    let accounts_backup_new =
        accounts_path.with_file_name(format!("{}{}", ANTIGRAVITY_ACCOUNTS_FILE, BACKUP_SUFFIX));
    let accounts_backup_old = accounts_path.with_file_name(format!(
        "{}{}",
        ANTIGRAVITY_ACCOUNTS_FILE, OLD_BACKUP_SUFFIX
    ));

    if accounts_backup_new.exists() {
        restore_backup_to_target(&accounts_backup_new, &accounts_path, "accounts")?;
        restored = true;
    } else if accounts_backup_old.exists() {
        restore_backup_to_target(&accounts_backup_old, &accounts_path, "accounts")?;
        restored = true;
    }

    if restored {
        Ok(())
    } else {
        Err("No backup files found".to_string())
    }
}

/// Pure function: Apply sync logic to config JSON
/// Returns the modified config Value
pub(crate) fn apply_sync_to_config(
    mut config: Value,
    proxy_url: &str,
    api_key: &str,
    models_to_sync: Option<&[ModelInput]>,
) -> Value {
    if !config.is_object() {
        config = serde_json::json!({});
    }

    if config.get("$schema").is_none() {
        config["$schema"] = Value::String("https://opencode.ai/config.json".to_string());
    }

    let normalized_url = normalize_opencode_base_url(proxy_url);

    ensure_object(&mut config, "provider");

    if let Some(provider) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
        ensure_provider_object(provider, ANTIGRAVITY_PROVIDER_ID);
        if let Some(ag_provider) = provider.get_mut(ANTIGRAVITY_PROVIDER_ID) {
            ensure_provider_string_field(ag_provider, "npm", "@ai-sdk/anthropic");
            ensure_provider_string_field(ag_provider, "name", "Antigravity Manager");
            merge_provider_options(ag_provider, &normalized_url, api_key);
            migrate_gemini_alias_models(ag_provider);
            merge_catalog_models(ag_provider, models_to_sync);
        }
    }

    config
}

/// Replace the provider's model list with the given inputs. The list mirrors the
/// models actually exposed by the upstream key, so models absent from the input are
/// dropped (unlike the Antigravity sync which merges). Known catalog ids still get
/// full catalog metadata, and user-defined fields on surviving models are preserved.
fn replace_provider_models(provider: &mut Value, model_inputs: Option<&[ModelInput]>) {
    if provider.get("models").is_none() {
        provider["models"] = serde_json::json!({});
    }

    // An absent or empty list means "keep whatever is there" — e.g. the user synced
    // before querying models, or called the HTTP API with no models field.
    let Some(inputs) = model_inputs else {
        return;
    };
    if inputs.is_empty() {
        return;
    }

    let catalog = build_model_catalog();
    let catalog_map: HashMap<&str, &ModelDef> = catalog.iter().map(|m| (m.id, m)).collect();
    let existing_models: serde_json::Map<String, Value> = provider
        .get("models")
        .and_then(|m| m.as_object())
        .cloned()
        .unwrap_or_default();

    let mut models = serde_json::Map::new();
    for input in inputs {
        let model_id = input.id.trim();
        if model_id.is_empty() {
            continue;
        }
        let entry = match lookup_catalog_model(&catalog_map, model_id) {
            Some(model_def) => {
                let catalog_model = build_model_json(model_def);
                match existing_models.get(model_id) {
                    Some(existing) if existing.is_object() => {
                        let mut merged = existing.as_object().unwrap().clone();
                        if let Some(catalog_obj) = catalog_model.as_object() {
                            for (key, value) in catalog_obj {
                                merged.insert(key.clone(), value.clone());
                            }
                        }
                        Value::Object(merged)
                    }
                    _ => catalog_model,
                }
            }
            None => {
                // Unknown upstream models often have manually configured limits,
                // tool support, or options that cannot be recovered from the catalog.
                let mut entry = existing_models
                    .get(model_id)
                    .and_then(Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                if let Value::Object(defaults) =
                    build_fallback_model_json(model_id, input.name.as_deref())
                {
                    for (key, value) in defaults {
                        entry.entry(key).or_insert(value);
                    }
                }
                Value::Object(entry)
            }
        };
        models.insert(model_id.to_string(), entry);
    }
    provider["models"] = Value::Object(models);
}

pub(crate) fn apply_openai_compatible_provider_sync(
    mut config: Value,
    provider_id: &str,
    provider_name: &str,
    proxy_url: &str,
    api_key: &str,
    models_to_sync: Option<&[ModelInput]>,
) -> Value {
    if !config.is_object() {
        config = serde_json::json!({});
    }

    if config.get("$schema").is_none() {
        config["$schema"] = Value::String("https://opencode.ai/config.json".to_string());
    }

    let normalized_url = normalize_opencode_base_url(proxy_url);
    let display_name = if provider_name.trim().is_empty() {
        "APIKEY.FUN"
    } else {
        provider_name.trim()
    };

    ensure_object(&mut config, "provider");

    if let Some(provider) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
        ensure_provider_object(provider, provider_id);
        if let Some(target) = provider.get_mut(provider_id) {
            ensure_provider_string_field(target, "npm", OPENAI_COMPATIBLE_NPM);
            ensure_provider_string_field(target, "name", display_name);
            ensure_object(target, "options");
            merge_provider_options(target, &normalized_url, api_key);
            replace_provider_models(target, models_to_sync);
        }
    }

    config
}

/// Pure function: Apply clear logic to config JSON
/// Returns the modified config Value
pub(crate) fn apply_clear_to_config(
    mut config: Value,
    proxy_url: Option<&str>,
    clear_legacy: bool,
) -> Value {
    if let Some(provider) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
        // 1. Remove antigravity-manager provider
        provider.remove(ANTIGRAVITY_PROVIDER_ID);

        // 2. Cleanup legacy entries if requested
        if clear_legacy {
            if let Some(proxy) = proxy_url {
                // Clean up provider.anthropic
                if let Some(anthropic) = provider.get_mut("anthropic") {
                    cleanup_legacy_provider(anthropic, proxy);
                }

                // Clean up provider.google
                if let Some(google) = provider.get_mut("google") {
                    cleanup_legacy_provider(google, proxy);
                }
            }
        }

        // Remove empty provider object if it has no entries
        if provider.is_empty() {
            if let Some(config_obj) = config.as_object_mut() {
                config_obj.remove("provider");
            }
        }
    }

    config
}
