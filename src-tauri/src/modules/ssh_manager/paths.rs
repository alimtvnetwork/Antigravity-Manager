use std::fs;
use std::path::{Path, PathBuf};

use super::*;

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
pub(crate) fn candidate_seed_node_paths() -> Vec<PathBuf> {
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
