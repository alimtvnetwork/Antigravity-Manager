use crate::error::AppError;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

use super::*;

/// Helper to construct a FleetMachineInfo from a registered SshConnectionRecord
pub(crate) fn build_ssh_mesh_machine_info(
    conn: &crate::modules::ssh_manager::SshConnectionRecord,
    now: i64,
) -> FleetMachineInfo {
    let port = if conn.port == 0 { 22 } else { conn.port };
    let mut is_online =
        crate::modules::ssh_manager::check_tcp_liveness(&conn.ip_address, port, 1500);
    if !is_online && crate::modules::ssh_manager::resolve_gitmap_bin().is_some() {
        if let Ok((is_gm_online, _)) =
            crate::modules::ssh_manager::check_node_via_gitmap(&conn.alias)
        {
            if is_gm_online {
                is_online = true;
            }
        }
    }
    let status = if is_online {
        "online".to_string()
    } else {
        "offline".to_string()
    };
    FleetMachineInfo {
        node_id: format!("ssh_{}", conn.alias),
        node_alias: conn.alias.clone(),
        alias: conn.alias.clone(),
        os_info: if conn.os.trim().is_empty() {
            None
        } else {
            Some(conn.os.clone())
        },
        ip_address: conn.ip_address.clone(),
        is_online,
        last_heartbeat_timestamp: if is_online { now } else { 0 },
        last_heartbeat_at: if is_online { now } else { 0 },
        uptime_seconds: 0,
        status,
        is_local: false,
        source: "ssh-mesh".to_string(),
        in_flight_prompts_count: 0,
        total_running_prompts: 0,
        total_instances: 1,
        active_instances: vec![FleetInstanceSummary {
            profile_id: format!("{}:default", conn.alias),
            profile_name: format!("{}:default", conn.alias),
            is_running: is_online,
            bound_account_id: None,
            bound_account_email: None,
        }],
        instances: Vec::new(),
        bound_emails: Vec::new(),
        active_accounts: Vec::new(),
        leases: Vec::new(),
    }
}

/// Collect all known peer connections from SSH connections, candidate gitmap-ssh-nodes.json paths, and GitMap cluster nodes
pub(crate) fn collect_all_peer_connections() -> Vec<crate::modules::ssh_manager::SshConnectionRecord>
{
    let mut all_conns = Vec::new();
    let mut seen_aliases = HashSet::new();
    let mut seen_ips = HashSet::new();

    // 1. Load from ssh_manager registry
    if let Ok(conns) = crate::modules::ssh_manager::load_ssh_connections() {
        for c in conns {
            let a_norm = c.alias.trim().to_ascii_lowercase();
            let ip_norm = c.ip_address.trim().to_string();
            if !a_norm.is_empty() && !seen_aliases.contains(&a_norm) {
                seen_aliases.insert(a_norm);
                if !ip_norm.is_empty() {
                    seen_ips.insert(ip_norm);
                }
                all_conns.push(c);
            }
        }
    }

    // 2. Load from candidate gitmap-ssh-nodes.json files
    for seed_path in crate::modules::ssh_manager::candidate_seed_node_paths() {
        if let Ok(raw) = std::fs::read_to_string(&seed_path) {
            if let Ok(conns) = crate::modules::ssh_manager::decode_connections_from_json(&raw) {
                for c in conns {
                    let a_norm = c.alias.trim().to_ascii_lowercase();
                    let ip_norm = c.ip_address.trim().to_string();
                    if !a_norm.is_empty() && !seen_aliases.contains(&a_norm) {
                        seen_aliases.insert(a_norm);
                        if !ip_norm.is_empty() {
                            seen_ips.insert(ip_norm);
                        }
                        all_conns.push(c);
                    }
                }
            }
        }
    }

    // 3. Load from GitMap cluster nodes CLI (gitmap cluster nodes --json / status)
    for gm_node in crate::modules::telegram_inbound::cluster::query_gitmap_cluster_nodes() {
        let a_norm = gm_node.alias.trim().to_ascii_lowercase();
        let ip_norm = gm_node.ip_address.trim().to_string();
        if !a_norm.is_empty() && !seen_aliases.contains(&a_norm) {
            seen_aliases.insert(a_norm);
            if !ip_norm.is_empty() && ip_norm != "Cluster Mesh" {
                seen_ips.insert(ip_norm.clone());
            }
            all_conns.push(crate::modules::ssh_manager::SshConnectionRecord {
                alias: gm_node.alias.clone(),
                ip_address: gm_node.ip_address.clone(),
                port: 22,
                username: "Administrator".to_string(),
                key_path: String::new(),
                os: "Windows".to_string(),
                encrypted_password: String::new(),
                encryption_algo: "RSA-2048".to_string(),
                os_group: None,
                os_version: None,
                build_version: None,
                first_run_at: None,
                created_at: None,
            });
        }
    }

    // 4. Canonical fallback: ensure all 6 GitMap cluster nodes (w1, main, u1, w2, w3, w4) are present
    for &(alias, ip, user, os) in crate::modules::ssh_manager::GITMAP_CLUSTER_NODES {
        let a_norm = alias.trim().to_ascii_lowercase();
        let ip_norm = ip.trim().to_string();
        if !a_norm.is_empty() && !seen_aliases.contains(&a_norm) && !seen_ips.contains(&ip_norm) {
            seen_aliases.insert(a_norm);
            seen_ips.insert(ip_norm.clone());
            all_conns.push(crate::modules::ssh_manager::SshConnectionRecord {
                alias: alias.to_string(),
                ip_address: ip.to_string(),
                port: 22,
                username: user.to_string(),
                key_path: String::new(),
                os: os.to_string(),
                encrypted_password: String::new(),
                encryption_algo: "RSA-2048".to_string(),
                os_group: Some(if os.eq_ignore_ascii_case("windows") {
                    "Windows".to_string()
                } else {
                    "Linux".to_string()
                }),
                os_version: None,
                build_version: None,
                first_run_at: None,
                created_at: None,
            });
        }
    }

    all_conns
}

/// Build local machine info plus all registered SSH fleet and GitMap nodes
pub fn build_local_and_ssh_fleet() -> Vec<FleetMachineInfo> {
    let mut machines = build_local_fallback_machine();
    let local_ip = machines
        .first()
        .map(|m| m.ip_address.clone())
        .unwrap_or_default();
    let local_alias = machines
        .first()
        .map(|m| m.alias.clone())
        .unwrap_or_default();
    let now = Utc::now().timestamp();

    let all_conns = collect_all_peer_connections();
    for conn in all_conns {
        if conn.ip_address == local_ip || conn.alias.eq_ignore_ascii_case(&local_alias) {
            if let Some(local_machine) = machines.first_mut() {
                local_machine.node_alias = conn.alias.clone();
                local_machine.alias = conn.alias;
            }
            continue;
        }
        machines.push(build_ssh_mesh_machine_info(&conn, now));
    }

    machines.sort_by(|a, b| {
        if a.is_local != b.is_local {
            return b.is_local.cmp(&a.is_local);
        }
        if a.is_online != b.is_online {
            return b.is_online.cmp(&a.is_online);
        }
        a.node_alias
            .to_lowercase()
            .cmp(&b.node_alias.to_lowercase())
    });

    machines
}

/// Query fleet machines aggregating nodes, instance profiles, and workspace leases across the cluster
pub async fn query_fleet_machines() -> Result<Vec<FleetMachineInfo>, AppError> {
    if is_supabase_circuit_open() {
        tracing::debug!(
            target: "supabase_sync",
            remaining_secs = get_supabase_cooldown_remaining_secs(),
            "Supabase circuit breaker is open; returning local and SSH fleet without network requests"
        );
        return Ok(build_local_and_ssh_fleet());
    }

    let config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                target: "supabase_sync",
                "Failed to load config for query_fleet_machines: {e}"
            );
            return Ok(build_local_and_ssh_fleet());
        }
    };
    if !config.is_sync_enabled {
        return Ok(build_local_and_ssh_fleet());
    }

    let root_ep = match config
        .endpoints
        .iter()
        .find(|ep| ep.is_enabled && ep.role == "root")
    {
        Some(ep) => ep.clone(),
        None => return Ok(build_local_and_ssh_fleet()),
    };

    let client = match SupabaseClient::new(&root_ep) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "[Fleet] Failed to create Supabase client for fleet query: {}",
                e
            );
            record_supabase_network_error();
            return Ok(build_local_and_ssh_fleet());
        }
    };

    let nodes_val = match client
        .select("nodes", "select=*&order=last_heartbeat_at.desc")
        .await
    {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!("[Fleet] Failed to query nodes from Supabase: {}", e);
            record_supabase_network_error();
            return Ok(build_local_and_ssh_fleet());
        }
    };

    if let Some(msg) = postgrest_error_message(&nodes_val) {
        tracing::warn!("[Fleet] PostgREST error querying nodes: {}", msg);
        record_supabase_network_error();
        return Ok(build_local_and_ssh_fleet());
    }

    let nodes_array = match nodes_val.as_array() {
        Some(arr) => arr.clone(),
        None => return Ok(build_local_and_ssh_fleet()),
    };

    let profiles_val = match client.select("instance_profiles", "select=*").await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!(
                target: "supabase_sync",
                endpoint = %root_ep.url,
                error = %e,
                "Failed to query instance_profiles table from Supabase"
            );
            record_supabase_network_error();
            serde_json::Value::Array(Vec::new())
        }
    };

    let mut profiles_by_node: HashMap<String, Vec<FleetInstanceSummary>> = HashMap::new();
    if let Some(prof_array) = profiles_val.as_array() {
        for p in prof_array {
            let node_id = p
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if node_id.is_empty() {
                tracing::debug!(
                    target: "supabase_sync",
                    "Skipping instance profile row with empty node_id"
                );
                continue;
            }
            let profile_id = p
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let profile_name = p
                .get("profile_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let is_active = p
                .get("is_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let status_str = p.get("status").and_then(|v| v.as_str()).unwrap_or_default();
            let is_running = is_active || status_str == "running";
            let bound_account_id = p
                .get("active_account_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let bound_account_email = p
                .get("active_account_email")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            profiles_by_node
                .entry(node_id)
                .or_default()
                .push(FleetInstanceSummary {
                    profile_id,
                    profile_name,
                    is_running,
                    bound_account_id,
                    bound_account_email,
                });
        }
    }

    let leases_val = match client.select("workspace_leases", "select=*").await {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!(
                target: "supabase_sync",
                endpoint = %root_ep.url,
                error = %e,
                "Failed to query workspace_leases table from Supabase"
            );
            serde_json::Value::Array(Vec::new())
        }
    };

    let now = Utc::now().timestamp();
    let mut leases_by_node: HashMap<String, Vec<FleetLeaseInfo>> = HashMap::new();
    if let Some(leases_array) = leases_val.as_array() {
        for l in leases_array {
            let node_id = l
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if node_id.is_empty() {
                tracing::debug!(
                    target: "supabase_sync",
                    "Skipping lease row with empty node_id"
                );
                continue;
            }
            let account_id = l
                .get("account_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let account_email = l
                .get("account_email")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let profile_name = l
                .get("profile_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let leased_at = l.get("leased_at").and_then(|v| v.as_i64()).unwrap_or(0);
            let expires_at = l.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0);
            let is_expired = expires_at < now;

            leases_by_node
                .entry(node_id)
                .or_default()
                .push(FleetLeaseInfo {
                    account_id,
                    account_email,
                    profile_name,
                    leased_at,
                    expires_at,
                    is_expired,
                });
        }
    }

    let local_node_id = get_local_node_id();
    let mut machines = Vec::new();
    let mut has_local = false;

    for n in &nodes_array {
        let node_id = n
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if node_id.is_empty() {
            tracing::warn!(
                target: "supabase_sync",
                row = ?n,
                "Skipping node row with missing or empty id"
            );
            continue;
        }
        let is_local = node_id == local_node_id;
        if is_local {
            has_local = true;
        }
        let ip_address = n
            .get("ip_address")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let mut node_alias = n
            .get("alias")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if (is_local || ip_address == get_local_ip())
            && (node_alias.is_empty()
                || node_alias == "agm-node-local"
                || node_alias.starts_with("Node-"))
        {
            node_alias = resolve_local_node_alias();
        }
        let uptime_seconds = if is_local {
            get_uptime_seconds()
        } else {
            n.get("uptime_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        };
        let last_heartbeat_timestamp = if is_local {
            now
        } else {
            n.get("last_heartbeat_at")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
        };
        let is_online = if is_local {
            true
        } else {
            (now - last_heartbeat_timestamp).abs() < 180
        };
        let os_info = n
            .get("os_info")
            .or_else(|| n.get("os"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                if is_local {
                    Some(std::env::consts::OS.to_string())
                } else {
                    None
                }
            });

        let in_flight_prompts_count = if is_local {
            let mut running_cnt = 0;
            if let Ok(conn) = crate::modules::repo_db::connect_db() {
                if let Ok(mut stmt) =
                    conn.prepare("SELECT count(*) FROM active_prompts WHERE status = 'running'")
                {
                    if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                        running_cnt = cnt;
                    }
                }
            }
            running_cnt
        } else {
            n.get("running_prompts_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize
        };

        let mut active_instances = profiles_by_node.remove(&node_id).unwrap_or_default();
        if is_local && active_instances.is_empty() {
            if let Ok(reg) = crate::modules::instance::load_registry() {
                for inst in reg.instances {
                    let is_running = crate::modules::instance::is_instance_running(
                        &inst.id,
                        &inst.data_dir,
                        inst.pid,
                    );
                    active_instances.push(FleetInstanceSummary {
                        profile_id: format!("{}_{}", local_node_id, inst.id),
                        profile_name: inst.name,
                        is_running,
                        bound_account_id: inst.bound_account_id,
                        bound_account_email: inst.bound_email,
                    });
                }
            }
        }

        let leases = leases_by_node.remove(&node_id).unwrap_or_default();

        let mut email_set = HashSet::new();
        for inst in &active_instances {
            if let Some(ref em) = inst.bound_account_email {
                let trimmed = em.trim();
                if !trimmed.is_empty() {
                    email_set.insert(trimmed.to_string());
                }
            }
        }
        for l in &leases {
            let trimmed = l.account_email.trim();
            if !trimmed.is_empty() {
                email_set.insert(trimmed.to_string());
            }
        }
        let mut bound_emails: Vec<String> = email_set.into_iter().collect();
        bound_emails.sort();

        let status = if is_online {
            "online".to_string()
        } else {
            "offline".to_string()
        };
        machines.push(FleetMachineInfo {
            node_id,
            node_alias: node_alias.clone(),
            alias: node_alias,
            os_info,
            ip_address,
            is_online,
            last_heartbeat_timestamp,
            last_heartbeat_at: last_heartbeat_timestamp,
            uptime_seconds,
            status,
            source: "supabase".to_string(),
            in_flight_prompts_count,
            total_running_prompts: in_flight_prompts_count,
            total_instances: active_instances.len(),
            active_instances,
            bound_emails: bound_emails.clone(),
            active_accounts: bound_emails,
            leases,
            is_local,
            ..Default::default()
        });
    }

    if !has_local {
        let fallback = build_local_fallback_machine();
        if let Some(local_machine) = fallback.into_iter().next() {
            machines.push(local_machine);
        }
    }

    // Merge registered SSH mesh nodes and GitMap peer nodes that are not present in Supabase
    let now = Utc::now().timestamp();
    let all_conns = collect_all_peer_connections();
    for conn in all_conns {
        let is_dup = machines.iter().any(|m| {
            m.ip_address == conn.ip_address
                || m.node_alias.eq_ignore_ascii_case(&conn.alias)
                || m.alias.eq_ignore_ascii_case(&conn.alias)
        });
        if !is_dup {
            machines.push(build_ssh_mesh_machine_info(&conn, now));
        }
    }

    machines.sort_by(|a, b| {
        if a.is_local != b.is_local {
            return b.is_local.cmp(&a.is_local);
        }
        if a.is_online != b.is_online {
            return b.is_online.cmp(&a.is_online);
        }
        a.node_alias
            .to_lowercase()
            .cmp(&b.node_alias.to_lowercase())
    });

    Ok(machines)
}
