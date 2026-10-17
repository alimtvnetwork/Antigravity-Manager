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

/// Resolve username placeholders (`${adminUser}`, `${linuxUser}`) and apply OS-appropriate defaults
pub fn resolve_username_placeholders(raw: &str, os: &str) -> String {
    let trimmed = raw.trim();
    let is_windows = os.trim().to_ascii_lowercase().contains("win");
    let default_user = if is_windows { "Administrator" } else { "root" };

    if trimmed.is_empty() {
        return default_user.to_string();
    }

    let resolved = trimmed
        .replace("${adminUser}", "Administrator")
        .replace("${linuxUser}", "root");

    let clean = resolved.trim();
    if clean.is_empty() {
        default_user.to_string()
    } else {
        clean.to_string()
    }
}

/// Normalize a list of SSH connections so that any non-empty password is encrypted with `encrypt_password_rsa` (`ag_rsa_v1_`)
pub fn ensure_connections_rsa_encrypted(conns: &[SshConnectionRecord]) -> Vec<SshConnectionRecord> {
    conns
        .iter()
        .map(|c| {
            let mut item = c.clone();
            if item.port == 0 {
                item.port = 22;
            }
            if item.encryption_algo.trim().is_empty() {
                item.encryption_algo = "RSA-2048".to_string();
            }
            let pw = item.encrypted_password.trim();
            if !pw.is_empty() && !pw.starts_with(crate::utils::crypto::ENCRYPTED_RSA_V1_PREFIX) {
                if let Ok(enc) = crate::utils::crypto::encrypt_password_rsa(pw) {
                    item.encrypted_password = enc;
                    item.encryption_algo = "RSA-2048".to_string();
                }
            }
            item
        })
        .collect()
}

/// Save SSH connections to local store as an SshNodesExportEnvelope (with RSA-2048 password encryption enforced)
pub fn save_ssh_connections(
    conns: &[SshConnectionRecord],
) -> Result<SshNodesExportEnvelope, String> {
    let safe_conns = ensure_connections_rsa_encrypted(conns);
    let envelope = build_envelope_from_connections(&safe_conns);
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
    let mut safe_conns = ensure_connections_rsa_encrypted(conns);
    for c in &mut safe_conns {
        c.username = resolve_username_placeholders(&c.username, &c.os);
    }
    let nodes: Vec<SshNodeExportItem> = safe_conns
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let key_path = resolve_usable_key_path(&c.key_path);
            let auth_method = if !key_path.is_empty() {
                "key".to_string()
            } else {
                "rsa-password".to_string()
            };
            SshNodeExportItem {
                worker_id: format!("worker-{}", idx + 1),
                numeric_id: idx + 1,
                alias: c.alias.clone(),
                ip_address: c.ip_address.clone(),
                username: resolve_username_placeholders(&c.username, &c.os),
                port: if c.port == 0 { 22 } else { c.port },
                os: if c.os.is_empty() {
                    "linux".to_string()
                } else {
                    c.os.clone()
                },
                auth_method,
                key_path,
                encrypted_password: c.encrypted_password.clone(),
                encryption_algo: "RSA-2048".to_string(),
            }
        })
        .collect();

    SshNodesExportEnvelope {
        schema_version: "1.0".to_string(),
        exported_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        total_nodes: safe_conns.len(),
        nodes,
        connections: safe_conns,
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
    let mapped: Vec<SshConnectionRecord> = nodes
        .iter()
        .map(|n| SshConnectionRecord {
            alias: n.alias.clone(),
            ip_address: n.ip_address.clone(),
            username: resolve_username_placeholders(&n.username, &n.os),
            port: if n.port == 0 { 22 } else { n.port },
            encrypted_password: n.encrypted_password.clone(),
            encryption_algo: "RSA-2048".to_string(),
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
        .collect();
    ensure_connections_rsa_encrypted(&mapped)
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

    for inc in &ensure_connections_rsa_encrypted(incoming_list) {
        let mut normalized_inc = inc.clone();
        normalized_inc.key_path = resolve_usable_key_path(&normalized_inc.key_path);
        if normalized_inc.os.is_empty() {
            normalized_inc.os = "linux".to_string();
        }
        normalized_inc.username =
            resolve_username_placeholders(&normalized_inc.username, &normalized_inc.os);
        if normalized_inc.port == 0 {
            normalized_inc.port = 22;
        }
        normalized_inc.encryption_algo = "RSA-2048".to_string();

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
            if normalized_inc.port != 0 && curr.port != normalized_inc.port {
                curr.port = normalized_inc.port;
                changed = true;
            }
            curr.encryption_algo = "RSA-2048".to_string();
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

/// Convert `SshConnectionRecord` into UI/CLI `SshNodeRecord`
pub fn to_ssh_node_record(conn: &SshConnectionRecord) -> SshNodeRecord {
    let has_pw = !conn.encrypted_password.trim().is_empty();
    let auth_method = if !conn.key_path.trim().is_empty() && has_pw {
        "rsa-key+password".to_string()
    } else if !conn.key_path.trim().is_empty() {
        "key".to_string()
    } else if has_pw {
        "rsa-password".to_string()
    } else {
        "agent".to_string()
    };
    SshNodeRecord {
        id: conn.alias.to_lowercase(),
        alias: conn.alias.clone(),
        host: conn.ip_address.clone(),
        port: if conn.port == 0 { 22 } else { conn.port },
        username: conn.username.clone(),
        auth_method,
        key_path: if conn.key_path.trim().is_empty() {
            None
        } else {
            Some(conn.key_path.clone())
        },
        encrypted_password: if has_pw {
            Some(conn.encrypted_password.clone())
        } else {
            None
        },
        has_password: has_pw,
        encryption_algo: if conn.encryption_algo.trim().is_empty() {
            "RSA-2048".to_string()
        } else {
            conn.encryption_algo.clone()
        },
        os: conn.os.clone(),
        source: "agm-ssh-vault".to_string(),
        last_status: "ready".to_string(),
    }
}

/// Save or update an SSH node, encrypting any raw password immediately with RSA-2048 (`ag_rsa_v1_`)
pub fn upsert_ssh_node_rsa(payload: &SaveSshNodeRsaPayload) -> Result<SshNodeRecord, String> {
    let alias = payload.alias.trim();
    let host = payload.host.trim();
    if alias.is_empty() || host.is_empty() {
        return Err("SSH node alias and host/IP cannot be empty".to_string());
    }
    let username = if payload.username.trim().is_empty() {
        "Administrator".to_string()
    } else {
        payload.username.trim().to_string()
    };
    let port = if payload.port == 0 { 22 } else { payload.port };

    let encrypted_password = if let Some(raw_pw) = payload.raw_password.as_deref() {
        let trimmed_pw = raw_pw.trim();
        if trimmed_pw.is_empty() {
            String::new()
        } else {
            crate::utils::crypto::encrypt_password_rsa(trimmed_pw)?
        }
    } else {
        String::new()
    };

    let record = SshConnectionRecord {
        alias: alias.to_string(),
        ip_address: host.to_string(),
        username,
        port,
        encrypted_password,
        encryption_algo: "RSA-2048".to_string(),
        key_path: payload.key_path.clone().unwrap_or_default(),
        os: payload.os.clone().unwrap_or_else(|| "windows".to_string()),
        os_group: None,
        os_version: None,
        build_version: None,
        first_run_at: None,
        created_at: Some(Utc::now().to_rfc3339()),
    };

    sync_ssh_connections_locally(std::slice::from_ref(&record))?;
    let all = load_ssh_connections()?;
    let saved = all
        .into_iter()
        .find(|c| c.alias.eq_ignore_ascii_case(alias))
        .unwrap_or(record);
    Ok(to_ssh_node_record(&saved))
}

/// Delete an SSH connection by alias or IP
pub fn delete_ssh_connection(id_or_alias: &str) -> Result<bool, String> {
    let target = id_or_alias.trim();
    if target.is_empty() {
        return Err("Node alias/id is empty".to_string());
    }
    let mut conns = load_ssh_connections()?;
    let before = conns.len();
    conns.retain(|c| {
        !c.alias.eq_ignore_ascii_case(target) && !c.ip_address.eq_ignore_ascii_case(target)
    });
    let removed = conns.len() < before;
    if removed {
        save_ssh_connections(&conns)?;
    }
    Ok(removed)
}
