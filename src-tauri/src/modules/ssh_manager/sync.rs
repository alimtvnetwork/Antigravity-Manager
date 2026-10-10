use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
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
