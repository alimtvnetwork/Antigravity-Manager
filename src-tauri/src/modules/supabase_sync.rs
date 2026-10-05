//! Supabase Synchronization & Node Registry Module
//! Manages local node identity, heartbeat pings, and instance profile state sync.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::account;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

static PROCESS_START_TIME: Lazy<AtomicU64> =
    Lazy::new(|| AtomicU64::new(Utc::now().timestamp() as u64));

static SYNC_RUNNING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Supabase sync global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub endpoints: Vec<SupabaseEndpoint>,
    pub node_alias: String,
    pub is_sync_enabled: bool,
    pub auto_prune_root_mb: u64,
    pub auto_prune_secondary_mb: u64,
    pub heartbeat_interval_secs: u64,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        let node_id = get_local_node_id();
        let short_id = if node_id.len() > 6 {
            &node_id[..6]
        } else {
            &node_id
        };
        Self {
            endpoints: Vec::new(),
            node_alias: format!("Node-{}", short_id),
            is_sync_enabled: false,
            auto_prune_root_mb: 400,
            auto_prune_secondary_mb: 200,
            heartbeat_interval_secs: 30,
        }
    }
}

/// Global shared configuration state
static GLOBAL_CONFIG: Lazy<Arc<RwLock<Option<SupabaseConfig>>>> =
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
        candidates.push(
            home.join(
                ".antigravity_tools/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
            ),
        );
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
        candidates.push(
            home.join(
                ".antigravity_tools/repo-secrets/02-antigravity-manager/vault/supabase_config.json",
            ),
        );
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
                        let _ = save_config(&cfg);
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
            let _ = save_config(&config);
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
                                let _ = save_config(&cfg);
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
            let _ = fs::write(alt_dir.join("supabase_config.json"), &data);
        }
    }

    Ok(())
}

/// Retrieve stable local node ID
pub fn get_local_node_id() -> String {
    machine_uid::get().unwrap_or_else(|_| "agm-node-local".to_string())
}

/// Retrieve local machine IP address
pub fn get_local_ip() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Uptime of this node in seconds
pub fn get_uptime_seconds() -> u64 {
    let start = PROCESS_START_TIME.load(Ordering::Relaxed);
    let now = Utc::now().timestamp() as u64;
    if now > start {
        now - start
    } else {
        0
    }
}

/// Send single heartbeat to the Root DB endpoint
pub async fn send_heartbeat(endpoint: &SupabaseEndpoint, node_alias: &str) -> Result<(), AppError> {
    let client = SupabaseClient::new(endpoint)?;
    let node_id = get_local_node_id();
    let local_ip = get_local_ip();
    let uptime = get_uptime_seconds();
    let now = Utc::now().timestamp();

    // Calculate active project count from instance registry
    let mut project_count = 1;
    let mut instances_to_sync = Vec::new();
    if let Ok(reg) = instance::load_registry() {
        project_count = reg.instances.len() as i32;
        instances_to_sync = reg.instances;
    }

    // 1. Upsert node record
    let node_payload = json!({
        "id": node_id,
        "alias": node_alias,
        "ip_address": local_ip,
        "uptime_seconds": uptime,
        "project_count": project_count,
        "last_heartbeat_at": now,
        "status": "online"
    });

    client.upsert("nodes", node_payload, "id").await?;

    // 2. Upsert instance profiles (Child of this node)
    for inst in instances_to_sync {
        let is_active = inst.is_default;
        let profile_id = format!("{}_{}", node_id, inst.id);
        let active_acc_id = inst.bound_account_id.clone().unwrap_or_default();
        let active_acc_email = inst.bound_email.clone().unwrap_or_default();
        let profile_payload = json!({
            "id": profile_id,
            "node_id": node_id,
            "profile_name": inst.name,
            "active_account_id": active_acc_id,
            "active_account_email": active_acc_email,
            "is_active": is_active,
            "quota_percent": 100,
            "status": if is_active { "running" } else { "idle" },
            "updated_at": now
        });
        let _ = client
            .upsert("instance_profiles", profile_payload, "id")
            .await;
    }

    Ok(())
}

fn resolve_sync_target(
    instance_id: &str,
) -> Result<(SupabaseEndpoint, instance::InstanceConfig, SupabaseConfig), AppError> {
    let config = load_config()?;
    let endpoint = config
        .endpoints
        .iter()
        .find(|ep| ep.is_enabled && ep.role == "root")
        .cloned()
        .ok_or_else(|| AppError::Config("No enabled root Supabase endpoint".to_string()))?;
    let registry = instance::load_registry().map_err(|e| AppError::Config(e.to_string()))?;
    let inst = registry
        .instances
        .into_iter()
        .find(|item| item.id == instance_id)
        .ok_or_else(|| AppError::Config(format!("Instance '{}' is not registered", instance_id)))?;
    Ok((endpoint, inst, config))
}

fn build_sync_payloads(
    config: &SupabaseConfig,
    inst: &instance::InstanceConfig,
    profile_id: &str,
    email: &str,
) -> (serde_json::Value, serde_json::Value) {
    let now = Utc::now().timestamp();
    let node_id = get_local_node_id();
    let node_payload = json!({
        "id": node_id,
        "alias": config.node_alias,
        "ip_address": get_local_ip(),
        "uptime_seconds": get_uptime_seconds(),
        "project_count": 1,
        "last_heartbeat_at": now,
        "status": "online"
    });
    let profile_payload = json!({
        "id": profile_id,
        "node_id": node_id,
        "profile_name": inst.name,
        "active_account_id": inst.bound_account_id.clone().unwrap_or_default(),
        "active_account_email": email,
        "is_active": inst.is_default,
        "quota_percent": 100,
        "status": if inst.is_default { "running" } else { "idle" },
        "updated_at": now
    });
    (node_payload, profile_payload)
}

async fn sync_node_and_profile(
    client: &SupabaseClient,
    config: &SupabaseConfig,
    inst: &instance::InstanceConfig,
    profile_id: &str,
    email: &str,
) -> Result<(), AppError> {
    let (node_payload, profile_payload) = build_sync_payloads(config, inst, profile_id, email);
    let node_written = client.upsert("nodes", node_payload, "id").await?;
    if let Some(message) = postgrest_error_message(&node_written) {
        return Err(AppError::Network(
            format!("Supabase node upsert failed: {}", message),
            None,
        ));
    }
    let written = client
        .upsert("instance_profiles", profile_payload, "id")
        .await
        .map_err(|e| AppError::Network(format!("Supabase upsert failed: {}", e), None))?;
    if let Some(message) = postgrest_error_message(&written) {
        return Err(AppError::Network(
            format!("Supabase profile upsert failed: {}", message),
            None,
        ));
    }
    Ok(())
}

async fn read_back_profile_email(
    client: &SupabaseClient,
    profile_id: &str,
) -> Result<String, AppError> {
    let rows = client
        .select(
            "instance_profiles",
            &format!("id=eq.{}&select=active_account_email", profile_id),
        )
        .await?;
    if let Some(message) = postgrest_error_message(&rows) {
        return Err(AppError::Network(
            format!("Supabase profile read failed: {}", message),
            None,
        ));
    }
    email_from_postgrest(&rows).ok_or_else(|| {
        AppError::Network(
            format!(
                "Supabase profile read-back returned no record for '{}'",
                profile_id
            ),
            None,
        )
    })
}

/// Upsert one instance profile to the root Supabase endpoint and read the stored email back.
pub async fn push_and_read_instance_email(instance_id: &str) -> Result<String, AppError> {
    let (endpoint, inst, config) = resolve_sync_target(instance_id)?;
    let client = SupabaseClient::new(&endpoint)?;
    let profile_id = format!("{}_{}", get_local_node_id(), inst.id);
    let email = inst.bound_email.clone().unwrap_or_default();

    sync_node_and_profile(&client, &config, &inst, &profile_id, &email).await?;
    let stored = read_back_profile_email(&client, &profile_id).await?;

    if !stored.eq_ignore_ascii_case(&email) {
        return Err(AppError::Config(format!(
            "Supabase stored '{}' for profile '{}' but the local instance is bound to '{}'",
            stored, profile_id, email
        )));
    }
    Ok(format!("profile {} email {} confirmed", profile_id, stored))
}

fn postgrest_error_message(value: &serde_json::Value) -> Option<String> {
    let code = value
        .get("code")
        .or_else(|| value.get("error_code"))
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let message_opt = value
        .get("message")
        .or_else(|| value.get("error"))
        .or_else(|| value.get("msg"))
        .and_then(|item| item.as_str());

    match (message_opt, code.is_empty()) {
        (Some(msg), false) => Some(format!("{} ({})", msg, code)),
        (Some(msg), true) => Some(msg.to_string()),
        (None, false) => Some(format!("PostgREST error ({})", code)),
        (None, true) => None,
    }
}

fn email_from_postgrest(value: &serde_json::Value) -> Option<String> {
    let row = value
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(value);
    let email = row
        .get("active_account_email")
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

/// Force an immediate heartbeat and instance registry sync to all root endpoints
pub async fn sync_local_node_now() -> Result<(), AppError> {
    let config = load_config()?;
    for ep in &config.endpoints {
        if ep.is_enabled && ep.role == "root" {
            let _ = send_heartbeat(ep, &config.node_alias).await;
        }
    }
    Ok(())
}

/// Trigger manual synchronization of local node and profiles to Supabase
pub async fn trigger_manual_sync() -> Result<(), AppError> {
    sync_local_node_now().await
}

/// Start background synchronization daemon
pub fn start_sync_worker() {
    let is_already_running = SYNC_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tauri::async_runtime::spawn(async move {
        // Enforce 60-second startup quiet period: nothing runs until 1 minute after launch
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

            let config = match load_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if !config.is_sync_enabled {
                continue;
            }

            for ep in &config.endpoints {
                if !ep.is_enabled {
                    continue;
                }
                if ep.role == "root" {
                    let _ = send_heartbeat(ep, &config.node_alias).await;
                }
            }
        }
    });
}

/// Migration summary for cross-database failover transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMigrationSummary {
    pub is_success: bool,
    pub nodes_migrated: usize,
    pub profiles_migrated: usize,
    pub leases_migrated: usize,
    pub commands_migrated: usize,
    pub message: String,
}

/// Migrate active and recent state from a source/damaged Supabase endpoint to a target endpoint
pub async fn migrate_database_data(
    source_endpoint_id: &str,
    target_endpoint_id: &str,
) -> Result<DataMigrationSummary, AppError> {
    let config = load_config()?;
    let source_ep = config
        .endpoints
        .iter()
        .find(|e| e.id == source_endpoint_id)
        .ok_or_else(|| {
            AppError::Config(format!(
                "Source endpoint '{}' not found",
                source_endpoint_id
            ))
        })?;
    let target_ep = config
        .endpoints
        .iter()
        .find(|e| e.id == target_endpoint_id)
        .ok_or_else(|| {
            AppError::Config(format!(
                "Target endpoint '{}' not found",
                target_endpoint_id
            ))
        })?;

    let source_client = SupabaseClient::new(source_ep)?;
    let target_client = SupabaseClient::new(target_ep)?;

    let mut nodes_migrated = 0;
    let mut profiles_migrated = 0;
    let mut leases_migrated = 0;
    let mut commands_migrated = 0;

    // 1. Migrate active nodes
    if let Ok(nodes_val) = source_client.select("nodes", "status=eq.online").await {
        if let Some(nodes_arr) = nodes_val.as_array() {
            for node in nodes_arr {
                if target_client
                    .upsert("nodes", node.clone(), "id")
                    .await
                    .is_ok()
                {
                    nodes_migrated += 1;
                }
            }
        }
    }

    // 2. Migrate active instance profiles
    if let Ok(prof_val) = source_client
        .select("instance_profiles", "status=eq.running")
        .await
    {
        if let Some(prof_arr) = prof_val.as_array() {
            for prof in prof_arr {
                if target_client
                    .upsert("instance_profiles", prof.clone(), "id")
                    .await
                    .is_ok()
                {
                    profiles_migrated += 1;
                }
            }
        }
    }

    // 3. Migrate active leases (unexpired)
    let now = Utc::now().timestamp();
    let lease_query = format!("expires_at=gt.{}", now);
    if let Ok(leases_val) = source_client.select("workspace_leases", &lease_query).await {
        if let Some(leases_arr) = leases_val.as_array() {
            for lease in leases_arr {
                if target_client
                    .upsert("workspace_leases", lease.clone(), "account_id")
                    .await
                    .is_ok()
                {
                    leases_migrated += 1;
                }
            }
        }
    }

    // 4. Migrate recent 50 commands
    if let Ok(cmd_val) = source_client
        .select("command_queue", "order=created_at.desc&limit=50")
        .await
    {
        if let Some(cmd_arr) = cmd_val.as_array() {
            for cmd in cmd_arr {
                if target_client
                    .upsert("command_queue", cmd.clone(), "id")
                    .await
                    .is_ok()
                {
                    commands_migrated += 1;
                }
            }
        }
    }

    let total = nodes_migrated + profiles_migrated + leases_migrated + commands_migrated;
    let msg =
        format!(
        "Successfully migrated {} records ({} nodes, {} profiles, {} leases, {} commands) to {}",
        total, nodes_migrated, profiles_migrated, leases_migrated, commands_migrated, target_ep.name
    );

    Ok(DataMigrationSummary {
        is_success: true,
        nodes_migrated,
        profiles_migrated,
        leases_migrated,
        commands_migrated,
        message: msg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_supabase_config() {
        let def = SupabaseConfig::default();
        assert!(!def.is_sync_enabled);
        assert_eq!(def.heartbeat_interval_secs, 30);
        assert_eq!(def.auto_prune_root_mb, 400);
        assert_eq!(def.auto_prune_secondary_mb, 200);
        assert!(def.node_alias.starts_with("Node-"));
        assert!(def.endpoints.is_empty());
    }

    #[test]
    fn test_supabase_config_serde_and_url_normalization() {
        let json_data = r#"{
            "endpoints": [
                {
                    "id": "ep1",
                    "name": "Endpoint 1",
                    "url": "https://sample.supabase.co/rest/v1/",
                    "api_key": "sample-key",
                    "role": "root",
                    "is_enabled": true,
                    "prune_threshold_mb": 400,
                    "priority": 1,
                    "notes": null,
                    "tags": []
                }
            ],
            "node_alias": "Node-Test",
            "is_sync_enabled": true,
            "auto_prune_root_mb": 400,
            "auto_prune_secondary_mb": 200,
            "heartbeat_interval_secs": 30
        }"#;

        let mut config: SupabaseConfig = serde_json::from_str(json_data).expect("valid json");
        assert_eq!(
            config.endpoints[0].url,
            "https://sample.supabase.co/rest/v1/"
        );

        // Normalize endpoints as done in load_config
        for ep in &mut config.endpoints {
            ep.url = normalize_supabase_url(&ep.url);
        }
        assert_eq!(config.endpoints[0].url, "https://sample.supabase.co");

        // Verify re-serialization maintains normalized URL
        let serialized = serde_json::to_string(&config).expect("serialization works");
        assert!(serialized.contains("\"https://sample.supabase.co\""));
        assert!(!serialized.contains("/rest/v1/\""));
    }

    #[test]
    fn test_postgrest_error_message_pgrst205() {
        let err_json = json!({
            "code": "PGRST205",
            "details": null,
            "hint": null,
            "message": "relation \"nodes\" does not exist"
        });
        let msg = postgrest_error_message(&err_json).expect("should extract error");
        assert_eq!(msg, "relation \"nodes\" does not exist (PGRST205)");

        let err_only_code = json!({ "code": "PGRST205" });
        let msg_code = postgrest_error_message(&err_only_code).expect("should extract error");
        assert_eq!(msg_code, "PostgREST error (PGRST205)");

        let ok_payload = json!({ "id": "node1", "status": "online" });
        assert!(postgrest_error_message(&ok_payload).is_none());
    }

    #[test]
    fn test_email_from_postgrest_readback() {
        let row_array = json!([{ "active_account_email": "test@example.com" }]);
        assert_eq!(
            email_from_postgrest(&row_array),
            Some("test@example.com".to_string())
        );

        let single_row = json!({ "active_account_email": "hello@example.com" });
        assert_eq!(
            email_from_postgrest(&single_row),
            Some("hello@example.com".to_string())
        );

        let empty_array = json!([]);
        assert_eq!(email_from_postgrest(&empty_array), None);

        let empty_email = json!([{ "active_account_email": "   " }]);
        assert_eq!(email_from_postgrest(&empty_email), None);
    }

    #[test]
    fn test_candidate_repo_secrets_paths() {
        let paths = candidate_repo_secrets_paths();
        assert!(!paths.is_empty());
        let path_strs: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        assert!(path_strs
            .iter()
            .any(|p| p.contains("03-supabase/01-own/supabase-credentials.json")));
        assert!(path_strs
            .iter()
            .any(|p| p.contains("03-supabase/02-lovable/supabase-credentials.json")));
    }

    #[test]
    fn test_auto_seed_from_repo_secrets_execution() {
        let mut cfg = SupabaseConfig::default();
        assert!(cfg.endpoints.is_empty());
        assert!(!cfg.is_sync_enabled);

        let seeded = auto_seed_from_repo_secrets(&mut cfg);
        let d_own =
            PathBuf::from("D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json");
        if d_own.exists() {
            assert!(seeded);
            assert!(!cfg.endpoints.is_empty());
            assert!(cfg.is_sync_enabled);
            if let Some(first) = cfg.endpoints.first() {
                assert_eq!(first.priority, 1);
                assert_eq!(first.role, "root");
            }
        }
    }
}
