use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use super::*;

/// Thread-safe 10-second TTL liveness cache for SSH nodes, allowing offline nodes to fast-fail in <10ms
pub static SSH_LIVENESS_CACHE: Lazy<RwLock<HashMap<String, (bool, Instant)>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Thread-safe 30-second TTL cache for cluster instances and quota balances
pub static CLUSTER_BALANCES_CACHE: Lazy<
    RwLock<Option<(Vec<ClusterInstanceBalanceItem>, Instant)>>,
> = Lazy::new(|| RwLock::new(None));

/// Invalidate cluster balances cache on demand (e.g. cluster reset or instance update)
pub fn invalidate_cluster_balances_cache() {
    if let Ok(mut cache) = CLUSTER_BALANCES_CACHE.write() {
        *cache = None;
    }
}

/// Invalidate cached SSH liveness for a specific host or all nodes
pub fn invalidate_ssh_liveness_cache(host: Option<&str>, port: Option<u16>) {
    if let Ok(mut cache) = SSH_LIVENESS_CACHE.write() {
        if let Some(h) = host {
            let p = port.unwrap_or(22);
            cache.remove(&format!("{}:{}", h.trim(), p));
            for &(alias, ip, _, _) in GITMAP_CLUSTER_NODES {
                if h.eq_ignore_ascii_case(alias) {
                    cache.remove(&format!("{}:{}", ip, p));
                } else if h == ip {
                    cache.remove(&format!("{}:{}", alias, p));
                }
            }
        } else {
            cache.clear();
        }
    }
}

/// Check TCP port liveness for a host/IP with 10s TTL cache, 1500ms minimum timeout and 2-stage retry ladder
pub fn check_tcp_liveness(host: &str, port: u16, timeout_ms: u64) -> bool {
    let clean_host = host.trim();
    let cache_key = format!("{}:{}", clean_host, port);

    // Fast-fail check from 10s TTL cache (<10ms)
    if let Ok(cache) = SSH_LIVENESS_CACHE.read() {
        if let Some(&(is_alive, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed() < Duration::from_secs(10) {
                return is_alive;
            }
        }
    }

    let effective_timeout = if timeout_ms == 0 {
        1500
    } else {
        timeout_ms.max(1500)
    };
    let duration = Duration::from_millis(effective_timeout);

    let mut is_connected = false;

    for attempt in 0..2 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }

        let addr_str = format!("{}:{}", clean_host, port);
        let mut connected = false;

        if let Ok(mut addrs) = addr_str.to_socket_addrs() {
            if let Some(sock_addr) = addrs.next() {
                connected = TcpStream::connect_timeout(&sock_addr, duration).is_ok();
            }
        }
        if !connected {
            if let Ok(sock_addr) = addr_str.parse::<SocketAddr>() {
                connected = TcpStream::connect_timeout(&sock_addr, duration).is_ok();
            }
        }

        if connected {
            is_connected = true;
            break;
        }
    }

    // Populate 10s TTL cache with result
    if let Ok(mut cache) = SSH_LIVENESS_CACHE.write() {
        cache.insert(cache_key, (is_connected, Instant::now()));
        // If this host is in GITMAP_CLUSTER_NODES, also populate its counterpart (alias or IP)
        for &(alias, ip, _, _) in GITMAP_CLUSTER_NODES {
            if clean_host.eq_ignore_ascii_case(alias) {
                cache.insert(format!("{}:{}", ip, port), (is_connected, Instant::now()));
                break;
            } else if clean_host == ip {
                cache.insert(
                    format!("{}:{}", alias, port),
                    (is_connected, Instant::now()),
                );
                break;
            }
        }
    }

    is_connected
}

/// Canonical GitMap cluster node descriptors (alias, ip, default_user, os)
pub const GITMAP_CLUSTER_NODES: &[(&str, &str, &str, &str)] = &[
    ("w2", "192.168.1.7", "Administrator", "windows"),
    ("w1", "192.168.1.3", "Administrator", "windows"),
    ("w3", "192.168.1.12", "Administrator", "windows"),
    ("w4", "192.168.1.13", "Administrator", "linux"),
    ("u1", "192.168.1.22", "root", "linux"),
    ("main", "192.168.1.20", "Administrator", "windows"),
];

/// Resolve a GitMap cluster node fallback from alias or IP
pub fn resolve_gitmap_cluster_fallback(token: &str) -> Option<SshConnectionRecord> {
    let clean = token.trim();
    if clean.is_empty() {
        return None;
    }
    let host_part = if let Some((_, h)) = clean.rsplit_once('@') {
        h.trim()
    } else {
        clean
    };
    for &(alias, ip, user, os) in GITMAP_CLUSTER_NODES {
        let is_alias_match = host_part.eq_ignore_ascii_case(alias);
        let is_ip_match = host_part == ip;
        if is_alias_match || is_ip_match {
            return Some(SshConnectionRecord {
                alias: alias.to_string(),
                ip_address: ip.to_string(),
                username: user.to_string(),
                port: 22,
                encrypted_password: String::new(),
                encryption_algo: "RSA-2048".to_string(),
                key_path: resolve_usable_key_path(""),
                os: os.to_string(),
                os_group: None,
                os_version: None,
                build_version: None,
                first_run_at: None,
                created_at: None,
            });
        }
    }
    None
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
            if matched.is_empty() {
                if let Some(gm_node) = resolve_gitmap_cluster_fallback(tok) {
                    resolved.push(gm_node);
                } else if let Some(adhoc) = parse_adhoc_connection(tok) {
                    resolved.push(adhoc);
                }
                continue;
            }

            for m in matched {
                let is_already_resolved = resolved.iter().any(|r: &SshConnectionRecord| {
                    r.ip_address == m.ip_address && r.alias == m.alias
                });
                if is_already_resolved {
                    continue;
                }
                resolved.push(m);
            }
        }
    }

    let has_exclusions = !exclusions.is_empty();
    if has_exclusions {
        resolved.retain(|c| {
            let is_alias_excluded = exclusions.contains(&c.alias.to_lowercase());
            let is_ip_excluded = exclusions.contains(&c.ip_address.to_lowercase());
            let is_excluded = is_alias_excluded || is_ip_excluded;
            let is_retained = !is_excluded;
            is_retained
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
        if c.alias.eq_ignore_ascii_case(host_part) || c.ip_address.eq_ignore_ascii_case(host_part) {
            return true;
        }
    }
    for &(alias, ip, _, _) in GITMAP_CLUSTER_NODES {
        let is_token_alias = token.eq_ignore_ascii_case(alias);
        let is_token_ip = token == ip;
        let is_conn_alias = c.alias.eq_ignore_ascii_case(alias);
        let is_conn_ip = c.ip_address == ip;
        if (is_token_alias && (is_conn_alias || is_conn_ip))
            || (is_token_ip && (is_conn_alias || is_conn_ip))
        {
            return true;
        }
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
        port: 22,
        encrypted_password: String::new(),
        encryption_algo: "RSA-2048".to_string(),
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
        "PubkeyAcceptedKeyTypes=+ssh-rsa".to_string(),
        "-o".to_string(),
        "ConnectTimeout=3".to_string(),
    ];
    if batch_mode {
        args.push("-o".to_string());
        args.push("BatchMode=yes".to_string());
    }
    let port = port_override.unwrap_or(if node.port == 0 { 22 } else { node.port });
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
                    let is_w2 =
                        node.alias.eq_ignore_ascii_case("w2") || node.ip_address == "192.168.1.7";
                    let mut is_tcp_online = check_tcp_liveness(&node.ip_address, port, 2500);
                    let is_gitmap_available = resolve_gitmap_bin().is_some();

                    if !is_tcp_online && is_gitmap_available {
                        if let Ok((is_gm_online, _)) = check_node_via_gitmap(&node.alias) {
                            if is_gm_online {
                                is_tcp_online = true;
                            }
                        }
                    }

                    if !is_tcp_online {
                        // Forward/fallback to gitmap ssh exec directly before declaring unreachable
                        let candidate_targets = if node.alias.eq_ignore_ascii_case(&node.ip_address)
                        {
                            vec![node.alias.as_str()]
                        } else {
                            vec![node.alias.as_str(), node.ip_address.as_str()]
                        };
                        for target_id in &candidate_targets {
                            if let Ok((gm_code, gm_out, gm_err)) =
                                exec_via_gitmap_ssh(target_id, &cmd_str)
                            {
                                let is_gm_success = gm_code == 0
                                    && !gm_out.contains("machine is off")
                                    && !gm_out.contains("All target machines are currently off")
                                    && !gm_out.to_lowercase().contains("connection timed out");
                                if is_gm_success {
                                    return SshExecNodeResult {
                                        alias: node.alias.clone(),
                                        target: format!("{}@{}", node.username, node.ip_address),
                                        ip_address: node.ip_address.clone(),
                                        username: node.username.clone(),
                                        port,
                                        exit_code: 0,
                                        stdout: gm_out,
                                        stderr: gm_err,
                                        duration_ms: start.elapsed().as_millis(),
                                        success: true,
                                    };
                                }
                            }
                        }

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
                    ssh_args.push(cmd_str.clone());

                    let mut cmd = Command::new("ssh");
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        cmd.creation_flags(0x08000000);
                    }
                    match cmd.args(&ssh_args).output() {
                        Ok(out) => {
                            let code = out.status.code().unwrap_or(1);
                            let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
                            let stderr_str = String::from_utf8_lossy(&out.stderr).to_string();

                            let is_exit_success = out.status.success();
                            let is_auth_failure = !is_exit_success
                                && (code == 255
                                    || stderr_str.to_lowercase().contains("permission denied")
                                    || stderr_str.to_lowercase().contains("authentication failed")
                                    || is_w2);

                            if is_auth_failure {
                                // Fallback execution branch to delegate to GitMap:
                                // gitmap ssh exec <node_alias_or_ip> "<command>"
                                let candidate_targets =
                                    if node.alias.eq_ignore_ascii_case(&node.ip_address) {
                                        vec![node.alias.as_str()]
                                    } else {
                                        vec![node.alias.as_str(), node.ip_address.as_str()]
                                    };

                                for target_id in &candidate_targets {
                                    if let Ok((gm_code, gm_out, gm_err)) =
                                        exec_via_gitmap_ssh(target_id, &cmd_str)
                                    {
                                        let is_gm_success = gm_code == 0
                                            && !gm_out.contains("machine is off")
                                            && !gm_out
                                                .contains("All target machines are currently off")
                                            && !gm_out
                                                .to_lowercase()
                                                .contains("connection timed out");
                                        if is_gm_success {
                                            return SshExecNodeResult {
                                                alias: node.alias.clone(),
                                                target: format!(
                                                    "{}@{}",
                                                    node.username, node.ip_address
                                                ),
                                                ip_address: node.ip_address.clone(),
                                                username: node.username.clone(),
                                                port,
                                                exit_code: 0,
                                                stdout: gm_out,
                                                stderr: gm_err,
                                                duration_ms: start.elapsed().as_millis(),
                                                success: true,
                                            };
                                        }
                                    }
                                }

                                // Stage 3: Auto-repair via GitMap fix-auth if target is authorized via password vault
                                for target_id in &candidate_targets {
                                    if let Ok(fix_msg) = fix_node_auth_via_gitmap(target_id, None) {
                                        tracing::info!(
                                            target: "ssh_manager",
                                            "Auto-repaired auth via GitMap for {}: {}",
                                            target_id,
                                            fix_msg
                                        );
                                        // Retry native OpenSSH once after key deployment
                                        let mut retry_cmd = Command::new("ssh");
                                        #[cfg(target_os = "windows")]
                                        {
                                            use std::os::windows::process::CommandExt;
                                            retry_cmd.creation_flags(0x08000000);
                                        }
                                        if let Ok(retry_out) = retry_cmd.args(&ssh_args).output() {
                                            let r_code = retry_out.status.code().unwrap_or(1);
                                            if retry_out.status.success() {
                                                return SshExecNodeResult {
                                                    alias: node.alias.clone(),
                                                    target: format!(
                                                        "{}@{}",
                                                        node.username, node.ip_address
                                                    ),
                                                    ip_address: node.ip_address.clone(),
                                                    username: node.username.clone(),
                                                    port,
                                                    exit_code: r_code,
                                                    stdout: String::from_utf8_lossy(
                                                        &retry_out.stdout,
                                                    )
                                                    .to_string(),
                                                    stderr: String::from_utf8_lossy(
                                                        &retry_out.stderr,
                                                    )
                                                    .to_string(),
                                                    duration_ms: start.elapsed().as_millis(),
                                                    success: true,
                                                };
                                            }
                                        }

                                        // If native retry failed, retry gitmap ssh exec once more after key fix
                                        if let Ok((r_gm_code, r_gm_out, r_gm_err)) =
                                            exec_via_gitmap_ssh(target_id, &cmd_str)
                                        {
                                            let is_retry_gm_success = r_gm_code == 0
                                                && !r_gm_out.contains("machine is off")
                                                && !r_gm_out
                                                    .to_lowercase()
                                                    .contains("connection timed out");
                                            if is_retry_gm_success {
                                                return SshExecNodeResult {
                                                    alias: node.alias.clone(),
                                                    target: format!(
                                                        "{}@{}",
                                                        node.username, node.ip_address
                                                    ),
                                                    ip_address: node.ip_address.clone(),
                                                    username: node.username.clone(),
                                                    port,
                                                    exit_code: 0,
                                                    stdout: r_gm_out,
                                                    stderr: r_gm_err,
                                                    duration_ms: start.elapsed().as_millis(),
                                                    success: true,
                                                };
                                            }
                                        }
                                        break;
                                    }
                                }
                            }

                            SshExecNodeResult {
                                alias: node.alias.clone(),
                                target: format!("{}@{}", node.username, node.ip_address),
                                ip_address: node.ip_address.clone(),
                                username: node.username.clone(),
                                port,
                                exit_code: code,
                                stdout: stdout_str,
                                stderr: stderr_str,
                                duration_ms: start.elapsed().as_millis(),
                                success: out.status.success(),
                            }
                        }
                        Err(e) => {
                            let candidate_targets =
                                if node.alias.eq_ignore_ascii_case(&node.ip_address) {
                                    vec![node.alias.as_str()]
                                } else {
                                    vec![node.alias.as_str(), node.ip_address.as_str()]
                                };
                            for target_id in &candidate_targets {
                                if let Ok((gm_code, gm_out, gm_err)) =
                                    exec_via_gitmap_ssh(target_id, &cmd_str)
                                {
                                    let is_gm_success =
                                        gm_code == 0 && !gm_out.contains("machine is off");
                                    if is_gm_success {
                                        return SshExecNodeResult {
                                            alias: node.alias.clone(),
                                            target: format!(
                                                "{}@{}",
                                                node.username, node.ip_address
                                            ),
                                            ip_address: node.ip_address.clone(),
                                            username: node.username.clone(),
                                            port,
                                            exit_code: 0,
                                            stdout: gm_out,
                                            stderr: gm_err,
                                            duration_ms: start.elapsed().as_millis(),
                                            success: true,
                                        };
                                    }
                                }
                            }
                            SshExecNodeResult {
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
                            }
                        }
                    }
                })
            })
            .collect();

        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    Ok(results)
}

/// Invoke `gitmap ssh fix-auth <target>` to automatically deploy keys using vaulted password credentials
pub fn fix_node_auth_via_gitmap(target: &str, pubkey_path: Option<&str>) -> Result<String, String> {
    let gitmap_bin = resolve_gitmap_bin()
        .ok_or_else(|| "GitMap CLI is not installed or detected in PATH".to_string())?;

    let mut cmd = Command::new(gitmap_bin);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.args(["ssh", "fix-auth", target.trim()]);
    let resolved_key_path = pubkey_path
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| ensure_default_ssh_key().ok().map(|k| k.public_path));
    if let Some(ref pk) = resolved_key_path {
        cmd.args(["-i", pk]);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("Failed to spawn gitmap ssh fix-auth: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

    if out.status.success() {
        tracing::info!(
            target: "ssh_manager",
            "gitmap ssh fix-auth for {target} succeeded: {stdout}"
        );
        Ok(stdout)
    } else {
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        tracing::warn!(
            target: "ssh_manager",
            "gitmap ssh fix-auth for {target} returned error: {msg}"
        );
        Err(msg)
    }
}

/// Execute command on remote host via `gitmap ssh exec <target> "<command>"`
pub fn exec_via_gitmap_ssh(
    target: &str,
    remote_cmd: &str,
) -> Result<(i32, String, String), String> {
    let gitmap_bin =
        resolve_gitmap_bin().ok_or_else(|| "GitMap CLI is not installed".to_string())?;

    let mut cmd = Command::new(gitmap_bin);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.args(["ssh", "exec", target.trim(), remote_cmd]);
    let out = cmd
        .output()
        .map_err(|e| format!("Failed to spawn gitmap ssh exec: {}", e))?;
    let code = out
        .status
        .code()
        .unwrap_or(if out.status.success() { 0 } else { 1 });
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    Ok((code, stdout, stderr))
}

/// Check node connectivity via `gitmap ssh check <target>`
pub fn check_node_via_gitmap(target: &str) -> Result<(bool, String), String> {
    let gitmap_bin =
        resolve_gitmap_bin().ok_or_else(|| "GitMap CLI is not installed".to_string())?;

    let mut cmd = Command::new(gitmap_bin);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.args(["ssh", "check", target.trim()]);
    let out = cmd
        .output()
        .map_err(|e| format!("Failed to run gitmap ssh check: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let is_online = stdout.contains("ONLINE")
        || stdout.contains("reachable")
        || stdout.contains("ready")
        || stdout.contains("●");
    Ok((is_online, stdout.trim().to_string()))
}

/// Suggested CLI command to install GitMap when not detected
pub const GITMAP_SUGGESTED_INSTALL_CMD: &str =
    "cargo install --git https://github.com/aukgit/gitmap gitmap --force";

/// Inspect local GitMap installation and RSA-2048 SSH vault status
pub fn check_gitmap_installed_status() -> SshGitmapStatus {
    let mut gitmap_installed = false;
    let mut gitmap_version = None;
    let mut gitmap_path = None;

    let mut gm_cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        gm_cmd.creation_flags(0x08000000);
    }
    if let Ok(out) = gm_cmd.arg("--version").output() {
        if out.status.success() {
            gitmap_installed = true;
            let v_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            gitmap_version = Some(if v_str.is_empty() {
                "installed".to_string()
            } else {
                v_str
            });
        }
    }

    if gitmap_installed {
        #[cfg(target_os = "windows")]
        let which_cmd = ("where.exe", "gitmap");
        #[cfg(not(target_os = "windows"))]
        let which_cmd = ("which", "gitmap");

        let mut which_proc = Command::new(which_cmd.0);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            which_proc.creation_flags(0x08000000);
        }
        if let Ok(out) = which_proc.arg(which_cmd.1).output() {
            if out.status.success() {
                if let Some(first_line) = String::from_utf8_lossy(&out.stdout).lines().next() {
                    gitmap_path = Some(first_line.trim().to_string());
                }
            }
        }
    }

    let rsa_fp = ensure_agm_rsa_vault_key().unwrap_or_else(|_| "RSA-2048:ready".to_string());
    let ssh_config_path = get_ssh_dir()
        .map(|d| d.join("config").to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.ssh/config".to_string());
    let total_nodes = load_ssh_connections().map(|c| c.len()).unwrap_or(0);

    SshGitmapStatus {
        gitmap_installed,
        gitmap_version,
        gitmap_path,
        rsa_vault_ready: true,
        rsa_public_key_fingerprint: rsa_fp,
        ssh_config_path,
        suggested_install_cmd: GITMAP_SUGGESTED_INSTALL_CMD.to_string(),
        total_nodes,
    }
}

/// Launch PowerShell (or shell) to install or upgrade GitMap CLI
pub fn install_gitmap_via_powershell(open_window: bool) -> Result<String, String> {
    let status = check_gitmap_installed_status();
    let install_script = format!(
        "Write-Host '🚀 Installing / Updating GitMap CLI for AGM Multi-Machine Control...' -ForegroundColor Cyan; \
        if (Get-Command gitmap -ErrorAction SilentlyContinue) {{ gitmap self-update }} \
        elseif (Get-Command cargo -ErrorAction SilentlyContinue) {{ {} }} \
        else {{ Write-Host 'Run: {}' -ForegroundColor Yellow }}",
        GITMAP_SUGGESTED_INSTALL_CMD, GITMAP_SUGGESTED_INSTALL_CMD
    );

    if open_window {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("powershell.exe");
            cmd.args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Start-Process powershell.exe -ArgumentList '-NoExit','-NoProfile','-Command','{}'",
                    install_script.replace('\'', "''")
                ),
            ]);
            let spawn_res = crate::modules::process::spawn_with_job_breakaway_fallback(&mut cmd);
            if let Err(e) = spawn_res {
                return Err(format!(
                    "Failed to launch PowerShell installer window: {}. Suggested command: {}",
                    e, GITMAP_SUGGESTED_INSTALL_CMD
                ));
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Justification: best-effort background shell launch on Unix
            crate::error::record_ignored(
                Command::new("sh")
                    .args(["-c", GITMAP_SUGGESTED_INSTALL_CMD])
                    .spawn(),
                "spawn gitmap install sh",
            );
        }
    }

    if status.gitmap_installed {
        Ok(format!(
            "GitMap is installed ({}). Triggered update / verification. Manual command: `{}`",
            status
                .gitmap_version
                .unwrap_or_else(|| "latest".to_string()),
            GITMAP_SUGGESTED_INSTALL_CMD
        ))
    } else {
        Ok(format!(
            "GitMap CLI was not detected. PowerShell installer launched! You can also run: `{}`",
            GITMAP_SUGGESTED_INSTALL_CMD
        ))
    }
}

/// Strip leading logger outputs (e.g. `INFO Log system initialized...`) before parsing JSON
pub(crate) fn extract_clean_json(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let obj_start = trimmed.find('{');
    let arr_start = trimmed.find('[');
    let start_idx = match (obj_start, arr_start) {
        (Some(o), Some(a)) => Some(o.min(a)),
        (Some(o), None) => Some(o),
        (None, Some(a)) => Some(a),
        (None, None) => None,
    }?;

    let is_obj = trimmed[start_idx..].starts_with('{');
    let end_char = if is_obj { '}' } else { ']' };
    let end_idx = trimmed.rfind(end_char)?;

    if end_idx >= start_idx {
        Some(&trimmed[start_idx..=end_idx])
    } else {
        None
    }
}

/// Return multi-machine instance & bound account balances across local machine and SSH/GitMap nodes
pub fn get_cluster_instances_and_balances() -> Result<Vec<ClusterInstanceBalanceItem>, String> {
    if let Ok(cache) = CLUSTER_BALANCES_CACHE.read() {
        if let Some((ref cached_items, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(30) {
                return Ok(cached_items.clone());
            }
        }
    }

    let mut items = Vec::new();

    let local_machine = crate::modules::email_watcher::detect_machine_name();
    let local_ip = crate::modules::email_watcher::detect_local_ip();
    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let target_model = app_cfg.auto_profile_switcher.target_model.clone();

    let ssh_conns = load_ssh_connections().unwrap_or_default();
    let matched_local_node = ssh_conns
        .iter()
        .find(|conn| !local_ip.is_empty() && conn.ip_address.trim() == local_ip.trim())
        .or_else(|| {
            ssh_conns
                .iter()
                .find(|conn| conn.alias.eq_ignore_ascii_case(&local_machine))
        });
    let effective_local_alias = matched_local_node
        .map(|n| n.alias.clone())
        .unwrap_or_else(|| local_machine.clone());

    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    let accounts = crate::modules::account::list_accounts().unwrap_or_default();
    let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();

    if reg.instances.is_empty() {
        let default_acc = accounts
            .iter()
            .find(|a| curr_id.as_deref() == Some(&a.id))
            .or_else(|| accounts.first());
        let (email, tier, q_4h, q_wk) = if let Some(acc) = default_acc {
            let t = acc
                .quota
                .as_ref()
                .and_then(|q| q.subscription_tier.as_deref())
                .unwrap_or("PRO")
                .to_string();
            let (h, w) =
                crate::modules::auto_switcher::extract_dual_window_quotas(acc, &target_model);
            (acc.email.clone(), t, h.unwrap_or(100.0), w.unwrap_or(100.0))
        } else {
            ("(unbound)".to_string(), "PRO".to_string(), 100.0, 100.0)
        };
        items.push(ClusterInstanceBalanceItem {
            machine_alias: effective_local_alias.clone(),
            machine_ip: local_ip.clone(),
            ip_address: Some(local_ip.clone()),
            error_message: None,
            latency_ms: Some(0),
            is_local: true,
            instance_id: "default".to_string(),
            instance_name: "default".to_string(),
            running: true,
            bound_email: email,
            tier,
            quota_4h_pct: q_4h,
            quota_weekly_pct: q_wk,
            active_model: target_model.clone(),
            status_label: "online".to_string(),
        });
    } else {
        for inst in &reg.instances {
            let pids =
                crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
            let running = !pids.is_empty()
                || crate::modules::instance::is_instance_running(
                    &inst.id,
                    &inst.data_dir,
                    inst.pid,
                );
            let bound_email = inst
                .bound_email
                .clone()
                .or_else(|| {
                    if inst.is_default || inst.id == "default" {
                        accounts
                            .iter()
                            .find(|a| curr_id.as_deref() == Some(&a.id))
                            .map(|a| a.email.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "(unbound)".to_string());
            let matched_acc = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&bound_email));
            let tier = matched_acc
                .and_then(|a| a.quota.as_ref())
                .and_then(|q| q.subscription_tier.as_deref())
                .unwrap_or("PRO")
                .to_string();
            let (q_4h, q_wk) = matched_acc
                .map(|a| {
                    crate::modules::auto_switcher::extract_dual_window_quotas(a, &target_model)
                })
                .unwrap_or((Some(100.0), Some(100.0)));

            items.push(ClusterInstanceBalanceItem {
                machine_alias: effective_local_alias.clone(),
                machine_ip: local_ip.clone(),
                ip_address: Some(local_ip.clone()),
                error_message: None,
                latency_ms: Some(0),
                is_local: true,
                instance_id: inst.id.clone(),
                instance_name: inst.name.clone(),
                running,
                bound_email,
                tier,
                quota_4h_pct: q_4h.unwrap_or(100.0),
                quota_weekly_pct: q_wk.unwrap_or(100.0),
                active_model: target_model.clone(),
                status_label: if running {
                    "running".to_string()
                } else {
                    "idle".to_string()
                },
            });
        }
    }

    // Include configured remote SSH nodes
    let remote_conns: Vec<SshConnectionRecord> = ssh_conns
        .into_iter()
        .filter(|conn| {
            (local_ip.is_empty() || conn.ip_address.trim() != local_ip.trim())
                && !conn.alias.eq_ignore_ascii_case(&effective_local_alias)
                && !conn.alias.eq_ignore_ascii_case(&local_machine)
        })
        .collect();

    let remote_items: Vec<Vec<ClusterInstanceBalanceItem>> = std::thread::scope(|s| {
        let handles: Vec<_> = remote_conns
            .iter()
            .map(|conn| {
                let target_model = target_model.clone();
                s.spawn(move || {
                    let mut node_items = Vec::new();
                    let port = if conn.port == 0 { 22 } else { conn.port };
                    let start = Instant::now();
                    let reachable = check_tcp_liveness(&conn.ip_address, port, 1500);
                    let elapsed_ms = start.elapsed().as_millis() as u64;

                    if !reachable {
                        node_items.push(ClusterInstanceBalanceItem {
                            machine_alias: conn.alias.clone(),
                            machine_ip: conn.ip_address.clone(),
                            ip_address: Some(conn.ip_address.clone()),
                            error_message: Some(format!(
                                "TCP port {} unreachable on {} (timeout 1500ms)",
                                port, conn.ip_address
                            )),
                            latency_ms: Some(elapsed_ms),
                            is_local: false,
                            instance_id: "default".to_string(),
                            instance_name: "default".to_string(),
                            running: false,
                            bound_email: String::new(),
                            tier: "SSH-NODE".to_string(),
                            quota_4h_pct: 0.0,
                            quota_weekly_pct: 0.0,
                            active_model: target_model.clone(),
                            status_label: "offline".to_string(),
                        });
                        return node_items;
                    }

                    let is_windows = conn.os.to_lowercase().contains("win")
                        || conn.username.eq_ignore_ascii_case("administrator")
                        || conn.alias.to_lowercase().starts_with('w');
                    let agm_bin = if is_windows { "agm.exe" } else { "agm" };

                    // Node reachable: query real instances via SSH with gitmap fallback
                    let remote_cmd = vec![
                        agm_bin.to_string(),
                        "instances".to_string(),
                        "list".to_string(),
                        "--json".to_string(),
                    ];
                    let mut raw_stdout = String::new();
                    if let Ok(res_list) =
                        exec_ssh_command(&conn.alias, &remote_cmd, Some(port), None, None)
                    {
                        if let Some(res) = res_list
                            .first()
                            .filter(|r| r.success && !r.stdout.is_empty())
                        {
                            raw_stdout = res.stdout.clone();
                        }
                    }
                    if raw_stdout.is_empty() {
                        let candidate_targets = if conn.alias.eq_ignore_ascii_case(&conn.ip_address)
                        {
                            vec![conn.alias.as_str()]
                        } else {
                            vec![conn.alias.as_str(), conn.ip_address.as_str()]
                        };
                        for target_id in candidate_targets {
                            if let Ok((code, gm_out, _)) =
                                exec_via_gitmap_ssh(target_id, &format!("{agm_bin} instances list --json"))
                            {
                                if code == 0 && !gm_out.is_empty() {
                                    raw_stdout = gm_out;
                                    break;
                                }
                            }
                        }
                    }

                    // Direct file inspection fallback over SSH if CLI failed or returned empty
                    if raw_stdout.is_empty() || extract_clean_json(&raw_stdout).is_none() {
                        let file_inst_cmd = if is_windows {
                            vec![
                                "powershell".to_string(),
                                "-NoProfile".to_string(),
                                "-NonInteractive".to_string(),
                                "-Command".to_string(),
                                "\"if (Test-Path $HOME\\.antigravity_tools\\instances\\instances.json) { Get-Content -Raw $HOME\\.antigravity_tools\\instances\\instances.json }\"".to_string(),
                            ]
                        } else {
                            vec![
                                "cat".to_string(),
                                "~/.antigravity_tools/instances/instances.json".to_string(),
                            ]
                        };
                        if let Ok(res_list) =
                            exec_ssh_command(&conn.alias, &file_inst_cmd, Some(port), None, None)
                        {
                            if let Some(res) = res_list
                                .first()
                                .filter(|r| r.success && !r.stdout.is_empty())
                            {
                                raw_stdout = res.stdout.clone();
                            }
                        }
                        if raw_stdout.is_empty() || extract_clean_json(&raw_stdout).is_none() {
                            let candidate_targets = if conn.alias.eq_ignore_ascii_case(&conn.ip_address)
                            {
                                vec![conn.alias.as_str()]
                            } else {
                                vec![conn.alias.as_str(), conn.ip_address.as_str()]
                            };
                            let file_inst_str = if is_windows {
                                "powershell -NoProfile -NonInteractive -Command \"if (Test-Path $HOME\\.antigravity_tools\\instances\\instances.json) { Get-Content -Raw $HOME\\.antigravity_tools\\instances\\instances.json }\""
                            } else {
                                "cat ~/.antigravity_tools/instances/instances.json"
                            };
                            for target_id in candidate_targets {
                                if let Ok((code, gm_out, _)) = exec_via_gitmap_ssh(target_id, file_inst_str) {
                                    if code == 0 && !gm_out.is_empty() {
                                        raw_stdout = gm_out;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Query remote accounts to extract real quota balances
                    let mut remote_accounts_json = String::new();
                    let acc_cmd = vec![
                        agm_bin.to_string(),
                        "accounts".to_string(),
                        "list".to_string(),
                        "--json".to_string(),
                    ];
                    if let Ok(res_list) =
                        exec_ssh_command(&conn.alias, &acc_cmd, Some(port), None, None)
                    {
                        if let Some(res) = res_list
                            .first()
                            .filter(|r| r.success && !r.stdout.is_empty())
                        {
                            remote_accounts_json = res.stdout.clone();
                        }
                    }
                    if remote_accounts_json.is_empty() {
                        let candidate_targets = if conn.alias.eq_ignore_ascii_case(&conn.ip_address)
                        {
                            vec![conn.alias.as_str()]
                        } else {
                            vec![conn.alias.as_str(), conn.ip_address.as_str()]
                        };
                        for target_id in candidate_targets {
                            if let Ok((code, gm_acc_out, _)) =
                                exec_via_gitmap_ssh(target_id, &format!("{agm_bin} accounts list --json"))
                            {
                                if code == 0 && !gm_acc_out.is_empty() {
                                    remote_accounts_json = gm_acc_out;
                                    break;
                                }
                            }
                        }
                    }

                    // Direct file inspection fallback over SSH for accounts if CLI failed or returned empty
                    if remote_accounts_json.is_empty() || extract_clean_json(&remote_accounts_json).is_none() {
                        let file_acc_cmd = if is_windows {
                            vec![
                                "powershell".to_string(),
                                "-NoProfile".to_string(),
                                "-NonInteractive".to_string(),
                                "-Command".to_string(),
                                "\"if (Test-Path $HOME\\.antigravity_tools\\accounts.json) { Get-Content -Raw $HOME\\.antigravity_tools\\accounts.json }\"".to_string(),
                            ]
                        } else {
                            vec![
                                "cat".to_string(),
                                "~/.antigravity_tools/accounts.json".to_string(),
                            ]
                        };
                        if let Ok(res_list) =
                            exec_ssh_command(&conn.alias, &file_acc_cmd, Some(port), None, None)
                        {
                            if let Some(res) = res_list
                                .first()
                                .filter(|r| r.success && !r.stdout.is_empty())
                            {
                                remote_accounts_json = res.stdout.clone();
                            }
                        }
                        if remote_accounts_json.is_empty() || extract_clean_json(&remote_accounts_json).is_none() {
                            let candidate_targets = if conn.alias.eq_ignore_ascii_case(&conn.ip_address)
                            {
                                vec![conn.alias.as_str()]
                            } else {
                                vec![conn.alias.as_str(), conn.ip_address.as_str()]
                            };
                            let file_acc_str = if is_windows {
                                "powershell -NoProfile -NonInteractive -Command \"if (Test-Path $HOME\\.antigravity_tools\\accounts.json) { Get-Content -Raw $HOME\\.antigravity_tools\\accounts.json }\""
                            } else {
                                "cat ~/.antigravity_tools/accounts.json"
                            };
                            for target_id in candidate_targets {
                                if let Ok((code, gm_out, _)) = exec_via_gitmap_ssh(target_id, file_acc_str) {
                                    if code == 0 && !gm_out.is_empty() {
                                        remote_accounts_json = gm_out;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Parse remote accounts
                    let clean_accs_str = extract_clean_json(&remote_accounts_json);
                    let remote_accs_val = clean_accs_str
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
                    let curr_account_id = remote_accs_val
                        .as_ref()
                        .and_then(|v| v.get("current_account_id"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let remote_accs_array = remote_accs_val
                        .as_ref()
                        .and_then(|v| {
                            v.get("accounts")
                                .and_then(|a| a.as_array())
                                .or_else(|| v.get("data").and_then(|d| d.as_array()))
                                .or_else(|| v.as_array())
                        });

                    if let Some(clean_inst_str) = extract_clean_json(&raw_stdout) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(clean_inst_str) {
                            let raw_instances = val
                                .get("instances")
                                .and_then(|d| d.as_array())
                                .or_else(|| val.get("data").and_then(|d| d.as_array()))
                                .or_else(|| val.as_array());
                            if let Some(inst_array) = raw_instances {
                                for inst_val in inst_array {
                                    let inst_id = inst_val
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("default")
                                        .to_string();
                                    let inst_name = inst_val
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(&inst_id)
                                        .to_string();
                                    let is_running = inst_val
                                        .get("is_running")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    let is_default = inst_val
                                        .get("is_default")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(inst_id == "default");
                                    let raw_email = inst_val
                                        .get("bound_email")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let mut clean_email = if raw_email.is_empty()
                                        || raw_email.contains("Administrator")
                                        || raw_email.contains("root")
                                        || raw_email.contains("${adminUser}")
                                        || raw_email.contains("${linuxUser}")
                                    {
                                        String::new()
                                    } else {
                                        raw_email
                                    };

                                    // Fallback to accounts.json.current_account_id when bound_email is empty for default instance
                                    if clean_email.is_empty() && is_default {
                                        if let Some(ref curr_id) = curr_account_id {
                                            if let Some(acc_array) = remote_accs_array {
                                                for acc in acc_array {
                                                    let a_id = acc
                                                        .get("id")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("");
                                                    if a_id == curr_id.as_str() {
                                                        let a_email = acc
                                                            .get("email")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("");
                                                        if !a_email.is_empty() {
                                                            clean_email = a_email.to_string();
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    let final_bound_email = if clean_email.is_empty() {
                                        "(unbound)".to_string()
                                    } else {
                                        clean_email.clone()
                                    };

                                    let mut tier = "PRO".to_string();
                                    let mut q_4h = 0.0;
                                    let mut q_wk = 0.0;
                                    let mut has_verified_quota = false;

                                    if let Some(acc_array) = remote_accs_array {
                                        for acc_val in acc_array {
                                            let acc_email = acc_val
                                                .get("email")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let acc_id = acc_val
                                                .get("id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let is_match = (!clean_email.is_empty()
                                                && acc_email.eq_ignore_ascii_case(&clean_email))
                                                || (clean_email.is_empty()
                                                    && is_default
                                                    && curr_account_id.as_deref() == Some(acc_id));

                                            if is_match {
                                                if let Ok(acc) = serde_json::from_value::<
                                                    crate::models::Account,
                                                >(
                                                    acc_val.clone()
                                                ) {
                                                    tier = acc
                                                        .quota
                                                        .as_ref()
                                                        .and_then(|q| q.subscription_tier.as_deref())
                                                        .unwrap_or("PRO")
                                                        .to_string();
                                                    let (pct4_opt, pctw_opt) =
                                                        crate::modules::auto_switcher::extract_dual_window_quotas(
                                                            &acc,
                                                            &target_model,
                                                        );
                                                    if let Some(p4) = pct4_opt {
                                                        q_4h = p4;
                                                        has_verified_quota = true;
                                                    }
                                                    if let Some(pw) = pctw_opt {
                                                        q_wk = pw;
                                                        has_verified_quota = true;
                                                    }
                                                } else if let Some(t) = acc_val
                                                    .get("quota")
                                                    .and_then(|q| q.get("subscription_tier"))
                                                    .and_then(|v| v.as_str())
                                                {
                                                    tier = t.to_string();
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    let status_label = if is_running {
                                        if has_verified_quota {
                                            "running".to_string()
                                        } else {
                                            "running (unverified-quota)".to_string()
                                        }
                                    } else {
                                        "idle".to_string()
                                    };

                                    node_items.push(ClusterInstanceBalanceItem {
                                        machine_alias: conn.alias.clone(),
                                        machine_ip: conn.ip_address.clone(),
                                        ip_address: Some(conn.ip_address.clone()),
                                        error_message: None,
                                        latency_ms: Some(elapsed_ms),
                                        is_local: false,
                                        instance_id: inst_id,
                                        instance_name: inst_name,
                                        running: is_running,
                                        bound_email: final_bound_email,
                                        tier,
                                        quota_4h_pct: q_4h,
                                        quota_weekly_pct: q_wk,
                                        active_model: target_model.clone(),
                                        status_label,
                                    });
                                }
                            }
                        }
                    }

                    if node_items.is_empty() {
                        let mut def_email = "(unbound)".to_string();
                        let mut def_tier = "SSH-NODE".to_string();
                        let mut def_q_4h = 0.0;
                        let mut def_q_wk = 0.0;
                        let mut def_has_quota = false;

                        if let Some(acc_array) = remote_accs_array {
                            let matched_def_acc = if let Some(ref curr_id) = curr_account_id {
                                acc_array
                                    .iter()
                                    .find(|a| a.get("id").and_then(|v| v.as_str()) == Some(curr_id.as_str()))
                                    .or_else(|| acc_array.first())
                            } else {
                                acc_array.first()
                            };

                            if let Some(acc_val) = matched_def_acc {
                                let a_email = acc_val
                                    .get("email")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if !a_email.is_empty() {
                                    def_email = a_email.to_string();
                                }
                                if let Ok(acc) = serde_json::from_value::<crate::models::Account>(
                                    acc_val.clone(),
                                ) {
                                    def_tier = acc
                                        .quota
                                        .as_ref()
                                        .and_then(|q| q.subscription_tier.as_deref())
                                        .unwrap_or("PRO")
                                        .to_string();
                                    let (pct4_opt, pctw_opt) =
                                        crate::modules::auto_switcher::extract_dual_window_quotas(
                                            &acc,
                                            &target_model,
                                        );
                                    if let Some(p4) = pct4_opt {
                                        def_q_4h = p4;
                                        def_has_quota = true;
                                    }
                                    if let Some(pw) = pctw_opt {
                                        def_q_wk = pw;
                                        def_has_quota = true;
                                    }
                                }
                            }
                        }

                        let status_label = if def_has_quota {
                            "idle".to_string()
                        } else {
                            "unverified-balance".to_string()
                        };

                        node_items.push(ClusterInstanceBalanceItem {
                            machine_alias: conn.alias.clone(),
                            machine_ip: conn.ip_address.clone(),
                            ip_address: Some(conn.ip_address.clone()),
                            error_message: None,
                            latency_ms: Some(elapsed_ms),
                            is_local: false,
                            instance_id: "default".to_string(),
                            instance_name: "default".to_string(),
                            running: false,
                            bound_email: def_email,
                            tier: def_tier,
                            quota_4h_pct: def_q_4h,
                            quota_weekly_pct: def_q_wk,
                            active_model: target_model.clone(),
                            status_label,
                        });
                    }

                    node_items
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    for node_group in remote_items {
        items.extend(node_group);
    }

    if let Ok(mut cache) = CLUSTER_BALANCES_CACHE.write() {
        *cache = Some((items.clone(), Instant::now()));
    }

    Ok(items)
}

/// Dispatch a prompt to a local or remote SSH machine & instance
pub async fn send_remote_ssh_prompt(
    payload: &RemotePromptDispatchPayload,
) -> Result<RemotePromptDispatchResult, String> {
    let raw_node = payload.node_alias.trim();
    let clean_node = raw_node
        .strip_prefix("ssh_")
        .unwrap_or(raw_node)
        .trim_matches(|c| c == '\'' || c == '"' || c == '`');
    let node_alias = if clean_node.is_empty() {
        "local".to_string()
    } else {
        clean_node.to_string()
    };

    let raw_inst = payload.instance_name.trim();
    let stripped_inst = if let Some((_, last)) = raw_inst.rsplit_once(':') {
        last.trim()
    } else {
        raw_inst
    };
    let clean_inst = stripped_inst
        .trim_matches(|c| c == '\'' || c == '"' || c == '`')
        .replace([';', '&', '|', '`', '$', '\n', '\r'], "")
        .trim()
        .to_string();
    let inst_name = if clean_inst.is_empty() || clean_inst.eq_ignore_ascii_case("(unbound)") {
        "default".to_string()
    } else {
        clean_inst
    };

    let text = payload.prompt_text.trim();
    if text.is_empty() {
        return Err("Prompt text cannot be empty".to_string());
    }

    let raw_target = payload
        .conversation_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .trim_matches(|c| c == '\'' || c == '"' || c == '`')
        .replace([';', '&', '|', '`', '$', '\n', '\r'], "")
        .trim()
        .to_string();
    let target_proj = if raw_target.is_empty() {
        "default".to_string()
    } else {
        raw_target
    };

    let local_machine = crate::modules::email_watcher::detect_machine_name();
    let local_ip = crate::modules::email_watcher::detect_local_ip();
    let ssh_conns = load_ssh_connections().unwrap_or_default();
    let matched_local_alias = ssh_conns
        .iter()
        .find(|conn| !local_ip.is_empty() && conn.ip_address.trim() == local_ip.trim())
        .map(|conn| conn.alias.clone());
    let is_local = node_alias.eq_ignore_ascii_case("local")
        || node_alias.eq_ignore_ascii_case("localhost")
        || node_alias == "127.0.0.1"
        || node_alias.eq_ignore_ascii_case("w1")
        || node_alias == "192.168.1.3"
        || node_alias.eq_ignore_ascii_case(&local_machine)
        || (!local_ip.is_empty() && node_alias.eq_ignore_ascii_case(&local_ip))
        || matched_local_alias
            .as_deref()
            .map(|a| node_alias.eq_ignore_ascii_case(a))
            .unwrap_or(false);

    let cmd_str = if is_local {
        format!("{} --instance {} {}", target_proj, inst_name, text)
    } else {
        format!(
            "{} --node {} --instance {} {}",
            target_proj, node_alias, inst_name, text
        )
    };

    let start = Instant::now();
    let reply = crate::modules::telegram_inbound::execute_prompt_injection(&cmd_str).await;
    let duration_ms = start.elapsed().as_millis();
    let is_error_reply = reply.contains("❌") || reply.contains("⚠️ <b>Missing");
    let is_send_success = !is_error_reply;
    let clean_msg = crate::modules::telegram_inbound::strip_telegram_html_tags(&reply);
    let (stdout, stderr) = if is_send_success {
        (clean_msg.clone(), String::new())
    } else {
        (String::new(), clean_msg.clone())
    };

    Ok(RemotePromptDispatchResult {
        success: is_send_success,
        node_alias,
        instance_name: inst_name,
        dispatch_route: if is_local {
            "local-instance".to_string()
        } else {
            "remote-ssh-gitmap".to_string()
        },
        message: clean_msg,
        command_executed: cmd_str,
        stdout,
        stderr,
        duration_ms,
    })
}
