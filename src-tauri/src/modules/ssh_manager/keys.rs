use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use std::io::Write;

use super::*;

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

pub(crate) fn compute_key_fingerprint(pub_path: &Path) -> String {
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
