use chrono::Utc;
use std::fs;

use super::*;

/// Load all stored SSH connections, auto-seeding from repo-secrets / gitmap if empty
pub fn load_ssh_connections() -> Result<Vec<SshConnectionRecord>, String> {
    let store_path = get_ssh_nodes_store_path()?;
    if store_path.exists() {
        if let Ok(raw) = fs::read_to_string(&store_path) {
            if let Ok(conns) = decode_connections_from_json(&raw) {
                if !conns.is_empty() {
                    return Ok(conns);
                }
            }
        }
    }

    // Auto-seed from candidate gitmap-ssh-nodes.json if available
    for seed_path in candidate_seed_node_paths() {
        if seed_path.exists() && seed_path.is_file() {
            if let Ok(raw) = fs::read_to_string(&seed_path) {
                if let Ok(mut conns) = decode_connections_from_json(&raw) {
                    if !conns.is_empty() {
                        for c in &mut conns {
                            c.key_path = resolve_usable_key_path(&c.key_path);
                        }
                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                        crate::error::record_ignored(
                            save_ssh_connections(&conns),
                            "save_ssh_connections",
                        );
                        return Ok(conns);
                    }
                }
            }
        }
    }

    Ok(Vec::new())
}

/// Save SSH connections to local store as an SshNodesExportEnvelope
pub fn save_ssh_connections(
    conns: &[SshConnectionRecord],
) -> Result<SshNodesExportEnvelope, String> {
    let envelope = build_envelope_from_connections(conns);
    let store_path = get_ssh_nodes_store_path()?;
    if let Some(parent) = store_path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }
    let json_str = serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("Failed to serialize SSH nodes envelope: {}", e))?;
    fs::write(&store_path, json_str)
        .map_err(|e| format!("Failed to write SSH nodes store: {}", e))?;
    Ok(envelope)
}

/// Build a GitMap-compatible SshNodesExportEnvelope from connections
pub fn build_envelope_from_connections(conns: &[SshConnectionRecord]) -> SshNodesExportEnvelope {
    let nodes: Vec<SshNodeExportItem> = conns
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let key_path = resolve_usable_key_path(&c.key_path);
            let auth_method = if !key_path.is_empty() {
                "key".to_string()
            } else {
                "password".to_string()
            };
            SshNodeExportItem {
                worker_id: format!("worker-{}", idx + 1),
                numeric_id: idx + 1,
                alias: c.alias.clone(),
                ip_address: c.ip_address.clone(),
                username: if c.username.is_empty() {
                    "root".to_string()
                } else {
                    c.username.clone()
                },
                port: 22,
                os: if c.os.is_empty() {
                    "linux".to_string()
                } else {
                    c.os.clone()
                },
                auth_method,
                key_path,
            }
        })
        .collect();

    SshNodesExportEnvelope {
        schema_version: "1.0".to_string(),
        exported_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        total_nodes: conns.len(),
        nodes,
        connections: conns.to_vec(),
    }
}

/// Decode connections from either SshNodesExportEnvelope or a raw array of connections/nodes
pub fn decode_connections_from_json(raw: &str) -> Result<Vec<SshConnectionRecord>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Empty JSON input".to_string());
    }

    if let Ok(parsed_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let (attrs_opt, unwrapped) =
            crate::modules::json_envelope::unpack_envelope(parsed_val.clone());
        let effective_val = if attrs_opt.is_some() {
            unwrapped
        } else {
            parsed_val
        };

        let mut extracted_nodes = Vec::new();
        if let Some(main_obj) = effective_val.get("mainMachine") {
            if let Ok(item) = serde_json::from_value::<SshNodeExportItem>(main_obj.clone()) {
                extracted_nodes.push(item);
            }
        }
        if let Some(nodes_arr) = effective_val.get("nodes").and_then(|n| n.as_array()) {
            for n_val in nodes_arr {
                if let Ok(item) = serde_json::from_value::<SshNodeExportItem>(n_val.clone()) {
                    extracted_nodes.push(item);
                }
            }
        }
        if !extracted_nodes.is_empty() {
            return Ok(nodes_to_connections(&extracted_nodes));
        }

        if let Some(conns_arr) = effective_val.get("connections").and_then(|c| c.as_array()) {
            if let Ok(conns) = serde_json::from_value::<Vec<SshConnectionRecord>>(
                serde_json::Value::Array(conns_arr.clone()),
            ) {
                if !conns.is_empty() {
                    return Ok(conns);
                }
            }
        }

        if let Ok(conns) = serde_json::from_value::<Vec<SshConnectionRecord>>(effective_val.clone())
        {
            return Ok(conns);
        }

        if let Ok(nodes) = serde_json::from_value::<Vec<SshNodeExportItem>>(effective_val) {
            return Ok(nodes_to_connections(&nodes));
        }
    }

    if let Ok(env) = serde_json::from_str::<SshNodesExportEnvelope>(trimmed) {
        if !env.connections.is_empty() {
            return Ok(env.connections);
        }
        if !env.nodes.is_empty() {
            return Ok(nodes_to_connections(&env.nodes));
        }
        return Ok(Vec::new());
    }

    if let Ok(conns) = serde_json::from_str::<Vec<SshConnectionRecord>>(trimmed) {
        return Ok(conns);
    }

    if let Ok(nodes) = serde_json::from_str::<Vec<SshNodeExportItem>>(trimmed) {
        return Ok(nodes_to_connections(&nodes));
    }

    Err("Unrecognized SSH nodes JSON format".to_string())
}

pub(crate) fn nodes_to_connections(nodes: &[SshNodeExportItem]) -> Vec<SshConnectionRecord> {
    let now = Utc::now().to_rfc3339();
    nodes
        .iter()
        .map(|n| SshConnectionRecord {
            alias: n.alias.clone(),
            ip_address: n.ip_address.clone(),
            username: if n.username.is_empty() {
                "root".to_string()
            } else {
                n.username.clone()
            },
            encrypted_password: String::new(),
            key_path: resolve_usable_key_path(&n.key_path),
            os: if n.os.is_empty() {
                "linux".to_string()
            } else {
                n.os.clone()
            },
            os_group: None,
            os_version: None,
            build_version: None,
            first_run_at: Some(now.clone()),
            created_at: Some(now.clone()),
        })
        .collect()
}

/// Smart match-and-update synchronization of incoming SSH connections
pub fn sync_ssh_connections_locally(
    incoming_list: &[SshConnectionRecord],
) -> Result<NodeSyncStats, String> {
    let mut existing = load_ssh_connections().unwrap_or_default();
    let mut stats = NodeSyncStats {
        total: incoming_list.len(),
        ..Default::default()
    };
    let now = Utc::now().to_rfc3339();

    for inc in incoming_list {
        let mut normalized_inc = inc.clone();
        normalized_inc.key_path = resolve_usable_key_path(&normalized_inc.key_path);
        if normalized_inc.username.is_empty() {
            normalized_inc.username = "root".to_string();
        }
        if normalized_inc.os.is_empty() {
            normalized_inc.os = "linux".to_string();
        }

        let pos = existing.iter().position(|e| {
            (!normalized_inc.alias.is_empty()
                && e.alias.eq_ignore_ascii_case(&normalized_inc.alias))
                || (!normalized_inc.ip_address.is_empty()
                    && e.ip_address
                        .eq_ignore_ascii_case(&normalized_inc.ip_address))
        });

        if let Some(idx) = pos {
            stats.matched += 1;
            let curr = &mut existing[idx];
            let mut changed = false;

            if !normalized_inc.alias.is_empty() && curr.alias != normalized_inc.alias {
                curr.alias = normalized_inc.alias.clone();
                changed = true;
            }
            if !normalized_inc.ip_address.is_empty() && curr.ip_address != normalized_inc.ip_address
            {
                curr.ip_address = normalized_inc.ip_address.clone();
                changed = true;
            }
            if !normalized_inc.username.is_empty() && curr.username != normalized_inc.username {
                curr.username = normalized_inc.username.clone();
                changed = true;
            }
            if !normalized_inc.encrypted_password.is_empty()
                && curr.encrypted_password != normalized_inc.encrypted_password
            {
                curr.encrypted_password = normalized_inc.encrypted_password.clone();
                changed = true;
            }
            if !normalized_inc.key_path.is_empty() && curr.key_path != normalized_inc.key_path {
                curr.key_path = normalized_inc.key_path.clone();
                changed = true;
            }
            if !normalized_inc.os.is_empty() && curr.os != normalized_inc.os {
                curr.os = normalized_inc.os.clone();
                changed = true;
            }

            if changed {
                stats.updated += 1;
            } else {
                stats.unchanged += 1;
            }
        } else {
            if normalized_inc.created_at.is_none() {
                normalized_inc.created_at = Some(now.clone());
            }
            if normalized_inc.first_run_at.is_none() {
                normalized_inc.first_run_at = Some(now.clone());
            }
            existing.push(normalized_inc);
            stats.inserted += 1;
        }
    }

    save_ssh_connections(&existing)?;
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(update_ssh_config(false), "update_ssh_config");
    Ok(stats)
}
