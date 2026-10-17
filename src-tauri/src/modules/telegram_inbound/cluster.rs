use crate::modules::email_watcher;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::process::Command;

use super::*;

/// Cluster node summary info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeInfo {
    pub alias: String,
    pub ip_address: String,
    pub status: String,
    pub source: String,
    pub last_seen: Option<String>,
    pub uptime_seconds: u64,
}

/// Parse GitMap cluster status output (e.g. Node vm-01: Connected (Last Seen: ...))
pub fn parse_gitmap_cluster_status(output: &str) -> Vec<ClusterNodeInfo> {
    let mut nodes = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Node ") {
            if let Some((alias, status_part)) = rest.split_once(':') {
                let alias = alias.trim().to_string();
                let status_trimmed = status_part.trim();
                let (status, last_seen) =
                    if let Some((st, seen_part)) = status_trimmed.split_once('(') {
                        let st = st.trim().to_string();
                        let seen = seen_part
                            .trim_end_matches(')')
                            .strip_prefix("Last Seen:")
                            .map(|s| s.trim().to_string());
                        (st, seen)
                    } else {
                        (status_trimmed.to_string(), None)
                    };

                nodes.push(ClusterNodeInfo {
                    alias,
                    ip_address: "Cluster Mesh".to_string(),
                    status,
                    source: "GitMap Fleet".to_string(),
                    last_seen,
                    uptime_seconds: 0,
                });
            }
        }
    }
    nodes
}

/// Query GitMap cluster nodes via CLI with JSON output first, falling back to text status
pub fn query_gitmap_cluster_nodes() -> Vec<ClusterNodeInfo> {
    // 1. Prefer gitmap cluster nodes --json for exact aliases and IPs
    let mut cmd_json = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd_json.creation_flags(0x08000000);
    }
    if let Ok(out) = cmd_json.args(["cluster", "nodes", "--json"]).output() {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                if let Some(arr) = val.as_array() {
                    let mut nodes = Vec::new();
                    for item in arr {
                        let alias = item["alias"].as_str().unwrap_or("").trim().to_string();
                        let ip = item["ip"]
                            .as_str()
                            .or_else(|| item["ipAddress"].as_str())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !alias.is_empty() {
                            nodes.push(ClusterNodeInfo {
                                alias,
                                ip_address: if ip.is_empty() {
                                    "Cluster Mesh".to_string()
                                } else {
                                    ip
                                },
                                status: "ready".to_string(),
                                source: "GitMap Fleet".to_string(),
                                last_seen: Some(Utc::now().to_rfc3339()),
                                uptime_seconds: 0,
                            });
                        }
                    }
                    if !nodes.is_empty() {
                        return nodes;
                    }
                }
            }
        }
    }

    // 2. Fallback to gitmap cluster status
    let mut cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output_res = cmd.args(["cluster", "status"]).output();
    if let Ok(out) = output_res {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let parsed = parse_gitmap_cluster_status(&stdout);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    Vec::new()
}

/// Resolve node network info: returns (canonical_alias, ip_address, signal_label)
pub fn resolve_node_network_info(target_or_alias: &str) -> (String, String, String) {
    let target = target_or_alias.trim();
    let local_ip = email_watcher::detect_local_ip();
    let local_name = email_watcher::detect_machine_name();
    let cluster_nodes = query_gitmap_cluster_nodes();

    // 1. Match against cluster nodes by alias or IP (authoritative cluster fleet ground truth)
    if let Some(matched) = cluster_nodes
        .iter()
        .find(|n| n.alias.eq_ignore_ascii_case(target) || n.ip_address.trim() == target)
    {
        let is_me = !local_ip.is_empty() && matched.ip_address.trim() == local_ip.trim();
        let signal = if is_me {
            "100% (Direct Local)".to_string()
        } else if matched.status.eq_ignore_ascii_case("ready")
            || matched.status.eq_ignore_ascii_case("connected")
            || matched.status.eq_ignore_ascii_case("online")
        {
            "98% (Online Mesh)".to_string()
        } else if matched.status.eq_ignore_ascii_case("offline") {
            "0% (Offline)".to_string()
        } else {
            matched.status.clone()
        };
        return (matched.alias.clone(), matched.ip_address.clone(), signal);
    }

    // 2. Check if target is explicitly this local machine (local, localhost, or matching local_ip)
    if target.eq_ignore_ascii_case("local")
        || target.eq_ignore_ascii_case("localhost")
        || (!local_ip.is_empty() && target.eq_ignore_ascii_case(&local_ip))
    {
        let my_alias = cluster_nodes
            .iter()
            .find(|n| !local_ip.is_empty() && n.ip_address.trim() == local_ip.trim())
            .map(|n| n.alias.clone())
            .unwrap_or_else(|| {
                if !local_name.is_empty() {
                    local_name.clone()
                } else {
                    "local".to_string()
                }
            });
        return (my_alias, local_ip, "100% (Direct Local)".to_string());
    }

    // 3. Fallback: check if target is a compound alias|ip
    if let Some((a, ip)) = target.split_once('|') {
        let (resolved_a, resolved_ip, sig) = resolve_node_network_info(a);
        let final_ip = if resolved_ip == "Unknown IP" || resolved_ip.is_empty() {
            ip.trim().to_string()
        } else {
            resolved_ip
        };
        return (resolved_a, final_ip, sig);
    }

    // 4. Fallback for non-cluster custom name matching local detected name
    if !local_name.is_empty() && target.eq_ignore_ascii_case(&local_name) {
        return (
            local_name,
            if !local_ip.is_empty() {
                local_ip
            } else {
                "127.0.0.1".to_string()
            },
            "100% (Direct Local)".to_string(),
        );
    }

    (
        target.to_string(),
        "Unknown IP".to_string(),
        "Unregistered Node".to_string(),
    )
}

/// Query Supabase Root DB nodes table
pub async fn query_supabase_cluster_nodes() -> Vec<ClusterNodeInfo> {
    let mut nodes = Vec::new();
    let local_config = supabase_sync::load_config().unwrap_or_default();
    for ep in &local_config.endpoints {
        if !ep.is_enabled || ep.role != "root" {
            continue;
        }
        if let Ok(client) = SupabaseClient::new(ep) {
            if let Ok(val) = client.select("nodes", "order=last_heartbeat_at.desc").await {
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        let alias = item["alias"].as_str().unwrap_or("Node").to_string();
                        let ip = item["ip_address"].as_str().unwrap_or("0.0.0.0").to_string();
                        let st = item["status"].as_str().unwrap_or("online").to_string();
                        let ut = item["uptime_seconds"].as_u64().unwrap_or(0);
                        let last_hb = item["last_heartbeat_at"].as_i64();
                        let seen_str = last_hb.map(|ts| {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| ts.to_string())
                        });

                        nodes.push(ClusterNodeInfo {
                            alias,
                            ip_address: ip,
                            status: st,
                            source: "Supabase Root DB".to_string(),
                            last_seen: seen_str,
                            uptime_seconds: ut,
                        });
                    }
                    break;
                }
            }
        }
    }
    nodes
}

/// Format comprehensive cluster nodes report (Local + GitMap Fleet + Supabase)
pub async fn format_cluster_nodes_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias.clone()
    };
    let local_ip = supabase_sync::get_local_ip();
    let local_uptime = supabase_sync::get_uptime_seconds();

    let mut all_nodes: Vec<ClusterNodeInfo> = Vec::new();

    // 1. Add Local Host Node
    all_nodes.push(ClusterNodeInfo {
        alias: local_alias.clone(),
        ip_address: local_ip,
        status: "online".to_string(),
        source: "Host Node".to_string(),
        last_seen: Some(Utc::now().to_rfc3339()),
        uptime_seconds: local_uptime,
    });

    // 2. Add GitMap cluster fleet nodes
    let gm_nodes = query_gitmap_cluster_nodes();
    for n in gm_nodes {
        if !all_nodes
            .iter()
            .any(|x| x.alias.eq_ignore_ascii_case(&n.alias))
        {
            all_nodes.push(n);
        }
    }

    // 3. Add Supabase cluster nodes
    let sb_nodes = query_supabase_cluster_nodes().await;
    for n in sb_nodes {
        if let Some(existing) = all_nodes
            .iter_mut()
            .find(|x| x.alias.eq_ignore_ascii_case(&n.alias))
        {
            if (existing.ip_address == "Cluster Mesh" || existing.ip_address == "0.0.0.0")
                && n.ip_address != "0.0.0.0"
            {
                existing.ip_address = n.ip_address;
            }
            if existing.uptime_seconds == 0 {
                existing.uptime_seconds = n.uptime_seconds;
            }
        } else {
            all_nodes.push(n);
        }
    }

    let mut rows = String::new();
    let total = all_nodes.len();
    for (i, node) in all_nodes.iter().enumerate() {
        let is_online = node.status.to_lowercase().contains("online")
            || node.status.to_lowercase().contains("connected");
        let badge = if is_online {
            "🟢 ONLINE"
        } else {
            "⚪ STANDBY"
        };

        let seen_display = node.last_seen.as_deref().unwrap_or("Active");
        let uptime_str = if node.uptime_seconds > 0 {
            format!(" | Uptime: {}m", node.uptime_seconds / 60)
        } else {
            String::new()
        };

        rows.push_str(&format!(
            "{}. {} <b>{}</b> (<code>{}</code>)\n   • <b>Fleet:</b> {} [{}]\n   • <b>Last Seen:</b> <code>{}</code>{}\n\n",
            i + 1,
            badge,
            clean_for_telegram_html(&node.alias, 48),
            clean_for_telegram_html(&node.ip_address, 48),
            clean_for_telegram_html(&node.source, 32),
            clean_for_telegram_html(&node.status, 24),
            clean_for_telegram_html(seen_display, 36),
            uptime_str
        ));
    }

    format!(
        "🖥️ <b>Antigravity VM Cluster Fleet ({} Nodes)</b>\n\n\
        {}\
        📋 <b>Fleet Commands:</b>\n\
        • <code>/nodes &lt;alias&gt; prompts</code> — Inspect running prompts on node\n\
        • <code>/prompt &lt;alias&gt; &lt;proj&gt; &lt;text&gt;</code> — Inject prompt to VM\n\
        • <code>/projects</code> — List active workspaces &amp; project IDs\n\
        • <code>/prompts</code> — List state database prompt queue",
        total, rows
    )
}

/// Format cluster nodes snapshot for Telegram response (backward compatibility)
pub async fn format_cluster_snapshot() -> String {
    format_cluster_nodes_report().await
}
