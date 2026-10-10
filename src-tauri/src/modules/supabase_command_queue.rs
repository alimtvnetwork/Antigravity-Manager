//! Secondary DB Command Queue & Cascade Execution Module
//! Polls inbound prompts/commands, executes them, and streams telemetry with multi-endpoint failover.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::supabase_client::{SupabaseClient, SupabaseEndpoint};
use crate::modules::supabase_sync::SupabaseConfig;
use crate::utils::command::CommandExtWrapper;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use uuid::Uuid;

/// Inbound command stored in Supabase command queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundDbCommand {
    pub id: String,
    pub target_node_id: String,
    pub source: String,
    pub command_text: String,
    pub status: String,
    pub created_at: i64,
    pub started_at: i64,
    pub completed_at: i64,
}

/// Retrieve secondary endpoints sorted by priority
pub fn get_secondary_endpoints(config: &SupabaseConfig) -> Vec<SupabaseEndpoint> {
    let mut endpoints: Vec<SupabaseEndpoint> = config
        .endpoints
        .iter()
        .filter(|e| e.is_enabled && e.role == "secondary")
        .cloned()
        .collect();
    endpoints.sort_by_key(|e| e.priority);
    endpoints
}

/// Poll pending commands from the first available secondary database (cascading failover)
pub async fn fetch_pending_commands(
    config: &SupabaseConfig,
    node_id: &str,
) -> (Option<SupabaseClient>, Vec<InboundDbCommand>) {
    let endpoints = get_secondary_endpoints(config);

    for ep in endpoints {
        let client = match SupabaseClient::new(&ep) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let query = format!(
            "status=eq.pending&(target_node_id=eq.{}|target_node_id=eq.*)&order=created_at.asc&limit=5",
            node_id
        );

        if let Ok(val) = client.select("command_queue", &query).await {
            if let Ok(cmds) = serde_json::from_value::<Vec<InboundDbCommand>>(val) {
                if !cmds.is_empty() {
                    return (Some(client), cmds);
                }
            }
        }
    }

    (None, Vec::new())
}

/// Execute a single inbound command and write telemetry back
pub async fn execute_and_report_command(
    client: &SupabaseClient,
    cmd: &InboundDbCommand,
    node_id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().timestamp();

    // 1. Mark command as running
    let run_payload = json!({
        "status": "running",
        "started_at": now
    });
    let query = format!("id=eq.{}", cmd.id);
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        client.update("command_queue", &query, run_payload).await,
        "update",
    );

    // 2. Execute command
    #[cfg(target_os = "windows")]
    let output_res = {
        let mut cmd_proc = Command::new("powershell");
        cmd_proc.creation_flags_windows().args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &cmd.command_text,
        ]);
        cmd_proc.output()
    };

    #[cfg(not(target_os = "windows"))]
    let output_res = Command::new("sh").args(["-c", &cmd.command_text]).output();

    let (stdout, stderr, exit_code) = match output_res {
        Ok(out) => (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code().unwrap_or(0),
        ),
        Err(e) => (String::new(), format!("Process execution error: {}", e), 1),
    };

    let finish_time = Utc::now().timestamp();
    let final_status = if exit_code == 0 {
        "completed"
    } else {
        "failed"
    };

    // 3. Record telemetry
    let telemetry_payload = json!({
        "id": Uuid::new_v4().to_string(),
        "command_id": cmd.id,
        "node_id": node_id,
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": exit_code,
        "created_at": finish_time
    });
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        client.insert("command_telemetry", telemetry_payload).await,
        "insert",
    );

    // 4. Update command queue status
    let finish_payload = json!({
        "status": final_status,
        "completed_at": finish_time
    });
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        client.update("command_queue", &query, finish_payload).await,
        "update",
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_secondary_endpoints_filtering_and_priority() {
        let config = SupabaseConfig {
            endpoints: vec![
                SupabaseEndpoint {
                    id: "sec_p2".to_string(),
                    name: "Secondary P2".to_string(),
                    url: "https://sec2.supabase.co".to_string(),
                    api_key: "k2".to_string(),
                    role: "secondary".to_string(),
                    is_enabled: true,
                    prune_threshold_mb: 200,
                    priority: 2,
                    notes: None,
                    tags: vec![],
                },
                SupabaseEndpoint {
                    id: "root_p1".to_string(),
                    name: "Root DB".to_string(),
                    url: "https://root.supabase.co".to_string(),
                    api_key: "kr".to_string(),
                    role: "root".to_string(),
                    is_enabled: true,
                    prune_threshold_mb: 400,
                    priority: 1,
                    notes: None,
                    tags: vec![],
                },
                SupabaseEndpoint {
                    id: "sec_disabled".to_string(),
                    name: "Disabled Secondary".to_string(),
                    url: "https://dis.supabase.co".to_string(),
                    api_key: "kd".to_string(),
                    role: "secondary".to_string(),
                    is_enabled: false,
                    prune_threshold_mb: 200,
                    priority: 0,
                    notes: None,
                    tags: vec![],
                },
                SupabaseEndpoint {
                    id: "sec_p1".to_string(),
                    name: "Secondary P1".to_string(),
                    url: "https://sec1.supabase.co".to_string(),
                    api_key: "k1".to_string(),
                    role: "secondary".to_string(),
                    is_enabled: true,
                    prune_threshold_mb: 200,
                    priority: 1,
                    notes: None,
                    tags: vec![],
                },
            ],
            node_alias: "Node-Test".to_string(),
            is_sync_enabled: true,
            auto_prune_root_mb: 400,
            auto_prune_secondary_mb: 200,
            heartbeat_interval_secs: 30,
        };

        let secondary = get_secondary_endpoints(&config);
        assert_eq!(secondary.len(), 2);
        assert_eq!(secondary[0].id, "sec_p1");
        assert_eq!(secondary[1].id, "sec_p2");
    }

    #[test]
    fn test_inbound_db_command_deserialization() {
        let json_data = r#"{
            "id": "cmd-123",
            "target_node_id": "node-alpha",
            "source": "telegram",
            "command_text": "git status",
            "status": "pending",
            "created_at": 1700000000,
            "started_at": 0,
            "completed_at": 0
        }"#;

        let cmd: InboundDbCommand = serde_json::from_str(json_data).expect("valid json command");
        assert_eq!(cmd.id, "cmd-123");
        assert_eq!(cmd.target_node_id, "node-alpha");
        assert_eq!(cmd.source, "telegram");
        assert_eq!(cmd.command_text, "git status");
        assert_eq!(cmd.status, "pending");
        assert_eq!(cmd.created_at, 1700000000);
    }
}
