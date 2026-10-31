use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Export SSH nodes to a JSON file and generate base64 one-liner
pub fn export_nodes_json(
    output_path: Option<&str>,
) -> Result<(PathBuf, SshNodesExportEnvelope, String), String> {
    let conns = load_ssh_connections()?;
    let envelope = build_envelope_from_connections(&conns);
    let target_file = match output_path.map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("gitmap-ssh-nodes.json"),
    };

    if let Some(parent) = target_file.parent() {
        if !parent.as_os_str().is_empty() {
            // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
            crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
        }
    }

    let full_envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/ssh-nodes", envelope.clone());
    let pretty = serde_json::to_string_pretty(&full_envelope)
        .map_err(|e| format!("Failed to serialize SSH nodes JSON: {}", e))?;
    fs::write(&target_file, &pretty)
        .map_err(|e| format!("Failed to write {}: {}", target_file.display(), e))?;

    let compact = serde_json::to_vec(&envelope).unwrap_or_default();
    let b64 = BASE64.encode(&compact);
    Ok((target_file, envelope, b64))
}

/// Import SSH nodes from a JSON file or base64 payload
pub fn import_nodes_json(
    input_path: Option<&str>,
    base64_payload: Option<&str>,
) -> Result<(String, NodeSyncStats), String> {
    if let Some(b64) = base64_payload.map(str::trim).filter(|s| !s.is_empty()) {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid base64 payload: {}", e))?;
        let raw = String::from_utf8(decoded)
            .map_err(|e| format!("Invalid UTF-8 in base64 payload: {}", e))?;
        let conns = decode_connections_from_json(&raw)?;
        let stats = sync_ssh_connections_locally(&conns)?;
        return Ok(("base64-inline".to_string(), stats));
    }

    let mut candidates = Vec::new();
    if let Some(p) = input_path.map(str::trim).filter(|s| !s.is_empty()) {
        candidates.push(PathBuf::from(p));
    }
    candidates.extend(candidate_seed_node_paths());

    for cand in candidates {
        if cand.exists() && cand.is_file() {
            let raw = fs::read_to_string(&cand)
                .map_err(|e| format!("Failed to read {}: {}", cand.display(), e))?;
            let conns = decode_connections_from_json(&raw)?;
            let stats = sync_ssh_connections_locally(&conns)?;
            return Ok((cand.to_string_lossy().to_string(), stats));
        }
    }

    Err("Could not find SSH nodes JSON file to import".to_string())
}

/// Parse `~/.ssh/config` host blocks into `SshConnectionRecord` items
pub fn parse_openssh_config_hosts(raw_config: &str) -> Vec<SshConnectionRecord> {
    let mut records = Vec::new();
    let mut cur_alias = String::new();
    let mut cur_hostname = String::new();
    let mut cur_user = String::new();
    let mut cur_port: u16 = 22;
    let mut cur_key = String::new();

    let flush = |alias: &str,
                 hostname: &str,
                 user: &str,
                 port: u16,
                 key: &str,
                 out: &mut Vec<SshConnectionRecord>| {
        let a = alias.trim();
        let h = if hostname.trim().is_empty() {
            a
        } else {
            hostname.trim()
        };
        if !a.is_empty() && !a.contains('*') && !a.contains('?') && !h.is_empty() {
            out.push(SshConnectionRecord {
                alias: a.to_string(),
                ip_address: h.to_string(),
                username: if user.trim().is_empty() {
                    "Administrator".to_string()
                } else {
                    user.trim().to_string()
                },
                port: if port == 0 { 22 } else { port },
                encrypted_password: String::new(),
                encryption_algo: "RSA-2048".to_string(),
                key_path: resolve_usable_key_path(key),
                os: "windows".to_string(),
                os_group: None,
                os_version: None,
                build_version: None,
                first_run_at: None,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            });
        }
    };

    for line in raw_config.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(kw) = parts.next() else {
            continue;
        };
        let rest = parts.collect::<Vec<_>>().join(" ");
        let rest_clean = rest.trim().trim_matches('"');

        if kw.eq_ignore_ascii_case("host") {
            flush(
                &cur_alias,
                &cur_hostname,
                &cur_user,
                cur_port,
                &cur_key,
                &mut records,
            );
            cur_alias = rest_clean
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            cur_hostname.clear();
            cur_user.clear();
            cur_port = 22;
            cur_key.clear();
        } else if kw.eq_ignore_ascii_case("hostname") {
            cur_hostname = rest_clean.to_string();
        } else if kw.eq_ignore_ascii_case("user") {
            cur_user = rest_clean.to_string();
        } else if kw.eq_ignore_ascii_case("port") {
            cur_port = rest_clean.parse::<u16>().unwrap_or(22);
        } else if kw.eq_ignore_ascii_case("identityfile") {
            cur_key = rest_clean.to_string();
        }
    }
    flush(
        &cur_alias,
        &cur_hostname,
        &cur_user,
        cur_port,
        &cur_key,
        &mut records,
    );
    records
}

/// Import SSH nodes from GitMap (`gitmap ssh nodes export-json`), `~/.gitmap/`, and `~/.ssh/config`
pub fn import_ssh_from_gitmap() -> Result<(String, NodeSyncStats), String> {
    let mut all_discovered: Vec<SshConnectionRecord> = Vec::new();
    let mut sources: Vec<String> = Vec::new();

    // 1. Try `gitmap cluster nodes --json`, `gitmap cluster nodes export-json`, or `gitmap ssh nodes`
    for args in [
        &["cluster", "nodes", "--json"][..],
        &["cluster", "nodes", "export-json"][..],
        &["ssh", "nodes", "export-json"][..],
        &["ssh", "nodes", "--json"][..],
    ] {
        let mut cmd = std::process::Command::new("gitmap");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        if let Ok(out) = cmd.args(args).output() {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(conns) = decode_connections_from_json(&stdout) {
                    if !conns.is_empty() {
                        all_discovered.extend(conns);
                        sources.push("gitmap-cli".to_string());
                        break;
                    }
                }
            }
        }
    }

    // 2. Try candidate JSON files on disk (`~/.gitmap/gitmap-ssh-nodes.json`, etc.)
    for cand in candidate_seed_node_paths() {
        if cand.exists() && cand.is_file() {
            if let Ok(raw) = fs::read_to_string(&cand) {
                if let Ok(conns) = decode_connections_from_json(&raw) {
                    if !conns.is_empty() {
                        all_discovered.extend(conns);
                        sources.push(cand.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }
    }

    // 3. Also inspect `~/.ssh/config` host blocks
    if let Ok(ssh_dir) = get_ssh_dir() {
        let cfg_path = ssh_dir.join("config");
        if cfg_path.exists() {
            if let Ok(raw_cfg) = fs::read_to_string(&cfg_path) {
                let cfg_hosts = parse_openssh_config_hosts(&raw_cfg);
                if !cfg_hosts.is_empty() {
                    all_discovered.extend(cfg_hosts);
                    sources.push("~/.ssh/config".to_string());
                }
            }
        }
    }

    if all_discovered.is_empty() {
        let existing = load_ssh_connections().unwrap_or_default();
        return Ok((
            "local-cache".to_string(),
            NodeSyncStats {
                total: existing.len(),
                matched: existing.len(),
                updated: 0,
                inserted: 0,
                unchanged: existing.len(),
            },
        ));
    }

    let stats = sync_ssh_connections_locally(&all_discovered)?;
    Ok((sources.join(" + "), stats))
}

/// Reset SSH connections to the canonical 6-node GitMap cluster topology (w1, main, u1, w2, w3, w4)
pub fn reset_ssh_cluster_nodes() -> Result<NodeSyncStats, String> {
    // 1. Scan candidate paths and existing connections to preserve credentials (passwords, key_path, port)
    let mut cred_map: HashMap<String, (String, String, String, u16)> = HashMap::new();

    let mut record_cred = |conn: &SshConnectionRecord| {
        let has_cred =
            !conn.encrypted_password.trim().is_empty() || !conn.key_path.trim().is_empty();
        if has_cred {
            let entry = (
                conn.encrypted_password.trim().to_string(),
                conn.encryption_algo.trim().to_string(),
                conn.key_path.trim().to_string(),
                if conn.port == 0 { 22 } else { conn.port },
            );
            let alias_key = conn.alias.trim().to_ascii_lowercase();
            if !alias_key.is_empty() {
                cred_map.insert(alias_key, entry.clone());
            }
            let ip_key = conn.ip_address.trim().to_string();
            if !ip_key.is_empty() {
                cred_map.insert(ip_key, entry);
            }
        }
    };

    for seed_path in candidate_seed_node_paths() {
        if seed_path.exists() && seed_path.is_file() {
            if let Ok(raw) = fs::read_to_string(&seed_path) {
                if let Ok(conns) = decode_connections_from_json(&raw) {
                    for c in conns {
                        record_cred(&c);
                    }
                }
            }
        }
    }

    if let Ok(existing) = load_ssh_connections() {
        for c in existing {
            record_cred(&c);
        }
    }

    // 2. Invalidate TCP liveness and cluster balances caches immediately
    invalidate_ssh_liveness_cache(None, None);
    invalidate_cluster_balances_cache();

    // 3. Clear stale cache: truncate node_display_cache.db and delete ssh_nodes.json
    if let Ok(data_dir) = crate::modules::account::get_data_dir() {
        let cache_db = data_dir.join("node_display_cache.db");
        if cache_db.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&cache_db) {
                // Justification: best-effort truncate node display cache before re-seeding
                crate::error::record_ignored(
                    conn.execute("DELETE FROM node_display_cache", []),
                    "truncate node_display_cache",
                );
            }
        }
        let nodes_json = data_dir.join("ssh_nodes.json");
        if nodes_json.exists() {
            // Justification: best effort remove stale ssh_nodes.json before re-seeding
            crate::error::record_ignored(
                fs::remove_file(&nodes_json),
                "remove stale ssh_nodes.json",
            );
        }
    }

    // 4. Re-seed all 6 canonical GitMap cluster nodes preserving credentials
    let mut canonical_records = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for &(alias, ip, user, os) in GITMAP_CLUSTER_NODES {
        let preserved = cred_map
            .get(&alias.to_ascii_lowercase())
            .or_else(|| cred_map.get(ip));

        let (enc_pw, enc_algo, key_p, port) = match preserved {
            Some((pw, algo, key, p)) => (
                pw.clone(),
                if algo.is_empty() {
                    "RSA-2048".to_string()
                } else {
                    algo.clone()
                },
                key.clone(),
                if *p == 0 { 22 } else { *p },
            ),
            None => (String::new(), "RSA-2048".to_string(), String::new(), 22),
        };

        canonical_records.push(SshConnectionRecord {
            alias: alias.to_string(),
            ip_address: ip.to_string(),
            port,
            username: user.to_string(),
            key_path: key_p,
            os: os.to_string(),
            encrypted_password: enc_pw,
            encryption_algo: enc_algo,
            os_group: Some(if os.eq_ignore_ascii_case("windows") {
                "Windows".to_string()
            } else {
                "Linux".to_string()
            }),
            os_version: None,
            build_version: None,
            first_run_at: Some(now.clone()),
            created_at: Some(now.clone()),
        });
    }

    let stats = sync_ssh_connections_locally(&canonical_records)?;
    // Ensure OpenSSH config reflects canonical entries
    // Justification: best-effort sync to ~/.ssh/config; logged
    crate::error::record_ignored(
        update_ssh_config(false),
        "reset_ssh_cluster update_ssh_config",
    );
    Ok(stats)
}
