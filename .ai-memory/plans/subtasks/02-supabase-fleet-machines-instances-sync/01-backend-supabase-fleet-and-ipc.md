---
plan: 02-supabase-fleet-machines-instances-sync
subtask: "01"
title: Backend Supabase Fleet Sync, Schema Extension, Aggregation Engine, and Tauri IPC Commands
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/02-supabase-fleet-machines-instances-sync/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/supabase_schema.rs
  - src-tauri/src/modules/supabase_sync.rs
  - src-tauri/src/commands/supabase.rs
  - src-tauri/src/lib.rs
status: pending
---

# Subtask 01: Backend Supabase Fleet Sync, Schema Extension, Aggregation Engine, and Tauri IPC Commands

## 1. Context & Motivation
Antigravity Manager coordinates multi-instance desktop environments running agentic workflows. When multiple machines or workers (e.g. `Worker-1`, `Worker-2`, local desktop) connect to a shared Supabase root database, operators need visibility into all fleet machines, what instances are configured, what account emails are actively bound, and how many agent prompts are actively running.

Currently:
1. `public.instance_profiles` in Supabase lacks a `running_prompts_count` field.
2. In `src-tauri/src/modules/supabase_sync.rs`, the heartbeat routine (`send_heartbeat`) registers instances but does not query `repo_db::discover_running_prompts_from_antigravity(&inst.id)` to report running prompt activity.
3. There is no aggregation engine that queries `nodes`, `instance_profiles`, and `workspace_leases` to build a unified list of `FleetMachineInfo` objects, nor is there a reliable local machine fallback if Supabase is offline or unseeded.
4. There are no Tauri IPC commands exposing fleet status (`get_supabase_fleet_machines`) or manual synchronization (`sync_supabase_now`) to the frontend.

This subtask implements the backend data structures, database schema extension, telemetry collection, aggregation logic, and Tauri IPC commands to satisfy these requirements.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Actions |
| :--- | :--- |
| `src-tauri/src/modules/supabase_schema.rs` | Update `ROOT_DB_SCHEMA_SQL` on table `public.instance_profiles` to add `running_prompts_count INT NOT NULL DEFAULT 0`. Add idempotent migration statement. |
| `src-tauri/src/modules/supabase_sync.rs` | 1. Define `FleetInstanceItem` and `FleetMachineInfo` structs with serde derives.<br/>2. Update `send_heartbeat()` and `build_sync_payloads()` to query `repo_db::discover_running_prompts_from_antigravity(&inst.id)` and upsert `running_prompts_count`.<br/>3. Implement `fetch_fleet_machines() -> AppResult<Vec<FleetMachineInfo>>` joining `nodes`, `instance_profiles`, and `workspace_leases`, with guaranteed local machine injection. |
| `src-tauri/src/commands/supabase.rs` | Implement `get_supabase_fleet_machines() -> AppResult<Vec<FleetMachineInfo>>` and `sync_supabase_now() -> AppResult<()>`. |
| `src-tauri/src/lib.rs` | Register `commands::get_supabase_fleet_machines` and `commands::sync_supabase_now` in `generate_handler![]`. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Extend Supabase Schema in `src-tauri/src/modules/supabase_schema.rs`

1. In `ROOT_DB_SCHEMA_SQL`, update the `CREATE TABLE IF NOT EXISTS public.instance_profiles` definition:
   ```sql
   -- 2. Instance Profiles Status Table (Child of Nodes)
   CREATE TABLE IF NOT EXISTS public.instance_profiles (
       id TEXT PRIMARY KEY,
       node_id TEXT NOT NULL REFERENCES public.nodes(id) ON DELETE CASCADE,
       profile_name TEXT NOT NULL,
       active_account_id TEXT NOT NULL DEFAULT '',
       active_account_email TEXT NOT NULL DEFAULT '',
       is_active BOOLEAN NOT NULL DEFAULT false,
       quota_percent INT NOT NULL DEFAULT 100,
       status TEXT NOT NULL DEFAULT 'idle',
       running_prompts_count INT NOT NULL DEFAULT 0,
       updated_at BIGINT NOT NULL DEFAULT 0
   );
   ```
2. Below table creation, append an idempotent column addition to ensure existing databases receive the new column without recreating the table:
   ```sql
   ALTER TABLE public.instance_profiles ADD COLUMN IF NOT EXISTS running_prompts_count INT NOT NULL DEFAULT 0;
   ```
3. Update unit tests in `supabase_schema.rs` to assert that `running_prompts_count` is present in `get_schema_sql("root")`.

---

### 3.2 Step 2: Telemetry Discovery & Heartbeat Augmentation in `src-tauri/src/modules/supabase_sync.rs`

1. In `send_heartbeat(endpoint: &SupabaseEndpoint, node_alias: &str) -> Result<(), AppError>`:
   - For each instance `inst` in `instances_to_sync`:
     - Query in-flight prompts using `crate::modules::repo_db::discover_running_prompts_from_antigravity(&inst.id)`.
     - Count active prompts: `let running_prompts_count = repo_db::discover_running_prompts_from_antigravity(&inst.id).len() as i32;`
     - Include `"running_prompts_count": running_prompts_count` in `profile_payload`.
2. In `build_sync_payloads(...)`:
   - Compute `running_prompts_count` and add it to `profile_payload`.

---

### 3.3 Step 3: Implement Aggregation Engine in `src-tauri/src/modules/supabase_sync.rs`

1. Declare the data transfer objects:
   ```rust
   /// Individual instance status item within a fleet machine
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct FleetInstanceItem {
       pub instance_id: String,
       pub profile_name: String,
       pub active_account_id: String,
       pub active_account_email: String,
       pub is_active: bool,
       pub quota_percent: i32,
       pub status: String,
       pub running_prompts_count: i32,
       pub updated_at: i64,
       pub lease_expires_at: Option<i64>,
   }

   /// Aggregated node / machine status in the multi-machine fleet
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct FleetMachineInfo {
       pub node_id: String,
       pub alias: String,
       pub ip_address: String,
       pub uptime_seconds: u64,
       pub last_heartbeat_at: i64,
       pub is_online: bool,
       pub is_local: bool,
       pub instances: Vec<FleetInstanceItem>,
       pub total_running_prompts: i32,
   }
   ```

2. Implement `pub async fn fetch_fleet_machines() -> Result<Vec<FleetMachineInfo>, AppError>`:
   - **Local Machine Synthesis**:
     - Read local node ID via `get_local_node_id()`.
     - Read local IP via `get_local_ip()`.
     - Read uptime via `get_uptime_seconds()`.
     - Load config via `load_config().unwrap_or_default()`, getting `node_alias`.
     - Load local instances via `instance::load_registry()`. For each instance, populate `FleetInstanceItem`, querying `repo_db::discover_running_prompts_from_antigravity(&inst.id).len() as i32`.
     - Compute `total_running_prompts` = sum of instance running prompt counts.
     - Construct `local_machine` with `is_local = true` and `is_online = true`.
   - **Endpoint Check**:
     - Locate an enabled endpoint with `role == "root"`.
     - If none exists or `is_sync_enabled == false`: Return `Ok(vec![local_machine])`.
   - **Remote Querying**:
     - Instantiate `SupabaseClient::new(root_ep)`.
     - Perform queries:
       ```rust
       let nodes_val = client.select("nodes", "select=*").await.unwrap_or(serde_json::Value::Array(vec![]));
       let profiles_val = client.select("instance_profiles", "select=*").await.unwrap_or(serde_json::Value::Array(vec![]));
       let leases_val = client.select("workspace_leases", "select=*").await.unwrap_or(serde_json::Value::Array(vec![]));
       ```
   - **Joining & Aggregation**:
     - Index `instance_profiles` by `node_id`.
     - Index `workspace_leases` by `node_id` or `account_id` to correlate lease expiration timestamps.
     - For each row in `nodes_val`:
       - If `node_id == local_node_id`: inject or update with local real-time `local_machine`.
       - If `node_id != local_node_id`:
         - Determine `is_online = (now - last_heartbeat_at) <= 180`.
         - Parse `FleetInstanceItem` list from associated `instance_profiles`, mapping `running_prompts_count` (defaulting to 0).
         - Calculate `total_running_prompts`.
         - Build `FleetMachineInfo` with `is_local = false`.
     - If the remote query did not contain `local_machine` (e.g. initial launch before first heartbeat), prepend `local_machine` at index 0.
     - Sort fleet: Local machine first (`is_local = true`), followed by remote nodes ordered by `alias`.
     - Return `Ok(fleet)`.

---

### 3.4 Step 4: Implement Tauri IPC Commands in `src-tauri/src/commands/supabase.rs`

1. Expose `get_supabase_fleet_machines`:
   ```rust
   #[tauri::command]
   pub async fn get_supabase_fleet_machines() -> AppResult<Vec<supabase_sync::FleetMachineInfo>> {
       supabase_sync::fetch_fleet_machines().await
   }
   ```
2. Expose `sync_supabase_now`:
   ```rust
   #[tauri::command]
   pub async fn sync_supabase_now() -> AppResult<()> {
       supabase_sync::trigger_manual_sync().await
   }
   ```

---

### 3.5 Step 5: Register IPC Commands in `src-tauri/src/lib.rs`

1. Locate the `generate_handler![]` macro in `src-tauri/src/lib.rs`.
2. Under the `// Supabase Cross-Node Synchronization commands` section, add:
   ```rust
   commands::get_supabase_fleet_machines,
   commands::sync_supabase_now,
   ```

---

## 4. Resilience & Fallback Guarantees

1. **Zero Frontend Failure on Offline / Unconfigured State**:
   If Supabase is unconfigured, credentials are missing, or network requests time out, `fetch_fleet_machines()` must NEVER return an `Err` that breaks the Instances UI. It must log a warning and return `Ok(vec![local_machine])`.
2. **Local Machine Real-Time Accuracy**:
   Even if the remote Supabase database has not yet received a heartbeat from the local node, the local node is always synthesized using live in-memory registry and core storage data, guaranteeing instant feedback.
3. **Graceful Deserialization**:
   When parsing PostgREST JSON responses, handle potential missing fields or null values (such as `running_prompts_count` on legacy records) with `.unwrap_or(0)` or `serde(default)`.

---

## 5. Coding Guidelines & Invariants Checklist

- [x] **Strictly Relative Paths**: All paths cited are relative to the repository root (e.g. `src-tauri/src/modules/supabase_sync.rs`).
- [x] **Positive Boolean Fields**:
  - `is_active` (never `inactive`)
  - `is_online` (never `offline`)
  - `is_local` (never `remote` or `non_local`)
- [x] **AppResult Return Types**: All commands return `AppResult<T>`.
- [x] **Zero Raw Grep**: Target files and symbols are explicitly defined.
- [x] **No Git Execution**: Implementation does not run `git` operations.
- [x] **Strict Subtask Boundary**: Does not touch frontend files (`src/` or `02-component-spec.md`).

---

## 6. Acceptance Criteria & Pre-flight Testing Protocol

| ID | Test Scenario | Acceptance Criteria |
| :--- | :--- | :--- |
| **AC-BE-01** | Database Schema Inspection | `get_supabase_schema_sql("root")` contains `running_prompts_count INT NOT NULL DEFAULT 0` and the idempotent `ALTER TABLE` statement. |
| **AC-BE-02** | Heartbeat Telemetry Serialization | When `send_heartbeat()` executes with active prompts, `running_prompts_count` is correctly serialized into the `instance_profiles` upsert payload. |
| **AC-BE-03** | Local Fallback Execution | Calling `get_supabase_fleet_machines` with sync disabled returns 1 item with `is_local == true`, `is_online == true`, and accurate local prompt counts. |
| **AC-BE-04** | Multi-Machine Aggregation | Calling `get_supabase_fleet_machines` with active remote nodes returns properly grouped `FleetMachineInfo` entries with local machine at index 0. |
| **AC-BE-05** | IPC Registration | `invoke('get_supabase_fleet_machines')` and `invoke('sync_supabase_now')` resolve successfully from the frontend without "command not found" errors. |
| **AC-BE-06** | Pre-flight Quality Gates | `cargo clippy --all-targets --all-features` and `cargo fmt -- --check` execute with zero warnings or errors. |
