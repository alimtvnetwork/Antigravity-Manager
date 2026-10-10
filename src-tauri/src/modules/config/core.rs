use crate::models::AppConfig;
use crate::modules::account::get_data_dir;
use std::fs;
use std::sync::{OnceLock, RwLock};
use tracing::{info, warn};

use super::*;

pub(crate) const CONFIG_FILE: &str = "gui_config.json";

pub(crate) const CONFIG_BAK_FILE: &str = "gui_config.json.bak";

pub(crate) static CONFIG_LOCK: OnceLock<RwLock<()>> = OnceLock::new();

pub(crate) fn config_lock() -> &'static RwLock<()> {
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
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(save_app_config(&config), "save_app_config");
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
                        // Justification: non-Result return value intentionally discarded — no error channel to track
                        let _ = migrate_config_value(&mut bak_val);
                        if let Ok(cfg) = serde_json::from_value::<AppConfig>(bak_val) {
                            info!(
                                "Successfully restored empty config from backup: {:?}",
                                bak_path
                            );
                            drop(_read_guard);
                            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                            crate::error::record_ignored(save_app_config(&cfg), "save_app_config");
                            return Ok(cfg);
                        }
                    }
                }
            }
        }

        drop(_read_guard);
        let config = AppConfig::new();
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(save_app_config(&config), "save_app_config");
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
                            // Justification: non-Result return value intentionally discarded — no error channel to track
                            let _ = migrate_config_value(&mut bak_val);
                            if let Ok(cfg) = serde_json::from_value::<AppConfig>(bak_val) {
                                info!(
                                    "Successfully recovered corrupted config from backup: {:?}",
                                    bak_path
                                );
                                drop(_read_guard);
                                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                crate::error::record_ignored(
                                    save_app_config(&cfg),
                                    "save_app_config",
                                );
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
            // Justification: best-effort file copy; logged for diagnosis
            crate::error::record_ignored(fs::copy(&config_path, &corrupt_path), "fs::copy");
            warn!(
                "Archived corrupted config to {:?}. Self-healing with default config.",
                corrupt_path
            );

            drop(_read_guard);
            let config = AppConfig::new();
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(save_app_config(&config), "save_app_config");
            return Ok(config);
        }
    };

    let (attrs_opt, unwrapped_v) = crate::modules::json_envelope::unpack_envelope(v.clone());
    let mut target_v = if attrs_opt.is_some() { unwrapped_v } else { v };

    let modified = migrate_config_value(&mut target_v);

    let config: AppConfig = serde_json::from_value(target_v)
        .map_err(|e| format!("failed_to_convert_config_after_migration: {}", e))?;

    // If migration occurred, auto-save once to clean up the file
    if modified {
        drop(_read_guard);
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(save_app_config(&config), "save_app_config");
    }

    Ok(config)
}
