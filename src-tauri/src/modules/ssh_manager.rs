use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SSH_CONFIG_MARKER_START: &str = "# --- agm ssh managed (do not edit) ---";
const SSH_CONFIG_MARKER_END: &str = "# --- end agm ssh managed ---";

fn default_ssh_port() -> u16 {
    22
}

fn default_os_type() -> String {
    "linux".to_string()
}

fn default_auth_method() -> String {
    "key".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodeExportItem {
    #[serde(default, alias = "workerId")]
    pub worker_id: String,
    #[serde(rename = "id", default)]
    pub numeric_id: usize,
    pub alias: String,
    #[serde(alias = "ipAddress")]
    pub ip_address: String,
    pub username: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default = "default_os_type")]
    pub os: String,
    #[serde(default = "default_auth_method", alias = "authMethod")]
    pub auth_method: String,
    #[serde(default, alias = "keyPath")]
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnectionRecord {
    pub alias: String,
    #[serde(alias = "ipAddress")]
    pub ip_address: String,
    pub username: String,
    #[serde(default, alias = "encryptedPassword")]
    pub encrypted_password: String,
    #[serde(default, alias = "keyPath")]
    pub key_path: String,
    #[serde(default = "default_os_type")]
    pub os: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "osGroup")]
    pub os_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "osVersion")]
    pub os_version: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "buildVersion"
    )]
    pub build_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "firstRunAt")]
    pub first_run_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodesExportEnvelope {
    pub schema_version: String,
    pub exported_at: String,
    pub total_nodes: usize,
    pub nodes: Vec<SshNodeExportItem>,
    pub connections: Vec<SshConnectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeSyncStats {
    pub total: usize,
    pub matched: usize,
    pub updated: usize,
    pub inserted: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyRecord {
    pub name: String,
    pub key_type: String,
    pub public_key: String,
    pub private_path: String,
    pub public_path: String,
    pub fingerprint: String,
    pub comment: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPublicKey {
    pub source_node: String,
    pub key_name: String,
    pub public_key: String,
    pub key_blob: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDeployNodeReport {
    pub alias: String,
    pub ip_address: String,
    pub username: String,
    pub os: String,
    pub online: bool,
    pub keys_deployed: usize,
    pub batch_auth_verified: bool,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDeploySummary {
    pub gathered_keys: Vec<DiscoveredPublicKey>,
    pub node_reports: Vec<KeyDeployNodeReport>,
    pub local_files_updated: usize,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecNodeResult {
    pub alias: String,
    pub target: String,
    pub ip_address: String,
    pub username: String,
    pub port: u16,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub success: bool,
}

/// Returns the user's `~/.ssh` directory
pub fn get_ssh_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let ssh_dir = home.join(".ssh");
    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)
            .map_err(|e| format!("Failed to create ~/.ssh directory: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Justification: best-effort permission hardening; logged
            crate::error::record_ignored(
                fs::set_permissions(&ssh_dir, fs::Permissions::from_mode(0o700)),
                "set_permissions",
            );
        }
    }
    Ok(ssh_dir)
}

/// Returns the path to `ssh_nodes.json` inside the AGM data directory
pub fn get_ssh_nodes_store_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("ssh_nodes.json"))
}

/// Candidate paths for auto-seeding SSH nodes from GitMap / repo-secrets
fn candidate_seed_node_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("gitmap-ssh-nodes.json"),
        PathBuf::from("D:/work/repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("D:/work/repo-secrets/01-gitmap/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/gitmap-ssh-nodes.json"),
    ];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".gitmap").join("gitmap-ssh-nodes.json"));
        paths.push(home.join("gitmap-ssh-nodes.json"));
    }
    paths
}

/// Resolve a usable local private key path, normalizing cross-machine paths if needed
pub fn resolve_usable_key_path(raw_key_path: &str) -> String {
    let trimmed = raw_key_path.trim();
    if !trimmed.is_empty() && Path::new(trimmed).exists() {
        return trimmed.to_string();
    }
    if let Ok(ssh_dir) = get_ssh_dir() {
        if !trimmed.is_empty() {
            if let Some(fname) = Path::new(&trimmed.replace('\\', "/")).file_name() {
                let candidate = ssh_dir.join(fname);
                if candidate.exists() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }
        for default_name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
            let p = ssh_dir.join(default_name);
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
    }
    trimmed.to_string()
}

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

fn nodes_to_connections(nodes: &[SshNodeExportItem]) -> Vec<SshConnectionRecord> {
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

/// Discover all local SSH key pairs in `~/.ssh`
pub fn discover_local_ssh_keys() -> Result<Vec<SshKeyRecord>, String> {
    let ssh_dir = get_ssh_dir()?;
    let mut keys = Vec::new();

    let entries =
        fs::read_dir(&ssh_dir).map_err(|e| format!("Failed to read ~/.ssh directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("pub") {
            continue;
        }
        let pub_content = match fs::read_to_string(&path) {
            Ok(c) => c.trim().to_string(),
            Err(_) => continue,
        };
        if pub_content.is_empty() {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("id_ed25519")
            .to_string();
        let priv_path = ssh_dir.join(&stem);

        let parts: Vec<&str> = pub_content.split_whitespace().collect();
        let key_type = parts
            .first()
            .map(|s| {
                if s.contains("ed25519") {
                    "ed25519"
                } else if s.contains("rsa") {
                    "rsa"
                } else if s.contains("ecdsa") {
                    "ecdsa"
                } else {
                    *s
                }
            })
            .unwrap_or("ed25519")
            .to_string();
        let comment = if parts.len() >= 3 {
            parts[2..].join(" ")
        } else {
            String::new()
        };

        let fingerprint = compute_key_fingerprint(&path);
        let created_at = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .map(|t| {
                let dt: chrono::DateTime<Utc> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

        keys.push(SshKeyRecord {
            name: stem,
            key_type,
            public_key: pub_content,
            private_path: priv_path.to_string_lossy().to_string(),
            public_path: path.to_string_lossy().to_string(),
            fingerprint,
            comment,
            created_at,
        });
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(keys)
}

fn compute_key_fingerprint(pub_path: &Path) -> String {
    if let Ok(out) = Command::new("ssh-keygen")
        .args(["-lf", &pub_path.to_string_lossy()])
        .output()
    {
        if out.status.success() {
            let line = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].to_string();
            }
        }
    }
    "SHA256:unknown".to_string()
}

/// Ensure at least one default SSH key exists (`id_ed25519` or `id_rsa`), adopting existing or generating new
pub fn ensure_default_ssh_key() -> Result<SshKeyRecord, String> {
    let keys = discover_local_ssh_keys()?;
    for pref in &["id_ed25519", "id_rsa"] {
        if let Some(found) = keys.iter().find(|k| k.name == *pref) {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(update_ssh_config(false), "update_ssh_config");
            return Ok(found.clone());
        }
    }
    if let Some(first) = keys.first() {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(update_ssh_config(false), "update_ssh_config");
        return Ok(first.clone());
    }
    create_ssh_key("id_ed25519", None)
}

/// Generate a new Ed25519 SSH key pair
pub fn create_ssh_key(name: &str, comment: Option<&str>) -> Result<SshKeyRecord, String> {
    let clean_name = if name.trim().is_empty() {
        "id_ed25519"
    } else {
        name.trim()
    };
    let ssh_dir = get_ssh_dir()?;
    let priv_path = ssh_dir.join(clean_name);
    let pub_path = ssh_dir.join(format!("{}.pub", clean_name));

    if priv_path.exists() {
        let keys = discover_local_ssh_keys()?;
        if let Some(existing) = keys.into_iter().find(|k| k.name == clean_name) {
            return Ok(existing);
        }
    }

    let default_comment = format!(
        "{}@{}",
        std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "admin".to_string()),
        std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "localhost".to_string())
    );
    let eff_comment = comment.unwrap_or(&default_comment);

    let out = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            &priv_path.to_string_lossy(),
            "-N",
            "",
            "-C",
            eff_comment,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ssh-keygen: {}", e))?;

    if !out.status.success() {
        return Err(format!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(update_ssh_config(false), "update_ssh_config");
    let keys = discover_local_ssh_keys()?;
    keys.into_iter()
        .find(|k| k.name == clean_name)
        .ok_or_else(|| {
            format!(
                "Created key {} not found after generation",
                pub_path.display()
            )
        })
}

/// Delete an SSH key pair by name
pub fn delete_ssh_key(name: &str) -> Result<String, String> {
    let clean = name.trim().trim_end_matches(".pub");
    if clean.is_empty() {
        return Err("Key name is required".to_string());
    }
    let ssh_dir = get_ssh_dir()?;
    let priv_path = ssh_dir.join(clean);
    let pub_path = ssh_dir.join(format!("{}.pub", clean));

    if !priv_path.exists() && !pub_path.exists() {
        return Err(format!(
            "SSH key '{}' not found in {}",
            clean,
            ssh_dir.display()
        ));
    }
    if priv_path.exists() {
        fs::remove_file(&priv_path)
            .map_err(|e| format!("Failed to delete {}: {}", priv_path.display(), e))?;
    }
    if pub_path.exists() {
        fs::remove_file(&pub_path)
            .map_err(|e| format!("Failed to delete {}: {}", pub_path.display(), e))?;
    }
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(update_ssh_config(false), "update_ssh_config");
    Ok(clean.to_string())
}

/// Copy an SSH public key to system clipboard
pub fn copy_ssh_public_key(name: Option<&str>) -> Result<SshKeyRecord, String> {
    let keys = discover_local_ssh_keys()?;
    if keys.is_empty() {
        return Err("No SSH keys found in ~/.ssh".to_string());
    }
    let target_key = if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
        let clean = n.trim_end_matches(".pub");
        keys.into_iter()
            .find(|k| k.name.eq_ignore_ascii_case(clean))
            .ok_or_else(|| format!("SSH key '{}' not found", clean))?
    } else {
        keys.into_iter()
            .find(|k| k.name == "id_ed25519")
            .or_else(|| discover_local_ssh_keys().ok()?.into_iter().next())
            .ok_or_else(|| "No default SSH key found".to_string())?
    };

    #[cfg(target_os = "windows")]
    {
        if let Ok(mut child) = Command::new("clip").stdin(Stdio::piped()).spawn() {
            if let Some(ref mut stdin) = child.stdin {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    stdin.write_all(target_key.public_key.as_bytes()),
                    "write_all",
                );
            }
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(child.wait(), "wait");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
            if let Some(ref mut stdin) = child.stdin {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    stdin.write_all(target_key.public_key.as_bytes()),
                    "write_all",
                );
            }
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(child.wait(), "wait");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    stdin.write_all(target_key.public_key.as_bytes()),
                    "write_all",
                );
            }
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(child.wait(), "wait");
        }
    }

    Ok(target_key)
}

/// Sanitize daemon-only SSH directives and update the managed `~/.ssh/config` block
pub fn update_ssh_config(sanitize_only: bool) -> Result<String, String> {
    let ssh_dir = get_ssh_dir()?;
    let config_path = ssh_dir.join("config");
    let existing = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    let sanitized = sanitize_ssh_config_content(&existing);
    if sanitize_only {
        fs::write(&config_path, &sanitized)
            .map_err(|e| format!("Failed to write {}: {}", config_path.display(), e))?;
        return Ok(sanitized);
    }

    let managed_block = build_managed_ssh_config_block();
    let updated = replace_managed_block(&sanitized, &managed_block);
    fs::write(&config_path, &updated)
        .map_err(|e| format!("Failed to write {}: {}", config_path.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Justification: best-effort permission hardening; logged
        crate::error::record_ignored(
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600)),
            "set_permissions",
        );
    }

    Ok(managed_block)
}

fn sanitize_ssh_config_content(content: &str) -> String {
    let daemon_directives = [
        "authorizedkeysfile",
        "authorizedkeyscommand",
        "authorizedkeyscommanduser",
        "authorizedprincipalsfile",
        "authorizedprincipalscommand",
        "subsystem",
        "permitrootlogin",
        "allowusers",
        "denyusers",
        "allowgroups",
        "denygroups",
        "clientaliveinterval",
        "clientalivecountmax",
        "strictmodes",
        "usepam",
        "maxauthtries",
        "maxsessions",
        "maxstartups",
        "pidfile",
        "printmotd",
        "printlastlog",
    ];

    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(first_word) = trimmed.split_whitespace().next() {
                let low = first_word.trim_end_matches('=').to_lowercase();
                if daemon_directives.contains(&low.as_str()) {
                    out.push(format!(
                        "# [agm removed server-only directive]: {}",
                        trimmed
                    ));
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn build_managed_ssh_config_block() -> String {
    let keys = discover_local_ssh_keys().unwrap_or_default();
    let conns = load_ssh_connections().unwrap_or_default();

    let default_key = keys
        .iter()
        .find(|k| k.name == "id_ed25519")
        .or_else(|| keys.iter().find(|k| k.name == "id_rsa"))
        .or_else(|| keys.first())
        .map(|k| k.private_path.clone())
        .unwrap_or_default();

    let mut b = String::new();
    b.push_str(SSH_CONFIG_MARKER_START);
    b.push('\n');

    if !default_key.is_empty() {
        b.push_str(&format!(
            "Host github.com\n    HostName github.com\n    User git\n    IdentityFile {}\n    IdentitiesOnly yes\n    PasswordAuthentication no\n\n",
            default_key
        ));
    }

    for c in conns {
        if c.alias.trim().is_empty() || c.ip_address.trim().is_empty() {
            continue;
        }
        let key_file = if !c.key_path.trim().is_empty() {
            resolve_usable_key_path(&c.key_path)
        } else {
            default_key.clone()
        };
        let user = if c.username.trim().is_empty() {
            "root"
        } else {
            c.username.trim()
        };
        b.push_str(&format!(
            "Host {}\n    HostName {}\n    User {}\n    Port 22\n",
            c.alias.trim(),
            c.ip_address.trim(),
            user
        ));
        if !key_file.is_empty() {
            b.push_str(&format!("    IdentityFile {}\n", key_file));
        }
        b.push_str("    StrictHostKeyChecking accept-new\n\n");
    }

    b.push_str(SSH_CONFIG_MARKER_END);
    b
}

fn replace_managed_block(content: &str, block: &str) -> String {
    if let (Some(start), Some(end)) = (
        content.find(SSH_CONFIG_MARKER_START),
        content.find(SSH_CONFIG_MARKER_END),
    ) {
        if end >= start {
            let end_idx = end + SSH_CONFIG_MARKER_END.len();
            let before = &content[..start];
            let after = &content[end_idx..];
            return format!("{}{}{}", before, block, after);
        }
    }
    if content.trim().is_empty() {
        format!("{}\n", block)
    } else {
        format!("{}\n\n{}\n", content.trim_end(), block)
    }
}

/// Validate a public key line and extract `(normalized_key_line, key_blob)`
pub fn validate_ssh_public_key(raw: &str) -> Result<(String, String), String> {
    let line = raw
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .ok_or_else(|| "Public key is empty".to_string())?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(
            "Invalid SSH public key format (expected: <type> <base64-blob> [comment])".to_string(),
        );
    }
    let key_type = parts[0];
    let is_valid_type = key_type.starts_with("ssh-") || key_type.starts_with("ecdsa-");
    if !is_valid_type {
        return Err(format!("Unsupported SSH public key type: {}", key_type));
    }
    let blob = parts[1].to_string();
    if blob.len() < 32 {
        return Err("SSH public key blob is too short".to_string());
    }
    Ok((line.to_string(), blob))
}

/// Install a public key into local `authorized_keys` (and `administrators_authorized_keys` on Windows) with deduplication
pub fn install_authorized_key_local(raw_key_or_path: &str) -> Result<Vec<String>, String> {
    let trimmed = raw_key_or_path.trim();
    let raw_content = if Path::new(trimmed).exists() && Path::new(trimmed).is_file() {
        fs::read_to_string(trimmed)
            .map_err(|e| format!("Failed to read public key file '{}': {}", trimmed, e))?
    } else {
        trimmed.to_string()
    };

    let (valid_key, key_blob) = validate_ssh_public_key(&raw_content)?;
    install_validated_key_local(&valid_key, &key_blob)
}

pub fn install_validated_key_local(valid_key: &str, key_blob: &str) -> Result<Vec<String>, String> {
    let mut updated_files = Vec::new();
    let ssh_dir = get_ssh_dir()?;
    let user_auth_keys = ssh_dir.join("authorized_keys");

    if append_key_if_missing(&user_auth_keys, valid_key, key_blob, false)? {
        updated_files.push(user_auth_keys.to_string_lossy().to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let program_data =
            std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        let admin_auth_keys = PathBuf::from(program_data)
            .join("ssh")
            .join("administrators_authorized_keys");
        if append_key_if_missing(&admin_auth_keys, valid_key, key_blob, true).unwrap_or(false) {
            updated_files.push(admin_auth_keys.to_string_lossy().to_string());
        }
    }

    Ok(updated_files)
}

fn append_key_if_missing(
    path: &Path,
    valid_key: &str,
    key_blob: &str,
    is_win_admin: bool,
) -> Result<bool, String> {
    if let Some(parent) = path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }
    let existing = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    let already_present = existing.lines().any(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        parts.len() >= 2 && parts[1] == key_blob
    });

    if already_present {
        return Ok(false);
    }

    let mut new_content = existing;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(valid_key);
    new_content.push('\n');

    fs::write(path, new_content.as_bytes())
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Justification: best-effort permission hardening; logged
        crate::error::record_ignored(
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)),
            "set_permissions",
        );
        // Justification: intentionally unused on Unix (only the Windows branch consumes it); suppresses the unused-variable warning.
        let _ = is_win_admin;
    }

    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy().to_string();
        if is_win_admin {
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(
                Command::new("icacls")
                    .args([
                        &path_str,
                        "/inheritance:r",
                        "/grant",
                        "SYSTEM:(F)",
                        "BUILTIN\\Administrators:(F)",
                    ])
                    .output(),
                "spawn icacls",
            );
        } else if let Ok(user) = std::env::var("USERNAME") {
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(
                Command::new("icacls")
                    .args([
                        &path_str,
                        "/inheritance:r",
                        "/grant",
                        &format!("{}:(F)", user),
                        "SYSTEM:(F)",
                    ])
                    .output(),
                "spawn icacls",
            );
        }
    }

    Ok(true)
}

/// Check TCP port 22 liveness for a host/IP
pub fn check_tcp_liveness(host: &str, port: u16, timeout_ms: u64) -> bool {
    let addr_str = format!("{}:{}", host.trim(), port);
    if let Ok(mut addrs) = addr_str.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            return TcpStream::connect_timeout(&sock_addr, Duration::from_millis(timeout_ms))
                .is_ok();
        }
    }
    if let Ok(sock_addr) = addr_str.parse::<SocketAddr>() {
        return TcpStream::connect_timeout(&sock_addr, Duration::from_millis(timeout_ms)).is_ok();
    }
    false
}

/// Resolve target string (`all`, `w1,w2`, `w1`, `user@ip`, or `ip`) into concrete `SshConnectionRecord`s
pub fn resolve_target_nodes(
    target: &str,
    except: Option<&str>,
) -> Result<Vec<SshConnectionRecord>, String> {
    let conns = load_ssh_connections().unwrap_or_default();
    let clean_target = target.trim();
    let is_all = clean_target.is_empty()
        || clean_target.eq_ignore_ascii_case("all")
        || clean_target.eq_ignore_ascii_case("nodes")
        || clean_target.eq_ignore_ascii_case("all-nodes");

    let exclusions: HashSet<String> = except
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut resolved = Vec::new();
    if is_all {
        resolved = conns;
    } else {
        let tokens: Vec<&str> = clean_target
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        for tok in tokens {
            let matched: Vec<SshConnectionRecord> = conns
                .iter()
                .filter(|c| is_connection_target_match(c, tok))
                .cloned()
                .collect();
            if !matched.is_empty() {
                for m in matched {
                    if !resolved.iter().any(|r: &SshConnectionRecord| {
                        r.ip_address == m.ip_address && r.alias == m.alias
                    }) {
                        resolved.push(m);
                    }
                }
            } else if let Some(adhoc) = parse_adhoc_connection(tok) {
                resolved.push(adhoc);
            }
        }
    }

    if !exclusions.is_empty() {
        resolved.retain(|c| {
            !exclusions.contains(&c.alias.to_lowercase())
                && !exclusions.contains(&c.ip_address.to_lowercase())
        });
    }

    Ok(resolved)
}

fn is_connection_target_match(c: &SshConnectionRecord, token: &str) -> bool {
    if c.alias.eq_ignore_ascii_case(token) || c.ip_address.eq_ignore_ascii_case(token) {
        return true;
    }
    let user_ip = format!("{}@{}", c.username, c.ip_address);
    let user_alias = format!("{}@{}", c.username, c.alias);
    if user_ip.eq_ignore_ascii_case(token) || user_alias.eq_ignore_ascii_case(token) {
        return true;
    }
    if let Some((_, host_part)) = token.rsplit_once('@') {
        return c.alias.eq_ignore_ascii_case(host_part)
            || c.ip_address.eq_ignore_ascii_case(host_part);
    }
    false
}

fn parse_adhoc_connection(token: &str) -> Option<SshConnectionRecord> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (user, host) = if let Some((u, h)) = trimmed.split_once('@') {
        (u.trim().to_string(), h.trim().to_string())
    } else {
        ("root".to_string(), trimmed.to_string())
    };
    if host.is_empty() {
        return None;
    }
    let os = if user.eq_ignore_ascii_case("administrator") {
        "windows".to_string()
    } else {
        "linux".to_string()
    };
    Some(SshConnectionRecord {
        alias: host.clone(),
        ip_address: host,
        username: user,
        encrypted_password: String::new(),
        key_path: resolve_usable_key_path(""),
        os,
        os_group: None,
        os_version: None,
        build_version: None,
        first_run_at: None,
        created_at: None,
    })
}

/// Build base `ssh` arguments for a connection record
fn build_ssh_args_for_node(
    node: &SshConnectionRecord,
    port_override: Option<u16>,
    identity_override: Option<&str>,
    batch_mode: bool,
) -> Vec<String> {
    let mut args = vec![
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-o".to_string(),
        "ConnectTimeout=6".to_string(),
    ];
    if batch_mode {
        args.push("-o".to_string());
        args.push("BatchMode=yes".to_string());
    }
    let port = port_override.unwrap_or(22);
    args.push("-p".to_string());
    args.push(port.to_string());

    let key = identity_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| resolve_usable_key_path(&node.key_path));

    if !key.is_empty() && Path::new(&key).exists() {
        args.push("-i".to_string());
        args.push(key);
    }

    let user = if node.username.trim().is_empty() {
        "root"
    } else {
        node.username.trim()
    };
    args.push(format!("{}@{}", user, node.ip_address.trim()));
    args
}

/// Execute an SSH command across one or more target nodes (sending & receiving stdout/stderr/exit_code)
pub fn exec_ssh_command(
    target: &str,
    command_args: &[String],
    port_override: Option<u16>,
    identity_override: Option<&str>,
    except: Option<&str>,
) -> Result<Vec<SshExecNodeResult>, String> {
    let nodes = resolve_target_nodes(target, except)?;
    if nodes.is_empty() {
        return Err(format!("No SSH target nodes matched '{}'", target));
    }

    let remote_cmd = command_args.join(" ");
    if remote_cmd.trim().is_empty() {
        return Err("Remote command cannot be empty".to_string());
    }

    let results: Vec<SshExecNodeResult> = std::thread::scope(|s| {
        let handles: Vec<_> = nodes
            .iter()
            .map(|node| {
                let cmd_str = remote_cmd.clone();
                let id_opt = identity_override.map(|s| s.to_string());
                s.spawn(move || {
                    let start = Instant::now();
                    let port = port_override.unwrap_or(22);
                    if !check_tcp_liveness(&node.ip_address, port, 2500) {
                        return SshExecNodeResult {
                            alias: node.alias.clone(),
                            target: format!("{}@{}", node.username, node.ip_address),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            port,
                            exit_code: 255,
                            stdout: String::new(),
                            stderr: format!(
                                "Node {} ({}) is unreachable on TCP port {}",
                                node.alias, node.ip_address, port
                            ),
                            duration_ms: start.elapsed().as_millis(),
                            success: false,
                        };
                    }

                    let mut ssh_args =
                        build_ssh_args_for_node(node, port_override, id_opt.as_deref(), true);
                    ssh_args.push(cmd_str);

                    match Command::new("ssh").args(&ssh_args).output() {
                        Ok(out) => {
                            let code = out.status.code().unwrap_or(1);
                            SshExecNodeResult {
                                alias: node.alias.clone(),
                                target: format!("{}@{}", node.username, node.ip_address),
                                ip_address: node.ip_address.clone(),
                                username: node.username.clone(),
                                port,
                                exit_code: code,
                                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                                duration_ms: start.elapsed().as_millis(),
                                success: out.status.success(),
                            }
                        }
                        Err(e) => SshExecNodeResult {
                            alias: node.alias.clone(),
                            target: format!("{}@{}", node.username, node.ip_address),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            port,
                            exit_code: 1,
                            stdout: String::new(),
                            stderr: format!("Failed to spawn ssh: {}", e),
                            duration_ms: start.elapsed().as_millis(),
                            success: false,
                        },
                    }
                })
            })
            .collect();

        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    Ok(results)
}

/// Build remote shell command to install one or more public keys on a remote node (Windows or Linux/macOS)
fn build_remote_key_install_command(pub_keys: &[String], is_windows_os: bool) -> String {
    let joined = pub_keys.join("\n");
    let b64 = BASE64.encode(joined.as_bytes());

    if is_windows_os {
        format!(
            "powershell -NoProfile -NonInteractive -Command \"$raw = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String('{}')); \
            $lines = $raw -split '`n' | ForEach-Object {{ $_.Trim() }} | Where-Object {{ $_ -ne '' }}; \
            $paths = @((Join-Path $HOME '.ssh\\authorized_keys'), 'C:\\ProgramData\\ssh\\administrators_authorized_keys'); \
            foreach ($p in $paths) {{ \
                $dir = Split-Path -Parent $p; \
                if (!(Test-Path $dir)) {{ New-Item -ItemType Directory -Force -Path $dir | Out-Null }}; \
                $existing = if (Test-Path $p) {{ Get-Content $p -Raw -ErrorAction SilentlyContinue }} else {{ '' }}; \
                if ($null -eq $existing) {{ $existing = '' }}; \
                $modified = $false; \
                foreach ($k in $lines) {{ \
                    $parts = $k -split '\\s+'; \
                    if ($parts.Count -ge 2) {{ \
                        $blob = $parts[1]; \
                        if ($existing -notmatch [regex]::Escape($blob)) {{ \
                            if ($existing.Length -gt 0 -and !$existing.EndsWith(\"`n\")) {{ $existing += \"`r`n\" }}; \
                            $existing += $k + \"`r`n\"; \
                            $modified = $true; \
                        }} \
                    }} \
                }}; \
                if ($modified -or !(Test-Path $p)) {{ Set-Content -Path $p -Value $existing -Encoding UTF8 -NoNewline }}; \
            }}; \
            icacls 'C:\\ProgramData\\ssh\\administrators_authorized_keys' /inheritance:r /grant 'SYSTEM:(F)' 'BUILTIN\\Administrators:(F)' | Out-Null; \
            Write-Output 'AUTH_KEYS_SYNCED_OK'\"",
            b64
        )
    } else {
        format!(
            "mkdir -p ~/.ssh && chmod 700 ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && \
            echo '{}' | base64 -d | while IFS= read -r line; do \
                blob=$(echo \"$line\" | awk '{{print $2}}'); \
                if [ -n \"$blob\" ] && ! grep -qF \"$blob\" ~/.ssh/authorized_keys 2>/dev/null; then \
                    echo \"$line\" >> ~/.ssh/authorized_keys; \
                fi; \
            done && echo AUTH_KEYS_SYNCED_OK",
            b64
        )
    }
}

/// Deploy local public key to one or more targets (`fix-auth` / `copy-id`)
pub fn deploy_auth_key_to_target(
    target: &str,
    pubkey_path_opt: Option<&str>,
) -> Result<Vec<KeyDeployNodeReport>, String> {
    let pub_key = if let Some(p) = pubkey_path_opt.map(str::trim).filter(|s| !s.is_empty()) {
        let content = fs::read_to_string(p)
            .map_err(|e| format!("Failed to read public key '{}': {}", p, e))?;
        let (valid, _) = validate_ssh_public_key(&content)?;
        valid
    } else {
        let key_rec = ensure_default_ssh_key()?;
        key_rec.public_key
    };

    let nodes = resolve_target_nodes(target, None)?;
    if nodes.is_empty() {
        return Err(format!("No target SSH nodes matched '{}'", target));
    }

    let pub_keys = vec![pub_key];
    let mut reports = Vec::new();

    for node in nodes {
        let online = check_tcp_liveness(&node.ip_address, 22, 2500);
        if !online {
            reports.push(KeyDeployNodeReport {
                alias: node.alias.clone(),
                ip_address: node.ip_address.clone(),
                username: node.username.clone(),
                os: node.os.clone(),
                online: false,
                keys_deployed: 0,
                batch_auth_verified: false,
                status: "OFFLINE".to_string(),
                detail: "TCP port 22 unreachable".to_string(),
            });
            continue;
        }

        let is_win = node.os.eq_ignore_ascii_case("windows")
            || node.username.eq_ignore_ascii_case("administrator");
        let remote_cmd = build_remote_key_install_command(&pub_keys, is_win);
        let mut ssh_args = build_ssh_args_for_node(&node, None, None, false);
        ssh_args.push(remote_cmd);

        let deploy_out = Command::new("ssh").args(&ssh_args).output();
        let mut verify_args = build_ssh_args_for_node(&node, None, None, true);
        verify_args.push("echo AGM_SSH_OK".to_string());
        let verified = Command::new("ssh")
            .args(&verify_args)
            .output()
            .map(|o| {
                o.status.success() && String::from_utf8_lossy(&o.stdout).contains("AGM_SSH_OK")
            })
            .unwrap_or(false);

        match deploy_out {
            Ok(out) if out.status.success() || verified => {
                reports.push(KeyDeployNodeReport {
                    alias: node.alias.clone(),
                    ip_address: node.ip_address.clone(),
                    username: node.username.clone(),
                    os: node.os.clone(),
                    online: true,
                    keys_deployed: 1,
                    batch_auth_verified: verified,
                    status: "AUTHORIZED".to_string(),
                    detail: "authorized_keys updated and verified".to_string(),
                });
            }
            Ok(out) => {
                reports.push(KeyDeployNodeReport {
                    alias: node.alias.clone(),
                    ip_address: node.ip_address.clone(),
                    username: node.username.clone(),
                    os: node.os.clone(),
                    online: true,
                    keys_deployed: 0,
                    batch_auth_verified: verified,
                    status: "FAILED".to_string(),
                    detail: String::from_utf8_lossy(&out.stderr).trim().to_string(),
                });
            }
            Err(e) => {
                reports.push(KeyDeployNodeReport {
                    alias: node.alias.clone(),
                    ip_address: node.ip_address.clone(),
                    username: node.username.clone(),
                    os: node.os.clone(),
                    online: true,
                    keys_deployed: 0,
                    batch_auth_verified: false,
                    status: "ERROR".to_string(),
                    detail: e.to_string(),
                });
            }
        }
    }

    Ok(reports)
}

/// Full 2-Phase Mesh Key Distribution (`agm ssh deploy keys [target] [--dry-run] [--except ...]`)
/// Phase 1: Gather public keys from local `~/.ssh/*.pub` and all online target nodes.
/// Phase 2: Deduplicate by key blob and distribute the unified `authorized_keys` bundle to local and all online target nodes.
pub fn deploy_mesh_keys(
    target: &str,
    except: Option<&str>,
    dry_run: bool,
) -> Result<MeshDeploySummary, String> {
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(ensure_default_ssh_key(), "ensure_default_ssh_key");
    let local_keys = discover_local_ssh_keys().unwrap_or_default();
    let mut gathered: Vec<DiscoveredPublicKey> = Vec::new();
    let mut seen_blobs: HashSet<String> = HashSet::new();

    for k in local_keys {
        if let Ok((valid_line, blob)) = validate_ssh_public_key(&k.public_key) {
            if seen_blobs.insert(blob.clone()) {
                gathered.push(DiscoveredPublicKey {
                    source_node: "local".to_string(),
                    key_name: format!("{}.pub", k.name),
                    public_key: valid_line,
                    key_blob: blob,
                });
            }
        }
    }

    let nodes = resolve_target_nodes(target, except)?;

    // Phase 1: Gather public keys from online remote nodes in parallel
    let remote_gathered: Vec<Vec<DiscoveredPublicKey>> = std::thread::scope(|s| {
        let handles: Vec<_> = nodes
            .iter()
            .map(|node| {
                s.spawn(move || {
                    let mut node_keys = Vec::new();
                    if !check_tcp_liveness(&node.ip_address, 22, 2000) {
                        return node_keys;
                    }
                    let is_win = node.os.eq_ignore_ascii_case("windows")
                        || node.username.eq_ignore_ascii_case("administrator");
                    let gather_cmd = if is_win {
                        "powershell -NoProfile -NonInteractive -Command \"Get-ChildItem -Path (Join-Path $HOME '.ssh\\*.pub') -ErrorAction SilentlyContinue | ForEach-Object { Write-Output ('FILE:' + $_.Name); Get-Content $_.FullName }\""
                    } else {
                        "for f in ~/.ssh/*.pub; do [ -f \"$f\" ] && echo \"FILE:$(basename \"$f\")\" && cat \"$f\"; done"
                    };
                    let mut ssh_args = build_ssh_args_for_node(node, None, None, true);
                    ssh_args.push(gather_cmd.to_string());
                    if let Ok(out) = Command::new("ssh").args(&ssh_args).output() {
                        if out.status.success() {
                            let stdout = String::from_utf8_lossy(&out.stdout);
                            let mut current_name = "id_ed25519.pub".to_string();
                            for line in stdout.lines() {
                                let trimmed = line.trim();
                                if let Some(fname) = trimmed.strip_prefix("FILE:") {
                                    current_name = fname.trim().to_string();
                                } else if let Ok((valid_line, blob)) =
                                    validate_ssh_public_key(trimmed)
                                {
                                    node_keys.push(DiscoveredPublicKey {
                                        source_node: node.alias.clone(),
                                        key_name: current_name.clone(),
                                        public_key: valid_line,
                                        key_blob: blob,
                                    });
                                }
                            }
                        }
                    }
                    node_keys
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    for list in remote_gathered {
        for rk in list {
            if seen_blobs.insert(rk.key_blob.clone()) {
                gathered.push(rk);
            }
        }
    }

    let all_pub_lines: Vec<String> = gathered.iter().map(|k| k.public_key.clone()).collect();
    let mut local_files_updated = 0;

    if !dry_run {
        for k in &gathered {
            if let Ok(files) = install_validated_key_local(&k.public_key, &k.key_blob) {
                local_files_updated += files.len();
            }
        }
    }

    // Phase 2: Distribute unified key bundle to all target nodes in parallel
    let node_reports: Vec<KeyDeployNodeReport> = std::thread::scope(|s| {
        let handles: Vec<_> = nodes
            .iter()
            .map(|node| {
                let pub_lines = all_pub_lines.clone();
                s.spawn(move || {
                    let online = check_tcp_liveness(&node.ip_address, 22, 2500);
                    if !online {
                        return KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: false,
                            keys_deployed: 0,
                            batch_auth_verified: false,
                            status: "OFFLINE".to_string(),
                            detail: "TCP port 22 unreachable".to_string(),
                        };
                    }
                    if dry_run {
                        return KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: pub_lines.len(),
                            batch_auth_verified: true,
                            status: "DRY-RUN".to_string(),
                            detail: format!(
                                "Would distribute {} key(s) to {}",
                                pub_lines.len(),
                                node.alias
                            ),
                        };
                    }

                    let is_win = node.os.eq_ignore_ascii_case("windows")
                        || node.username.eq_ignore_ascii_case("administrator");
                    let cmd = build_remote_key_install_command(&pub_lines, is_win);
                    let mut ssh_args = build_ssh_args_for_node(node, None, None, true);
                    ssh_args.push(cmd);

                    match Command::new("ssh").args(&ssh_args).output() {
                        Ok(out) if out.status.success() => KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: pub_lines.len(),
                            batch_auth_verified: true,
                            status: "SYNCED".to_string(),
                            detail: format!("Synced {} authorized key(s)", pub_lines.len()),
                        },
                        Ok(out) => KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: 0,
                            batch_auth_verified: false,
                            status: "AUTH-NEEDED".to_string(),
                            detail: String::from_utf8_lossy(&out.stderr).trim().to_string(),
                        },
                        Err(e) => KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: 0,
                            batch_auth_verified: false,
                            status: "ERROR".to_string(),
                            detail: e.to_string(),
                        },
                    }
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    Ok(MeshDeploySummary {
        gathered_keys: gathered,
        node_reports,
        local_files_updated,
        dry_run,
    })
}

/// Export full SSH bundle (`gitmap-ssh-nodes.json`, `ssh_keys.json`, and `keys/*.pub`)
pub fn export_all_bundle(dir_opt: Option<&str>) -> Result<(PathBuf, usize, usize), String> {
    let base_dir = match dir_opt.map(str::trim).filter(|s| !s.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from(".agm-ssh-bundle"),
    };
    let keys_dir = base_dir.join("keys");
    fs::create_dir_all(&keys_dir).map_err(|e| {
        format!(
            "Failed to create bundle directory {}: {}",
            keys_dir.display(),
            e
        )
    })?;

    let nodes_file = base_dir.join("gitmap-ssh-nodes.json");
    let (_, env, _) = export_nodes_json(Some(&nodes_file.to_string_lossy()))?;

    let keys = discover_local_ssh_keys().unwrap_or_default();
    let keys_manifest = serde_json::to_string_pretty(&keys)
        .map_err(|e| format!("Failed to serialize ssh_keys.json: {}", e))?;
    fs::write(base_dir.join("ssh_keys.json"), keys_manifest)
        .map_err(|e| format!("Failed to write ssh_keys.json: {}", e))?;

    for k in &keys {
        let dest = keys_dir.join(format!("{}.pub", k.name));
        // Justification: best-effort file write; failure is logged and surfaces on the next read
        crate::error::record_ignored(fs::write(dest, format!("{}\n", k.public_key)), "fs::write");
    }

    Ok((base_dir, keys.len(), env.total_nodes))
}

/// Import full SSH bundle (`gitmap-ssh-nodes.json` + `keys/*.pub`)
pub fn import_all_bundle(dir_opt: Option<&str>) -> Result<(usize, NodeSyncStats), String> {
    let base_dir = match dir_opt.map(str::trim).filter(|s| !s.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from(".agm-ssh-bundle"),
    };

    let nodes_file = base_dir.join("gitmap-ssh-nodes.json");
    let stats = if nodes_file.exists() {
        import_nodes_json(Some(&nodes_file.to_string_lossy()), None)?.1
    } else {
        import_nodes_json(None, None)
            .map(|r| r.1)
            .unwrap_or_default()
    };

    let mut installed_keys = 0;
    let keys_dir = base_dir.join("keys");
    if keys_dir.exists() {
        if let Ok(entries) = fs::read_dir(&keys_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("pub") {
                    if let Ok(updated) = install_authorized_key_local(&p.to_string_lossy()) {
                        if !updated.is_empty() {
                            installed_keys += 1;
                        }
                    }
                }
            }
        }
    }

    Ok((installed_keys, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ssh_public_key() {
        let sample = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl user@host";
        let (line, blob) = validate_ssh_public_key(sample).unwrap();
        assert_eq!(line, sample);
        assert_eq!(
            blob,
            "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"
        );
    }

    #[test]
    fn test_decode_connections_from_envelope() {
        let json = r#"{
            "schema_version": "1.0",
            "exported_at": "2026-09-27T10:10:07Z",
            "total_nodes": 1,
            "nodes": [
                {
                    "worker_id": "worker-1",
                    "id": 1,
                    "alias": "w1",
                    "ip_address": "192.168.1.3",
                    "username": "Administrator",
                    "port": 22,
                    "os": "windows",
                    "auth_method": "key",
                    "key_path": ""
                }
            ],
            "connections": []
        }"#;
        let conns = decode_connections_from_json(json).unwrap();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].alias, "w1");
        assert_eq!(conns[0].ip_address, "192.168.1.3");
    }
}
