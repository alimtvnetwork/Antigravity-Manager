use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

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
