//! Headless decentralized fleet synchronization module.
//! Enables multi-machine telemetry and instance/account state synchronization
//! across nodes over channels such as Email and Telegram without Supabase.

use crate::error::AppError;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Telemetry and identity descriptor for a fleet machine node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetNodeInfo {
    pub node_id: String,
    pub node_alias: String,
    pub local_ip: String,
    pub os_info: String,
    pub agm_version: String,
    pub uptime_seconds: u64,
    pub is_online: bool,
    pub last_heartbeat_at: i64,
    pub sync_transport: String,
}

/// Instance state synchronized across fleet nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInstanceSync {
    pub instance_id: String,
    pub name: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub bound_account_id: Option<String>,
    pub bound_email: Option<String>,
    pub data_dir: String,
    pub executable_path: Option<String>,
    pub memory_mb: Option<f64>,
}

/// Workspace state synchronized across fleet nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetWorkspaceSync {
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub is_running: bool,
    pub running_prompts_count: u32,
    pub active_prompt_snippet: Option<String>,
}

/// Envelope payload for fleet synchronization messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSyncPayload {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_message_type")]
    pub message_type: String,
    pub node: FleetNodeInfo,
    #[serde(default)]
    pub instances: Vec<FleetInstanceSync>,
    #[serde(default)]
    pub accounts: Vec<serde_json::Value>,
    #[serde(default)]
    pub workspaces: Vec<FleetWorkspaceSync>,
    pub timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_quotas: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_signals: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast_forward_signals: Option<Vec<serde_json::Value>>,
}

fn default_schema() -> String {
    "agm-fleet-sync-v1.0".to_string()
}

fn default_message_type() -> String {
    "heartbeat".to_string()
}

/// Path to dedicated `fleet_sync.db` in the application data directory
pub fn get_fleet_sync_db_path() -> Result<PathBuf, AppError> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| AppError::Config(format!("Failed to get data dir: {e}")))?;
    Ok(data_dir.join("fleet_sync.db"))
}

/// Open and initialize connection to the fleet sync database
pub fn connect_fleet_db() -> Result<Connection, AppError> {
    let db_path = get_fleet_sync_db_path()?;
    if let Some(parent) = db_path.parent() {
        // Justification: best-effort directory creation; later file open fails loudly if directory creation failed
        crate::error::record_ignored(
            std::fs::create_dir_all(parent),
            "create fleet sync db directory",
        );
    }
    let conn = Connection::open(&db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

/// Initialize database schema: FleetNodes, FleetInstances, FleetWorkspaces, FleetSummaryView
pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS FleetNodes (
            node_id TEXT PRIMARY KEY,
            node_alias TEXT NOT NULL,
            local_ip TEXT NOT NULL,
            os_info TEXT NOT NULL,
            agm_version TEXT NOT NULL,
            uptime_seconds INTEGER NOT NULL,
            is_online INTEGER NOT NULL DEFAULT 1,
            last_heartbeat_at INTEGER NOT NULL,
            sync_transport TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS FleetInstances (
            node_id TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            name TEXT NOT NULL,
            is_running INTEGER NOT NULL DEFAULT 0,
            pid INTEGER,
            bound_account_id TEXT,
            bound_email TEXT,
            data_dir TEXT NOT NULL,
            executable_path TEXT,
            memory_mb REAL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (node_id, instance_id)
        );

        CREATE TABLE IF NOT EXISTS FleetWorkspaces (
            node_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            is_running INTEGER NOT NULL DEFAULT 0,
            running_prompts_count INTEGER NOT NULL DEFAULT 0,
            active_prompt_snippet TEXT,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (node_id, project_id)
        );

        CREATE VIEW IF NOT EXISTS FleetSummaryView AS
        SELECT 
            n.node_id,
            n.node_alias,
            n.local_ip,
            n.os_info,
            n.agm_version,
            n.uptime_seconds,
            n.is_online,
            n.last_heartbeat_at,
            n.sync_transport,
            COUNT(DISTINCT i.instance_id) AS total_instances,
            SUM(CASE WHEN i.is_running = 1 THEN 1 ELSE 0 END) AS running_instances,
            COUNT(DISTINCT w.project_id) AS total_workspaces,
            SUM(CASE WHEN w.is_running = 1 THEN 1 ELSE 0 END) AS running_workspaces
        FROM FleetNodes n
        LEFT JOIN FleetInstances i ON n.node_id = i.node_id
        LEFT JOIN FleetWorkspaces w ON n.node_id = w.node_id
        GROUP BY n.node_id;
        "#,
    )?;
    Ok(())
}

/// Helper to extract clean JSON substring from potential Markdown blocks or surrounding telemetry text
pub fn extract_json_envelope(raw: &str) -> &str {
    let trimmed = raw.trim();
    let s = if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else {
        trimmed
    };

    if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}')) {
        if start <= end {
            return &s[start..=end];
        }
    }
    s
}

fn synthesize_node_info(v: &serde_json::Value) -> FleetNodeInfo {
    let node_id = v
        .get("node_id")
        .and_then(|s| s.as_str())
        .unwrap_or("remote-node")
        .to_string();
    let node_alias = v
        .get("node_alias")
        .or_else(|| v.get("alias"))
        .or_else(|| v.get("machine_name"))
        .and_then(|s| s.as_str())
        .unwrap_or(&node_id)
        .to_string();
    let local_ip = v
        .get("local_ip")
        .or_else(|| v.get("ip"))
        .and_then(|s| s.as_str())
        .unwrap_or("127.0.0.1")
        .to_string();
    let os_info = v
        .get("os_info")
        .or_else(|| v.get("os"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let agm_version = v
        .get("agm_version")
        .or_else(|| v.get("version"))
        .and_then(|s| s.as_str())
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let uptime_seconds = v
        .get("uptime_seconds")
        .or_else(|| v.get("uptime"))
        .and_then(|u| u.as_u64())
        .unwrap_or(0);
    let is_online = v.get("is_online").and_then(|o| o.as_bool()).unwrap_or(true);
    let last_heartbeat_at = v
        .get("last_heartbeat_at")
        .or_else(|| v.get("timestamp"))
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| Utc::now().timestamp());
    let sync_transport = v
        .get("sync_transport")
        .or_else(|| v.get("transport"))
        .and_then(|t| t.as_str())
        .unwrap_or("json")
        .to_string();

    FleetNodeInfo {
        node_id,
        node_alias,
        local_ip,
        os_info,
        agm_version,
        uptime_seconds,
        is_online,
        last_heartbeat_at,
        sync_transport,
    }
}

/// Helper to parse incoming fleet sync JSON, accommodating pure envelopes and flexible Telegram / Email JSON variants
pub fn parse_fleet_sync_payload(payload_json: &str) -> Result<FleetSyncPayload, AppError> {
    let clean_json = extract_json_envelope(payload_json);

    // 1. Direct deserialization path
    if let Ok(payload) = serde_json::from_str::<FleetSyncPayload>(clean_json) {
        return Ok(payload);
    }

    // 2. Fallback ladder for flexible Telegram / Email payloads
    let v: serde_json::Value = serde_json::from_str(clean_json)
        .map_err(|e| AppError::Config(format!("Failed to deserialize fleet sync JSON: {e}")))?;

    let node = if let Some(node_val) = v.get("node") {
        serde_json::from_value::<FleetNodeInfo>(node_val.clone())
            .unwrap_or_else(|_| synthesize_node_info(&v))
    } else {
        synthesize_node_info(&v)
    };

    let instances: Vec<FleetInstanceSync> = v
        .get("instances")
        .and_then(|i| serde_json::from_value(i.clone()).ok())
        .unwrap_or_default();

    let accounts: Vec<serde_json::Value> = v
        .get("accounts")
        .and_then(|a| serde_json::from_value(a.clone()).ok())
        .or_else(|| {
            v.get("email_quotas")
                .and_then(|q| serde_json::from_value(q.clone()).ok())
        })
        .unwrap_or_default();

    let workspaces: Vec<FleetWorkspaceSync> = v
        .get("workspaces")
        .and_then(|w| serde_json::from_value(w.clone()).ok())
        .unwrap_or_default();

    let instance_count = v
        .get("instance_count")
        .or_else(|| v.get("instances_count"))
        .and_then(|c| c.as_u64())
        .map(|c| c as u32)
        .or_else(|| {
            if !instances.is_empty() {
                Some(instances.len() as u32)
            } else {
                None
            }
        });

    let email_quotas = v
        .get("email_quotas")
        .and_then(|eq| serde_json::from_value(eq.clone()).ok());

    let switch_signals = v
        .get("switch_signals")
        .and_then(|ss| serde_json::from_value(ss.clone()).ok());

    let fast_forward_signals = v
        .get("fast_forward_signals")
        .and_then(|ff| serde_json::from_value(ff.clone()).ok());

    let timestamp = v
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| Utc::now().timestamp());

    let schema = v
        .get("$schema")
        .and_then(|s| s.as_str())
        .unwrap_or("agm-fleet-sync-v1.0")
        .to_string();

    let message_type = v
        .get("message_type")
        .or_else(|| v.get("action"))
        .and_then(|m| m.as_str())
        .unwrap_or("heartbeat")
        .to_string();

    Ok(FleetSyncPayload {
        schema,
        message_type,
        node,
        instances,
        accounts,
        workspaces,
        timestamp,
        instance_count,
        email_quotas,
        switch_signals,
        fast_forward_signals,
    })
}

/// Ingest an incoming fleet sync JSON payload from any transport channel (e.g. Email, Telegram)
pub fn ingest_fleet_sync_payload(payload_json: &str, transport: &str) -> Result<(), AppError> {
    let mut payload = parse_fleet_sync_payload(payload_json)?;

    if !transport.trim().is_empty() {
        payload.node.sync_transport = transport.trim().to_string();
    }

    let mut conn = connect_fleet_db()?;
    let tx = conn.transaction()?;

    let now = Utc::now().timestamp();
    let is_online_int = if payload.node.is_online { 1 } else { 0 };

    tx.execute(
        r#"
        INSERT INTO FleetNodes (
            node_id, node_alias, local_ip, os_info, agm_version,
            uptime_seconds, is_online, last_heartbeat_at, sync_transport, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(node_id) DO UPDATE SET
            node_alias = excluded.node_alias,
            local_ip = excluded.local_ip,
            os_info = excluded.os_info,
            agm_version = excluded.agm_version,
            uptime_seconds = excluded.uptime_seconds,
            is_online = excluded.is_online,
            last_heartbeat_at = excluded.last_heartbeat_at,
            sync_transport = excluded.sync_transport,
            updated_at = excluded.updated_at
        "#,
        params![
            payload.node.node_id,
            payload.node.node_alias,
            payload.node.local_ip,
            payload.node.os_info,
            payload.node.agm_version,
            payload.node.uptime_seconds as i64,
            is_online_int,
            payload.node.last_heartbeat_at,
            payload.node.sync_transport,
            now,
        ],
    )?;

    // Replace instances for this node
    tx.execute(
        "DELETE FROM FleetInstances WHERE node_id = ?1",
        params![payload.node.node_id],
    )?;

    let instances_to_store: Vec<FleetInstanceSync> = if payload.instances.is_empty() {
        if let Some(count) = payload.instance_count {
            (0..count)
                .map(|idx| FleetInstanceSync {
                    instance_id: format!("{}-inst-{}", payload.node.node_id, idx + 1),
                    name: format!("Instance #{}", idx + 1),
                    is_running: true,
                    pid: None,
                    bound_account_id: None,
                    bound_email: None,
                    data_dir: "".to_string(),
                    executable_path: None,
                    memory_mb: None,
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        payload.instances.clone()
    };

    for inst in &instances_to_store {
        let is_running_int = if inst.is_running { 1 } else { 0 };
        tx.execute(
            r#"
            INSERT INTO FleetInstances (
                node_id, instance_id, name, is_running, pid,
                bound_account_id, bound_email, data_dir, executable_path, memory_mb, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                payload.node.node_id,
                inst.instance_id,
                inst.name,
                is_running_int,
                inst.pid,
                inst.bound_account_id,
                inst.bound_email,
                inst.data_dir,
                inst.executable_path,
                inst.memory_mb,
                now,
            ],
        )?;
    }

    // Replace workspaces for this node
    tx.execute(
        "DELETE FROM FleetWorkspaces WHERE node_id = ?1",
        params![payload.node.node_id],
    )?;

    for ws in &payload.workspaces {
        let is_running_int = if ws.is_running { 1 } else { 0 };
        tx.execute(
            r#"
            INSERT INTO FleetWorkspaces (
                node_id, project_id, repo_name, repo_path,
                is_running, running_prompts_count, active_prompt_snippet, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                payload.node.node_id,
                ws.project_id,
                ws.repo_name,
                ws.repo_path,
                is_running_int,
                ws.running_prompts_count,
                ws.active_prompt_snippet,
                now,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Generate telemetry and state payload for the current local machine
pub fn generate_local_fleet_sync_payload() -> Result<FleetSyncPayload, AppError> {
    let node_id = match machine_uid::get() {
        Ok(uid) => uid,
        Err(e) => {
            // Justification: fallback to synthetic local node ID when machine GUID cannot be read
            crate::error::record_ignored(Err::<(), _>(&e), "machine_uid get");
            "agm-node-local".to_string()
        }
    };

    let node_alias = crate::modules::supabase_sync::resolve_local_node_alias();
    let local_ip = get_local_ip();
    let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    let agm_version = env!("CARGO_PKG_VERSION").to_string();
    let uptime_seconds = sysinfo::System::uptime();
    let now = Utc::now().timestamp();

    let node = FleetNodeInfo {
        node_id,
        node_alias,
        local_ip,
        os_info,
        agm_version,
        uptime_seconds,
        is_online: true,
        last_heartbeat_at: now,
        sync_transport: "local".to_string(),
    };

    // Gather instances
    let mut instances = Vec::new();
    match crate::modules::instance::list_instances() {
        Ok(statuses) => {
            for status in statuses {
                instances.push(FleetInstanceSync {
                    instance_id: status.config.id,
                    name: status.config.name,
                    is_running: status.is_running,
                    pid: status.pid,
                    bound_account_id: status.config.bound_account_id,
                    bound_email: status.config.bound_email,
                    data_dir: status.config.data_dir,
                    executable_path: status.config.executable_path,
                    memory_mb: status.memory_mb,
                });
            }
        }
        Err(e) => {
            // Justification: instance listing is best-effort during local payload assembly
            crate::error::record_ignored(Err::<(), _>(&e), "list instances for fleet payload");
        }
    }

    // Gather accounts
    let mut accounts = Vec::new();
    match crate::modules::account::list_accounts() {
        Ok(acc_list) => {
            for acc in acc_list {
                match serde_json::to_value(&acc) {
                    Ok(v) => accounts.push(v),
                    Err(e) => {
                        // Justification: skip malformed account serialization during local payload assembly
                        crate::error::record_ignored(
                            Err::<(), _>(&e),
                            "serialize account for fleet payload",
                        );
                    }
                }
            }
        }
        Err(e) => {
            // Justification: account listing is best-effort during local payload assembly
            crate::error::record_ignored(Err::<(), _>(&e), "list accounts for fleet payload");
        }
    }

    // Gather workspaces
    let mut workspaces = Vec::new();
    match crate::modules::repo_db::project_queries::list_running_projects() {
        Ok(projects) => {
            for p in projects {
                let prompts = crate::modules::repo_db::discover_running_prompts_from_antigravity(
                    &p.instance_id,
                );
                let running_count = prompts
                    .iter()
                    .filter(|pr| pr.status == "running" || pr.status == "executing")
                    .count() as u32;
                let snippet = prompts
                    .first()
                    .map(|pr| pr.prompt_content.chars().take(120).collect::<String>());

                workspaces.push(FleetWorkspaceSync {
                    project_id: p.id,
                    repo_name: p.repo_name,
                    repo_path: p.repo_path,
                    is_running: p.is_running,
                    running_prompts_count: running_count,
                    active_prompt_snippet: snippet,
                });
            }
        }
        Err(e) => {
            // Justification: running projects discovery is best-effort during local payload assembly
            crate::error::record_ignored(
                Err::<(), _>(&e),
                "list running projects for fleet payload",
            );
        }
    }

    // Gather email quotas summary
    let mut email_quotas = Vec::new();
    match crate::modules::account::list_accounts() {
        Ok(acc_list) => {
            for acc in &acc_list {
                let quota_pct =
                    crate::modules::auto_switcher::calculate_account_quota(acc, "gemini-2.5-pro");
                let (q5h, q7d) = crate::modules::auto_switcher::extract_dual_window_quotas(
                    acc,
                    "gemini-2.5-pro",
                );
                email_quotas.push(serde_json::json!({
                    "email": acc.email,
                    "account_id": acc.id,
                    "disabled": acc.disabled,
                    "disabled_reason": acc.disabled_reason,
                    "quota_percent": quota_pct,
                    "rolling_5h_percent": q5h,
                    "weekly_7d_percent": q7d,
                    "priority": acc.priority,
                }));
            }
        }
        Err(e) => {
            // Justification: email quotas extraction is best-effort during local payload assembly
            crate::error::record_ignored(
                Err::<(), _>(&e),
                "list accounts for email quotas telemetry",
            );
        }
    }

    // Gather switch and fast-forwarding signals
    let app_config = crate::modules::config::load_app_config().unwrap_or_default();
    let switcher_cfg = app_config.auto_profile_switcher;
    let switcher_status = crate::modules::auto_switcher::get_status();

    let mut switch_signals = Vec::new();
    switch_signals.push(serde_json::json!({
        "auto_switcher_enabled": switcher_cfg.is_enabled,
        "target_model": switcher_cfg.target_model,
        "active_email": switcher_status.active_account_email,
        "active_quota": switcher_status.current_quota_percent,
        "low_quota_threshold_percent": switcher_cfg.low_quota_threshold_percent,
        "cooldown_period_minutes": switcher_cfg.account_cooldown_minutes,
        "monitored_instances_count": switcher_status.monitored_instances.len(),
    }));

    let mut fast_forward_signals = Vec::new();
    fast_forward_signals.push(serde_json::json!({
        "auto_fast_forward_on_critical": switcher_cfg.auto_fast_forward_on_critical,
        "fast_forward_shortcut": switcher_cfg.fast_forward_shortcut,
        "has_running_instances": instances.iter().any(|i| i.is_running),
    }));

    let instances_count = instances.len() as u32;

    Ok(FleetSyncPayload {
        schema: default_schema(),
        message_type: default_message_type(),
        node,
        instances,
        accounts,
        workspaces,
        timestamp: now,
        instance_count: Some(instances_count),
        email_quotas: Some(email_quotas),
        switch_signals: Some(switch_signals),
        fast_forward_signals: Some(fast_forward_signals),
    })
}

/// Generate pure JSON telemetry string for the current local machine node
pub fn generate_fleet_sync_json() -> Result<String, AppError> {
    let payload = generate_local_fleet_sync_payload()?;
    serde_json::to_string_pretty(&payload)
        .map_err(|e| AppError::Config(format!("Failed to serialize fleet sync payload: {e}")))
}

/// Ingest incoming fleet sync JSON from raw string across Email, Telegram, or HTTP transport channels
pub fn ingest_fleet_sync_json(payload_json: &str, transport: &str) -> Result<(), AppError> {
    ingest_fleet_sync_payload(payload_json, transport)
}

/// Retrieve summary list of fleet machines stored in local database
pub fn get_fleet_summary_machines() -> Result<Vec<FleetNodeInfo>, AppError> {
    let conn = connect_fleet_db()?;

    // Self-ingest local state so the current machine is always represented in the fleet view
    match generate_local_fleet_sync_payload() {
        Ok(local_payload) => match serde_json::to_string(&local_payload) {
            Ok(payload_json) => {
                // Justification: self-ingestion of local heartbeat is best-effort before querying summary
                crate::error::record_ignored(
                    ingest_fleet_sync_payload(&payload_json, "local"),
                    "self ingest local fleet payload",
                );
            }
            Err(e) => {
                // Justification: local payload serialization failure logged
                crate::error::record_ignored(
                    Err::<(), _>(&e),
                    "serialize local payload for self-ingest",
                );
            }
        },
        Err(e) => {
            // Justification: local payload generation failure logged
            crate::error::record_ignored(
                Err::<(), _>(&e),
                "generate local payload for self-ingest",
            );
        }
    }

    let mut stmt = conn.prepare(
        "SELECT node_id, node_alias, local_ip, os_info, agm_version, uptime_seconds, is_online, last_heartbeat_at, sync_transport 
         FROM FleetNodes 
         ORDER BY last_heartbeat_at DESC, node_id ASC"
    )?;

    let now = Utc::now().timestamp();
    let rows = stmt.query_map([], |row| {
        let is_online_raw: i64 = row.get(6)?;
        let last_heartbeat: i64 = row.get(7)?;
        let is_online = is_online_raw != 0 && (now - last_heartbeat) < 300;
        Ok(FleetNodeInfo {
            node_id: row.get(0)?,
            node_alias: row.get(1)?,
            local_ip: row.get(2)?,
            os_info: row.get(3)?,
            agm_version: row.get(4)?,
            uptime_seconds: row.get::<_, i64>(5)? as u64,
            is_online,
            last_heartbeat_at: last_heartbeat,
            sync_transport: row.get(8)?,
        })
    })?;

    let mut machines = Vec::new();
    for machine in rows {
        machines.push(machine?);
    }
    Ok(machines)
}

fn get_local_ip() -> String {
    let socket_res = std::net::UdpSocket::bind("0.0.0.0:0");
    match socket_res {
        Ok(socket) => match socket.connect("8.8.8.8:80") {
            Ok(()) => match socket.local_addr() {
                Ok(addr) => return addr.ip().to_string(),
                Err(e) => {
                    // Justification: best-effort local IP address lookup fallback
                    crate::error::record_ignored(
                        Err::<(), _>(&e),
                        "get local address for local ip",
                    );
                }
            },
            Err(e) => {
                // Justification: best-effort network probe fallback to loopback
                crate::error::record_ignored(Err::<(), _>(&e), "connect udp socket for local ip");
            }
        },
        Err(e) => {
            // Justification: best-effort socket bind fallback to loopback
            crate::error::record_ignored(Err::<(), _>(&e), "bind udp socket for local ip");
        }
    }
    "127.0.0.1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_envelope() {
        let raw = "```json\n{\"foo\": \"bar\"}\n```";
        assert_eq!(extract_json_envelope(raw), "{\"foo\": \"bar\"}");

        let surrounded = "Status telemetry:\n{\"a\": 1}\nEnd of message.";
        assert_eq!(extract_json_envelope(surrounded), "{\"a\": 1}");
    }

    #[test]
    fn test_parse_telegram_wrapped_fleet_sync_payload() {
        let telegram_msg = r#"
```json
{
  "$schema": "agm-fleet-sync-v1.0",
  "message_type": "heartbeat",
  "node_id": "tele-node-42",
  "node_alias": "Telegram Worker",
  "instance_count": 3,
  "email_quotas": [
    {
      "email": "user@example.com",
      "quota_percent": 88.5
    }
  ],
  "switch_signals": [
    {
      "auto_switcher_enabled": true
    }
  ],
  "fast_forward_signals": [
    {
      "auto_fast_forward_on_critical": true
    }
  ]
}
```
"#;
        let parsed = parse_fleet_sync_payload(telegram_msg).expect("should parse telegram payload");
        assert_eq!(parsed.node.node_id, "tele-node-42");
        assert_eq!(parsed.node.node_alias, "Telegram Worker");
        assert_eq!(parsed.instance_count, Some(3));
        assert!(parsed.email_quotas.is_some());
        assert!(parsed.switch_signals.is_some());
        assert!(parsed.fast_forward_signals.is_some());
    }

    #[test]
    fn test_parse_email_loose_json_payload() {
        let email_msg = r#"{
            "node": {
                "node_id": "email-node-1",
                "node_alias": "Email Worker",
                "local_ip": "10.0.0.5",
                "os_info": "linux x86_64",
                "agm_version": "4.8.3",
                "uptime_seconds": 1200,
                "is_online": true,
                "last_heartbeat_at": 1700000000,
                "sync_transport": "email"
            },
            "instance_count": 2,
            "instances": [],
            "email_quotas": [
                {"email": "dev@agm.local", "quota_percent": 95.0}
            ],
            "switch_signals": [
                {"active_email": "dev@agm.local"}
            ]
        }"#;

        let parsed = parse_fleet_sync_payload(email_msg).expect("should parse email payload");
        assert_eq!(parsed.node.node_id, "email-node-1");
        assert_eq!(parsed.instance_count, Some(2));
        assert_eq!(parsed.email_quotas.unwrap().len(), 1);
        assert_eq!(parsed.switch_signals.unwrap().len(), 1);
    }

    #[test]
    fn test_fleet_sync_e2e_ingest_and_query_pipeline() {
        let sample_payload = r##"{
            "schema": "agm-fleet-sync-v1.0",
            "message_type": "heartbeat",
            "node": {
                "node_id": "test-e2e-node-99",
                "node_alias": "E2E Worker Machine",
                "local_ip": "192.168.1.99",
                "os_info": "windows 11 x86_64",
                "agm_version": "4.207.0",
                "uptime_seconds": 3600,
                "is_online": true,
                "last_heartbeat_at": 1729051200,
                "sync_transport": "telegram"
            },
            "instance_count": 2,
            "instances": [
                {
                    "instance_id": "inst-alpha",
                    "instance_name": "#1 Main Instance",
                    "bound_email": "alpha@example.com",
                    "quota_percent": 94.5,
                    "is_running": true,
                    "active_prompts_count": 1,
                    "assigned_project": "Antigravity-Manager",
                    "memory_mb": 420.0
                }
            ],
            "email_quotas": [
                {"email": "alpha@example.com", "quota_percent": 94.5}
            ],
            "switch_signals": [
                {"auto_switcher_enabled": true, "target_model": "gemini-2.5-pro"}
            ],
            "fast_forward_signals": [
                {"auto_fast_forward_on_critical": true}
            ],
            "workspaces": [
                {"workspace_name": "Antigravity-Manager", "project_path": "d:/work/Antigravity-Manager", "running_prompts": 1}
            ],
            "timestamp": 1729051200
        }"##;

        // Ingest over Telegram transport
        let res = ingest_fleet_sync_payload(sample_payload, "telegram");
        assert!(
            res.is_ok(),
            "Failed to ingest fleet sync payload: {:?}",
            res.err()
        );

        // Query summary machines
        let machines = get_fleet_summary_machines().expect("Failed to get fleet summary machines");
        let found = machines.iter().find(|m| m.node_id == "test-e2e-node-99");
        assert!(
            found.is_some(),
            "Node test-e2e-node-99 should be present in fleet summary"
        );

        let node = found.unwrap();
        assert_eq!(node.node_alias, "E2E Worker Machine");
        assert_eq!(node.local_ip, "192.168.1.99");
        assert_eq!(node.sync_transport, "telegram");
    }
}
