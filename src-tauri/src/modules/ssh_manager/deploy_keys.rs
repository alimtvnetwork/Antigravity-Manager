use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::*;

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

pub(crate) fn append_key_if_missing(
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
