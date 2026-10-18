use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Return `~/.agm/ssh` directory for AGM RSA vault keypair and SSH metadata
pub fn get_agm_ssh_vault_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let dir = home.join(".agm").join("ssh");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create ~/.agm/ssh: {}", e))?;
    }
    Ok(dir)
}

/// Candidate paths for auto-seeding SSH nodes from GitMap / repo-secrets
pub(crate) fn candidate_seed_node_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("gitmap-ssh-nodes.json"),
        PathBuf::from("04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/04-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/01-gitmap/04-ssh-nodes.json"),
        PathBuf::from("D:/work/repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("D:/work/repo-secrets/01-gitmap/04-ssh-nodes.json"),
    ];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".gitmap").join("gitmap-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("04-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("ssh-nodes.json"));
        paths.push(home.join("gitmap-ssh-nodes.json"));
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(&local_app_data)
                .join("gitmap-cli")
                .join("data")
                .join("gitmap-ssh-nodes.json"),
        );
        paths.push(
            PathBuf::from(&local_app_data)
                .join("gitmap-cli")
                .join("data")
                .join("04-ssh-nodes.json"),
        );
    }
    paths
}

/// Candidate paths for auto-seeding VM passwords and cluster fallback credentials
pub(crate) fn candidate_vmpass_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("vmpass.json"),
        PathBuf::from("06-vmpass.json"),
        PathBuf::from("../repo-secrets/vmpass.json"),
        PathBuf::from("../repo-secrets/06-vmpass.json"),
        PathBuf::from("../repo-secrets/01-gitmap/vmpass.json"),
        PathBuf::from("../repo-secrets/01-gitmap/06-vmpass.json"),
        PathBuf::from("../../repo-secrets/vmpass.json"),
        PathBuf::from("../../repo-secrets/01-gitmap/06-vmpass.json"),
        PathBuf::from("D:/work/repo-secrets/vmpass.json"),
        PathBuf::from("D:/work/repo-secrets/01-gitmap/06-vmpass.json"),
    ];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".gitmap").join("vmpass.json"));
        paths.push(home.join(".gitmap").join("06-vmpass.json"));
        paths.push(home.join("vmpass.json"));
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(&local_app_data)
                .join("gitmap-cli")
                .join("data")
                .join("vmpass.json"),
        );
        paths.push(
            PathBuf::from(&local_app_data)
                .join("gitmap-cli")
                .join("data")
                .join("06-vmpass.json"),
        );
    }
    paths
}

/// Resolve the local path to the GitMap CLI binary if available
pub fn resolve_gitmap_bin() -> Option<PathBuf> {
    if let Some(env_bin) = std::env::var_os("GITMAP_BIN") {
        let pb = PathBuf::from(env_bin);
        if pb.exists() {
            return Some(pb);
        }
    }

    let bin_name = if cfg!(target_os = "windows") {
        "gitmap.exe"
    } else {
        "gitmap"
    };

    let mut cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    if let Ok(out) = cmd.arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("gitmap"));
        }
    }

    for rel in &[
        PathBuf::from(".").join(bin_name),
        PathBuf::from("..").join("gitmap").join(bin_name),
        PathBuf::from("D:/work/gitmap").join(bin_name),
    ] {
        if rel.exists() {
            return Some(rel.clone());
        }
    }

    if let Ok(curr_exe) = std::env::current_exe() {
        if let Some(parent) = curr_exe.parent() {
            for cand in &[
                parent.join(bin_name),
                parent.join("..").join("gitmap").join(bin_name),
            ] {
                if cand.exists() {
                    return Some(cand.clone());
                }
            }
        }
    }

    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        let cand = PathBuf::from(cargo_home).join("bin").join(bin_name);
        if cand.exists() {
            return Some(cand);
        }
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        for cand in &[
            PathBuf::from(&local_app_data)
                .join("gitmap-cli")
                .join(bin_name),
            PathBuf::from(&local_app_data)
                .join("Programs")
                .join("gitmap")
                .join(bin_name),
        ] {
            if cand.exists() {
                return Some(cand.clone());
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        for cand in &[
            home.join(".cargo").join("bin").join(bin_name),
            home.join(".local").join("bin").join(bin_name),
            home.join("AppData")
                .join("Local")
                .join("gitmap-cli")
                .join(bin_name),
            home.join("work").join("gitmap").join(bin_name),
        ] {
            if cand.exists() {
                return Some(cand.clone());
            }
        }
    }

    None
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
