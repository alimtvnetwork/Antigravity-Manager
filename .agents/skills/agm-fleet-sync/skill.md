---
name: agm-fleet-sync
description: Specialized skill for managing the Antigravity-Manager headless decentralized fleet synchronization, multi-machine node discovery, telemetry broadcasts over Email/Telegram without Supabase, and agm fleet CLI commands.
---

# AGM Headless Fleet Synchronization & Decentralized Multi-Node Telemetry

Governs multi-machine node discovery, decentralized telemetry broadcasting, instance and workspace state aggregation across nodes without requiring Supabase PostgreSQL, and local tracking via `fleet_sync.db` in Antigravity-Manager (`src-tauri/src/modules/fleet_sync.rs` and `src-tauri/src/bin/agm/fleet_cmds.rs`).

---

## 1. Architectural Overview

For headless servers, isolated VPCs, or environments without external database connections, **Fleet Sync** provides peer-to-peer and channel-mediated state replication across multiple Antigravity-Manager hosts:

```
+--------------------------------------------------------------------------------------------------+
|                                        Fleet Nodes                                               |
|       Host Alpha (192.168.1.10)        Host Beta (192.168.1.20)        Host Gamma (Headless VPS) |
+-------------------------------------------------+------------------------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                                Decentralized Transport Layer                                     |
|     1. Email Transport (IMAP/SMTP with encrypted JSON envelopes)                                  |
|     2. Telegram Bot Transport (Bounded message blocks and status alerts)                          |
|     3. LAN HTTP / Direct Socket Peer-to-Peer Polling                                             |
+-------------------------------------------------+------------------------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                            Local Storage: fleet_sync.db (WAL Mode)                               |
|   - FleetNodes: node_id, node_alias, local_ip, os_info, agm_version, uptime, is_online           |
|   - FleetInstances: instance_id, pid, is_running, bound_account, memory_mb                       |
|   - FleetWorkspaces: project_id, repo_name, repo_path, running_prompts_count, prompt_snippet    |
|   - FleetSummaryView: Aggregated node health, online status, active instances & prompts          |
+--------------------------------------------------------------------------------------------------+
```

---

## 2. Key Structures & Envelope Schema

### Schema Envelope (`agm-fleet-sync-v1.0`)
All fleet synchronization messages are serialized via `FleetSyncPayload`:

```rust
pub struct FleetSyncPayload {
    pub schema: String,             // "agm-fleet-sync-v1.0"
    pub message_type: String,       // "heartbeat", "instance_change", "quota_alert"
    pub node: FleetNodeInfo,        // Node hardware, OS, IP, and version metadata
    pub instances: Vec<FleetInstanceSync>, // Active sandboxed instances
    pub accounts: Vec<Value>,       // Quota & account status
    pub workspaces: Vec<FleetWorkspaceSync>, // Running project repos & prompt counts
    pub timestamp: i64,             // Unix timestamp in seconds
}
```

### Node Identity & State Descriptors
- **`FleetNodeInfo`**: Records `node_id`, `node_alias`, `local_ip`, `os_info`, `agm_version`, `uptime_seconds`, `is_online`, and `last_heartbeat_at`.
- **`FleetInstanceSync`**: Tracks sandbox runtime data (`instance_id`, `name`, `is_running`, `pid`, `bound_account_id`, `bound_email`, `data_dir`, `executable_path`, `memory_mb`).
- **`FleetWorkspaceSync`**: Tracks workspace activities (`project_id`, `repo_name`, `repo_path`, `is_running`, `running_prompts_count`, `active_prompt_snippet`).

---

## 3. Dedicated Database Architecture (`fleet_sync.db`)

Fleet synchronization isolates its state into a dedicated SQLite database located in the application data directory:

- **Concurrency**: WAL mode (`PRAGMA journal_mode = WAL;`) and synchronous normal (`PRAGMA synchronous = NORMAL;`).
- **Heartbeat & Liveness Invariant**: Nodes are flagged offline when `last_heartbeat_at` exceeds the configured TTL threshold (default 120 seconds).
- **Relational View (`FleetSummaryView`)**: Aggregates total and running instances and workspaces per node for single-query dashboard queries.

---

## 4. CLI Interface Reference

```bash
# Display summary list of all known fleet nodes and their online states
agm fleet nodes

# Output machine-readable JSON fleet topology for monitoring scripts
agm fleet nodes --json

# Send a ping probe to verify node responsiveness
agm fleet ping

# Trigger immediate broadcast of local instance and workspace telemetry
agm fleet broadcast

# Synchronize state with known peers across channels
agm fleet sync

# Inspect detailed health and resource consumption of the local fleet daemon
agm fleet health
```
