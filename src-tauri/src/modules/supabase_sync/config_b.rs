use crate::error::AppError;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use crate::modules::*;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use super::*;

/// Load configuration from disk
pub fn load_config() -> Result<SupabaseConfig, AppError> {
    let path = get_config_path()?;
    let mut is_new_config = false;
    let mut config = if !path.exists() {
        let mut loaded = None;
        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            let alt_path = PathBuf::from(appdata)
                .join("antigravity-manager")
                .join("supabase_config.json");
            if alt_path.exists() {
                if let Ok(data) = fs::read_to_string(&alt_path) {
                    let clean = data.trim_start_matches('\u{feff}');
                    if let Ok((mut cfg, _)) =
                        crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean)
                    {
                        for ep in &mut cfg.endpoints {
                            ep.url = normalize_supabase_url(&ep.url);
                        }
                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                        crate::error::record_ignored(save_config(&cfg), "save_config");
                        loaded = Some(cfg);
                    }
                }
            }
        }
        match loaded {
            Some(cfg) => cfg,
            None => {
                is_new_config = true;
                SupabaseConfig::default()
            }
        }
    } else {
        let data = fs::read_to_string(&path).map_err(|e| AppError::Io(e))?;
        let clean = data.trim_start_matches('\u{feff}');
        let mut cfg: SupabaseConfig =
            match crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean) {
                Ok((cfg, _)) => cfg,
                Err(_) => serde_json::from_str(clean).map_err(|e| {
                    AppError::Config(format!("Failed to parse Supabase config: {}", e))
                })?,
            };
        for ep in &mut cfg.endpoints {
            ep.url = normalize_supabase_url(&ep.url);
        }
        cfg
    };

    if config.endpoints.is_empty() {
        if auto_seed_from_repo_secrets(&mut config) {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(save_config(&config), "save_config");
        } else {
            for seed_path in candidate_seed_config_paths() {
                if seed_path.exists() {
                    if let Ok(content) = fs::read_to_string(&seed_path) {
                        let clean = content.trim_start_matches('\u{feff}');
                        let parsed =
                            crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean)
                                .map(|(cfg, _)| cfg)
                                .or_else(|_| {
                                    serde_json::from_str::<SupabaseConfig>(clean)
                                        .map_err(|e| e.to_string())
                                });
                        if let Ok(mut cfg) = parsed {
                            if !cfg.endpoints.is_empty() {
                                cfg.is_sync_enabled = true;
                                for ep in &mut cfg.endpoints {
                                    ep.url = normalize_supabase_url(&ep.url);
                                }
                                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                crate::error::record_ignored(save_config(&cfg), "save_config");
                                return Ok(cfg);
                            }
                        }
                    }
                }
            }
        }
    }

    if is_new_config {
        save_config(&config)?;
    }

    Ok(config)
}

/// Auto-discover Supabase credentials from repo-secrets, normalize URLs (stripping /rest/v1), enable sync, save, and return updated config.
pub fn auto_discover_supabase_credentials() -> crate::error::AppResult<SupabaseConfig> {
    let mut config = load_config().unwrap_or_default();
    // Justification: non-Result return value intentionally discarded — no error channel to track
    let _ = auto_seed_from_repo_secrets(&mut config);

    for seed_path in candidate_seed_config_paths() {
        if seed_path.exists() {
            if let Ok(content) = fs::read_to_string(&seed_path) {
                let clean = content.trim_start_matches('\u{feff}');
                let parsed =
                    crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean)
                        .map(|(cfg, _)| cfg)
                        .or_else(|_| {
                            serde_json::from_str::<SupabaseConfig>(clean).map_err(|e| e.to_string())
                        });
                if let Ok(seed_cfg) = parsed {
                    for ep in seed_cfg.endpoints {
                        let norm_url = normalize_supabase_url(&ep.url);
                        let exists = config
                            .endpoints
                            .iter()
                            .any(|e| normalize_supabase_url(&e.url) == norm_url || e.id == ep.id);
                        if !exists {
                            config.endpoints.push(ep);
                        }
                    }
                }
            }
        }
    }

    for ep in &mut config.endpoints {
        ep.url = normalize_supabase_url(&ep.url);
    }
    config.endpoints.sort_by_key(|e| e.priority);
    config.is_sync_enabled = true;
    save_config(&config)?;

    if !config.endpoints.is_empty() {
        start_sync_worker();
        crate::modules::supabase_pruner::start_pruner_worker();
    }

    Ok(config)
}

/// Export configuration as standard portable JSON envelope
pub fn export_config_json(config: &SupabaseConfig) -> Result<String, AppError> {
    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/supabase-endpoints", config.clone());
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| AppError::Config(format!("Failed to serialize Supabase config: {}", e)))
}

/// Save configuration to disk
pub fn save_config(config: &SupabaseConfig) -> Result<(), AppError> {
    let mut clean_config = config.clone();
    for ep in &mut clean_config.endpoints {
        ep.url = normalize_supabase_url(&ep.url);
    }
    let path = get_config_path()?;
    let data = serde_json::to_string_pretty(&clean_config)
        .map_err(|e| AppError::Config(format!("Failed to serialize Supabase config: {}", e)))?;
    fs::write(&path, &data).map_err(|e| AppError::Io(e))?;

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let alt_dir = PathBuf::from(appdata).join("antigravity-manager");
        if alt_dir.exists() {
            // Justification: best-effort file write; failure is logged and surfaces on the next read
            crate::error::record_ignored(
                fs::write(alt_dir.join("supabase_config.json"), &data),
                "fs::write",
            );
        }
    }

    Ok(())
}
