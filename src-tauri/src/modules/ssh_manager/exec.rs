use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::collections::HashSet;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::*;

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

pub(crate) fn is_connection_target_match(c: &SshConnectionRecord, token: &str) -> bool {
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

pub(crate) fn parse_adhoc_connection(token: &str) -> Option<SshConnectionRecord> {
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
pub(crate) fn build_ssh_args_for_node(
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
