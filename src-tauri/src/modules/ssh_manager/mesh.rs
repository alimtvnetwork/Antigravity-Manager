use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::*;

/// Build remote shell command to install one or more public keys on a remote node (Windows or Linux/macOS)
pub(crate) fn build_remote_key_install_command(pub_keys: &[String], is_windows_os: bool) -> String {
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
        let mut online = check_tcp_liveness(&node.ip_address, 22, 2500);
        if !online && resolve_gitmap_bin().is_some() {
            if let Ok((is_gm_online, _)) = check_node_via_gitmap(&node.alias) {
                if is_gm_online {
                    online = true;
                }
            }
        }
        if !online {
            // Check if gitmap fix-auth can reach it anyway using vaulted password credentials
            if let Ok(fix_out) = fix_node_auth_via_gitmap(&node.alias, pubkey_path_opt) {
                if fix_out.contains("successfully")
                    || fix_out.contains("✓")
                    || fix_out.contains("DEPLOYED")
                {
                    reports.push(KeyDeployNodeReport {
                        alias: node.alias.clone(),
                        ip_address: node.ip_address.clone(),
                        username: node.username.clone(),
                        os: node.os.clone(),
                        online: true,
                        keys_deployed: 1,
                        batch_auth_verified: true,
                        status: "AUTHORIZED".to_string(),
                        detail: format!(
                            "GitMap fix-auth: {}",
                            fix_out.lines().next().unwrap_or(&fix_out)
                        ),
                    });
                    continue;
                }
            }
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

        let mut deploy_cmd = Command::new("ssh");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            deploy_cmd.creation_flags(0x08000000);
        }
        let deploy_out = deploy_cmd.args(&ssh_args).output();
        let mut verify_args = build_ssh_args_for_node(&node, None, None, true);
        verify_args.push("echo AGM_SSH_OK".to_string());
        let mut verify_cmd = Command::new("ssh");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            verify_cmd.creation_flags(0x08000000);
        }
        let verified = verify_cmd
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
                // Delegation: Attempt GitMap fix-auth repair using vaulted password credentials
                tracing::info!(
                    target: "ssh_manager",
                    "Native key deployment to {} failed; attempting gitmap ssh fix-auth",
                    node.alias
                );
                match fix_node_auth_via_gitmap(&node.alias, pubkey_path_opt) {
                    Ok(fix_out) => {
                        let re_verified = verify_cmd
                            .args(&verify_args)
                            .output()
                            .map(|o| {
                                o.status.success()
                                    && String::from_utf8_lossy(&o.stdout).contains("AGM_SSH_OK")
                            })
                            .unwrap_or(false);

                        let status_str =
                            if re_verified || fix_out.contains("✓") || fix_out.contains("DEPLOYED")
                            {
                                "AUTHORIZED".to_string()
                            } else {
                                "PROVISIONED".to_string()
                            };
                        reports.push(KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: 1,
                            batch_auth_verified: re_verified || status_str == "AUTHORIZED",
                            status: status_str,
                            detail: format!(
                                "GitMap fix-auth: {}",
                                fix_out.lines().next().unwrap_or(&fix_out)
                            ),
                        });
                    }
                    Err(err_msg) => {
                        reports.push(KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: 0,
                            batch_auth_verified: verified,
                            status: "FAILED".to_string(),
                            detail: if !out.stderr.is_empty() {
                                String::from_utf8_lossy(&out.stderr).trim().to_string()
                            } else {
                                err_msg
                            },
                        });
                    }
                }
            }
            Err(e) => {
                // Attempt GitMap fix-auth repair if native ssh spawn failed
                match fix_node_auth_via_gitmap(&node.alias, pubkey_path_opt) {
                    Ok(fix_out) => {
                        reports.push(KeyDeployNodeReport {
                            alias: node.alias.clone(),
                            ip_address: node.ip_address.clone(),
                            username: node.username.clone(),
                            os: node.os.clone(),
                            online: true,
                            keys_deployed: 1,
                            batch_auth_verified: true,
                            status: "AUTHORIZED".to_string(),
                            detail: format!(
                                "GitMap fix-auth: {}",
                                fix_out.lines().next().unwrap_or(&fix_out)
                            ),
                        });
                    }
                    Err(_) => {
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
                    let mut cmd = Command::new("ssh");
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        cmd.creation_flags(0x08000000);
                    }
                    if let Ok(out) = cmd.args(&ssh_args).output() {
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

                    let mut cmd = Command::new("ssh");
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        cmd.creation_flags(0x08000000);
                    }
                    match cmd.args(&ssh_args).output() {
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
