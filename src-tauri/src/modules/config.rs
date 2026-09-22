use serde_json;
use std::fs;
use std::sync::{OnceLock, RwLock};

use super::account::get_data_dir;
use crate::models::AppConfig;
use tracing::{info, warn};

const CONFIG_FILE: &str = "gui_config.json";
const CONFIG_BAK_FILE: &str = "gui_config.json.bak";

static CONFIG_LOCK: OnceLock<RwLock<()>> = OnceLock::new();

fn config_lock() -> &'static RwLock<()> {
    CONFIG_LOCK.get_or_init(|| RwLock::new(()))
}

/// Load application configuration with self-healing and concurrency protection
pub fn load_app_config() -> Result<AppConfig, String> {
    let _read_guard = config_lock()
        .read()
        .map_err(|e| format!("config_lock_poisoned: {}", e))?;

    let data_dir = get_data_dir()?;
    let config_path = data_dir.join(CONFIG_FILE);
    let bak_path = data_dir.join(CONFIG_BAK_FILE);

    if !config_path.exists() {
        drop(_read_guard);
        let config = AppConfig::new();
        // [FIX #1460] Persist initial config to prevent new API Key on every refresh
        let _ = save_app_config(&config);
        return Ok(config);
    }

    let mut content = String::new();
    let mut read_err = None;
    for attempt in 0..3 {
        match fs::read_to_string(&config_path) {
            Ok(c) => {
                content = c;
                read_err = None;
                break;
            }
            Err(e) => {
                read_err = Some(e);
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
            }
        }
    }

    if let Some(e) = read_err {
        return Err(format!("failed_to_read_config_file: {}", e));
    }

    // [SELF-HEALING] If file is empty or only whitespace, restore from backup or generate default
    let trimmed = content.trim();
    if trimmed.is_empty() {
        warn!(
            "Empty gui_config.json encountered at {:?}. Initiating self-healing recovery...",
            config_path
        );

        if bak_path.exists() {
            if let Ok(bak_content) = fs::read_to_string(&bak_path) {
                let bak_trimmed = bak_content.trim();
                if !bak_trimmed.is_empty() {
                    if let Ok(mut bak_val) = serde_json::from_str::<serde_json::Value>(&bak_content)
                    {
                        let _ = migrate_config_value(&mut bak_val);
                        if let Ok(cfg) = serde_json::from_value::<AppConfig>(bak_val) {
                            info!(
                                "Successfully restored empty config from backup: {:?}",
                                bak_path
                            );
                            drop(_read_guard);
                            let _ = save_app_config(&cfg);
                            return Ok(cfg);
                        }
                    }
                }
            }
        }

        drop(_read_guard);
        let config = AppConfig::new();
        let _ = save_app_config(&config);
        return Ok(config);
    }

    let mut v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(val) => val,
        Err(parse_err) => {
            warn!(
                "Failed to parse gui_config.json ({:?}). Attempting self-healing recovery...",
                parse_err
            );

            // Attempt restore from backup
            if bak_path.exists() {
                if let Ok(bak_content) = fs::read_to_string(&bak_path) {
                    let bak_trimmed = bak_content.trim();
                    if !bak_trimmed.is_empty() {
                        if let Ok(mut bak_val) =
                            serde_json::from_str::<serde_json::Value>(&bak_content)
                        {
                            let _ = migrate_config_value(&mut bak_val);
                            if let Ok(cfg) = serde_json::from_value::<AppConfig>(bak_val) {
                                info!(
                                    "Successfully recovered corrupted config from backup: {:?}",
                                    bak_path
                                );
                                drop(_read_guard);
                                let _ = save_app_config(&cfg);
                                return Ok(cfg);
                            }
                        }
                    }
                }
            }

            // Archive corrupted file for user diagnostics
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let corrupt_path = data_dir.join(format!("gui_config.json.corrupt.{}", timestamp));
            let _ = fs::copy(&config_path, &corrupt_path);
            warn!(
                "Archived corrupted config to {:?}. Self-healing with default config.",
                corrupt_path
            );

            drop(_read_guard);
            let config = AppConfig::new();
            let _ = save_app_config(&config);
            return Ok(config);
        }
    };

    let modified = migrate_config_value(&mut v);

    let config: AppConfig = serde_json::from_value(v)
        .map_err(|e| format!("failed_to_convert_config_after_migration: {}", e))?;

    // If migration occurred, auto-save once to clean up the file
    if modified {
        drop(_read_guard);
        let _ = save_app_config(&config);
    }

    Ok(config)
}

/// Migrate configuration JSON values across versions.
/// Returns true if any value was modified and needs persistence.
pub fn migrate_config_value(v: &mut serde_json::Value) -> bool {
    let mut modified = false;

    // [MIGRATION] Default auto_sync to true for all existing users upgrading from legacy versions.
    // Legacy configs created before v4.35.0 had auto_sync: false by default.
    // When auto_sync_migrated is missing, we migrate auto_sync to true and stamp auto_sync_migrated: true.
    if v.get("auto_sync_migrated").is_none() {
        v["auto_sync"] = serde_json::Value::Bool(true);
        v["auto_sync_migrated"] = serde_json::Value::Bool(true);
        modified = true;
    }

    // Migration logic
    if let Some(proxy) = v.get_mut("proxy") {
        // [FIX #1738] Enhanced type checking for custom_mapping
        // Ensures the field is always parsed as an object, preventing type mismatch errors
        let mut custom_mapping = match proxy.get("custom_mapping") {
            Some(m) if m.is_object() => m.as_object().unwrap().clone(),
            Some(m) => {
                // If custom_mapping is not an object type (e.g., string), log warning and reset to empty
                tracing::warn!(
                    "Invalid custom_mapping type (expected object, got {:?}), resetting to empty",
                    m
                );
                serde_json::Map::new()
            }
            None => serde_json::Map::new(),
        };

        // Migrate Anthropic mapping
        if let Some(anthropic) = proxy
            .get_mut("anthropic_mapping")
            .and_then(|m| m.as_object_mut())
        {
            for (k, v) in anthropic.iter() {
                // Only move non-series fields, as series fields are now handled by Preset logic or builtin tables
                if !k.ends_with("-series") {
                    if !custom_mapping.contains_key(k) {
                        custom_mapping.insert(k.clone(), v.clone());
                    }
                }
            }
            // Remove old field
            proxy.as_object_mut().unwrap().remove("anthropic_mapping");
            modified = true;
        }

        // Migrate OpenAI mapping
        if let Some(openai) = proxy
            .get_mut("openai_mapping")
            .and_then(|m| m.as_object_mut())
        {
            for (k, v) in openai.iter() {
                if !k.ends_with("-series") {
                    if !custom_mapping.contains_key(k) {
                        custom_mapping.insert(k.clone(), v.clone());
                    }
                }
            }
            // Remove old field
            proxy.as_object_mut().unwrap().remove("openai_mapping");
            modified = true;
        }

        // Migrate log retention max_disk_mb: if 0, smoothly recover to 1024 MiB default
        if let Some(log_retention) = proxy
            .get_mut("log_retention")
            .and_then(|m| m.as_object_mut())
        {
            if let Some(max_disk_mb) = log_retention.get("max_disk_mb").and_then(|v| v.as_u64()) {
                if max_disk_mb == 0 {
                    log_retention.insert("max_disk_mb".to_string(), serde_json::Value::from(1024));
                    modified = true;
                }
            }
        }

        if modified {
            proxy.as_object_mut().unwrap().insert(
                "custom_mapping".to_string(),
                serde_json::Value::Object(custom_mapping),
            );
        }
    }

    modified
}

/// Save application configuration (atomic write with backup)
pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let _write_guard = config_lock()
        .write()
        .map_err(|e| format!("config_lock_poisoned: {}", e))?;

    let data_dir = get_data_dir()?;
    let config_path = data_dir.join(CONFIG_FILE);
    let bak_path = data_dir.join(CONFIG_BAK_FILE);

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("failed_to_serialize_config: {}", e))?;

    // If existing config is non-empty, create backup before atomic replacement
    if config_path.exists() {
        if let Ok(meta) = config_path.metadata() {
            if meta.len() > 0 {
                let _ = fs::copy(&config_path, &bak_path);
            }
        }
    }

    crate::utils::fs::write_atomic(&config_path, content.as_bytes())
        .map_err(|e| format!("failed_to_save_config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_load_app_config_self_heals_empty_string() {
        let _env_guard = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir =
            std::env::temp_dir().join(format!("test_config_empty_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let orig_env = std::env::var("ABV_DATA_DIR").ok();
        std::env::set_var("ABV_DATA_DIR", temp_dir.to_str().unwrap());

        // Create 0-byte file
        let cfg_path = temp_dir.join("gui_config.json");
        fs::write(&cfg_path, "").unwrap();

        let loaded = load_app_config().expect("Should self-heal empty config file");
        assert!(!loaded.proxy.api_key.is_empty());

        // Restore env
        if let Some(prev) = orig_env {
            std::env::set_var("ABV_DATA_DIR", prev);
        } else {
            std::env::remove_var("ABV_DATA_DIR");
        }
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_app_config_recovers_from_backup() {
        let _env_guard = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir =
            std::env::temp_dir().join(format!("test_config_bak_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let orig_env = std::env::var("ABV_DATA_DIR").ok();
        std::env::set_var("ABV_DATA_DIR", temp_dir.to_str().unwrap());

        // Corrupted main file, valid backup file
        let cfg_path = temp_dir.join("gui_config.json");
        let bak_path = temp_dir.join("gui_config.json.bak");
        fs::write(&cfg_path, "{ corrupted json").unwrap();

        let mut valid_config = AppConfig::new();
        valid_config.language = "zh-TW".to_string();
        let valid_json = serde_json::to_string_pretty(&valid_config).unwrap();
        fs::write(&bak_path, valid_json).unwrap();

        let loaded = load_app_config().expect("Should recover from backup file");
        assert_eq!(loaded.language, "zh-TW");

        // Restore env
        if let Some(prev) = orig_env {
            std::env::set_var("ABV_DATA_DIR", prev);
        } else {
            std::env::remove_var("ABV_DATA_DIR");
        }
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_migrate_auto_sync_unmigrated_sets_true() {
        let mut v = json!({
            "language": "en",
            "theme": "system",
            "auto_sync": false
        });
        let modified = migrate_config_value(&mut v);
        assert!(modified);
        assert!(v["auto_sync"].as_bool().unwrap_or(false));
        assert!(v["auto_sync_migrated"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_migrate_auto_sync_already_migrated_preserves_false() {
        let mut v = json!({
            "language": "en",
            "theme": "system",
            "auto_sync": false,
            "auto_sync_migrated": true
        });
        let modified = migrate_config_value(&mut v);
        assert!(!modified);
        assert!(!v["auto_sync"].as_bool().unwrap_or(true));
        assert!(v["auto_sync_migrated"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_migrate_auto_sync_missing_sets_true() {
        let mut v = json!({
            "language": "en",
            "theme": "system"
        });
        let modified = migrate_config_value(&mut v);
        assert!(modified);
        assert!(v["auto_sync"].as_bool().unwrap_or(false));
        assert!(v["auto_sync_migrated"].as_bool().unwrap_or(false));
    }
}
