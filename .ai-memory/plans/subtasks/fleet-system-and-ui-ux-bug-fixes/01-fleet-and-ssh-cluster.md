# Subtask Plan 01: Fleet Node Identity Resolution, GitMap Node Parity & SSH Cluster Reset

- **Worker:** Worker 01
- **Task Slug:** `fleet-system-and-ui-ux-bug-fixes`
- **File:** `.ai-memory/plans/subtasks/fleet-system-and-ui-ux-bug-fixes/01-fleet-and-ssh-cluster.md`
- **Parent Spec:** `02-spec/21-app/fleet-system-and-ui-ux-bug-fixes/01-architecture-spec.md`
- **Target Release:** `v4.160.0`
- **Status:** READY FOR IMPLEMENTATION

---

## 1. Objectives

1. **Local Node Identity Resolution (Cluster-IP Primacy):**  
   Implement authoritative Cluster-IP primacy so that machine `192.168.1.3` deterministically resolves to node alias `w1` across `supabase_sync/node.rs`, `migration.rs`, `query.rs`, `commands/supabase.rs`, and `fleet_sync.rs`.
2. **GitMap Cluster Nodes Parity (6-Node Invariant):**  
   Ensure all 6 canonical cluster nodes (`w1`, `main`, `u1`, `w2`, `w3`, `w4`) are preserved and displayed in `FleetMachinesTable.tsx` and `useFleetMachines.ts`. Eliminate frontend filters dropping `is_local: true` and guarantee complete peer seeding in `query.rs`.
3. **SSH Cluster Reset Subsystem:**  
   Implement backend `reset_ssh_cluster_nodes()` in `ssh_manager/sync.rs`, expose Tauri command `reset_ssh_cluster` in `commands/ssh.rs` and `lib/run.rs`, wire through `sshService.ts`, and mount "Reset Cluster" action buttons with `RotateCcw` in `FleetMachinesTable.tsx` and `SshGitmapSection.tsx`.

---

## 2. Target Files

| Layer | File Path | Scope of Edit |
|:------|:----------|:--------------|
| **Rust Backend** | `src-tauri/src/modules/supabase_sync/node.rs` | Implement `resolve_local_node_alias()` with Cluster-IP primacy ladder |
| **Rust Backend** | `src-tauri/src/modules/supabase_sync/migration.rs` | Wire `resolve_local_node_alias()` into `build_local_fallback_machine()` |
| **Rust Backend** | `src-tauri/src/modules/supabase_sync/query.rs` | Seed all 6 `GITMAP_CLUSTER_NODES` in `collect_all_peer_connections()`, deduplicate without dropping `w1` |
| **Rust Backend** | `src-tauri/src/commands/supabase.rs` | Wire `resolve_local_node_alias()` into `get_local_node_info` command |
| **Rust Backend** | `src-tauri/src/modules/fleet_sync.rs` | Wire `resolve_local_node_alias()` into `generate_local_fleet_sync_payload()` |
| **Rust Backend** | `src-tauri/src/modules/ssh_manager/sync.rs` | Implement `reset_ssh_cluster_nodes()` resetting to 6 canonical nodes |
| **Rust Backend** | `src-tauri/src/commands/ssh.rs` | Expose `reset_ssh_cluster` command |
| **Rust Backend** | `src-tauri/src/lib/run.rs` | Register `commands::reset_ssh_cluster` in Tauri invoke handler |
| **TypeScript Frontend** | `src/services/sshService.ts` | Add `resetClusterNodes()` invoking `reset_ssh_cluster` |
| **TypeScript Frontend** | `src/components/instances/fleet/useFleetMachines.ts` | Preserve local node `w1` (`is_local: true`); do not filter out local node |
| **TypeScript Frontend** | `src/components/instances/FleetMachinesTable.tsx` | Mount "Reset Cluster" button with `RotateCcw` in header segmented pill capsule |
| **TypeScript Frontend** | `src/components/settings/EmailNotifications/SshGitmapSection.tsx` | Mount "Reset Cluster" button with `RotateCcw` in toolbar segmented pill capsule |

---

## 3. Step-by-Step Implementation Instructions

### Step 1: Implement `resolve_local_node_alias()` in `supabase_sync/node.rs`
- In `src-tauri/src/modules/supabase_sync/node.rs`:
  - Implement `pub fn resolve_local_node_alias() -> String`.
  - Follow the 5-stage ladder:
    1. Retrieve `local_ip = get_local_ip()`.
    2. If `local_ip` matches `GITMAP_CLUSTER_NODES` (`"192.168.1.3"` -> `"w1"`), return authoritative alias `"w1"`.
    3. Check `load_ssh_connections()` for IP match.
    4. Check `load_config()?.node_alias`.
    5. Fallback to `email_vault_db::get_notification_settings()?.local_machine_name` or `sysinfo::System::host_name()`.

### Step 2: Wire `resolve_local_node_alias()` into Migration, Query, and Commands
- In `src-tauri/src/modules/supabase_sync/migration.rs`:
  - In `build_local_fallback_machine()`, replace `let node_alias = config.node_alias;` with:
    ```rust
    let resolved_alias = resolve_local_node_alias();
    let node_alias = if !resolved_alias.is_empty() { resolved_alias } else { config.node_alias };
    ```
  - Assign `node_alias: node_alias.clone()`, `alias: node_alias`.
- In `src-tauri/src/modules/supabase_sync/query.rs`:
  - In `build_local_and_ssh_fleet()`, ensure local machine has `alias: resolve_local_node_alias()`.
  - In `query_fleet_machines()`, if a machine record is `is_local` or matches `get_local_ip()`, ensure its alias is backfilled with `resolve_local_node_alias()`.
- In `src-tauri/src/commands/supabase.rs`:
  - In `get_local_node_info()`, set:
    ```rust
    node_alias: supabase_sync::resolve_local_node_alias(),
    ```

### Step 3: Implement Cluster-IP Primacy in `fleet_sync.rs`
- In `src-tauri/src/modules/fleet_sync.rs`:
  - In `generate_local_fleet_sync_payload()`:
    - Replace `sysinfo::System::host_name().unwrap_or_else(...)` with `crate::modules::supabase_sync::resolve_local_node_alias()`.
    - Ensure generated `FleetNodeInfo` sends `node_alias: "w1"` when running on `192.168.1.3`.

### Step 4: Ensure all 6 `GITMAP_CLUSTER_NODES` in `query.rs`
- In `src-tauri/src/modules/supabase_sync/query.rs`:
  - In `collect_all_peer_connections()`:
    - Add Step 4: Iterate over `crate::modules::ssh_manager::GITMAP_CLUSTER_NODES`.
    - For each `(alias, ip, user, os)`, if neither `alias` nor `ip` is already present in `seen_aliases` / `seen_ips`:
      - Push canonical `SshConnectionRecord` with port 22, username, and OS.
  - In `build_local_and_ssh_fleet()`:
    - Insert local fallback machine (`w1`) as the first element (`is_local: true`).
    - In peer loop, skip any connection where `conn.ip_address == local_ip` or `conn.alias.eq_ignore_ascii_case(&local_alias)`.
    - Result: exactly 6 nodes (`w1` [Local] + `main`, `u1`, `w2`, `w3`, `w4`).

### Step 5: Implement `reset_ssh_cluster_nodes()` in `ssh_manager/sync.rs`
- In `src-tauri/src/modules/ssh_manager/sync.rs`:
  - Add function:
    ```rust
    pub fn reset_ssh_cluster_nodes() -> Result<NodeSyncStats, String>
    ```
  - Construct 6 `SshConnectionRecord` instances from `GITMAP_CLUSTER_NODES`.
  - Call `sync_ssh_connections_locally(&canonical_records)`.
  - Call `update_ssh_config(false)`.
  - Return `Ok(stats)`.

### Step 6: Expose Tauri Command & Register in `run.rs`
- In `src-tauri/src/commands/ssh.rs`:
  - Add:
    ```rust
    #[tauri::command]
    pub async fn reset_ssh_cluster() -> Result<NodeSyncStats, String> {
        tokio::task::spawn_blocking(ssh_manager::reset_ssh_cluster_nodes)
            .await
            .map_err(|e| format!("Task join error: {}", e))?
    }
    ```
- In `src-tauri/src/lib/run.rs`:
  - Add `commands::reset_ssh_cluster` to the `invoke_handler![...]` array.

### Step 7: Add `resetClusterNodes()` in `src/services/sshService.ts`
- In `src/services/sshService.ts`:
  - Export `resetClusterNodes(): Promise<NodeSyncStats>`:
    ```typescript
    async resetClusterNodes(): Promise<NodeSyncStats> {
        try {
            return await invoke<NodeSyncStats>('reset_ssh_cluster');
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'sshService.resetClusterNodes',
                triggerAction: 'reset_ssh_cluster',
            });
            throw err;
        }
    },
    ```

### Step 8: Update `useFleetMachines.ts` to Preserve Local Node
- In `src/components/instances/fleet/useFleetMachines.ts`:
  - In `fetchMachines()` and `init()`:
    - Remove the filter that excluded `is_local`:
      ```typescript
      // DO NOT filter out is_local!
      let filtered = data;
      ```
    - If `data.length === 0`, fallback to `mapSshNodesToFleet(sshNodes, effectiveLocalId)`.
    - In `mapSshNodesToFleet`: do not discard the local node; allow it to be mapped so 6 nodes are preserved.
    - Sort machines with local node pinned first: `(a, b) => (b.is_local ? 1 : 0) - (a.is_local ? 1 : 0)`.

### Step 9: Mount "Reset Cluster" in `FleetMachinesTable.tsx`
- In `src/components/instances/FleetMachinesTable.tsx`:
  - Import `RotateCcw` from `lucide-react`.
  - Add `isResettingCluster` state.
  - Implement `handleResetCluster`:
    - Calls `sshService.resetClusterNodes()`.
    - Re-fetches fleet machines via `handleManualRefresh()`.
  - Insert button in the segmented pill capsule toolbar (`divide-x`):
    - Styled with `text-slate-300 hover:text-white hover:bg-slate-700/50`.
    - Icon `RotateCcw className={cn("w-3.5 h-3.5 text-amber-400", isResettingCluster && "animate-spin")}`.
    - Title: `"Reset Cluster Nodes to Canonical Defaults"`.

### Step 10: Mount "Reset Cluster" in `SshGitmapSection.tsx`
- In `src/components/settings/EmailNotifications/SshGitmapSection.tsx`:
  - Import `RotateCcw` from `lucide-react`.
  - Add `isResettingCluster` state.
  - Implement `handleResetCluster`:
    - Calls `sshService.resetClusterNodes()`.
    - Displays toast with reset statistics.
    - Calls `loadAllSshData()` and `handleRefreshClusterBalances()`.
  - Mount button inside the contiguous segmented pill capsule toolbar alongside `Import SSH from GitMap`.

---

## 4. Verification & Quality Gates

1. **Rust Pre-flight Checks:**
   - Execute formatting gate: `cargo fmt -- --check`
   - Execute Clippy gate: `cargo clippy --all-targets --all-features`
   - Run targeted unit tests:
     - `cargo test --lib modules::supabase_sync::tests`
     - `cargo test --lib modules::ssh_manager::tests`
2. **Frontend Quality Gates:**
   - Run TypeScript / Vite build gate: `npm run build`
   - Verify zero JSX / CSS styling regression.
3. **Behavioral Invariant Audits:**
   - Verify `resolve_local_node_alias()` returns `"w1"` for IP `192.168.1.3`.
   - Verify `FleetMachinesTable` renders all 6 nodes (`w1`, `main`, `u1`, `w2`, `w3`, `w4`).
   - Verify clicking "Reset Cluster" restores missing nodes and invokes `reset_ssh_cluster`.
