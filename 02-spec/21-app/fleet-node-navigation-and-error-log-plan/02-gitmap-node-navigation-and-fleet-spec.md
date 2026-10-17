# Specification: Multi-Machine GitMap Node Discovery, 4-Tier Fleet Mesh Architecture & CLI Navigation Parity

- **Slug**: `fleet-node-navigation-and-error-log-plan`
- **Module**: `Fleet Mesh, Remote Node Navigation, GitMap Integration & Multi-Machine Instances Matrix`
- **Specification Path**: `02-spec/21-app/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-spec.md`
- **Parent Plan**: `.ai-memory/plans/fleet-node-navigation-and-error-log-plan.md`
- **Subtask Plan**: `.ai-memory/plans/subtasks/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-plan.md`
- **Mode**: `PLAN MODE (Non-Destructive Architectural Specification & Actionable AI Instructions)`
- **Status**: `[APPROVED & IMPLEMENTATION READY]`

---

## User Request (Verbatim)

> "Antigravity Manager (`Antigravity Manager (AGM)`) have lots of errors in the log. I have recorded it. Can you please look into those error logs and write into the plan mode? Because the code is going into refactor, we cannot fix it. Make sure we put this as a plan so that we can fix it later on. Also, try to tie it with the new refactors. If something changes, include that into the plan. Do not execute or fix this issue. Include the log, create a detailed AI instruction so that AI can do the fixes later on. The feed section, or when we are in the instance tab, the other machine information is still not visible. Check the gitmap, that gitmap connects with the nodes correctly. But in your case, we do not have the connection with the other machine, and we cannot get the information here in the UI level and also the CLI level. You need to have navigation command like gitmap. You need to have help checkouts, information that you could request to other Antigravity Manager (`Antigravity Manager (AGM)`) through SSH or communications like gitmap. The Antigravity Manager (`Antigravity Manager (AGM)`) should be able to read gitmap configuration aliasing and can influence itself as well. Focus on this."

---

## 1. Executive Summary & Architectural Motivation

While GitMap successfully maintains a resilient, high-speed multi-node SSH cluster (`gitmap nodes --json`, managing 6 nodes: `w1`, `w2`, `w3`, `w4`, `u1`, and `main`), Antigravity Manager (`AGM`) currently fails to surface remote machines in both its GUI (`src/pages/Instances.tsx`) and CLI (`src-tauri/src/bin/agm/fleet_cmds.rs`).

This disconnect leaves operators blind to remote cluster capacity, unable to inspect instances running on remote nodes, and unable to execute remote commands or navigation routines from AGM.

Investigation reveals that AGM's fleet reporting was architected with a hard, single-point-of-failure coupling to cloud Supabase PostgREST synchronization. When Supabase is disabled, paused, or unreachable:
1. The frontend (`src/components/instances/fleet/useFleetMachines.ts`) aborts entirely and renders a disabled placeholder.
2. The backend (`src-tauri/src/modules/supabase_sync/query.rs`) falls back to returning only the local machine.
3. The frontend filter subsequently strips that single local machine from the "Other Machines" table, resulting in an empty list `[]`.
4. The CLI commands (`agm fleet nodes`, `agm fleet ping`) are hardcoded single-node stubs.
5. AGM's parser calls unsupported CLI flags on GitMap (`gitmap ssh nodes --json`) and cannot deserialize GitMap's output schema due to missing Serde aliases.

This specification designs a **4-Tier Discovery Ladder Architecture**, establishes **GitMap-parity CLI navigation commands**, standardizes **configuration aliasing and bidirectional self-influence**, and specifies the **Instances Tab Remote Machine Matrix UI**.

---

## 2. Multi-Machine GitMap Node Discovery Topology

### 2.1 GitMap Working Cluster Configuration

GitMap defines its multi-node cluster topology within two configuration envelopes adhering to Schema v2.0:
- `gitmap-ssh-nodes.json` (workspace root and `~/.gitmap/`)
- `gitmap-ssh.json` (workspace root)

Both files use the standardized two-tier JSON Envelope format:

```json
{
  "attributes": {
    "type": "ssh-nodes",
    "source": "gitmap-ssh-nodes.json",
    "how": "gitmap ssh export",
    "version": "2.0",
    "gitmapVersion": "6.563.0",
    "timestamp": "2026-10-17T14:50:10Z"
  },
  "variables": {
    "repoDir": "${workDir}\\gitmap",
    "secretsDir": ".",
    "workDir": "D:\\work"
  },
  "data": {
    "schemaVersion": "2.0",
    "exportedAt": "2026-10-17T14:50:10Z",
    "totalNodes": 6,
    "nodes": [
      {
        "workerId": "worker-1",
        "id": 1,
        "alias": "w3",
        "ipAddress": "192.168.1.12",
        "username": "Administrator",
        "port": 22,
        "os": "windows",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      },
      {
        "workerId": "worker-2",
        "id": 2,
        "alias": "w1",
        "ipAddress": "192.168.1.3",
        "username": "Administrator",
        "port": 22,
        "os": "win",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      },
      {
        "workerId": "worker-3",
        "id": 3,
        "alias": "w2",
        "ipAddress": "192.168.1.7",
        "username": "Administrator",
        "port": 22,
        "os": "windows",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      },
      {
        "workerId": "worker-4",
        "id": 4,
        "alias": "w4",
        "ipAddress": "192.168.1.13",
        "username": "Administrator",
        "port": 22,
        "os": "linux",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      },
      {
        "workerId": "worker-5",
        "id": 5,
        "alias": "u1",
        "ipAddress": "192.168.1.22",
        "username": "a",
        "port": 22,
        "os": "linux",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      },
      {
        "workerId": "worker-6",
        "id": 6,
        "alias": "main",
        "ipAddress": "192.168.1.20",
        "username": "administrator",
        "port": 22,
        "os": "win",
        "authMethod": "key",
        "keyPath": "C:\\Users\\Administrator\\.ssh\\id_rsa"
      }
    ]
  }
}
```

### 2.2 GitMap Live CLI Output Schema (`gitmap nodes --json`)

Executing `gitmap nodes --json` produces a verified JSON array containing dynamic status and subsystem mappings:

```json
[
  {
    "alias": "main",
    "role": "worker",
    "host": "192.168.1.20",
    "port": 22,
    "user": "administrator",
    "status": "● ready",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": true
  },
  {
    "alias": "w3",
    "role": "worker",
    "host": "192.168.1.12",
    "port": 22,
    "user": "Administrator",
    "status": "○ offline (timeout)",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": false
  },
  {
    "alias": "w1",
    "role": "worker",
    "host": "192.168.1.3",
    "port": 22,
    "user": "Administrator",
    "status": "● ready",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": true
  },
  {
    "alias": "w2",
    "role": "worker",
    "host": "192.168.1.7",
    "port": 22,
    "user": "Administrator",
    "status": "○ offline (timeout)",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": false
  },
  {
    "alias": "w4",
    "role": "worker",
    "host": "192.168.1.13",
    "port": 22,
    "user": "Administrator",
    "status": "○ offline (timeout)",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": false
  },
  {
    "alias": "u1",
    "role": "worker",
    "host": "192.168.1.22",
    "port": 22,
    "user": "a",
    "status": "○ offline (timeout)",
    "subsystems": ["SSH", "Cluster", "SC"],
    "enrolled_at": "2026-09-29 14:38:44",
    "is_online": false
  }
]
```

### 2.3 Live Network Reachability Matrix

| Node Alias | Host (IP:Port) | OS Target | User | Live Status | Subsystems | Reachability Notes |
|---|---|---|---|---|---|---|
| `main` | `192.168.1.20:22` | Windows | `administrator` | **● ONLINE** | SSH, Cluster, SC | Primary workstation / local orchestrator |
| `w1` | `192.168.1.3:22` | Windows | `Administrator` | **● ONLINE** | SSH, Cluster, SC | Worker 1, verified TCP 22 open (latency: 2ms) |
| `w2` | `192.168.1.7:22` | Windows | `Administrator` | **○ OFFLINE** | SSH, Cluster, SC | Worker 2, powered down / Wi-Fi sleep |
| `w3` | `192.168.1.12:22` | Windows | `Administrator` | **○ OFFLINE** | SSH, Cluster, SC | Worker 3, TCP timeout (1500ms) |
| `w4` | `192.168.1.13:22` | Linux | `Administrator` | **○ OFFLINE** | SSH, Cluster, SC | Worker 4, TCP timeout (1500ms) |
| `u1` | `192.168.1.22:22` | Linux | `a` | **○ OFFLINE** | SSH, Cluster, SC | Ubuntu worker, TCP timeout (1500ms) |

---

## 3. Root Cause Analysis: Missing Remote Machines in AGM (GAP-01 to GAP-06)

A forensic trace through AGM's frontend, backend, CLI, and SSH integration reveals six compounding gaps (GAP-01 through GAP-06) causing remote machines to be completely invisible.

### GAP-01: Frontend Short-Circuit on Supabase Sync Status

- **Location**: `src/components/instances/fleet/useFleetMachines.ts:62-69` and `src/components/instances/FleetMachinesTable.tsx:27-29`
- **Code Trace**:
  ```typescript
  // useFleetMachines.ts
  const config = await supabaseService.getConfig().catch(() => null);
  const syncActive = Boolean(config?.is_sync_enabled);
  setIsSyncEnabled(syncActive);

  if (!syncActive) {
      setMachines([]);
      return;
  }
  ```
  ```tsx
  // FleetMachinesTable.tsx
  if (!isLoading && !isSyncEnabled) {
      return <FleetDisabledState />;
  }
  ```
- **Root Cause**: If Supabase sync is disabled (`is_sync_enabled = false`), unconfigured, or offline, the hook immediately wipes all machines (`setMachines([])`) and exits. `FleetMachinesTable` replaces the entire view with `<FleetDisabledState />`, instructing the user to configure Supabase. AGM never attempts to probe the local SSH mesh, GitMap nodes, or peer caches.

### GAP-02: Backend PostgREST Monolithic Dependency & Local-Only Fallback

- **Location**: `src-tauri/src/modules/supabase_sync/query.rs:21-32` and `src-tauri/src/modules/supabase_sync/migration.rs:142-210`
- **Code Trace**:
  ```rust
  // query.rs
  if !config.is_sync_enabled {
      return Ok(build_local_fallback_machine());
  }
  let root_ep = match config.endpoints.iter().find(|ep| ep.is_enabled && ep.role == "root") {
      Some(ep) => ep.clone(),
      None => return Ok(build_local_fallback_machine()),
  };
  ```
  ```rust
  // migration.rs
  pub fn build_local_fallback_machine() -> Vec<FleetMachineInfo> {
      let node_id = get_local_node_id();
      // ... only populates local machine telemetry
      vec![local_machine]
  }
  ```
- **Root Cause**: When Supabase is disabled or unreachable, `query_fleet_machines()` returns `build_local_fallback_machine()`. This function strictly constructs a `Vec` containing only the current local machine (`vec![local_machine]`). It never consults `load_ssh_connections()`, `gitmap-ssh-nodes.json`, or `fleet_sync.db`.

### GAP-03: Self-Canceling Frontend Local ID Filter

- **Location**: `src/components/instances/fleet/useFleetMachines.ts:80-82`
- **Code Trace**:
  ```typescript
  const data = await fetchFleetMachinesApi();
  const filtered = data.filter((m) => m.node_id !== effectiveLocalId);
  setMachines(filtered);
  ```
- **Root Cause**: When GAP-02 executes and returns `[local_machine]`, the frontend receives an array containing only the local node. Line 81 filters out `effectiveLocalId` (the local node itself). The resulting array is guaranteed to be empty `[]`. In `FleetMachinesTable.tsx`, `machines.length === 0` renders `<FleetEmptyState />`, giving the impression that no other machines exist on the network.

### GAP-04: Siloed SSH Manager Connectivity Decoupled from Instances Tab

- **Location**: `src-tauri/src/modules/ssh_manager/exec.rs:417-570` and `src/services/sshService.ts`
- **Code Trace**:
  ```rust
  // exec.rs
  pub fn get_cluster_instances_and_balances() -> Result<Vec<ClusterInstanceBalanceItem>, String> {
      // ... loads ssh_conns, tests TCP liveness, resolves username placeholders
      let ssh_conns = load_ssh_connections().unwrap_or_default();
      for conn in ssh_conns {
          let reachable = check_tcp_liveness(&conn.ip_address, port, 1500);
          // ... constructs remote balance item
      }
  }
  ```
- **Root Cause**: AGM already possesses fully functional multi-node SSH discovery and TCP reachability verification (`get_cluster_instances_and_balances`). However, this logic is siloed exclusively within `Settings -> Email Notifications -> SSH / GitMap` (`src/components/settings/EmailNotifications/SshGitmapSection.tsx`). The primary Instances view (`src/pages/Instances.tsx`) does not invoke or merge this data, isolating remote machine topology from daily instance workflows.

### GAP-05: CLI `agm fleet nodes` Hardcoded Local Stub

- **Location**: `src-tauri/src/bin/agm/fleet_cmds.rs:40-62`
- **Code Trace**:
  ```rust
  pub(crate) fn cmd_fleet_nodes(args: &[String]) {
      let ctx = CliContext::parse(args);
      let (name, ip) = email_sender::get_local_node_identity();
      if ctx.json_output {
          let data = serde_json::json!([{ "node_alias": name, "local_ip": ip, "is_leader": true }]);
          CliEnvelope::ok("fleet nodes", None, data).print_and_exit();
      }
      println!("Fleet Nodes:\n  ● {} ({}) - Online", name, ip);
  }
  ```
- **Root Cause**: The terminal CLI command `agm fleet nodes` is a hardcoded stub that prints only the local node identity. It performs zero lookups against GitMap, SSH configs, or Supabase endpoints. Operators inspecting the fleet from the terminal receive incomplete, single-node output.

### GAP-06: Unsupported GitMap CLI Arguments & Deserialization Schema Incompatibility

- **Location**: `src-tauri/src/modules/ssh_manager/sync.rs:173-195` and `src-tauri/src/modules/ssh_manager/types.rs:25-79`
- **Code Trace**:
  ```rust
  // sync.rs:173
  for args in [
      &["ssh", "nodes", "export-json"][..],
      &["ssh", "nodes", "--json"][..],
  ] {
      let mut cmd = std::process::Command::new("gitmap");
      // ...
  }
  ```
  ```rust
  // types.rs:26-47
  pub struct SshNodeExportItem {
      pub alias: String,
      #[serde(alias = "ipAddress", alias = "ip")]
      pub ip_address: String,
      pub username: String,
      // ... missing alias = "host", alias = "user"
  }
  ```
- **Root Causes**:
  1. **Invalid Arguments**: AGM invokes `gitmap ssh nodes --json` and `gitmap ssh nodes export-json`. In GitMap, `--json` is a flag on `gitmap nodes`, not on `gitmap ssh nodes`. Calling `gitmap ssh nodes --json` prints an interactive colored ASCII terminal table, which fails JSON decoding.
  2. **Schema Incompatibility**: When `gitmap nodes --json` is invoked directly, GitMap emits fields `"host": "192.168.1.3"` and `"user": "Administrator"`. In AGM's `SshNodeExportItem` and `SshConnectionRecord`, `ip_address` only aliased `"ipAddress"` and `"ip"`, while `username` had no aliases. Consequently, Serde fails to deserialize valid GitMap JSON payloads.

---

## 4. 4-Tier Discovery Ladder Architecture

To guarantee high availability and eliminate single-point-of-failure vulnerabilities, AGM must transition from a Supabase-only model to a **4-Tier Discovery Ladder**.

```
+----------------------------------------------------------------------------------------------------+
|                                    4-Tier Discovery Ladder                                         |
|                                                                                                    |
|   +--------------------------------------------------------------------------------------------+   |
|   | Tier 1: Cloud Supabase Sync (Primary Cloud Backbone)                                       |   |
|   | - High-scale cross-network discovery, active instance profiles, workspace lease locks.     |   |
|   | - Fallback condition: is_sync_enabled == false OR endpoint unreachable OR paused (503/404).|   |
|   +---------------------------------------------+----------------------------------------------+   |
|                                                 | (On Unavailability / Concurrent Layering)        |
|                                                 v                                                  |
|   +--------------------------------------------------------------------------------------------+   |
|   | Tier 2: Local GitMap SSH Mesh (High-Speed Local LAN Cluster)                              |   |
|   | - Subprocess `gitmap nodes --json`, `gitmap-ssh-nodes.json`, `~/.ssh/config`.              |   |
|   | - Concurrent TCP liveness probes (1500ms timeout), key/password credential mapping.       |   |
|   +---------------------------------------------+----------------------------------------------+   |
|                                                 | (On Zero SSH Nodes / Offline LAN)                |
|                                                 v                                                  |
|   +--------------------------------------------------------------------------------------------+   |
|   | Tier 3: Decentralized SQLite Fleet Sync (Peer Broadcast Mesh)                              |   |
|   | - Local `fleet_sync.db` synchronized via IMAP/SMTP email envelopes or Telegram daemons.    |   |
|   | - Cached last-known node heartbeats and instance states from background telemetry.         |   |
|   +---------------------------------------------+----------------------------------------------+   |
|                                                 | (Ultimate Safety Baseline)                       |
|                                                 v                                                  |
|   +--------------------------------------------------------------------------------------------+   |
|   | Tier 4: Local Node Telemetry (Self-Healing Baseline)                                       |   |
|   | - Local hardware identity, PID scanner, running prompts in `repodb.sqlite`.                |   |
|   +--------------------------------------------------------------------------------------------+   |
+----------------------------------------------------------------------------------------------------+
```

### 4.1 Tier Specification & Resolution Order

1. **Tier 1 — Cloud Supabase Sync**:
   - Polled when `is_sync_enabled = true` and `endpoints` has an active `root` entry.
   - Provides global cluster visibility across WAN/LAN, instance profile registrations, and distributed lease locks (`[AGM IN-USE]`).
   - If Supabase returns an error, timeout, or pause condition, execution transparently degrades to Tier 2 without throwing user-facing modal errors.

2. **Tier 2 — Local GitMap SSH Mesh**:
   - Read from `gitmap nodes --json`, candidate JSON files (`gitmap-ssh-nodes.json`), and AGM's local `ssh_nodes.json`.
   - Probes each discovered node via non-blocking TCP socket (`port: 22`, timeout: `1500ms`).
   - Populates remote machine rows, measuring round-trip latency (`latency_ms`) and resolving OS/user identities.

3. **Tier 3 — Decentralized SQLite Fleet Sync**:
   - Queries `fleet_sync.db` (`fleet_nodes` table) populated by background email sync (`email_watcher`) and Telegram telemetry.
   - Restores last-known machine configurations for nodes on disjoint subnets where direct SSH cannot establish a route.

4. **Tier 4 — Local Node Telemetry**:
   - Generates the authoritative local machine record with live instance PIDs, active prompt counts, and local IP.
   - Guaranteed to succeed even in fully offline, isolated, or air-gapped environments.

### 4.2 Deduplication & Merge Strategy

When nodes are discovered across multiple tiers, they are merged into a unified `FleetMachineInfo` record using the following precedence rules:

```rust
pub fn merge_fleet_nodes(
    supabase_nodes: Vec<FleetMachineInfo>,
    gitmap_nodes: Vec<FleetMachineInfo>,
    sqlite_nodes: Vec<FleetMachineInfo>,
    local_node: FleetMachineInfo,
) -> Vec<FleetMachineInfo>
```

- **Identity Key**: Nodes are correlated by `node_id` or normalized `ip_address` / `node_alias`.
- **Attribute Fusion**:
  * **Status & Reachability**: Live TCP probe from Tier 2 takes precedence for real-time reachability (`is_online`, `latency_ms`).
  * **Instance Profiles & Leases**: Tier 1 (Supabase) data is preserved when available; if absent, Tier 2 SSH queries populate active instances.
  * **Source Label**: Set to `"hybrid"` if present in both Supabase and GitMap SSH, `"gitmap-ssh"` if discovered via GitMap, `"supabase"` if cloud-only, and `"local"` for the current machine.

---

## 5. GitMap Configuration Aliasing & Bidirectional Self-Influence

AGM and GitMap must form a symbiotic, self-influencing relationship where configuration, aliases, and node topology changes flow bidirectionally without manual intervention.

### 5.1 Configuration Discovery & Candidate Paths

AGM discovers GitMap node configurations through a prioritized resolution ladder:
1. Workspace root: `./gitmap-ssh-nodes.json`
2. Workspace root: `./gitmap-ssh.json`
3. Home directory: `~/.gitmap/gitmap-ssh-nodes.json`
4. Home directory: `~/.gitmap/ssh-nodes.json`
5. Repo secrets directory: `../repo-secrets/01-gitmap/gitmap-ssh-nodes.json`
6. OpenSSH User Config: `~/.ssh/config`

### 5.2 Variable Interpolation Engine

When reading `gitmap-ssh-nodes.json`, AGM dynamically interpolates configuration variables:
- `${workDir}`: Resolves to the active workspace directory (e.g., `D:\work`).
- `${repoDir}`: Resolves to `${workDir}\gitmap`.
- `${secretsDir}`: Resolves to the secrets directory.

### 5.3 Bidirectional Self-Influence Protocol

```
+---------------------------------------------------------------------------------+
|                               Bidirectional Sync                                |
|                                                                                 |
|   +-----------------------+                         +-----------------------+   |
|   |        GitMap         |                         |  Antigravity Manager  |   |
|   |                       |                         |         (AGM)         |   |
|   |   gitmap nodes add    |  ===================>   |  Inherits node alias, |   |
|   |   gitmap ssh join     |   Auto-sync on startup  |  IP, OS, and keys     |   |
|   |                       |   & manual refresh      |  into ssh_nodes.json  |   |
|   |                       |                         |                       |   |
|   |   gitmap-ssh-nodes    |  <===================   |  Deploys encrypted    |   |
|   |   .json (Envelope)    |   Export & SSH Key sync |  RSA bundles & seeds  |   |
|   +-----------------------+                         +-----------------------+   |
+---------------------------------------------------------------------------------+
```

1. **GitMap Enrolls Node → AGM Inherits**:
   - When an operator runs `gitmap nodes add` or `gitmap ssh join` in GitMap, GitMap writes the new node to `gitmap-ssh-nodes.json`.
   - On startup or manual refresh, AGM's `import_ssh_from_gitmap()` detects the update, decrypts/normalizes the records, and populates AGM's internal store without requiring UI configuration.
2. **AGM Generates Keys → GitMap Consumes**:
   - When AGM generates an SSH keypair in `~/.agm/ssh/`, it deploys the public key to GitMap's managed key index, allowing seamless SSH commands from both tools.

---

## 6. GitMap-Parity Navigation Commands Specification

AGM CLI (`bin/agm`) is extended with four primary navigation subcommands matching GitMap's terminal workflow:

### 6.1 `agm fleet nodes`

Aggregates local node, GitMap SSH mesh nodes, and Supabase peers into a unified terminal overview.

- **Usage**:
  ```bash
  agm fleet nodes [--json] [--online] [--all]
  ```
- **CLI Table Output**:
  ```
  AGM Fleet Nodes:
    ALIAS   ROLE     HOST (IP:PORT)     USER           STATUS       SOURCE      LATENCY  INSTANCES
    ------------------------------------------------------------------------------------------------
    main    worker   192.168.1.20:22    administrator  ● online     local       0ms      2 active
    w1      worker   192.168.1.3:22     Administrator  ● online     gitmap-ssh  2ms      1 active
    w2      worker   192.168.1.7:22     Administrator  ○ offline    gitmap-ssh  timeout  —
    w3      worker   192.168.1.12:22    Administrator  ○ offline    gitmap-ssh  timeout  —
    w4      worker   192.168.1.13:22    Administrator  ○ offline    gitmap-ssh  timeout  —
    u1      worker   192.168.1.22:22    a              ○ offline    gitmap-ssh  timeout  —

    Total: 6 node(s) | 2 online, 4 offline
  ```
- **Machine-Readable JSON Output (`--json`)**: Emits `CliEnvelope::ok("fleet nodes", ...)` with full node schemas.

### 6.2 `agm fleet ping`

Executes concurrent TCP and ICMP reachability checks across targeted nodes with exact latency measurement.

- **Usage**:
  ```bash
  agm fleet ping [alias|all] [--timeout <ms>] [--json]
  ```
- **Examples**:
  ```bash
  agm fleet ping w1
  # [PONG] Node 'w1' (192.168.1.3:22) is REACHABLE - Latency: 2ms

  agm fleet ping all
  # [PING] Probing 6 cluster nodes concurrently (timeout: 1500ms)...
  #   ● main (192.168.1.20:22): 0ms
  #   ● w1   (192.168.1.3:22):  2ms
  #   ○ w2   (192.168.1.7:22):  unreachable (connection timed out)
  #   ○ w3   (192.168.1.12:22): unreachable (connection timed out)
  #   ○ w4   (192.168.1.13:22): unreachable (connection timed out)
  #   ○ u1   (192.168.1.22:22): unreachable (connection timed out)
  ```

### 6.3 `agm fleet exec`

Dispatches a remote command across one or all reachable nodes via SSH manager, streaming output back to the terminal.

- **Usage**:
  ```bash
  agm fleet exec <alias|all> "<command>" [--json]
  ```
- **Examples**:
  ```bash
  agm fleet exec w1 "agm instances --json"
  agm fleet exec all "uptime"
  ```

### 6.4 `agm fleet instances`

Interrogates remote Antigravity Manager instances over SSH, returning active profiles, models, and running prompt status.

- **Usage**:
  ```bash
  agm fleet instances <alias> [--json]
  ```
- **Output**:
  ```
  Remote Instances on 'w1' (192.168.1.3):
    ID          NAME             PID     STATUS   MODEL                 BOUND EMAIL
    ----------------------------------------------------------------------------------------
    worker-01   Worker Profile 1 14220   RUNNING  gemini-2.5-pro        dev1@example.com
  ```

---

## 7. UI Instance Tab Remote Matrix Specification

### 7.1 Component Integration Point (`src/pages/Instances.tsx`)

The remote fleet matrix renders directly beneath local instances, separated by a subtle divider:

```tsx
{/* Local Instances Grid / Table */}
<InstanceTable ... />

{/* Multi-Machine Remote Matrix */}
<div className="mt-8 pt-6 border-t border-slate-200/60 dark:border-slate-800/80">
    <FleetMachinesTable />
</div>
```

### 7.2 Enriched Table Columns & Visual Primitives

`src/components/instances/FleetMachinesTable.tsx` and `src/components/instances/fleet/FleetMachineRow.tsx` are updated to display:

1. **Machine & Status**:
   - Node Alias with leader/worker indicator.
   - Status Indicator: Emerald pulsing dot (`ONLINE`) or Slate dot (`OFFLINE`).
   - OS Badge: Windows / Linux / macOS glyph.
   - Node ID hash (`#w1-192-168-1-3`).
2. **Discovery Source Badge**:
   - `GitMap SSH`: Cyan badge (`bg-cyan-500/15 text-cyan-400 border-cyan-500/30`).
   - `Supabase`: Purple badge (`bg-purple-500/15 text-purple-400 border-purple-500/30`).
   - `Hybrid`: Emerald badge (`bg-emerald-500/15 text-emerald-400 border-emerald-500/30`).
3. **IP Address & Latency**:
   - Monospace IP address with one-click copy button.
   - Live round-trip latency pill (e.g., `2ms` in emerald, `>100ms` in amber).
4. **Running Instance(s)**:
   - Interactive profile chips displaying profile name and bound model.
   - Chip click navigates to prompt tree or remote prompt dispatch modal.
5. **Bound Account Email(s)**:
   - Masked email addresses (`d***1@example.com`) with row-level eye icon toggle for unmasking.
6. **Action Controls (Segmented Pill Capsule)**:
   - Contiguous capsule (`rounded-full bg-slate-800/60 border border-slate-700/60 divide-x divide-slate-700/60`):
     * `[Ping]`: Triggers instantaneous TCP probe and refreshes latency badge.
     * `[SSH]`: Launches native shell session or copies SSH connection string (`ssh Administrator@192.168.1.3`).
     * `[Inspect]`: Opens remote instance inspection drawer.

---

## 8. Actionable AI Instructions for Future Implementation

Future AI agents implementing this specification must execute the following sequential checklist without deviating from established invariants:

### Phase 1: Core SSH Manager GitMap Command & Deserialization Fix
- [ ] In `src-tauri/src/modules/ssh_manager/sync.rs:173-195`:
  * Replace invalid arguments `&["ssh", "nodes", "export-json"]` and `&["ssh", "nodes", "--json"]` with `&["nodes", "--json"]`.
  * Ensure process spawning applies `0x08000000` (CREATE_NO_WINDOW) on Windows to prevent console flashing.
- [ ] In `src-tauri/src/modules/ssh_manager/types.rs`:
  * Add `#[serde(alias = "host")]` to `SshNodeExportItem.ip_address` and `SshConnectionRecord.ip_address`.
  * Add `#[serde(alias = "user")]` to `SshNodeExportItem.username` and `SshConnectionRecord.username`.
  * Add `#[serde(default)]` to optional GitMap fields (`role`, `status`, `subsystems`, `is_online`).
- [ ] In `src-tauri/src/modules/ssh_manager/connections.rs`:
  * Update `decode_connections_from_json()` to handle GitMap root arrays containing `host` and `user` keys.

### Phase 2: Unified 4-Tier Fleet Query Engine
- [ ] In `src-tauri/src/modules/supabase_sync/query.rs`:
  * Refactor `query_fleet_machines()` into a 4-Tier resolver:
    1. Check Tier 1 (Supabase).
    2. Concurrently query Tier 2 (`ssh_manager::load_ssh_connections()`, checking TCP reachability with `1500ms` timeout).
    3. Query Tier 3 (`fleet_sync.db`).
    4. Construct Tier 4 (Local node).
  * Merge all discovered nodes into a unified `Vec<FleetMachineInfo>` using `merge_fleet_nodes()`.
  * Ensure that if Supabase is disabled or fails, Tier 2 SSH nodes are returned rather than an empty list.
- [ ] In `src-tauri/src/modules/supabase_sync/migration.rs`:
  * Extend `build_local_fallback_machine()` to include Tier 2 GitMap SSH nodes when Supabase is disabled.

### Phase 3: Frontend Resilience & Multi-Source Store
- [ ] In `src/components/instances/fleet/useFleetMachines.ts`:
  * Remove the short-circuit `if (!syncActive) { setMachines([]); return; }`.
  * Set `isSyncEnabled` to represent general cluster availability rather than strictly Supabase.
  * Update filtering logic: Only filter out the local machine if `machines.length > 1`; retain remote nodes discovered via GitMap SSH.
- [ ] In `src/components/instances/fleet/fleetTypes.ts`:
  * Add fields to `FleetMachineInfo`:
    * `source?: 'supabase' | 'gitmap-ssh' | 'hybrid' | 'local'`
    * `latency_ms?: number | null`
    * `role?: string`
    * `status?: string`

### Phase 4: UI Remote Machine Matrix & Action Controls
- [ ] In `src/components/instances/FleetMachinesTable.tsx`:
  * Update table headers to include `Source`, `Host (IP:Port)`, `Latency`, and `Actions`.
  * Replace `FleetDisabledState` with a non-blocking informational banner if Supabase is offline but SSH nodes are active.
- [ ] In `src/components/instances/fleet/FleetMachineRow.tsx`:
  * Render Source Badge (`GitMap SSH` cyan / `Supabase` purple / `Hybrid` emerald).
  * Render latency pill next to IP address.
  * Render segmented pill capsule with `Ping`, `SSH`, and `Inspect` buttons.
- [ ] In `src/components/instances/fleet/FleetTableStates.tsx`:
  * Update `FleetEmptyState` copy to guide users toward adding SSH nodes or launching GitMap.

### Phase 5: CLI Navigation Suite Parity
- [ ] In `src-tauri/src/bin/agm/fleet_cmds.rs`:
  * Rewrite `cmd_fleet_nodes()` to invoke the 4-Tier resolver and render an ASCII table matching GitMap CLI.
  * Implement `cmd_fleet_ping()` to execute concurrent TCP checks across `alias` or `all`.
  * Implement `cmd_fleet_exec()` to dispatch remote SSH commands via `ssh_manager::exec`.
  * Implement `cmd_fleet_instances()` to query remote instances over SSH.

### Phase 6: Targeted Test Verification & Regression Shielding
- [ ] In `src-tauri/src/modules/ssh_manager/tests.rs`:
  * Add unit test `test_gitmap_nodes_json_deserialization()` verifying that raw `gitmap nodes --json` output deserializes into `SshConnectionRecord` with correct `ip_address` and `username`.
- [ ] In `src-tauri/src/modules/supabase_sync/tests.rs`:
  * Add unit test `test_query_fleet_machines_fallback_includes_ssh_nodes()` verifying that when Supabase config is missing, GitMap SSH nodes are returned.
- [ ] In `src/components/instances/__tests__/fleetMachinesTable.test.ts`:
  * Add tests asserting remote nodes are rendered when Supabase sync is disabled.

---

## 9. Verification & Quality Gates

The implementation must pass the following pre-flight checks:
1. `cd src-tauri && cargo fmt -- --check`
2. `cd src-tauri && cargo clippy --all-targets --all-features`
3. `npm run build`
4. CLI execution verification:
   * `cargo run --bin agm -- fleet nodes` outputs both `main` and `w1`.
   * `cargo run --bin agm -- fleet ping w1` reports `[PONG] Node 'w1' is REACHABLE`.
