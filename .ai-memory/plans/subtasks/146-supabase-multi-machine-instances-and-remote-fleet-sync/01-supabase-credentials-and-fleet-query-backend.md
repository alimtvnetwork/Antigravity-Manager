---
plan: 146-supabase-multi-machine-instances-and-remote-fleet-sync
subtask: "01"
title: Supabase Credentials Auto-Discovery & Multi-Node Fleet Query Aggregation
domain: backend/sync
target_files:
  - src-tauri/src/modules/supabase_sync.rs
  - src-tauri/src/modules/supabase_client.rs
  - src-tauri/src/modules/supabase_schema.rs
status: pending
---

# 01 — Supabase Credentials Auto-Discovery & Multi-Node Fleet Query Aggregation

## 1. Objective

Implement the backend core for multi-machine fleet state aggregation across the cluster using Supabase Root DB. This includes:
1. Verifying and activating automated credentials harvesting from local `repo-secrets` paths into `supabase_config.json`.
2. Defining relational data models (`FleetMachineInfo`, `FleetInstanceSummary`, `FleetLeaseInfo`) in `src-tauri/src/modules/supabase_sync.rs`.
3. Implementing the aggregation engine `query_fleet_machines()` which queries `public.nodes`, `public.instance_profiles`, and `public.workspace_leases` from the Supabase Root DB.
4. Implementing resilient fallback logic ensuring that if Supabase sync is disabled, offline, or unreachable, the system gracefully falls back to reporting local machine telemetry without throwing errors.

---

## 2. Technical Scope & Implementation Specifications

### 2.1 Repo-Secrets Verification & Auto-Discovery

The credentials auto-discovery subsystem in `src-tauri/src/modules/supabase_sync.rs` handles auto-seeding endpoints from `repo-secrets`.

1. **Candidate Search Paths:**
   Verify discovery against:
   - `D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json`
   - `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json`
   - `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
   - `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
   - `../repo-secrets/...` and `../../repo-secrets/...` relative paths.
2. **Payload Parsing & Normalization:**
   - Unpack JSON envelope metadata (stripping UTF-8 BOM `\u{feff}`).
   - Base64-decode obfuscated keys if `encoding_format: base64` is declared.
   - Clean URLs using `normalize_supabase_url()` (stripping trailing `/rest/v1` and slashes).
   - Ensure the primary endpoint receives `role: "root"`, `priority: 1`, and `prune_threshold_mb: 400`.
3. **Trigger Invariant:**
   - Ensure `auto_discover_supabase_credentials()` automatically sets `is_sync_enabled = true`, saves the configuration to `supabase_config.json`, starts the sync background worker via `start_sync_worker()`, and returns the refreshed configuration.

### 2.2 Relational Data Models

Define the fleet models in `src-tauri/src/modules/supabase_sync.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Summary of an IDE instance registered on a fleet node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInstanceSummary {
    pub profile_id: String,
    pub profile_name: String,
    pub is_running: bool,
    pub bound_account_id: Option<String>,
    pub bound_account_email: Option<String>,
}

/// Active account lease held by a node in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetLeaseInfo {
    pub account_id: String,
    pub account_email: String,
    pub profile_name: String,
    pub leased_at: i64,
    pub expires_at: i64,
    pub is_expired: bool,
}

/// Comprehensive machine info aggregated across nodes, profiles, and leases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetMachineInfo {
    pub node_id: String,
    pub node_alias: String,
    pub os_info: Option<String>,
    pub ip_address: String,
    pub is_online: bool,
    pub last_heartbeat_timestamp: i64,
    pub uptime_seconds: u64,
    pub in_flight_prompts_count: usize,
    pub active_instances: Vec<FleetInstanceSummary>,
    pub bound_emails: Vec<String>,
    pub leases: Vec<FleetLeaseInfo>,
    pub is_local: bool,
}
```

### 2.3 Aggregation Function: `query_fleet_machines()`

Implement `pub async fn query_fleet_machines() -> Result<Vec<FleetMachineInfo>, AppError>` in `src-tauri/src/modules/supabase_sync.rs`:

1. **Config Validation:**
   - Load `SupabaseConfig` via `load_config()`.
   - Check `config.is_sync_enabled`. If `false`, return local fallback machine immediately.
   - Find enabled endpoint with `role == "root"`. If none exists, return local fallback machine.
2. **PostgREST Multi-Table Queries:**
   - Construct `SupabaseClient::new(&root_ep)?`.
   - `nodes_val = client.select("nodes", "select=*&order=last_heartbeat_at.desc").await?`
   - `profiles_val = client.select("instance_profiles", "select=*").await?`
   - `leases_val = client.select("workspace_leases", "select=*").await?`
3. **Data Transformation & Merging:**
   - Deserialize rows into typed intermediate vectors or JSON arrays.
   - Index instance profiles by `node_id` into a map `HashMap<String, Vec<FleetInstanceSummary>>`.
     * `is_running = p.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false) || p.get("status").and_then(|v| v.as_str()).map_or(false, |s| s == "running")`.
   - Index workspace leases by `node_id` into a map `HashMap<String, Vec<FleetLeaseInfo>>`.
     * `is_expired = lease.expires_at < now`.
   - Iterate over nodes:
     * Evaluate liveness: `is_online = (now - last_heartbeat_at) < 180`.
     * Detect local host: `is_local = (node_id == local_node_id)`.
     * In-flight prompts count: extract `running_prompts_count` from node row. If `is_local`, read the live local SQLite count for maximum freshness.
     * Extract unique bound emails: collect from both `node_profiles.bound_account_email` and `node_leases.account_email`, trimming and filtering empty strings.
4. **Deterministic Sorting Invariant:**
   - The local node (`is_local == true`) is always sorted at index 0.
   - Remote online machines are sorted next (alphabetical by `node_alias`).
   - Remote offline machines are sorted last (alphabetical by `node_alias`).

### 2.4 Resilient Fallback Handling

Implement `pub fn build_local_fallback_machine() -> FleetMachineInfo`:

1. Read local node credentials:
   - `node_id = get_local_node_id()`
   - `node_alias = load_config().map(|c| c.node_alias).unwrap_or("Node-Local".into())`
   - `ip_address = get_local_ip()`
   - `uptime_seconds = get_uptime_seconds()`
   - `last_heartbeat_timestamp = Utc::now().timestamp()`
   - `is_online = true`, `is_local = true`
2. In-flight prompts:
   - Query local SQLite `SELECT count(*) FROM active_prompts WHERE status = 'running'` (defaults to 0 on database read error).
3. Local instance profiles:
   - Read local instance registry (`crate::modules::instance::load_registry()`).
   - Map each local instance to `FleetInstanceSummary`:
     * `is_running = is_instance_running(&inst.id, &inst.data_dir, inst.pid)`
     * `bound_account_id = inst.bound_account_id`
     * `bound_account_email = inst.bound_email`
4. Assemble and return `FleetMachineInfo` wrapped in `vec![local_machine]`.

---

## 3. Deliverables & Acceptance Checklist

- [ ] `auto_discover_supabase_credentials()` properly pulls from `repo-secrets` and activates sync.
- [ ] `FleetMachineInfo`, `FleetInstanceSummary`, and `FleetLeaseInfo` structs declared and derived with `Serialize, Deserialize, Clone, Debug`.
- [ ] `query_fleet_machines()` queries `nodes`, `instance_profiles`, and `workspace_leases` from the Root DB.
- [ ] Online status accurately computed using the 180-second heartbeat window.
- [ ] Bound emails correctly aggregated and deduplicated from profiles and leases.
- [ ] Robust fallback to `build_local_fallback_machine()` when sync is disabled or Supabase is unreachable.
- [ ] Local machine guaranteed to appear at index 0 with `is_local: true`.
