# Plan: Supabase Cross-Node Synchronization & Split-DB Architecture

## Status: Completed

## User Request Summary
The user requested verification of prior fixes and the implementation of a distributed, cross-node synchronization architecture using Supabase under its 500 MB free-tier budget, following the Split-DB pattern:
1. **Root DB:** Node registry (machine ID, alias, IP, uptime, project count), instance profiles (running, active, queued), and global workspace/account leases with TTL to prevent multi-node collision. Zero prompts or garbage data in root DB.
2. **Secondary Split DB(s):** Prompts, command queues, and terminal/PowerShell instructions, with multi-endpoint overflow chaining and cascading failover.
3. **500 MB Free-Tier Cap & Auto-Pruning:** Configurable soft-caps (default 400 MB root, 200 MB secondary) with background FIFO auto-pruning and selective live data migration.
4. **Reversible Credential Obfuscation:** Export/import database settings in JSON and YAML with API keys obfuscated via multi-round Base64 encoding with embedded iteration count (e.g. `4:<data>` or `4$<data>`).
5. **Multi-Endpoint UI & Migration Workflow:** UI and Tauri IPC commands to register endpoints, test connections, verify tables, view schema DDL, and migrate data.
6. **Distributed Collision Prevention:** Prevent two machines from selecting the same account or workspace concurrently.
7. **SQLite vs Supabase Clarification:** Address whether Supabase can have SQLite-type DB for cross-platform/cross-machine communication.

## Architectural Clarification: Supabase vs SQLite
- **Can Supabase have SQLite?**
  - Supabase is a cloud-hosted **PostgreSQL** service and does not run SQLite natively on its servers.
  - However, our architecture adopts a **Hybrid Split-DB Model**:
    1. **Local Nodes (SQLite):** Each local Antigravity-Manager node uses modular SQLite databases (`accounts.db`, `proxy.db`, `tasks.db`, etc.) for zero-latency local operations and offline resilience.
    2. **Cloud Coordination (Supabase PostgREST):** Cross-node synchronization occurs over lightweight, stateless HTTP PostgREST calls (`/rest/v1/...`) to Supabase. This avoids PostgreSQL TCP connection pool limits on the free tier.
    3. **Free-Tier Protection:** By storing lightweight metadata in the Root DB and routing high-frequency prompts to Secondary DBs with automatic FIFO pruning, storage remains well within the 500 MB free tier.

## Completed Tasks & Deliverables

### Task-01: Root DB Architecture & Schema Specification
- **Files:** `src-tauri/src/modules/supabase_schema.rs`, `src-tauri/src/modules/supabase_sync.rs`, `src-tauri/src/modules/workspace_lease_manager.rs`
- **Deliverables:**
  - Defined `nodes` table for machine registration and heartbeat telemetry.
  - Defined `instance_profiles` table tracking active, running, and queued profiles.
  - Defined `workspace_leases` table and atomic PL/pgSQL stored procedure `acquire_workspace_lease` enforcing lease TTL and preventing concurrent account usage.
  - Heartbeat worker strictly updates node telemetry without storing prompt or command data in the Root DB.

### Task-02: Secondary Split DBs (Prompt & Command Queues)
- **Files:** `src-tauri/src/modules/supabase_command_queue.rs`, `src-tauri/src/modules/supabase_schema.rs`
- **Deliverables:**
  - Defined `command_queue` (`id`, `target_node_id`, `source`, `command_text`, `status`, `timestamps`).
  - Defined `command_telemetry` (`id`, `command_id`, `node_id`, `stdout`, `stderr`, `exit_code`).
  - Implemented cascading failover across priority-sorted secondary endpoints.
  - Implemented safe process execution wrapper (`powershell` on Windows, `sh` on Linux/macOS) with hidden window execution.

### Task-03: Free-Tier Cap Enforcement & Auto-Pruning Engine
- **Files:** `src-tauri/src/modules/supabase_pruner.rs`, `src-tauri/src/modules/supabase_sync.rs`
- **Deliverables:**
  - Configurable soft-caps: default 400 MB for Root DB, 200 MB for Secondary DBs.
  - Background daemon (`start_pruner_worker`) executing every 5 minutes to prune completed/failed commands in FIFO order and delete expired workspace leases.
  - Cross-database live migration pipeline (`migrate_database_data`) transferring active nodes, running profiles, unexpired leases, and recent commands to a fresh endpoint.

### Task-04: Multi-Iteration Base64 Hash-Encoding for Credentials
- **Files:** `src-tauri/src/modules/iterative_codec.rs`, `src-tauri/src/commands/supabase.rs`
- **Deliverables:**
  - Implemented multi-round Base64 encoding (`encode_iterative`) with iteration count prefix.
  - Supported both `N:<data>` and `N$<data>` prefixes in `decode_iterative`.
  - Added clean YAML serialization (`format_yaml_bundle`) and parser (`parse_yaml_bundle`) alongside JSON for export/import.
  - Enforced short functions (<= 8-15 lines) and strict boolean principles.

### Task-05: Settings Management, Multi-Endpoint UI & Migration Workflow
- **Files:** `src/components/settings/SupabaseSyncSettings.tsx`, `src/services/supabaseService.ts`, `src-tauri/src/commands/supabase.rs`
- **Deliverables:**
  - Multi-endpoint management UI (add, edit, test, delete, toggle, priority).
  - Interactive connection health checks and table verification badges.
  - Schema viewer modal providing copyable DDL for Root and Secondary databases.
  - Migration modal enabling one-click selective state transfer between endpoints.
  - Export/Import modal supporting JSON and YAML formats with custom iteration rounds.

### Task-06: Global State Synchronization & Remote Control Integration
- **Files:** `src/components/accounts/AccountTable.tsx`, `src-tauri/src/modules/workspace_lease_manager.rs`
- **Deliverables:**
  - Account table displays active remote leases and owner node aliases.
  - Account switching checks remote lease lock before switching, preventing dual-node conflict.
  - Local node telemetry (`node_id`, `node_alias`, `ip_address`, `uptime_seconds`) exposed via Tauri IPC.

## Verification
- Codebase inspected and verified:
  - All Rust modules compile cleanly and adhere to boolean rules.
  - All TypeScript services and components correctly typed and linked to Tauri IPC commands.
  - Strict relative git paths enforced across all plans and documentation.
