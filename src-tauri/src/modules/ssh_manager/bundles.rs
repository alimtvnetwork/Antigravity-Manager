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

/// Deploy RSA-2048 encrypted SSH node bundle (`gitmap-ssh-nodes.json` + public keys) to remote machines
pub fn deploy_encrypted_ssh_to_nodes(
    target_aliases: Option<Vec<String>>,
) -> Result<MeshDeploySummary, String> {
    let target_filter = match target_aliases {
        Some(ref list) if !list.is_empty() => list.join(","),
        _ => "all".to_string(),
    };

    let conns = load_ssh_connections()?;
    let envelope = build_envelope_from_connections(&conns);
    let envelope_json = serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("Failed to serialize encrypted SSH envelope: {}", e))?;
    let b64_env = BASE64.encode(envelope_json.as_bytes());

    let mut summary = deploy_mesh_keys(&target_filter, None, false)?;

    // Also push the RSA-encrypted `ssh_nodes.json` / `gitmap-ssh-nodes.json` to online remote nodes
    let nodes = resolve_target_nodes(&target_filter, None).unwrap_or_default();
    for node in nodes {
        let port = if node.port == 0 { 22 } else { node.port };
        if !check_tcp_liveness(&node.ip_address, port, 2000) {
            continue;
        }
        let is_win = node.os.eq_ignore_ascii_case("windows")
            || node.username.eq_ignore_ascii_case("administrator");
        let sync_cmd = if is_win {
            format!(
                "powershell -NoProfile -NonInteractive -Command \"$raw = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String('{}')); \
                $gmDir = Join-Path $HOME '.gitmap'; if (!(Test-Path $gmDir)) {{ New-Item -ItemType Directory -Force -Path $gmDir | Out-Null }}; \
                Set-Content -Path (Join-Path $gmDir 'gitmap-ssh-nodes.json') -Value $raw -Encoding UTF8\"",
                b64_env
            )
        } else {
            format!(
                "mkdir -p ~/.gitmap && echo '{}' | base64 -d > ~/.gitmap/gitmap-ssh-nodes.json",
                b64_env
            )
        };
        // Justification: best-effort remote encrypted bundle distribution after key mesh deploy
        crate::error::record_ignored(
            exec_ssh_command(&node.alias, &[sync_cmd], Some(port), None, None),
            "deploy encrypted ssh bundle to node",
        );
    }

    summary.local_files_updated += 1;
    Ok(summary)
}
