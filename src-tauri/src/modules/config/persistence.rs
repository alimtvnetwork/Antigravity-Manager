use crate::models::AppConfig;
use crate::modules::account::get_data_dir;
use std::fs;

use super::*;

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
                // Justification: best-effort file copy; logged for diagnosis
                crate::error::record_ignored(fs::copy(&config_path, &bak_path), "fs::copy");
            }
        }
    }

    crate::utils::fs::write_atomic(&config_path, content.as_bytes())
        .map_err(|e| format!("failed_to_save_config: {}", e))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    pub(crate) fn test_load_app_config_self_heals_empty_string() {
        let _env_guard = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir =
            std::env::temp_dir().join(format!("test_config_empty_{}", uuid::Uuid::new_v4()));
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create_dir_all");
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
        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "remove_dir_all");
    }

    #[test]
    pub(crate) fn test_load_app_config_recovers_from_backup() {
        let _env_guard = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir =
            std::env::temp_dir().join(format!("test_config_bak_{}", uuid::Uuid::new_v4()));
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create_dir_all");
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
        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "remove_dir_all");
    }

    #[test]
    pub(crate) fn test_migrate_auto_sync_unmigrated_sets_true() {
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
    pub(crate) fn test_migrate_auto_sync_already_migrated_preserves_false() {
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
    pub(crate) fn test_migrate_auto_sync_missing_sets_true() {
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
