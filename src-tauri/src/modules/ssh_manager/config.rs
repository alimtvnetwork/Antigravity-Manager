use std::fs;

use super::*;

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

pub(crate) fn sanitize_ssh_config_content(content: &str) -> String {
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

pub(crate) fn build_managed_ssh_config_block() -> String {
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

pub(crate) fn replace_managed_block(content: &str, block: &str) -> String {
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
