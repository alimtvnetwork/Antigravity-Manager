//! Supabase vault JSON import (plan 88, subtasks 004-005).
//!
//! Merges one parsed JSON value into a `SupabaseConfig`. Accepts the
//! `agm/supabase-endpoints` and `agm/supabase-credentials` envelope schemas
//! and the plain config shapes (an `endpoints` array, a bare endpoints
//! array, or a credentials object). Applies `is_sync_enabled` from the file
//! when present. `node_alias` in the file is intentionally ignored: it is
//! never read, written, or printed (resolved ambiguity 02). Key material is
//! never included in the returned messages — endpoint ids and counts only.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use super::*;
use crate::modules::{json_envelope, supabase_client};

/// Outcome of merging one JSON file into the config.
#[derive(Debug, Clone, Default)]
pub struct SupabaseFileImport {
    pub endpoints_added: usize,
    pub credentials_added: usize,
    /// `is_sync_enabled` as declared by the file, when present.
    pub sync_flag: Option<bool>,
    /// Human-readable, secret-free progress messages.
    pub messages: Vec<String>,
}

fn read_sync_flag(val: &serde_json::Value) -> Option<bool> {
    val.get("is_sync_enabled")
        .and_then(|v| v.as_bool())
        .or_else(|| {
            val.get("credentials")
                .and_then(|c| c.get("is_sync_enabled"))
                .and_then(|v| v.as_bool())
        })
}

/// Merge one parsed Supabase JSON value into `cfg`.
pub fn merge_supabase_json_into_config(
    cfg: &mut SupabaseConfig,
    raw_val: serde_json::Value,
    source_label: &str,
) -> SupabaseFileImport {
    let mut out = SupabaseFileImport::default();

    // Unpack envelope if present
    let (attrs_opt, val) = json_envelope::unpack_envelope(raw_val.clone());
    let is_b64 = attrs_opt
        .as_ref()
        .and_then(|a| a.encoding.as_deref())
        .map(|enc| enc.eq_ignore_ascii_case("base64"))
        .unwrap_or(false)
        || raw_val.get("encoding_format").and_then(|f| f.as_str()) == Some("base64");

    out.sync_flag = read_sync_flag(&val);

    let decode_val = |raw: Option<&serde_json::Value>| -> Option<String> {
        let s = raw.and_then(|v| v.as_str())?;
        if is_b64 {
            if let Ok(b) = STANDARD.decode(s.trim()) {
                if let Ok(utf) = String::from_utf8(b) {
                    return Some(utf.trim().to_string());
                }
            }
        }
        Some(s.trim().to_string())
    };

    let mut merge_endpoint = |cfg: &mut SupabaseConfig, mut new_ep: supabase_client::SupabaseEndpoint| {
        new_ep.url = supabase_client::normalize_supabase_url(&new_ep.url);
        if let Some(pos) = cfg.endpoints.iter().position(|e| {
            e.id == new_ep.id || supabase_client::normalize_supabase_url(&e.url) == new_ep.url
        }) {
            cfg.endpoints[pos] = new_ep;
        } else {
            cfg.endpoints.push(new_ep);
        }
        out.endpoints_added += 1;
    };

    if let Some(arr) = val.get("endpoints").and_then(|v| v.as_array()) {
        if let Ok(eps) = serde_json::from_value::<Vec<supabase_client::SupabaseEndpoint>>(
            serde_json::Value::Array(arr.clone()),
        ) {
            for ep in eps {
                merge_endpoint(cfg, ep);
            }
            out.messages
                .push(format!("Merged endpoints from '{}'.", source_label));
        }
    } else if let Ok(eps) =
        serde_json::from_value::<Vec<supabase_client::SupabaseEndpoint>>(val.clone())
    {
        for ep in eps {
            merge_endpoint(cfg, ep);
        }
        out.messages
            .push(format!("Merged endpoints array from '{}'.", source_label));
    } else if let Some(creds) = val
        .get("credentials")
        .and_then(|v| v.as_object())
        .or_else(|| val.as_object())
    {
        let endpoint_url = decode_val(creds.get("endpoint").or_else(|| creds.get("url")));
        let token = decode_val(creds.get("token").or_else(|| creds.get("api_key")));
        let service =
            decode_val(creds.get("service")).unwrap_or_else(|| "supabase-service".to_string());
        let role_opt = decode_val(creds.get("role"));
        let tag_opt = decode_val(creds.get("tag"));
        let notes_opt = decode_val(creds.get("notes"));
        if let (Some(url), Some(tok)) = (endpoint_url, token) {
            let clean_id = format!("ep-{}", service.to_lowercase().replace([' ', '_'], "-"));
            let role = if let Some(r) = role_opt.filter(|r| !r.is_empty()) {
                r.to_lowercase()
            } else if service.to_lowercase().contains("root")
                || service.to_lowercase().contains("lovable")
            {
                "root".to_string()
            } else {
                "secondary".to_string()
            };
            let norm_url = supabase_client::normalize_supabase_url(&url);
            let mut tags = vec![role.clone()];
            if let Some(t) = tag_opt {
                if !tags.contains(&t) {
                    tags.push(t);
                }
            }
            if !tags.contains(&service) {
                tags.push(service.clone());
            }
            let new_ep = supabase_client::SupabaseEndpoint {
                id: clean_id.clone(),
                name: format!("Supabase ({})", service),
                url: norm_url,
                api_key: tok,
                role: role.clone(),
                is_enabled: true,
                prune_threshold_mb: if role == "root" { 400 } else { 200 },
                priority: if role == "root" { 1 } else { 2 },
                notes: notes_opt
                    .or_else(|| Some(format!("Imported from credentials JSON ({})", source_label))),
                tags,
            };
            // Credentials count separately from plain endpoint merges:
            // merge_endpoint bumps endpoints_added, so reclassify it.
            merge_endpoint(cfg, new_ep);
            out.endpoints_added -= 1;
            out.credentials_added += 1;
            out.messages.push(format!(
                "Ingested credentials database '{}' from '{}'",
                clean_id, source_label
            ));
        }
    }

    out
}

/// Finalize a config after one or more file merges: drop placeholder
/// endpoints when real ones exist, dedupe by normalized URL, sort by
/// priority, and apply the sync flag (files that declare
/// `is_sync_enabled` win; otherwise sync is enabled, matching the
/// historical `load-json` behavior).
pub fn finalize_imported_config(cfg: &mut SupabaseConfig, sync_flag: Option<bool>) {
    if cfg
        .endpoints
        .iter()
        .any(|e| !supabase_client::is_placeholder_or_mock_endpoint(e))
    {
        cfg.endpoints
            .retain(|e| !supabase_client::is_placeholder_or_mock_endpoint(e));
        let mut seen_urls = std::collections::HashSet::new();
        cfg.endpoints
            .retain(|e| seen_urls.insert(supabase_client::normalize_supabase_url(&e.url)));
    }
    cfg.endpoints.sort_by_key(|e| e.priority);
    cfg.is_sync_enabled = sync_flag.unwrap_or(true);
}
