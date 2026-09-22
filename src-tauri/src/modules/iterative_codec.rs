//! Iterative Base64 Codec and Configuration Serialization Module
//! Protects API keys with multi-pass Base64 encoding (<rounds>$<hash>) and handles JSON/YAML exports.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::supabase_client::SupabaseEndpoint;
use crate::modules::supabase_sync::SupabaseConfig;
use base64::prelude::*;
use serde::{Deserialize, Serialize};

/// Exportable Supabase bundle structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseExportBundle {
    pub version: String,
    pub node_alias: String,
    pub is_sync_enabled: bool,
    pub auto_prune_root_mb: u64,
    pub auto_prune_secondary_mb: u64,
    pub heartbeat_interval_secs: u64,
    pub endpoints: Vec<SupabaseEndpoint>,
}

/// Encode plaintext with N iterations of Base64 and prefix with "<rounds>$"
pub fn encode_iterative(data: &str, rounds: u32) -> String {
    let effective_rounds = if rounds == 0 { 1 } else { rounds };
    let mut current = data.to_string();

    for _ in 0..effective_rounds {
        current = BASE64_STANDARD.encode(current.as_bytes());
    }

    format!("{}${}", effective_rounds, current)
}

/// Decode iterative Base64 string prefixed with "<rounds>$" or "<rounds>:"
pub fn decode_iterative(encoded: &str) -> Result<String, AppError> {
    let trimmed = encoded.trim();

    let parts = trimmed.split_once('$').or_else(|| trimmed.split_once(':'));
    if let Some((rounds_part, payload_part)) = parts {
        let rounds: u32 = rounds_part
            .parse()
            .map_err(|_| AppError::Config(format!("Invalid iteration prefix: {}", rounds_part)))?;

        let mut current_payload = payload_part.to_string();

        for _ in 0..rounds {
            let decoded_bytes = BASE64_STANDARD
                .decode(current_payload.as_bytes())
                .map_err(|e| AppError::Config(format!("Base64 decode failed: {}", e)))?;
            current_payload = String::from_utf8(decoded_bytes)
                .map_err(|e| AppError::Config(format!("UTF-8 decode failed: {}", e)))?;
        }

        return Ok(current_payload);
    }

    // Fallback: single pass decode if valid base64, otherwise return trimmed
    if let Ok(bytes) = BASE64_STANDARD.decode(trimmed.as_bytes()) {
        if let Ok(str_val) = String::from_utf8(bytes) {
            return Ok(str_val);
        }
    }

    Ok(trimmed.to_string())
}

fn format_yaml_endpoint(ep: &SupabaseEndpoint) -> String {
    format!(
        "  - id: \"{}\"\n    name: \"{}\"\n    url: \"{}\"\n    api_key: \"{}\"\n    role: \"{}\"\n    is_enabled: {}\n    prune_threshold_mb: {}\n    priority: {}\n",
        ep.id, ep.name, ep.url, ep.api_key, ep.role, ep.is_enabled, ep.prune_threshold_mb, ep.priority
    )
}

fn format_yaml_bundle(bundle: &SupabaseExportBundle) -> String {
    let mut out = format!(
        "version: \"{}\"\nnode_alias: \"{}\"\nis_sync_enabled: {}\nauto_prune_root_mb: {}\nauto_prune_secondary_mb: {}\nheartbeat_interval_secs: {}\nendpoints:\n",
        bundle.version, bundle.node_alias, bundle.is_sync_enabled, bundle.auto_prune_root_mb, bundle.auto_prune_secondary_mb, bundle.heartbeat_interval_secs
    );
    for ep in &bundle.endpoints {
        out.push_str(&format_yaml_endpoint(ep));
    }
    out
}

fn parse_yaml_bundle(content: &str) -> Result<SupabaseExportBundle, AppError> {
    let mut version = "1.0.0".to_string();
    let mut node_alias = "Node-Local".to_string();
    let mut is_sync_enabled = false;
    let mut auto_prune_root_mb = 400;
    let mut auto_prune_secondary_mb = 200;
    let mut heartbeat_interval_secs = 30;
    let mut endpoints = Vec::new();
    let mut cur_ep: Option<SupabaseEndpoint> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().trim_start_matches("- ").trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');

            if trimmed.starts_with("- ") {
                if let Some(ep) = cur_ep.take() {
                    endpoints.push(ep);
                }
                cur_ep = Some(SupabaseEndpoint {
                    id: String::new(),
                    name: String::new(),
                    url: String::new(),
                    api_key: String::new(),
                    role: "secondary".to_string(),
                    is_enabled: true,
                    prune_threshold_mb: 200,
                    priority: 1,
                });
            }

            if let Some(ep) = cur_ep.as_mut() {
                match key {
                    "id" => ep.id = val.to_string(),
                    "name" => ep.name = val.to_string(),
                    "url" => ep.url = val.to_string(),
                    "api_key" => ep.api_key = val.to_string(),
                    "role" => ep.role = val.to_string(),
                    "is_enabled" => ep.is_enabled = val.parse().unwrap_or(true),
                    "prune_threshold_mb" => ep.prune_threshold_mb = val.parse().unwrap_or(200),
                    "priority" => ep.priority = val.parse().unwrap_or(1),
                    _ => {}
                }
            } else {
                match key {
                    "version" => version = val.to_string(),
                    "node_alias" => node_alias = val.to_string(),
                    "is_sync_enabled" => is_sync_enabled = val.parse().unwrap_or(false),
                    "auto_prune_root_mb" => auto_prune_root_mb = val.parse().unwrap_or(400),
                    "auto_prune_secondary_mb" => {
                        auto_prune_secondary_mb = val.parse().unwrap_or(200)
                    }
                    "heartbeat_interval_secs" => {
                        heartbeat_interval_secs = val.parse().unwrap_or(30)
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(ep) = cur_ep {
        endpoints.push(ep);
    }

    Ok(SupabaseExportBundle {
        version,
        node_alias,
        is_sync_enabled,
        auto_prune_root_mb,
        auto_prune_secondary_mb,
        heartbeat_interval_secs,
        endpoints,
    })
}

/// Export Supabase configuration with iterative Base64 encoded keys
pub fn export_config_string(
    config: &SupabaseConfig,
    format_type: &str,
    rounds: u32,
) -> Result<String, AppError> {
    let mut export_endpoints = Vec::new();

    for ep in &config.endpoints {
        let mut ep_clone = ep.clone();
        ep_clone.api_key = encode_iterative(&ep.api_key, rounds);
        export_endpoints.push(ep_clone);
    }

    let bundle = SupabaseExportBundle {
        version: "1.0.0".to_string(),
        node_alias: config.node_alias.clone(),
        is_sync_enabled: config.is_sync_enabled,
        auto_prune_root_mb: config.auto_prune_root_mb,
        auto_prune_secondary_mb: config.auto_prune_secondary_mb,
        heartbeat_interval_secs: config.heartbeat_interval_secs,
        endpoints: export_endpoints,
    };

    if format_type.eq_ignore_ascii_case("yaml") || format_type.eq_ignore_ascii_case("yml") {
        Ok(format_yaml_bundle(&bundle))
    } else {
        serde_json::to_string_pretty(&bundle)
            .map_err(|e| AppError::Config(format!("Export serialization failed: {}", e)))
    }
}

/// Import Supabase configuration and decode iterative Base64 keys
pub fn import_config_string(content: &str) -> Result<SupabaseConfig, AppError> {
    let trimmed = content.trim();
    let bundle: SupabaseExportBundle = match serde_json::from_str(trimmed) {
        Ok(b) => b,
        Err(_) => parse_yaml_bundle(trimmed)?,
    };

    let mut restored_endpoints = Vec::new();

    for ep in bundle.endpoints {
        let mut ep_clone = ep;
        ep_clone.api_key = decode_iterative(&ep_clone.api_key)?;
        restored_endpoints.push(ep_clone);
    }

    let config = SupabaseConfig {
        endpoints: restored_endpoints,
        node_alias: bundle.node_alias,
        is_sync_enabled: bundle.is_sync_enabled,
        auto_prune_root_mb: bundle.auto_prune_root_mb,
        auto_prune_secondary_mb: bundle.auto_prune_secondary_mb,
        heartbeat_interval_secs: bundle.heartbeat_interval_secs,
    };

    Ok(config)
}
