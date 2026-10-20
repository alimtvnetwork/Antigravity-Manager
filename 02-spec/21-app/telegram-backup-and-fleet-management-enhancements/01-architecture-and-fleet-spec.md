# Architecture & Fleet Management Specification

**Task Slug**: `telegram-backup-and-fleet-management-enhancements`  
**Location**: `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/01-architecture-and-fleet-spec.md`  

---

## 1. Machine Identification & GitMap Primacy

### 1.1 Local Node Reality
1. `gitmap ip` reliably outputs the local machine IPv4 address: `192.168.1.3`.
2. In `gitmap cluster nodes`, `192.168.1.3:22` is assigned the canonical alias **`w1`**.
3. When Antigravity Manager is running on `192.168.1.3`, the current local machine is **`w1`**, NOT `main` (`192.168.1.20`).
4. Therefore, **`w1` is the current node and must be labeled as `Lead` and `Local`**, sorting to the top of the Shared Cluster / Fleet table.
5. `main` (`192.168.1.20`) is a remote worker machine enrolled in the SSH mesh.

### 1.2 Inverted Lead Logic Correction
In `src/components/instances/fleet/useFleetMachines.ts` and `FleetMachineRow.tsx`:
- Previously, `aliasLower === 'main'` was hardcoded as `is_lead: true`.
- Correction: The local machine (detected via `isLocalNode` or matching local IP `192.168.1.3` / alias `w1`) is assigned `is_lead: true` and `is_local: true`.
- `sortFleetMachines` must prioritize `is_local && is_lead` (`w1`) at the very top of the fleet hierarchy, followed by other online mesh nodes.

---

## 2. Fleet Caching & Performance Architecture

### 2.1 Problem: Stuck Skeleton Loading State
- On initial mount of the instances page or fleet tab, `isLoading` defaults to `true`.
- The UI renders `<FleetLoadingState />` (an empty container with 3 gray skeleton bars).
- Backend `get_cluster_instances_and_balances` queries 6 SSH nodes sequentially/in parallel with TCP liveness probes and SSH commands, taking 5 to 15 seconds. If any node times out, the table remains frozen on the skeleton.

### 2.2 Solution: 2-Tier Caching Pipeline
1. **Frontend Local Storage Cache**:
   - Cache key: `agm_fleet_machines_cache_v2`.
   - On initial mount of `useFleetMachines`, synchronously populate `machines` from `localStorage` if present, immediately setting `isLoading: false`. The user sees the fleet table in under 5 milliseconds with zero layout shift or stuck skeleton.
   - Background re-fetch is initiated silently (`isRefreshing: true`). When fresh data arrives, `machines` updates smoothly and writes back to `localStorage`.
2. **Backend Rust In-Memory TTL Cache**:
   - In `src-tauri/src/modules/ssh_manager/exec.rs`, wrap `get_cluster_instances_and_balances` with a 30-second `Mutex<Option<(Instant, Vec<ClusterInstanceBalanceItem>)>>` cache.
   - Concurrent or rapid IPC requests return cached cluster state immediately without launching redundant SSH sub-processes.
   - The "Reset Cluster" and "Refresh" buttons bypass this cache (`force_refresh: true`).

---

## 3. Cross-Machine Remote Prompt Dispatching

### 3.1 Requirement
Users must be able to dispatch prompts from the current machine to any remote machine's active instances directly within the Shared Cluster / Fleet Nodes table.

### 3.2 UI & Service Integration
1. In `src/components/instances/fleet/FleetMachineRow.tsx`:
   - Add an action column with an interactive `[ 💬 Send Prompt ]` button.
   - Clicking opens a compact remote prompt modal or inline drawer allowing the user to select the target instance (e.g. `main:default`, `v3`) and enter prompt text or choose from templates.
2. In `src/services/sshService.ts` & backend:
   - Route through `sshService.sendRemoteSshPrompt({ node_alias, instance_target, prompt_text })`.
   - Provide instant toast feedback upon dispatch.
