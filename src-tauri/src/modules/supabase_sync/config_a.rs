use crate::error::AppError;
use crate::modules::account;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use crate::modules::*;
use base64::engine::general_purpose::STANDARD;
use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::*;

pub(crate) static GLOBAL_CONFIG: Lazy<Arc<RwLock<Option<SupabaseConfig>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Path to supabase_config.json
pub fn get_config_path() -> Result<PathBuf, AppError> {
    let data_dir = account::get_data_dir()
        .map_err(|e| AppError::Config(format!("Failed to get data directory: {}", e)))?;
    Ok(data_dir.join("supabase_config.json"))
}

/// Candidate locations for auto-discovering seed Supabase configuration
pub fn candidate_seed_config_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(dir_str) = std::env::var("REPO_SECRETS_DIR") {
        let p = PathBuf::from(&dir_str);
        candidates.push(p.join("02-antigravity-and-event-manager/vault/supabase_config.json"));
        candidates.push(p.join("02-antigravity-manager/vault/supabase_config.json"));
        candidates.push(p.join("03-supabase/01-own/supabase-credentials.json"));
        candidates.push(p.join("03-supabase/02-lovable/supabase-credentials.json"));
        candidates.push(p.join("vault/supabase_config.json"));
    }
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    if let Some(home) = dirs::home_dir() {
        candidates.push(
            home.join("repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json"),
        );
        candidates
            .push(home.join("repo-secrets/02-antigravity-manager/vault/supabase_config.json"));
        candidates.push(home.join("repo-secrets/03-supabase/01-own/supabase-credentials.json"));
        candidates.push(home.join("repo-secrets/03-supabase/02-lovable/supabase-credentials.json"));
        candidates.push(home.join(
            ".antigravity_tools/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
        ));
    }
    candidates
}

/// Candidate locations for auto-discovering repo-secrets Supabase credentials files
pub fn candidate_repo_secrets_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(dir_str) = std::env::var("REPO_SECRETS_DIR") {
        let p = PathBuf::from(&dir_str);
        if p.is_file() || p.extension().map_or(false, |ext| ext == "json") {
            candidates.push(p.clone());
        }
        candidates.push(p.join("02-antigravity-and-event-manager/vault/supabase_config.json"));
        candidates.push(p.join("02-antigravity-manager/vault/supabase_config.json"));
        candidates.push(p.join("03-supabase/01-own/supabase-credentials.json"));
        candidates.push(p.join("03-supabase/02-lovable/supabase-credentials.json"));
        candidates.push(p.join("supabase-credentials.json"));
    }

    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "C:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "../../repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/02-antigravity-manager/vault/supabase_config.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/03-supabase/01-own/supabase-credentials.json",
    ));
    candidates.push(PathBuf::from(
        "repo-secrets/03-supabase/02-lovable/supabase-credentials.json",
    ));

    if let Some(home) = dirs::home_dir() {
        candidates.push(
            home.join("repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json"),
        );
        candidates
            .push(home.join("repo-secrets/02-antigravity-manager/vault/supabase_config.json"));
        candidates.push(home.join("repo-secrets/03-supabase/01-own/supabase-credentials.json"));
        candidates.push(home.join("repo-secrets/03-supabase/02-lovable/supabase-credentials.json"));
        candidates.push(home.join(
            ".antigravity_tools/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
        ));
    }

    candidates
}

/// Auto-discover and seed Supabase endpoint configurations from repo-secrets credentials files
pub fn auto_seed_from_repo_secrets(cfg: &mut SupabaseConfig) -> bool {
    let mut seeded = false;
    let mut visited_paths = HashSet::new();

    for path in candidate_repo_secrets_paths() {
        if !path.exists() {
            continue;
        }

        // Canonicalize to avoid parsing identical files repeatedly
        let path_key = path.canonicalize().unwrap_or_else(|_| path.clone());
        if !visited_paths.insert(path_key) {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to read repo-secrets file at {:?}: {}", path, e);
                continue;
            }
        };

        let clean = content.trim_start_matches('\u{feff}');
        let raw_val: serde_json::Value = match serde_json::from_str(clean) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse JSON in repo-secrets file {:?}: {}",
                    path,
                    e
                );
                continue;
            }
        };

        // Determine if this is an envelope or raw JSON
        let (attrs_opt, val) = crate::modules::json_envelope::unpack_envelope(raw_val.clone());

        // If envelope metadata is present, check type compatibility
        if let Some(ref attrs) = attrs_opt {
            let dt = attrs.data_type.to_lowercase();
            if !dt.contains("supabase") && !dt.contains("credentials") && !dt.contains("endpoints")
            {
                continue;
            }
        }

        let is_b64 = attrs_opt
            .as_ref()
            .and_then(|a| a.encoding.as_deref())
            .map(|enc| enc.eq_ignore_ascii_case("base64"))
            .unwrap_or(false)
            || raw_val
                .get("encoding_format")
                .and_then(|f| f.as_str())
                .map(|f| f.eq_ignore_ascii_case("base64"))
                .unwrap_or(false);

        let decode_str = |val_ref: Option<&serde_json::Value>| -> Option<String> {
            let s = val_ref.and_then(|v| v.as_str())?;
            if is_b64 {
                if let Ok(b) = STANDARD.decode(s.trim()) {
                    if let Ok(utf) = String::from_utf8(b) {
                        return Some(utf.trim().to_string());
                    }
                }
            }
            Some(s.trim().to_string())
        };

        // Case 1: Credentials object
        let creds_obj = val
            .get("credentials")
            .and_then(|v| v.as_object())
            .or_else(|| val.as_object());

        if let Some(creds) = creds_obj {
            let endpoint_url = decode_str(creds.get("endpoint").or_else(|| creds.get("url")));
            let token = decode_str(creds.get("token").or_else(|| creds.get("api_key")));
            let service =
                decode_str(creds.get("service")).unwrap_or_else(|| "supabase-service".to_string());
            let role_opt = decode_str(creds.get("role"));
            let tag_opt = decode_str(creds.get("tag"));
            let notes_opt = decode_str(creds.get("notes"));

            if let (Some(url), Some(tok)) = (endpoint_url, token) {
                if !url.is_empty() && !tok.is_empty() {
                    let norm_url = normalize_supabase_url(&url);
                    let clean_id =
                        format!("ep-{}", service.to_lowercase().replace([' ', '_'], "-"));

                    let role = if let Some(r) = role_opt.filter(|r| !r.is_empty()) {
                        r.to_lowercase()
                    } else if service.to_lowercase().contains("root")
                        || service.to_lowercase().contains("lovable")
                    {
                        "root".to_string()
                    } else {
                        "secondary".to_string()
                    };

                    let is_root = role == "root";
                    let file_name = path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("credentials");

                    let mut tags = vec![role.clone()];
                    if let Some(ref t) = tag_opt {
                        if !tags.contains(t) {
                            tags.push(t.clone());
                        }
                    }
                    if !tags.contains(&service) {
                        tags.push(service.clone());
                    }

                    let notes = notes_opt.or_else(|| {
                        Some(format!("Auto-discovered from repo-secrets ({})", file_name))
                    });

                    let new_ep = SupabaseEndpoint {
                        id: clean_id.clone(),
                        name: format!("Supabase ({})", service),
                        url: norm_url.clone(),
                        api_key: tok,
                        role: role.clone(),
                        is_enabled: true,
                        prune_threshold_mb: if is_root { 400 } else { 200 },
                        priority: if is_root { 1 } else { 2 },
                        notes,
                        tags,
                    };

                    let already_exists = cfg
                        .endpoints
                        .iter()
                        .any(|e| normalize_supabase_url(&e.url) == norm_url || e.id == clean_id);

                    if !already_exists {
                        tracing::info!(
                            "Auto-discovered Supabase endpoint '{}' ({}) from {:?}",
                            clean_id,
                            norm_url,
                            path
                        );
                        cfg.endpoints.push(new_ep);
                        seeded = true;
                    }
                }
            }
        }

        // Case 2: Array of endpoints
        if let Some(arr) = val.get("endpoints").and_then(|v| v.as_array()) {
            if let Ok(eps) = serde_json::from_value::<Vec<SupabaseEndpoint>>(
                serde_json::Value::Array(arr.clone()),
            ) {
                for mut ep in eps {
                    ep.url = normalize_supabase_url(&ep.url);
                    let norm = ep.url.clone();
                    let id = ep.id.clone();
                    let already_exists = cfg
                        .endpoints
                        .iter()
                        .any(|e| normalize_supabase_url(&e.url) == norm || e.id == id);
                    if !already_exists {
                        cfg.endpoints.push(ep);
                        seeded = true;
                    }
                }
            }
        }
    }

    if seeded {
        cfg.endpoints.sort_by_key(|e| e.priority);
        cfg.is_sync_enabled = true;
    }

    seeded
}
