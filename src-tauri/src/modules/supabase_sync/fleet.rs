use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::*;

pub(crate) fn construct_local_machine_info(
    config: &SupabaseConfig,
    leases: &[PostgrestLeaseRow],
) -> FleetMachineInfo {
    let local_node_id = get_local_node_id();
    let local_ip = get_local_ip();
    let uptime = get_uptime_seconds();
    let now = Utc::now().timestamp();
    let alias = if config.node_alias.is_empty() {
        let short_id = if local_node_id.len() > 6 {
            &local_node_id[..6]
        } else {
            &local_node_id
        };
        format!("Node-{}", short_id)
    } else {
        config.node_alias.clone()
    };

    let mut local_instances = Vec::new();
    let mut total_running = 0;
    let mut active_accounts_set = HashSet::new();

    if let Ok(inst_statuses) = instance::list_instances() {
        for status in inst_statuses {
            let inst = status.config;
            let prompts =
                crate::modules::repo_db::discover_running_prompts_from_antigravity(&inst.id);
            let running_count = prompts
                .iter()
                .filter(|p| p.status == "running" || p.status == "executing")
                .count();
            total_running += running_count;

            let email = inst.bound_email.clone().unwrap_or_default();
            let acc_id = inst.bound_account_id.clone().unwrap_or_default();
            if email.len() > 0 {
                active_accounts_set.insert(email.clone());
            } else if acc_id.len() > 0 {
                active_accounts_set.insert(acc_id.clone());
            }

            let is_active = status.is_running || inst.is_default;
            let inst_status_str = if is_active {
                "running".to_string()
            } else {
                "idle".to_string()
            };

            let mut is_leased = false;
            let mut lease_expires_at = None;
            for l in leases {
                let is_active_lease = l.expires_at > now;
                let has_bound_account = acc_id.len() > 0;
                let has_bound_email = email.len() > 0;
                let is_matching_account = (has_bound_account && l.account_id == acc_id)
                    || (has_bound_email && l.account_email == email);
                let is_matching_node = l.node_id == local_node_id;
                let has_inst_name = inst.name.len() > 0;
                let is_matching_profile = has_inst_name && l.profile_name == inst.name;

                if is_active_lease
                    && (is_matching_account || (is_matching_node && is_matching_profile))
                {
                    is_leased = true;
                    lease_expires_at = Some(l.expires_at);
                    break;
                }
            }

            local_instances.push(FleetInstanceItem {
                instance_id: inst.id,
                profile_name: inst.name,
                bound_account_id: acc_id,
                bound_account_email: email,
                is_active,
                status: inst_status_str,
                running_prompts_count: running_count,
                updated_at: now,
                is_leased,
                lease_expires_at,
            });
        }
    }

    let mut active_accounts: Vec<String> = active_accounts_set.into_iter().collect();
    active_accounts.sort();

    let total_instances = local_instances.len();

    FleetMachineInfo {
        node_id: local_node_id,
        node_alias: alias.clone(),
        alias,
        ip_address: local_ip,
        uptime_seconds: uptime,
        last_heartbeat_at: now,
        last_heartbeat_timestamp: now,
        status: "online".to_string(),
        is_online: true,
        is_local: true,
        source: "local".to_string(),
        total_instances,
        total_running_prompts: total_running,
        in_flight_prompts_count: total_running,
        active_accounts: active_accounts.clone(),
        bound_emails: active_accounts,
        instances: local_instances,
        ..Default::default()
    }
}

/// Fetch list of fleet machines with child instance profiles and active leases from Supabase.
/// Falls back to returning local machine info if Supabase is offline or unconfigured.
pub async fn fetch_fleet_machines() -> crate::error::AppResult<Vec<FleetMachineInfo>> {
    let config = load_config().unwrap_or_default();
    let local_node_id = get_local_node_id();
    let now = Utc::now().timestamp();

    let root_ep = config
        .endpoints
        .iter()
        .find(|ep| ep.is_enabled && ep.role == "root")
        .cloned();

    let client = match root_ep {
        Some(ref ep) => match SupabaseClient::new(ep) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize Supabase client for fleet query: {}",
                    e
                );
                None
            }
        },
        None => None,
    };

    let mut raw_nodes: Vec<PostgrestNodeRow> = Vec::new();
    let mut raw_profiles: Vec<PostgrestProfileRow> = Vec::new();
    let mut raw_leases: Vec<PostgrestLeaseRow> = Vec::new();

    if let Some(ref c) = client {
        if let Ok(val) = c
            .select("nodes", "select=*&order=last_heartbeat_at.desc")
            .await
        {
            raw_nodes = serde_json::from_value(val).unwrap_or_default();
        }
        if let Ok(val) = c.select("instance_profiles", "select=*").await {
            raw_profiles = serde_json::from_value(val).unwrap_or_default();
        }
        if let Ok(val) = c.select("workspace_leases", "select=*").await {
            raw_leases = serde_json::from_value(val).unwrap_or_default();
        }
    }

    if raw_nodes.is_empty() {
        let local_machine = construct_local_machine_info(&config, &raw_leases);
        return Ok(vec![local_machine]);
    }

    let mut profiles_by_node: std::collections::HashMap<String, Vec<PostgrestProfileRow>> =
        std::collections::HashMap::new();
    for p in raw_profiles {
        profiles_by_node
            .entry(p.node_id.clone())
            .or_default()
            .push(p);
    }

    let mut machines = Vec::new();
    let mut has_local_node = false;

    for node in raw_nodes {
        let is_local = node.id == local_node_id;
        if is_local {
            has_local_node = true;
            let mut local_info = construct_local_machine_info(&config, &raw_leases);
            local_info.source = "supabase".to_string();
            machines.push(local_info);
            continue;
        }

        let node_profiles = profiles_by_node.remove(&node.id).unwrap_or_default();
        let mut instances = Vec::new();
        let mut active_accounts_set = HashSet::new();
        let mut total_running_prompts = 0;

        for prof in node_profiles {
            let prefix = format!("{}_", node.id);
            let instance_id = if prof.id.starts_with(&prefix) {
                prof.id[prefix.len()..].to_string()
            } else {
                prof.id.clone()
            };

            let running_count = prof.running_prompts_count.unwrap_or(0);
            total_running_prompts += running_count;

            if prof.active_account_email.len() > 0 {
                active_accounts_set.insert(prof.active_account_email.clone());
            } else if prof.active_account_id.len() > 0 {
                active_accounts_set.insert(prof.active_account_id.clone());
            }

            let mut is_leased = false;
            let mut lease_expires_at = None;
            for l in &raw_leases {
                let is_active_lease = l.expires_at > now;
                let has_prof_acc_id = prof.active_account_id.len() > 0;
                let has_prof_acc_email = prof.active_account_email.len() > 0;
                let is_matching_account = (has_prof_acc_id
                    && l.account_id == prof.active_account_id)
                    || (has_prof_acc_email && l.account_email == prof.active_account_email);
                let is_matching_node = l.node_id == node.id;
                let has_prof_name = prof.profile_name.len() > 0;
                let is_matching_profile = has_prof_name && l.profile_name == prof.profile_name;

                if is_active_lease
                    && (is_matching_account || (is_matching_node && is_matching_profile))
                {
                    is_leased = true;
                    lease_expires_at = Some(l.expires_at);
                    break;
                }
            }

            instances.push(FleetInstanceItem {
                instance_id,
                profile_name: prof.profile_name,
                bound_account_id: prof.active_account_id,
                bound_account_email: prof.active_account_email,
                is_active: prof.is_active,
                status: prof.status,
                running_prompts_count: running_count,
                updated_at: prof.updated_at,
                is_leased,
                lease_expires_at,
            });
        }

        let mut active_accounts: Vec<String> = active_accounts_set.into_iter().collect();
        active_accounts.sort();

        let total_instances = instances.len();

        let is_online = node.status == "online";
        machines.push(FleetMachineInfo {
            node_id: node.id,
            node_alias: node.alias.clone(),
            alias: node.alias,
            ip_address: node.ip_address,
            uptime_seconds: node.uptime_seconds,
            last_heartbeat_at: node.last_heartbeat_at,
            last_heartbeat_timestamp: node.last_heartbeat_at,
            status: node.status,
            is_online,
            is_local: false,
            source: "supabase".to_string(),
            total_instances,
            total_running_prompts,
            in_flight_prompts_count: total_running_prompts,
            active_accounts: active_accounts.clone(),
            bound_emails: active_accounts,
            instances,
            ..Default::default()
        });
    }

    if has_local_node {
        // Local node is already present in machines
    } else {
        let local_machine = construct_local_machine_info(&config, &raw_leases);
        machines.insert(0, local_machine);
    }

    // Sort machines: local machine is first, followed by online remote nodes sorted by heartbeat desc or alias
    machines.sort_by(|a, b| match (a.is_local, b.is_local) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => {
            let is_a_online = a.status == "online";
            let is_b_online = b.status == "online";
            match (is_a_online, is_b_online) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b
                    .last_heartbeat_at
                    .cmp(&a.last_heartbeat_at)
                    .then_with(|| a.alias.to_lowercase().cmp(&b.alias.to_lowercase())),
            }
        }
    });

    Ok(machines)
}

/// Start background synchronization daemon
pub fn start_sync_worker() {
    let is_already_running = SYNC_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tauri::async_runtime::spawn(async move {
        // Enforce 60-second startup quiet period: nothing runs until 1 minute after launch
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

            let config = match load_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if !config.is_sync_enabled {
                continue;
            }

            for ep in &config.endpoints {
                if !ep.is_enabled {
                    continue;
                }
                if ep.role == "root" {
                    // Justification: best-effort call; failure logged without changing control flow
                    crate::error::record_ignored(
                        send_heartbeat(ep, &config.node_alias).await,
                        "send_heartbeat",
                    );
                }
            }
        }
    });
}
