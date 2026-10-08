---
plan: 146-supabase-multi-machine-instances-and-remote-fleet-sync
subtask: "02"
title: Heartbeat Prompt Telemetry Enhancement & IPC Command Registration
domain: backend/ipc
target_files:
  - src-tauri/src/modules/supabase_sync.rs
  - src-tauri/src/commands/supabase.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
  - src/services/supabaseService.ts
status: pending
---

# 02 — Heartbeat Prompt Telemetry Enhancement & IPC Command Registration

## 1. Objective

Enhance the background heartbeat mechanism in `src-tauri/src/modules/supabase_sync.rs` with live in-flight prompt count telemetry, implement the `get_fleet_machines` Tauri IPC command in `src-tauri/src/commands/supabase.rs`, register the command in `src-tauri/src/lib.rs`, and expose the typed client method in `src/services/supabaseService.ts`.

---

## 2. Technical Scope & Implementation Specifications

### 2.1 Heartbeat Live Prompt Telemetry Enhancement

#### File: `src-tauri/src/modules/supabase_sync.rs`
In `send_heartbeat(endpoint: &SupabaseEndpoint, node_alias: &str)`:
1. **Compute Live In-Flight Prompts:**
   Query the local SQLite database `active_prompts` table for records currently executing:
   ```rust
   let mut running_prompts_count = 0;
   if let Ok(conn) = crate::modules::repo_db::connect_db() {
       if let Ok(mut stmt) = conn.prepare("SELECT count(*) FROM active_prompts WHERE status = 'running'") {
           if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
               running_prompts_count = cnt;
           }
       }
   }
   ```
2. **Inject into Node Upsert Payload:**
   Update the `node_payload` JSON object:
   ```rust
   let node_payload = json!({
       "id": node_id,
       "alias": node_alias,
       "ip_address": local_ip,
       "uptime_seconds": uptime,
       "project_count": project_count,
       "running_prompts_count": running_prompts_count,
       "last_heartbeat_at": now,
       "status": "online"
   });
   ```
3. **Graceful PostgREST Column Fallback:**
   If the remote Supabase database instance has not yet applied the `running_prompts_count` column migration, PostgREST returns an HTTP 400 Bad Request error mentioning column non-existence.
   The heartbeat sender must catch this specific condition, fall back to upserting a payload without `running_prompts_count`, and log an administrative warning rather than failing the entire heartbeat worker cycle.

### 2.2 Tauri Command Implementation

#### File: `src-tauri/src/commands/supabase.rs`
1. Re-export `FleetMachineInfo`, `FleetInstanceSummary`, and `FleetLeaseInfo` from `crate::modules::supabase_sync`:
   ```rust
   pub use crate::modules::supabase_sync::{FleetInstanceSummary, FleetLeaseInfo, FleetMachineInfo};
   ```
2. Implement the IPC command:
   ```rust
   #[tauri::command]
   pub async fn get_fleet_machines() -> AppResult<Vec<FleetMachineInfo>> {
       supabase_sync::query_fleet_machines().await
   }
   ```

### 2.3 Command Registration

#### 1. Module Re-export: `src-tauri/src/commands/mod.rs`
`supabase.rs` is already re-exported in `src-tauri/src/commands/mod.rs` via:
```rust
pub mod supabase;
pub use supabase::*;
```
Verify that `get_fleet_machines` and the associated structs are public and accessible to `crate::commands::get_fleet_machines`.

#### 2. Tauri Handler Registration: `src-tauri/src/lib.rs`
Register `commands::get_fleet_machines` inside the `tauri::generate_handler![...]` macro list directly under the Supabase Cross-Node Synchronization block:
```rust
            // Supabase Cross-Node Synchronization commands
            commands::get_supabase_config,
            commands::save_supabase_config,
            commands::test_supabase_endpoint,
            commands::check_supabase_endpoint_tables,
            commands::migrate_supabase_data,
            commands::get_supabase_schema_sql,
            commands::export_supabase_config,
            commands::import_supabase_config,
            commands::acquire_account_lease,
            commands::release_account_lease,
            commands::list_active_account_leases,
            commands::get_local_node_info,
            commands::auto_discover_supabase_credentials,
            commands::get_fleet_machines,
```

### 2.4 TypeScript Service Layer Extension

#### File: `src/services/supabaseService.ts`
1. **Export Canonical Interfaces:**
   ```typescript
   export interface FleetInstanceSummary {
       profile_id: string;
       profile_name: string;
       is_running: boolean;
       bound_account_id?: string;
       bound_account_email?: string;
   }

   export interface FleetLeaseInfo {
       account_id: string;
       account_email: string;
       profile_name: string;
       leased_at: number;
       expires_at: number;
       is_expired: boolean;
   }

   export interface FleetMachineInfo {
       node_id: string;
       node_alias: string;
       os_info?: string;
       ip_address: string;
       is_online: boolean;
       last_heartbeat_timestamp: number;
       uptime_seconds: number;
       in_flight_prompts_count: number;
       active_instances: FleetInstanceSummary[];
       bound_emails: string[];
       leases?: FleetLeaseInfo[];
       is_local?: boolean;
   }
   ```
2. **Implement Service Method:**
   ```typescript
   async getFleetMachines(): Promise<FleetMachineInfo[]> {
       try {
           return await invoke<FleetMachineInfo[]>('get_fleet_machines');
       } catch (error) {
           console.warn('[SupabaseService] getFleetMachines failed:', error);
           useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
           return [];
       }
   },
   ```

---

## 3. Deliverables & Acceptance Checklist

- [ ] `send_heartbeat()` computes live `running_prompts_count` via `repo_db` and includes it in `node_payload`.
- [ ] PostgREST column missing fallback handled cleanly without breaking heartbeat cycle.
- [ ] `get_fleet_machines` Tauri command implemented in `src-tauri/src/commands/supabase.rs`.
- [ ] Command verified in `src-tauri/src/commands/mod.rs`.
- [ ] Command registered in `generate_handler!` in `src-tauri/src/lib.rs`.
- [ ] TypeScript interfaces `FleetInstanceSummary`, `FleetLeaseInfo`, `FleetMachineInfo` exported in `src/services/supabaseService.ts`.
- [ ] `supabaseService.getFleetMachines()` implemented with error boundary and empty array fallback.
